//! MCP (Model Context Protocol) over ContextVM
//!
//! This crate provides a Rust implementation of agent-to-agent communication
//! using the ContextVM protocol, bridging Nostr and MCP.

pub mod config;
pub mod core;
pub mod encryption;
pub mod gateway;
pub mod proxy;
pub mod relay;
pub mod signer;
pub mod transport;

#[cfg(feature = "agent")]
pub mod ollama;

pub use core::{error::Error, types::*};
