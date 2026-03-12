use core::ops::Range;
use std::io::{BufRead, Seek, SeekFrom};

use serde::de::SeqAccess;

use crate::value::de::{Deserializer, error};

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
