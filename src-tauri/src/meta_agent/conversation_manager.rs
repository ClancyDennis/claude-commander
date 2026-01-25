// Conversation history management for MetaAgent
//
// This module handles the conversation history, message building,
// and conversation-related utilities.

use crate::ai_client::types::ImageSource;
use crate::ai_client::{Message, RichContentBlock, RichMessage, RichMessageContent};
use crate::types::{ChatMessage, ImageAttachment};

/// Manages the conversation history for the MetaAgent
pub struct ConversationManager {
    history: Vec<Message>,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Add a user message to the conversation history
    pub fn add_user_message(&mut self, content: String) {
        self.history.push(Message {
            role: "user".to_string(),
            content,
        });
    }

    /// Add an assistant message to the conversation history
    pub fn add_assistant_message(&mut self, content: String) {
        self.history.push(Message {
            role: "assistant".to_string(),
            content,
        });
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
    pub fn clone_history(&self) -> Vec<Message> {
        self.history.clone()
    }

    /// Clear the conversation history
    pub fn clear(&mut self) {
        self.history.clear();
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
}
