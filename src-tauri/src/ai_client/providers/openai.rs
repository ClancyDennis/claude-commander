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

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI provider implementation
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    http_client: Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            http_client: Client::new(),
        }
    }

    /// Convert tools to OpenAI format
    fn convert_tools(tools: &[Tool]) -> Vec<Value> {
        tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    }
                })
            })
            .collect()
    }

    /// Convert simple messages to OpenAI format
    fn convert_messages(messages: &[Message]) -> Vec<Value> {
        messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect()
    }

    /// Convert rich messages to OpenAI format
    /// OpenAI format is different from Claude:
    /// - Assistant with tool calls: has tool_calls array
    /// - Tool results: separate messages with role="tool"
    fn convert_rich_messages(messages: &[RichMessage]) -> Vec<Value> {
        let mut openai_messages: Vec<Value> = Vec::new();

        for msg in messages {
            match &msg.content {
                RichMessageContent::Text(s) => {
                    openai_messages.push(json!({
                        "role": msg.role,
                        "content": s
                    }));
                }
                RichMessageContent::Blocks(blocks) => {
                    if msg.role == "assistant" {
                        // Assistant message with potential tool calls
                        let mut text_content: Option<String> = None;
                        let mut tool_calls: Vec<Value> = Vec::new();

                        for block in blocks {
                            match block {
                                RichContentBlock::Text { text } => {
                                    text_content = Some(text.clone());
                                }
                                RichContentBlock::ToolUse { id, name, input } => {
                                    tool_calls.push(json!({
                                        "id": id,
                                        "type": "function",
                                        "function": {
                                            "name": name,
                                            "arguments": serde_json::to_string(input).unwrap_or_default()
                                        }
                                    }));
                                }
                                RichContentBlock::ToolResult { .. } => {
                                    // Tool results don't appear in assistant messages
                                }
                            }
                        }

                        let mut assistant_msg = json!({ "role": "assistant" });
                        if let Some(text) = text_content {
                            assistant_msg["content"] = json!(text);
                        }
                        if !tool_calls.is_empty() {
                            assistant_msg["tool_calls"] = json!(tool_calls);
                        }
                        openai_messages.push(assistant_msg);
                    } else if msg.role == "user" {
                        // User message with potential tool results
                        for block in blocks {
                            match block {
                                RichContentBlock::Text { text } => {
                                    openai_messages.push(json!({
                                        "role": "user",
                                        "content": text
                                    }));
                                }
                                RichContentBlock::ToolResult {
                                    tool_use_id,
                                    content,
                                    ..
                                } => {
                                    openai_messages.push(json!({
                                        "role": "tool",
                                        "tool_call_id": tool_use_id,
                                        "content": content
                                    }));
                                }
                                RichContentBlock::ToolUse { .. } => {
                                    // Tool uses don't appear in user messages
                                }
                            }
                        }
                    }
                }
            }
        }

        openai_messages
    }

    /// Convert OpenAI response to unified AIResponse
    fn convert_response(response: OpenAIResponse) -> Result<AIResponse, AIError> {
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::ParseError("No choices in OpenAI response".to_string()))?;

        let mut content_blocks = Vec::new();

        // Add text content if present
        if let Some(text) = &choice.message.content {
            if !text.is_empty() {
                content_blocks.push(ContentBlock::Text { text: text.clone() });
            }
        }

        // Add tool calls if present
        if let Some(tool_calls) = &choice.message.tool_calls {
            for tool_call in tool_calls {
                let args: Value =
                    serde_json::from_str(&tool_call.function.arguments).unwrap_or(json!({}));

                content_blocks.push(ContentBlock::ToolUse {
                    id: tool_call.id.clone(),
                    name: tool_call.function.name.clone(),
                    input: args,
                });
            }
        }

        let stop_reason = match choice.finish_reason.as_str() {
            "stop" => Some("end_turn".to_string()),
            "tool_calls" => Some("tool_use".to_string()),
            "length" => Some("length".to_string()),
            other => Some(other.to_string()),
        };

        Ok(AIResponse {
            id: response.id,
            role: "assistant".to_string(),
            content: content_blocks,
            model: response.model,
            stop_reason,
            usage: Usage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
        })
    }

    /// Build and send request to OpenAI API
    async fn send_request(&self, body: Value) -> Result<AIResponse, AIError> {
        let response = self
            .http_client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let response = check_response_status(response, "OpenAI").await?;

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(format!("Failed to parse OpenAI response: {}", e)))?;

        Self::convert_response(openai_response)
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn send_message(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        let openai_messages = Self::convert_messages(&messages);

        let mut body = json!({
            "model": self.model,
            "messages": openai_messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(Self::convert_tools(&tools));
            body["tool_choice"] = json!("auto");
        }

        self.send_request(body).await
    }

    async fn send_message_with_system(
        &self,
        system_prompt: &str,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        // For OpenAI, prepend a system message
        let mut openai_messages = vec![json!({
            "role": "system",
            "content": system_prompt
        })];
        openai_messages.extend(Self::convert_messages(&messages));

        let mut body = json!({
            "model": self.model,
            "messages": openai_messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(Self::convert_tools(&tools));
            body["tool_choice"] = json!("auto");
        }

        self.send_request(body).await
    }

    async fn send_rich_message(
        &self,
        messages: Vec<RichMessage>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        let openai_messages = Self::convert_rich_messages(&messages);

        let mut body = json!({
            "model": self.model,
            "messages": openai_messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(Self::convert_tools(&tools));
            body["tool_choice"] = json!("auto");
        }

        self.send_request(body).await
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn model(&self) -> &str {
        &self.model
    }
}

// Internal response structures for OpenAI API
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    #[allow(dead_code)]
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    #[allow(dead_code)]
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}
