//! Core types for ContextVM protocol

use serde::{Deserialize, Serialize};

/// Encryption mode for transport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionMode {
    /// Encrypt messages if incoming message was encrypted
    Optional,
    /// Enforce encryption for all messages
    Required,
    /// Disable encryption entirely
    Disabled,
}

impl Default for EncryptionMode {
    fn default() -> Self {
        Self::Optional
    }
}

/// Server information for announcements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub picture: Option<String>,
    pub website: Option<String>,
    pub about: Option<String>,
}

/// Client session state
#[derive(Debug, Clone)]
pub struct ClientSession {
    pub client_pubkey: String,
    pub is_initialized: bool,
    pub is_encrypted: bool,
    pub last_activity: std::time::Instant,
}

impl ClientSession {
    pub fn new(client_pubkey: String, is_encrypted: bool) -> Self {
        Self {
            client_pubkey,
            is_initialized: false,
            is_encrypted,
            last_activity: std::time::Instant::now(),
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    pub fn mark_initialized(&mut self) {
        self.is_initialized = true;
    }
}
