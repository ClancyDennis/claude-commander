use crate::agent_manager::AgentManager;
use crate::ai_client::{AIClient, ContentBlock, Message};
use crate::tool_registry::ToolRegistry;
use crate::types::{
    ChatMessage, ChatResponse, ChatUsage, CommanderAction, MetaAgentThinkingEvent,
    MetaAgentToolCallEvent, QueueStatus, QueuedAgentResult,
    AgentResultStatus, ResultQueueUpdatedEvent, ToolCall,
};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

const MAX_TOOL_CALLS_PER_MESSAGE: usize = 20;
const MAX_ITERATIONS: usize = 10;

pub struct MetaAgent {
    conversation_history: Vec<Message>,
    tool_registry: ToolRegistry,
    ai_client: AIClient,
    /// Queue of agent results waiting to be processed
    result_queue: VecDeque<QueuedAgentResult>,
}

impl MetaAgent {
    pub fn new_with_client(ai_client: AIClient) -> Self {
        Self {
            conversation_history: Vec::new(),
            tool_registry: ToolRegistry::new(),
            ai_client,
            result_queue: VecDeque::new(),
        }
    }

    pub fn new() -> Result<Self, String> {
        let ai_client = AIClient::from_env()
            .map_err(|e| format!("Failed to initialize AI client: {}", e))?;
        Ok(Self::new_with_client(ai_client))
    }

    // =========================================================================
    // Result Queue Management
    // =========================================================================

    /// Add a result to the queue when an agent completes
    pub fn queue_agent_result(&mut self, result: QueuedAgentResult) {
        self.result_queue.push_back(result);
    }

    /// Get the next pending result from the queue
    pub fn pop_next_result(&mut self) -> Option<QueuedAgentResult> {
        self.result_queue.pop_front()
    }

    /// Get the current queue status
    pub fn get_queue_status(&self) -> QueueStatus {
        QueueStatus {
            pending: self.result_queue.len(),
            items: self.result_queue.iter().map(|r| r.summary()).collect(),
        }
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
        if let Some(mut result) = self.result_queue.pop_front() {
            result.status = AgentResultStatus::Processing;

            // Format the result as a message to process
            let message = format!(
                "Agent in {} has completed. Here are the results:\n\n{}",
                result.working_dir, result.output
            );

            // Process it through the normal message flow
            let response = self.process_user_message(message, agent_manager, app_handle.clone()).await?;

            // Emit queue updated event
            self.emit_queue_updated(&app_handle);

            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    /// Emit event when queue is updated
    fn emit_queue_updated(&self, app_handle: &AppHandle) {
        let _ = app_handle.emit(
            "result-queue:updated",
            ResultQueueUpdatedEvent {
                queue_status: self.get_queue_status(),
            },
        );
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
        let result = match tool_name {
            "CreateWorkerAgent" => {
                self.create_worker_agent(input.clone(), agent_manager, app_handle.clone())
                    .await
            }
            "SendPromptToWorker" => {
                self.send_prompt_to_worker(input.clone(), agent_manager, app_handle.clone()).await
            }
            "StopWorkerAgent" => {
                self.stop_worker_agent(input.clone(), agent_manager).await
            }
            "ListWorkerAgents" => {
                self.list_worker_agents(agent_manager).await
            }
            "GetAgentOutput" => {
                self.get_agent_output(input.clone(), agent_manager, app_handle.clone()).await
            }
            "NavigateToAgent" => {
                self.navigate_to_agent(input.clone(), app_handle.clone()).await
            }
            "ToggleToolPanel" => {
                self.toggle_tool_panel(input.clone(), app_handle.clone()).await
            }
            "ShowNotification" => {
                self.show_notification(input.clone(), app_handle.clone()).await
            }
            "ListDirectory" => {
                self.list_directory(input.clone()).await
            }
            "ShipDataToAgent" => {
                self.ship_data_to_agent(input.clone(), agent_manager, app_handle.clone()).await
            }
            "CreateChainedAgent" => {
                self.create_chained_agent(input.clone(), agent_manager, app_handle.clone()).await
            }
            "QuickAction" => {
                self.quick_action(input.clone(), agent_manager, app_handle.clone()).await
            }
            _ => json!({
                "success": false,
                "error": format!("Unknown tool: {}", tool_name)
            }),
        };

        // Emit commander:action event for the action log sidebar
        self.emit_commander_action(tool_name, &input, &result, &app_handle);

        result
    }

    /// Emit a commander action event for the action log sidebar
    fn emit_commander_action(&self, tool_name: &str, input: &Value, result: &Value, app_handle: &AppHandle) {
        let description = self.format_action_description(tool_name, input, result);
        let agent_id = self.extract_agent_id(input);
        let success = result["success"].as_bool().unwrap_or(true);
        let timestamp = chrono::Utc::now().timestamp_millis();

        let action = CommanderAction {
            action_type: tool_name.to_string(),
            description,
            timestamp,
            agent_id,
            success,
        };

        let _ = app_handle.emit("commander:action", action);
    }

    /// Format a human-readable description of an action
    fn format_action_description(&self, tool_name: &str, input: &Value, result: &Value) -> String {
        match tool_name {
            "CreateWorkerAgent" => {
                let dir = input["working_dir"].as_str().unwrap_or("unknown");
                // Shorten path for display
                let short_dir = if dir.len() > 30 {
                    format!("...{}", &dir[dir.len()-27..])
                } else {
                    dir.to_string()
                };
                if let Some(agent_id) = result["agent_id"].as_str() {
                    format!("Created agent {} in {}", &agent_id[..8.min(agent_id.len())], short_dir)
                } else {
                    format!("Created agent in {}", short_dir)
                }
            }
            "SendPromptToWorker" => {
                let agent_id = input["agent_id"].as_str().unwrap_or("?");
                let short_id = &agent_id[..8.min(agent_id.len())];
                format!("Sent task to agent {}", short_id)
            }
            "StopWorkerAgent" => {
                let agent_id = input["agent_id"].as_str().unwrap_or("?");
                let short_id = &agent_id[..8.min(agent_id.len())];
                format!("Stopped agent {}", short_id)
            }
            "ListWorkerAgents" => {
                let count = result["agents"].as_array().map(|a| a.len()).unwrap_or(0);
                format!("Listed {} agents", count)
            }
            "GetAgentOutput" => {
                let agent_id = input["agent_id"].as_str().unwrap_or("?");
                let short_id = &agent_id[..8.min(agent_id.len())];
                format!("Retrieved output from agent {}", short_id)
            }
            "NavigateToAgent" => {
                let agent_id = input["agent_id"].as_str().unwrap_or("?");
                let short_id = &agent_id[..8.min(agent_id.len())];
                format!("Navigated to agent {}", short_id)
            }
            "ToggleToolPanel" => {
                let show = input["show"].as_bool().unwrap_or(true);
                if show { "Showed tool panel".to_string() } else { "Hid tool panel".to_string() }
            }
            "ShowNotification" => {
                let msg = input["message"].as_str().unwrap_or("");
                let short_msg = if msg.len() > 30 {
                    format!("{}...", &msg[..27])
                } else {
                    msg.to_string()
                };
                format!("Notification: {}", short_msg)
            }
            "ListDirectory" => {
                let path = input["path"].as_str().unwrap_or("?");
                format!("Listed directory {}", path)
            }
            "ShipDataToAgent" => {
                let source = input["source_agent_id"].as_str().unwrap_or("?");
                let target = input["target_agent_id"].as_str().unwrap_or("?");
                format!("Shipped data {} → {}",
                    &source[..8.min(source.len())],
                    &target[..8.min(target.len())])
            }
            "CreateChainedAgent" => {
                let source = input["source_agent_id"].as_str().unwrap_or("?");
                format!("Created chained agent from {}", &source[..8.min(source.len())])
            }
            "QuickAction" => {
                let action = input["action"].as_str().unwrap_or("unknown");
                format!("Quick action: {}", action)
            }
            _ => format!("Executed {}", tool_name),
        }
    }

    /// Extract agent_id from tool input if present
    fn extract_agent_id(&self, input: &Value) -> Option<String> {
        input["agent_id"].as_str().map(|s| s.to_string())
            .or_else(|| input["source_agent_id"].as_str().map(|s| s.to_string()))
            .or_else(|| input["target_agent_id"].as_str().map(|s| s.to_string()))
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
                "error": "Validation failed: working_dir is required. Use the ListDirectory tool to explore the filesystem and find a valid directory, or ask the user for a working directory path."
            });
        }

        // Check if the directory exists
        if !std::path::Path::new(working_dir).exists() {
            return json!({
                "success": false,
                "error": format!("Validation failed: Directory '{}' does not exist. Use the ListDirectory tool to explore available directories (e.g., ListDirectory with path '~' or '/home'), or ask the user for a valid path.", working_dir)
            });
        }

        let github_url = input["github_url"].as_str().map(|s| s.to_string());

        let manager = agent_manager.lock().await;
        match manager.create_agent(working_dir.to_string(), github_url, None, crate::types::AgentSource::Meta, Arc::new(app_handle.clone())).await {
            Ok(agent_id) => {
                drop(manager);

                // Send initial prompt if provided
                if let Some(initial_prompt) = input["initial_prompt"].as_str() {
                    let manager = agent_manager.lock().await;
                    if let Err(e) = manager.send_prompt(&agent_id, initial_prompt, Some(Arc::new(app_handle.clone()))).await {
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
        match manager.send_prompt(agent_id, prompt, Some(Arc::new(app_handle))).await {
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

    async fn get_agent_output(&self, input: Value, agent_manager: Arc<Mutex<AgentManager>>, _app_handle: AppHandle) -> Value {
        let agent_id = input["agent_id"].as_str().unwrap_or("");
        let last_n = input["last_n"].as_u64().unwrap_or(10) as usize;

        if agent_id.is_empty() {
            return json!({
                "success": false,
                "error": "agent_id is required"
            });
        }

        let manager = agent_manager.lock().await;
        match manager.get_agent_outputs(agent_id, last_n).await {
            Ok(outputs) => {
                // Format outputs as readable text
                let mut formatted_output = String::new();

                for output in outputs.iter() {
                    match output.output_type.as_str() {
                        "text" => {
                            formatted_output.push_str(&format!("Assistant: {}\n\n", output.content));
                        }
                        "tool_use" => {
                            formatted_output.push_str(&format!("🔧 Using tool: {}\n",
                                output.parsed_json
                                    .as_ref()
                                    .and_then(|j| j.get("name"))
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("unknown")
                            ));
                        }
                        "tool_result" => {
                            formatted_output.push_str(&format!("Tool result: {}\n\n", output.content));
                        }
                        "result" => {
                            formatted_output.push_str("\n--- Final Results ---\n");
                            if let Some(parsed) = &output.parsed_json {
                                if let Some(cost) = parsed.get("total_cost_usd").and_then(|v| v.as_f64()) {
                                    formatted_output.push_str(&format!("Cost: ${:.4}\n", cost));
                                }
                                if let Some(usage) = parsed.get("usage") {
                                    if let Some(input_tokens) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                                        if let Some(output_tokens) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                                            formatted_output.push_str(&format!("\nTokens: {} input, {} output\n", input_tokens, output_tokens));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Get agent info for working directory
                let agent_info = manager.list_agents().await.into_iter()
                    .find(|a| a.id == agent_id);
                let agent_name = agent_info.map(|a| a.working_dir).unwrap_or_else(|| agent_id.to_string());

                json!({
                    "success": true,
                    "agent_id": agent_id,
                    "output_count": outputs.len(),
                    "outputs": formatted_output,
                    "summary": format!("Retrieved {} outputs from agent in {}", outputs.len(), agent_name)
                })
            }
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to get agent output: {}", e)
            })
        }
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
                tool_calls: None,
                timestamp: chrono::Utc::now().timestamp_millis() - ((self.conversation_history.len() - i) as i64 * 1000),
            })
            .collect()
    }

    async fn list_directory(&self, input: Value) -> Value {
        let path = input["path"].as_str().unwrap_or("");
        if path.is_empty() {
            return json!({
                "success": false,
                "error": "path is required"
            });
        }

        // Expand ~ to home directory
        let expanded_path = if path.starts_with("~") {
            if let Ok(home) = std::env::var("HOME") {
                path.replacen("~", &home, 1)
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        };

        // Check if path exists
        let path_obj = std::path::Path::new(&expanded_path);
        if !path_obj.exists() {
            return json!({
                "success": false,
                "error": format!("Path '{}' does not exist", expanded_path)
            });
        }

        if !path_obj.is_dir() {
            return json!({
                "success": false,
                "error": format!("Path '{}' is not a directory", expanded_path)
            });
        }

        // Read directory contents
        match std::fs::read_dir(&expanded_path) {
            Ok(entries) => {
                let mut items = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        let file_type = if entry.path().is_dir() {
                            "directory"
                        } else {
                            "file"
                        };
                        items.push(json!({
                            "name": file_name,
                            "type": file_type,
                            "path": entry.path().to_string_lossy().to_string()
                        }));
                    }
                }

                // Sort: directories first, then alphabetically
                items.sort_by(|a, b| {
                    let a_type = a["type"].as_str().unwrap_or("");
                    let b_type = b["type"].as_str().unwrap_or("");
                    let a_name = a["name"].as_str().unwrap_or("");
                    let b_name = b["name"].as_str().unwrap_or("");

                    match (a_type, b_type) {
                        ("directory", "file") => std::cmp::Ordering::Less,
                        ("file", "directory") => std::cmp::Ordering::Greater,
                        _ => a_name.cmp(b_name),
                    }
                });

                json!({
                    "success": true,
                    "path": expanded_path,
                    "items": items,
                    "count": items.len()
                })
            }
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to read directory: {}", e)
            }),
        }
    }

    // =========================================================================
    // New Tools: Data Shipping Between Agents
    // =========================================================================

    /// Ship data from one agent's output to another agent as context
    async fn ship_data_to_agent(
        &self,
        input: Value,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Value {
        let source_id = input["source_agent_id"].as_str().unwrap_or("");
        let target_id = input["target_agent_id"].as_str().unwrap_or("");
        let prompt = input["prompt_with_context"].as_str().unwrap_or("");
        let selector = input["data_selector"].as_str().unwrap_or("last_output");

        if source_id.is_empty() || target_id.is_empty() || prompt.is_empty() {
            return json!({
                "success": false,
                "error": "source_agent_id, target_agent_id, and prompt_with_context are required"
            });
        }

        // Get source agent's data
        let manager = agent_manager.lock().await;

        // Determine how much output to get based on selector
        let last_n = match selector {
            "last_output" => 1,
            "all_outputs" => 0,  // 0 means all
            "final_result" => 1,
            _ => 1,
        };

        let source_data = match manager.get_agent_outputs(source_id, last_n).await {
            Ok(outputs) => {
                let mut data = String::new();
                for output in outputs.iter() {
                    match output.output_type.as_str() {
                        "text" => data.push_str(&format!("{}\n", output.content)),
                        "result" if selector == "final_result" => {
                            data.push_str(&format!("{}\n", output.content));
                        }
                        _ => {
                            if selector == "all_outputs" {
                                data.push_str(&format!("{}\n", output.content));
                            }
                        }
                    }
                }
                data
            }
            Err(e) => {
                return json!({
                    "success": false,
                    "error": format!("Failed to get source agent output: {}", e)
                });
            }
        };

        if source_data.is_empty() {
            return json!({
                "success": false,
                "error": "Source agent has no output to ship"
            });
        }

        // Construct prompt with context
        let full_prompt = format!(
            "Context from previous agent work:\n---\n{}\n---\n\n{}",
            source_data.trim(),
            prompt
        );

        // Send to target agent
        match manager.send_prompt(target_id, &full_prompt, Some(Arc::new(app_handle))).await {
            Ok(_) => json!({
                "success": true,
                "message": format!("Shipped data from {} to {}", &source_id[..8.min(source_id.len())], &target_id[..8.min(target_id.len())]),
                "data_length": source_data.len()
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to send to target agent: {}", e)
            }),
        }
    }

    /// Create a new agent that receives context from an existing agent's output
    async fn create_chained_agent(
        &self,
        input: Value,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Value {
        let source_id = input["source_agent_id"].as_str().unwrap_or("");
        let working_dir = input["working_dir"].as_str().unwrap_or("");
        let prompt = input["prompt"].as_str().unwrap_or("");

        if source_id.is_empty() || working_dir.is_empty() || prompt.is_empty() {
            return json!({
                "success": false,
                "error": "source_agent_id, working_dir, and prompt are required"
            });
        }

        // Check if working directory exists
        if !std::path::Path::new(working_dir).exists() {
            return json!({
                "success": false,
                "error": format!("Working directory '{}' does not exist", working_dir)
            });
        }

        // Get source agent's output
        let manager = agent_manager.lock().await;
        let source_data = match manager.get_agent_outputs(source_id, 0).await {
            Ok(outputs) => {
                let mut data = String::new();
                for output in outputs.iter() {
                    if output.output_type == "text" || output.output_type == "result" {
                        data.push_str(&format!("{}\n", output.content));
                    }
                }
                data
            }
            Err(e) => {
                return json!({
                    "success": false,
                    "error": format!("Failed to get source agent output: {}", e)
                });
            }
        };

        // Create the new agent
        match manager.create_agent(
            working_dir.to_string(),
            None,
            None,
            crate::types::AgentSource::Meta,
            Arc::new(app_handle.clone()),
        ).await {
            Ok(new_agent_id) => {
                // Construct prompt with context from source agent
                let full_prompt = if source_data.is_empty() {
                    prompt.to_string()
                } else {
                    format!(
                        "Context from previous agent work:\n---\n{}\n---\n\n{}",
                        source_data.trim(),
                        prompt
                    )
                };

                // Send the prompt to the new agent
                if let Err(e) = manager.send_prompt(&new_agent_id, &full_prompt, Some(Arc::new(app_handle))).await {
                    return json!({
                        "success": true,
                        "agent_id": new_agent_id,
                        "warning": format!("Agent created but initial prompt failed: {}", e),
                        "chained_from": source_id
                    });
                }

                json!({
                    "success": true,
                    "agent_id": new_agent_id,
                    "chained_from": source_id,
                    "context_length": source_data.len(),
                    "status": "created and running with context"
                })
            }
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to create agent: {}", e)
            }),
        }
    }

    // =========================================================================
    // Quick Actions
    // =========================================================================

    /// Execute common quick actions
    async fn quick_action(
        &self,
        input: Value,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> Value {
        let action = input["action"].as_str().unwrap_or("");

        match action {
            "status" => {
                // List all agents with their status
                self.list_worker_agents(agent_manager).await
            }
            "stop_all" => {
                // Stop all running agents
                let manager = agent_manager.lock().await;
                let agents = manager.list_agents().await;
                let mut stopped = 0;
                let mut errors = Vec::new();

                for agent in agents {
                    match manager.stop_agent(&agent.id).await {
                        Ok(_) => stopped += 1,
                        Err(e) => errors.push(format!("{}: {}", &agent.id[..8.min(agent.id.len())], e)),
                    }
                }

                // Notify user
                let _ = app_handle.emit(
                    "notification:show",
                    json!({
                        "message": format!("Stopped {} agents", stopped),
                        "type": if errors.is_empty() { "success" } else { "warning" }
                    }),
                );

                json!({
                    "success": true,
                    "stopped_count": stopped,
                    "errors": errors
                })
            }
            "queue" => {
                // Return queue status
                let status = self.get_queue_status();
                json!({
                    "success": true,
                    "queue": status
                })
            }
            "clear_completed" => {
                // This would clear completed agents from tracking
                // For now, just return success
                json!({
                    "success": true,
                    "message": "Cleared completed agents from display"
                })
            }
            _ => json!({
                "success": false,
                "error": format!("Unknown quick action: '{}'. Available actions: status, stop_all, queue, clear_completed", action)
            }),
        }
    }
}
