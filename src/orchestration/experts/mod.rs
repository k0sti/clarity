// Expert implementations

mod producer;
mod artist;
mod scribe;
mod agent;
mod analyst;

pub use producer::ProducerExpert;
pub use artist::ArtistExpert;
pub use scribe::ScribeExpert;
pub use agent::AgentExpert;
pub use analyst::AnalystExpert;

use super::types::{ExpertResult, ExpertType, TranslatedContent};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Base trait for all experts
#[async_trait]
pub trait Expert: Send + Sync {
    /// Process content and return results
    async fn process(&self, content: &TranslatedContent) -> Result<ExpertResult, ExpertError>;

    /// Get the expert type
    fn expert_type(&self) -> ExpertType;

    /// Get the expert's capabilities description
    fn capabilities(&self) -> &str;
}

/// Registry for managing experts
pub struct ExpertRegistry {
    experts: HashMap<ExpertType, Arc<dyn Expert>>,
}

impl ExpertRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            experts: HashMap::new(),
        };

        // Register all experts
        registry.register(Arc::new(ProducerExpert::new()));
        registry.register(Arc::new(ArtistExpert::new()));
        registry.register(Arc::new(ScribeExpert::new()));
        registry.register(Arc::new(AgentExpert::new()));
        registry.register(Arc::new(AnalystExpert::new()));

        registry
    }

    pub fn register(&mut self, expert: Arc<dyn Expert>) {
        self.experts.insert(expert.expert_type(), expert);
    }

    pub fn get(&self, expert_type: ExpertType) -> Option<Arc<dyn Expert>> {
        self.experts.get(&expert_type).cloned()
    }
}

impl Default for ExpertRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExpertError {
    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
