// Context summarization for conversation compaction
//
// This module uses the LIGHT_TASK_MODEL (haiku) to generate quality summaries
// of conversation history during context compaction.

use crate::ai_client::{AIClient, Message, Provider};
use crate::commands::config_loader::env_keys;

/// Default light model for summarization
const DEFAULT_LIGHT_MODEL: &str = "claude-haiku-4-5-20251101";

/// Summarizer for compacting conversation history
pub struct ContextSummarizer {
    light_model: String,
}

impl ContextSummarizer {
    /// Create a new context summarizer
    pub fn new() -> Self {
        let light_model = std::env::var(env_keys::LIGHT_TASK_MODEL)
            .unwrap_or_else(|_| DEFAULT_LIGHT_MODEL.to_string());

        Self { light_model }
    }

    /// Create an AI client configured with the light model
    fn create_light_client(&self) -> Result<AIClient, String> {
        // Try Anthropic first
        if let Ok(api_key) = std::env::var(env_keys::ANTHROPIC_API_KEY) {
            if !api_key.is_empty() {
                return Ok(AIClient::new(Provider::Claude {
                    api_key,
                    model: self.light_model.clone(),
                }));
            }
        }

        // Fall back to OpenAI
        if let Ok(api_key) = std::env::var(env_keys::OPENAI_API_KEY) {
            if !api_key.is_empty() {
                // For OpenAI, use gpt-4o-mini as the "light" model
                return Ok(AIClient::new(Provider::OpenAI {
                    api_key,
                    model: "gpt-4o-mini".to_string(),
                }));
            }
        }

        Err("No API key available for summarization".to_string())
    }

    /// Generate a summary of messages being compacted
    pub async fn summarize_messages(&self, messages: &[Message]) -> Result<String, String> {
        if messages.is_empty() {
            return Ok(String::new());
        }

        let client = self.create_light_client()?;

        // Build the content to summarize
        let content_to_summarize = messages
            .iter()
            .map(|m| format!("[{}]: {}", m.role, truncate_for_summary(&m.content, 500)))
            .collect::<Vec<_>>()
            .join("\n\n");

        // Create the summarization prompt
        let prompt = format!(
            r#"Summarize the following conversation history concisely. Focus on:
1. Key decisions made
2. Important task progress or completed steps
3. Critical identifiers (agent IDs, file paths, variable names)
4. Any unresolved issues or pending actions

Keep the summary under 500 words. Be factual and specific.

CONVERSATION:
{}

SUMMARY:"#,
            content_to_summarize
        );

        // Send to the light model
        let response = client
            .send_message(vec![Message {
                role: "user".to_string(),
                content: prompt,
            }])
            .await
            .map_err(|e| format!("Summarization failed: {}", e))?;

        // Extract text content from response
        let summary = response
            .content
            .iter()
            .filter_map(|block| {
                if let crate::ai_client::ContentBlock::Text { text } = block {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if summary.is_empty() {
            // Fallback: create a basic summary from message count
            Ok(format!(
                "[Previous context: {} messages compacted. Unable to generate detailed summary.]",
                messages.len()
            ))
        } else {
            Ok(format!(
                "[CONTEXT SUMMARY - {} messages compacted]\n{}",
                messages.len(),
                summary.trim()
            ))
        }
    }

    /// Generate an emergency summary (shorter, for critical situations)
    pub async fn emergency_summarize(&self, messages: &[Message]) -> Result<String, String> {
        if messages.is_empty() {
            return Ok(String::new());
        }

        let client = self.create_light_client()?;

        // Build condensed content
        let content_to_summarize = messages
            .iter()
            .map(|m| format!("[{}]: {}", m.role, truncate_for_summary(&m.content, 200)))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"Create a very brief summary (under 200 words) of this conversation. Focus only on:
1. Current task status
2. Most recent actions
3. Any critical identifiers

CONVERSATION:
{}

BRIEF SUMMARY:"#,
            content_to_summarize
        );

        let response = client
            .send_message(vec![Message {
                role: "user".to_string(),
                content: prompt,
            }])
            .await
            .map_err(|e| format!("Emergency summarization failed: {}", e))?;

        let summary = response
            .content
            .iter()
            .filter_map(|block| {
                if let crate::ai_client::ContentBlock::Text { text } = block {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if summary.is_empty() {
            Ok(format!(
                "[EMERGENCY COMPACTION: {} messages removed]",
                messages.len()
            ))
        } else {
            Ok(format!(
                "[EMERGENCY CONTEXT SUMMARY - {} messages compacted]\n{}",
                messages.len(),
                summary.trim()
            ))
        }
    }

    /// Create a fallback summary without calling the AI (for when API is unavailable)
    pub fn fallback_summary(messages: &[Message]) -> String {
        if messages.is_empty() {
            return String::new();
        }

        // Extract key information heuristically
        let mut agent_ids: Vec<String> = Vec::new();
        let mut recent_actions: Vec<String> = Vec::new();

        for msg in messages.iter().rev().take(5) {
            // Look for agent IDs
            if let Some(start) = msg.content.find("agent_id") {
                let snippet = &msg.content[start..];
                if let Some(end) = snippet.find([',', '}', '\n']) {
                    agent_ids.push(snippet[..end].to_string());
                }
            }

            // Capture first 100 chars of recent messages
            if msg.role == "assistant" && recent_actions.len() < 3 {
                let truncated = truncate_for_summary(&msg.content, 100);
                recent_actions.push(truncated);
            }
        }

        let mut summary = format!(
            "[CONTEXT COMPACTED - {} messages removed]\n",
            messages.len()
        );

        if !agent_ids.is_empty() {
            summary.push_str(&format!("Active agents: {}\n", agent_ids.join(", ")));
        }

        if !recent_actions.is_empty() {
            summary.push_str("Recent context:\n");
            for action in recent_actions {
                summary.push_str(&format!("- {}\n", action));
            }
        }

        summary
    }
}

impl Default for ContextSummarizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate content for inclusion in summary prompts
fn truncate_for_summary(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        content.to_string()
    } else {
        format!("{}...", &content[..max_chars])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_for_summary() {
        let short = "Hello";
        assert_eq!(truncate_for_summary(short, 100), "Hello");

        let long = "a".repeat(200);
        let truncated = truncate_for_summary(&long, 50);
        assert_eq!(truncated.len(), 53); // 50 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_fallback_summary() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Start the task".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "Starting agent with agent_id: test-123".to_string(),
            },
        ];

        let summary = ContextSummarizer::fallback_summary(&messages);
        assert!(summary.contains("2 messages"));
    }

    #[test]
    fn test_empty_fallback() {
        let summary = ContextSummarizer::fallback_summary(&[]);
        assert!(summary.is_empty());
    }
}
