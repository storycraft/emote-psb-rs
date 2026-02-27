use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io::{self, BufRead, Read, Seek, SeekFrom};

use async_compression::tokio::bufread::ZlibDecoder;
use encoding::{all::UTF_8, Encoding};
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader, ReadBuf, Take};

use crate::{
    header::PsbHeader,
    offsets::PsbOffsets,
    reader::error::{MdfOpenError, PsbOpenError},
    value::{binary_tree::PsbBinaryTree, PsbValue},
    PsbError, PsbErrorKind, PsbRefs, PSB_MDF_SIGNATURE, PSB_SIGNATURE,
};

pub struct PsbFile {
    pub version: u16,
    pub encrypted: bool,
}

impl PsbFile {
    pub fn open_psb<T: Read + Seek>(mut stream: T) -> Result<Self, PsbError> {
        let start = stream.stream_position().unwrap();

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
        let (_, strings) =
            Self::read_strings(offsets.strings.data_pos + start as u32, &mut stream)?;

        let refs = PsbRefs::new(names, strings);

        Ok(PsbFile::new(header, refs, offsets, stream))
    }

    pub fn read_names<T: Read + Seek>(stream: &mut T) -> Result<(u64, Vec<String>), PsbError> {
        let mut names = Vec::<String>::new();

        let (read, btree) = PsbBinaryTree::from_bytes(stream)?;

        for raw_string in btree.unwrap() {
            let name = UTF_8
                .decode(&raw_string, encoding::DecoderTrap::Replace)
                .unwrap();

            names.push(name);
        }

        Ok((read, names))
    }

    pub fn read_strings<T: Read + Seek>(
        data_pos: u32,
        stream: &mut T,
    ) -> Result<(u64, Vec<String>), PsbError> {
        let mut strings = Vec::<String>::new();

        let (offsets_read, string_offsets) = match PsbValue::from_bytes(stream)? {
            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None)),
        }?;

        let mut reader = BufReader::new(stream.by_ref());
        let string_offsets = string_offsets.unwrap();

        let mut read = 0_usize;
        for offset in string_offsets {
            let mut buffer = Vec::new();

            reader.seek(SeekFrom::Start(data_pos as u64 + offset))?;
            read += reader.read_until(0x00, &mut buffer)?;

            // Decode excluding nul
            let string = UTF_8
                .decode(&buffer[..buffer.len() - 1], encoding::DecoderTrap::Replace)
                .unwrap();
            strings.push(string);
        }

        Ok((offsets_read + read as u64, strings))
    }
}
