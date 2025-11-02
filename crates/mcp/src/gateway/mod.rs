//! Gateway module for exposing local MCP server over Nostr

use crate::core::error::{Error, Result};
use cvm::{NostrServerTransport, NostrServerTransportConfig, NostrSigner};

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
        let transport = NostrServerTransport::new(signer, config).await.map_err(Error::from)?;

        Ok(Self { transport })
    }

    /// Announce the server to the relay
    pub async fn announce(&self) -> Result<()> {
        self.transport.announce().await.map_err(Error::from)
    }

    /// Publish tools list to the relay
    pub async fn publish_tools(&self, tools: Vec<serde_json::Value>) -> Result<()> {
        self.transport.publish_tools(tools).await.map_err(Error::from)
    }

    /// Start the gateway (also announces the server)
    pub async fn start(&self) -> Result<()> {
        // Announce server before starting to listen
        self.announce().await?;

        self.transport.start().await.map_err(Error::from)
    }

    /// Get reference to the transport
    pub fn transport(&self) -> &NostrServerTransport {
        &self.transport
    }
}
