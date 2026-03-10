use serde::de::{IntoDeserializer, MapAccess, Visitor};

use crate::value::de::Error;

pub struct SpecialTypeDeserializer<I> {
    marker: &'static str,
    inner: Option<I>,
}

impl<'de, I> SpecialTypeDeserializer<I>
where
    I: IntoDeserializer<'de, Error>,
{
    pub fn new(marker: &'static str, inner: I) -> Self {
        Self {
            marker,
            inner: Some(inner),
        }
    }

    pub fn deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }
}

impl<'de, I> MapAccess<'de> for SpecialTypeDeserializer<I>
where
    I: IntoDeserializer<'de, Error>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.inner.is_some() {
            seed.deserialize(self.marker.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let Some(de) = self.inner.take() else {
            unreachable!();
        };

        seed.deserialize(de.into_deserializer())
    }
}
