//! AI Client module providing a unified interface for multiple AI providers.
//!
//! This module supports Claude (Anthropic) and OpenAI providers with a common
//! interface for sending messages, handling tool calls, and managing conversations.

pub mod error;
pub mod models;
pub mod providers;
pub mod types;

pub use error::AIError;
pub use providers::{AIProvider, ClaudeProvider, OpenAIProvider};
pub use types::{
    AIResponse, ContentBlock, Message, Provider, RichContentBlock, RichMessage, RichMessageContent,
    Tool, Usage,
};

use std::sync::Arc;

/// Main AI client that wraps provider-specific implementations
pub struct AIClient {
    provider: Arc<dyn AIProvider>,
}

impl AIClient {
    /// Create a new AIClient with the specified provider configuration
    pub fn new(provider: Provider) -> Self {
        let provider: Arc<dyn AIProvider> = match provider {
            Provider::Claude { api_key, model } => Arc::new(ClaudeProvider::new(api_key, model)),
            Provider::OpenAI { api_key, model } => Arc::new(OpenAIProvider::new(api_key, model)),
        };

        Self { provider }
    }

    /// Create an AIClient from environment variables
    ///
    /// Provider is inferred from the PRIMARY_MODEL setting:
    /// - If model starts with "gpt-" or "o1-" or "o3-" → use OpenAI
    /// - Otherwise → use Anthropic (claude-* or aliases like sonnet/opus/haiku)
    pub fn from_env() -> Result<Self, AIError> {
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());
        let openai_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());

        // Get the primary model setting
        let model = std::env::var("PRIMARY_MODEL")
            .ok()
            .filter(|m| !m.is_empty());

        // Infer provider from model name, or fall back to Claude if available
        let use_openai = if let Some(ref m) = model {
            Self::is_openai_model(m)
        } else {
            // No model set - use Claude first if key exists, otherwise OpenAI
            anthropic_key.is_none() && openai_key.is_some()
        };

        if use_openai {
            if let Some(api_key) = openai_key {
                let openai_model = model.unwrap_or_else(|| "gpt-4o".to_string());
                return Ok(Self::new(Provider::OpenAI {
                    api_key,
                    model: openai_model,
                }));
            }
            return Err(AIError::ConfigError(
                "OpenAI model selected but OPENAI_API_KEY is not configured".to_string(),
            ));
        }

        // Use Anthropic
        if let Some(api_key) = anthropic_key {
            let claude_model = model.unwrap_or_else(models::get_default_claude_model);
            return Ok(Self::new(Provider::Claude {
                api_key,
                model: claude_model,
            }));
        }

        // Last resort: try OpenAI if available
        if let Some(api_key) = openai_key {
            return Ok(Self::new(Provider::OpenAI {
                api_key,
                model: "gpt-4o".to_string(),
            }));
        }

        Err(AIError::ConfigError(
            "No API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable"
                .to_string(),
        ))
    }

    /// Check if a model name indicates OpenAI
    fn is_openai_model(model: &str) -> bool {
        let lower = model.to_lowercase();
        lower.starts_with("gpt-") || lower.starts_with("o1-") || lower.starts_with("o3-")
    }

    /// Create an OpenAI-based client (preferred for orchestration)
    pub fn openai_from_env() -> Result<Self, AIError> {
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let model = std::env::var("OPENAI_ORCHESTRATOR_MODEL")
                .or_else(|_| std::env::var("OPENAI_MODEL"))
                .unwrap_or_else(|_| "gpt-4o".to_string());
            return Ok(Self::new(Provider::OpenAI { api_key, model }));
        }
        Err(AIError::ConfigError(
            "OPENAI_API_KEY environment variable not set".to_string(),
        ))
    }

    /// Create a light/fast AI client for simple tasks (summarization, prompt generation)
    ///
    /// Uses LIGHT_TASK_MODEL env var (provider inferred from model name).
    /// Defaults to claude-haiku-4-5 for Claude, gpt-4o-mini for OpenAI.
    pub fn light_from_env() -> Result<Self, AIError> {
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());
        let openai_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());

        let model = std::env::var("LIGHT_TASK_MODEL")
            .ok()
            .filter(|m| !m.is_empty());

        // Infer provider from model name
        let use_openai = if let Some(ref m) = model {
            Self::is_openai_model(m)
        } else {
            anthropic_key.is_none() && openai_key.is_some()
        };

        if use_openai {
            if let Some(api_key) = openai_key {
                let openai_model = model.unwrap_or_else(|| "gpt-4o-mini".to_string());
                return Ok(Self::new(Provider::OpenAI {
                    api_key,
                    model: openai_model,
                }));
            }
            return Err(AIError::ConfigError(
                "OpenAI light model selected but OPENAI_API_KEY is not configured".to_string(),
            ));
        }

        // Use Anthropic
        if let Some(api_key) = anthropic_key {
            let claude_model = model.unwrap_or_else(|| "claude-haiku-4-5".to_string());
            return Ok(Self::new(Provider::Claude {
                api_key,
                model: claude_model,
            }));
        }

        // Last resort: try OpenAI
        if let Some(api_key) = openai_key {
            return Ok(Self::new(Provider::OpenAI {
                api_key,
                model: "gpt-4o-mini".to_string(),
            }));
        }

        Err(AIError::ConfigError(
            "No API key available for light model. Set ANTHROPIC_API_KEY or OPENAI_API_KEY"
                .to_string(),
        ))
    }

    /// Create an AI client for security monitoring
    ///
    /// Uses SECURITY_MODEL env var (provider inferred from model name).
    /// Defaults to the main Claude model for Claude, gpt-4o for OpenAI.
    pub fn security_from_env() -> Result<Self, AIError> {
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());
        let openai_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());

        let model = std::env::var("SECURITY_MODEL")
            .ok()
            .filter(|m| !m.is_empty());

        // Infer provider from model name
        let use_openai = if let Some(ref m) = model {
            Self::is_openai_model(m)
        } else {
            anthropic_key.is_none() && openai_key.is_some()
        };

        if use_openai {
            if let Some(api_key) = openai_key {
                let openai_model = model.unwrap_or_else(|| "gpt-4o".to_string());
                return Ok(Self::new(Provider::OpenAI {
                    api_key,
                    model: openai_model,
                }));
            }
            return Err(AIError::ConfigError(
                "OpenAI security model selected but OPENAI_API_KEY is not configured".to_string(),
            ));
        }

        // Use Anthropic
        if let Some(api_key) = anthropic_key {
            let claude_model = model.unwrap_or_else(models::get_default_claude_model);
            return Ok(Self::new(Provider::Claude {
                api_key,
                model: claude_model,
            }));
        }

        // Last resort: try OpenAI
        if let Some(api_key) = openai_key {
            return Ok(Self::new(Provider::OpenAI {
                api_key,
                model: "gpt-4o".to_string(),
            }));
        }

        Err(AIError::ConfigError(
            "No API key available for security model. Set ANTHROPIC_API_KEY or OPENAI_API_KEY"
                .to_string(),
        ))
    }

    /// Send messages with optional tool definitions
    pub async fn send_message_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<Tool>,
    ) -> Result<AIResponse, AIError> {
        self.provider.send_message(messages, Some(tools)).await
    }

    /// Send messages with a system prompt and tools
    pub async fn send_message_with_system_and_tools(
        &self,
        system_prompt: &str,
        messages: Vec<Message>,
        tools: Vec<Tool>,
    ) -> Result<AIResponse, AIError> {
        self.provider
            .send_message_with_system(system_prompt, messages, Some(tools))
            .await
    }

    /// Send simple messages without tools
    pub async fn send_message(&self, messages: Vec<Message>) -> Result<AIResponse, AIError> {
        self.provider.send_message(messages, None).await
    }

    /// Send rich messages with structured content blocks (for multi-turn tool conversations)
    pub async fn send_rich_message_with_tools(
        &self,
        messages: Vec<RichMessage>,
        tools: Vec<Tool>,
    ) -> Result<AIResponse, AIError> {
        self.provider.send_rich_message(messages, Some(tools)).await
    }

    /// Send rich messages with a system prompt and tools
    ///
    /// The system prompt is prepended as a user/assistant exchange since the
    /// rich message API doesn't have a dedicated system prompt parameter.
    pub async fn send_rich_message_with_system_and_tools(
        &self,
        system_prompt: &str,
        messages: Vec<RichMessage>,
        tools: Vec<Tool>,
    ) -> Result<AIResponse, AIError> {
        let mut full_messages = vec![
            RichMessage {
                role: "user".to_string(),
                content: RichMessageContent::Text(format!(
                    "System instructions:\n\n{}",
                    system_prompt
                )),
            },
            RichMessage {
                role: "assistant".to_string(),
                content: RichMessageContent::Text(
                    "I understand and will follow these instructions.".to_string(),
                ),
            },
        ];
        full_messages.extend(messages);
        self.provider
            .send_rich_message(full_messages, Some(tools))
            .await
    }

    /// Get the provider name (e.g., "Claude", "OpenAI")
    pub fn get_provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Get the model name being used
    pub fn get_model_name(&self) -> &str {
        self.provider.model()
    }

    // Re-export model functions for backwards compatibility

    /// Returns the latest recommended Claude model
    pub fn get_default_claude_model() -> String {
        models::get_default_claude_model()
    }

    /// List available Claude models
    pub async fn list_claude_models(api_key: &str) -> Result<Vec<String>, AIError> {
        models::list_claude_models(api_key).await
    }

    /// List available OpenAI models
    pub async fn list_openai_models(api_key: &str) -> Result<Vec<String>, AIError> {
        models::list_openai_models(api_key).await
    }
}
