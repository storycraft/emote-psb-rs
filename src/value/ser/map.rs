use byteorder::WriteBytesExt;
use serde::ser::{Impossible, SerializeMap, SerializeStruct, SerializeStructVariant};

use crate::value::{
    PSB_TYPE_OBJECT,
    ser::{
        Error, Serializer, State,
        buffer::{BufferObject, BufferValue},
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
    type Ok = ();
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
    map_index_start: usize,
    state: State<'a>,
}

impl<'a> MapSerializer<'a> {
    pub fn new(state: State<'a>, len: Option<usize>) -> Self {
        if let Some(len) = len {
            state.buf.values.reserve(len + 1);
            state.ser.keys.reserve(len);
            state.ser.map_indexes.reserve(len);
        }

        let map_index = state.buf.values.len();
        state.buf.values.push(BufferValue::Invalid);
        let data_start = state.buf.bytes.len();
        let key_start = state.ser.keys.len();
        let map_index_start = state.ser.map_indexes.len();
        Self {
            len: 0,
            map_index,
            data_start,
            key_start,
            map_index_start,
            state,
        }
    }
}

impl<'a> SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<K>(&mut self, key: &K) -> Result<(), Self::Error>
    where
        K: ?Sized + serde::Serialize,
    {
        key.serialize(NameSerializer(self.state.reborrow_mut()))?;
        self.len += 1;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let index = self.state.buf.values.len();
        value.serialize(Serializer(self.state.reborrow_mut()))?;
        self.state.ser.map_indexes.push(index);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        debug_assert_eq!(self.state.ser.keys.len() - self.key_start, self.len);
        debug_assert_eq!(
            self.state.ser.map_indexes.len() - self.map_index_start,
            self.len
        );

        self.state.ser.permutations.reserve(self.len);
        for i in 0..self.len {
            self.state.ser.permutations.push(i);
        }
        self.state.ser.permutations.sort_unstable_by(|a, b| {
            self.state.ser.keys[self.key_start + a].cmp(&self.state.ser.keys[self.key_start + b])
        });
        self.state.ser.keys[self.key_start..].sort_unstable();

        self.state.ser.offsets.reserve(self.len);
        self.state.buf.indexes.reserve(self.len);
        let index_start = self.state.buf.indexes.len();
        let mut offset = 0;
        for src_i in 0..self.len {
            let dest_i = self.state.ser.permutations[src_i];
            let value_index = self.state.ser.map_indexes[self.map_index_start + dest_i];
            self.state.ser.offsets.push(offset);
            self.state.buf.indexes.push(value_index);

            offset += self.state.buf.values[value_index].size(self.state.buf) as u64;
        }
        self.state.ser.permutations.clear();

        let header_start = self.state.buf.bytes.len();
        self.state.buf.bytes.write_u8(PSB_TYPE_OBJECT)?;
        write_uint_array(
            &mut self.state.buf.bytes,
            &self.state.ser.keys[self.key_start..],
        )?;
        write_uint_array(&mut self.state.buf.bytes, &self.state.ser.offsets)?;
        let header_end = self.state.buf.bytes.len();

        self.state.ser.keys.drain(self.key_start..);
        self.state.ser.map_indexes.drain(self.map_index_start..);
        self.state.ser.offsets.clear();

        let index = self.state.buf.objects.len();
        self.state.buf.objects.push(BufferObject {
            len: self.len,
            data_start: self.data_start,
            header_start,
            header_end,
            index_start,
        });

        self.state.buf.values[self.map_index] = BufferValue::Object { index };
        Ok(())
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

struct NameSerializer<'a>(State<'a>);

impl<'a> serde::Serializer for NameSerializer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let index = self.0.buf.names.get_index_of(v).ok_or(Error::InvalidKey)?;
        self.0
            .ser
            .keys
            .push(index.try_into().map_err(|_| Error::IndexOverflow)?);
        Ok(())
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
