//! Context VM (CVM) - Nostr Context VM Implementation
//!
//! This crate implements the Context VM interface for bridging Nostr's Data Vending Machines (DVM)
//! with the Model Context Protocol (MCP).

/// Main error type for CVM operations
#[derive(Debug, thiserror::Error)]
pub enum CvmError {
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

pub type Result<T> = std::result::Result<T, CvmError>;
