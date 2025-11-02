//! Context VM (CVM) - Nostr Context VM Implementation
//!
//! This crate implements the Context VM interface for bridging Nostr's Data Vending Machines (DVM)
//! with the Model Context Protocol (MCP).
//!
//! The CVM provides the low-level Nostr transport layer that MCP and other protocols can build upon.

pub mod core;
pub mod transport;
pub mod relay;
pub mod signer;
pub mod encryption;

// Re-export commonly used types
pub use core::{
    constants, error, types,
    error::{Error, Result},
    types::{EncryptionMode, ServerInfo, ClientSession},
};

pub use transport::client::{NostrClientTransport, NostrClientTransportConfig};
pub use transport::server::{NostrServerTransport, NostrServerTransportConfig, IncomingMessage};

pub use relay::RelayPool;
pub use signer::{Keys, NostrSigner, PublicKey, from_sk, generate};
