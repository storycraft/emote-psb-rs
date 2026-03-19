//! Error types for PSB reading and writing operations.

use std::io;

use thiserror::Error;

use crate::value::{de, ser};

#[derive(Debug, Error)]
/// Error returned when opening (reading) a PSB file fails.
pub enum PsbOpenError {
    /// The stream does not begin with the expected PSB signature bytes.
    #[error("invalid psb signature")]
    InvalidSignature,

    /// The name table embedded in the PSB file is malformed.
    #[error("invalid names")]
    Names(#[source] de::Error),

    /// The string table embedded in the PSB file is malformed.
    #[error("invalid strings")]
    Strings(#[source] de::Error),

    /// The resource table embedded in the PSB file is malformed.
    #[error("invalid resources")]
    Resources(#[source] de::Error),

    /// An I/O error occurred while reading the stream.
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
/// Error returned when writing a PSB file fails.
pub enum PsbWriteError {
    /// A value could not be serialized.
    #[error(transparent)]
    Serialize(#[from] ser::Error),

    /// An I/O error occurred while writing the stream.
    #[error(transparent)]
    Io(#[from] io::Error),
}
