mod error;
mod map;
mod seq;
mod special;

pub use error::Error;

use core::ops::Range;
use std::io::{BufRead, ErrorKind, Seek};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{de::value::SeqAccessDeserializer, forward_to_deserialize_any};

use crate::{
    psb::table::StringTable,
    value::{
        PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
        PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
        PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0,
        PSB_TYPE_INTEGER_ARRAY_N, PSB_TYPE_INTEGER_N, PSB_TYPE_LIST, PSB_TYPE_NULL,
        PSB_TYPE_OBJECT, PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE, PsbCompilerArray,
        PsbCompilerBinaryTree, PsbCompilerBool, PsbCompilerDecimal, PsbCompilerNumber,
        PsbCompilerResource, PsbCompilerString, PsbExtraResource, PsbResource, PsbUIntArray,
        de::{
            map::PsbObject,
            seq::{List, UIntArray},
            special::SpecialTypeDeserializer,
        },
        util::{read_partial_int, read_partial_uint, read_uint_array},
    },
};

pub struct Deserializer<'a, T> {
    names: &'a StringTable,
    strings: &'a StringTable,
    buf: Vec<u64>,
    stream: T,
}

impl<'a, T: BufRead + Seek> Deserializer<'a, T> {
    pub fn new(names: &'a StringTable, strings: &'a StringTable, stream: T) -> Self {
        Self {
            names,
            strings,
            buf: vec![],
            stream,
        }
    }

    fn peek_ty(&mut self) -> Result<u8, Error> {
        self.stream
            .fill_buf()?
            .first()
            .copied()
            .ok_or(Error::Io(ErrorKind::UnexpectedEof.into()))
    }

    fn read_uint_array_buf(&mut self) -> Result<Range<usize>, Error> {
        let start = self.buf.len();
        let len = read_uint_array(&mut self.stream, &mut self.buf)?;
        Ok(start..(start + len))
    }
}

impl<T: BufRead + Seek> serde::Deserializer<'static> for &mut Deserializer<'_, T> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'static>,
    {
        const PSB_TYPE_INTEGER_START: u8 = PSB_TYPE_INTEGER_N;
        const PSB_TYPE_INTEGER_MAX: u8 = PSB_TYPE_INTEGER_N + 8;
        const PSB_TYPE_RESOURCE_START: u8 = PSB_TYPE_RESOURCE_N + 1;
        const PSB_TYPE_RESOURCE_MAX: u8 = PSB_TYPE_RESOURCE_N + 4;
        const PSB_TYPE_STRING_START: u8 = PSB_TYPE_STRING_N + 1;
        const PSB_TYPE_STRING_MAX: u8 = PSB_TYPE_STRING_N + 4;
        const PSB_TYPE_EXTRA_START: u8 = PSB_TYPE_EXTRA_N + 1;
        const PSB_TYPE_EXTRA_MAX: u8 = PSB_TYPE_EXTRA_N + 4;
        const PSB_TYPE_INTEGER_ARRAY_START: u8 = PSB_TYPE_INTEGER_ARRAY_N + 1;
        const PSB_TYPE_INTEGER_ARRAY_MAX: u8 = PSB_TYPE_INTEGER_ARRAY_N + 8;

        match self.stream.read_u8()? {
            PSB_TYPE_NULL => visitor.visit_unit(),

            PSB_TYPE_FALSE => visitor.visit_bool(false),
            PSB_TYPE_TRUE => visitor.visit_bool(true),

            PSB_TYPE_DOUBLE => visitor.visit_f64(self.stream.read_f64::<LittleEndian>()?),
            PSB_TYPE_FLOAT0 => visitor.visit_f32(0.0),
            PSB_TYPE_FLOAT => visitor.visit_f32(self.stream.read_f32::<LittleEndian>()?),

            value_type @ PSB_TYPE_INTEGER_START..=PSB_TYPE_INTEGER_MAX => visitor.visit_i64(
                read_partial_int(&mut self.stream, value_type - PSB_TYPE_INTEGER_N)?,
            ),

            value_type @ PSB_TYPE_STRING_START..=PSB_TYPE_STRING_MAX => {
                let idx: u32 = read_partial_uint(&mut self.stream, value_type - PSB_TYPE_STRING_N)?
                    .try_into()
                    .map_err(|_| Error::InvalidValue)?;

                visitor.visit_str(self.strings.get(idx as _).ok_or(Error::InvalidValue)?)
            }

            value_type @ PSB_TYPE_RESOURCE_START..=PSB_TYPE_RESOURCE_MAX => {
                let idx: u32 =
                    read_partial_uint(&mut self.stream, value_type - PSB_TYPE_RESOURCE_N)?
                        .try_into()
                        .map_err(|_| Error::InvalidValue)?;

                SpecialTypeDeserializer::new(PsbResource::MARKER, idx).deserialize(visitor)
            }

            value_type @ PSB_TYPE_EXTRA_START..=PSB_TYPE_EXTRA_MAX => {
                let idx: u32 = read_partial_uint(&mut self.stream, value_type - PSB_TYPE_EXTRA_N)?
                    .try_into()
                    .map_err(|_| Error::InvalidValue)?;

                SpecialTypeDeserializer::new(PsbExtraResource::MARKER, idx).deserialize(visitor)
            }

            ty @ PSB_TYPE_INTEGER_ARRAY_START..=PSB_TYPE_INTEGER_ARRAY_MAX => {
                let len = read_partial_uint(&mut self.stream, ty - PSB_TYPE_INTEGER_ARRAY_N)?;
                let item_byte_size = self.stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N;

                SpecialTypeDeserializer::new(
                    PsbUIntArray::MARKER,
                    SeqAccessDeserializer::new(UIntArray::new(
                        len as _,
                        item_byte_size,
                        &mut self.stream,
                    )),
                )
                .deserialize(visitor)
            }

            PSB_TYPE_LIST => {
                let offsets = self.read_uint_array_buf()?;
                let buf_start = offsets.start;
                let data_start = self.stream.stream_position()?;
                let res = visitor.visit_seq(List::new(data_start, offsets, self));
                self.buf.drain(buf_start..);
                res
            }

            PSB_TYPE_OBJECT => {
                let names = self.read_uint_array_buf()?;
                let buf_start = names.start;
                let offsets = self.read_uint_array_buf()?;
                let data_start = self.stream.stream_position()?;
                let res = visitor.visit_map(PsbObject::new(self, data_start, names, offsets));
                self.buf.drain(buf_start..);
                res
            }

            PSB_COMPILER_INTEGER => {
                SpecialTypeDeserializer::new(PsbCompilerNumber::MARKER, ()).deserialize(visitor)
            }
            PSB_COMPILER_STRING => {
                SpecialTypeDeserializer::new(PsbCompilerString::MARKER, ()).deserialize(visitor)
            }
            PSB_COMPILER_RESOURCE => {
                SpecialTypeDeserializer::new(PsbCompilerResource::MARKER, ()).deserialize(visitor)
            }
            PSB_COMPILER_ARRAY => {
                SpecialTypeDeserializer::new(PsbCompilerArray::MARKER, ()).deserialize(visitor)
            }
            PSB_COMPILER_DECIMAL => {
                SpecialTypeDeserializer::new(PsbCompilerDecimal::MARKER, ()).deserialize(visitor)
            }
            PSB_COMPILER_BOOL => {
                SpecialTypeDeserializer::new(PsbCompilerBool::MARKER, ()).deserialize(visitor)
            }
            PSB_COMPILER_BINARY_TREE => {
                SpecialTypeDeserializer::new(PsbCompilerBinaryTree::MARKER, ()).deserialize(visitor)
            }

            ty => Err(Error::InvalidValueType(ty)),
        }
    }

    forward_to_deserialize_any! {
        <W: Visitor<'static>>
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct enum identifier
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'static>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'static>,
    {
        if self.peek_ty()? == PSB_TYPE_NULL {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'static>,
    {
        visitor.visit_unit()
    }
}
