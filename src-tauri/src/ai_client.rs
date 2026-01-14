use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::fmt;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_API_VERSION: &str = "2023-06-01";
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug)]
pub enum AIError {
    HttpError(reqwest::Error),
    ApiError(String),
    ParseError(String),
    ConfigError(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AIError::HttpError(e) => write!(f, "HTTP error: {}", e),
            AIError::ApiError(e) => write!(f, "API error: {}", e),
            AIError::ParseError(e) => write!(f, "Parse error: {}", e),
            AIError::ConfigError(e) => write!(f, "Config error: {}", e),
        }
    }
}

impl Error for AIError {}

impl From<reqwest::Error> for AIError {
    fn from(err: reqwest::Error) -> Self {
        AIError::HttpError(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub id: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone)]
pub enum Provider {
    Claude { api_key: String, model: String },
    OpenAI { api_key: String, model: String },
}

pub struct AIClient {
    provider: Provider,
    http_client: Client,
}

impl AIClient {
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            http_client: Client::new(),
        }
    }

    pub fn from_env() -> Result<Self, AIError> {
        // Try Claude first
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let model = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| Self::get_default_claude_model());
            return Ok(Self::new(Provider::Claude { api_key, model }));
        }

        // Try OpenAI second
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let model = std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o".to_string());
            return Ok(Self::new(Provider::OpenAI { api_key, model }));
        }

        Err(AIError::ConfigError(
            "No API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable".to_string()
        ))
    }

    /// Returns the latest recommended Claude model
    /// Priority: claude-sonnet-4-5 > claude-sonnet-4 > claude-3-5-sonnet
    fn get_default_claude_model() -> String {
        // Based on Anthropic's naming pattern, try latest versions first
        let candidates = vec![
            "claude-sonnet-4-5-20250929",  // Latest Sonnet 4.5
            "claude-sonnet-4-20250514",     // Sonnet 4
            "claude-3-5-sonnet-20241022",   // Latest 3.5
        ];

        // Return the first candidate (latest)
        candidates[0].to_string()
    }

    /// List available Claude models (requires API call)
    pub async fn list_claude_models(api_key: &str) -> Result<Vec<String>, AIError> {
        // Note: Anthropic doesn't have a public models endpoint yet
        // Return known models in order of preference
        Ok(vec![
            "claude-sonnet-4-5-20250929".to_string(),
            "claude-sonnet-4-20250514".to_string(),
            "claude-opus-4-5-20251101".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-sonnet-20240620".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ])
    }

    /// List available OpenAI models (requires API call)
    pub async fn list_openai_models(api_key: &str) -> Result<Vec<String>, AIError> {
        let http_client = Client::new();
        let response = http_client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::ApiError(format!("Failed to fetch OpenAI models: {}", error_text)));
        }

        let models_response: OpenAIModelsResponse = response.json().await
            .map_err(|e| AIError::ParseError(format!("Failed to parse models: {}", e)))?;

        // Filter for chat models only
        let chat_models: Vec<String> = models_response.data
            .into_iter()
            .filter(|m| m.id.starts_with("gpt-4") || m.id.starts_with("gpt-3.5"))
            .map(|m| m.id)
            .collect();

        Ok(chat_models)
    }

    pub async fn send_message_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<Tool>,
        max_tokens: u32,
    ) -> Result<AIResponse, AIError> {
        match &self.provider {
            Provider::Claude { api_key, model } => {
                self.send_claude_message(api_key, model, messages, Some(tools), max_tokens).await
            }
            Provider::OpenAI { api_key, model } => {
                self.send_openai_message(api_key, model, messages, Some(tools), max_tokens).await
            }
        }
    }

    pub async fn send_message(
        &self,
        messages: Vec<Message>,
        max_tokens: u32,
    ) -> Result<AIResponse, AIError> {
        match &self.provider {
            Provider::Claude { api_key, model } => {
                self.send_claude_message(api_key, model, messages, None, max_tokens).await
            }
            Provider::OpenAI { api_key, model } => {
                self.send_openai_message(api_key, model, messages, None, max_tokens).await
            }
        }
    }

    async fn send_claude_message(
        &self,
        api_key: &str,
        model: &str,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
        max_tokens: u32,
    ) -> Result<AIResponse, AIError> {
        let mut body = json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        let response = self
            .http_client
            .post(CLAUDE_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", CLAUDE_API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::ApiError(format!(
                "Claude API error: {}",
                error_text
            )));
        }

        let claude_response: ClaudeAPIResponse = response.json().await.map_err(|e| {
            AIError::ParseError(format!("Failed to parse Claude response: {}", e))
        })?;

        Ok(AIResponse {
            id: claude_response.id,
            role: claude_response.role,
            content: claude_response.content,
            model: claude_response.model,
            stop_reason: claude_response.stop_reason,
            usage: Usage {
                input_tokens: claude_response.usage.input_tokens,
                output_tokens: claude_response.usage.output_tokens,
            },
        })
    }

    async fn send_openai_message(
        &self,
        api_key: &str,
        model: &str,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
        max_tokens: u32,
    ) -> Result<AIResponse, AIError> {
        // Convert messages to OpenAI format
        let openai_messages: Vec<Value> = messages
            .iter()
            .map(|m| json!({
                "role": m.role,
                "content": m.content,
            }))
            .collect();

        let mut body = json!({
            "model": model,
            "messages": openai_messages,
            "max_tokens": max_tokens,
        });

        // Convert tools to OpenAI format if provided
        if let Some(tools) = tools {
            let openai_tools: Vec<Value> = tools
                .iter()
                .map(|t| json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    }
                }))
                .collect();
            body["tools"] = json!(openai_tools);
            body["tool_choice"] = json!("auto");
        }

        let response = self
            .http_client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::ApiError(format!(
                "OpenAI API error: {}",
                error_text
            )));
        }

        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            AIError::ParseError(format!("Failed to parse OpenAI response: {}", e))
        })?;

        // Convert OpenAI response to our unified format
        let choice = openai_response.choices.first()
            .ok_or_else(|| AIError::ParseError("No choices in OpenAI response".to_string()))?;

        let mut content_blocks = Vec::new();

        // Add text content if present
        if let Some(text) = &choice.message.content {
            if !text.is_empty() {
                content_blocks.push(ContentBlock::Text {
                    text: text.clone(),
                });
            }
        }

        // Add tool calls if present
        if let Some(tool_calls) = &choice.message.tool_calls {
            for tool_call in tool_calls {
                let args: Value = serde_json::from_str(&tool_call.function.arguments)
                    .unwrap_or(json!({}));

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
            "length" => Some("max_tokens".to_string()),
            other => Some(other.to_string()),
        };

        Ok(AIResponse {
            id: openai_response.id,
            role: "assistant".to_string(),
            content: content_blocks,
            model: openai_response.model,
            stop_reason,
            usage: Usage {
                input_tokens: openai_response.usage.prompt_tokens,
                output_tokens: openai_response.usage.completion_tokens,
            },
        })
    }

    pub fn get_provider_name(&self) -> &str {
        match &self.provider {
            Provider::Claude { .. } => "Claude",
            Provider::OpenAI { .. } => "OpenAI",
        }
    }

    pub fn get_model_name(&self) -> &str {
        match &self.provider {
            Provider::Claude { model, .. } => model,
            Provider::OpenAI { model, .. } => model,
        }
    }
}

// Internal response structures for Claude
#[derive(Debug, Deserialize)]
struct ClaudeAPIResponse {
    id: String,
    #[serde(rename = "type")]
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

// Internal response structures for OpenAI
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
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
    total_tokens: u32,
}

// Response structure for OpenAI models list endpoint
#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelInfo {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}
