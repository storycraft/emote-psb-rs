use core::{ops::Range, pin::pin};
use std::io::{self, SeekFrom};

use async_stream::try_stream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};

use crate::value::{
    PsbNameIndex, PsbPrimitive,
    io::{
        PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
        PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
        PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0,
        PSB_TYPE_INTEGER_ARRAY_N, PSB_TYPE_INTEGER_N, PSB_TYPE_LIST, PSB_TYPE_NONE, PSB_TYPE_NULL,
        PSB_TYPE_OBJECT, PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE,
        error::PsbValueReadError,
    },
    number::PsbNumber,
    utill::PsbValueReadExt,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PsbStreamValue {
    Primitive(PsbPrimitive),
    UintArray { len: u64, item_byte_size: u8 },
    List,
    Object,
}

#[derive(Debug)]
pub struct PsbValueReader<T> {
    stream: T,
    buf: Vec<u64>,
}

impl<T: AsyncRead + Unpin> PsbValueReader<T> {
    pub const fn new(stream: T) -> Self {
        Self {
            stream,
            buf: vec![],
        }
    }

    pub async fn next(&mut self) -> Result<PsbStreamValue, PsbValueReadError> {
        const PSB_TYPE_INTEGER_ARRAY_START: u8 = PSB_TYPE_INTEGER_ARRAY_N + 1;
        const PSB_TYPE_INTEGER_ARRAY_MAX: u8 = PSB_TYPE_INTEGER_ARRAY_N + 8;

        let value_type = self.stream.read_u8().await?;
        if let Some(primitive) = self.read_primitive(value_type).await? {
            return Ok(PsbStreamValue::Primitive(primitive));
        }

        match value_type {
            value_type @ PSB_TYPE_INTEGER_ARRAY_START..=PSB_TYPE_INTEGER_ARRAY_MAX => {
                let len = self
                    .stream
                    .read_partial_uint(value_type - PSB_TYPE_INTEGER_ARRAY_N)
                    .await?;
                let item_byte_size = self.stream.read_u8().await? - PSB_TYPE_INTEGER_ARRAY_N;

                Ok(PsbStreamValue::UintArray {
                    len,
                    item_byte_size,
                })
            }

            PSB_TYPE_LIST => Ok(PsbStreamValue::List),
            PSB_TYPE_OBJECT => Ok(PsbStreamValue::Object),

            value_type => Err(PsbValueReadError::InvalidValueType(value_type)),
        }
    }

    /// Read stream of u64 values
    pub fn next_uint_array(
        &mut self,
        item_byte_size: u8,
        len: u64,
    ) -> impl Stream<Item = io::Result<u64>> {
        read_uint_array(&mut self.stream, item_byte_size, len)
    }

    /// Read stream of items in list
    pub async fn next_list<'a>(&'a mut self) -> Result<PsbListAccess<'a, T>, PsbValueReadError>
    where
        T: AsyncSeek,
    {
        let offsets = self.read_uint_array_buf().await?;
        let data_start = self.stream.stream_position().await?;

        Ok(PsbListAccess {
            buf_reader: PsbBufReader::new(self, offsets.start),
            offsets,
            data_start,
        })
    }

    /// Read streams of items in object
    pub async fn next_object<'a>(&'a mut self) -> Result<PsbObjectAccess<'a, T>, PsbValueReadError>
    where
        T: AsyncSeek,
    {
        let names = self.read_uint_array_buf().await?;
        let offsets = self.read_uint_array_buf().await?;

        let data_start = self.stream.stream_position().await?;
        Ok(PsbObjectAccess {
            buf_reader: PsbBufReader::new(self, names.start),
            names,
            offsets,
            data_start,
        })
    }

    async fn read_uint_array_buf(&mut self) -> Result<Range<usize>, PsbValueReadError> {
        let PsbStreamValue::UintArray {
            item_byte_size,
            len,
        } = self.next().await?
        else {
            return Err(PsbValueReadError::InvalidValue);
        };
        let mut list = pin!(read_uint_array(&mut self.stream, item_byte_size, len));
        let start = self.buf.len();
        while let Some(name) = list.try_next().await? {
            self.buf.push(name);
        }
        let end = self.buf.len();

        Ok(start..end)
    }

    async fn read_primitive(
        &mut self,
        value_type: u8,
    ) -> Result<Option<PsbPrimitive>, PsbValueReadError> {
        const PSB_TYPE_INTEGER_START: u8 = PSB_TYPE_INTEGER_N;
        const PSB_TYPE_INTEGER_MAX: u8 = PSB_TYPE_INTEGER_N + 8;
        const PSB_TYPE_RESOURCE_START: u8 = PSB_TYPE_RESOURCE_N + 1;
        const PSB_TYPE_RESOURCE_MAX: u8 = PSB_TYPE_RESOURCE_N + 4;
        const PSB_TYPE_STRING_START: u8 = PSB_TYPE_STRING_N + 1;
        const PSB_TYPE_STRING_MAX: u8 = PSB_TYPE_STRING_N + 4;
        const PSB_TYPE_EXTRA_START: u8 = PSB_TYPE_EXTRA_N + 1;
        const PSB_TYPE_EXTRA_MAX: u8 = PSB_TYPE_EXTRA_N + 4;

        match value_type {
            PSB_TYPE_NONE => Ok(Some(PsbPrimitive::None)),
            PSB_TYPE_NULL => Ok(Some(PsbPrimitive::Null)),

            PSB_TYPE_FALSE => Ok(Some(PsbPrimitive::Bool(false))),
            PSB_TYPE_TRUE => Ok(Some(PsbPrimitive::Bool(true))),

            PSB_TYPE_DOUBLE => Ok(Some(PsbPrimitive::Number(PsbNumber::Double(
                self.stream.read_f64_le().await?,
            )))),
            PSB_TYPE_FLOAT0 => Ok(Some(PsbPrimitive::Number(PsbNumber::Float(0.0)))),
            PSB_TYPE_FLOAT => Ok(Some(PsbPrimitive::Number(PsbNumber::Float(
                self.stream.read_f32_le().await?,
            )))),

            value_type @ PSB_TYPE_INTEGER_START..=PSB_TYPE_INTEGER_MAX => {
                Ok(Some(PsbPrimitive::Number(PsbNumber::Integer(
                    self.stream
                        .read_partial_int(value_type - PSB_TYPE_INTEGER_N)
                        .await?,
                ))))
            }

            value_type @ PSB_TYPE_RESOURCE_START..=PSB_TYPE_RESOURCE_MAX => {
                Ok(Some(PsbPrimitive::Resource(
                    self.stream
                        .read_partial_uint(value_type - PSB_TYPE_RESOURCE_N)
                        .await?
                        .try_into()
                        .map_err(|_| PsbValueReadError::InvalidValue)?,
                )))
            }

            value_type @ PSB_TYPE_STRING_START..=PSB_TYPE_STRING_MAX => {
                Ok(Some(PsbPrimitive::String(
                    self.stream
                        .read_partial_uint(value_type - PSB_TYPE_STRING_N)
                        .await?
                        .try_into()
                        .map_err(|_| PsbValueReadError::InvalidValue)?,
                )))
            }

            value_type @ PSB_TYPE_EXTRA_START..=PSB_TYPE_EXTRA_MAX => {
                Ok(Some(PsbPrimitive::ExtraResource(
                    self.stream
                        .read_partial_uint(value_type - PSB_TYPE_EXTRA_N)
                        .await?
                        .try_into()
                        .map_err(|_| PsbValueReadError::InvalidValue)?,
                )))
            }

            PSB_COMPILER_INTEGER => Ok(Some(PsbPrimitive::CompilerNumber)),
            PSB_COMPILER_STRING => Ok(Some(PsbPrimitive::CompilerString)),
            PSB_COMPILER_RESOURCE => Ok(Some(PsbPrimitive::CompilerResource)),
            PSB_COMPILER_ARRAY => Ok(Some(PsbPrimitive::CompilerArray)),
            PSB_COMPILER_DECIMAL => Ok(Some(PsbPrimitive::CompilerDecimal)),
            PSB_COMPILER_BOOL => Ok(Some(PsbPrimitive::CompilerBool)),
            PSB_COMPILER_BINARY_TREE => Ok(Some(PsbPrimitive::CompilerBinaryTree)),

            _ => Ok(None),
        }
    }
}

#[derive(Debug)]
struct PsbBufReader<'a, T> {
    inner: &'a mut PsbValueReader<T>,
    buf_start: usize,
}

impl<'a, T> PsbBufReader<'a, T> {
    const fn new(inner: &'a mut PsbValueReader<T>, buf_start: usize) -> Self {
        Self { inner, buf_start }
    }
}

impl<'a, T> Drop for PsbBufReader<'a, T> {
    fn drop(&mut self) {
        self.inner.buf.drain(self.buf_start..);
    }
}

#[derive(Debug)]
pub struct PsbListAccess<'a, T> {
    buf_reader: PsbBufReader<'a, T>,
    offsets: Range<usize>,
    data_start: u64,
}

impl<'a, T> PsbListAccess<'a, T>
where
    T: AsyncRead + AsyncSeek + Unpin,
{
    pub async fn next(&mut self) -> io::Result<Option<&mut PsbValueReader<T>>> {
        let Some(offset) = self.offsets.next() else {
            return Ok(None);
        };

        self.buf_reader
            .inner
            .stream
            .seek(SeekFrom::Start(
                self.data_start + self.buf_reader.inner.buf[offset],
            ))
            .await?;
        Ok(Some(self.buf_reader.inner))
    }
}

#[derive(Debug)]
pub struct PsbObjectAccess<'a, T> {
    buf_reader: PsbBufReader<'a, T>,
    names: Range<usize>,
    offsets: Range<usize>,
    data_start: u64,
}

impl<'a, T> PsbObjectAccess<'a, T>
where
    T: AsyncRead + AsyncSeek + Unpin,
{
    pub async fn next(&mut self) -> io::Result<Option<(PsbNameIndex, &mut PsbValueReader<T>)>> {
        let Some(name) = self.names.next() else {
            return Ok(None);
        };
        let Some(offset) = self.offsets.next() else {
            return Ok(None);
        };

        let index = PsbNameIndex(self.buf_reader.inner.buf[name]);
        self.buf_reader
            .inner
            .stream
            .seek(SeekFrom::Start(
                self.data_start + self.buf_reader.inner.buf[offset],
            ))
            .await?;
        Ok(Some((index, self.buf_reader.inner)))
    }
}

fn read_uint_array(
    stream: &mut (impl AsyncRead + Unpin),
    item_byte_size: u8,
    len: u64,
) -> impl Stream<Item = io::Result<u64>> {
    try_stream!(for _ in 0..len {
        yield stream.read_partial_uint(item_byte_size).await?;
    })
}
