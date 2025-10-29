//! Client-side Nostr transport for MCP

use crate::core::{
    constants::*, error::{Error, Result}, types::*,
};
use crate::relay::RelayPool;
use nostr_sdk::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Client-side transport configuration
pub struct NostrClientTransportConfig {
    pub relay_urls: Vec<String>,
    pub encryption_mode: EncryptionMode,
}

impl Default for NostrClientTransportConfig {
    fn default() -> Self {
        Self {
            relay_urls: vec!["wss://relay.damus.io".to_string()],
            encryption_mode: EncryptionMode::Optional,
        }
    }
}

/// Client-side Nostr transport
pub struct NostrClientTransport {
    relay_pool: Arc<RelayPool>,
    config: NostrClientTransportConfig,
    pending_requests: Arc<RwLock<HashMap<EventId, tokio::sync::oneshot::Sender<UnsignedEvent>>>>,
}

impl NostrClientTransport {
    /// Create a new client transport
    pub async fn new<T>(
        signer: T,
        config: NostrClientTransportConfig,
    ) -> Result<Self>
    where
        T: IntoNostrSigner,
    {
        let relay_pool = Arc::new(RelayPool::new(signer).await?);

        Ok(Self {
            relay_pool,
            config,
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Connect to relays and start listening
    pub async fn connect(&self) -> Result<()> {
        self.relay_pool.connect(&self.config.relay_urls).await?;

        let client = self.relay_pool.client();
        let pubkey = client
            .signer()
            .await
            .map_err(|e| Error::Other(e.to_string()))?
            .get_public_key()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        // Subscribe to responses (both regular and encrypted)
        let filter = Filter::new()
            .kinds(vec![
                Kind::from(CTXVM_MESSAGES_KIND),
                Kind::from(GIFT_WRAP_KIND),
            ])
            .pubkey(pubkey);

        client
            .subscribe(filter, None)
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;

        // Start listening for responses
        let pending_requests = self.pending_requests.clone();
        let client_clone = client.clone();

        tokio::spawn(async move {
            Self::handle_responses(client_clone, pending_requests).await;
        });

        Ok(())
    }

    async fn handle_responses(
        client: Arc<Client>,
        pending_requests: Arc<RwLock<HashMap<EventId, tokio::sync::oneshot::Sender<UnsignedEvent>>>>,
    ) {
        let mut notifications = client.notifications();

        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                Self::handle_response(*event, &pending_requests, &client).await;
            }
        }
    }

    async fn handle_response(
        event: Event,
        pending_requests: &Arc<RwLock<HashMap<EventId, tokio::sync::oneshot::Sender<UnsignedEvent>>>>,
        client: &Arc<Client>,
    ) {
        // Unwrap gift wrap if needed
        let actual_event = if event.kind == Kind::from(GIFT_WRAP_KIND) {
            match client.as_ref().unwrap_gift_wrap(&event).await {
                Ok(unwrapped) => unwrapped.rumor,
                Err(err) => {
                    tracing::error!("Failed to unwrap gift wrap: {}", err);
                    return;
                }
            }
        } else {
            // Convert Event to UnsignedEvent for consistency
            UnsignedEvent::new(
                event.pubkey,
                event.created_at,
                event.kind,
                event.tags.clone(),
                event.content.clone(),
            )
        };

        // Find the request event ID in tags
        let request_id = actual_event.tags.iter().find_map(|tag| {
            if let Some(TagStandard::Event { event_id, .. }) = tag.as_standardized() {
                Some(event_id)
            } else {
                None
            }
        });

        if let Some(request_id) = request_id {
            let mut pending = pending_requests.write().await;
            if let Some(sender) = pending.remove(&request_id) {
                let _ = sender.send(actual_event);
            }
        }
    }

    /// Send a request to a server
    pub async fn send_request(
        &self,
        server_pubkey: &PublicKey,
        request: McpMessage,
        use_encryption: bool,
    ) -> Result<McpMessage> {
        let request_json = request.to_json()?;
        let client = self.relay_pool.client();

        let builder = EventBuilder::new(Kind::from(CTXVM_MESSAGES_KIND), request_json)
            .tag(Tag::public_key(*server_pubkey));

        let event = client.sign_event_builder(builder).await
            .map_err(|e| Error::Other(e.to_string()))?;

        let event_id = event.id;

        // Create a channel to receive the response
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending_requests.write().await.insert(event_id, tx);

        // Send the event
        let _final_event_id = if use_encryption {
            // Convert to UnsignedEvent for gift wrapping
            let rumor = UnsignedEvent::new(
                event.pubkey,
                event.created_at,
                event.kind,
                event.tags.clone(),
                event.content.clone(),
            );
            let output = client.as_ref()
                .gift_wrap(server_pubkey, rumor, Vec::<Tag>::new())
                .await
                .map_err(|e| Error::Encryption(e.to_string()))?;
            output.val
        } else {
            self.relay_pool.publish(event).await?
        };

        // Wait for response with timeout
        let response_event = tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|_| Error::Transport("Response channel closed".to_string()))?;

        // Parse response
        let response = McpMessage::from_json(&response_event.content)?;

        Ok(response)
    }
}
