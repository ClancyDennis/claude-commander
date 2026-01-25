//! Expectation generation from user prompts.
//!
//! This module provides LLM-based analysis of user prompts to predict
//! expected tools, paths, and behaviors for security monitoring.

use std::collections::HashSet;

use crate::ai_client::{AIClient, AIResponse, ContentBlock, Message, Tool};

use super::prompts::{format_expectation_generation_message, EXPECTATION_GENERATION_SYSTEM_PROMPT};
use super::session_expectations::InitialExpectations;

/// Generator for initial expectations from user prompts.
///
/// Uses an LLM to analyze the user's task prompt and predict what tools,
/// paths, and behaviors the agent will likely need.
pub struct ExpectationGenerator<'a> {
    ai_client: &'a AIClient,
}

impl<'a> ExpectationGenerator<'a> {
    /// Create a new ExpectationGenerator with a reference to an AI client.
    pub fn new(ai_client: &'a AIClient) -> Self {
        Self { ai_client }
    }

    /// Generate initial expectations from a user prompt.
    ///
    /// This analyzes the prompt to predict what tools, paths, and behaviors
    /// the agent will likely need, enabling immediate anomaly detection.
    pub async fn generate(
        &self,
        prompt: &str,
        working_dir: &str,
    ) -> Result<InitialExpectations, String> {
        let user_message = format_expectation_generation_message(working_dir, prompt);

        let messages = vec![Message {
            role: "user".to_string(),
            content: format!(
                "{}\n\n{}",
                EXPECTATION_GENERATION_SYSTEM_PROMPT, user_message
            ),
        }];

        // Create the expectations tool
        let expectations_tool = create_tool();

        // Send to LLM
        let response = self
            .ai_client
            .send_message_with_tools(messages, vec![expectations_tool])
            .await
            .map_err(|e| format!("LLM expectation generation failed: {}", e))?;

        // Parse response
        parse_response(response)
    }
}

/// Create the tool definition for structured expectations output.
pub(crate) fn create_tool() -> Tool {
    Tool {
        name: "generate_expectations".to_string(),
        description: "Report the expected tools, paths, and behaviors for a user task".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "required": ["expected_tools", "expected_path_patterns", "network_likely", "destructive_likely", "confidence", "reasoning"],
            "properties": {
                "expected_tools": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of tool names the agent will likely use: Read, Write, Edit, Bash, Glob, Grep, WebFetch, WebSearch, Task, TodoWrite"
                },
                "expected_path_patterns": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "File path patterns the agent will likely access. Use glob syntax: 'README.md', '*.rs', 'src/**/*.ts'. Relative to working directory unless absolute."
                },
                "network_likely": {
                    "type": "boolean",
                    "description": "Will the task likely require network access (web fetch, API calls)?"
                },
                "destructive_likely": {
                    "type": "boolean",
                    "description": "Will the task likely involve destructive operations (delete files, overwrite)?"
                },
                "bash_patterns": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Expected bash command patterns if Bash is needed: 'npm *', 'cargo *', 'git *'"
                },
                "confidence": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1,
                    "description": "Confidence in these predictions (0-1)"
                },
                "reasoning": {
                    "type": "string",
                    "description": "Brief explanation of why these tools/paths are expected"
                }
            }
        }),
    }
}

/// Parse the LLM response into InitialExpectations.
pub(crate) fn parse_response(response: AIResponse) -> Result<InitialExpectations, String> {
    // Look for tool use in response
    for content in response.content {
        if let ContentBlock::ToolUse { name, input, .. } = content {
            if name == "generate_expectations" {
                return parse_input(input);
            }
        }
    }

    // If no tool use, return defaults
    Ok(InitialExpectations::default())
}

/// Parse the tool input into InitialExpectations.
pub(crate) fn parse_input(input: serde_json::Value) -> Result<InitialExpectations, String> {
    let expected_tools: HashSet<String> = input
        .get("expected_tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(|| {
            // Default safe tools
            ["Read", "Glob", "Grep", "Edit", "TodoWrite"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        });

    let expected_path_patterns: Vec<String> = input
        .get("expected_path_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(|| vec!["**/*".to_string()]);

    let network_likely = input
        .get("network_likely")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let destructive_likely = input
        .get("destructive_likely")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let bash_patterns: Vec<String> = input
        .get("bash_patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let confidence = input
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.7);

    let reasoning = input
        .get("reasoning")
        .and_then(|v| v.as_str())
        .unwrap_or("LLM-generated expectations")
        .to_string();

    Ok(InitialExpectations {
        expected_tools,
        expected_path_patterns,
        network_likely,
        destructive_likely,
        bash_patterns,
        confidence,
        reasoning,
    })
}
