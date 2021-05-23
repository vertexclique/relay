use thiserror::Error;
use std::io;

pub type Result<T> = std::result::Result<T, RelayError>;

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("Serialization Error")]
    Serde(#[from] serde_json::Error),
    #[error("Io Error")]
    IO(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}

