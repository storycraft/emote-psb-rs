use std::io;

use thiserror::Error;

use crate::value::{de, ser};

#[derive(Debug, Error)]
pub enum PsbOpenError {
    #[error("invalid psb signature")]
    InvalidSignature,

    #[error("invalid names")]
    Names(#[source] de::Error),

    #[error("invalid strings")]
    Strings(#[source] de::Error),

    #[error("invalid resources")]
    Resources(#[source] de::Error),

    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum PsbWriteError {
    #[error(transparent)]
    Serialize(#[from] ser::Error),

    #[error(transparent)]
    Io(#[from] io::Error),
}
