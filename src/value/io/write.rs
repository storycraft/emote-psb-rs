use std::io;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::value::{
    PsbPrimitive,
    io::{
        PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
        PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
        PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0, PSB_TYPE_INTEGER_N,
        PSB_TYPE_NONE, PSB_TYPE_NULL, PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE,
    },
    number::PsbNumber,
    utill::{get_n, get_uint_n},
};

#[derive(Debug)]
pub struct PsbValueWriter<T> {
    stream: T,
}

impl<T: AsyncWrite + Unpin> PsbValueWriter<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub async fn write_primitive(&mut self, primitive: PsbPrimitive) -> io::Result<()> {
        match primitive {
            PsbPrimitive::None => {
                self.stream.write_u8(PSB_TYPE_NONE).await?;
            }
            PsbPrimitive::Null => {
                self.stream.write_u8(PSB_TYPE_NULL).await?;
            }
            PsbPrimitive::Bool(value) => {
                if value {
                    self.stream.write_u8(PSB_TYPE_TRUE).await?;
                } else {
                    self.stream.write_u8(PSB_TYPE_FALSE).await?;
                }
            }
            PsbPrimitive::Number(number) => match number {
                PsbNumber::Integer(v) => {
                    let n = get_n(v);
                    self.stream.write_u8(PSB_TYPE_INTEGER_N + n).await?;
                    self.stream
                        .write_all(&v.to_le_bytes()[..n as usize])
                        .await?;
                }

                PsbNumber::Double(v) => {
                    self.stream.write_u8(PSB_TYPE_DOUBLE).await?;
                    self.stream.write_f64_le(v).await?;
                }

                PsbNumber::Float(v) => {
                    if v == 0.0 {
                        self.stream.write_u8(PSB_TYPE_FLOAT0).await?;
                    } else {
                        self.stream.write_u8(PSB_TYPE_FLOAT).await?;
                        self.stream.write_f32_le(v).await?;
                    }
                }
            },
            PsbPrimitive::Resource(index) => {
                let n = get_uint_n(index as _);
                self.stream.write_u8(PSB_TYPE_RESOURCE_N + n).await?;
                self.stream
                    .write_all(&index.to_le_bytes()[..n as usize])
                    .await?;
            }
            PsbPrimitive::String(index) => {
                let n = get_uint_n(index as _);
                self.stream.write_u8(PSB_TYPE_STRING_N + n).await?;
                self.stream
                    .write_all(&index.to_le_bytes()[..n as usize])
                    .await?;
            }
            PsbPrimitive::ExtraResource(index) => {
                let n = get_uint_n(index as _);
                self.stream.write_u8(PSB_TYPE_EXTRA_N + n).await?;
                self.stream
                    .write_all(&index.to_le_bytes()[..n as usize])
                    .await?;
            }
            PsbPrimitive::CompilerNumber => {
                self.stream.write_u8(PSB_COMPILER_INTEGER).await?;
            }
            PsbPrimitive::CompilerString => {
                self.stream.write_u8(PSB_COMPILER_STRING).await?;
            }
            PsbPrimitive::CompilerResource => {
                self.stream.write_u8(PSB_COMPILER_RESOURCE).await?;
            }
            PsbPrimitive::CompilerDecimal => {
                self.stream.write_u8(PSB_COMPILER_DECIMAL).await?;
            }
            PsbPrimitive::CompilerArray => {
                self.stream.write_u8(PSB_COMPILER_ARRAY).await?;
            }
            PsbPrimitive::CompilerBool => {
                self.stream.write_u8(PSB_COMPILER_BOOL).await?;
            }
            PsbPrimitive::CompilerBinaryTree => {
                self.stream.write_u8(PSB_COMPILER_BINARY_TREE).await?;
            }
        }

        Ok(())
    }

    pub async fn write_list() {
        
    }
}
