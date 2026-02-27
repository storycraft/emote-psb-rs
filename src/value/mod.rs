pub mod binary_tree;
pub mod collection;
pub mod io;
pub mod number;
mod utill;

use collection::{PsbList, PsbObject, PsbUintArray};
use number::PsbNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PsbPrimitive {
    None,
    Null,
    Bool(bool),
    Number(PsbNumber),

    String(u32),
    Resource(u32),
    ExtraResource(u64),

    CompilerNumber,
    CompilerString,
    CompilerResource,
    CompilerDecimal,
    CompilerArray,
    CompilerBool,
    CompilerBinaryTree,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PsbValue {
    Primitive(PsbPrimitive),
    IntArray(PsbUintArray),
    List(PsbList),
    Object(PsbObject),
}

impl PsbValue {
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
