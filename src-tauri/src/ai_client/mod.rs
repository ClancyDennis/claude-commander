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
    /// Provider is inferred from the ANTHROPIC_MODEL setting:
    /// - If model starts with "gpt-" or "o1-" or "o3-" → use OpenAI
    /// - Otherwise → use Anthropic (claude-* or aliases like sonnet/opus/haiku)
    ///
    /// META_AGENT_PROVIDER can override this: "anthropic" or "openai" forces that provider
    pub fn from_env() -> Result<Self, AIError> {
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());
        let openai_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());

        // Get the model setting (ANTHROPIC_MODEL is the "primary model" in the UI)
        let model = std::env::var("ANTHROPIC_MODEL")
            .ok()
            .filter(|m| !m.is_empty());

        // Check for explicit provider override
        let provider_override = std::env::var("META_AGENT_PROVIDER")
            .ok()
            .filter(|p| !p.is_empty() && p.to_lowercase() != "auto");

        // Determine provider: explicit override > infer from model name > auto (Claude first)
        let use_openai = if let Some(ref override_val) = provider_override {
            override_val.to_lowercase() == "openai"
        } else if let Some(ref m) = model {
            Self::is_openai_model(m)
        } else {
            // No model set, no override - use auto logic (Claude first if key exists)
            anthropic_key.is_none() && openai_key.is_some()
        };

        if use_openai {
            if let Some(api_key) = openai_key {
                // Use ANTHROPIC_MODEL if it's an OpenAI model, otherwise fall back to OPENAI_MODEL
                let openai_model = if model
                    .as_ref()
                    .map(|m| Self::is_openai_model(m))
                    .unwrap_or(false)
                {
                    model.unwrap()
                } else {
                    std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string())
                };
                return Ok(Self::new(Provider::OpenAI {
                    api_key,
                    model: openai_model,
                }));
            }
            return Err(AIError::ConfigError(
                "OpenAI provider selected but OPENAI_API_KEY is not configured".to_string(),
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
            let openai_model =
                std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            return Ok(Self::new(Provider::OpenAI {
                api_key,
                model: openai_model,
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
    /// Uses LIGHT_TASK_MODEL env var, defaults to claude-haiku-4-5-20251101.
    /// Falls back to OpenAI gpt-4o-mini if no Anthropic key is available.
    pub fn light_from_env() -> Result<Self, AIError> {
        let light_model = std::env::var("LIGHT_TASK_MODEL")
            .unwrap_or_else(|_| "claude-haiku-4-5-20251101".to_string());

        // Try Anthropic first
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            if !api_key.is_empty() {
                return Ok(Self::new(Provider::Claude {
                    api_key,
                    model: light_model,
                }));
            }
        }

        // Fall back to OpenAI with gpt-4o-mini
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                return Ok(Self::new(Provider::OpenAI {
                    api_key,
                    model: "gpt-4o-mini".to_string(),
                }));
            }
        }

        Err(AIError::ConfigError(
            "No API key available for light model. Set ANTHROPIC_API_KEY or OPENAI_API_KEY"
                .to_string(),
        ))
    }

    /// Create an AI client for security monitoring
    ///
    /// Uses SECURITY_MODEL env var, defaults to the main Claude model.
    /// Falls back to OpenAI if no Anthropic key is available.
    pub fn security_from_env() -> Result<Self, AIError> {
        let security_model =
            std::env::var("SECURITY_MODEL").unwrap_or_else(|_| models::get_default_claude_model());

        // Try Anthropic first
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            if !api_key.is_empty() {
                return Ok(Self::new(Provider::Claude {
                    api_key,
                    model: security_model,
                }));
            }
        }

        // Fall back to OpenAI
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                // For OpenAI fallback, use gpt-4o for security (needs reasoning capability)
                return Ok(Self::new(Provider::OpenAI {
                    api_key,
                    model: "gpt-4o".to_string(),
                }));
            }
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
