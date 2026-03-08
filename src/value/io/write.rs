use std::io::{self, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::value::{
    PsbPrimitive,
    io::{
        PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
        PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
        PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0,
        PSB_TYPE_INTEGER_ARRAY_N, PSB_TYPE_INTEGER_N, PSB_TYPE_NONE, PSB_TYPE_NULL,
        PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE, error::PsbValueWriteError,
    },
    number::PsbNumber,
    util::{get_n, get_uint_n, write_partial_uint},
};

#[derive(Debug)]
pub struct PsbStreamValueWriter<T> {
    stream: T,
}

impl<T: Write> PsbStreamValueWriter<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub fn write_primitive(&mut self, primitive: PsbPrimitive) -> io::Result<()> {
        match primitive {
            PsbPrimitive::None => {
                self.stream.write_u8(PSB_TYPE_NONE)?;
            }
            PsbPrimitive::Null => {
                self.stream.write_u8(PSB_TYPE_NULL)?;
            }
            PsbPrimitive::Bool(value) => {
                if value {
                    self.stream.write_u8(PSB_TYPE_TRUE)?;
                } else {
                    self.stream.write_u8(PSB_TYPE_FALSE)?;
                }
            }
            PsbPrimitive::Number(number) => match number {
                PsbNumber::Integer(v) => {
                    let n = get_n(v);
                    self.stream.write_u8(PSB_TYPE_INTEGER_N + n)?;
                    self.stream.write_all(&v.to_le_bytes()[..n as usize])?;
                }

                PsbNumber::Double(v) => {
                    self.stream.write_u8(PSB_TYPE_DOUBLE)?;
                    self.stream.write_f64::<LittleEndian>(v)?;
                }

                PsbNumber::Float(v) => {
                    if v == 0.0 {
                        self.stream.write_u8(PSB_TYPE_FLOAT0)?;
                    } else {
                        self.stream.write_u8(PSB_TYPE_FLOAT)?;
                        self.stream.write_f32::<LittleEndian>(v)?;
                    }
                }
            },
            PsbPrimitive::Resource(index) => {
                let n = get_uint_n(index as _);
                self.stream.write_u8(PSB_TYPE_RESOURCE_N + n)?;
                self.stream.write_all(&index.to_le_bytes()[..n as usize])?;
            }
            PsbPrimitive::String(index) => {
                let n = get_uint_n(index as _);
                self.stream.write_u8(PSB_TYPE_STRING_N + n)?;
                self.stream.write_all(&index.to_le_bytes()[..n as usize])?;
            }
            PsbPrimitive::ExtraResource(index) => {
                let n = get_uint_n(index as _);
                self.stream.write_u8(PSB_TYPE_EXTRA_N + n)?;
                self.stream.write_all(&index.to_le_bytes()[..n as usize])?;
            }
            PsbPrimitive::CompilerNumber => {
                self.stream.write_u8(PSB_COMPILER_INTEGER)?;
            }
            PsbPrimitive::CompilerString => {
                self.stream.write_u8(PSB_COMPILER_STRING)?;
            }
            PsbPrimitive::CompilerResource => {
                self.stream.write_u8(PSB_COMPILER_RESOURCE)?;
            }
            PsbPrimitive::CompilerDecimal => {
                self.stream.write_u8(PSB_COMPILER_DECIMAL)?;
            }
            PsbPrimitive::CompilerArray => {
                self.stream.write_u8(PSB_COMPILER_ARRAY)?;
            }
            PsbPrimitive::CompilerBool => {
                self.stream.write_u8(PSB_COMPILER_BOOL)?;
            }
            PsbPrimitive::CompilerBinaryTree => {
                self.stream.write_u8(PSB_COMPILER_BINARY_TREE)?;
            }
        }

        Ok(())
    }

    pub fn write_uint_array(&mut self, arr: &[u64]) -> Result<(), PsbValueWriteError> {
        let item_byte_size = arr.iter().max().copied().map(get_uint_n).unwrap_or(1);
        if item_byte_size > 8 {
            return Err(PsbValueWriteError::InvalidInput);
        }

        let len = arr.len() as u64;
        let len_n = get_uint_n(len);
        self.stream.write_u8(PSB_TYPE_INTEGER_ARRAY_N + len_n)?;
        write_partial_uint(&mut self.stream, len, len_n)?;
        self.stream
            .write_u8(item_byte_size + PSB_TYPE_INTEGER_ARRAY_N)?;

        for &v in arr {
            write_partial_uint(&mut self.stream, v, item_byte_size)?;
        }
        Ok(())
    }
}
