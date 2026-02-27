pub mod binary_tree;
pub mod collection;
pub mod error;
pub mod number;
pub mod reference;
mod utill;

use collection::{PsbList, PsbObject, PsbUintArray};
use number::PsbNumber;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek};

use crate::value::{error::PsbValueReadError, reference::PsbString, utill::PsbValueStreamExt};

use self::reference::PsbResource;

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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PsbValue {
    None,
    Null,
    Bool(bool),
    Number(PsbNumber),
    IntArray(PsbUintArray),

    String(PsbString),

    List(PsbList),
    Object(PsbObject),

    Resource(PsbResource),
    ExtraResource(PsbResource),

    CompilerNumber,
    CompilerString,
    CompilerResource,
    CompilerDecimal,
    CompilerArray,
    CompilerBool,
    CompilerBinaryTree,
}

impl PsbValue {
    /// Non recursed types
    async fn read_io_non_recursion_typed(
        value_type: u8,
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<PsbValue, PsbValueReadError> {
        const PSB_TYPE_INTEGER_START: u8 = PSB_TYPE_INTEGER_N;
        const PSB_TYPE_INTEGER_MAX: u8 = PSB_TYPE_INTEGER_N + 8;
        const PSB_TYPE_INTEGER_ARRAY_START: u8 = PSB_TYPE_INTEGER_ARRAY_N + 1;
        const PSB_TYPE_INTEGER_ARRAY_MAX: u8 = PSB_TYPE_INTEGER_ARRAY_N + 8;
        const PSB_TYPE_RESOURCE_START: u8 = PSB_TYPE_RESOURCE_N + 1;
        const PSB_TYPE_RESOURCE_MAX: u8 = PSB_TYPE_RESOURCE_N + 4;
        const PSB_TYPE_STRING_START: u8 = PSB_TYPE_STRING_N + 1;
        const PSB_TYPE_STRING_MAX: u8 = PSB_TYPE_STRING_N + 4;
        const PSB_TYPE_EXTRA_START: u8 = PSB_TYPE_EXTRA_N + 1;
        const PSB_TYPE_EXTRA_MAX: u8 = PSB_TYPE_EXTRA_N + 8;

        match value_type {
            PSB_TYPE_NONE => Ok(PsbValue::None),
            PSB_TYPE_NULL => Ok(PsbValue::Null),

            PSB_TYPE_FALSE => Ok(PsbValue::Bool(false)),
            PSB_TYPE_TRUE => Ok(PsbValue::Bool(true)),

            PSB_TYPE_DOUBLE => Ok(PsbValue::Number(PsbNumber::Double(
                stream.read_f64_le().await?,
            ))),
            PSB_TYPE_FLOAT0 => Ok(PsbValue::Number(PsbNumber::Float(0.0))),
            PSB_TYPE_FLOAT => Ok(PsbValue::Number(PsbNumber::Float(
                stream.read_f32_le().await?,
            ))),

            value_type @ PSB_TYPE_INTEGER_START..=PSB_TYPE_INTEGER_MAX => {
                Ok(PsbValue::Number(PsbNumber::Integer(
                    stream
                        .read_partial_int(value_type - PSB_TYPE_INTEGER_N)
                        .await?,
                )))
            }

            value_type @ PSB_TYPE_INTEGER_ARRAY_START..=PSB_TYPE_INTEGER_ARRAY_MAX => {
                let len = stream
                    .read_partial_uint(value_type - PSB_TYPE_INTEGER_ARRAY_N)
                    .await?;
                Ok(PsbValue::IntArray(
                    PsbUintArray::read_io(stream, len as usize).await?,
                ))
            }

            value_type @ PSB_TYPE_RESOURCE_START..=PSB_TYPE_RESOURCE_MAX => {
                Ok(PsbValue::Resource(PsbResource(
                    stream
                        .read_partial_uint(value_type - PSB_TYPE_RESOURCE_N)
                        .await?,
                )))
            }

            value_type @ PSB_TYPE_STRING_START..=PSB_TYPE_STRING_MAX => {
                Ok(PsbValue::String(PsbString(
                    stream
                        .read_partial_uint(value_type - PSB_TYPE_STRING_N)
                        .await?,
                )))
            }

            value_type @ PSB_TYPE_EXTRA_START..=PSB_TYPE_EXTRA_MAX => {
                Ok(PsbValue::ExtraResource(PsbResource(
                    stream
                        .read_partial_uint(value_type - PSB_TYPE_EXTRA_N)
                        .await?,
                )))
            }

            PSB_COMPILER_INTEGER => Ok(PsbValue::CompilerNumber),
            PSB_COMPILER_STRING => Ok(PsbValue::CompilerString),
            PSB_COMPILER_RESOURCE => Ok(PsbValue::CompilerResource),
            PSB_COMPILER_ARRAY => Ok(PsbValue::CompilerArray),
            PSB_COMPILER_BOOL => Ok(PsbValue::CompilerBool),
            PSB_COMPILER_BINARY_TREE => Ok(PsbValue::CompilerBinaryTree),

            value_type => Err(PsbValueReadError::InvalidValueType(value_type)),
        }
    }

    /// Types requires recursion
    async fn read_io_typed(
        value_type: u8,
        stream: &mut (impl AsyncRead + AsyncSeek + Unpin),
    ) -> Result<PsbValue, PsbValueReadError> {
        match value_type {
            PSB_TYPE_LIST => Ok(PsbValue::List(PsbList::read_io(stream).await?)),
            PSB_TYPE_OBJECT => Ok(PsbValue::Object(PsbObject::read_io(stream).await?)),

            value_type => Self::read_io_non_recursion_typed(value_type, stream).await,
        }
    }

    pub async fn read_io(
        stream: &mut (impl AsyncRead + AsyncSeek + Unpin),
    ) -> Result<PsbValue, PsbValueReadError> {
        Self::read_io_typed(stream.read_u8().await?, stream).await
    }

    pub(crate) async fn read_io_non_recursion(
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<PsbValue, PsbValueReadError> {
        Self::read_io_non_recursion_typed(stream.read_u8().await?, stream).await
    }

    // pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
    //     match self {
    //         PsbValue::None => {
    //             stream.write_u8(PSB_TYPE_NONE)?;
    //             Ok(1)
    //         }
    //         PsbValue::Null => {
    //             stream.write_u8(PSB_TYPE_NULL)?;
    //             Ok(1)
    //         }
    //         PsbValue::Bool(value) => {
    //             if *value {
    //                 stream.write_u8(PSB_TYPE_TRUE)?;
    //             } else {
    //                 stream.write_u8(PSB_TYPE_FALSE)?;
    //             }

    //             Ok(1)
    //         }
    //         PsbValue::Number(number) => {
    //             match number {
    //                 PsbNumber::Integer(integer) => {
    //                     let n = if *integer == 0 {
    //                         0
    //                     } else {
    //                         PsbNumber::get_n(*integer)
    //                     };

    //                     stream.write_u8(PSB_TYPE_INTEGER_N + n)?;
    //                 }

    //                 PsbNumber::Double(_) => {
    //                     stream.write_u8(PSB_TYPE_DOUBLE)?;
    //                 }

    //                 PsbNumber::Float(float) => {
    //                     if *float == 0_f32 {
    //                         stream.write_u8(PSB_TYPE_FLOAT0)?;
    //                     } else {
    //                         stream.write_u8(PSB_TYPE_FLOAT)?;
    //                     }
    //                 }
    //             }

    //             Ok(1 + number.write_bytes(stream)?)
    //         }

    //         PsbValue::IntArray(array) => {
    //             stream.write_u8(PSB_TYPE_INTEGER_ARRAY_N + array.get_n())?;

    //             Ok(1 + array.write_bytes(stream)?)
    //         }

    //         PsbValue::Resource(res) => {
    //             stream.write_u8(PSB_TYPE_RESOURCE_N + res.get_n())?;

    //             Ok(1 + res.write_bytes(stream)?)
    //         }
    //         PsbValue::ExtraResource(res) => {
    //             stream.write_u8(PSB_TYPE_EXTRA_N + res.get_n())?;

    //             Ok(1 + res.write_bytes(stream)?)
    //         }

    //         PsbValue::CompilerNumber => {
    //             stream.write_u8(PSB_COMPILER_INTEGER)?;
    //             Ok(1)
    //         }
    //         PsbValue::CompilerString => {
    //             stream.write_u8(PSB_COMPILER_STRING)?;
    //             Ok(1)
    //         }
    //         PsbValue::CompilerResource => {
    //             stream.write_u8(PSB_COMPILER_RESOURCE)?;
    //             Ok(1)
    //         }
    //         PsbValue::CompilerDecimal => {
    //             stream.write_u8(PSB_COMPILER_DECIMAL)?;
    //             Ok(1)
    //         }
    //         PsbValue::CompilerArray => {
    //             stream.write_u8(PSB_COMPILER_ARRAY)?;
    //             Ok(1)
    //         }
    //         PsbValue::CompilerBool => {
    //             stream.write_u8(PSB_COMPILER_BOOL)?;
    //             Ok(1)
    //         }
    //         PsbValue::CompilerBinaryTree => {
    //             stream.write_u8(PSB_COMPILER_BINARY_TREE)?;
    //             Ok(1)
    //         }

    //         _ => Err(PsbError::new(PsbErrorKind::InvalidPSBValue, None)),
    //     }
    // }

    // pub fn write_bytes_refs(
    //     &self,
    //     stream: &mut impl Write,
    //     table: &PsbRefs,
    // ) -> Result<u64, PsbError> {
    //     match &self {
    //         PsbValue::String(string) => {
    //             let n = PsbNumber::get_uint_n(match table.find_string_index(string.string()) {
    //                 Some(ref_index) => Ok(ref_index),

    //                 None => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None)),
    //             }?);

    //             stream.write_u8(PSB_TYPE_STRING_N + n)?;

    //             Ok(1 + string.write_bytes(stream, table)?)
    //         }

    //         PsbValue::List(list) => {
    //             stream.write_u8(PSB_TYPE_LIST)?;

    //             Ok(1 + list.write_bytes(stream, table)?)
    //         }

    //         PsbValue::Object(object) => {
    //             stream.write_u8(PSB_TYPE_OBJECT)?;

    //             Ok(1 + object.write_bytes(stream, table)?)
    //         }

    //         _ => self.write_bytes(stream),
    //     }
    // }
}
