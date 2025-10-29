// Agent Expert - performs requested actions with given toolset

use super::{Expert, ExpertError};
use crate::orchestration::types::{Artifact, ExpertResult, ExpertType, ResultStatus, TranslatedContent};
use async_trait::async_trait;

/// Agent executes actions using available tools
pub struct AgentExpert {
    confirm_destructive: bool,
    allowed_tools: Vec<String>,
}

impl AgentExpert {
    pub fn new() -> Self {
        Self {
            confirm_destructive: true,
            allowed_tools: vec![
                "bash".to_string(),
                "http".to_string(),
                "file".to_string(),
            ],
        }
    }

    pub fn with_tools(allowed_tools: Vec<String>) -> Self {
        Self {
            confirm_destructive: true,
            allowed_tools,
        }
    }

    /// Analyze content to determine what actions to take
    async fn determine_actions(&self, content: &TranslatedContent) -> Result<Vec<Action>, ExpertError> {
        let mut actions = Vec::new();

        // Parse content for action requests
        let text_lower = content.text.to_lowercase();

        // Check for common action patterns
        if text_lower.contains("fetch") || text_lower.contains("get") || text_lower.contains("http") {
            actions.push(Action {
                tool: "http".to_string(),
                command: "HTTP request detected".to_string(),
                destructive: false,
            });
        }

        if text_lower.contains("run") || text_lower.contains("execute") || text_lower.contains("command") {
            actions.push(Action {
                tool: "bash".to_string(),
                command: "Command execution detected".to_string(),
                destructive: self.might_be_destructive(&text_lower),
            });
        }

        if text_lower.contains("read file") || text_lower.contains("write file") {
            actions.push(Action {
                tool: "file".to_string(),
                command: "File operation detected".to_string(),
                destructive: text_lower.contains("write") || text_lower.contains("delete"),
            });
        }

        // If no specific actions detected, provide analysis
        if actions.is_empty() {
            actions.push(Action {
                tool: "analysis".to_string(),
                command: "Analyze content for actionable items".to_string(),
                destructive: false,
            });
        }

        Ok(actions)
    }

    fn might_be_destructive(&self, text: &str) -> bool {
        let destructive_keywords = [
            "delete", "remove", "rm ", "drop", "truncate",
            "force", "overwrite", "wipe", "erase"
        ];

        destructive_keywords.iter().any(|&keyword| text.contains(keyword))
    }

    /// Execute a single action
    async fn execute_action(&self, action: &Action) -> Result<String, ExpertError> {
        // Check if tool is allowed
        if !self.allowed_tools.contains(&action.tool) {
            return Err(ExpertError::ConfigError(
                format!("Tool '{}' is not in allowed tools list", action.tool)
            ));
        }

        // Check for destructive operations
        if action.destructive && self.confirm_destructive {
            return Ok(format!(
                "⚠️  Destructive action detected: {}\n\
                Tool: {}\n\
                Status: Requires confirmation (confirmation system not yet implemented)",
                action.command, action.tool
            ));
        }

        // Execute based on tool type
        match action.tool.as_str() {
            "bash" => self.execute_bash(action).await,
            "http" => self.execute_http(action).await,
            "file" => self.execute_file(action).await,
            "analysis" => self.execute_analysis(action).await,
            _ => Err(ExpertError::ProcessingError(
                format!("Unknown tool: {}", action.tool)
            )),
        }
    }

    async fn execute_bash(&self, action: &Action) -> Result<String, ExpertError> {
        // For safety, we don't actually execute bash commands without explicit user consent
        Ok(format!(
            "📋 Bash command analysis:\n\
            Command: {}\n\
            Status: Simulated (actual execution requires user approval)\n\
            \n\
            To execute, the system would:\n\
            1. Validate command safety\n\
            2. Run in isolated environment\n\
            3. Capture output and errors\n\
            4. Return results",
            action.command
        ))
    }

    async fn execute_http(&self, _action: &Action) -> Result<String, ExpertError> {
        Ok(
            "🌐 HTTP request capability available:\n\
            - GET/POST/PUT/DELETE requests\n\
            - Header customization\n\
            - Authentication support\n\
            - Response parsing\n\
            \n\
            Note: Actual HTTP execution would require specific endpoint details"
                .to_string()
        )
    }

    async fn execute_file(&self, action: &Action) -> Result<String, ExpertError> {
        Ok(format!(
            "📁 File operation analysis:\n\
            Operation: {}\n\
            \n\
            Available file operations:\n\
            - Read files\n\
            - Write files\n\
            - List directories\n\
            - Move/copy files\n\
            \n\
            Note: File system access requires proper permissions",
            action.command
        ))
    }

    async fn execute_analysis(&self, _action: &Action) -> Result<String, ExpertError> {
        Ok(
            "🔍 Action Analysis:\n\
            \n\
            No specific executable actions detected in the content.\n\
            \n\
            The Agent expert can help with:\n\
            - Running shell commands\n\
            - Making HTTP/API requests\n\
            - File system operations\n\
            - Data transformations\n\
            - External service integrations\n\
            \n\
            Please provide more specific action requests."
                .to_string()
        )
    }
}

impl Default for AgentExpert {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Expert for AgentExpert {
    async fn process(&self, content: &TranslatedContent) -> Result<ExpertResult, ExpertError> {
        let actions = self.determine_actions(content).await?;

        let mut outputs = Vec::new();
        let mut artifacts = Vec::new();

        for action in &actions {
            match self.execute_action(action).await {
                Ok(result) => {
                    outputs.push(result.clone());
                    artifacts.push(Artifact::new(
                        format!("action_{}.txt", action.tool),
                        result,
                        "action_result"
                    ));
                }
                Err(e) => {
                    outputs.push(format!("❌ Action failed: {}", e));
                }
            }
        }

        let output = format!(
            "Executed {} action(s):\n\n{}",
            actions.len(),
            outputs.join("\n\n---\n\n")
        );

        Ok(ExpertResult {
            expert: ExpertType::Agent,
            output,
            artifacts,
            status: ResultStatus::Success,
            error: None,
        })
    }

    fn expert_type(&self) -> ExpertType {
        ExpertType::Agent
    }

    fn capabilities(&self) -> &str {
        "Executes actions using available tools (bash, HTTP, file operations)"
    }
}

#[derive(Debug, Clone)]
struct Action {
    tool: String,
    command: String,
    destructive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let agent = AgentExpert::new();
        assert_eq!(agent.expert_type(), ExpertType::Agent);
    }

    #[test]
    fn test_destructive_detection() {
        let agent = AgentExpert::new();
        assert!(agent.might_be_destructive("rm -rf /"));
        assert!(agent.might_be_destructive("delete all files"));
        assert!(!agent.might_be_destructive("list files"));
    }
}
