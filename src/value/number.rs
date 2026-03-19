/// A PSB numeric value, which may be a signed 64-bit integer, a 32-bit float, or a 64-bit double.
#[derive(
    Debug, Clone, Copy, PartialEq, derive_more::From, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum PsbNumber {
    /// A signed 64-bit integer value.
    Integer(#[from] i64),
    /// A 64-bit double-precision floating-point value.
    Double(#[from] f64),
    /// A 32-bit single-precision floating-point value.
    Float(#[from] f32),
}
