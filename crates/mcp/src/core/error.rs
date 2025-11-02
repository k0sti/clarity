//! Error types for MCP

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CVM error: {0}")]
    Cvm(#[from] cvm::Error),

    #[error("MCP protocol error: {0}")]
    Protocol(String),

    #[error("Invalid MCP message: {0}")]
    InvalidMessage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
