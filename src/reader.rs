/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LittleEndian};
use encoding::{Encoding, all::UTF_8};
use flate2::read::ZlibDecoder;

use crate::{PsbError, PsbErrorKind, PsbFile, PsbRefTable, PSB_MDF_SIGNATURE, PSB_SIGNATURE, header::{MdfHeader, PsbHeader}, types::{PsbValue, binary_tree::BinaryTree}};

pub struct PsbReader;

impl PsbReader {
    
    /// Open psb file as PsbFile using stream
    pub fn open_psb_file<T: Read + Seek>(mut stream: T) -> Result<PsbFile<T>, PsbError> {
        let (entry_point, header, table) = Self::open_psb(&mut stream)?;

        PsbFile::new(header, table, entry_point, stream)
    }

    pub fn open_mdf_file<T: Read + Seek>(mut stream: T) -> Result<PsbFile<Cursor<Vec<u8>>>, PsbError> {
        let signature = stream.read_u32::<LittleEndian>()?;
        if signature != PSB_MDF_SIGNATURE {
            return Err(PsbError::new(PsbErrorKind::InvalidFile, None));
        }

        let (_, mdf_header) = MdfHeader::from_bytes(&mut stream)?;

        let mut compressed_buffer = Vec::new();

        stream.take(mdf_header.size as u64).read_to_end(&mut compressed_buffer)?;

        let mut decoder = ZlibDecoder::new(&compressed_buffer[..]);

        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);

        let (entry_point, header, table) = Self::open_psb(&mut cursor)?;

        PsbFile::new(header, table, entry_point, cursor)
    }

    /// Read entrypoint, header, scn table
    pub fn open_psb<T: Read + Seek>(stream: &mut T) -> Result<(u64, PsbHeader, PsbRefTable), PsbError> {
        let start = stream.seek(SeekFrom::Current(0)).unwrap();

        let signature = stream.read_u32::<LittleEndian>()?;
        if signature != PSB_SIGNATURE {
            return Err(PsbError::new(PsbErrorKind::InvalidFile, None));
        }

        let (_, header) = PsbHeader::from_bytes(stream)?;

        let _ = stream.read_u32::<LittleEndian>()?;

        // Name offset pos
        let name_offset_pos = stream.read_u32::<LittleEndian>()?;
        let strings_offset_pos = stream.read_u32::<LittleEndian>()?;
        let strings_data_pos = stream.read_u32::<LittleEndian>()?;
        let resource_offset_pos = stream.read_u32::<LittleEndian>()?;
        let resource_length_pos = stream.read_u32::<LittleEndian>()?;
        let resource_data_pos = stream.read_u32::<LittleEndian>()?;
        let entry_point = start + stream.read_u32::<LittleEndian>()? as u64;

        let mut names = Vec::<String>::new();
        let mut strings = Vec::<String>::new();
        let mut resources = Vec::<Vec<u8>>::new();
        let mut extra = Vec::<Vec<u8>>::new();

        let _header_checksum: Option<u32>;
        if header.version > 2 {
            // Adler32
            _header_checksum = Some(stream.read_u32::<LittleEndian>()?);

            if header.version > 3 {
                let extra_offset_pos = stream.read_u32::<LittleEndian>()?;
                let extra_length_pos = stream.read_u32::<LittleEndian>()?;
                let extra_data_pos = stream.read_u32::<LittleEndian>()?;

                stream.seek(SeekFrom::Start(start + extra_offset_pos as u64))?;
                let (_, extra_offsets) = match PsbValue::from_bytes(stream)? {

                    (read, PsbValue::IntArray(array)) => Ok((read, array)),
        
                    _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
        
                }?;
        
                stream.seek(SeekFrom::Start(start + extra_length_pos as u64))?;
                let (_, extra_lengths) = match PsbValue::from_bytes(stream)? {

                    (read, PsbValue::IntArray(array)) => Ok((read, array)),
        
                    _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
        
                }?;

                if extra_offsets.len() < extra_lengths.len() {
                    return Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None));
                }

                // Extra
                let extra_offsets = extra_offsets.unwrap();
                let extra_lengths = extra_lengths.unwrap();
                for i in 0..extra_offsets.len() {
                    let mut buffer = Vec::new();
        
                    stream.seek(SeekFrom::Start(start + extra_data_pos as u64 + extra_offsets[i]))?;
                    stream.take(extra_lengths[i] as u64).read_to_end(&mut buffer)?;
        
                    extra.push(buffer);
                }
            }
        } else {
            _header_checksum = None;
        }

        // Names
        {
            stream.seek(SeekFrom::Start(start + name_offset_pos as u64))?;
            let (_, btree) = BinaryTree::from_bytes(stream)?;

            for raw_string in btree.unwrap() {
                let name = UTF_8.decode(&raw_string, encoding::DecoderTrap::Replace).unwrap();
                names.push(name);
            }
        }

    
        // Strings
        {
            stream.seek(SeekFrom::Start(start + strings_offset_pos as u64))?;
            let (_, string_offsets) = match PsbValue::from_bytes(stream)? {
    
                (read, PsbValue::IntArray(array)) => Ok((read, array)),
    
                _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
    
            }?;

            let mut reader = BufReader::new(stream.by_ref());
            let string_offsets = string_offsets.unwrap();
            for offset in string_offsets {
                let mut buffer = Vec::new();
    
                reader.seek(SeekFrom::Start(start + strings_data_pos as u64 + offset))?;
                reader.read_until(0x00, &mut buffer)?;
    
                // Decode excluding nul
                let string = UTF_8.decode(&buffer[..buffer.len() - 1], encoding::DecoderTrap::Replace).unwrap();
    
                strings.push(string);
            }
        }

        // Resources
        {
            stream.seek(SeekFrom::Start(start + resource_offset_pos as u64))?;
            let (_, resource_offsets) = match PsbValue::from_bytes(stream)? {
    
                (read, PsbValue::IntArray(array)) => Ok((read, array)),
    
                _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
    
            }?;
    
            stream.seek(SeekFrom::Start(start + resource_length_pos as u64))?;
            let (_, resource_lengths) = match PsbValue::from_bytes(stream)? {
    
                (read, PsbValue::IntArray(array)) => Ok((read, array)),
    
                _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
    
            }?;
    
            if resource_offsets.len() < resource_lengths.len() {
                return Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None));
            }

            let resource_offsets = resource_offsets.unwrap();
            let resource_lengths = resource_lengths.unwrap();
            for i in 0..resource_offsets.len() {
                let mut buffer = Vec::new();
    
                stream.seek(SeekFrom::Start(start + resource_data_pos as u64 + resource_offsets[i]))?;
                stream.take(resource_lengths[i] as u64).read_to_end(&mut buffer)?;
    
                resources.push(buffer);
            }
        }

        let table = PsbRefTable::new(names, strings, resources, extra);

        Ok((entry_point, header, table))
    }

}