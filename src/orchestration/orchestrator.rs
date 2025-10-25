// Orchestrator - uses LLM to route content to appropriate experts

use super::experts::ExpertRegistry;
use super::types::{
    ExecutionMode, ExpertResult, ExpertType, OrchestratorConfig, RoutingDecision, TranslatedContent,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main orchestrator that routes content to experts using LLM reasoning
pub struct Orchestrator {
    config: OrchestratorConfig,
    client: reqwest::Client,
    registry: ExpertRegistry,
}

impl Orchestrator {
    /// Create a new orchestrator with the given model
    pub fn new(model: impl Into<String>) -> Self {
        let mut config = OrchestratorConfig::default();
        config.model = model.into();
        Self::with_config(config)
    }

    /// Create orchestrator with custom configuration
    pub fn with_config(config: OrchestratorConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.max_routing_time))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(1)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            registry: ExpertRegistry::new(),
        }
    }

    /// Process translated content through the expert system
    pub async fn process(&self, content: TranslatedContent) -> Result<Vec<ExpertResult>, OrchestratorError> {
        // Get routing decision from LLM
        let decision = self.route(&content).await?;

        println!("🎯 Routing decision: {:?}", decision.experts);
        println!("💭 Reasoning: {}", decision.reasoning);

        // Execute based on mode
        match decision.execution {
            ExecutionMode::Parallel => self.execute_parallel(&content, &decision.experts).await,
            ExecutionMode::Sequential => self.execute_sequential(&content, &decision.experts).await,
        }
    }

    /// Get routing decision from LLM
    async fn route(&self, content: &TranslatedContent) -> Result<RoutingDecision, OrchestratorError> {
        let system_prompt = self.build_routing_prompt();
        let user_prompt = self.build_content_prompt(content);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            stream: false,
            format: None, // Don't use JSON format constraint - some models don't support it well
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.config.ollama_endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| OrchestratorError::NetworkError(format!(
                "Failed to connect to Ollama at {}: {}. Is Ollama running?",
                self.config.ollama_endpoint, e
            )))?;

        if !response.status().is_success() {
            return Err(OrchestratorError::LlmError(format!(
                "LLM request failed: {}",
                response.status()
            )));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| OrchestratorError::ParseError(e.to_string()))?;

        // Debug: print what we got from LLM
        eprintln!("🔍 LLM Response: {}", chat_response.message.content);

        // Check if response is empty
        if chat_response.message.content.trim().is_empty() {
            return Err(OrchestratorError::LlmError(
                "LLM returned empty response. Check if Ollama is running and the model is available.".to_string()
            ));
        }

        // Extract JSON from response (may be wrapped in code blocks or have extra text)
        let json_str = self.extract_json(&chat_response.message.content)?;

        // Parse the routing decision from JSON
        let decision: RoutingDecision = serde_json::from_str(&json_str)
            .map_err(|e| {
                eprintln!("❌ Failed to parse JSON. Extracted content:\n{}", json_str);
                OrchestratorError::ParseError(format!(
                    "Failed to parse routing decision: {}. Response was: '{}'",
                    e, json_str
                ))
            })?;

        Ok(decision)
    }

    /// Extract JSON from LLM response (handles code blocks and extra text)
    fn extract_json(&self, content: &str) -> Result<String, OrchestratorError> {
        let content = content.trim();

        // Try to find JSON within code blocks
        if let Some(start) = content.find("```json") {
            if let Some(end) = content[start..].find("```") {
                let json_start = start + 7; // length of "```json"
                let json_end = start + end;
                return Ok(content[json_start..json_end].trim().to_string());
            }
        }

        // Try to find JSON within plain code blocks
        if let Some(start) = content.find("```") {
            if let Some(end) = content[start + 3..].find("```") {
                let json_start = start + 3;
                let json_end = start + 3 + end;
                return Ok(content[json_start..json_end].trim().to_string());
            }
        }

        // Try to find JSON object directly
        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                if end > start {
                    return Ok(content[start..=end].to_string());
                }
            }
        }

        // Return as-is if no JSON markers found
        Ok(content.to_string())
    }

    /// Build the system prompt for routing
    fn build_routing_prompt(&self) -> String {
        format!(
            r#"You are a routing orchestrator for an AI expert system. Analyze the content and determine which expert(s) should handle it.

Available experts:
- Producer: {}
- Artist: {}
- Scribe: {}
- Agent: {}
- Analyst: {}

Respond with JSON in this exact format:
{{
  "experts": ["ExpertName1", "ExpertName2"],
  "reasoning": "Brief explanation of why these experts",
  "execution": "parallel" or "sequential"
}}

Guidelines:
- Choose 1-3 experts most relevant to the task
- Use "parallel" if experts can work independently
- Use "sequential" if one expert's output feeds into another
- Producer handles file creation
- Artist handles creative content
- Scribe handles documentation/note-taking
- Agent handles action execution
- Analyst handles research and analysis

Respond ONLY with valid JSON, no other text."#,
            ExpertType::Producer.description(),
            ExpertType::Artist.description(),
            ExpertType::Scribe.description(),
            ExpertType::Agent.description(),
            ExpertType::Analyst.description(),
        )
    }

    /// Build the user prompt with content details
    fn build_content_prompt(&self, content: &TranslatedContent) -> String {
        let mut prompt = format!(
            "Content Type: {:?}\n\n",
            content.content_type
        );

        if let Some(summary) = &content.summary {
            prompt.push_str(&format!("Summary: {}\n\n", summary));
        }

        if !content.metadata.is_empty() {
            prompt.push_str("Metadata:\n");
            for (key, value) in &content.metadata {
                prompt.push_str(&format!("  {}: {}\n", key, value));
            }
            prompt.push('\n');
        }

        prompt.push_str("Content:\n");
        // Truncate content if too long
        let text = if content.text.len() > 2000 {
            format!("{}...\n[truncated, {} total chars]", &content.text[..2000], content.text.len())
        } else {
            content.text.clone()
        };
        prompt.push_str(&text);

        prompt
    }

    /// Execute experts in parallel
    async fn execute_parallel(
        &self,
        content: &TranslatedContent,
        experts: &[ExpertType],
    ) -> Result<Vec<ExpertResult>, OrchestratorError> {
        let mut handles = Vec::new();

        for expert_type in experts {
            let expert = self
                .registry
                .get(*expert_type)
                .ok_or_else(|| OrchestratorError::ExpertNotFound(*expert_type))?;

            let content_clone = content.clone();
            let expert_clone = expert.clone(); // Clone the Arc
            let handle = tokio::spawn(async move {
                expert_clone.process(&content_clone).await
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    eprintln!("Expert processing error: {}", e);
                }
                Err(e) => {
                    eprintln!("Task join error: {}", e);
                }
            }
        }

        Ok(results)
    }

    /// Execute experts sequentially
    async fn execute_sequential(
        &self,
        content: &TranslatedContent,
        experts: &[ExpertType],
    ) -> Result<Vec<ExpertResult>, OrchestratorError> {
        let mut results = Vec::new();
        let mut current_content = content.clone();

        for expert_type in experts {
            let expert = self
                .registry
                .get(*expert_type)
                .ok_or_else(|| OrchestratorError::ExpertNotFound(*expert_type))?;

            match expert.process(&current_content).await {
                Ok(result) => {
                    // For sequential execution, next expert gets previous output
                    if !result.output.is_empty() {
                        current_content.text = result.output.clone();
                    }
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("Expert {} failed: {}", expert_type.as_str(), e);
                    results.push(ExpertResult::failed(*expert_type, e.to_string()));
                    // Continue with remaining experts
                }
            }
        }

        Ok(results)
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: Message,
}

#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Expert not found: {0:?}")]
    ExpertNotFound(ExpertType),

    #[error("Expert error: {0}")]
    ExpertError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = Orchestrator::new("test-model");
        assert_eq!(orchestrator.config.model, "test-model");
    }

    #[test]
    fn test_routing_prompt_contains_experts() {
        let orchestrator = Orchestrator::new("test");
        let prompt = orchestrator.build_routing_prompt();
        assert!(prompt.contains("Producer"));
        assert!(prompt.contains("Artist"));
        assert!(prompt.contains("Scribe"));
        assert!(prompt.contains("Agent"));
        assert!(prompt.contains("Analyst"));
    }
}
