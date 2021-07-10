use thiserror::Error;
use std::io;
use isahc::http::method::InvalidMethod;

pub type Result<T> = std::result::Result<T, RelayError>;

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("Serialization Error")]
    Serde(#[from] serde_json::Error),
    #[error("Io Error")]
    IO(#[from] io::Error),
    #[error("Client Side Invalid Method")]
    ClientInvalidMethod(#[from] InvalidMethod),
    #[error("Internal Client Dispatch Error")]
    ClientDispatch(#[from] isahc::http::Error),
    #[error("HTTP Error")]
    Http(#[from] isahc::Error),
    #[error("JIT Instantiation Error")]
    JITInstantiation(#[from] wasmer::InstantiationError),
    #[error("JIT Compilation Error")]
    JITCompilation(#[from] wasmer::CompileError),
    #[error("Processor execution problem `{0}`")]
    Processor(String),
}