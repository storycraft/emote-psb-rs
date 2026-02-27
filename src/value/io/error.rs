use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PsbValueReadError {
    #[error("invalid psb value type: {0}")]
    InvalidValueType(u8),

    #[error("invalid psb value")]
    InvalidValue,

    #[error(transparent)]
    Io(#[from] io::Error),
}
