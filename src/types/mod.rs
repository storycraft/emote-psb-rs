/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod collection;
pub mod number;
pub mod reference;
pub mod binary_tree;
pub mod string;

use std::io::{Read, Seek, Write};

use collection::{PsbUintArray, PsbList, PsbObject};
use number::PsbNumber;

use crate::{PsbError, PsbErrorKind, PsbRefs};
use byteorder::{ReadBytesExt, WriteBytesExt};

use self::{reference::PsbReference, string::PsbString};

pub const PSB_TYPE_NONE: u8 = 0x00;

pub const PSB_TYPE_NULL: u8 = 0x01;

pub const PSB_TYPE_FALSE: u8 = 0x02;
pub const PSB_TYPE_TRUE: u8 = 0x03;

/// 0 <= N <= 8
pub const PSB_TYPE_INTEGER_N: u8 = 0x04;
pub const PSB_TYPE_FLOAT0: u8 = 0x1d;
pub const PSB_TYPE_FLOAT: u8 = 0x1e;
pub const PSB_TYPE_DOUBLE: u8 = 0x1f;

/// 1 <= N <= 8
pub const PSB_TYPE_INTEGER_ARRAY_N: u8 = 0x0C;

/// 1 <= N <= 4
pub const PSB_TYPE_STRING_N: u8 = 0x14;

/// 1 <= N <= 4
pub const PSB_TYPE_RESOURCE_N: u8 = 0x18;

pub const PSB_TYPE_LIST: u8 = 0x20;
pub const PSB_TYPE_OBJECT: u8 = 0x21;

/// 1 <= N <= 8
pub const PSB_TYPE_EXTRA_N: u8 = 0x21;

pub const PSB_COMPILER_INTEGER: u8 = 0x80;
pub const PSB_COMPILER_STRING: u8 = 0x81;
pub const PSB_COMPILER_RESOURCE: u8 = 0x82;
pub const PSB_COMPILER_DECIMAL: u8 = 0x83;
pub const PSB_COMPILER_ARRAY: u8 = 0x84;
pub const PSB_COMPILER_BOOL: u8 = 0x85;
pub const PSB_COMPILER_BINARY_TREE: u8 = 0x86;

#[derive(Debug, PartialEq)]
pub enum PsbValue {

    None, Null,
    Bool(bool),
    Number(PsbNumber),
    IntArray(PsbUintArray),

    String(PsbString),
    StringRef(PsbReference),

    List(PsbList),
    Object(PsbObject),

    Resource(PsbReference),
    ExtraResource(PsbReference),

    CompilerNumber,
    CompilerString,
    CompilerResource,
    CompilerDecimal,
    CompilerArray,
    CompilerBool,
    CompilerBinaryTree

}

impl PsbValue {

    fn from_bytes_type<T: Read + Seek>(value_type: u8, stream: &mut T) -> Result<(u64, PsbValue), PsbError> {
        match value_type {
            PSB_TYPE_NONE => Ok((1, PsbValue::None)),
            PSB_TYPE_NULL => Ok((1, PsbValue::Null)),

            PSB_TYPE_FALSE => Ok((1, PsbValue::Bool(false))),
            PSB_TYPE_TRUE => Ok((1, PsbValue::Bool(true))),
            
            PSB_TYPE_DOUBLE => {
                let (read, val) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(val)))
            },

            PSB_TYPE_FLOAT0 => {
                let (read, val) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(val)))
            },

            PSB_TYPE_FLOAT => {
                let (read, val) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(val)))
            },

            _ if value_type > PSB_TYPE_STRING_N && value_type <= PSB_TYPE_STRING_N + 4 => {
                let (read, string_ref) = PsbReference::from_bytes(value_type - PSB_TYPE_STRING_N, stream)?;

                Ok((read + 1, PsbValue::StringRef(string_ref)))
            },

            _ if value_type >= PSB_TYPE_INTEGER_N && value_type <= PSB_TYPE_INTEGER_N + 8 => {
                let (read, number) = PsbNumber::from_bytes(value_type, stream)?;
                Ok((read + 1, PsbValue::Number(number)))
            },

            _ if value_type > PSB_TYPE_INTEGER_ARRAY_N && value_type <= PSB_TYPE_INTEGER_ARRAY_N + 8 => {
                let (read, array) = PsbUintArray::from_bytes(value_type - PSB_TYPE_INTEGER_ARRAY_N, stream)?;
                Ok((read + 1, PsbValue::IntArray(array)))
            },

            _ if value_type > PSB_TYPE_RESOURCE_N && value_type <= PSB_TYPE_RESOURCE_N + 4 => {
                let (read, map) = PsbReference::from_bytes(value_type - PSB_TYPE_RESOURCE_N, stream)?;

                Ok((read + 1, PsbValue::Resource(map)))
            },

            _ if value_type > PSB_TYPE_EXTRA_N && value_type <= PSB_TYPE_EXTRA_N + 4 => {
                let (read, map) = PsbReference::from_bytes(value_type - PSB_TYPE_EXTRA_N, stream)?;

                Ok((read + 1, PsbValue::ExtraResource(map)))
            },

            PSB_COMPILER_INTEGER => Ok((1, PsbValue::CompilerNumber)),
            PSB_COMPILER_STRING => Ok((1, PsbValue::CompilerString)),
            PSB_COMPILER_RESOURCE => Ok((1, PsbValue::CompilerResource)),
            PSB_COMPILER_ARRAY => Ok((1, PsbValue::CompilerArray)),
            PSB_COMPILER_BOOL => Ok((1, PsbValue::CompilerBool)),
            PSB_COMPILER_BINARY_TREE => Ok((1, PsbValue::CompilerBinaryTree)),

            _ => {
                Err(PsbError::new(PsbErrorKind::InvalidPSBValue, None))
            }
        }
    }

    pub fn from_bytes<T: Read + Seek>(stream: &mut T) -> Result<(u64, PsbValue), PsbError> {
        Self::from_bytes_type(stream.read_u8()?, stream)
    }

    pub fn from_bytes_refs<T: Read + Seek>(stream: &mut T, table: &PsbRefs) -> Result<(u64, PsbValue), PsbError> {
        let value_type = stream.read_u8()?;

        match value_type {

            PSB_TYPE_LIST => {
                let (read, list) = PsbList::from_bytes(stream, table)?;

                Ok((read + 1, PsbValue::List(list)))
            },

            PSB_TYPE_OBJECT => {
                let (read, map) = PsbObject::from_bytes(stream, table)?;

                Ok((read + 1, PsbValue::Object(map)))
            },

            _ => {
                Self::from_bytes_type(value_type, stream)
            }

        }
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        match self {
            PsbValue::None => {
                stream.write_u8(PSB_TYPE_NONE)?;
                Ok(1)
            },
            PsbValue::Null => {
                stream.write_u8(PSB_TYPE_NULL)?;
                Ok(1)
            },
            PsbValue::Bool(value) => {
                if *value {
                    stream.write_u8(PSB_TYPE_TRUE)?;
                } else {
                    stream.write_u8(PSB_TYPE_FALSE)?;
                }

                Ok(1)
            },
            PsbValue::Number(number) => {
                match number {
                    PsbNumber::Integer(integer) => {
                        let n = PsbNumber::get_n(*integer);
                        stream.write_u8(PSB_TYPE_INTEGER_N + n)?;
                    },

                    PsbNumber::Double(_) => {
                        stream.write_u8(PSB_TYPE_DOUBLE)?;
                    },

                    PsbNumber::Float(float) => {
                        if *float == 0_f32 {
                            stream.write_u8(PSB_TYPE_FLOAT0)?;
                        } else {
                            stream.write_u8(PSB_TYPE_FLOAT)?;
                        }
                    }
                }

                Ok(1 + number.write_bytes(stream)?)
            },

            PsbValue::IntArray(array) => {
                stream.write_u8(PSB_TYPE_INTEGER_ARRAY_N + array.get_n())?;

                Ok(1 + array.write_bytes(stream)?)
            },

            PsbValue::StringRef(string_ref) => {
                stream.write_u8(PSB_TYPE_STRING_N + string_ref.get_n())?;

                Ok(1 + string_ref.write_bytes(stream)?)
            },

            PsbValue::Resource(res) => {
                stream.write_u8(PSB_TYPE_RESOURCE_N + res.get_n())?;

                Ok(1 + res.write_bytes(stream)?)
            },
            PsbValue::ExtraResource(res) => {
                stream.write_u8(PSB_TYPE_EXTRA_N + res.get_n())?;

                Ok(1 + res.write_bytes(stream)?)
            },

            PsbValue::CompilerNumber => {
                stream.write_u8(PSB_COMPILER_INTEGER)?;
                Ok(1)
            },
            PsbValue::CompilerString => {
                stream.write_u8(PSB_COMPILER_STRING)?;
                Ok(1)
            },
            PsbValue::CompilerResource => {
                stream.write_u8(PSB_COMPILER_RESOURCE)?;
                Ok(1)
            },
            PsbValue::CompilerDecimal => {
                stream.write_u8(PSB_COMPILER_DECIMAL)?;
                Ok(1)
            },
            PsbValue::CompilerArray => {
                stream.write_u8(PSB_COMPILER_ARRAY)?;
                Ok(1)
            },
            PsbValue::CompilerBool => {
                stream.write_u8(PSB_COMPILER_BOOL)?;
                Ok(1)
            },
            PsbValue::CompilerBinaryTree => {
                stream.write_u8(PSB_COMPILER_BINARY_TREE)?;
                Ok(1)
            },

            _ => {
                Err(PsbError::new(PsbErrorKind::InvalidPSBValue, None))
            }
        }
    }

    pub fn write_bytes_refs(&self, stream: &mut impl Write, table: &PsbRefs) -> Result<u64, PsbError> {
        match &self {

            PsbValue::String(string) => {
                let n = PsbNumber::get_uint_n(match table.find_string_index(string.string()) {

                    Some(ref_index) => {
                        Ok(ref_index)
                    },
        
                    None => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
                }?).max(1);
                
                stream.write_u8(PSB_TYPE_STRING_N + n)?;

                Ok(1 + string.write_bytes(stream, table)?)
            },

            PsbValue::List(list) => {
                stream.write_u8(PSB_TYPE_LIST)?;

                Ok(1 + list.write_bytes(stream, table)?)
            },

            PsbValue::Object(object) => {
                stream.write_u8(PSB_TYPE_OBJECT)?;

                Ok(1 + object.write_bytes(stream, table)?)
            },

            _ => self.write_bytes(stream)

        }
    }

}