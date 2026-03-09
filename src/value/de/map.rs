use core::ops::Range;
use std::io::{BufRead, Seek, SeekFrom};

use serde::{
    de::{IntoDeserializer, MapAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::{
    psb::table::StringTable,
    value::{
        PsbNameIndex,
        de::{Deserializer, Error, error},
    },
};

pub struct PsbObject<'a, 'b, T> {
    data_start: u64,
    names: Range<usize>,
    offsets: Range<usize>,
    inner: &'b mut Deserializer<'a, T>,
}

impl<'a, 'b, T> PsbObject<'a, 'b, T> {
    pub fn new(
        inner: &'b mut Deserializer<'a, T>,
        data_start: u64,
        names: Range<usize>,
        offsets: Range<usize>,
    ) -> Self {
        Self {
            data_start,
            names,
            offsets,
            inner,
        }
    }
}

impl<'a, 'b, T> MapAccess<'static> for PsbObject<'a, 'b, T>
where
    T: BufRead + Seek,
{
    type Error = error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'static>,
    {
        let Some(index) = self.names.next() else {
            return Ok(None);
        };

        Ok(Some(seed.deserialize(NameDeserializer {
            names: self.inner.names,
            id: self.inner.buf[index] as _,
        })?))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'static>,
    {
        let index = self.offsets.next().ok_or(error::Error::InvalidValue)?;
        let offset = self.inner.buf[index];
        self.inner
            .stream
            .seek(SeekFrom::Start(self.data_start + offset))?;
        seed.deserialize(&mut *self.inner)
    }
}

struct NameDeserializer<'a> {
    names: &'a StringTable,
    id: usize,
}

impl<'a> serde::Deserializer<'static> for NameDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let name = self.names.get(self.id).ok_or(error::Error::InvalidValue)?;
        visitor.visit_str(name)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        if name == PsbNameIndex::MARKER {
            return visitor.visit_newtype_struct(self.id.into_deserializer());
        }

        self.deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        <V: Visitor<'static>>
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
