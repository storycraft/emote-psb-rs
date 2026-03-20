//! PSB numeric value type.

/// A PSB numeric value, which may be a signed 64-bit integer, a 32-bit float, or a 64-bit double.
#[derive(Debug, Clone, Copy, PartialEq, derive_more::From)]
#[from(forward)]
pub enum PsbNumber {
    /// A signed 64-bit integer value.
    Integer(i64),
    /// A 64-bit double-precision floating-point value.
    Double(f64),
    /// A 32-bit single-precision floating-point value.
    Float(f32),
}
