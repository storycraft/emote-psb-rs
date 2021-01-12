/*
 * Created on Tue Jan 12 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::{PsbError, PsbErrorKind};

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

#[derive(Debug, Clone, Copy)]
pub struct PsbOffsets {

    pub name_offset: u32,
    pub strings: PsbStringOffset,

    pub resources: PsbResourcesOffset,

    pub entry_point: u32,

    pub checksum: Option<u32>,
    pub extra: Option<PsbResourcesOffset>

}

impl PsbOffsets {
    
    pub fn from_bytes(version: u16, stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        let name_offset = stream.read_u32::<LittleEndian>()?;
        let (strings_read, strings) = PsbStringOffset::from_bytes(stream)?;
        let (resources_read, resources) = PsbResourcesOffset::from_bytes(stream)?;

        let entry_point = stream.read_u32::<LittleEndian>()?;

        let (checksum_read, checksum) = if version > 2 {
            (4, Some(stream.read_u32::<LittleEndian>()?))
        } else {
            (0, None)
        };

        let (extra_read, extra) = if version > 3 {
            let (read, extra) = PsbResourcesOffset::from_bytes(stream)?;
            (read, Some(extra))
        } else {
            (0, None)
        };

        Ok((8 + strings_read + resources_read + checksum_read + extra_read, Self {
            name_offset,
            strings,
            resources,
            entry_point,
            checksum,
            extra
        }))
    }

    pub fn write_bytes(&self, version: u16, stream: &mut impl Write) -> Result<u64, PsbError> {
        stream.write_u32::<LittleEndian>(self.name_offset)?;
        let strings_written = self.strings.write_bytes(stream)?;
        let resources_written = self.resources.write_bytes(stream)?;
        stream.write_u32::<LittleEndian>(self.entry_point)?;
        
        let checksum_written: u64;
        let extra_written: u64;
        if version > 2 {
            stream.write_u32::<LittleEndian>(self.checksum.unwrap_or(0))?;
            checksum_written = 4;
            
            if version > 3 {
                if self.extra.is_none() {
                    return Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None));
                }

                extra_written = self.extra.unwrap().write_bytes(stream)?;
            } else {
                extra_written = 0;
            }
        } else {
            checksum_written = 0;
            extra_written = 0;
        }

        Ok(8 + strings_written + resources_written + checksum_written + extra_written)
    }

}

impl Default for PsbOffsets {
    fn default() -> Self {
        Self {
            name_offset: 0,
            strings: PsbStringOffset::default(),
            resources: PsbResourcesOffset::default(),
            entry_point: 0,
            checksum: Some(0),
            extra: Some(PsbResourcesOffset::default())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsbResourcesOffset {

    pub offset_pos: u32,
    pub lengths_pos: u32,
    pub data_pos: u32

}

impl PsbResourcesOffset {
    
    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        Ok((12, Self {
            offset_pos: stream.read_u32::<LittleEndian>()?,
            lengths_pos: stream.read_u32::<LittleEndian>()?,
            data_pos: stream.read_u32::<LittleEndian>()?,
        }))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        stream.write_u32::<LittleEndian>(self.offset_pos)?;
        stream.write_u32::<LittleEndian>(self.lengths_pos)?;
        stream.write_u32::<LittleEndian>(self.data_pos)?;

        Ok(12)
    }

}

impl Default for PsbResourcesOffset {
    fn default() -> Self {
        Self {
            offset_pos: 0,
            lengths_pos: 0,
            data_pos: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsbStringOffset {

    pub offset_pos: u32,
    pub data_pos: u32

}

impl PsbStringOffset {
    
    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        Ok((8, Self {
            offset_pos: stream.read_u32::<LittleEndian>()?,
            data_pos: stream.read_u32::<LittleEndian>()?,
        }))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        stream.write_u32::<LittleEndian>(self.offset_pos)?;
        stream.write_u32::<LittleEndian>(self.data_pos)?;

        Ok(8)
    }

}

impl Default for PsbStringOffset {
    fn default() -> Self {
        Self {
            offset_pos: 0,
            data_pos: 0
        }
    }
}