// Orchestrator Agent Types
//
// Type definitions for the orchestrator agent's conversation model and actions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Action requested by the orchestrator
#[derive(Debug, Clone)]
pub enum OrchestratorAction {
    /// Continue the tool loop - no phase change
    Continue,
    /// Transition to planning phase
    StartPlanning { summary: String },
    /// Approve the plan and proceed
    ApprovePlan { assessment: String },
    /// Start execution phase
    StartExecution { notes: Option<String> },
    /// Start verification phase
    StartVerification { focus_areas: Vec<String> },
    /// Pipeline completed successfully
    Complete { summary: String },
    /// Iterate on execution (fix issues)
    Iterate { issues: Vec<String>, suggestions: Vec<String> },
    /// Go back to planning
    Replan { reason: String, issues: Vec<String>, suggestions: Vec<String> },
    /// Give up on the pipeline
    GiveUp { reason: String },
}

/// Message in the orchestrator's conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: ConversationContent,
}

/// Content of a conversation message (text or tool results)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConversationContent {
    Text(String),
    Blocks(Vec<ContentBlockValue>),
}

/// A content block that can be text, tool_use, or tool_result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockValue {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: Value },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_message_serialization() {
        let msg = ConversationMessage {
            role: "user".to_string(),
            content: ConversationContent::Text("Hello".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }
}
