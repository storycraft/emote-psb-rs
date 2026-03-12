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
    next_offset: usize,
    map_index: usize,
    data_start: usize,
    key_start: usize,
    offset_start: usize,
    buf: &'a mut Buffer,
}

impl<'a> MapSerializer<'a> {
    pub fn new(buf: &'a mut Buffer) -> Self {
        let map_index = buf.values.len();
        buf.values.push(BufferValue::Invalid);
        let data_start = buf.bytes.len();
        let key_start = buf.keys.len();
        let offset_start = buf.offsets.len();
        Self {
            len: 0,
            next_offset: 0,
            map_index,
            data_start,
            key_start,
            offset_start,
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

        let offset = self.next_offset;
        self.next_offset += self.buf.values[index].size(self.buf);
        self.buf.offsets.push(offset as u64);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let header_start = self.buf.bytes.len();
        self.buf.bytes.write_u8(PSB_TYPE_OBJECT)?;
        debug_assert_eq!(self.buf.keys.len() - self.key_start, self.len);
        write_uint_array(&mut self.buf.bytes, &self.buf.keys[self.key_start..])?;
        debug_assert_eq!(self.buf.offsets.len() - self.offset_start, self.len);
        write_uint_array(&mut self.buf.bytes, &self.buf.offsets[self.offset_start..])?;
        let header_end = self.buf.bytes.len();

        self.buf.keys.drain(self.key_start..);
        self.buf.offsets.drain(self.offset_start..);

        let index = self.buf.objects.len();
        self.buf.objects.push(BufferObject {
            len: self.len,
            header_offset: header_start - self.data_start,
            header_size: header_end - header_start,
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
