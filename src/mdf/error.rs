use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MdfOpenError {
    #[error("invalid mdf signature")]
    InvalidSignature,

    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum MdfCreateError {
    #[error("failed to write header")]
    Header(
        #[from]
        #[source]
        io::Error,
    ),
}
