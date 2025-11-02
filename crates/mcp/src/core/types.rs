//! Core types for MCP protocol

use serde::{Deserialize, Serialize};

/// MCP JSON-RPC message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpMessage {
    Request(serde_json::Value),
    Response(serde_json::Value),
    Notification(serde_json::Value),
}

impl McpMessage {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
