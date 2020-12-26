/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::{ScnError, ScnErrorKind};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::{PSB_TYPE_DOUBLE, PSB_TYPE_FLOAT, PSB_TYPE_INTEGER, PSB_TYPE_FLOAT0};

#[derive(Debug)]
pub enum PsbNumber {

    Integer(u64),
    Double(f64),
    Float(f32)

}

impl PsbNumber {

    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), ScnError> {
        let number_type = stream.read_u8()?;

        match number_type {

            PSB_TYPE_DOUBLE => {
                Ok((9, Self::Double(stream.read_f64::<LittleEndian>()?)))
            },

            PSB_TYPE_FLOAT => {
                Ok((5, Self::Float(stream.read_f32::<LittleEndian>()?)))
            },

            PSB_TYPE_FLOAT0 => {
                Ok((1, Self::Float(0_f32)))
            },

            _ if number_type >= PSB_TYPE_INTEGER && number_type < PSB_TYPE_INTEGER + 8 => {
                let number_size = number_type - PSB_TYPE_INTEGER;
                
                let (read, val) = Self::read_integer(number_size, stream)?;

                Ok((1 + read, Self::Integer(val)))
            }

            _ => {
                Err(ScnError::new(ScnErrorKind::InvalidPSBValue, None))
            }

        }
    }

    /// Read integer with given size.
    pub fn read_integer(number_size: u8, stream: &mut impl Read) -> Result<(u64, u64), ScnError> {
        if number_size == 0 {
            Ok((1, 0))
        } else if number_size <= 8 {
            let mut buf = [0_u8; 8];

            stream.read_exact(&mut buf[..number_size as usize])?;

            Ok((1 + number_size as u64, u64::from_le_bytes(buf)))
        } else {
            Err(ScnError::new(ScnErrorKind::InvalidPSBValue, None))
        }
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {
        match self {

            PsbNumber::Integer(val) => {
                if *val <= 0 {
                    stream.write_u8(PSB_TYPE_INTEGER)?;
                    Ok(1)
                } else if *val <= 0xff {
                    stream.write_u8(PSB_TYPE_INTEGER + 1)?;
                    stream.write_all(&val.to_le_bytes()[..1])?;
                    Ok(2)
                } else if *val <= 0xffff {
                    stream.write_u8(PSB_TYPE_INTEGER + 2)?;
                    stream.write_all(&val.to_le_bytes()[..2])?;
                    Ok(3)
                } else if *val <= 0xffffff {
                    stream.write_u8(PSB_TYPE_INTEGER + 3)?;
                    stream.write_all(&val.to_le_bytes()[..3])?;
                    Ok(4)
                } else if *val <= 0xffffffff {
                    stream.write_u8(PSB_TYPE_INTEGER + 4)?;
                    stream.write_all(&val.to_le_bytes()[..4])?;
                    Ok(5)
                } else if *val <= 0xffffffffff {
                    stream.write_u8(PSB_TYPE_INTEGER + 5)?;
                    stream.write_all(&val.to_le_bytes()[..5])?;
                    Ok(6)
                } else if *val <= 0xffffffffffff {
                    stream.write_u8(PSB_TYPE_INTEGER + 6)?;
                    stream.write_all(&val.to_le_bytes()[..6])?;
                    Ok(7)
                } else if *val <= 0xffffffffffffff {
                    stream.write_u8(PSB_TYPE_INTEGER + 7)?;
                    stream.write_all(&val.to_le_bytes()[..7])?;
                    Ok(8)
                } else {
                    stream.write_u8(PSB_TYPE_INTEGER + 8)?;
                    stream.write_all(&val.to_le_bytes()[..8])?;
                    Ok(9)
                }
            },

            PsbNumber::Double(val) => {
                stream.write_u8(PSB_TYPE_DOUBLE)?;
                stream.write_f64::<LittleEndian>(*val)?;
                Ok(9)
            },

            PsbNumber::Float(val) => {
                if *val == 0f32 {
                    stream.write_u8(PSB_TYPE_FLOAT0)?;
                    Ok(1)
                } else {
                    stream.write_u8(PSB_TYPE_FLOAT)?;
                    stream.write_f32::<LittleEndian>(*val)?;

                    Ok(5)
                }
            }

        }
    }

}