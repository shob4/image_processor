use std::{array::TryFromSliceError, io::Error as ioErr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("problem reading file: {0}")]
    Io(#[from] ioErr),

    #[error("{0}")]
    CustomError(String),

    #[error("problem turning bytes into primitive: {0}")]
    TryFrom(#[from] TryFromSliceError),
}
