use core::pin::pin;

use async_stream::try_stream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};

use crate::value::{
    PsbPrimitive,
    io::{
        PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
        PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
        PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0,
        PSB_TYPE_INTEGER_ARRAY_N, PSB_TYPE_INTEGER_N, PSB_TYPE_LIST, PSB_TYPE_NONE, PSB_TYPE_NULL,
        PSB_TYPE_OBJECT, PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE,
        error::PsbValueReadError,
    },
    number::PsbNumber,
    utill::PsbValueStreamExt,
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
}

impl<T: AsyncRead + Unpin> PsbValueReader<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub async fn read_next(&mut self) -> Result<PsbStreamValue, PsbValueReadError> {
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
    pub fn read_uint_array(
        &mut self,
        item_byte_size: u8,
        len: u64,
    ) -> impl Stream<Item = Result<u64, PsbValueReadError>> {
        try_stream!(for _ in 0..len {
            yield self.stream.read_partial_uint(item_byte_size).await?;
        })
    }

    /// Read stream of positions in list
    pub fn read_list(&mut self) -> impl Stream<Item = Result<u64, PsbValueReadError>>
    where
        T: AsyncSeek,
    {
        try_stream!(
            let PsbStreamValue::UintArray { item_byte_size, len } = self.read_next().await? else {
                Err(PsbValueReadError::InvalidValue)?;
                return;
            };
            let list_start = self.stream.stream_position().await? + item_byte_size as u64 * len;

            let mut offsets = pin!(self.read_uint_array(item_byte_size, len));
            while let Some(offset) = offsets.try_next().await? {
                yield list_start + offset;
            }
        )
    }

    /// Read stream of names and positions in object
    pub fn read_object(
        &mut self,
    ) -> impl Stream<Item = Result<(u64, u64), PsbValueReadError>>
    where
        T: AsyncSeek,
    {
        try_stream!(
            let PsbStreamValue::UintArray { item_byte_size, len } = self.read_next().await? else {
                Err(PsbValueReadError::InvalidValue)?;
                return;
            };
            let names = self.read_uint_array(item_byte_size, len).try_collect::<Vec<_>>().await?;

            let PsbStreamValue::UintArray { item_byte_size, len } = self.read_next().await? else {
                Err(PsbValueReadError::InvalidValue)?;
                return;
            };
            let object_start = self.stream.stream_position().await? + item_byte_size as u64 * len;
            let mut offsets = pin!(self.read_uint_array(item_byte_size, len));

            let mut names_iter = names.into_iter();
            while let Some(offset) = offsets.try_next().await? {
                let Some(name) = names_iter.next() else {
                    Err(PsbValueReadError::InvalidValue)?;
                    return;
                };

                yield (name, object_start + offset);
            }
        )
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
        const PSB_TYPE_EXTRA_MAX: u8 = PSB_TYPE_EXTRA_N + 8;

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
                        .await?,
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
