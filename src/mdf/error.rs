//! Error types for MDF reading and writing operations.

use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
/// Error returned when opening (reading) an MDF file fails.
pub enum MdfOpenError {
    /// The stream does not begin with the expected MDF signature bytes.
    #[error("invalid mdf signature")]
    InvalidSignature,

    /// An I/O error occurred while reading the stream.
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
/// Error returned when creating (writing) an MDF file fails.
pub enum MdfCreateError {
    /// An I/O error occurred while writing the MDF header.
    #[error("failed to write header")]
    Header(
        #[from]
        #[source]
        io::Error,
    ),
}
