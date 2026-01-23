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
    /// Tries Claude first (ANTHROPIC_API_KEY), then OpenAI (OPENAI_API_KEY)
    pub fn from_env() -> Result<Self, AIError> {
        // Try Claude first
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let model = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| models::get_default_claude_model());
            return Ok(Self::new(Provider::Claude { api_key, model }));
        }

        // Try OpenAI second
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
            return Ok(Self::new(Provider::OpenAI { api_key, model }));
        }

        Err(AIError::ConfigError(
            "No API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable"
                .to_string(),
        ))
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
