use core::fmt::Display;
use std::io;

use serde::de;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid psb value type: {0}")]
    InvalidValueType(u8),

    #[error("invalid psb value")]
    InvalidValue,

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("{0}")]
    Message(String),
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
