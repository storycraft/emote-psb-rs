#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct PsbString(#[from] pub u64);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct PsbNameIndex(#[from] pub u64);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PsbResource(#[from] pub u64);
