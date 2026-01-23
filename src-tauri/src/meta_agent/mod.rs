// Meta-agent module

mod action_logger;
pub mod helpers;
mod result_queue;
mod system_prompt;
pub mod tools;

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::ai_client::types::ImageSource;
use crate::ai_client::{
    AIClient, ContentBlock, Message, RichContentBlock, RichMessage, RichMessageContent,
};
use crate::tool_registry::ToolRegistry;
use crate::types::{
    AgentResultStatus, ChatMessage, ChatResponse, ChatUsage, ImageAttachment,
    MetaAgentThinkingEvent, MetaAgentToolCallEvent, QueueStatus, QueuedAgentResult, ToolCall,
};
use system_prompt::META_AGENT_SYSTEM_PROMPT;

use result_queue::ResultQueue;

const MAX_TOOL_CALLS_PER_MESSAGE: usize = 20;
const MAX_ITERATIONS: usize = 10;

pub struct MetaAgent {
    conversation_history: Vec<Message>,
    tool_registry: ToolRegistry,
    ai_client: AIClient,
    result_queue: ResultQueue,
}

impl MetaAgent {
    pub fn new_with_client(ai_client: AIClient) -> Self {
        Self {
            conversation_history: Vec::new(),
            tool_registry: ToolRegistry::new(),
            ai_client,
            result_queue: ResultQueue::new(),
        }
    }

    pub fn new() -> Result<Self, String> {
        let ai_client =
            AIClient::from_env().map_err(|e| format!("Failed to initialize AI client: {}", e))?;
        Ok(Self::new_with_client(ai_client))
    }

    // =========================================================================
    // Result Queue Management
    // =========================================================================

    /// Add a result to the queue when an agent completes
    pub fn queue_agent_result(&mut self, result: QueuedAgentResult) {
        self.result_queue.push(result);
    }

    /// Get the next pending result from the queue
    pub fn pop_next_result(&mut self) -> Option<QueuedAgentResult> {
        self.result_queue.pop()
    }

    /// Get the current queue status
    pub fn get_queue_status(&self) -> QueueStatus {
        self.result_queue.status()
    }

    /// Check if there are pending results in the queue
    pub fn has_pending_results(&self) -> bool {
        !self.result_queue.is_empty()
    }

    /// Process the next result in the queue
    pub async fn process_next_queued_result(
        &mut self,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Result<Option<ChatResponse>, String> {
        if let Some(mut result) = self.result_queue.pop() {
            result.status = AgentResultStatus::Processing;

            // Format the result as a message to process
            let message = format!(
                "Agent in {} has completed. Here are the results:\n\n{}",
                result.working_dir, result.output
            );

            // Process it through the normal message flow
            let response = self
                .process_user_message(message, agent_manager, app_handle.clone())
                .await?;

            // Emit queue updated event
            self.result_queue.emit_updated(&app_handle);

            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    // =========================================================================
    // Message Processing
    // =========================================================================

    pub async fn process_user_message(
        &mut self,
        user_message: String,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Result<ChatResponse, String> {
        // Emit thinking event
        app_handle
            .emit(
                "meta-agent:thinking",
                MetaAgentThinkingEvent { is_thinking: true },
            )
            .map_err(|e| format!("Failed to emit thinking event: {}", e))?;

        // Add user message to history
        self.conversation_history.push(Message {
            role: "user".to_string(),
            content: user_message.clone(),
        });

        let mut iteration = 0;
        let mut tool_call_count = 0;
        let mut final_response: Option<ChatResponse> = None;

        // Agent loop: keep calling API until we get a non-tool-use response
        while iteration < MAX_ITERATIONS && tool_call_count < MAX_TOOL_CALLS_PER_MESSAGE {
            iteration += 1;

            // Call AI API with system prompt and tools
            let response = self
                .ai_client
                .send_message_with_system_and_tools(
                    META_AGENT_SYSTEM_PROMPT,
                    self.conversation_history.clone(),
                    self.tool_registry.get_all_tools().to_vec(),
                )
                .await
                .map_err(|e| format!("AI API error: {}", e))?;

            // Process response content
            let mut text_content = String::new();
            let mut tool_calls = Vec::new();
            let mut tool_results = Vec::new();

            for content_block in &response.content {
                match content_block {
                    ContentBlock::Text { text } => {
                        text_content.push_str(text);
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        tool_call_count += 1;

                        // Execute tool
                        let queue_status_fn = || self.get_queue_status();
                        let tool_result = tools::execute_tool(
                            name,
                            input.clone(),
                            agent_manager.clone(),
                            app_handle.clone(),
                            queue_status_fn,
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
                        app_handle
                            .emit(
                                "meta-agent:tool-call",
                                MetaAgentToolCallEvent {
                                    tool_name: name.clone(),
                                    input: input.clone(),
                                    output: tool_result.clone(),
                                    timestamp,
                                },
                            )
                            .ok();

                        // Add tool result to send back to Claude
                        tool_results.push(serde_json::json!({
                            "type": "tool_result",
                            "tool_use_id": id,
                            "content": serde_json::to_string(&tool_result).unwrap_or_else(|_| "{}".to_string()),
                        }));
                    }
                }
            }

            // Add assistant message to history
            let assistant_content = if !tool_results.is_empty() {
                serde_json::to_string(&response.content).unwrap_or_else(|_| text_content.clone())
            } else {
                text_content.clone()
            };

            self.conversation_history.push(Message {
                role: "assistant".to_string(),
                content: assistant_content,
            });

            // If there were tool calls, send tool results back
            if !tool_results.is_empty() {
                self.conversation_history.push(Message {
                    role: "user".to_string(),
                    content: serde_json::to_string(&tool_results)
                        .unwrap_or_else(|_| "[]".to_string()),
                });
                continue; // Continue the loop to get final response
            }

            // No tool calls - we have our final response
            let timestamp = chrono::Utc::now().timestamp_millis();
            final_response = Some(ChatResponse {
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
                    input_tokens: response.usage.input_tokens,
                    output_tokens: response.usage.output_tokens,
                },
            });
            break;
        }

        // Emit thinking stopped
        app_handle
            .emit(
                "meta-agent:thinking",
                MetaAgentThinkingEvent { is_thinking: false },
            )
            .ok();

        final_response.ok_or_else(|| {
            "Meta-agent failed to produce a response within iteration limits".to_string()
        })
    }

    /// Process a user message with an optional image attachment
    pub async fn process_user_message_with_image(
        &mut self,
        user_message: String,
        image: Option<ImageAttachment>,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Result<ChatResponse, String> {
        // If no image, use the regular method
        if image.is_none() {
            return self
                .process_user_message(user_message, agent_manager, app_handle)
                .await;
        }

        let image = image.unwrap();

        // Emit thinking event
        app_handle
            .emit(
                "meta-agent:thinking",
                MetaAgentThinkingEvent { is_thinking: true },
            )
            .map_err(|e| format!("Failed to emit thinking event: {}", e))?;

        // Build rich content blocks with image first (recommended by Claude)
        let mut content_blocks = vec![RichContentBlock::Image {
            source: ImageSource {
                source_type: "base64".to_string(),
                media_type: image.mime_type.clone(),
                data: image.base64_data.clone(),
            },
        }];

        // Add text if not empty
        if !user_message.is_empty() {
            content_blocks.push(RichContentBlock::Text {
                text: user_message.clone(),
            });
        }

        // Build conversation history as rich messages
        let mut rich_messages: Vec<RichMessage> = self
            .conversation_history
            .iter()
            .map(|msg| RichMessage {
                role: msg.role.clone(),
                content: RichMessageContent::Text(msg.content.clone()),
            })
            .collect();

        // Add the new user message with image
        rich_messages.push(RichMessage {
            role: "user".to_string(),
            content: RichMessageContent::Blocks(content_blocks),
        });

        // Also add to simple conversation history for future reference
        // (store just the text, image is handled in this request)
        self.conversation_history.push(Message {
            role: "user".to_string(),
            content: if user_message.is_empty() {
                "[Image attached]".to_string()
            } else {
                format!("[Image attached] {}", user_message)
            },
        });

        let mut iteration = 0;
        let mut tool_call_count = 0;
        let mut final_response: Option<ChatResponse> = None;

        // Agent loop: keep calling API until we get a non-tool-use response
        while iteration < MAX_ITERATIONS && tool_call_count < MAX_TOOL_CALLS_PER_MESSAGE {
            iteration += 1;

            // For first iteration, use rich messages with image
            // For subsequent iterations, use simple messages (image already processed)
            let response = if iteration == 1 {
                // Prepend system prompt as first message
                let mut messages_with_system = vec![RichMessage {
                    role: "user".to_string(),
                    content: RichMessageContent::Text(format!(
                        "System instructions (follow these for all interactions):\n\n{}",
                        system_prompt::META_AGENT_SYSTEM_PROMPT
                    )),
                }];

                // Add assistant acknowledgment
                messages_with_system.push(RichMessage {
                    role: "assistant".to_string(),
                    content: RichMessageContent::Text(
                        "I understand and will follow these instructions.".to_string(),
                    ),
                });

                // Add the rest of the conversation
                messages_with_system.extend(rich_messages.clone());

                self.ai_client
                    .send_rich_message_with_tools(
                        messages_with_system,
                        self.tool_registry.get_all_tools().to_vec(),
                    )
                    .await
                    .map_err(|e| format!("AI API error: {}", e))?
            } else {
                self.ai_client
                    .send_message_with_system_and_tools(
                        system_prompt::META_AGENT_SYSTEM_PROMPT,
                        self.conversation_history.clone(),
                        self.tool_registry.get_all_tools().to_vec(),
                    )
                    .await
                    .map_err(|e| format!("AI API error: {}", e))?
            };

            // Process response content
            let mut text_content = String::new();
            let mut tool_calls = Vec::new();
            let mut tool_results = Vec::new();

            for content_block in &response.content {
                match content_block {
                    ContentBlock::Text { text } => {
                        text_content.push_str(text);
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        tool_call_count += 1;

                        // Execute tool
                        let queue_status_fn = || self.get_queue_status();
                        let tool_result = tools::execute_tool(
                            name,
                            input.clone(),
                            agent_manager.clone(),
                            app_handle.clone(),
                            queue_status_fn,
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
                        app_handle
                            .emit(
                                "meta-agent:tool-call",
                                MetaAgentToolCallEvent {
                                    tool_name: name.clone(),
                                    input: input.clone(),
                                    output: tool_result.clone(),
                                    timestamp,
                                },
                            )
                            .ok();

                        // Add tool result to send back to Claude
                        tool_results.push(serde_json::json!({
                            "type": "tool_result",
                            "tool_use_id": id,
                            "content": serde_json::to_string(&tool_result).unwrap_or_else(|_| "{}".to_string()),
                        }));
                    }
                }
            }

            // Add assistant message to history
            let assistant_content = if !tool_results.is_empty() {
                serde_json::to_string(&response.content).unwrap_or_else(|_| text_content.clone())
            } else {
                text_content.clone()
            };

            self.conversation_history.push(Message {
                role: "assistant".to_string(),
                content: assistant_content,
            });

            // If there were tool calls, send tool results back
            if !tool_results.is_empty() {
                self.conversation_history.push(Message {
                    role: "user".to_string(),
                    content: serde_json::to_string(&tool_results)
                        .unwrap_or_else(|_| "[]".to_string()),
                });
                continue; // Continue the loop to get final response
            }

            // No tool calls - we have our final response
            let timestamp = chrono::Utc::now().timestamp_millis();
            final_response = Some(ChatResponse {
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
                    input_tokens: response.usage.input_tokens,
                    output_tokens: response.usage.output_tokens,
                },
            });
            break;
        }

        // Emit thinking stopped
        app_handle
            .emit(
                "meta-agent:thinking",
                MetaAgentThinkingEvent { is_thinking: false },
            )
            .ok();

        final_response.ok_or_else(|| {
            "Meta-agent failed to produce a response within iteration limits".to_string()
        })
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    pub fn get_conversation_history(&self) -> &[Message] {
        &self.conversation_history
    }

    pub fn clear_conversation_history(&mut self) {
        self.conversation_history.clear();
    }

    pub fn get_ai_client(&self) -> &AIClient {
        &self.ai_client
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage> {
        self.conversation_history
            .iter()
            .enumerate()
            .map(|(i, msg)| ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                image: None,
                tool_calls: None,
                timestamp: chrono::Utc::now().timestamp_millis()
                    - ((self.conversation_history.len() - i) as i64 * 1000),
            })
            .collect()
    }
}
