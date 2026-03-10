mod error;

use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};
pub use error::Error;

use crate::value::{
    PSB_TYPE_DOUBLE, PSB_TYPE_FALSE, PSB_TYPE_FLOAT, PSB_TYPE_FLOAT0, PSB_TYPE_INTEGER_N,
    PSB_TYPE_NONE, PSB_TYPE_NULL, PSB_TYPE_TRUE,
    util::{get_n, get_uint_n, write_partial_int, write_partial_uint},
};

pub struct Serializer<T> {
    stream: T,
}

impl<T> serde::Serializer for &mut Serializer<T>
where
    T: Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            self.stream.write_u8(PSB_TYPE_TRUE)?;
        } else {
            self.stream.write_u8(PSB_TYPE_FALSE)?;
        }

        Ok(())
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
            return Ok(());
        }

        let n = get_n(v);
        self.stream.write_u8(PSB_TYPE_INTEGER_N + n)?;
        write_partial_int(&mut self.stream, v, n)?;
        Ok(())
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
            return Ok(());
        }

        let n = get_uint_n(v);
        self.stream.write_u8(PSB_TYPE_INTEGER_N + n)?;
        write_partial_uint(&mut self.stream, v, n)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if v == 0.0 {
            self.stream.write_u8(PSB_TYPE_FLOAT0)?;
            return Ok(());
        }

        self.stream.write_u8(PSB_TYPE_FLOAT)?;
        self.stream.write_f32::<LittleEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.stream.write_u8(PSB_TYPE_DOUBLE)?;
        self.stream.write_f64::<LittleEndian>(v)?;
        Ok(())
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
        self.stream.write_u8(PSB_TYPE_NULL)?;
        Ok(())
    }

    fn serialize_some<V>(self, value: &V) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.stream.write_u8(PSB_TYPE_NONE)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<V>(
        self,
        name: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<V>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}
