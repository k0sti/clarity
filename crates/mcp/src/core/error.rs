//! Error types for ContextVM

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Nostr error: {0}")]
    Nostr(#[from] nostr_sdk::client::Error),

    #[error("Nostr event error: {0}")]
    NostrEvent(#[from] nostr_sdk::event::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid event kind: {0}")]
    InvalidEventKind(u16),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Encryption required but not supported")]
    EncryptionRequired,

    #[error("Timeout")]
    Timeout,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
