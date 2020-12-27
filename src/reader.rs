/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LittleEndian};
use encoding::{Encoding, all::UTF_8};
use flate2::read::ZlibDecoder;

use crate::{SCN_MDF_SIGNATURE, SCN_SIGNATURE, ScnError, ScnErrorKind, ScnFile, ScnRefTable, header::{MdfHeader, ScnHeader}, psb::PsbValue};

pub struct ScnReader;

impl ScnReader {
    
    /// Open scn file as ScnFile using stream
    pub fn open_scn_file<T: Read + Seek>(mut stream: T) -> Result<ScnFile<T>, ScnError> {
        let (entry_point, header, table) = Self::open_scn(&mut stream)?;

        ScnFile::new(header, table, entry_point, stream)
    }

    pub fn open_mdf_file<T: Read + Seek>(mut stream: T) -> Result<ScnFile<Cursor<Vec<u8>>>, ScnError> {
        let signature = stream.read_u32::<LittleEndian>()?;
        if signature != SCN_MDF_SIGNATURE {
            return Err(ScnError::new(ScnErrorKind::InvalidFile, None));
        }

        let (_, mdf_header) = MdfHeader::from_bytes(&mut stream)?;

        let mut compressed_buffer = Vec::new();

        stream.take(mdf_header.size as u64).read_to_end(&mut compressed_buffer)?;

        let mut decoder = ZlibDecoder::new(&compressed_buffer[..]);

        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);

        let (entry_point, header, table) = Self::open_scn(&mut cursor)?;

        ScnFile::new(header, table, entry_point, cursor)
    }

    /// Read entrypoint, header, scn table
    pub fn open_scn<T: Read + Seek>(stream: &mut T) -> Result<(u64, ScnHeader, ScnRefTable), ScnError> {
        let start = stream.seek(SeekFrom::Current(0)).unwrap();

        let signature = stream.read_u32::<LittleEndian>()?;
        if signature != SCN_SIGNATURE {
            return Err(ScnError::new(ScnErrorKind::InvalidFile, None));
        }

        let (_, header) = ScnHeader::from_bytes(stream)?;

        // Header(including offsets) size
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
        
                    _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))
        
                }?;
        
                stream.seek(SeekFrom::Start(start + extra_length_pos as u64))?;
                let (_, extra_lengths) = match PsbValue::from_bytes(stream)? {

                    (read, PsbValue::IntArray(array)) => Ok((read, array)),
        
                    _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))
        
                }?;

                if extra_offsets.len() < extra_lengths.len() {
                    return Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None));
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

        stream.seek(SeekFrom::Start(start + name_offset_pos as u64))?;
        let (_, charsets) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))

        }?;
        let (_, name_datas) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))

        }?;
        let (_, name_indexes) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))

        }?;

        stream.seek(SeekFrom::Start(start + strings_offset_pos as u64))?;
        let (_, string_offsets) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))

        }?;

        stream.seek(SeekFrom::Start(start + resource_offset_pos as u64))?;
        let (_, resource_offsets) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))

        }?;

        stream.seek(SeekFrom::Start(start + resource_length_pos as u64))?;
        let (_, resource_lengths) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None))

        }?;

        if resource_offsets.len() < resource_lengths.len() {
            return Err(ScnError::new(ScnErrorKind::InvalidOffsetTable, None));
        }

        // Names
        let charsets = charsets.unwrap();
        let name_datas = name_datas.unwrap();
        let name_indexes = name_indexes.unwrap();
        for index in name_indexes {
            let mut buffer = Vec::<u8>::new();
            
            let mut chr = name_datas[index as usize];

            while chr != 0 {
                let code = name_datas[chr as usize];

                let decoded = chr - charsets[code as usize];

                chr = code;

                buffer.push(decoded as u8);
            }

            buffer.reverse();
            let name = UTF_8.decode(&buffer, encoding::DecoderTrap::Replace).unwrap();

            names.push(name);
        }


        // Strings
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

        // Resources
        let resource_offsets = resource_offsets.unwrap();
        let resource_lengths = resource_lengths.unwrap();
        for i in 0..resource_offsets.len() {
            let mut buffer = Vec::new();

            stream.seek(SeekFrom::Start(start + resource_data_pos as u64 + resource_offsets[i]))?;
            stream.take(resource_lengths[i] as u64).read_to_end(&mut buffer)?;

            resources.push(buffer);
        }

        let table = ScnRefTable::new(names, strings, resources, extra);

        Ok((entry_point, header, table))
    }

}