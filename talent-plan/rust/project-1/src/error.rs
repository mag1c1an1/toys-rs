use std::io;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum KvsError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("Key not found")]
    KeyNotFound,
    #[error("unexpected command type")]
    UnexpectedCommandType,
}


pub type Result<T, E = KvsError> = std::result::Result<T, E>;
