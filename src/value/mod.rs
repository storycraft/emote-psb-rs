pub mod binary_tree;
pub mod io;
pub mod number;
mod util;

use number::PsbNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PsbPrimitive {
    None,
    Null,
    Bool(bool),
    Number(PsbNumber),

    String(u32),
    Resource(u32),
    ExtraResource(u32),

    CompilerNumber,
    CompilerString,
    CompilerResource,
    CompilerDecimal,
    CompilerArray,
    CompilerBool,
    CompilerBinaryTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct PsbNameIndex(#[from] pub u64);
