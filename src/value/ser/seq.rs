use std::io::{Seek, Write};

use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

use crate::value::ser::{Error, Serializer};

pub struct SeqSerializer<'a, T>(pub &'a mut Serializer<T>);

impl<T: Seek + Write> SerializeSeq for SeqSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    fn serialize_element<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<T: Seek + Write> SerializeTuple for SeqSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    fn serialize_element<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<T: Seek + Write> SerializeTupleStruct for SeqSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    fn serialize_field<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<T: Seek + Write> SerializeTupleVariant for SeqSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    fn serialize_field<V>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}
