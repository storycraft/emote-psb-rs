use byteorder::WriteBytesExt;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

use crate::value::{
    PSB_TYPE_LIST,
    ser::{
        Error, Serializer, State,
        buffer::{BufferObject, BufferValue},
    },
    util::write_uint_array,
};

pub struct SeqSerializer<'a> {
    len: usize,
    list_index: usize,
    data_start: usize,
    temp_index_start: usize,
    state: State<'a>,
}

impl<'a> SeqSerializer<'a> {
    pub fn new(state: State<'a>, len: Option<usize>) -> Self {
        if let Some(len) = len {
            state.buf.values.reserve(len + 1);
            state.ser.keys.reserve(len);
            state.ser.map_indexes.reserve(len);
        }

        let list_index = state.buf.values.len();
        state.buf.values.push(BufferValue::Invalid);
        let data_start = state.buf.bytes.len();
        let temp_index_start = state.ser.map_indexes.len();
        Self {
            len: 0,
            list_index,
            data_start,
            temp_index_start,
            state,
        }
    }
}

impl<'a> SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.len += 1;

        let index = self.state.buf.values.len();
        value.serialize(Serializer(self.state.reborrow_mut()))?;
        self.state.ser.map_indexes.push(index);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut offset = 0;
        for i in 0..self.len {
            let value_index = self.state.ser.map_indexes[self.temp_index_start + i];
            self.state.buf.indexes.push(value_index);
            self.state.ser.offsets.push(offset);
            offset += self.state.buf.values[value_index].size(self.state.buf) as u64;
        }
        let index_start = self.state.buf.indexes.len();
        self.state
            .buf
            .indexes
            .extend(self.state.ser.map_indexes.drain(self.temp_index_start..));

        let header_start = self.state.buf.bytes.len();
        self.state.buf.bytes.write_u8(PSB_TYPE_LIST)?;
        write_uint_array(&mut self.state.buf.bytes, &self.state.ser.offsets)?;
        let header_end = self.state.buf.bytes.len();
        self.state.ser.offsets.clear();

        let index = self.state.buf.objects.len();
        self.state.buf.objects.push(BufferObject {
            len: self.len,
            data_start: self.data_start,
            header_start,
            header_end,
            index_start,
        });

        self.state.buf.values[self.list_index] = BufferValue::Object { index };
        Ok(())
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
