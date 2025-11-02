//! MCP (Model Context Protocol) over ContextVM
//!
//! This crate provides a Rust implementation of agent-to-agent communication
//! using the ContextVM protocol, bridging Nostr and MCP.

pub mod config;
pub mod core;
pub mod gateway;
pub mod proxy;

#[cfg(feature = "agent")]
pub mod ollama;

// Re-export CVM types and modules
pub use cvm::{
    self,
    encryption, relay, signer, transport,
    EncryptionMode, ServerInfo, ClientSession,
    NostrClientTransport, NostrClientTransportConfig,
    NostrServerTransport, NostrServerTransportConfig,
    IncomingMessage,
    RelayPool,
    Keys, NostrSigner, PublicKey,
};

// Export MCP-specific types
pub use core::{Error, Result, types::*};
