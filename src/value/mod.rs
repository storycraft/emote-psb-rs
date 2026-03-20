//! PSB value types and serde serialization/deserialization.

pub mod de;
pub mod number;
pub mod ser;

mod impls;
pub(crate) mod util;

use std::collections::HashMap;

use number::PsbNumber;
use smol_str::SmolStr;

/// PSB type tag: null / unit value.
pub const PSB_TYPE_NULL: u8 = 0x01;

/// PSB type tag: boolean `false`.
pub const PSB_TYPE_FALSE: u8 = 0x02;
/// PSB type tag: boolean `true`.
pub const PSB_TYPE_TRUE: u8 = 0x03;

/// PSB type tag base for variable-width signed integers (N bytes follow, 0 ≤ N ≤ 8).
pub const PSB_TYPE_INTEGER_N: u8 = 0x04;
/// PSB type tag: 32-bit float zero (`0.0f32`).
pub const PSB_TYPE_FLOAT0: u8 = 0x1d;
/// PSB type tag: 32-bit float (4 bytes follow).
pub const PSB_TYPE_FLOAT: u8 = 0x1e;
/// PSB type tag: 64-bit double (8 bytes follow).
pub const PSB_TYPE_DOUBLE: u8 = 0x1f;

/// PSB type tag base for packed integer arrays (1 ≤ N ≤ 8 bytes per element).
pub const PSB_TYPE_INTEGER_ARRAY_N: u8 = 0x0C;

/// PSB type tag base for string references (1 ≤ N ≤ 4 bytes of index).
pub const PSB_TYPE_STRING_N: u8 = 0x14;

/// PSB type tag base for resource references (1 ≤ N ≤ 4 bytes of index).
pub const PSB_TYPE_RESOURCE_N: u8 = 0x18;

/// PSB type tag: ordered list of values.
pub const PSB_TYPE_LIST: u8 = 0x20;
/// PSB type tag: keyed object (map of string → value).
pub const PSB_TYPE_OBJECT: u8 = 0x21;

/// PSB type tag base for extra resource references (1 ≤ N ≤ 4 bytes of index).
pub const PSB_TYPE_EXTRA_N: u8 = 0x21;

/// PSB compiler directive tag: integer placeholder.
pub const PSB_COMPILER_INTEGER: u8 = 0x80;
/// PSB compiler directive tag: string placeholder.
pub const PSB_COMPILER_STRING: u8 = 0x81;
/// PSB compiler directive tag: resource placeholder.
pub const PSB_COMPILER_RESOURCE: u8 = 0x82;
/// PSB compiler directive tag: decimal placeholder.
pub const PSB_COMPILER_DECIMAL: u8 = 0x83;
/// PSB compiler directive tag: array placeholder.
pub const PSB_COMPILER_ARRAY: u8 = 0x84;
/// PSB compiler directive tag: boolean placeholder.
pub const PSB_COMPILER_BOOL: u8 = 0x85;
/// PSB compiler directive tag: binary-tree placeholder.
pub const PSB_COMPILER_BINARY_TREE: u8 = 0x86;

/// Variants of psb data type
#[derive(Debug, Clone, PartialEq, derive_more::From)]
#[from(forward)]
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
    #[from(skip)]
    Resource(u32),
    /// A extra resource index
    #[from(skip)]
    ExtraResource(u32),

    /// List of values
    List(Vec<PsbValue>),

    /// PSB intrinsic type: [`PsbCompilerNumber`]
    CompilerNumber,
    /// PSB intrinsic type: [`PsbCompilerString`]
    CompilerString,
    /// PSB intrinsic type: [`PsbCompilerResource`]
    CompilerResource,
    /// PSB intrinsic type: [`PsbCompilerDecimal`]
    CompilerDecimal,
    /// PSB intrinsic type: [`PsbCompilerArray`]
    CompilerArray,
    /// PSB intrinsic type: [`PsbCompilerBool`]
    CompilerBool,
    /// PSB intrinsic type: [`PsbCompilerBinaryTree`]
    CompilerBinaryTree,

    /// Map of values
    Object(HashMap<SmolStr, PsbValue>),
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
                    let __inner = __Inner::deserialize(__deserializer)?;
                    Ok(Self $((  __inner.field as $val ))? )
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
