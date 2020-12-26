/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::ScnError;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// SCN file header
#[derive(Debug)]
pub struct ScnHeader {

    /// Version. (1, 2, 3, 4)
    pub version: u16,
    /// != 0 if encrypted
    pub encryption: u16

}

impl ScnHeader {

    /// Read header from current position.
    /// Returns read size, ScnHeader tuple.
    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), ScnError> {
        let version = stream.read_u16::<LittleEndian>()?;
        let encryption = stream.read_u16::<LittleEndian>()?;

        Ok((4, Self {
            version,
            encryption
        }))
    }

    /// Write scn header to stream.
    /// Returns written size.
    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {
        stream.write_u16::<LittleEndian>(self.version)?;
        stream.write_u16::<LittleEndian>(self.encryption)?;
        
        Ok(8)
    }

}

/// MDF (compressed scn) file header
pub struct MdfHeader {

    /// Compressed size
    pub size: u32

}

impl MdfHeader {

    /// Read header from current position.
    /// Returns read size, MdfHeader tuple.
    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), ScnError> {
        Ok((4, Self { size: stream.read_u32::<LittleEndian>()? }))
    }

    /// Write scn header to stream.
    /// Returns written size.
    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {
        stream.write_u32::<LittleEndian>(self.size)?;
        
        Ok(4)
    }

}