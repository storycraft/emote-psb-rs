#[derive(Debug, Clone, Copy, PartialEq, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PsbNumber {
    Integer(#[from] i64),
    Double(#[from] f64),
    Float(#[from] f32),
}
