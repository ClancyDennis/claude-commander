// Conversation history management for MetaAgent
//
// This module handles the conversation history, message building,
// context tracking, and conversation-related utilities including
// context compaction for long-running sessions.

use crate::agent_runs_db::MetaMessageRecord;
use crate::ai_client::types::ImageSource;
use crate::ai_client::{Message, RichContentBlock, RichMessage, RichMessageContent};
use crate::types::{ChatMessage, ImageAttachment};

use super::context_config::ContextConfig;
use super::context_summarizer::ContextSummarizer;
use super::context_tracker::{ContextInfo, ContextState, ContextTracker};

/// Manages the conversation history for the MetaAgent
pub struct ConversationManager {
    history: Vec<Message>,
    context_tracker: ContextTracker,
    summarizer: ContextSummarizer,
    /// Summary of compacted context (prepended to history when needed)
    context_summary: Option<String>,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self::with_config(ContextConfig::default())
    }

    /// Create a new conversation manager with specific context configuration
    pub fn with_config(config: ContextConfig) -> Self {
        Self {
            history: Vec::new(),
            context_tracker: ContextTracker::new(config),
            summarizer: ContextSummarizer::new(),
            context_summary: None,
        }
    }

    /// Set the system prompt for token tracking
    pub fn set_system_prompt(&mut self, prompt: &str) {
        self.context_tracker.set_system_prompt(prompt);
    }

    /// Add a user message to the conversation history
    pub fn add_user_message(&mut self, content: String) {
        self.context_tracker.add_message_tokens(&content);
        self.history.push(Message {
            role: "user".to_string(),
            content,
        });
    }

    /// Add an assistant message to the conversation history
    pub fn add_assistant_message(&mut self, content: String) {
        self.context_tracker.add_message_tokens(&content);
        self.history.push(Message {
            role: "assistant".to_string(),
            content,
        });
    }

    /// Record token usage from an API response
    pub fn record_usage(&mut self, input_tokens: u32, output_tokens: u32) {
        self.context_tracker
            .record_usage(input_tokens, output_tokens);
    }

    /// Get context info for tool results
    pub fn get_context_info(&self) -> ContextInfo {
        self.context_tracker.get_context_info()
    }

    /// Get the current context state
    #[allow(dead_code)] // Utility method for potential API exposure
    pub fn get_context_state(&self) -> ContextState {
        self.context_tracker.get_state()
    }

    /// Check if compaction is needed
    #[allow(dead_code)] // Utility method for potential API exposure
    pub fn needs_compaction(&self) -> bool {
        self.context_tracker.needs_compaction()
    }

    /// Compact the conversation history if needed (call at idle moments)
    /// Returns true if compaction was performed
    pub async fn compact_if_needed(&mut self) -> bool {
        if !self.context_tracker.needs_compaction() {
            return false;
        }

        let is_emergency = self.context_tracker.needs_emergency_compaction();
        let preserve_count = if is_emergency {
            4 // Keep only 2 turns in emergency
        } else {
            self.context_tracker.config().preserve_recent_messages
        };

        if self.history.len() <= preserve_count {
            return false; // Nothing to compact
        }

        // Split history into messages to compact and messages to keep
        let split_point = self.history.len() - preserve_count;
        let messages_to_compact: Vec<Message> = self.history.drain(..split_point).collect();

        eprintln!(
            "[ConversationManager] Compacting {} messages (emergency: {})",
            messages_to_compact.len(),
            is_emergency
        );

        // Generate summary using the light model
        let summary_result = if is_emergency {
            self.summarizer
                .emergency_summarize(&messages_to_compact)
                .await
        } else {
            self.summarizer
                .summarize_messages(&messages_to_compact)
                .await
        };

        let summary = match summary_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "[ConversationManager] Summarization failed: {}, using fallback",
                    e
                );
                ContextSummarizer::fallback_summary(&messages_to_compact)
            }
        };

        // Store or append to existing summary
        let summary_tokens = ContextTracker::count_tokens(&summary);
        if let Some(existing) = &self.context_summary {
            self.context_summary = Some(format!("{}\n\n{}", existing, summary));
        } else {
            self.context_summary = Some(summary);
        }

        // Calculate remaining history tokens
        let remaining_tokens: usize = self
            .history
            .iter()
            .map(|m| ContextTracker::count_tokens(&m.content) + 4)
            .sum();

        // Reset the tracker with new token counts
        self.context_tracker
            .reset_after_compaction(summary_tokens, remaining_tokens);

        eprintln!(
            "[ConversationManager] Compaction complete. New context usage: {:.1}%",
            self.context_tracker.usage_percent()
        );

        true
    }

    /// Add a user message with an image attachment (stores text representation)
    pub fn add_user_message_with_image(&mut self, text: String, _image: &ImageAttachment) {
        let content = if text.is_empty() {
            "[Image attached]".to_string()
        } else {
            format!("[Image attached] {}", text)
        };
        self.add_user_message(content);
    }

    /// Get the conversation history as a slice
    pub fn get_history(&self) -> &[Message] {
        &self.history
    }

    /// Get a clone of the conversation history
    #[allow(dead_code)] // Utility method for potential API exposure
    pub fn clone_history(&self) -> Vec<Message> {
        self.history.clone()
    }

    /// Get conversation history with context summary prepended (for API calls)
    pub fn get_history_with_summary(&self) -> Vec<Message> {
        let mut result = Vec::new();

        // Prepend context summary as a system-like user message if available
        if let Some(summary) = &self.context_summary {
            result.push(Message {
                role: "user".to_string(),
                content: format!(
                    "[PREVIOUS CONTEXT - The following is a summary of earlier conversation that was compacted to save context space:]\n\n{}",
                    summary
                ),
            });
            result.push(Message {
                role: "assistant".to_string(),
                content: "I understand. I'll continue with this context in mind.".to_string(),
            });
        }

        result.extend(self.history.clone());
        result
    }

    /// Check if there is a context summary
    pub fn has_context_summary(&self) -> bool {
        self.context_summary.is_some()
    }

    /// Get the context summary if present
    #[allow(dead_code)] // Utility method for potential API exposure
    pub fn get_context_summary(&self) -> Option<&str> {
        self.context_summary.as_deref()
    }

    /// Clear the conversation history and reset context tracking
    pub fn clear(&mut self) {
        self.history.clear();
        self.context_summary = None;
        self.context_tracker = ContextTracker::new(self.context_tracker.config().clone());
    }

    /// Load conversation history from database records
    pub fn load_from_records(&mut self, records: &[MetaMessageRecord]) {
        // Clear existing history
        self.clear();

        // Load messages from records, sorted by message_index
        let mut sorted_records = records.to_vec();
        sorted_records.sort_by_key(|r| r.message_index);

        for record in sorted_records {
            self.context_tracker.add_message_tokens(&record.content);
            self.history.push(Message {
                role: record.role.clone(),
                content: record.content.clone(),
            });
        }

        eprintln!(
            "[ConversationManager] Loaded {} messages from records",
            self.history.len()
        );
    }

    /// Convert the conversation history to ChatMessage format for the frontend
    pub fn to_chat_messages(&self) -> Vec<ChatMessage> {
        self.history
            .iter()
            .enumerate()
            .map(|(i, msg)| ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                image: None,
                tool_calls: None,
                timestamp: chrono::Utc::now().timestamp_millis()
                    - ((self.history.len() - i) as i64 * 1000),
            })
            .collect()
    }

    /// Build rich messages for API calls that include images
    pub fn build_rich_messages(&self) -> Vec<RichMessage> {
        self.history
            .iter()
            .map(|msg| RichMessage {
                role: msg.role.clone(),
                content: RichMessageContent::Text(msg.content.clone()),
            })
            .collect()
    }

    /// Create rich content blocks for a message with an image
    pub fn build_image_content_blocks(
        text: &str,
        image: &ImageAttachment,
    ) -> Vec<RichContentBlock> {
        let mut content_blocks = vec![RichContentBlock::Image {
            source: ImageSource {
                source_type: "base64".to_string(),
                media_type: image.mime_type.clone(),
                data: image.base64_data.clone(),
            },
        }];

        // Add text if not empty
        if !text.is_empty() {
            content_blocks.push(RichContentBlock::Text {
                text: text.to_string(),
            });
        }

        content_blocks
    }

    /// Build rich messages with an image as the latest user message
    pub fn build_rich_messages_with_image(
        &self,
        text: &str,
        image: &ImageAttachment,
        system_prompt: &str,
    ) -> Vec<RichMessage> {
        // Start with system prompt as first message
        let mut messages = vec![
            RichMessage {
                role: "user".to_string(),
                content: RichMessageContent::Text(format!(
                    "System instructions (follow these for all interactions):\n\n{}",
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

        // Add existing conversation history
        messages.extend(self.build_rich_messages());

        // Add the new user message with image
        let content_blocks = Self::build_image_content_blocks(text, image);
        messages.push(RichMessage {
            role: "user".to_string(),
            content: RichMessageContent::Blocks(content_blocks),
        });

        messages
    }
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_messages() {
        let mut manager = ConversationManager::new();

        manager.add_user_message("Hello".to_string());
        manager.add_assistant_message("Hi there!".to_string());

        assert_eq!(manager.get_history().len(), 2);
        assert_eq!(manager.get_history()[0].role, "user");
        assert_eq!(manager.get_history()[1].role, "assistant");
    }

    #[test]
    fn test_clear_history() {
        let mut manager = ConversationManager::new();
        manager.add_user_message("Test".to_string());

        manager.clear();

        assert!(manager.get_history().is_empty());
        assert!(!manager.has_context_summary());
    }

    #[test]
    fn test_to_chat_messages() {
        let mut manager = ConversationManager::new();
        manager.add_user_message("Hello".to_string());

        let chat_messages = manager.to_chat_messages();

        assert_eq!(chat_messages.len(), 1);
        assert_eq!(chat_messages[0].role, "user");
        assert_eq!(chat_messages[0].content, "Hello");
    }

    #[test]
    fn test_context_tracking() {
        let mut manager = ConversationManager::new();
        manager.set_system_prompt("You are a helpful assistant.");

        manager.add_user_message("Hello".to_string());
        assert!(manager.get_context_info().usage_percent > 0.0);
    }

    #[test]
    fn test_history_with_summary() {
        let mut manager = ConversationManager::new();
        manager.add_user_message("Hello".to_string());

        // Without summary, should just return history
        let history = manager.get_history_with_summary();
        assert_eq!(history.len(), 1);
    }
}
