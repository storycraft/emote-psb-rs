use byteorder::WriteBytesExt;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

use crate::value::{
    PSB_TYPE_LIST,
    ser::{
        Error, Serializer,
        buffer::{Buffer, BufferObject, BufferValue},
    },
    util::write_uint_array,
};

pub struct SeqSerializer<'a> {
    len: usize,
    next_offset: usize,
    list_index: usize,
    data_start: usize,
    offset_start: usize,
    buf: &'a mut Buffer,
}

impl<'a> SeqSerializer<'a> {
    pub fn new(buf: &'a mut Buffer) -> Self {
        let list_index = buf.values.len();
        buf.values.push(BufferValue::Invalid);
        let data_start = buf.bytes.len();
        let offset_start = buf.offsets.len();
        Self {
            len: 0,
            next_offset: 0,
            list_index,
            data_start,
            offset_start,
            buf,
        }
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

        let index = self.buf.values.len();
        value.serialize(Serializer(self.buf))?;

        let offset = self.next_offset;
        self.next_offset += self.buf.values[index].size(self.buf);
        self.buf.offsets.push(offset as _);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let header_start = self.buf.bytes.len();
        self.buf.bytes.write_u8(PSB_TYPE_LIST)?;
        write_uint_array(&mut self.buf.bytes, &self.buf.offsets[self.offset_start..])?;
        debug_assert_eq!(self.buf.offsets.len() - self.offset_start, self.len);
        let header_end = self.buf.bytes.len();

        self.buf.offsets.drain(self.offset_start..);

        let index = self.buf.objects.len();
        self.buf.objects.push(BufferObject {
            len: self.len,
            header_offset: header_start - self.data_start,
            header_size: header_end - header_start,
        });

        self.buf.values[self.list_index] = BufferValue::Object { index };
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
