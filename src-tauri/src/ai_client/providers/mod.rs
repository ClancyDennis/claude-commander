pub mod claude;
pub mod openai;

pub use claude::ClaudeProvider;
pub use openai::OpenAIProvider;

use crate::ai_client::error::AIError;
use crate::ai_client::types::{AIResponse, Message, RichMessage, Tool};
use async_trait::async_trait;

/// Trait for AI provider implementations
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Send simple messages to the AI
    async fn send_message(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError>;

    /// Send simple messages to the AI with a system prompt
    async fn send_message_with_system(
        &self,
        system_prompt: &str,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError>;

    /// Send rich messages with structured content blocks
    async fn send_rich_message(
        &self,
        messages: Vec<RichMessage>,
        tools: Option<Vec<Tool>>,
    ) -> Result<AIResponse, AIError>;

    /// Get the provider name (e.g., "Claude", "OpenAI")
    fn name(&self) -> &str;

    /// Get the model being used
    fn model(&self) -> &str;
}
