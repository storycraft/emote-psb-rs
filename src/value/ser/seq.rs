use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

use crate::value::ser::{
    Error, Serializer,
    buffer::{Buffer, BufferValue},
};

pub struct SeqSerializer<'a> {
    len: usize,
    buf: &'a mut Buffer,
}

impl<'a> SeqSerializer<'a> {
    pub const fn new(buf: &'a mut Buffer) -> Self {
        Self { len: 0, buf }
    }
}

impl<'a> SerializeSeq for SeqSerializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.len += 1;
        value.serialize(Serializer(self.buf))?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let index = self.buf.values.len() - self.len;
        self.buf.values[index] = BufferValue::List { len: self.len };
        Ok(self.buf)
    }
}

impl<'a> SerializeTuple for SeqSerializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleVariant for SeqSerializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}
