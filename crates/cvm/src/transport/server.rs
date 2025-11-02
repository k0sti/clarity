//! Server-side Nostr transport for ContextVM

use crate::core::{
    constants::*, error::{Error, Result}, types::*,
};
use crate::relay::RelayPool;
use nostr_sdk::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Incoming message metadata
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub content: String,
    pub sender_pubkey: PublicKey,
    pub event_id: EventId,
    pub is_encrypted: bool,
}

/// Server-side transport configuration
pub struct NostrServerTransportConfig {
    pub relay_urls: Vec<String>,
    pub encryption_mode: EncryptionMode,
    pub server_info: Option<ServerInfo>,
    pub session_timeout: Duration,
}

impl Default for NostrServerTransportConfig {
    fn default() -> Self {
        Self {
            relay_urls: vec!["wss://relay.damus.io".to_string()],
            encryption_mode: EncryptionMode::Optional,
            server_info: None,
            session_timeout: Duration::from_secs(300),
        }
    }
}

/// Server-side Nostr transport
pub struct NostrServerTransport {
    relay_pool: Arc<RelayPool>,
    config: NostrServerTransportConfig,
    sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
}

impl NostrServerTransport {
    /// Create a new server transport
    pub async fn new<T>(
        signer: T,
        config: NostrServerTransportConfig,
    ) -> Result<Self>
    where
        T: IntoNostrSigner,
    {
        let relay_pool = Arc::new(RelayPool::new(signer).await?);

        Ok(Self {
            relay_pool,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Announce server to the relay
    pub async fn announce(&self) -> Result<()> {
        // Connect to relays first if not already connected
        self.relay_pool.connect(&self.config.relay_urls).await?;

        let client = self.relay_pool.client();

        // Build server announcement
        let server_info = self.config.server_info.as_ref().ok_or_else(|| {
            Error::Other("Server info not configured for announcement".to_string())
        })?;

        let announcement = serde_json::json!({
            "name": server_info.name,
            "version": server_info.version,
            "about": server_info.about,
        });

        let announcement_json = serde_json::to_string(&announcement)
            .map_err(|e| Error::Other(format!("Failed to serialize announcement: {}", e)))?;

        // Publish as kind 11316 (server announcement)
        let builder = EventBuilder::new(Kind::from(SERVER_ANNOUNCEMENT_KIND), announcement_json);

        let output = client.send_event_builder(builder).await
            .map_err(|e| Error::Transport(e.to_string()))?;

        tracing::info!("Published server announcement: {}", output.val);

        Ok(())
    }

    /// Publish tools list to the relay
    pub async fn publish_tools(&self, tools: Vec<serde_json::Value>) -> Result<()> {
        // Connect to relays first if not already connected
        self.relay_pool.connect(&self.config.relay_urls).await?;

        let client = self.relay_pool.client();

        // Build tools list in MCP format
        let tools_list = serde_json::json!({
            "tools": tools
        });

        let tools_json = serde_json::to_string(&tools_list)
            .map_err(|e| Error::Other(format!("Failed to serialize tools: {}", e)))?;

        // Publish as kind 11317 (tools list)
        let builder = EventBuilder::new(Kind::from(TOOLS_LIST_KIND), tools_json);

        let output = client.send_event_builder(builder).await
            .map_err(|e| Error::Transport(e.to_string()))?;

        tracing::info!("Published tools list ({} tools): {}", tools.len(), output.val);

        Ok(())
    }

    /// Start listening for incoming MCP requests
    pub async fn start(&self) -> Result<()> {
        // Connect to relays
        self.relay_pool.connect(&self.config.relay_urls).await?;

        let client = self.relay_pool.client();
        let pubkey = client.signer().await.map_err(|e| Error::Other(e.to_string()))?
            .get_public_key().await.map_err(|e| Error::Other(e.to_string()))?;

        // Subscribe to messages targeting this server (both regular and encrypted)
        let filter = Filter::new()
            .kinds(vec![
                Kind::from(CTXVM_MESSAGES_KIND),
                Kind::from(GIFT_WRAP_KIND),
            ])
            .pubkey(pubkey);

        tracing::info!("Server listening on pubkey: {}", pubkey.to_hex());

        // Subscribe
        client
            .subscribe(filter, None)
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;

        // Handle events in a loop
        self.handle_subscription().await
    }

    async fn handle_subscription(&self) -> Result<()> {
        let client = self.relay_pool.client();
        let mut notifications = client.notifications();

        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if let Err(e) = self.handle_event(*event).await {
                    tracing::error!("Error handling event: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn handle_event(&self, event: Event) -> Result<()> {
        // Check if it's a gift-wrapped event
        let (actual_event, is_encrypted) = if event.kind == Kind::from(GIFT_WRAP_KIND) {
            let client = self.relay_pool.client();
            let unwrapped = client
                .unwrap_gift_wrap(&event)
                .await
                .map_err(|e| Error::Decryption(e.to_string()))?;
            (unwrapped.rumor, true)
        } else {
            // Convert Event to UnsignedEvent for consistency
            let rumor = UnsignedEvent::new(
                event.pubkey,
                event.created_at,
                event.kind,
                event.tags.clone(),
                event.content.clone(),
            );
            (rumor, false)
        };

        // Get or create session
        let client_pubkey = actual_event.pubkey.to_hex();
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .entry(client_pubkey.clone())
            .or_insert_with(|| ClientSession::new(client_pubkey, is_encrypted));

        session.update_activity();

        // Log received message (actual processing should be done by higher-level code)
        tracing::debug!("Received message from {}: {}", actual_event.pubkey.to_hex(), actual_event.content);

        Ok(())
    }

    /// Send a response to a client
    pub async fn send_response(
        &self,
        client_pubkey: &PublicKey,
        response_json: String,
        request_event_id: &EventId,
        use_encryption: bool,
    ) -> Result<EventId> {
        let client = self.relay_pool.client();

        let builder = EventBuilder::new(Kind::from(CTXVM_MESSAGES_KIND), response_json)
            .tag(Tag::public_key(*client_pubkey))
            .tag(Tag::event(*request_event_id));

        let final_event_id = if use_encryption {
            // Use client to send gift-wrapped
            let event = client.sign_event_builder(builder).await
                .map_err(|e| Error::Other(e.to_string()))?;
            // Convert to UnsignedEvent for gift wrapping
            let rumor = UnsignedEvent::new(
                event.pubkey,
                event.created_at,
                event.kind,
                event.tags.clone(),
                event.content.clone(),
            );
            let output = client.as_ref()
                .gift_wrap(client_pubkey, rumor, Vec::<Tag>::new())
                .await
                .map_err(|e| Error::Encryption(e.to_string()))?;
            output.val
        } else {
            let output = client.send_event_builder(builder).await
                .map_err(|e| Error::Transport(e.to_string()))?;
            output.val
        };

        Ok(final_event_id)
    }

    /// Clean up inactive sessions
    pub async fn cleanup_inactive_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let timeout = self.config.session_timeout;

        sessions.retain(|_, session| session.last_activity.elapsed() < timeout);
    }
}
