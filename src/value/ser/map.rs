use byteorder::WriteBytesExt;
use serde::ser::{Impossible, SerializeMap, SerializeStruct, SerializeStructVariant};

use crate::value::{
    PSB_TYPE_OBJECT,
    ser::{
        Error, Serializer,
        buffer::{Buffer, BufferObject, BufferValue},
        special::SpecialValueSerializer,
        value::{ref_type::RefTypeSerializer, unit::UnitTypeSerializer},
    },
    util::write_uint_array,
};

pub enum StructSerializer<'a> {
    Map(MapSerializer<'a>),
    RefTy(SpecialValueSerializer<RefTypeSerializer<'a>>),
    UnitTy(SpecialValueSerializer<UnitTypeSerializer<'a>>),
}

impl<'a> SerializeStruct for StructSerializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
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

pub struct MapSerializer<'a> {
    len: usize,
    map_index: usize,
    data_start: usize,
    key_start: usize,
    temp_index_start: usize,
    buf: &'a mut Buffer,
}

impl<'a> MapSerializer<'a> {
    pub fn new(buf: &'a mut Buffer, len: Option<usize>) -> Self {
        if let Some(len) = len {
            buf.values.reserve(len + 1);
            buf.keys.reserve(len);
            buf.map_indexes.reserve(len);
        }

        let map_index = buf.values.len();
        buf.values.push(BufferValue::Invalid);
        let data_start = buf.bytes.len();
        let key_start = buf.keys.len();
        let temp_index_start = buf.map_indexes.len();
        Self {
            len: 0,
            map_index,
            data_start,
            key_start,
            temp_index_start,
            buf,
        }
    }
}

impl<'a> SerializeMap for MapSerializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    fn serialize_key<K>(&mut self, key: &K) -> Result<(), Self::Error>
    where
        K: ?Sized + serde::Serialize,
    {
        key.serialize(NameSerializer(self.buf))?;
        self.len += 1;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let index = self.buf.values.len();
        value.serialize(Serializer(self.buf))?;
        self.buf.map_indexes.push(index);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        debug_assert_eq!(self.buf.keys.len() - self.key_start, self.len);
        debug_assert_eq!(self.buf.map_indexes.len() - self.temp_index_start, self.len);

        self.buf.permutations.reserve(self.len);
        for i in 0..self.len {
            self.buf.permutations.push(i);
        }
        self.buf.permutations.sort_unstable_by(|a, b| {
            self.buf.keys[self.key_start + a].cmp(&self.buf.keys[self.key_start + b])
        });
        self.buf.keys[self.key_start..].sort_unstable();

        let index_start = self.buf.indexes.len();
        let mut offset = 0;
        for src_i in 0..self.len {
            let dest_i = self.buf.permutations[src_i];
            let value_index = self.buf.map_indexes[self.temp_index_start + dest_i];
            self.buf.offsets.push(offset);
            self.buf.indexes.push(value_index);

            offset += self.buf.values[value_index].size(self.buf) as u64;
        }
        self.buf.permutations.clear();

        let header_start = self.buf.bytes.len();
        self.buf.bytes.write_u8(PSB_TYPE_OBJECT)?;
        write_uint_array(&mut self.buf.bytes, &self.buf.keys[self.key_start..])?;
        write_uint_array(&mut self.buf.bytes, &self.buf.offsets)?;
        let header_end = self.buf.bytes.len();

        self.buf.keys.drain(self.key_start..);
        self.buf.map_indexes.drain(self.temp_index_start..);
        self.buf.offsets.clear();

        let index = self.buf.objects.len();
        self.buf.objects.push(BufferObject {
            len: self.len,
            data_start: self.data_start,
            header_start,
            header_end,
            index_start,
        });

        self.buf.values[self.map_index] = BufferValue::Object { index };
        Ok(self.buf)
    }
}

impl SerializeStruct for MapSerializer<'_> {
    type Ok = <Self as SerializeMap>::Ok;
    type Error = <Self as SerializeMap>::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

impl SerializeStructVariant for MapSerializer<'_> {
    type Ok = <Self as SerializeMap>::Ok;
    type Error = <Self as SerializeMap>::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

struct NameSerializer<'a>(&'a mut Buffer);

impl<'a> serde::Serializer for NameSerializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let index = self.0.names.get_index_of(v).ok_or(Error::InvalidKey)?;
        self.0
            .keys
            .push(index.try_into().map_err(|_| Error::IndexOverflow)?);
        Ok(self.0)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::InvalidKey)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::InvalidKey)
    }
}
