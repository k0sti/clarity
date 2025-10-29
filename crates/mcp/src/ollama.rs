//! Ollama LLM integration module

use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Generate tools for an agent based on its subject using LLM
pub async fn generate_tools_for_subject(
    ollama_host: &str,
    model: &str,
    subject: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    let prompt = format!(
        r#"You are a tool designer for an AI agent expert in: {subject}

Based on this subject, design 2-5 useful tools this agent should provide.

Guidelines:
- Each tool should relate to the subject expertise
- Tools should be practical and actionable
- If the subject doesn't need specialized tools, provide just ONE tool called "query" for general questions
- Return ONLY valid JSON, no other text

Return format (JSON array):
[
  {{
    "name": "tool_name",
    "description": "What this tool does",
    "inputSchema": {{
      "type": "object",
      "properties": {{
        "param_name": {{
          "type": "string",
          "description": "parameter description"
        }}
      }},
      "required": ["param_name"]
    }}
  }}
]

Example for "gardening" subject:
[
  {{
    "name": "plant_care_advice",
    "description": "Provides care instructions for a specific plant",
    "inputSchema": {{
      "type": "object",
      "properties": {{
        "plant_name": {{
          "type": "string",
          "description": "Name of the plant"
        }},
        "issue": {{
          "type": "string",
          "description": "Optional issue or question about the plant"
        }}
      }},
      "required": ["plant_name"]
    }}
  }},
  {{
    "name": "seasonal_tasks",
    "description": "Lists gardening tasks for a specific season and region",
    "inputSchema": {{
      "type": "object",
      "properties": {{
        "season": {{
          "type": "string",
          "description": "Season (spring, summer, fall, winter)"
        }},
        "region": {{
          "type": "string",
          "description": "Geographic region or climate zone"
        }}
      }},
      "required": ["season"]
    }}
  }},
  {{
    "name": "query",
    "description": "Ask any general question about gardening",
    "inputSchema": {{
      "type": "object",
      "properties": {{
        "question": {{
          "type": "string",
          "description": "Your gardening question"
        }}
      }},
      "required": ["question"]
    }}
  }}
]

Now generate tools for subject: {subject}
Return ONLY the JSON array:"#
    );

    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        format: Some("json".to_string()),
    };

    let url = format!("{}/api/generate", ollama_host);
    let response = client.post(&url).json(&request).send().await?;

    let ollama_response: OllamaResponse = response.json().await?;

    // Parse the JSON response
    let tools: Vec<serde_json::Value> = serde_json::from_str(&ollama_response.response)
        .map_err(|e| format!("Failed to parse tools JSON: {}. Response was: {}", e, ollama_response.response))?;

    // Validate that we got at least one tool
    if tools.is_empty() {
        // Fallback to default query tool
        Ok(vec![serde_json::json!({
            "name": "query",
            "description": format!("Ask questions about {}", subject),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "Your question"
                    }
                },
                "required": ["question"]
            }
        })])
    } else {
        Ok(tools)
    }
}
