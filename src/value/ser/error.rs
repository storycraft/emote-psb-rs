use core::fmt::Display;
use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("index exceed u32 limit")]
    IndexOverflow,

    #[error("only string key is valid for psb object")]
    InvalidKey,

    #[error("invalid psb specific value. marker: {0}")]
    InvalidValue(&'static str),

    #[error("{0}")]
    Message(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
