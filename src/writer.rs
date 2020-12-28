/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{SCN_SIGNATURE, PsbError, PsbRefTable, header::PsbHeader, types::{PsbValue, collection::PsbIntArray}};

pub struct ScnWriter<T: Write + Seek> {

    pub header: PsbHeader,

    pub ref_table: PsbRefTable,

    pub entry: PsbValue,

    stream: T

}

impl<T: Write + Seek> ScnWriter<T> {

    pub fn new(
        header: PsbHeader,
        ref_table: PsbRefTable,
        entry: PsbValue,
        stream: T
    ) -> Self {
        Self {
            header,
            ref_table,
            entry,
            stream
        }
    }

    /// Write file and finish stream
    pub fn finish(mut self) -> Result<(u64, T), PsbError> {
        let file_start = self.stream.seek(SeekFrom::Current(0)).unwrap();

        self.stream.write_u32::<LittleEndian>(SCN_SIGNATURE)?;
        self.header.write_bytes(&mut self.stream)?;

        // Offsets
        let offset_start_pos = self.stream.seek(SeekFrom::Current(0))?;
        for _ in 0..7 {
            // Offsets prefill
            self.stream.write_u32::<LittleEndian>(0)?;
        }

        if self.header.version > 2 {
            // Checksum prefill
            self.stream.write_u32::<LittleEndian>(0)?;

            if self.header.version > 3 {
                // Extra prefill
                self.stream.write_u32::<LittleEndian>(0)?;
                self.stream.write_u32::<LittleEndian>(0)?;
                self.stream.write_u32::<LittleEndian>(0)?;
            }
        }
        
        let mut name_offset_pos = 0_u32;
        let mut string_offset_pos = 0_u32;
        let mut string_data_pos = 0_u32;
        let mut resource_offset_pos = 0_u32;
        let mut resource_lengths_pos = 0_u32;
        let mut resource_data_pos = 0_u32;
        let mut entry_point_pos = 0_u32;

        let mut extra_offset_pos = 0_u32;
        let mut extra_lengths_pos = 0_u32;
        let mut extra_data_pos = 0_u32;

        // TODO:
        // Names
        {
            name_offset_pos = 0;
        }
        
        // Strings
        {
            let mut string_buffer = Vec::<u8>::new();
            let mut index_list = Vec::<u64>::new();

            for string in self.ref_table.strings() {
                let bytes = string.as_bytes();

                index_list.push(string_buffer.len() as u64);
                string_buffer.write_all(bytes)?;
                string_buffer.write_u8(0)?;
            }

            string_offset_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::new(index_list).write_bytes(&mut self.stream)?;

            string_data_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            self.stream.write_all(&string_buffer)?;
        }

        // Root Entry
        {
            entry_point_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            self.entry.write_bytes(&mut self.stream)?;
        }

        // Resources
        {
            let mut resource_buffer = Vec::<u8>::new();
            let mut index_list = Vec::<u64>::new();
            let mut length_list = Vec::<u64>::new();

            for res in self.ref_table.resources() {
                index_list.push(resource_buffer.len() as u64);
                length_list.push(res.len() as u64);

                resource_buffer.write_all(res)?;
            }

            resource_offset_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::new(index_list).write_bytes(&mut self.stream)?;

            resource_lengths_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::new(length_list).write_bytes(&mut self.stream)?;

            resource_data_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            self.stream.write_all(&resource_buffer)?;
        }

        // Extra resources support from 4
        if self.header.version > 3 {
            let mut resource_buffer = Vec::<u8>::new();
            let mut index_list = Vec::<u64>::new();
            let mut length_list = Vec::<u64>::new();

            for res in self.ref_table.extra() {
                index_list.push(resource_buffer.len() as u64);
                length_list.push(res.len() as u64);

                resource_buffer.write_all(res)?;
            }

            extra_offset_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::new(index_list).write_bytes(&mut self.stream)?;

            extra_lengths_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::new(length_list).write_bytes(&mut self.stream)?;

            extra_data_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            self.stream.write_all(&resource_buffer)?;
        }

        // Rewrite entries
        let file_end = self.stream.seek(SeekFrom::Current(0)).unwrap();

        self.stream.seek(SeekFrom::Start(offset_start_pos))?;

        self.stream.write_u32::<LittleEndian>(name_offset_pos)?;
        self.stream.write_u32::<LittleEndian>(string_offset_pos)?;
        self.stream.write_u32::<LittleEndian>(string_data_pos)?;
        self.stream.write_u32::<LittleEndian>(resource_offset_pos)?;
        self.stream.write_u32::<LittleEndian>(resource_lengths_pos)?;
        self.stream.write_u32::<LittleEndian>(resource_data_pos)?;
        self.stream.write_u32::<LittleEndian>(entry_point_pos)?;
        if self.header.version > 2 {
            // TODO: Checksum
            self.stream.write_u32::<LittleEndian>(0)?;

            if self.header.version > 3 {
                self.stream.write_u32::<LittleEndian>(extra_offset_pos)?;
                self.stream.write_u32::<LittleEndian>(extra_lengths_pos)?;
                self.stream.write_u32::<LittleEndian>(extra_data_pos)?;
            }
        }

        self.stream.seek(SeekFrom::Start(file_end))?;

        Ok((file_end - file_start, self.stream))
    }

}