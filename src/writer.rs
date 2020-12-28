/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Seek, SeekFrom, Write};

use adler::Adler32;
use byteorder::{LittleEndian, WriteBytesExt};

use crate::{PSB_SIGNATURE, PsbError, PsbRefTable, header::PsbHeader, types::{PsbValue, collection::PsbIntArray}};

pub struct PsbWriter<T: Write + Seek> {

    pub header: PsbHeader,

    pub ref_table: PsbRefTable,

    pub entry: PsbValue,

    stream: T

}

impl<T: Write + Seek> PsbWriter<T> {

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

        self.stream.write_u32::<LittleEndian>(PSB_SIGNATURE)?;
        self.header.write_bytes(&mut self.stream)?;
        
        let header_length = match self.header.version {
            version if version < 3 => 28,
            version if version == 3 => 32,
            
            _ => 44
        };

        self.stream.write_u32::<LittleEndian>(header_length)?;

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
        
        let name_offset_pos: u32;
        let string_offset_pos: u32;
        let string_data_pos: u32;
        let resource_offset_pos: u32;
        let resource_lengths_pos: u32;
        let resource_data_pos: u32;
        let entry_point_pos: u32;

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
            let mut index_list = Vec::<u64>::new();

            string_data_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            for string in self.ref_table.strings() {
                let bytes = string.as_bytes();

                let current_pos = self.stream.seek(SeekFrom::Current(0)).unwrap();

                index_list.push(current_pos - string_data_pos as u64);
                self.stream.write_all(bytes)?;
                self.stream.write_u8(0)?;
            }

            string_offset_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::from(index_list).write_bytes(&mut self.stream)?;
        }

        // Root Entry
        {
            entry_point_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            self.entry.write_bytes(&mut self.stream)?;
        }

        // Resources
        {
            let mut index_list = Vec::<u64>::new();
            let mut length_list = Vec::<u64>::new();

            resource_data_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            for res in self.ref_table.resources() {
                let current_pos = self.stream.seek(SeekFrom::Current(0)).unwrap();

                index_list.push(current_pos - resource_data_pos as u64);
                length_list.push(res.len() as u64);

                self.stream.write_all(res)?;
            }

            resource_offset_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::from(index_list).write_bytes(&mut self.stream)?;

            resource_lengths_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::from(length_list).write_bytes(&mut self.stream)?;
        }

        // Extra resources support from 4
        if self.header.version > 3 {
            let mut index_list = Vec::<u64>::new();
            let mut length_list = Vec::<u64>::new();

            extra_data_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            for res in self.ref_table.resources() {
                let current_pos = self.stream.seek(SeekFrom::Current(0)).unwrap();

                index_list.push(current_pos - extra_data_pos as u64);
                length_list.push(res.len() as u64);

                self.stream.write_all(res)?;
            }

            extra_offset_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::from(index_list).write_bytes(&mut self.stream)?;

            extra_lengths_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() as u32;
            PsbIntArray::from(length_list).write_bytes(&mut self.stream)?;
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
            let mut adler = Adler32::new();

            adler.write_slice(&header_length.to_le_bytes());
            adler.write_slice(&name_offset_pos.to_le_bytes());
            adler.write_slice(&string_offset_pos.to_le_bytes());
            adler.write_slice(&string_data_pos.to_le_bytes());
            adler.write_slice(&resource_offset_pos.to_le_bytes());
            adler.write_slice(&resource_lengths_pos.to_le_bytes());
            adler.write_slice(&resource_data_pos.to_le_bytes());
            adler.write_slice(&entry_point_pos.to_le_bytes());

            self.stream.write_u32::<LittleEndian>(adler.checksum())?;

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