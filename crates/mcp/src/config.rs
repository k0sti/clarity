//! Configuration file support for MCP agents

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConfig {
    #[serde(default)]
    pub nostr: NostrConfig,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub encryption: EncryptionConfig,
    #[serde(default)]
    pub keys: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent: AgentInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nostr: Option<NostrConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama: Option<OllamaConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<EncryptionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedConfig {
    pub agent: AgentInfo,
    pub nostr: NostrConfig,
    pub ollama: OllamaConfig,
    pub encryption: EncryptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrConfig {
    /// Private key in nsec or hex format (optional - will be generated if not present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,

    /// Nostr relay URLs
    #[serde(default = "default_relays")]
    pub relays: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_host")]
    pub host: String,

    #[serde(default = "default_ollama_model")]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    #[serde(default = "default_encryption_mode")]
    pub mode: String, // "optional", "required", "disabled"
}

// Defaults

fn default_relays() -> Vec<String> {
    vec!["wss://strfry.atlantislabs.space".to_string()]
}

fn default_ollama_host() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama3.2".to_string()
}

fn default_encryption_mode() -> String {
    "optional".to_string()
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: default_ollama_host(),
            model: default_ollama_model(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            mode: default_encryption_mode(),
        }
    }
}

impl SharedConfig {
    /// Load shared configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: SharedConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save shared configuration to TOML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get private key for a specific agent or user
    pub fn get_key(&self, agent_id: &str) -> Option<String> {
        self.keys.get(agent_id).filter(|k| !k.is_empty()).cloned()
    }
}

impl Default for SharedConfig {
    fn default() -> Self {
        Self {
            nostr: NostrConfig::default(),
            ollama: OllamaConfig::default(),
            encryption: EncryptionConfig::default(),
            keys: HashMap::new(),
        }
    }
}

impl AgentConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: AgentConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

impl MergedConfig {
    /// Load and merge shared config with agent-specific config
    pub fn load(shared_path: &str, agent_path: &str, agent_id: &str) -> anyhow::Result<Self> {
        // Load shared config
        let shared = SharedConfig::from_file(shared_path).unwrap_or_default();

        // Load agent config
        let agent_config = AgentConfig::from_file(agent_path)?;

        // Merge configs
        let mut nostr = agent_config.nostr.unwrap_or_else(|| shared.nostr.clone());

        // Override private key from shared config if available
        if nostr.private_key.is_none() {
            nostr.private_key = shared.get_key(agent_id);
        }

        Ok(Self {
            agent: agent_config.agent,
            nostr,
            ollama: agent_config.ollama.unwrap_or_else(|| shared.ollama.clone()),
            encryption: agent_config.encryption.unwrap_or_else(|| shared.encryption.clone()),
        })
    }
}

impl Default for NostrConfig {
    fn default() -> Self {
        Self {
            private_key: None,
            relays: default_relays(),
        }
    }
}
