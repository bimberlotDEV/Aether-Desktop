use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::tools::NativeToolId;

#[derive(Debug, Clone, PartialEq)]
pub enum ModelMessage {
    System(String),
    User(String),
    Assistant(String),
    AssistantToolRequests {
        content: Option<String>,
        requests: Vec<ModelToolRequest>,
    },
    ToolResult(ModelToolResult),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelToolDescriptor {
    pub tool_id: NativeToolId,
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelToolRequest {
    pub request_id: String,
    pub tool_name: String,
    pub arguments_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelToolResultStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelToolResult {
    pub request_id: String,
    pub tool_name: String,
    pub tool_id: Option<NativeToolId>,
    pub status: ModelToolResultStatus,
    pub output: Value,
}

#[derive(Debug, Clone)]
pub struct ModelTurnRequest {
    pub model: String,
    pub messages: Vec<ModelMessage>,
    pub tools: Vec<ModelToolDescriptor>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub thinking_enabled: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelTurnOutcome {
    pub tool_requests: Vec<ModelToolRequest>,
}
