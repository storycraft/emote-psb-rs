//! PSB numeric value type.

use serde::de::Unexpected;

/// A PSB numeric value, which may be a signed 64-bit integer, a 32-bit float, or a 64-bit double.
#[derive(Debug, Clone, Copy, PartialEq, derive_more::From, serde::Serialize)]
#[serde(untagged)]
pub enum PsbNumber {
    /// A signed 64-bit integer value.
    Integer(#[from] i64),
    /// A 64-bit double-precision floating-point value.
    Double(#[from] f64),
    /// A 32-bit single-precision floating-point value.
    Float(#[from] f32),
}

impl<'de> serde::Deserialize<'de> for PsbNumber {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PsbNumberVisitor;

        impl<'de> serde::de::Visitor<'de> for PsbNumberVisitor {
            type Value = PsbNumber;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a PSB number (integer, float, or double)")
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<PsbNumber, E> {
                Ok(PsbNumber::Integer(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PsbNumber::Integer(i64::try_from(v).map_err(|_| {
                    E::invalid_type(Unexpected::Unsigned(v), &self)
                })?))
            }

            fn visit_f32<E: serde::de::Error>(self, v: f32) -> Result<PsbNumber, E> {
                Ok(PsbNumber::Float(v))
            }

            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<PsbNumber, E> {
                Ok(PsbNumber::Double(v))
            }
        }

        deserializer.deserialize_any(PsbNumberVisitor)
    }
}
