pub mod psb;
pub mod value;

pub mod mdf;

/// psb file signature
pub const PSB_SIGNATURE: u32 = 0x425350;

/// compressed psb file signature
pub const PSB_MDF_SIGNATURE: u32 = 0x66646D;
