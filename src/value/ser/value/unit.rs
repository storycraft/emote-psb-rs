use std::io::Write;

use byteorder::WriteBytesExt;
use serde::ser::Impossible;

use crate::value::ser::{Error, Serializer};

pub struct UnitTypeSerializer<'a, T> {
    marker: &'static str,
    ty: u8,
    inner: &'a mut Serializer<T>,
}

impl<'a, T> UnitTypeSerializer<'a, T> {
    pub const fn new(marker: &'static str, ty: u8, inner: &'a mut Serializer<T>) -> Self {
        Self { marker, ty, inner }
    }
}

impl<T: Write> serde::Serializer for UnitTypeSerializer<'_, T> {
    type Ok = u64;
    type Error = Error;

    type SerializeSeq = Impossible<u64, Error>;
    type SerializeTuple = Impossible<u64, Error>;
    type SerializeTupleStruct = Impossible<u64, Error>;
    type SerializeTupleVariant = Impossible<u64, Error>;
    type SerializeMap = Impossible<u64, Error>;
    type SerializeStruct = Impossible<u64, Error>;
    type SerializeStructVariant = Impossible<u64, Error>;

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.stream.write_u8(self.ty)?;
        Ok(1)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_some<V>(self, _value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_newtype_variant<V>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::InvalidValue(self.marker))
    }
}
