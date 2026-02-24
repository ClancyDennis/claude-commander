use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Simple message for basic conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Image source for vision API (base64 encoded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String, // "base64"
    pub media_type: String, // "image/png", "image/jpeg", etc.
    pub data: String,       // base64 encoded image data
}

/// Rich content block for multi-turn tool conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RichContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// Message with rich content (structured blocks instead of just strings)
/// Used for multi-turn tool conversations where we need to preserve tool_use and tool_result blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichMessage {
    pub role: String,
    pub content: RichMessageContent,
}

/// Content of a rich message - either a string or structured blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RichMessageContent {
    Text(String),
    Blocks(Vec<RichContentBlock>),
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Content block in AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

/// Unified AI response format across providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub id: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Provider configuration
#[derive(Debug, Clone)]
pub enum Provider {
    Claude { api_key: String, model: String },
    OpenAI { api_key: String, model: String },
    Gemini { api_key: String, model: String },
}
