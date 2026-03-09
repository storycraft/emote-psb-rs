use std::io;

use thiserror::Error;

use crate::value::de;

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
