//! Proxy module for accessing remote MCP servers via Nostr

use crate::core::error::{Error, Result};
use crate::core::types::McpMessage;
use cvm::{NostrClientTransport, NostrClientTransportConfig, NostrSigner, PublicKey};

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
        let transport = NostrClientTransport::new(signer, config).await.map_err(Error::from)?;

        Ok(Self { transport })
    }

    /// Connect to relays
    pub async fn connect(&self) -> Result<()> {
        self.transport.connect().await.map_err(Error::from)
    }

    /// Send a request to a remote server
    pub async fn request(
        &self,
        server_pubkey: &PublicKey,
        request: McpMessage,
        use_encryption: bool,
    ) -> Result<McpMessage> {
        // Convert McpMessage to JSON string
        let request_json = request.to_json()?;

        // Send via transport (which now works with JSON strings)
        let response_json = self.transport.send_request(server_pubkey, request_json, use_encryption)
            .await
            .map_err(Error::from)?;

        // Parse response back to McpMessage
        let response = McpMessage::from_json(&response_json)?;

        Ok(response)
    }
}
