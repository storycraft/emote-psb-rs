mod buffer;
mod error;
mod map;
mod seq;
mod special;
mod string;
mod value;

pub use buffer::Buffer;
pub use error::Error;

use std::io::Write;

use serde::{Serialize, ser::SerializeSeq};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::value::{
    PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
    PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
    PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0, PSB_TYPE_INTEGER_N,
    PSB_TYPE_NULL, PSB_TYPE_RESOURCE_N, PSB_TYPE_STRING_N, PSB_TYPE_TRUE, PsbCompilerArray,
    PsbCompilerBinaryTree, PsbCompilerBool, PsbCompilerDecimal, PsbCompilerNumber,
    PsbCompilerResource, PsbCompilerString, PsbExtraResource, PsbResource,
    ser::{
        map::{MapSerializer, StructSerializer},
        seq::SeqSerializer,
        special::SpecialValueSerializer,
        string::StringCollector,
        value::{ref_type::RefTypeSerializer, unit::UnitTypeSerializer},
    },
    util::{get_n, get_uint_n},
};

pub fn serialize(value: &impl Serialize, buf: &mut Buffer) -> Result<(), Error> {
    value.serialize(StringCollector(buf))?;
    buf.names.sort_unstable();
    buf.strings.sort_unstable();
    value.serialize(Serializer(buf))?;
    Ok(())
}

struct Serializer<'a>(&'a mut Buffer);

impl<'a> serde::Serializer for Serializer<'a> {
    type Ok = &'a mut Buffer;
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = SeqSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = StructSerializer<'a>;
    type SerializeStructVariant = MapSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.0.write_value(|bytes| {
            Ok(bytes.write_u8(if v { PSB_TYPE_TRUE } else { PSB_TYPE_FALSE })?)
        })?;

        Ok(self.0)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as _)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as _)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as _)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        if v == 0 {
            self.0
                .write_value(|bytes| Ok(bytes.write_u8(PSB_TYPE_INTEGER_N)?))?;
            return Ok(self.0);
        }

        let n = get_n(v);
        self.0.write_value(|bytes| {
            bytes.write_u8(PSB_TYPE_INTEGER_N + n)?;
            bytes.write_all(&v.to_le_bytes()[..n as usize])?;
            Ok(())
        })?;
        Ok(self.0)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as _)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as _)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as _)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from_ne_bytes(v.to_ne_bytes()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if v == 0.0 {
            self.0
                .write_value(|bytes| Ok(bytes.write_u8(PSB_TYPE_FLOAT0)?))?;
            return Ok(self.0);
        }

        self.0.write_value(|bytes| {
            bytes.write_u8(PSB_TYPE_FLOAT)?;
            bytes.write_f32::<LittleEndian>(v)?;
            Ok(())
        })?;
        Ok(self.0)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.0.write_value(|bytes| {
            bytes.write_u8(PSB_TYPE_DOUBLE)?;
            bytes.write_f64::<LittleEndian>(v)?;
            Ok(())
        })?;
        Ok(self.0)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(v as _)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let index = self.0.strings.get_index_of(v).ok_or(Error::InvalidKey)?;
        let n = get_uint_n(index as _);
        if n > 4 {
            return Err(Error::IndexOverflow);
        }

        self.0.write_value(|bytes| {
            bytes.write_u8(PSB_TYPE_STRING_N + n)?;
            bytes.write_all(&index.to_le_bytes()[..n as usize])?;
            Ok(())
        })?;
        Ok(self.0)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for b in v {
            seq.serialize_element(b)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<V>(self, value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.0
            .write_value(|bytes| Ok(bytes.write_u8(PSB_TYPE_NULL)?))?;
        Ok(self.0)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<V>(
        self,
        _name: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<V>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer::new(self.0, len))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer::new(self.0, len))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        match name {
            PsbResource::MARKER => Ok(StructSerializer::RefTy(SpecialValueSerializer::new(
                name,
                RefTypeSerializer::new(name, PSB_TYPE_RESOURCE_N, self.0),
            ))),

            PsbExtraResource::MARKER => Ok(StructSerializer::RefTy(SpecialValueSerializer::new(
                name,
                RefTypeSerializer::new(name, PSB_TYPE_EXTRA_N, self.0),
            ))),

            PsbCompilerNumber::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_INTEGER, self.0),
            ))),

            PsbCompilerString::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_STRING, self.0),
            ))),

            PsbCompilerResource::MARKER => {
                Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                    name,
                    UnitTypeSerializer::new(name, PSB_COMPILER_RESOURCE, self.0),
                )))
            }

            PsbCompilerArray::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_ARRAY, self.0),
            ))),

            PsbCompilerDecimal::MARKER => {
                Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                    name,
                    UnitTypeSerializer::new(name, PSB_COMPILER_DECIMAL, self.0),
                )))
            }

            PsbCompilerBool::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_BOOL, self.0),
            ))),

            PsbCompilerBinaryTree::MARKER => {
                Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                    name,
                    UnitTypeSerializer::new(name, PSB_COMPILER_BINARY_TREE, self.0),
                )))
            }

            _ => Ok(StructSerializer::Map(self.serialize_map(Some(len))?)),
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}
