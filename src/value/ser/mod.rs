mod error;
mod map;
mod seq;
mod special;
mod value;

pub use error::Error;

use std::io::{Seek, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::value::{
    PSB_COMPILER_ARRAY, PSB_COMPILER_BINARY_TREE, PSB_COMPILER_BOOL, PSB_COMPILER_DECIMAL,
    PSB_COMPILER_INTEGER, PSB_COMPILER_RESOURCE, PSB_COMPILER_STRING, PSB_TYPE_DOUBLE,
    PSB_TYPE_EXTRA_N, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0, PSB_TYPE_INTEGER_N,
    PSB_TYPE_NULL, PSB_TYPE_RESOURCE_N, PSB_TYPE_TRUE, PsbCompilerArray, PsbCompilerBinaryTree,
    PsbCompilerBool, PsbCompilerDecimal, PsbCompilerNumber, PsbCompilerResource, PsbCompilerString,
    PsbExtraResource, PsbResource, PsbUIntArray,
    ser::{
        map::{MapSerializer, StructSerializer},
        seq::SeqSerializer,
        special::SpecialValueSerializer,
        value::{ref_type::RefTypeSerializer, unit::UnitTypeSerializer},
    },
    util::{get_n, get_uint_n, write_partial_int, write_partial_uint},
};

pub struct Serializer<T> {
    stream: T,
    names_buf: Vec<u64>,
    offsets_buf: Vec<u64>,
}

impl<'a, T> serde::Serializer for &'a mut Serializer<T>
where
    T: Write + Seek,
{
    type Ok = u64;
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, T>;
    type SerializeTuple = SeqSerializer<'a, T>;
    type SerializeTupleStruct = SeqSerializer<'a, T>;
    type SerializeTupleVariant = SeqSerializer<'a, T>;
    type SerializeMap = MapSerializer<'a, T>;
    type SerializeStruct = StructSerializer<'a, T>;
    type SerializeStructVariant = MapSerializer<'a, T>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            self.stream.write_u8(PSB_TYPE_TRUE)?;
        } else {
            self.stream.write_u8(PSB_TYPE_FALSE)?;
        }

        Ok(1)
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
            self.stream.write_u8(PSB_TYPE_INTEGER_N)?;
            return Ok(1);
        }

        let n = get_n(v);
        self.stream.write_u8(PSB_TYPE_INTEGER_N + n)?;
        write_partial_int(&mut self.stream, v, n)?;
        Ok(1 + n as u64)
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
        if v == 0 {
            self.stream.write_u8(PSB_TYPE_INTEGER_N)?;
            return Ok(1);
        }

        let n = get_uint_n(v);
        self.stream.write_u8(PSB_TYPE_INTEGER_N + n)?;
        write_partial_uint(&mut self.stream, v, n)?;
        Ok(1 + n as u64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if v == 0.0 {
            self.stream.write_u8(PSB_TYPE_FLOAT0)?;
            return Ok(1);
        }

        self.stream.write_u8(PSB_TYPE_FLOAT)?;
        self.stream.write_f32::<LittleEndian>(v)?;
        Ok(5)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.stream.write_u8(PSB_TYPE_DOUBLE)?;
        self.stream.write_f64::<LittleEndian>(v)?;
        Ok(9)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(v as _)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
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
        self.stream.write_u8(PSB_TYPE_NULL)?;
        Ok(1)
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

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer(self))
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

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // TODO
        Ok(MapSerializer(self))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        match name {
            PsbUIntArray::MARKER => {
                todo!()
            }

            PsbResource::MARKER => Ok(StructSerializer::RefTy(SpecialValueSerializer::new(
                name,
                RefTypeSerializer::new(name, PSB_TYPE_RESOURCE_N, self),
            ))),

            PsbExtraResource::MARKER => Ok(StructSerializer::RefTy(SpecialValueSerializer::new(
                name,
                RefTypeSerializer::new(name, PSB_TYPE_EXTRA_N, self),
            ))),

            PsbCompilerNumber::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_INTEGER, self),
            ))),

            PsbCompilerString::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_STRING, self),
            ))),

            PsbCompilerResource::MARKER => {
                Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                    name,
                    UnitTypeSerializer::new(name, PSB_COMPILER_RESOURCE, self),
                )))
            }

            PsbCompilerArray::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_ARRAY, self),
            ))),

            PsbCompilerDecimal::MARKER => {
                Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                    name,
                    UnitTypeSerializer::new(name, PSB_COMPILER_DECIMAL, self),
                )))
            }

            PsbCompilerBool::MARKER => Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                name,
                UnitTypeSerializer::new(name, PSB_COMPILER_BOOL, self),
            ))),

            PsbCompilerBinaryTree::MARKER => {
                Ok(StructSerializer::UnitTy(SpecialValueSerializer::new(
                    name,
                    UnitTypeSerializer::new(name, PSB_COMPILER_BINARY_TREE, self),
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
