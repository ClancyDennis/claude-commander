use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::ai_client::error::{check_response_status, AIError};
use crate::ai_client::providers::AIProvider;
use crate::ai_client::types::{
    AIResponse, ContentBlock, Message, RichContentBlock, RichMessage, RichMessageContent, Tool,
    Usage,
};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_API_VERSION: &str = "2023-06-01";

/// Claude (Anthropic) provider implementation
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    http_client: Client,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            http_client: Client::new(),
        }
    }

    /// Convert rich messages to Claude API format
    fn convert_rich_messages(messages: &[RichMessage]) -> Vec<Value> {
        messages
            .iter()
            .map(|m| {
                let content = match &m.content {
                    RichMessageContent::Text(s) => json!(s),
                    RichMessageContent::Blocks(blocks) => {
                        let claude_blocks: Vec<Value> = blocks
                            .iter()
                            .map(|block| match block {
                                RichContentBlock::Text { text } => json!({
                                    "type": "text",
                                    "text": text
                                }),
                                RichContentBlock::ToolUse { id, name, input } => json!({
                                    "type": "tool_use",
                                    "id": id,
                                    "name": name,
                                    "input": input
                                }),
                                RichContentBlock::ToolResult {
                                    tool_use_id,
                                    content,
                                    is_error,
                                } => {
                                    let mut result = json!({
                                        "type": "tool_result",
                                        "tool_use_id": tool_use_id,
                                        "content": content
                                    });
                                    if let Some(err) = is_error {
                                        result["is_error"] = json!(err);
                                    }
                                    result
                                }
                            })
                            .collect();
                        json!(claude_blocks)
                    }
                };
                json!({
                    "role": m.role,
                    "content": content
                })
            })
            .collect()
    }

    /// Convert Claude API response to unified AIResponse
    fn convert_response(response: ClaudeAPIResponse) -> AIResponse {
        AIResponse {
            id: response.id,
            role: response.role,
            content: response.content,
            model: response.model,
            stop_reason: response.stop_reason,
            usage: Usage {
                input_tokens: response.usage.input_tokens,
                output_tokens: response.usage.output_tokens,
            },
        }
    }

    /// Build and send request to Claude API
    async fn send_request(&self, body: Value) -> Result<AIResponse, AIError> {
        let response = self
            .http_client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", CLAUDE_API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let response = check_response_status(response, "Claude").await?;

        let claude_response: ClaudeAPIResponse = response.json().await.map_err(|e| {
            AIError::ParseError(format!("Failed to parse Claude response: {}", e))
        })?;

        Ok(Self::convert_response(claude_response))
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn send_message(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        let mut body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        self.send_request(body).await
    }

    async fn send_rich_message(
        &self,
        messages: Vec<RichMessage>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        let claude_messages = Self::convert_rich_messages(&messages);

        let mut body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": claude_messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        self.send_request(body).await
    }

    fn name(&self) -> &str {
        "Claude"
    }

    fn model(&self) -> &str {
        &self.model
    }
}

// Internal response structures for Claude API
#[derive(Debug, Deserialize)]
struct ClaudeAPIResponse {
    id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    response_type: String,
    role: String,
    content: Vec<ContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}
