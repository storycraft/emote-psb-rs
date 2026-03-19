use core::fmt::Display;
use std::io;

use thiserror::Error;

/// Error returned by the PSB serializer.
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occurred while writing the stream.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// A resource or string index exceeded the maximum representable value (`u32::MAX`).
    #[error("index exceed u32 limit")]
    IndexOverflow,

    /// A map key could not be serialized (only string keys are valid in PSB objects).
    #[error("invalid key for psb object")]
    InvalidKey,

    /// A PSB intrinsic value was given in an unexpected context.
    #[error("invalid psb specific value. marker: {0}")]
    InvalidValue(&'static str),

    /// A custom error message produced by serde.
    #[error("{0}")]
    Message(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
