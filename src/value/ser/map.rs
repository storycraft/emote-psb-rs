use std::io::{Seek, Write};

use serde::ser::{SerializeMap, SerializeStruct, SerializeStructVariant};

use crate::value::ser::{
    Error, Serializer,
    special::SpecialValueSerializer,
    value::{ref_type::RefTypeSerializer, unit::UnitTypeSerializer},
};

pub enum StructSerializer<'a, T: Write + Seek> {
    Map(MapSerializer<'a, T>),
    RefTy(SpecialValueSerializer<RefTypeSerializer<'a, T>>),
    UnitTy(SpecialValueSerializer<UnitTypeSerializer<'a, T>>),
}

impl<T> SerializeStruct for StructSerializer<'_, T>
where
    T: Write + Seek,
{
    type Ok = u64;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        match self {
            StructSerializer::Map(se) => SerializeStruct::serialize_field(se, key, value),
            StructSerializer::RefTy(se) => se.serialize_field(key, value),
            StructSerializer::UnitTy(se) => se.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            StructSerializer::Map(se) => SerializeMap::end(se),
            StructSerializer::RefTy(se) => se.end(),
            StructSerializer::UnitTy(se) => se.end(),
        }
    }
}

pub struct MapSerializer<'a, T>(pub &'a mut Serializer<T>);

impl<T: Write + Seek> SerializeMap for MapSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    fn serialize_key<K>(&mut self, key: &K) -> Result<(), Self::Error>
    where
        K: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn serialize_value<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<T: Write + Seek> SerializeStruct for MapSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

impl<T: Write + Seek> SerializeStructVariant for MapSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeStruct::end(self)
    }
}
