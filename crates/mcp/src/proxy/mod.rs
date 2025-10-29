//! Proxy module for accessing remote MCP servers via Nostr

use crate::core::error::Result;
use crate::core::types::McpMessage;
use crate::transport::client::{NostrClientTransport, NostrClientTransportConfig};
use crate::signer::NostrSigner;
use nostr_sdk::prelude::PublicKey;

/// Proxy for accessing remote Nostr-based MCP servers
pub struct Proxy {
    transport: NostrClientTransport,
}

impl Proxy {
    /// Create a new proxy
    pub async fn new(
        signer: impl NostrSigner + 'static,
        config: NostrClientTransportConfig,
    ) -> Result<Self> {
        let transport = NostrClientTransport::new(signer, config).await?;

        Ok(Self { transport })
    }

    /// Connect to relays
    pub async fn connect(&self) -> Result<()> {
        self.transport.connect().await
    }

    /// Send a request to a remote server
    pub async fn request(
        &self,
        server_pubkey: &PublicKey,
        request: McpMessage,
        use_encryption: bool,
    ) -> Result<McpMessage> {
        self.transport.send_request(server_pubkey, request, use_encryption).await
    }
}
