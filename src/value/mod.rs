pub mod de;
pub mod number;
pub mod ser;

pub(crate) mod util;

use indexmap::IndexMap;
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
/// Variants of psb data type
pub enum PsbValue {
    /// An empty or null type
    Null,
    /// A bool value
    Bool(bool),
    /// A numberic value
    Number(PsbNumber),
    /// A string value
    String(SmolStr),

    /// A resource index
    Resource(PsbResource),
    /// A extra resource index
    ExtraResource(PsbExtraResource),

    /// List of values
    List(Vec<PsbValue>),

    /// Map of values
    ///
    /// The order needs to be preserved. Use data types which preserves order.
    Object(IndexMap<SmolStr, PsbValue>),

    /// PSB intrinsic type
    CompilerNumber(PsbCompilerNumber),
    /// PSB intrinsic type
    CompilerString(PsbCompilerString),
    /// PSB intrinsic type
    CompilerResource(PsbCompilerResource),
    /// PSB intrinsic type
    CompilerDecimal(PsbCompilerDecimal),
    /// PSB intrinsic type
    CompilerArray(PsbCompilerArray),
    /// PSB intrinsic type
    CompilerBool(PsbCompilerBool),
    /// PSB intrinsic type
    CompilerBinaryTree(PsbCompilerBinaryTree),
}

macro_rules! define_special_type {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident $(: $val:ty)? = $marker:literal
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        $(#[$meta])*
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

define_special_type!(
    /// PSB intrinsic marker
    pub PsbResource: u32 = "__PSB@RESOURCE"
);
define_special_type!(
    /// PSB intrinsic marker
    pub PsbExtraResource: u32 = "__PSB@EXTRA@RESOURCE"
);

define_special_type!(
    /// PSB intrinsic marker
    pub PsbCompilerNumber = "__PSB@CP@NUMBER"
);
define_special_type!(
    /// PSB intrinsic marker
    pub PsbCompilerString = "__PSB@CP@STRING"
);
define_special_type!(
    /// PSB intrinsic marker
    pub PsbCompilerResource = "__PSB@CP@RESOURCE"
);
define_special_type!(
    /// PSB intrinsic marker
    pub PsbCompilerDecimal = "__PSB@CP@DECIMAL"
);
define_special_type!(
    /// PSB intrinsic marker
    pub PsbCompilerArray = "__PSB@CP@ARRAY"
);
define_special_type!(
    /// PSB intrinsic marker
    pub PsbCompilerBool = "__PSB@CP@BOOL"
);
define_special_type!(
    /// PSB intrinsic marker
    pub PsbCompilerBinaryTree = "__PSB@CP@BTREE"
);
