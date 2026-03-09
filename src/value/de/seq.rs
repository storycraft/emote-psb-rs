use core::ops::Range;
use std::io::{BufRead, Seek, SeekFrom};

use serde::de::{IntoDeserializer, SeqAccess, value::U64Deserializer};

use crate::value::{
    de::{Deserializer, error},
    util::read_partial_uint,
};

pub struct UIntArray<'a, T> {
    remaining: usize,
    item_byte_size: u8,
    stream: &'a mut T,
}

impl<'a, T> UIntArray<'a, T> {
    pub const fn new(len: usize, item_byte_size: u8, stream: &'a mut T) -> Self {
        Self {
            remaining: len,
            item_byte_size,
            stream,
        }
    }
}

impl<'a, 'de, T> SeqAccess<'de> for UIntArray<'a, T>
where
    T: BufRead + Seek,
{
    type Error = error::Error;

    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        let read = read_partial_uint(self.stream, self.item_byte_size)?;
        let de: U64Deserializer<error::Error> = read.into_deserializer();
        Ok(Some(seed.deserialize(de)?))
    }
}

pub struct List<'a, 'b, T> {
    data_start: u64,
    offsets: Range<usize>,
    inner: &'b mut Deserializer<'a, T>,
}

impl<'a, 'b, T> List<'a, 'b, T> {
    pub const fn new(
        data_start: u64,
        offsets: Range<usize>,
        inner: &'b mut Deserializer<'a, T>,
    ) -> Self {
        Self {
            data_start,
            offsets,
            inner,
        }
    }
}

impl<'a, 'b, T> SeqAccess<'static> for List<'a, 'b, T>
where
    T: BufRead + Seek,
{
    type Error = error::Error;

    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>, Self::Error>
    where
        V: serde::de::DeserializeSeed<'static>,
    {
        let Some(index) = self.offsets.next() else {
            return Ok(None);
        };
        let offset = self.inner.buf[index];
        self.inner
            .stream
            .seek(SeekFrom::Start(self.data_start + offset))?;
        Ok(Some(seed.deserialize(&mut *self.inner)?))
    }
}
