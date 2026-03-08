#[derive(
    Debug, Clone, Copy, PartialEq, derive_more::From, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum PsbNumber {
    Integer(#[from] i64),
    Double(#[from] f64),
    Float(#[from] f32),
}
