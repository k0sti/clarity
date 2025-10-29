//! Gateway module for exposing local MCP server over Nostr

use crate::core::error::Result;
use crate::transport::server::{NostrServerTransport, NostrServerTransportConfig};
use crate::signer::NostrSigner;

/// Gateway that bridges local MCP server to Nostr network
pub struct Gateway {
    transport: NostrServerTransport,
}

impl Gateway {
    /// Create a new gateway
    pub async fn new(
        signer: impl NostrSigner + 'static,
        config: NostrServerTransportConfig,
    ) -> Result<Self> {
        let transport = NostrServerTransport::new(signer, config).await?;

        Ok(Self { transport })
    }

    /// Announce the server to the relay
    pub async fn announce(&self) -> Result<()> {
        self.transport.announce().await
    }

    /// Publish tools list to the relay
    pub async fn publish_tools(&self, tools: Vec<serde_json::Value>) -> Result<()> {
        self.transport.publish_tools(tools).await
    }

    /// Start the gateway (also announces the server)
    pub async fn start(&self) -> Result<()> {
        // Announce server before starting to listen
        self.announce().await?;

        self.transport.start().await
    }

    /// Get reference to the transport
    pub fn transport(&self) -> &NostrServerTransport {
        &self.transport
    }
}
