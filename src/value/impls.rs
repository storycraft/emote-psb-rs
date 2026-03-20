use std::collections::HashMap;

use serde::{
    Deserialize, Serialize,
    de::{DeserializeSeed, Unexpected, Visitor},
};
use smol_str::SmolStr;

use crate::value::*;

impl From<i64> for PsbValue {
    fn from(v: i64) -> Self {
        Self::Number(PsbNumber::from(v))
    }
}

impl From<f32> for PsbValue {
    fn from(v: f32) -> Self {
        Self::Number(PsbNumber::from(v))
    }
}

impl From<f64> for PsbValue {
    fn from(v: f64) -> Self {
        Self::Number(PsbNumber::from(v))
    }
}

impl From<&str> for PsbValue {
    fn from(v: &str) -> Self {
        Self::String(v.into())
    }
}

impl From<String> for PsbValue {
    fn from(v: String) -> Self {
        Self::String(v.into())
    }
}

impl From<PsbResource> for PsbValue {
    fn from(v: PsbResource) -> Self {
        Self::Resource(v.0)
    }
}

impl From<PsbExtraResource> for PsbValue {
    fn from(v: PsbExtraResource) -> Self {
        Self::ExtraResource(v.0)
    }
}

impl From<PsbCompilerNumber> for PsbValue {
    fn from(_: PsbCompilerNumber) -> Self {
        Self::CompilerNumber
    }
}

impl From<PsbCompilerString> for PsbValue {
    fn from(_: PsbCompilerString) -> Self {
        Self::CompilerString
    }
}

impl From<PsbCompilerResource> for PsbValue {
    fn from(_: PsbCompilerResource) -> Self {
        Self::CompilerResource
    }
}

impl From<PsbCompilerDecimal> for PsbValue {
    fn from(_: PsbCompilerDecimal) -> Self {
        Self::CompilerDecimal
    }
}

impl From<PsbCompilerArray> for PsbValue {
    fn from(_: PsbCompilerArray) -> Self {
        Self::CompilerArray
    }
}

impl From<PsbCompilerBool> for PsbValue {
    fn from(_: PsbCompilerBool) -> Self {
        Self::CompilerBool
    }
}

impl From<PsbCompilerBinaryTree> for PsbValue {
    fn from(_: PsbCompilerBinaryTree) -> Self {
        Self::CompilerBinaryTree
    }
}

impl Serialize for PsbValue {
    #[inline]
    fn serialize<S>(&self, se: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            PsbValue::Null => se.serialize_none(),
            PsbValue::Bool(v) => se.serialize_bool(v),
            PsbValue::Number(v) => v.serialize(se),
            PsbValue::String(ref v) => v.serialize(se),
            PsbValue::Resource(v) => PsbResource(v).serialize(se),
            PsbValue::ExtraResource(v) => PsbExtraResource(v).serialize(se),
            PsbValue::List(ref v) => v.serialize(se),
            PsbValue::CompilerNumber => PsbCompilerNumber.serialize(se),
            PsbValue::CompilerString => PsbCompilerString.serialize(se),
            PsbValue::CompilerResource => PsbCompilerResource.serialize(se),
            PsbValue::CompilerDecimal => PsbCompilerDecimal.serialize(se),
            PsbValue::CompilerArray => PsbCompilerArray.serialize(se),
            PsbValue::CompilerBool => PsbCompilerBool.serialize(se),
            PsbValue::CompilerBinaryTree => PsbCompilerBinaryTree.serialize(se),
            PsbValue::Object(ref v) => v.serialize(se),
        }
    }
}

impl<'de> Deserialize<'de> for PsbValue {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = PsbValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a PSB value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::Bool(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::Number(NumberVisitor.visit_i64(v)?))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::Number(NumberVisitor.visit_u64(v)?))
            }

            #[inline]
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::Number(NumberVisitor.visit_f32(v)?))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::Number(NumberVisitor.visit_f64(v)?))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::String(SmolStr::from(v)))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::String(SmolStr::from(v)))
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());
                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }

                Ok(PsbValue::List(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                match map.next_key_seed(KeyClassifier)? {
                    Some(KeyClass::Resource) => Ok(PsbValue::Resource(map.next_value()?)),
                    Some(KeyClass::ExtraResource) => Ok(PsbValue::ExtraResource(map.next_value()?)),

                    Some(KeyClass::CompilerNumber) => Ok(PsbValue::CompilerNumber),
                    Some(KeyClass::CompilerString) => Ok(PsbValue::CompilerString),
                    Some(KeyClass::CompilerResource) => Ok(PsbValue::CompilerResource),
                    Some(KeyClass::CompilerDecimal) => Ok(PsbValue::CompilerDecimal),
                    Some(KeyClass::CompilerArray) => Ok(PsbValue::CompilerArray),
                    Some(KeyClass::CompilerBool) => Ok(PsbValue::CompilerBool),
                    Some(KeyClass::CompilerBinaryTree) => Ok(PsbValue::CompilerBinaryTree),

                    Some(KeyClass::Object(first_key)) => {
                        let mut object =
                            HashMap::with_capacity(map.size_hint().unwrap_or_default());
                        object.insert(first_key, map.next_value()?);
                        while let Some((key, value)) = map.next_entry()? {
                            object.insert(key, value);
                        }

                        Ok(PsbValue::Object(object))
                    }
                    None => Ok(PsbValue::Object(HashMap::new())),
                }
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbValue::Null)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_unit()
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                PsbValue::deserialize(deserializer)
            }
        }

        de.deserialize_any(ValueVisitor)
    }
}

impl Serialize for PsbNumber {
    #[inline]
    fn serialize<S>(&self, se: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PsbNumber::Integer(v) => se.serialize_i64(*v),
            PsbNumber::Double(v) => se.serialize_f64(*v),
            PsbNumber::Float(v) => se.serialize_f32(*v),
        }
    }
}

impl<'de> Deserialize<'de> for PsbNumber {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NumberVisitor)
    }
}

struct NumberVisitor;

impl<'de> Visitor<'de> for NumberVisitor {
    type Value = PsbNumber;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a PSB number (integer, float, or double)")
    }

    #[inline]
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<PsbNumber, E> {
        Ok(PsbNumber::Integer(v))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PsbNumber::Integer(i64::try_from(v).map_err(|_| {
            E::invalid_type(Unexpected::Unsigned(v), &self)
        })?))
    }

    #[inline]
    fn visit_f32<E: serde::de::Error>(self, v: f32) -> Result<PsbNumber, E> {
        Ok(PsbNumber::Float(v))
    }

    #[inline]
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<PsbNumber, E> {
        Ok(PsbNumber::Double(v))
    }
}

struct KeyClassifier;

impl<'de> DeserializeSeed<'de> for KeyClassifier {
    type Value = KeyClass;

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyClassifier {
    type Value = KeyClass;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match v {
            PsbResource::MARKER => KeyClass::Resource,
            PsbExtraResource::MARKER => KeyClass::ExtraResource,
            PsbCompilerNumber::MARKER => KeyClass::CompilerNumber,
            PsbCompilerString::MARKER => KeyClass::CompilerString,
            PsbCompilerResource::MARKER => KeyClass::CompilerResource,
            PsbCompilerDecimal::MARKER => KeyClass::CompilerDecimal,
            PsbCompilerArray::MARKER => KeyClass::CompilerArray,
            PsbCompilerBool::MARKER => KeyClass::CompilerBool,
            PsbCompilerBinaryTree::MARKER => KeyClass::CompilerBinaryTree,
            v => KeyClass::Object(SmolStr::new(v)),
        })
    }
}

enum KeyClass {
    Resource,
    ExtraResource,
    CompilerNumber,
    CompilerString,
    CompilerResource,
    CompilerDecimal,
    CompilerArray,
    CompilerBool,
    CompilerBinaryTree,
    Object(SmolStr),
}
