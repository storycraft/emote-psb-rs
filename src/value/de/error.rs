use core::fmt::Display;
use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
/// Error returned by the PSB deserializer.
pub enum Error {
    /// An unknown or unsupported PSB type tag was encountered.
    #[error("invalid psb value type: {0}")]
    InvalidValueType(u8),

    /// A value could not be interpreted (e.g. out-of-range index or invalid UTF-8).
    #[error("invalid psb value")]
    InvalidValue,

    /// An I/O error occurred while reading the stream.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// A custom error message produced by serde.
    #[error("{0}")]
    Message(String),
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
