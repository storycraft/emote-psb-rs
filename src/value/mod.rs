pub mod io;
pub mod number;
mod util;

use number::PsbNumber;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
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
