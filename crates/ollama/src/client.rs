//! Ollama API client

use serde::{Deserialize, Serialize};

/// Ollama API client
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    /// Get the reqwest client
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

#[derive(Serialize)]
pub struct GenerateOptions {
    pub temperature: f32,
    pub top_p: f32,
    pub num_predict: i32,
}

#[derive(Deserialize)]
pub struct GenerateResponse {
    pub response: String,
    #[serde(default)]
    pub total_duration: u64,
    #[serde(default)]
    pub load_duration: u64,
    #[serde(default)]
    pub prompt_eval_count: i32,
    #[serde(default)]
    pub eval_count: i32,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct StreamResponse {
    pub message: Message,
    pub done: bool,
}
