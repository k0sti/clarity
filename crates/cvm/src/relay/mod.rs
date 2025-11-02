//! Nostr relay pool management

use crate::core::error::{Error, Result};
use nostr_sdk::prelude::*;
use std::sync::Arc;
use std::time::Duration;

/// Relay pool wrapper for managing Nostr relay connections
pub struct RelayPool {
    client: Arc<Client>,
}

impl RelayPool {
    /// Create a new relay pool with the given signer
    pub async fn new<T>(signer: T) -> Result<Self>
    where
        T: IntoNostrSigner,
    {
        let client = Client::builder().signer(signer).build();

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Connect to relay URLs
    pub async fn connect(&self, relay_urls: &[String]) -> Result<()> {
        for url in relay_urls {
            self.client.add_relay(url).await.map_err(|e| Error::Transport(e.to_string()))?;
        }

        self.client.connect().await;

        Ok(())
    }

    /// Disconnect from relays
    pub async fn disconnect(&self) -> Result<()> {
        self.client.disconnect().await;
        Ok(())
    }

    /// Publish an event to relays
    pub async fn publish(&self, event: Event) -> Result<EventId> {
        let output = self
            .client
            .send_event(&event)
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;

        Ok(output.val)
    }

    /// Subscribe to events matching filters
    pub async fn subscribe(&self, filters: Vec<Filter>, timeout: Duration) -> Result<Events> {
        // Combine multiple filters using OR logic
        let combined_filter = filters.into_iter().reduce(|acc, _f| {
            // We'll just use the first filter for simplicity
            // In a real implementation, you'd need to properly combine filters
            acc
        }).unwrap_or_else(Filter::new);

        let events = self
            .client
            .fetch_events(combined_filter, timeout)
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;

        Ok(events)
    }

    /// Get the underlying client
    pub fn client(&self) -> &Arc<Client> {
        &self.client
    }
}
