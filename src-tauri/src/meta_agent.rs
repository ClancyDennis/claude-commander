use crate::agent_manager::AgentManager;
use crate::ai_client::{AIClient, ContentBlock, Message};
use crate::tool_registry::ToolRegistry;
use crate::types::{ChatMessage, ChatResponse, ChatUsage, MetaAgentThinkingEvent, MetaAgentToolCallEvent, ToolCall};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

const MAX_TOOL_CALLS_PER_MESSAGE: usize = 20;
const MAX_ITERATIONS: usize = 10;
const MAX_TOKENS: u32 = 4096;

pub struct MetaAgent {
    conversation_history: Vec<Message>,
    tool_registry: ToolRegistry,
    ai_client: AIClient,
}

impl MetaAgent {
    pub fn new_with_client(ai_client: AIClient) -> Self {
        Self {
            conversation_history: Vec::new(),
            tool_registry: ToolRegistry::new(),
            ai_client,
        }
    }

    pub fn new() -> Result<Self, String> {
        let ai_client = AIClient::from_env()
            .map_err(|e| format!("Failed to initialize AI client: {}", e))?;
        Ok(Self::new_with_client(ai_client))
    }

    pub async fn process_user_message(
        &mut self,
        user_message: String,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Result<ChatResponse, String> {
        // Emit thinking event
        app_handle
            .emit("meta-agent:thinking", MetaAgentThinkingEvent { is_thinking: true })
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

            // Call AI API with tools
            let response = self
                .ai_client
                .send_message_with_tools(
                    self.conversation_history.clone(),
                    self.tool_registry.get_all_tools().to_vec(),
                    MAX_TOKENS,
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
                        let tool_result = self
                            .execute_tool(
                                name,
                                input.clone(),
                                agent_manager.clone(),
                                app_handle.clone(),
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
                        tool_results.push(json!({
                            "type": "tool_result",
                            "tool_use_id": id,
                            "content": serde_json::to_string(&tool_result).unwrap_or_else(|_| "{}".to_string()),
                        }));
                    }
                }
            }

            // Add assistant message to history
            let assistant_content = if !tool_results.is_empty() {
                // If we had tool calls, construct a message with both text and tool uses
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
                    content: serde_json::to_string(&tool_results).unwrap_or_else(|_| "[]".to_string()),
                });
                continue; // Continue the loop to get final response
            }

            // No tool calls - we have our final response
            let timestamp = chrono::Utc::now().timestamp_millis();
            final_response = Some(ChatResponse {
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: text_content,
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
            .emit("meta-agent:thinking", MetaAgentThinkingEvent { is_thinking: false })
            .ok();

        final_response.ok_or_else(|| {
            "Meta-agent failed to produce a response within iteration limits".to_string()
        })
    }

    async fn execute_tool(
        &self,
        tool_name: &str,
        input: Value,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Value {
        match tool_name {
            "CreateWorkerAgent" => {
                self.create_worker_agent(input, agent_manager, app_handle)
                    .await
            }
            "SendPromptToWorker" => {
                self.send_prompt_to_worker(input, agent_manager, app_handle).await
            }
            "StopWorkerAgent" => {
                self.stop_worker_agent(input, agent_manager).await
            }
            "ListWorkerAgents" => {
                self.list_worker_agents(agent_manager).await
            }
            "GetAgentOutput" => {
                self.get_agent_output(input, app_handle).await
            }
            "NavigateToAgent" => {
                self.navigate_to_agent(input, app_handle).await
            }
            "ToggleToolPanel" => {
                self.toggle_tool_panel(input, app_handle).await
            }
            "ShowNotification" => {
                self.show_notification(input, app_handle).await
            }
            _ => json!({
                "success": false,
                "error": format!("Unknown tool: {}", tool_name)
            }),
        }
    }

    async fn create_worker_agent(
        &self,
        input: Value,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Value {
        let working_dir = input["working_dir"].as_str().unwrap_or("");
        if working_dir.is_empty() {
            return json!({
                "success": false,
                "error": "working_dir is required"
            });
        }

        let github_url = input["github_url"].as_str().map(|s| s.to_string());

        let manager = agent_manager.lock().await;
        match manager.create_agent(working_dir.to_string(), github_url, app_handle.clone()).await {
            Ok(agent_id) => {
                drop(manager);

                // Send initial prompt if provided
                if let Some(initial_prompt) = input["initial_prompt"].as_str() {
                    let manager = agent_manager.lock().await;
                    if let Err(e) = manager.send_prompt(&agent_id, initial_prompt, Some(app_handle.clone())).await {
                        return json!({
                            "success": true,
                            "agent_id": agent_id,
                            "warning": format!("Agent created but initial prompt failed: {}", e)
                        });
                    }
                    drop(manager);
                }

                // Navigate to agent if requested
                if input["navigate"].as_bool().unwrap_or(false) {
                    app_handle
                        .emit("agent:navigate", json!({ "agent_id": agent_id }))
                        .ok();
                }

                json!({
                    "success": true,
                    "agent_id": agent_id,
                    "status": "created and running"
                })
            }
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to create agent: {}", e)
            }),
        }
    }

    async fn send_prompt_to_worker(
        &self,
        input: Value,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Value {
        let agent_id = input["agent_id"].as_str().unwrap_or("");
        let prompt = input["prompt"].as_str().unwrap_or("");

        if agent_id.is_empty() || prompt.is_empty() {
            return json!({
                "success": false,
                "error": "agent_id and prompt are required"
            });
        }

        let manager = agent_manager.lock().await;
        match manager.send_prompt(agent_id, prompt, Some(app_handle)).await {
            Ok(_) => json!({
                "success": true,
                "message": "Prompt sent successfully"
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to send prompt: {}", e)
            }),
        }
    }

    async fn stop_worker_agent(
        &self,
        input: Value,
        agent_manager: Arc<Mutex<AgentManager>>,
    ) -> Value {
        let agent_id = input["agent_id"].as_str().unwrap_or("");

        if agent_id.is_empty() {
            return json!({
                "success": false,
                "error": "agent_id is required"
            });
        }

        let manager = agent_manager.lock().await;
        match manager.stop_agent(agent_id).await {
            Ok(_) => json!({
                "success": true,
                "message": "Agent stopped successfully"
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to stop agent: {}", e)
            }),
        }
    }

    async fn list_worker_agents(&self, agent_manager: Arc<Mutex<AgentManager>>) -> Value {
        let manager = agent_manager.lock().await;
        let agents = manager.list_agents().await;
        json!({
            "success": true,
            "agents": agents
        })
    }

    async fn get_agent_output(&self, input: Value, _app_handle: AppHandle) -> Value {
        let agent_id = input["agent_id"].as_str().unwrap_or("");
        let last_n = input["last_n"].as_u64().unwrap_or(10) as usize;

        if agent_id.is_empty() {
            return json!({
                "success": false,
                "error": "agent_id is required"
            });
        }

        // Emit event to request output (frontend will respond via a different mechanism)
        // For MVP, we return a placeholder
        json!({
            "success": true,
            "message": format!("Requested last {} outputs for agent {}", last_n, agent_id),
            "note": "Output retrieval requires frontend integration"
        })
    }

    async fn navigate_to_agent(&self, input: Value, app_handle: AppHandle) -> Value {
        let agent_id = input["agent_id"].as_str().unwrap_or("");

        if agent_id.is_empty() {
            return json!({
                "success": false,
                "error": "agent_id is required"
            });
        }

        match app_handle.emit("agent:navigate", json!({ "agent_id": agent_id })) {
            Ok(_) => json!({
                "success": true,
                "message": "Navigation event emitted"
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to emit navigation event: {}", e)
            }),
        }
    }

    async fn toggle_tool_panel(&self, input: Value, app_handle: AppHandle) -> Value {
        let show = input["show"].as_bool().unwrap_or(true);

        match app_handle.emit("tool-panel:toggle", json!({ "show": show })) {
            Ok(_) => json!({
                "success": true,
                "message": if show { "Tool panel shown" } else { "Tool panel hidden" }
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to toggle tool panel: {}", e)
            }),
        }
    }

    async fn show_notification(&self, input: Value, app_handle: AppHandle) -> Value {
        let message = input["message"].as_str().unwrap_or("");
        let notification_type = input["type"].as_str().unwrap_or("info");

        if message.is_empty() {
            return json!({
                "success": false,
                "error": "message is required"
            });
        }

        match app_handle.emit(
            "notification:show",
            json!({
                "message": message,
                "type": notification_type
            }),
        ) {
            Ok(_) => json!({
                "success": true,
                "message": "Notification emitted"
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to show notification: {}", e)
            }),
        }
    }

    pub fn get_conversation_history(&self) -> &[Message] {
        &self.conversation_history
    }

    pub fn clear_conversation_history(&mut self) {
        self.conversation_history.clear();
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage> {
        self.conversation_history
            .iter()
            .enumerate()
            .map(|(i, msg)| ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                tool_calls: None,
                timestamp: chrono::Utc::now().timestamp_millis() - ((self.conversation_history.len() - i) as i64 * 1000),
            })
            .collect()
    }
}
