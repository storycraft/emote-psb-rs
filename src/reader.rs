/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LittleEndian};
use encoding::{Encoding, all::UTF_8};
use flate2::read::ZlibDecoder;

use crate::{PSB_MDF_SIGNATURE, PSB_SIGNATURE, PsbError, PsbErrorKind, PsbFile, PsbRefs, header::{MdfHeader, PsbHeader}, offsets::PsbOffsets, types::{PsbValue, binary_tree::PsbBinaryTree}};

pub struct PsbReader;

impl PsbReader {

    pub fn open_mdf<T: Read + Seek>(mut stream: T) -> Result<PsbFile<Cursor<Vec<u8>>>, PsbError> {
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

        Self::open_psb(Cursor::new(buffer))
    }

    /// Read as PsbFile
    pub fn open_psb<T: Read + Seek>(mut stream: T) -> Result<PsbFile<T>, PsbError> {
        let start = stream.seek(SeekFrom::Current(0)).unwrap();

        let signature = stream.read_u32::<LittleEndian>()?;
        if signature != PSB_SIGNATURE {
            return Err(PsbError::new(PsbErrorKind::InvalidFile, None));
        }

        let (_, header) = PsbHeader::from_bytes(&mut stream)?;

        let _ = stream.read_u32::<LittleEndian>()?;

        // offsets
        let (_, offsets) = PsbOffsets::from_bytes(header.version, &mut stream)?;

        stream.seek(SeekFrom::Start(start + offsets.name_offset as u64))?;
        let (_, names) = Self::read_names(&mut stream)?;

        stream.seek(SeekFrom::Start(start + offsets.strings.offset_pos as u64))?;
        let (_, strings) = Self::read_strings(offsets.strings.data_pos + start as u32, &mut stream)?;

        let refs = PsbRefs::new(names, strings);

        Ok(
            PsbFile::new(
                header,
                refs,
                offsets,
                stream
            )
        )
    }

    pub fn read_names<T: Read + Seek>(stream: &mut T) -> Result<(u64, Vec<String>), PsbError> {
        let mut names = Vec::<String>::new();

        let (read, btree) = PsbBinaryTree::from_bytes(stream)?;

        for raw_string in btree.unwrap() {
            let name = UTF_8.decode(&raw_string, encoding::DecoderTrap::Replace).unwrap();
            
            names.push(name);
        }

        Ok((read, names))
    }

    pub fn read_strings<T: Read + Seek>(data_pos: u32, stream: &mut T) -> Result<(u64, Vec<String>), PsbError> {
        let mut strings = Vec::<String>::new();

        let (offsets_read, string_offsets) = match PsbValue::from_bytes(stream)? {
    
            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;

        let mut reader = BufReader::new(stream.by_ref());
        let string_offsets = string_offsets.unwrap();

        let mut read = 0_usize;
        for offset in string_offsets {
            let mut buffer = Vec::new();

            reader.seek(SeekFrom::Start(data_pos as u64 + offset as u64))?;
            read += reader.read_until(0x00, &mut buffer)?;

            // Decode excluding nul
            let string = UTF_8.decode(&buffer[..buffer.len() - 1], encoding::DecoderTrap::Replace).unwrap();
            strings.push(string);
        }

        Ok((offsets_read + read as u64, strings))
    }

}