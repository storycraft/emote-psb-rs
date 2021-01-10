/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::PsbError;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// PSB file header
#[derive(Debug, Clone, Copy)]
pub struct PsbHeader {

    /// Version. (1, 2, 3, 4)
    pub version: u16,
    /// != 0 if encrypted
    pub encryption: u16

}

impl PsbHeader {

    /// Read header from current position.
    /// Returns read size, ScnHeader tuple.
    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        let version = stream.read_u16::<LittleEndian>()?;
        let encryption = stream.read_u16::<LittleEndian>()?;

        Ok((4, Self {
            version,
            encryption
        }))
    }

    /// Write scn header to stream.
    /// Returns written size.
    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        stream.write_u16::<LittleEndian>(self.version)?;
        stream.write_u16::<LittleEndian>(self.encryption)?;
        
        Ok(8)
    }

}

/// MDF (compressed psb) file header
pub struct MdfHeader {

    /// Compressed size
    pub size: u32

}

impl MdfHeader {

    /// Read header from current position.
    /// Returns read size, MdfHeader tuple.
    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        Ok((4, Self { size: stream.read_u32::<LittleEndian>()? }))
    }

    /// Write mdf header to stream.
    /// Returns written size.
    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        stream.write_u32::<LittleEndian>(self.size)?;
        
        Ok(4)
    }

}