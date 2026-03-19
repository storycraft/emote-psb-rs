/// Serde deserializer for PSB binary data.
pub mod de;

/// PSB numeric value type.
pub mod number;

/// Serde serializer for PSB binary data.
pub mod ser;

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
    String(String),

    /// A resource index
    Resource(PsbResource),
    /// A extra resource index
    ExtraResource(PsbExtraResource),

    /// List of values
    List(Vec<PsbValue>),

    /// Map of values
    Object(HashMap<SmolStr, PsbValue>),

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
