#[derive(Debug, Clone, Copy, PartialEq, derive_more::From, serde::Serialize)]
#[serde(untagged)]
pub enum PsbNumber {
    Integer(#[from] i64),
    Double(#[from] f64),
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
