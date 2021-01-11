/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Seek, SeekFrom, Write};

use adler::Adler32;
use byteorder::{LittleEndian, WriteBytesExt};

use crate::{PSB_SIGNATURE, PsbError, PsbRefTable, header::PsbHeader, types::{PsbValue, binary_tree::PsbBinaryTree, collection::PsbIntArray}};

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

        let offsets_end_pos_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start;
        self.stream.write_u32::<LittleEndian>(0)?;

        // Offsets
        let offset_start_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start;
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
        let offsets_end_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start;
        
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

        // Names
        {
            let mut buffer_list = Vec::<Vec<u8>>::new();

            for name in self.ref_table.names() {
                buffer_list.push(name.as_bytes().into());
            }

            name_offset_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            PsbBinaryTree::from(buffer_list).write_bytes(&mut self.stream)?;
        }

        // Root Entry
        {
            entry_point_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            self.entry.write_bytes(&mut self.stream)?;
        }

        // Strings
        {
            let mut offset_list = Vec::<u64>::new();

            let mut total_len = 0_u64;
            for string in self.ref_table.strings().iter() {
                let len = string.as_bytes().len() as u64;
                
                offset_list.push(total_len);

                total_len += len + 1;
            }

            string_offset_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            PsbValue::IntArray(PsbIntArray::from(offset_list)).write_bytes(&mut self.stream)?;

            string_data_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            for string in self.ref_table.strings().iter() {
                self.stream.write_all(string.as_bytes())?;
                self.stream.write_u8(0)?;
            }
        }

        // Resources
        {
            let mut offset_list = Vec::<u64>::new();
            let mut length_list = Vec::<u64>::new();

            let mut total_len = 0_u64;
            for res in self.ref_table.resources().iter() {
                let len = res.len() as u64;

                offset_list.push(total_len);
                length_list.push(len);

                total_len += len;
            }

            resource_offset_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            PsbValue::IntArray(PsbIntArray::from(offset_list)).write_bytes(&mut self.stream)?;

            resource_lengths_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            PsbValue::IntArray(PsbIntArray::from(length_list)).write_bytes(&mut self.stream)?;

            resource_data_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            for res in self.ref_table.resources().iter() {
                self.stream.write_all(res)?;
            }
        }

        // Extra resources support from 4
        if self.header.version > 3 {
            let mut offset_list = Vec::<u64>::new();
            let mut length_list = Vec::<u64>::new();

            let mut total_len = 0_u64;
            for res in self.ref_table.extra().iter() {
                let len = res.len() as u64;

                offset_list.push(total_len);
                length_list.push(len);

                total_len += len;
            }

            extra_offset_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            PsbValue::IntArray(PsbIntArray::from(offset_list)).write_bytes(&mut self.stream)?;

            extra_lengths_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            PsbValue::IntArray(PsbIntArray::from(length_list)).write_bytes(&mut self.stream)?;

            extra_data_pos = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            for res in self.ref_table.extra().iter() {
                self.stream.write_all(res)?;
            }
        }

        // Rewrite entries
        let file_end = self.stream.seek(SeekFrom::Current(0)).unwrap();

        self.stream.seek(SeekFrom::Start(offsets_end_pos_pos))?;
        self.stream.write_u32::<LittleEndian>(offsets_end_pos as u32)?;

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

            adler.write_slice(&(offset_start_pos as u32).to_le_bytes());
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