use core::ops::Range;
use std::io::{BufRead, Seek, SeekFrom};

use serde::de::{IntoDeserializer, MapAccess, value::StrDeserializer};

use crate::value::de::{Deserializer, error};

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
        let name = self
            .inner
            .names
            .get(self.inner.buf[index] as _)
            .ok_or(error::Error::InvalidValue)?;
        let de: StrDeserializer<error::Error> = name.into_deserializer();
        Ok(Some(seed.deserialize(de)?))
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
