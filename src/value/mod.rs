pub mod de;
pub mod number;
pub mod ser;

pub(crate) mod util;

use std::collections::HashMap;

use number::PsbNumber;
use smol_str::SmolStr;

pub const PSB_TYPE_NULL: u8 = 0x01;

pub const PSB_TYPE_FALSE: u8 = 0x02;
pub const PSB_TYPE_TRUE: u8 = 0x03;

/// 0 <= N <= 8
pub const PSB_TYPE_INTEGER_N: u8 = 0x04;
pub const PSB_TYPE_FLOAT0: u8 = 0x1d;
pub const PSB_TYPE_FLOAT: u8 = 0x1e;
pub const PSB_TYPE_DOUBLE: u8 = 0x1f;

/// 1 <= N <= 8
pub const PSB_TYPE_INTEGER_ARRAY_N: u8 = 0x0C;

/// 1 <= N <= 4
pub const PSB_TYPE_STRING_N: u8 = 0x14;

/// 1 <= N <= 4
pub const PSB_TYPE_RESOURCE_N: u8 = 0x18;

pub const PSB_TYPE_LIST: u8 = 0x20;
pub const PSB_TYPE_OBJECT: u8 = 0x21;

/// 1 <= N <= 4
pub const PSB_TYPE_EXTRA_N: u8 = 0x21;

pub const PSB_COMPILER_INTEGER: u8 = 0x80;
pub const PSB_COMPILER_STRING: u8 = 0x81;
pub const PSB_COMPILER_RESOURCE: u8 = 0x82;
pub const PSB_COMPILER_DECIMAL: u8 = 0x83;
pub const PSB_COMPILER_ARRAY: u8 = 0x84;
pub const PSB_COMPILER_BOOL: u8 = 0x85;
pub const PSB_COMPILER_BINARY_TREE: u8 = 0x86;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PsbValue {
    Null,
    Bool(bool),
    Number(PsbNumber),
    String(String),

    Resource(PsbResource),
    ExtraResource(PsbExtraResource),

    List(Vec<PsbValue>),
    Object(HashMap<SmolStr, PsbValue>),

    CompilerNumber(PsbCompilerNumber),
    CompilerString(PsbCompilerString),
    CompilerResource(PsbCompilerResource),
    CompilerDecimal(PsbCompilerDecimal),
    CompilerArray(PsbCompilerArray),
    CompilerBool(PsbCompilerBool),
    CompilerBinaryTree(PsbCompilerBinaryTree),
}

macro_rules! define_special_type {
    ($vis:vis $name:ident $(: $val:ty)? = $marker:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        $vis struct $name $((pub $val))?;

        #[allow(unused_parens)]
        const _: () = {
            impl $name {
                pub(crate) const MARKER: &str = $marker;
            }

            impl Copy for $name where for<'lt> ($($val)?): Copy {}

            $(
                impl From<$val> for $name {
                    fn from(v: $val) -> Self {
                        Self(v)
                    }
                }
            )?

            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(rename = $marker)]
            #[repr(transparent)]
            struct __Inner {
                #[serde(rename = $marker)]
                field: ($($val)?),
            }

            impl serde::Serialize for $name {
                fn serialize<S>(&self, __serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    // SAFETY: project transparent wrapper
                    let proj = unsafe { ::core::mem::transmute::<&Self, &__Inner>(self) };
                    proj.serialize(__serializer)
                }
            }

            impl<'de> serde::Deserialize<'de> for $name {
                fn deserialize<D>(__deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    Ok(Self $((__Inner::deserialize(__deserializer)?.field as $val))? )
                }
            }
        };
    };
}

define_special_type!(pub PsbResource: u32 = "__PSB@RESOURCE");
define_special_type!(pub PsbExtraResource: u32 = "__PSB@EXTRA@RESOURCE");

define_special_type!(pub PsbCompilerNumber = "__PSB@CP@NUMBER");
define_special_type!(pub PsbCompilerString = "__PSB@CP@STRING");
define_special_type!(pub PsbCompilerResource = "__PSB@CP@RESOURCE");
define_special_type!(pub PsbCompilerDecimal = "__PSB@CP@DECIMAL");
define_special_type!(pub PsbCompilerArray = "__PSB@CP@ARRAY");
define_special_type!(pub PsbCompilerBool = "__PSB@CP@BOOL");
define_special_type!(pub PsbCompilerBinaryTree = "__PSB@CP@BTREE");
