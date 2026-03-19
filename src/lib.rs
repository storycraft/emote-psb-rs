//! # emote-psb
//!
//! Serde-based serialization and deserialization library for the E-mote PSB/MDF binary
//! data format.
//!
//! ## Overview
//!
//! E-mote PSB (`.psb`, `.scn`) is a proprietary binary format used by
//! E-mote animation data. MDF (`.mdf`) is a zlib-compressed wrapper around a
//! PSB file.
//!
//! ## Reading a PSB file
//!
//! ```no_run
//! use emote_psb::{psb::read::PsbFile, value::PsbValue};
//! use std::{fs::File, io::BufReader};
//!
//! let file = BufReader::new(File::open("sample.psb").unwrap());
//! let mut psb = PsbFile::open(file).unwrap();
//! let root: PsbValue = psb.deserialize_root().unwrap();
//! ```
//!
//! ## Writing a PSB file
//!
//! ```no_run
//! use emote_psb::{psb::write::PsbWriter, value::PsbValue};
//! use std::{fs::File, io::BufWriter};
//!
//! let root = PsbValue::Null;
//! let out = BufWriter::new(File::create("out.psb").unwrap());
//! let writer = PsbWriter::new(3, false, &root, out).unwrap();
//! writer.finish().unwrap();
//! ```

/// PSB/MDF reading and writing support.
pub mod psb;

/// PSB value types and serde serialization/deserialization.
pub mod value;

/// MDF (compressed PSB) reading and writing support.
pub mod mdf;

/// PSB file signature (`"PSB"` as a little-endian `u32`).
pub const PSB_SIGNATURE: u32 = 0x425350;

/// MDF (compressed PSB) file signature (`"mdf"` as a little-endian `u32`).
pub const PSB_MDF_SIGNATURE: u32 = 0x66646D;
