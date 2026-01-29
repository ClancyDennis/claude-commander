// Tool loop engine for MetaAgent
//
// This module handles the iteration/tool loop logic that processes
// AI responses and executes tool calls until a final response is produced.

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::ai_client::{AIClient, AIResponse, ContentBlock, Message, RichMessage, Tool, Usage};
use crate::error::{ApiError, AppError, AppResult};
use crate::types::{
    ChatMessage, ChatResponse, ChatUsage, MetaAgentToolCallEvent, QueueStatus, ToolCall,
};

use super::context_tracker::ContextInfo;
use super::output_compressor::OutputCompressor;
use super::tools::{self, IterationContext, PendingQuestion, SleepState, ToolExecutionResult};

/// Configuration for the tool loop engine
pub struct ToolLoopConfig {
    /// Maximum number of iterations before giving up
    pub max_iterations: usize,
    /// Maximum number of tool calls per message
    pub max_tool_calls: usize,
    /// Maximum characters for tool output before compression
    pub max_tool_output_chars: usize,
}

impl Default for ToolLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 40,
            max_tool_calls: 200,
            max_tool_output_chars: 10_000,
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
    /// Whether CompleteTask was called - signals loop should exit
    pub should_complete: bool,
    /// The completion message from CompleteTask (if called)
    pub completion_message: Option<String>,
    /// Whether Sleep completed - signals iteration counter should reset
    pub should_reset_iterations: bool,
}

/// Extended ChatResponse that includes cumulative usage
pub struct ToolLoopResult {
    /// The chat response
    pub response: ChatResponse,
    /// Cumulative token usage across all iterations
    pub total_usage: Usage,
}

/// The tool loop engine handles the iteration over AI responses and tool execution
pub struct ToolLoopEngine {
    config: ToolLoopConfig,
    output_compressor: OutputCompressor,
}

impl ToolLoopEngine {
    pub fn new() -> Self {
        let config = ToolLoopConfig::default();
        let output_compressor = OutputCompressor::new(config.max_tool_output_chars);
        Self {
            config,
            output_compressor,
        }
    }

    #[allow(dead_code)]
    pub fn with_config(config: ToolLoopConfig) -> Self {
        let output_compressor = OutputCompressor::new(config.max_tool_output_chars);
        Self {
            config,
            output_compressor,
        }
    }

    /// Process a single AI response, executing any tool calls
    #[allow(clippy::too_many_arguments)]
    pub async fn process_response(
        &self,
        response: &AIResponse,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: &AppHandle,
        sleep_state: Arc<Mutex<SleepState>>,
        pending_question: Arc<Mutex<Option<PendingQuestion>>>,
        queue_status_fn: impl Fn() -> QueueStatus,
        iteration_ctx: IterationContext,
    ) -> ResponseProcessingResult {
        let mut text_content = String::new();
        let mut tool_calls = Vec::new();
        let mut tool_results = Vec::new();
        let mut tool_call_count = 0;
        let mut should_complete = false;
        let mut completion_message = None;
        let mut should_reset_iterations = false;

        for content_block in &response.content {
            match content_block {
                ContentBlock::Text { text } => {
                    text_content.push_str(text);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    tool_call_count += 1;

                    // Execute tool - returns ToolExecutionResult
                    let tool_execution_result = tools::execute_tool(
                        name,
                        input.clone(),
                        agent_manager.clone(),
                        app_handle.clone(),
                        sleep_state.clone(),
                        pending_question.clone(),
                        &queue_status_fn,
                        iteration_ctx.clone(),
                    )
                    .await;

                    // Get the result value for logging/events
                    let tool_result = tool_execution_result.to_value();

                    // Compress the tool result before storing in history
                    let compressed_result = self.output_compressor.compress(&tool_result);

                    // Check if this is a completion signal
                    match &tool_execution_result {
                        ToolExecutionResult::Complete(msg) => {
                            should_complete = true;
                            completion_message = Some(msg.clone());
                        }
                        ToolExecutionResult::SleepComplete(_) => {
                            should_reset_iterations = true;
                        }
                        ToolExecutionResult::Continue(_) => {}
                    }

                    // Create tool call record (with compressed result for history)
                    let tool_call = ToolCall {
                        id: id.clone(),
                        tool_name: name.clone(),
                        input: input.clone(),
                        output: Some(compressed_result.clone()),
                    };
                    tool_calls.push(tool_call);

                    // Emit tool call event (with full result for UI)
                    let timestamp = chrono::Utc::now().timestamp_millis();
                    let _ = app_handle.emit(
                        "meta-agent:tool-call",
                        MetaAgentToolCallEvent {
                            tool_name: name.clone(),
                            input: input.clone(),
                            output: tool_result, // Full result for UI display
                            timestamp,
                        },
                    );

                    // Add compressed tool result to send back to Claude
                    tool_results.push(serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": id,
                        "content": serde_json::to_string(&compressed_result).unwrap_or_else(|_| "{}".to_string()),
                    }));
                }
            }
        }

        ResponseProcessingResult {
            text_content,
            tool_calls,
            tool_results,
            tool_call_count,
            should_complete,
            completion_message,
            should_reset_iterations,
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
    /// until CompleteTask is called or a response without tool calls is produced.
    /// Sleep tool resets the iteration counter, allowing the meta-agent to work
    /// indefinitely by periodically sleeping.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_loop<F, G>(
        &self,
        ai_client: &AIClient,
        system_prompt: &str,
        conversation_history: &mut Vec<Message>,
        tools: Vec<Tool>,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: &AppHandle,
        sleep_state: Arc<Mutex<SleepState>>,
        pending_question: Arc<Mutex<Option<PendingQuestion>>>,
        queue_status_fn: F,
        context_info_fn: G,
    ) -> AppResult<ToolLoopResult>
    where
        F: Fn() -> QueueStatus,
        G: Fn() -> Option<ContextInfo>,
    {
        let mut iteration = 0;
        let mut tool_call_count = 0;
        let mut final_response: Option<ChatResponse> = None;
        let max_iterations = self.config.max_iterations;
        let mut total_usage = Usage {
            input_tokens: 0,
            output_tokens: 0,
        };

        while iteration < max_iterations && tool_call_count < self.config.max_tool_calls {
            iteration += 1;

            // Create iteration context to pass to tools (with context info if available)
            let iteration_ctx = if let Some(ctx_info) = context_info_fn() {
                IterationContext::with_context_info(iteration, max_iterations, ctx_info)
            } else {
                IterationContext::new(iteration, max_iterations)
            };

            // Call AI API with system prompt and tools
            let response = ai_client
                .send_message_with_system_and_tools(
                    system_prompt,
                    conversation_history.clone(),
                    tools.clone(),
                )
                .await
                .map_err(|e| AppError::Api(ApiError::Network(format!("AI API error: {}", e))))?;

            // Accumulate usage
            total_usage.input_tokens += response.usage.input_tokens;
            total_usage.output_tokens += response.usage.output_tokens;

            // Process the response
            let result = self
                .process_response(
                    &response,
                    agent_manager.clone(),
                    app_handle,
                    sleep_state.clone(),
                    pending_question.clone(),
                    &queue_status_fn,
                    iteration_ctx,
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

            // Check if CompleteTask was called - exit the loop with completion message
            if result.should_complete {
                let completion_text = result
                    .completion_message
                    .unwrap_or_else(|| result.text_content.clone());
                final_response = Some(Self::build_final_response(
                    completion_text,
                    result.tool_calls,
                    &response.usage,
                ));
                break;
            }

            // Reset iteration counter if Sleep was called
            if result.should_reset_iterations {
                eprintln!(
                    "[MetaAgent] Sleep completed - resetting iteration counter from {} to 0",
                    iteration
                );
                iteration = 0;
            }

            // If there were tool calls, send tool results back
            if !result.tool_results.is_empty() {
                conversation_history.push(Message {
                    role: "user".to_string(),
                    content: Self::build_tool_results_content(&result.tool_results),
                });
                continue; // Continue the loop to get next response
            }

            // No tool calls - we have our final response
            final_response = Some(Self::build_final_response(
                result.text_content,
                result.tool_calls,
                &response.usage,
            ));
            break;
        }

        final_response
            .map(|response| ToolLoopResult {
                response,
                total_usage,
            })
            .ok_or_else(|| {
                AppError::Internal(
                    "Meta-agent failed to produce a response within iteration limits".to_string(),
                )
            })
    }

    /// Run the tool loop with rich messages (for image support)
    ///
    /// This is used on the first iteration when processing a message with an image.
    /// Subsequent iterations use the regular loop.
    /// Sleep tool resets the iteration counter.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_loop_with_initial_rich_messages<F, G>(
        &self,
        ai_client: &AIClient,
        system_prompt: &str,
        initial_rich_messages: Vec<RichMessage>,
        conversation_history: &mut Vec<Message>,
        tools: Vec<Tool>,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: &AppHandle,
        sleep_state: Arc<Mutex<SleepState>>,
        pending_question: Arc<Mutex<Option<PendingQuestion>>>,
        queue_status_fn: F,
        context_info_fn: G,
    ) -> AppResult<ToolLoopResult>
    where
        F: Fn() -> QueueStatus,
        G: Fn() -> Option<ContextInfo>,
    {
        let mut iteration = 0;
        let mut tool_call_count = 0;
        let mut final_response: Option<ChatResponse> = None;
        let max_iterations = self.config.max_iterations;
        let mut total_usage = Usage {
            input_tokens: 0,
            output_tokens: 0,
        };

        while iteration < max_iterations && tool_call_count < self.config.max_tool_calls {
            iteration += 1;

            // Create iteration context to pass to tools (with context info if available)
            let iteration_ctx = if let Some(ctx_info) = context_info_fn() {
                IterationContext::with_context_info(iteration, max_iterations, ctx_info)
            } else {
                IterationContext::new(iteration, max_iterations)
            };

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

            // Accumulate usage
            total_usage.input_tokens += response.usage.input_tokens;
            total_usage.output_tokens += response.usage.output_tokens;

            // Process the response
            let result = self
                .process_response(
                    &response,
                    agent_manager.clone(),
                    app_handle,
                    sleep_state.clone(),
                    pending_question.clone(),
                    &queue_status_fn,
                    iteration_ctx,
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

            // Check if CompleteTask was called - exit the loop with completion message
            if result.should_complete {
                let completion_text = result
                    .completion_message
                    .unwrap_or_else(|| result.text_content.clone());
                final_response = Some(Self::build_final_response(
                    completion_text,
                    result.tool_calls,
                    &response.usage,
                ));
                break;
            }

            // Reset iteration counter if Sleep was called
            if result.should_reset_iterations {
                eprintln!(
                    "[MetaAgent] Sleep completed - resetting iteration counter from {} to 0",
                    iteration
                );
                iteration = 0;
            }

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

        final_response
            .map(|response| ToolLoopResult {
                response,
                total_usage,
            })
            .ok_or_else(|| {
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
        assert_eq!(config.max_iterations, 40);
        assert_eq!(config.max_tool_calls, 200);
        assert_eq!(config.max_tool_output_chars, 10_000);
    }

    #[test]
    fn test_build_tool_results_content() {
        let results = vec![serde_json::json!({"type": "tool_result", "tool_use_id": "123"})];
        let content = ToolLoopEngine::build_tool_results_content(&results);
        assert!(content.contains("tool_result"));
        assert!(content.contains("123"));
    }

    #[test]
    fn test_output_compressor_initialized() {
        let engine = ToolLoopEngine::new();
        // Just verify it was created without panicking
        assert_eq!(engine.config.max_tool_output_chars, 10_000);
    }
}
