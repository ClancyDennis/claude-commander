// Context summarization for conversation compaction
//
// This module uses the LIGHT_TASK_MODEL (haiku) to generate quality summaries
// of conversation history during context compaction.

use crate::ai_client::{AIClient, Message};

/// Summarizer for compacting conversation history
pub struct ContextSummarizer;

impl ContextSummarizer {
    /// Create a new context summarizer
    pub fn new() -> Self {
        Self
    }

    /// Generate a summary of messages being compacted
    pub async fn summarize_messages(&self, messages: &[Message]) -> Result<String, String> {
        if messages.is_empty() {
            return Ok(String::new());
        }

        let client = AIClient::light_from_env()
            .map_err(|e| format!("Failed to create light client: {}", e))?;

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
        eprintln!(
            "[LLM][{}][{}] Summarizing {} messages",
            client.get_provider_name(),
            client.get_model_name(),
            messages.len()
        );
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

        let client = AIClient::light_from_env()
            .map_err(|e| format!("Failed to create light client: {}", e))?;

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

        eprintln!(
            "[LLM][{}][{}] Emergency summarizing {} messages",
            client.get_provider_name(),
            client.get_model_name(),
            messages.len()
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
