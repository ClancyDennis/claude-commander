use std::time::Duration;

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

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Google Gemini provider implementation
pub struct GeminiProvider {
    api_key: String,
    model: String,
    http_client: Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            http_client: Client::builder()
                .timeout(Duration::from_secs(180))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Convert tools to Gemini format (function declarations)
    fn convert_tools(tools: &[Tool]) -> Vec<Value> {
        let declarations: Vec<Value> = tools
            .iter()
            .map(|t| {
                // Gemini uses a slightly different schema format - strip out
                // "additionalProperties" and other unsupported keys
                let mut params = t.input_schema.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.remove("additionalProperties");
                }
                json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": params,
                })
            })
            .collect();

        vec![json!({
            "functionDeclarations": declarations
        })]
    }

    /// Convert simple messages to Gemini format
    fn convert_messages(messages: &[Message]) -> Vec<Value> {
        messages
            .iter()
            .map(|m| {
                let role = match m.role.as_str() {
                    "assistant" => "model",
                    _ => "user",
                };
                json!({
                    "role": role,
                    "parts": [{"text": m.content}]
                })
            })
            .collect()
    }

    /// Convert rich messages to Gemini format
    /// Gemini uses a different tool call format:
    /// - Model responses with function calls: parts contain functionCall objects
    /// - User responses with function results: parts contain functionResponse objects
    fn convert_rich_messages(messages: &[RichMessage]) -> Vec<Value> {
        let mut gemini_messages: Vec<Value> = Vec::new();

        for msg in messages {
            let role = match msg.role.as_str() {
                "assistant" => "model",
                _ => "user",
            };

            match &msg.content {
                RichMessageContent::Text(s) => {
                    gemini_messages.push(json!({
                        "role": role,
                        "parts": [{"text": s}]
                    }));
                }
                RichMessageContent::Blocks(blocks) => {
                    let mut parts: Vec<Value> = Vec::new();

                    for block in blocks {
                        match block {
                            RichContentBlock::Text { text } => {
                                parts.push(json!({"text": text}));
                            }
                            RichContentBlock::Image { source } => {
                                parts.push(json!({
                                    "inlineData": {
                                        "mimeType": source.media_type,
                                        "data": source.data
                                    }
                                }));
                            }
                            RichContentBlock::ToolUse { id: _, name, input } => {
                                parts.push(json!({
                                    "functionCall": {
                                        "name": name,
                                        "args": input
                                    }
                                }));
                            }
                            RichContentBlock::ToolResult {
                                tool_use_id: _,
                                content,
                                is_error,
                            } => {
                                // For Gemini, function responses need the function name.
                                // Since we don't have it here, we use a generic response.
                                // The name is extracted from the preceding functionCall.
                                let response_value = if is_error.unwrap_or(false) {
                                    json!({"error": content})
                                } else {
                                    json!({"result": content})
                                };
                                parts.push(json!({
                                    "functionResponse": {
                                        "name": "_tool_response",
                                        "response": response_value
                                    }
                                }));
                            }
                        }
                    }

                    if !parts.is_empty() {
                        gemini_messages.push(json!({
                            "role": role,
                            "parts": parts
                        }));
                    }
                }
            }
        }

        gemini_messages
    }

    /// Convert Gemini response to unified AIResponse
    fn convert_response(response: GeminiResponse) -> Result<AIResponse, AIError> {
        let candidate = response
            .candidates
            .first()
            .ok_or_else(|| AIError::ParseError("No candidates in Gemini response".to_string()))?;

        let mut content_blocks = Vec::new();

        for part in &candidate.content.parts {
            if let Some(text) = &part.text {
                if !text.is_empty() {
                    content_blocks.push(ContentBlock::Text { text: text.clone() });
                }
            }
            if let Some(fc) = &part.function_call {
                let args = fc.args.clone().unwrap_or(json!({}));
                // Generate a unique ID for this tool call
                let id = format!("gemini_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0"));
                content_blocks.push(ContentBlock::ToolUse {
                    id,
                    name: fc.name.clone(),
                    input: args,
                });
            }
        }

        let stop_reason = candidate.finish_reason.as_ref().map(|r| match r.as_str() {
            "STOP" => "end_turn".to_string(),
            "MAX_TOKENS" => "length".to_string(),
            "SAFETY" => "safety".to_string(),
            other => other.to_lowercase(),
        });

        let usage = if let Some(meta) = &response.usage_metadata {
            Usage {
                input_tokens: meta.prompt_token_count.unwrap_or(0),
                output_tokens: meta.candidates_token_count.unwrap_or(0),
            }
        } else {
            Usage {
                input_tokens: 0,
                output_tokens: 0,
            }
        };

        Ok(AIResponse {
            id: format!("gemini-{}", uuid::Uuid::new_v4()),
            role: "assistant".to_string(),
            content: content_blocks,
            model: response
                .model_version
                .unwrap_or_else(|| "gemini".to_string()),
            stop_reason,
            usage,
        })
    }

    /// Build and send request to Gemini API
    async fn send_request(
        &self,
        contents: Vec<Value>,
        system_instruction: Option<&str>,
        tools: Option<&[Tool]>,
    ) -> Result<AIResponse, AIError> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_URL, self.model, self.api_key
        );

        let mut body = json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": 8192,
            }
        });

        if let Some(system) = system_instruction {
            body["systemInstruction"] = json!({
                "parts": [{"text": system}]
            });
        }

        if let Some(tools) = tools {
            if !tools.is_empty() {
                body["tools"] = json!(Self::convert_tools(tools));
                // Enable automatic function calling mode
                body["toolConfig"] = json!({
                    "functionCallingConfig": {
                        "mode": "AUTO"
                    }
                });
            }
        }

        let body_str = serde_json::to_string(&body).unwrap_or_default();
        eprintln!(
            "[LLM][Gemini][{}] Sending API request (body size: {} bytes)",
            self.model,
            body_str.len()
        );

        let response = self
            .http_client
            .post(&url)
            .header("content-type", "application/json")
            .body(body_str)
            .send()
            .await?;

        let response = check_response_status(response, "Gemini").await?;

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(format!("Failed to parse Gemini response: {}", e)))?;

        if let Some(meta) = &gemini_response.usage_metadata {
            eprintln!(
                "[LLM][Gemini][{}] Response received - tokens: in={}, out={}",
                self.model,
                meta.prompt_token_count.unwrap_or(0),
                meta.candidates_token_count.unwrap_or(0)
            );
        }

        Self::convert_response(gemini_response)
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn send_message(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        let contents = Self::convert_messages(&messages);
        self.send_request(contents, None, tools.as_deref()).await
    }

    async fn send_message_with_system(
        &self,
        system_prompt: &str,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        let contents = Self::convert_messages(&messages);
        self.send_request(contents, Some(system_prompt), tools.as_deref())
            .await
    }

    async fn send_rich_message(
        &self,
        messages: Vec<RichMessage>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError> {
        let contents = Self::convert_rich_messages(&messages);
        self.send_request(contents, None, tools.as_deref()).await
    }

    fn name(&self) -> &str {
        "Gemini"
    }

    fn model(&self) -> &str {
        &self.model
    }
}

// ============================================================================
// Gemini API response structures
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    usage_metadata: Option<GeminiUsageMetadata>,
    model_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: GeminiContent,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    #[allow(dead_code)]
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiPart {
    text: Option<String>,
    function_call: Option<GeminiFunctionCall>,
    #[allow(dead_code)]
    function_response: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiFunctionCall {
    name: String,
    args: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
    #[allow(dead_code)]
    total_token_count: Option<u32>,
}
