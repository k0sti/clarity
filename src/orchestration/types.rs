// Core types for the orchestration system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type of content being processed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Code,
    Document,
    Audio,
    Video,
    Image,
    Archive,
    Structured, // JSON, YAML, XML, etc.
    Unknown,
}

impl ContentType {
    /// Detect content type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Text
            "txt" | "md" | "markdown" => ContentType::Text,

            // Code
            "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "java" | "rb" | "php"
            | "sh" | "bash" | "zsh" => ContentType::Code,

            // Documents
            "pdf" | "doc" | "docx" | "odt" | "rtf" => ContentType::Document,

            // Audio
            "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => ContentType::Audio,

            // Video
            "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" => ContentType::Video,

            // Images
            "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" | "bmp" => ContentType::Image,

            // Archives
            "zip" | "tar" | "gz" | "7z" | "rar" | "bz2" => ContentType::Archive,

            // Structured
            "json" | "yaml" | "yml" | "xml" | "toml" | "csv" => ContentType::Structured,

            _ => ContentType::Unknown,
        }
    }
}

/// Content after translation into structured text form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedContent {
    pub content_type: ContentType,
    pub text: String,
    pub metadata: HashMap<String, String>,
    pub summary: Option<String>,
}

impl TranslatedContent {
    pub fn new(content_type: ContentType, text: String) -> Self {
        Self {
            content_type,
            text,
            metadata: HashMap::new(),
            summary: None,
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }
}

/// Expert specializations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExpertType {
    Producer,  // Creates artifacts and files
    Artist,    // Generates creative content
    Scribe,    // Documents in Obsidian vault
    Agent,     // Executes actions with tools
    Analyst,   // Research and analysis
}

impl ExpertType {
    pub fn as_str(&self) -> &str {
        match self {
            ExpertType::Producer => "Producer",
            ExpertType::Artist => "Artist",
            ExpertType::Scribe => "Scribe",
            ExpertType::Agent => "Agent",
            ExpertType::Analyst => "Analyst",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ExpertType::Producer => "Creates files, artifacts, and structured outputs",
            ExpertType::Artist => "Generates creative content (stories, images, designs)",
            ExpertType::Scribe => "Documents information in Obsidian vault",
            ExpertType::Agent => "Executes actions using available tools",
            ExpertType::Analyst => "Researches topics and provides analysis",
        }
    }
}

/// How experts should be executed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Parallel,    // All experts run simultaneously
    Sequential,  // Experts run one after another
}

/// Routing decision from the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub experts: Vec<ExpertType>,
    pub reasoning: String,
    pub execution: ExecutionMode,
}

/// Status of an expert's processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResultStatus {
    Success,
    Partial,  // Completed with warnings
    Failed,
}

/// An artifact produced by an expert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub content: String,
    pub artifact_type: String,  // "file", "note", "report", etc.
    pub path: Option<PathBuf>,
}

impl Artifact {
    pub fn new(name: impl Into<String>, content: impl Into<String>, artifact_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            artifact_type: artifact_type.into(),
            path: None,
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
}

/// Result from an expert's processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertResult {
    pub expert: ExpertType,
    pub output: String,
    pub artifacts: Vec<Artifact>,
    pub status: ResultStatus,
    pub error: Option<String>,
}

impl ExpertResult {
    pub fn success(expert: ExpertType, output: String) -> Self {
        Self {
            expert,
            output,
            artifacts: Vec::new(),
            status: ResultStatus::Success,
            error: None,
        }
    }

    pub fn failed(expert: ExpertType, error: String) -> Self {
        Self {
            expert,
            output: String::new(),
            artifacts: Vec::new(),
            status: ResultStatus::Failed,
            error: Some(error),
        }
    }

    pub fn with_artifacts(mut self, artifacts: Vec<Artifact>) -> Self {
        self.artifacts = artifacts;
        self
    }
}

/// Configuration for the orchestration system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_temperature")]
    pub temperature: f32,

    #[serde(default = "default_max_routing_time")]
    pub max_routing_time: u64,

    #[serde(default = "default_fallback")]
    pub fallback_expert: ExpertType,

    #[serde(default = "default_endpoint")]
    pub ollama_endpoint: String,
}

fn default_model() -> String {
    "gpt-oss:20b".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_routing_time() -> u64 {
    30000
}

fn default_fallback() -> ExpertType {
    ExpertType::Analyst
}

fn default_endpoint() -> String {
    "http://localhost:11434".to_string()
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            temperature: default_temperature(),
            max_routing_time: default_max_routing_time(),
            fallback_expert: default_fallback(),
            ollama_endpoint: default_endpoint(),
        }
    }
}

/// Configuration for individual experts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpertConfig {
    #[serde(default)]
    pub producer: ProducerConfig,

    #[serde(default)]
    pub scribe: ScribeConfig,

    #[serde(default)]
    pub agent: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerConfig {
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    #[serde(default = "default_language")]
    pub default_language: String,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("./artifacts")
}

fn default_language() -> String {
    "rust".to_string()
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            default_language: default_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScribeConfig {
    #[serde(default = "default_vault_path")]
    pub vault_path: PathBuf,

    #[serde(default = "default_vault_location")]
    pub default_location: String,
}

fn default_vault_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join("obsidian/vault"))
        .unwrap_or_else(|| PathBuf::from("~/obsidian/vault"))
}

fn default_vault_location() -> String {
    "Clarity".to_string()
}

impl Default for ScribeConfig {
    fn default() -> Self {
        Self {
            vault_path: default_vault_path(),
            default_location: default_vault_location(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_confirm")]
    pub confirm_destructive: bool,

    #[serde(default = "default_allowed_tools")]
    pub allowed_tools: Vec<String>,
}

fn default_confirm() -> bool {
    true
}

fn default_allowed_tools() -> Vec<String> {
    vec!["bash".to_string(), "http".to_string(), "file".to_string()]
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            confirm_destructive: default_confirm(),
            allowed_tools: default_allowed_tools(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        assert_eq!(ContentType::from_extension("rs"), ContentType::Code);
        assert_eq!(ContentType::from_extension("mp3"), ContentType::Audio);
        assert_eq!(ContentType::from_extension("json"), ContentType::Structured);
        assert_eq!(ContentType::from_extension("unknown"), ContentType::Unknown);
    }

    #[test]
    fn test_expert_descriptions() {
        let producer = ExpertType::Producer;
        assert_eq!(producer.as_str(), "Producer");
        assert!(!producer.description().is_empty());
    }
}
