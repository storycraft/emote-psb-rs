use std::io;

use thiserror::Error;

use crate::value::error::PsbValueReadError;

#[derive(Debug, Error)]
pub enum PsbOpenError {
    #[error("invalid psb signature")]
    InvalidSignature,

    #[error("invalid names")]
    Names(#[source] PsbValueReadError),

    #[error("invalid strings")]
    Strings(#[source] PsbValueReadError),

    #[error("invalid resources")]
    Resources(#[source] PsbValueReadError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
