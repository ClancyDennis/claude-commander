// Tool loop engine for MetaAgent
//
// This module handles the iteration/tool loop logic that processes
// AI responses and executes tool calls until a final response is produced.

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::ai_client::{AIClient, AIResponse, ContentBlock, Message, RichMessage, Tool};
use crate::error::{ApiError, AppError, AppResult};
use crate::types::{
    ChatMessage, ChatResponse, ChatUsage, MetaAgentToolCallEvent, QueueStatus, ToolCall,
};

use super::tools;

/// Configuration for the tool loop engine
pub struct ToolLoopConfig {
    /// Maximum number of iterations before giving up
    pub max_iterations: usize,
    /// Maximum number of tool calls per message
    pub max_tool_calls: usize,
}

impl Default for ToolLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            max_tool_calls: 20,
        }
    }
}

/// Result of processing a single AI response
pub struct ResponseProcessingResult {
    /// Text content extracted from the response
    pub text_content: String,
    /// Tool calls that were made
    pub tool_calls: Vec<ToolCall>,
    /// Tool results to send back to the AI (if any tool calls were made)
    pub tool_results: Vec<serde_json::Value>,
    /// Number of tool calls in this response
    pub tool_call_count: usize,
}

/// The tool loop engine handles the iteration over AI responses and tool execution
pub struct ToolLoopEngine {
    config: ToolLoopConfig,
}

impl ToolLoopEngine {
    pub fn new() -> Self {
        Self {
            config: ToolLoopConfig::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_config(config: ToolLoopConfig) -> Self {
        Self { config }
    }

    /// Process a single AI response, executing any tool calls
    pub async fn process_response(
        &self,
        response: &AIResponse,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: &AppHandle,
        queue_status_fn: impl Fn() -> QueueStatus,
    ) -> ResponseProcessingResult {
        let mut text_content = String::new();
        let mut tool_calls = Vec::new();
        let mut tool_results = Vec::new();
        let mut tool_call_count = 0;

        for content_block in &response.content {
            match content_block {
                ContentBlock::Text { text } => {
                    text_content.push_str(text);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    tool_call_count += 1;

                    // Execute tool
                    let tool_result = tools::execute_tool(
                        name,
                        input.clone(),
                        agent_manager.clone(),
                        app_handle.clone(),
                        &queue_status_fn,
                    )
                    .await;

                    // Create tool call record
                    let tool_call = ToolCall {
                        id: id.clone(),
                        tool_name: name.clone(),
                        input: input.clone(),
                        output: Some(tool_result.clone()),
                    };
                    tool_calls.push(tool_call);

                    // Emit tool call event
                    let timestamp = chrono::Utc::now().timestamp_millis();
                    let _ = app_handle.emit(
                        "meta-agent:tool-call",
                        MetaAgentToolCallEvent {
                            tool_name: name.clone(),
                            input: input.clone(),
                            output: tool_result.clone(),
                            timestamp,
                        },
                    );

                    // Add tool result to send back to Claude
                    tool_results.push(serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": id,
                        "content": serde_json::to_string(&tool_result).unwrap_or_else(|_| "{}".to_string()),
                    }));
                }
            }
        }

        ResponseProcessingResult {
            text_content,
            tool_calls,
            tool_results,
            tool_call_count,
        }
    }

    /// Build the assistant message content for the conversation history
    pub fn build_assistant_content(
        response: &AIResponse,
        text_content: &str,
        has_tool_results: bool,
    ) -> String {
        if has_tool_results {
            serde_json::to_string(&response.content).unwrap_or_else(|_| text_content.to_string())
        } else {
            text_content.to_string()
        }
    }

    /// Build the tool results message content for the conversation history
    pub fn build_tool_results_content(tool_results: &[serde_json::Value]) -> String {
        serde_json::to_string(tool_results).unwrap_or_else(|_| "[]".to_string())
    }

    /// Build the final ChatResponse from processed results
    pub fn build_final_response(
        text_content: String,
        tool_calls: Vec<ToolCall>,
        usage: &crate::ai_client::Usage,
    ) -> ChatResponse {
        let timestamp = chrono::Utc::now().timestamp_millis();
        ChatResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content: text_content,
                image: None,
                tool_calls: if !tool_calls.is_empty() {
                    Some(tool_calls)
                } else {
                    None
                },
                timestamp,
            },
            usage: ChatUsage {
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
            },
        }
    }

    /// Run the main tool loop
    ///
    /// This function handles the iteration over AI responses, executing tool calls
    /// until a final response without tool calls is produced.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_loop<F>(
        &self,
        ai_client: &AIClient,
        system_prompt: &str,
        conversation_history: &mut Vec<Message>,
        tools: Vec<Tool>,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: &AppHandle,
        queue_status_fn: F,
    ) -> AppResult<ChatResponse>
    where
        F: Fn() -> QueueStatus,
    {
        let mut iteration = 0;
        let mut tool_call_count = 0;
        let mut final_response: Option<ChatResponse> = None;

        while iteration < self.config.max_iterations && tool_call_count < self.config.max_tool_calls
        {
            iteration += 1;

            // Call AI API with system prompt and tools
            let response = ai_client
                .send_message_with_system_and_tools(
                    system_prompt,
                    conversation_history.clone(),
                    tools.clone(),
                )
                .await
                .map_err(|e| AppError::Api(ApiError::Network(format!("AI API error: {}", e))))?;

            // Process the response
            let result = self
                .process_response(
                    &response,
                    agent_manager.clone(),
                    app_handle,
                    &queue_status_fn,
                )
                .await;

            tool_call_count += result.tool_call_count;

            // Add assistant message to history
            let assistant_content = Self::build_assistant_content(
                &response,
                &result.text_content,
                !result.tool_results.is_empty(),
            );

            conversation_history.push(Message {
                role: "assistant".to_string(),
                content: assistant_content,
            });

            // If there were tool calls, send tool results back
            if !result.tool_results.is_empty() {
                conversation_history.push(Message {
                    role: "user".to_string(),
                    content: Self::build_tool_results_content(&result.tool_results),
                });
                continue; // Continue the loop to get final response
            }

            // No tool calls - we have our final response
            final_response = Some(Self::build_final_response(
                result.text_content,
                result.tool_calls,
                &response.usage,
            ));
            break;
        }

        final_response.ok_or_else(|| {
            AppError::Internal(
                "Meta-agent failed to produce a response within iteration limits".to_string(),
            )
        })
    }

    /// Run the tool loop with rich messages (for image support)
    ///
    /// This is used on the first iteration when processing a message with an image.
    /// Subsequent iterations use the regular loop.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_loop_with_initial_rich_messages<F>(
        &self,
        ai_client: &AIClient,
        system_prompt: &str,
        initial_rich_messages: Vec<RichMessage>,
        conversation_history: &mut Vec<Message>,
        tools: Vec<Tool>,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: &AppHandle,
        queue_status_fn: F,
    ) -> AppResult<ChatResponse>
    where
        F: Fn() -> QueueStatus,
    {
        let mut iteration = 0;
        let mut tool_call_count = 0;
        let mut final_response: Option<ChatResponse> = None;

        while iteration < self.config.max_iterations && tool_call_count < self.config.max_tool_calls
        {
            iteration += 1;

            // For first iteration, use rich messages with image
            // For subsequent iterations, use simple messages
            let response = if iteration == 1 {
                ai_client
                    .send_rich_message_with_tools(initial_rich_messages.clone(), tools.clone())
                    .await
                    .map_err(|e| AppError::Api(ApiError::Network(format!("AI API error: {}", e))))?
            } else {
                ai_client
                    .send_message_with_system_and_tools(
                        system_prompt,
                        conversation_history.clone(),
                        tools.clone(),
                    )
                    .await
                    .map_err(|e| AppError::Api(ApiError::Network(format!("AI API error: {}", e))))?
            };

            // Process the response
            let result = self
                .process_response(
                    &response,
                    agent_manager.clone(),
                    app_handle,
                    &queue_status_fn,
                )
                .await;

            tool_call_count += result.tool_call_count;

            // Add assistant message to history
            let assistant_content = Self::build_assistant_content(
                &response,
                &result.text_content,
                !result.tool_results.is_empty(),
            );

            conversation_history.push(Message {
                role: "assistant".to_string(),
                content: assistant_content,
            });

            // If there were tool calls, send tool results back
            if !result.tool_results.is_empty() {
                conversation_history.push(Message {
                    role: "user".to_string(),
                    content: Self::build_tool_results_content(&result.tool_results),
                });
                continue;
            }

            // No tool calls - we have our final response
            final_response = Some(Self::build_final_response(
                result.text_content,
                result.tool_calls,
                &response.usage,
            ));
            break;
        }

        final_response.ok_or_else(|| {
            AppError::Internal(
                "Meta-agent failed to produce a response within iteration limits".to_string(),
            )
        })
    }
}

impl Default for ToolLoopEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ToolLoopConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.max_tool_calls, 20);
    }

    #[test]
    fn test_build_tool_results_content() {
        let results = vec![serde_json::json!({"type": "tool_result", "tool_use_id": "123"})];
        let content = ToolLoopEngine::build_tool_results_content(&results);
        assert!(content.contains("tool_result"));
        assert!(content.contains("123"));
    }
}
