/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::{PsbError, PsbErrorKind};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::{PSB_TYPE_DOUBLE, PSB_TYPE_FLOAT, PSB_TYPE_INTEGER_N, PSB_TYPE_FLOAT0};

#[derive(Debug, Clone, PartialEq)]
pub enum PsbNumber {

    Integer(i64),
    Double(f64),
    Float(f32)

}

impl PsbNumber {

    pub fn from_bytes(number_type: u8, stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        match number_type {

            PSB_TYPE_DOUBLE => {
                Ok((8, Self::Double(stream.read_f64::<LittleEndian>()?)))
            },

            PSB_TYPE_FLOAT => {
                Ok((4, Self::Float(stream.read_f32::<LittleEndian>()?)))
            },

            PSB_TYPE_FLOAT0 => {
                Ok((0, Self::Float(0_f32)))
            },

            _ if number_type >= PSB_TYPE_INTEGER_N && number_type <= PSB_TYPE_INTEGER_N + 8 => {
                let number_size = number_type - PSB_TYPE_INTEGER_N;

                let (read, val) = Self::read_integer(number_size, stream)?;

                Ok((read, Self::Integer(val)))
            }

            _ => {
                Err(PsbError::new(PsbErrorKind::InvalidPSBValue, None))
            }

        }
    }

    /// Read integer with given size.
    pub fn read_integer(number_size: u8, stream: &mut impl Read) -> Result<(u64, i64), PsbError> {
        if number_size == 0 {
            Ok((1, 0))
        } else if number_size <= 8 {
            let mut buf = [0_u8; 8];

            stream.read_exact(&mut buf[..number_size as usize])?;

            Ok((number_size as u64, i64::from_le_bytes(buf)))
        } else {
            Err(PsbError::new(PsbErrorKind::InvalidPSBValue, None))
        }
    }

    pub fn get_n(number: u64) -> u8 {
        if number <= 0 {
            0
        } else if number <= 0xff {
            1
        } else if number <= 0xffff {
            2
        } else if number <= 0xffffff {
            3
        } else if number <= 0xffffffff {
            4
        } else if number <= 0xffffffffff {
            5
        } else if number <= 0xffffffffffff {
            6
        } else if number <= 0xffffffffffffff {
            7
        } else {
            8
        }
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        match self {
            PsbNumber::Integer(val) => {
                let n = Self::get_n(*val as u64);

                Self::write_integer(n, *val, stream)?;
                Ok(n as u64)
            },

            PsbNumber::Double(val) => {
                stream.write_f64::<LittleEndian>(*val)?;
                Ok(8)
            },

            PsbNumber::Float(val) => {
                if *val == 0f32 {
                    Ok(0)
                } else {
                    stream.write_f32::<LittleEndian>(*val)?;

                    Ok(4)
                }
            }

        }
    }

    pub fn write_integer(n: u8, number: i64, stream: &mut impl Write) -> Result<u8, PsbError> {
        if n > 0 {
            stream.write_all(&number.to_le_bytes()[..n as usize])?;

            return Ok(n);
        }

        Ok(0)
    }

}

impl From<i8> for PsbNumber {
    fn from(number: i8) -> Self {
        PsbNumber::Integer(number as i64)
    }
}

impl From<i16> for PsbNumber {
    fn from(number: i16) -> Self {
        PsbNumber::Integer(number as i64)
    }
}

impl From<i32> for PsbNumber {
    fn from(number: i32) -> Self {
        PsbNumber::Integer(number as i64)
    }
}

impl From<i64> for PsbNumber {
    fn from(number: i64) -> Self {
        PsbNumber::Integer(number)
    }
}

impl From<f32> for PsbNumber {
    fn from(number: f32) -> Self {
        PsbNumber::Float(number)
    }
}

impl From<f64> for PsbNumber {
    fn from(number: f64) -> Self {
        PsbNumber::Double(number)
    }
}