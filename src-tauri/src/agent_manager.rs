use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::github;
use crate::types::{
    AgentInfo, AgentOutputEvent, AgentStatistics, AgentStatsEvent, AgentStatus, AgentStatusEvent,
    AgentInputRequiredEvent, AgentActivityEvent, OutputMetadata,
};

pub struct AgentProcess {
    pub info: AgentInfo,
    pub child: Option<Child>,
    pub stdin_tx: Option<tokio::sync::mpsc::Sender<String>>,
    pub last_activity: Arc<Mutex<Instant>>,
    pub is_processing: Arc<Mutex<bool>>,
    pub pending_input: Arc<Mutex<bool>>,
    pub stats: Arc<Mutex<AgentStatistics>>,
}

pub struct AgentManager {
    pub agents: Arc<Mutex<HashMap<String, AgentProcess>>>,
    pub session_to_agent: Arc<Mutex<HashMap<String, String>>>,
    pub hook_port: u16,
}

impl AgentManager {
    pub fn new(hook_port: u16) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            session_to_agent: Arc::new(Mutex::new(HashMap::new())),
            hook_port,
        }
    }

    pub async fn create_agent(
        &self,
        working_dir: String,
        github_url: Option<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<String, String> {
        let agent_id = uuid::Uuid::new_v4().to_string();

        // Create a temporary settings file for hooks
        let settings_path = std::env::temp_dir().join(format!("claude_hooks_{}.json", agent_id));
        let hooks_config = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": format!("curl -s -X POST http://127.0.0.1:{}/hook -H 'Content-Type: application/json' -d @-", self.hook_port)
                    }]
                }],
                "PostToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": format!("curl -s -X POST http://127.0.0.1:{}/hook -H 'Content-Type: application/json' -d @-", self.hook_port)
                    }]
                }],
                "Stop": [{
                    "hooks": [{
                        "type": "command",
                        "command": format!("curl -s -X POST http://127.0.0.1:{}/hook -H 'Content-Type: application/json' -d @-", self.hook_port)
                    }]
                }]
            }
        });

        std::fs::write(&settings_path, serde_json::to_string_pretty(&hooks_config).unwrap())
            .map_err(|e| format!("Failed to create settings file: {}", e))?;

        // Spawn claude process (use full path since nvm paths may not be in PATH)
        let claude_path = std::env::var("HOME")
            .map(|home| format!("{}/.nvm/versions/node/v23.11.1/bin/claude", home))
            .unwrap_or_else(|_| "claude".to_string());

        let mut child = Command::new(&claude_path)
            .args([
                "-p",
                "--verbose",
                "--permission-mode",
                "bypassPermissions",
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--settings",
                settings_path.to_str().unwrap(),
            ])
            .current_dir(&working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn claude: {}", e))?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;

        // Create channel for sending prompts
        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(32);

        // Spawn task to handle stdin
        let _stdin_handle = tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(prompt) = stdin_rx.recv().await {
                if stdin.write_all(prompt.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        let now_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        // Build GitHub context if available
        let github_context = github::build_github_context(&working_dir, github_url);

        let agent_info = AgentInfo {
            id: agent_id.clone(),
            working_dir: working_dir.clone(),
            status: AgentStatus::Running,
            session_id: None,
            last_activity: Some(now_millis),
            is_processing: false,
            pending_input: false,
            github_context,
        };

        // Store agent
        let last_activity = Arc::new(Mutex::new(Instant::now()));
        let is_processing = Arc::new(Mutex::new(false));
        let pending_input = Arc::new(Mutex::new(false));

        // Initialize statistics
        let session_start = chrono::Utc::now().to_rfc3339();
        let stats = Arc::new(Mutex::new(AgentStatistics {
            agent_id: agent_id.clone(),
            total_prompts: 0,
            total_tool_calls: 0,
            total_output_bytes: 0,
            session_start: session_start.clone(),
            last_activity: session_start,
            total_tokens_used: None,
            total_cost_usd: None,
        }));

        {
            let mut agents = self.agents.lock().await;
            agents.insert(
                agent_id.clone(),
                AgentProcess {
                    info: agent_info.clone(),
                    child: Some(child),
                    stdin_tx: Some(stdin_tx),
                    last_activity: last_activity.clone(),
                    is_processing: is_processing.clone(),
                    pending_input: pending_input.clone(),
                    stats: stats.clone(),
                },
            );
        }

        // Emit event to notify frontend about new agent
        app_handle
            .emit(
                "agent:status",
                AgentStatusEvent {
                    agent_id: agent_id.clone(),
                    status: AgentStatus::Running,
                    info: Some(agent_info),
                },
            )
            .ok();

        // Spawn task to read stdout
        let agents_clone = self.agents.clone();
        let session_map = self.session_to_agent.clone();
        let app_handle_clone = app_handle.clone();
        let agent_id_clone = agent_id.clone();
        let last_activity_clone = last_activity.clone();
        let is_processing_clone = is_processing.clone();
        let pending_input_clone = pending_input.clone();
        let stats_clone = stats.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[DEBUG] Received line: {}", line);

                // Update activity timestamp
                *last_activity_clone.lock().await = Instant::now();

                // Try to parse as JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    // Extract session_id if present
                    if let Some(session_id) = json.get("session_id").and_then(|v| v.as_str()) {
                        let mut map = session_map.lock().await;
                        map.insert(session_id.to_string(), agent_id_clone.clone());

                        let mut agents = agents_clone.lock().await;
                        if let Some(agent) = agents.get_mut(&agent_id_clone) {
                            agent.info.session_id = Some(session_id.to_string());
                        }
                    }

                    // Parse stream-json format
                    let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");

                    match msg_type {
                        "system" => {
                            // System initialization message - could show in UI
                            let content = serde_json::to_string_pretty(&json).unwrap_or(line.clone());
                            let byte_size = content.len();

                            // Update stats
                            {
                                let mut stats = stats_clone.lock().await;
                                stats.total_output_bytes += byte_size as u64;
                                stats.last_activity = chrono::Utc::now().to_rfc3339();
                            }

                            let _ = app_handle_clone.emit("agent:output", AgentOutputEvent {
                                agent_id: agent_id_clone.clone(),
                                output_type: "system".to_string(),
                                content,
                                parsed_json: Some(json.clone()),
                                metadata: Some(OutputMetadata {
                                    language: Some("json".to_string()),
                                    line_count: None,
                                    byte_size: Some(byte_size),
                                    is_truncated: false,
                                }),
                            });
                        },
                        "assistant" => {
                            // Assistant message with potential tool uses
                            // Mark as not processing since assistant is responding
                            *is_processing_clone.lock().await = false;

                            let mut has_tool_use = false;
                            let mut last_text_output = String::new();

                            if let Some(message) = json.get("message") {
                                if let Some(content_array) = message.get("content").and_then(|v| v.as_array()) {
                                    for content_block in content_array {
                                        if let Some(block_type) = content_block.get("type").and_then(|v| v.as_str()) {
                                            match block_type {
                                                "text" => {
                                                    if let Some(text) = content_block.get("text").and_then(|v| v.as_str()) {
                                                        last_text_output = text.to_string();
                                                        let byte_size = text.len();
                                                        let line_count = text.lines().count();

                                                        // Update stats
                                                        {
                                                            let mut stats = stats_clone.lock().await;
                                                            stats.total_output_bytes += byte_size as u64;
                                                            stats.last_activity = chrono::Utc::now().to_rfc3339();
                                                        }

                                                        let _ = app_handle_clone.emit("agent:output", AgentOutputEvent {
                                                            agent_id: agent_id_clone.clone(),
                                                            output_type: "text".to_string(),
                                                            content: text.to_string(),
                                                            parsed_json: None,
                                                            metadata: Some(OutputMetadata {
                                                                language: None,
                                                                line_count: Some(line_count),
                                                                byte_size: Some(byte_size),
                                                                is_truncated: false,
                                                            }),
                                                        });
                                                    }
                                                },
                                                "tool_use" => {
                                                    has_tool_use = true;
                                                    // Mark as processing when using tools
                                                    *is_processing_clone.lock().await = true;

                                                    // Increment tool call count
                                                    {
                                                        let mut stats = stats_clone.lock().await;
                                                        stats.total_tool_calls += 1;
                                                        stats.last_activity = chrono::Utc::now().to_rfc3339();
                                                    }

                                                    let tool_name = content_block.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                                    let tool_input = content_block.get("input").map(|v| serde_json::to_string_pretty(v).unwrap_or_default()).unwrap_or_default();
                                                    let content = format!("🔧 Using tool: {}\nInput:\n{}", tool_name, tool_input);
                                                    let byte_size = content.len();
                                                    let line_count = content.lines().count();

                                                    // Update output byte stats
                                                    {
                                                        let mut stats = stats_clone.lock().await;
                                                        stats.total_output_bytes += byte_size as u64;
                                                    }

                                                    let _ = app_handle_clone.emit("agent:output", AgentOutputEvent {
                                                        agent_id: agent_id_clone.clone(),
                                                        output_type: "tool_use".to_string(),
                                                        content,
                                                        parsed_json: content_block.get("input").cloned(),
                                                        metadata: Some(OutputMetadata {
                                                            language: None,
                                                            line_count: Some(line_count),
                                                            byte_size: Some(byte_size),
                                                            is_truncated: false,
                                                        }),
                                                    });
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }

                            // If there's text output and no tool use, agent is likely waiting for input
                            if !has_tool_use && !last_text_output.is_empty() {
                                *pending_input_clone.lock().await = true;

                                // Update agent status to WaitingForInput
                                {
                                    let mut agents = agents_clone.lock().await;
                                    if let Some(agent) = agents.get_mut(&agent_id_clone) {
                                        agent.info.status = AgentStatus::WaitingForInput;
                                        agent.info.pending_input = true;
                                    }
                                }

                                // Emit input required event
                                let _ = app_handle_clone.emit("agent:input_required", AgentInputRequiredEvent {
                                    agent_id: agent_id_clone.clone(),
                                    last_output: last_text_output,
                                });
                            }
                        },
                        "user" => {
                            // User message (tool results)
                            if let Some(message) = json.get("message") {
                                if let Some(content_array) = message.get("content").and_then(|v| v.as_array()) {
                                    for content_block in content_array {
                                        if let Some(block_type) = content_block.get("type").and_then(|v| v.as_str()) {
                                            if block_type == "tool_result" {
                                                if let Some(content) = content_block.get("content") {
                                                    let result_text = if let Some(s) = content.as_str() {
                                                        s.to_string()
                                                    } else {
                                                        serde_json::to_string_pretty(content).unwrap_or_default()
                                                    };
                                                    let is_error = content_block.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
                                                    let byte_size = result_text.len();
                                                    let line_count = result_text.lines().count();

                                                    // Update stats
                                                    {
                                                        let mut stats = stats_clone.lock().await;
                                                        stats.total_output_bytes += byte_size as u64;
                                                        stats.last_activity = chrono::Utc::now().to_rfc3339();
                                                    }

                                                    let _ = app_handle_clone.emit("agent:output", AgentOutputEvent {
                                                        agent_id: agent_id_clone.clone(),
                                                        output_type: if is_error { "error".to_string() } else { "tool_result".to_string() },
                                                        content: result_text,
                                                        parsed_json: if !content.is_string() { Some(content.clone()) } else { None },
                                                        metadata: Some(OutputMetadata {
                                                            language: None,
                                                            line_count: Some(line_count),
                                                            byte_size: Some(byte_size),
                                                            is_truncated: false,
                                                        }),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        "result" => {
                            // Final result - extract cost and token data
                            let content = serde_json::to_string_pretty(&json).unwrap_or(line.clone());
                            let byte_size = content.len();

                            // Extract usage statistics from result
                            if let Some(usage) = json.get("usage") {
                                let total_tokens = usage.get("total_tokens")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32);

                                // Calculate cost (approximate based on Claude Sonnet pricing)
                                // Input: $3/MTok, Output: $15/MTok
                                let cost = if let (Some(input_tokens), Some(output_tokens)) = (
                                    usage.get("input_tokens").and_then(|v| v.as_u64()),
                                    usage.get("output_tokens").and_then(|v| v.as_u64()),
                                ) {
                                    let input_cost = (input_tokens as f64 / 1_000_000.0) * 3.0;
                                    let output_cost = (output_tokens as f64 / 1_000_000.0) * 15.0;
                                    Some(input_cost + output_cost)
                                } else {
                                    None
                                };

                                // Update stats
                                {
                                    let mut stats = stats_clone.lock().await;
                                    if let Some(tokens) = total_tokens {
                                        stats.total_tokens_used = Some(
                                            stats.total_tokens_used.unwrap_or(0) + tokens
                                        );
                                    }
                                    if let Some(c) = cost {
                                        stats.total_cost_usd = Some(
                                            stats.total_cost_usd.unwrap_or(0.0) + c
                                        );
                                    }
                                    stats.total_output_bytes += byte_size as u64;
                                    stats.last_activity = chrono::Utc::now().to_rfc3339();
                                }

                                // Emit updated stats
                                let stats_snapshot = stats_clone.lock().await.clone();
                                let _ = app_handle_clone.emit("agent:stats", AgentStatsEvent {
                                    agent_id: agent_id_clone.clone(),
                                    stats: stats_snapshot,
                                });
                            }

                            let _ = app_handle_clone.emit("agent:output", AgentOutputEvent {
                                agent_id: agent_id_clone.clone(),
                                output_type: "result".to_string(),
                                content,
                                parsed_json: Some(json.clone()),
                                metadata: Some(OutputMetadata {
                                    language: Some("json".to_string()),
                                    line_count: None,
                                    byte_size: Some(byte_size),
                                    is_truncated: false,
                                }),
                            });
                        },
                        _ => {
                            // Unknown type - emit as-is
                            let byte_size = line.len();

                            // Update stats
                            {
                                let mut stats = stats_clone.lock().await;
                                stats.total_output_bytes += byte_size as u64;
                                stats.last_activity = chrono::Utc::now().to_rfc3339();
                            }

                            let _ = app_handle_clone.emit("agent:output", AgentOutputEvent {
                                agent_id: agent_id_clone.clone(),
                                output_type: msg_type.to_string(),
                                content: line.clone(),
                                parsed_json: None,
                                metadata: Some(OutputMetadata {
                                    language: None,
                                    line_count: Some(line.lines().count()),
                                    byte_size: Some(byte_size),
                                    is_truncated: false,
                                }),
                            });
                        }
                    }
                } else {
                    // Not JSON, emit as plain text
                    let byte_size = line.len();
                    let line_count = line.lines().count();

                    // Update stats
                    {
                        let mut stats = stats_clone.lock().await;
                        stats.total_output_bytes += byte_size as u64;
                        stats.last_activity = chrono::Utc::now().to_rfc3339();
                    }

                    let event = AgentOutputEvent {
                        agent_id: agent_id_clone.clone(),
                        output_type: "text".to_string(),
                        content: line,
                        parsed_json: None,
                        metadata: Some(OutputMetadata {
                            language: None,
                            line_count: Some(line_count),
                            byte_size: Some(byte_size),
                            is_truncated: false,
                        }),
                    };
                    let _ = app_handle_clone.emit("agent:output", event);
                }
            }

            // Process ended
            let mut agents = agents_clone.lock().await;
            if let Some(agent) = agents.get_mut(&agent_id_clone) {
                agent.info.status = AgentStatus::Stopped;
            }

            let _ = app_handle_clone.emit(
                "agent:status",
                AgentStatusEvent {
                    agent_id: agent_id_clone,
                    status: AgentStatus::Stopped,
                    info: None,
                },
            );
        });

        // Spawn task to read stderr
        let app_handle_clone2 = app_handle.clone();
        let agent_id_clone2 = agent_id.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let byte_size = line.len();
                let line_count = line.lines().count();

                let event = AgentOutputEvent {
                    agent_id: agent_id_clone2.clone(),
                    output_type: "error".to_string(),
                    content: line,
                    parsed_json: None,
                    metadata: Some(OutputMetadata {
                        language: None,
                        line_count: Some(line_count),
                        byte_size: Some(byte_size),
                        is_truncated: false,
                    }),
                };
                let _ = app_handle_clone2.emit("agent:output", event);
            }
        });

        Ok(agent_id)
    }

    pub async fn send_prompt(&self, agent_id: &str, prompt: &str, app_handle: Option<tauri::AppHandle>) -> Result<(), String> {
        eprintln!("[DEBUG] Sending prompt to agent {}: {}", agent_id, prompt);

        let agents = self.agents.lock().await;
        let agent = agents
            .get(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        // Clear pending input flag and set processing
        *agent.pending_input.lock().await = false;
        *agent.is_processing.lock().await = true;

        // Emit activity event to update UI
        if let Some(app_handle) = app_handle.as_ref() {
            let now_millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            app_handle.emit("agent:activity", AgentActivityEvent {
                agent_id: agent_id.to_string(),
                is_processing: true,
                pending_input: false,
                last_activity: now_millis,
            }).ok();
        }

        // Increment prompt counter
        {
            let mut stats = agent.stats.lock().await;
            stats.total_prompts += 1;
            stats.last_activity = chrono::Utc::now().to_rfc3339();
        }

        let stdin_tx = agent
            .stdin_tx
            .as_ref()
            .ok_or_else(|| "Agent stdin not available".to_string())?;

        // Format as stream-json message (JSONL format)
        let message = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": prompt
                }]
            }
        });

        let json_line = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        stdin_tx
            .send(json_line)
            .await
            .map_err(|e| format!("Failed to send prompt: {}", e))?;

        eprintln!("[DEBUG] Prompt sent successfully");
        Ok(())
    }

    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.lock().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        if let Some(mut child) = agent.child.take() {
            let _ = child.kill().await;
        }

        agent.info.status = AgentStatus::Stopped;
        agent.stdin_tx = None;

        Ok(())
    }

    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.lock().await;
        agents.values().map(|a| a.info.clone()).collect()
    }

    pub async fn get_agent_by_session(&self, session_id: &str) -> Option<String> {
        let map = self.session_to_agent.lock().await;
        map.get(session_id).cloned()
    }

    pub async fn get_agent_statistics(&self, agent_id: &str) -> Result<AgentStatistics, String> {
        let agents = self.agents.lock().await;
        let agent = agents
            .get(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        let stats = agent.stats.lock().await.clone();
        Ok(stats)
    }
}
