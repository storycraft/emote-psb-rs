use serde::de::{IntoDeserializer, MapAccess, Visitor};

use crate::value::de::Error;

pub struct SpecialTypeDeserializer<T> {
    marker: &'static str,
    value: Option<T>,
}

impl<T> SpecialTypeDeserializer<T>
where
    T: IntoDeserializer<'static, Error>,
{
    pub const fn new(marker: &'static str, value: T) -> Self {
        Self {
            marker,
            value: Some(value),
        }
    }

    pub fn deserialize<V: Visitor<'static>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_map(self)
    }
}

impl<T> MapAccess<'static> for SpecialTypeDeserializer<T>
where
    T: IntoDeserializer<'static, Error>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'static>,
    {
        if self.value.is_some() {
            seed.deserialize(self.marker.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'static>,
    {
        let Some(value) = self.value.take() else {
            unreachable!();
        };

        seed.deserialize(value.into_deserializer())
    }
}
