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
    list_index: usize,
    data_start: usize,
    temp_index_start: usize,
    buf: &'a mut Buffer,
}

impl<'a> SeqSerializer<'a> {
    pub fn new(buf: &'a mut Buffer, len: Option<usize>) -> Self {
        if let Some(len) = len {
            buf.values.reserve(len + 1);
            buf.map_indexes.reserve(len);
        }

        let list_index = buf.values.len();
        buf.values.push(BufferValue::Invalid);
        let data_start = buf.bytes.len();
        let temp_index_start = buf.map_indexes.len();
        Self {
            len: 0,
            list_index,
            data_start,
            temp_index_start,
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
        self.buf.map_indexes.push(index);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut offset = 0;
        for i in 0..self.len {
            let value_index = self.buf.map_indexes[self.temp_index_start + i];
            self.buf.indexes.push(value_index);
            self.buf.offsets.push(offset);
            offset += self.buf.values[value_index].size(self.buf) as u64;
        }
        let index_start = self.buf.indexes.len();
        self.buf
            .indexes
            .extend(self.buf.map_indexes.drain(self.temp_index_start..));

        let header_start = self.buf.bytes.len();
        self.buf.bytes.write_u8(PSB_TYPE_LIST)?;
        write_uint_array(&mut self.buf.bytes, &self.buf.offsets)?;
        let header_end = self.buf.bytes.len();
        self.buf.offsets.clear();

        let index = self.buf.objects.len();
        self.buf.objects.push(BufferObject {
            len: self.len,
            data_start: self.data_start,
            header_start,
            header_end,
            index_start,
        });

        self.buf.values[self.list_index] = BufferValue::Object { index };
        Ok(self.buf)
    }
}

impl SerializeTuple for SeqSerializer<'_> {
    type Ok = <Self as SerializeSeq>::Ok;
    type Error = <Self as SerializeSeq>::Error;

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

impl SerializeTupleStruct for SeqSerializer<'_> {
    type Ok = <Self as SerializeSeq>::Ok;
    type Error = <Self as SerializeSeq>::Error;

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

impl SerializeTupleVariant for SeqSerializer<'_> {
    type Ok = <Self as SerializeSeq>::Ok;
    type Error = <Self as SerializeSeq>::Error;

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
