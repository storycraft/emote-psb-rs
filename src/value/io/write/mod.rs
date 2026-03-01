pub mod ext;

use core::num::NonZeroU8;
use std::io;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::value::{
    PsbPrimitive,
    io::{
        PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
        PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
        PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0,
        PSB_TYPE_INTEGER_ARRAY_N, PSB_TYPE_INTEGER_N, PSB_TYPE_LIST, PSB_TYPE_NONE, PSB_TYPE_NULL,
        PSB_TYPE_OBJECT, PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE,
        error::PsbValueWriteError,
    },
    number::PsbNumber,
    utill::{get_n, get_uint_n, write_partial_uint},
};

#[derive(Debug)]
pub struct PsbStreamValueWriter<T> {
    stream: T,
}

impl<T: AsyncWrite + Unpin> PsbStreamValueWriter<T> {
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

    pub async fn begin_uint_array<'a>(
        &'a mut self,
        len: u64,
        item_byte_size: NonZeroU8,
    ) -> Result<PsbUintArrayWriter<'a, T>, PsbValueWriteError> {
        let item_byte_size = item_byte_size.get();
        if item_byte_size > 8 {
            return Err(PsbValueWriteError::InvalidInput);
        }

        let len_n = get_uint_n(len);
        self.stream
            .write_u8(PSB_TYPE_INTEGER_ARRAY_N + len_n)
            .await?;
        write_partial_uint(&mut self.stream, len, len_n).await?;
        self.stream
            .write_u8(item_byte_size + PSB_TYPE_INTEGER_ARRAY_N)
            .await?;
        Ok(PsbUintArrayWriter {
            inner: self,
            item_byte_size,
        })
    }

    pub async fn begin_list<'a>(
        &'a mut self,
        len: u64,
        offset_byte_size: NonZeroU8,
    ) -> Result<PsbListEntryWriter<'a, T>, PsbValueWriteError> {
        self.stream.write_u8(PSB_TYPE_LIST).await?;
        Ok(PsbListEntryWriter(
            self.begin_uint_array(len, offset_byte_size).await?,
        ))
    }

    pub async fn begin_object<'a>(
        &'a mut self,
        len: u64,
        name_byte_size: NonZeroU8,
    ) -> Result<PsbObjectNameWriter<'a, T>, PsbValueWriteError> {
        self.stream.write_u8(PSB_TYPE_OBJECT).await?;
        Ok(PsbObjectNameWriter {
            inner: self.begin_uint_array(len, name_byte_size).await?,
            len,
        })
    }
}

#[derive(Debug)]
pub struct PsbUintArrayWriter<'a, T> {
    inner: &'a mut PsbStreamValueWriter<T>,
    item_byte_size: u8,
}

impl<'a, T> PsbUintArrayWriter<'a, T>
where
    T: AsyncWrite + Unpin,
{
    pub async fn write_next(&mut self, v: u64) -> io::Result<()> {
        write_partial_uint(&mut self.inner.stream, v, self.item_byte_size).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PsbListEntryWriter<'a, T>(PsbUintArrayWriter<'a, T>);

impl<'a, T> PsbListEntryWriter<'a, T>
where
    T: AsyncWrite + Unpin,
{
    pub async fn next_entry(&mut self, offset: u64) -> io::Result<()> {
        self.0.write_next(offset).await
    }
}

#[derive(Debug)]
pub struct PsbObjectNameWriter<'a, T> {
    inner: PsbUintArrayWriter<'a, T>,
    len: u64,
}

impl<'a, T> PsbObjectNameWriter<'a, T>
where
    T: AsyncWrite + Unpin,
{
    #[inline]
    pub async fn next_key(&mut self, name: u64) -> io::Result<()> {
        self.inner.write_next(name).await
    }

    pub async fn finish(
        self,
        offset_byte_size: NonZeroU8,
    ) -> Result<PsbObjectOffsetWriter<'a, T>, PsbValueWriteError> {
        Ok(PsbObjectOffsetWriter(
            self.inner
                .inner
                .begin_uint_array(self.len, offset_byte_size)
                .await?,
        ))
    }
}

#[derive(Debug)]
pub struct PsbObjectOffsetWriter<'a, T>(PsbUintArrayWriter<'a, T>);

impl<'a, T> PsbObjectOffsetWriter<'a, T>
where
    T: AsyncWrite + Unpin,
{
    pub async fn next_entry(&mut self, name: u64) -> io::Result<()> {
        self.0.write_next(name).await
    }
}
