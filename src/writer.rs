/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom, Write};

use adler::Adler32;
use byteorder::{LittleEndian, WriteBytesExt};
use flate2::{Compression, bufread::ZlibEncoder};

use crate::{PSB_MDF_SIGNATURE, PSB_SIGNATURE, PsbError, PsbRefs, VirtualPsb, header::MdfHeader, offsets::{PsbOffsets, PsbResourcesOffset, PsbStringOffset}, types::{PsbValue, binary_tree::PsbBinaryTree, collection::PsbUintArray}};

pub struct PsbWriter<T> {

    pub psb: VirtualPsb,

    stream: T

}

impl<T: Write> PsbWriter<T> {

    pub fn write_names(names: &Vec<String>, stream: &mut T) -> Result<u64, PsbError> {
        let mut buffer_list = Vec::<Vec<u8>>::new();

        for name in names.iter() {
            buffer_list.push(name.as_bytes().into());
        }

        PsbBinaryTree::from(buffer_list).write_bytes(stream)
    }

}

impl<T: Write + Seek> PsbWriter<T> {

    pub fn new(
        psb: VirtualPsb,
        stream: T
    ) -> Self {
        Self {
            psb,
            stream
        }
    }

    /// Write file and finish stream
    pub fn finish(mut self) -> Result<u64, PsbError> {
        let file_start = self.stream.seek(SeekFrom::Current(0)).unwrap();

        let (header, resources, extra, root) = self.psb.unwrap();

        self.stream.write_u32::<LittleEndian>(PSB_SIGNATURE)?;
        header.write_bytes(&mut self.stream)?;

        let offsets_end_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start;
        self.stream.write_u32::<LittleEndian>(0)?;

        // Offsets
        let offset_start_pos = self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start;
        let mut offsets = PsbOffsets::default();

        // Offsets prefill
        offsets.write_bytes(header.version, &mut self.stream)?;

        let offsets_end = self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start;

        let refs = {
            let mut names = Vec::new();
            let mut strings = Vec::new();

            root.collect_names(&mut names);
            root.collect_strings(&mut strings);

            names.sort();
            strings.sort();

            PsbRefs::new(names, strings)
        };

        // Names
        {
            offsets.name_offset = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            Self::write_names(refs.names(), &mut self.stream)?;
        }

        // Root Entry
        {
            offsets.entry_point = (self.stream.seek(SeekFrom::Current(0)).unwrap() - file_start) as u32;
            PsbValue::Object(root).write_bytes_refs(&mut self.stream, &refs)?;
        }

        // Strings
        {
            let (_, strings) = Self::write_strings(refs.strings(), &mut self.stream)?;

            offsets.strings = strings;
        }

        // Resources
        {
            let (_, res_offsets) = Self::write_resources(&resources, &mut self.stream)?;
            offsets.resources = res_offsets;
        }

        // Extra resources support from 4
        if header.version > 3 {
            let (_, extra_offsets) = Self::write_resources(&extra, &mut self.stream)?;
            offsets.extra = Some(extra_offsets);
        }

        // Rewrite entries
        let file_end = self.stream.seek(SeekFrom::Current(0)).unwrap();

        self.stream.seek(SeekFrom::Start(offsets_end_pos))?;
        self.stream.write_u32::<LittleEndian>(offsets_end as u32)?;

        if header.version > 2 {
            let mut adler = Adler32::new();

            adler.write_slice(&(offset_start_pos as u32).to_le_bytes());
            adler.write_slice(&offsets.name_offset.to_le_bytes());
            adler.write_slice(&offsets.strings.offset_pos.to_le_bytes());
            adler.write_slice(&offsets.strings.data_pos.to_le_bytes());
            adler.write_slice(&offsets.resources.offset_pos.to_le_bytes());
            adler.write_slice(&offsets.resources.lengths_pos.to_le_bytes());
            adler.write_slice(&offsets.resources.data_pos.to_le_bytes());
            adler.write_slice(&offsets.entry_point.to_le_bytes());
            
            offsets.checksum = Some(adler.checksum());
        }

        self.stream.seek(SeekFrom::Start(offset_start_pos))?;
        offsets.write_bytes(header.version, &mut self.stream)?;

        self.stream.seek(SeekFrom::Start(file_end))?;

        Ok(file_end - file_start)
    }

    /// Write resources. Returns written size, PsbResourcesOffset tuple
    pub fn write_resources(resources: &Vec<Vec<u8>>, stream: &mut T) -> Result<(u64, PsbResourcesOffset), PsbError> {
        let mut offset_list = Vec::<u64>::new();
        let mut length_list = Vec::<u64>::new();

        let mut total_len = 0_u64;
        for res in resources.iter() {
            let len = res.len() as u64;

            offset_list.push(total_len);
            length_list.push(len);

            total_len += len;
        }

        let offset_pos = (stream.seek(SeekFrom::Current(0)).unwrap()) as u32;
        let offsets_written = PsbValue::IntArray(PsbUintArray::from(offset_list)).write_bytes(stream)?;

        let lengths_pos = (stream.seek(SeekFrom::Current(0)).unwrap()) as u32;
        let lengths_written = PsbValue::IntArray(PsbUintArray::from(length_list)).write_bytes(stream)?;

        let data_pos = (stream.seek(SeekFrom::Current(0)).unwrap()) as u32;
        let mut data_written = 0_u64;
        for res in resources.iter() {
            data_written += res.len() as u64;
            stream.write_all(res)?;
        }

        Ok((offsets_written + lengths_written + data_written, PsbResourcesOffset {
            offset_pos,
            lengths_pos,
            data_pos
        }))
    }

    /// Write strings. Returns written size, PsbStringOffset tuple
    pub fn write_strings(strings: &Vec<String>, stream: &mut T) -> Result<(u64, PsbStringOffset), PsbError> {
        let mut offset_list = Vec::<u64>::new();

        let mut total_len = 0_u64;
        for string in strings.iter() {
            let len = string.as_bytes().len() as u64;
            
            offset_list.push(total_len);

            total_len += len + 1;
        }

        let offset_pos = stream.seek(SeekFrom::Current(0)).unwrap() as u32;
        let offset_written = PsbValue::IntArray(PsbUintArray::from(offset_list)).write_bytes(stream)?;

        let data_pos = stream.seek(SeekFrom::Current(0)).unwrap() as u32;
        for string in strings.iter() {
            stream.write_all(string.as_bytes())?;
            stream.write_u8(0)?;
        }

        Ok((offset_written + total_len as u64, PsbStringOffset {
            offset_pos,
            data_pos
        }))
    }

}

pub struct MdfWriter<R, W> {

    read: R,
    stream: W

}

impl<R: Read, W: Write + Seek> MdfWriter<R, W> {

    pub fn new(read: R, stream: W) -> Self {
        Self {
            read,
            stream
        }
    }

    /// Write mdf file.
    /// Returns written size
    pub fn finish(mut self) -> Result<u64, PsbError> {
        let mut reader = BufReader::new(self.read);

        let mut encoder = ZlibEncoder::new(&mut reader, Compression::best());

        // Write signature first
        self.stream.write_u32::<LittleEndian>(PSB_MDF_SIGNATURE)?;

        let header_pos = self.stream.seek(SeekFrom::Current(0)).unwrap();
        // Prefill header
        MdfHeader { size: 0 }.write_bytes(&mut self.stream)?;
        
        io::copy(&mut encoder, &mut self.stream)?;
        let total_out = encoder.total_out();

        let end_pos = self.stream.seek(SeekFrom::Current(0)).unwrap();

        // Fill header
        self.stream.seek(SeekFrom::Start(header_pos)).unwrap();
        MdfHeader { size: total_out as u32 }.write_bytes(&mut self.stream)?;

        self.stream.seek(SeekFrom::Start(end_pos)).unwrap();
        Ok(total_out + 8)
    }

}

pub struct PsbMdfWriter<T> {

    buffer: Cursor<Vec<u8>>,
    psb: VirtualPsb,
    stream: T

}

impl<T: Write + Seek> PsbMdfWriter<T> {

    pub fn new(psb: VirtualPsb, stream: T) -> Self {
        Self {
            buffer: Default::default(),
            psb,
            stream
        }
    }

    /// Write mdf file.
    /// Returns written size
    pub fn finish(mut self) -> Result<u64, PsbError> {
        let psb_writer = PsbWriter::new(self.psb, &mut self.buffer);
        psb_writer.finish()?;

        let mdf_writer = MdfWriter::new(self.buffer, self.stream);

        mdf_writer.finish()
    }
}