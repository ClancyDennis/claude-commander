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
use crate::logger::Logger;
use crate::types::{
    AgentInfo, AgentOutputEvent, AgentStatistics, AgentStatsEvent, AgentStatus, AgentStatusEvent,
    AgentInputRequiredEvent, AgentActivityEvent, OutputMetadata,
};
use crate::agent_runs_db::{AgentRunsDB, AgentRun, RunStatus};

pub struct AgentProcess {
    pub info: AgentInfo,
    pub child: Option<Child>,
    pub stdin_tx: Option<tokio::sync::mpsc::Sender<String>>,
    pub last_activity: Arc<Mutex<Instant>>,
    pub is_processing: Arc<Mutex<bool>>,
    pub pending_input: Arc<Mutex<bool>>,
    pub stats: Arc<Mutex<AgentStatistics>>,
    pub output_buffer: Arc<Mutex<Vec<AgentOutputEvent>>>,
    pub copied_instruction_files: Vec<String>,  // Track copied files for cleanup
}

pub struct AgentManager {
    pub agents: Arc<Mutex<HashMap<String, AgentProcess>>>,
    pub session_to_agent: Arc<Mutex<HashMap<String, String>>>,
    pub hook_port: u16,
    pub logger: Option<Arc<Logger>>,
    pub runs_db: Option<Arc<AgentRunsDB>>,
    pub on_agent_created: Option<Arc<dyn Fn(String, crate::types::AgentSource) + Send + Sync>>,
}

impl AgentManager {
    pub fn new(hook_port: u16) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            session_to_agent: Arc::new(Mutex::new(HashMap::new())),
            hook_port,
            logger: None,
            runs_db: None,
            on_agent_created: None,
        }
    }

    pub fn with_logger(hook_port: u16, logger: Arc<Logger>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            session_to_agent: Arc::new(Mutex::new(HashMap::new())),
            hook_port,
            logger: Some(logger),
            runs_db: None,
            on_agent_created: None,
        }
    }

    pub fn with_logger_and_db(hook_port: u16, logger: Arc<Logger>, runs_db: Arc<AgentRunsDB>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            session_to_agent: Arc::new(Mutex::new(HashMap::new())),
            hook_port,
            logger: Some(logger),
            runs_db: Some(runs_db),
            on_agent_created: None,
        }
    }

    /// Set callback to be invoked when an agent is created
    pub fn set_on_agent_created<F>(&mut self, callback: F)
    where
        F: Fn(String, crate::types::AgentSource) + Send + Sync + 'static,
    {
        self.on_agent_created = Some(Arc::new(callback));
    }

    pub async fn create_agent(
        &self,
        working_dir: String,
        github_url: Option<String>,
        selected_instruction_files: Option<Vec<String>>,
        source: crate::types::AgentSource,
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

        // Copy instruction files to .claude/ directory if any are selected
        let copied_files = if let Some(files) = selected_instruction_files {
            crate::instruction_manager::copy_instruction_files(&working_dir, &files)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to copy instruction files: {}", e);
                    Vec::new()
                })
        } else {
            Vec::new()
        };

        // Spawn claude process (use full path since nvm paths may not be in PATH)
        // First try CLAUDE_PATH env var, then try to construct from HOME, finally fallback to just "claude"
        let claude_path = std::env::var("CLAUDE_PATH")
            .or_else(|_| {
                std::env::var("HOME")
                    .map(|home| format!("{}/.nvm/versions/node/v23.11.1/bin/claude", home))
            })
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
        let github_context = github::build_github_context(&working_dir, github_url.clone());

        let agent_info = AgentInfo {
            id: agent_id.clone(),
            working_dir: working_dir.clone(),
            status: AgentStatus::Running,
            session_id: None,
            last_activity: Some(now_millis),
            is_processing: false,
            pending_input: false,
            github_context: github_context.clone(),
            source: source.clone(),
            pooled: None,
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
            model_usage: None,
            duration_api_ms: None,
            duration_ms: None,
            num_turns: None,
        }));

        // Initialize output buffer
        let output_buffer = Arc::new(Mutex::new(Vec::new()));

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
                    output_buffer: output_buffer.clone(),
                    copied_instruction_files: copied_files,
                },
            );
        }

        // Record run in database
        if let Some(ref runs_db) = self.runs_db {
            let github_context_json = github_context.as_ref()
                .map(|gc| serde_json::to_string(gc).ok())
                .flatten();

            let run = AgentRun {
                id: None,
                agent_id: agent_id.clone(),
                session_id: None,
                working_dir: working_dir.clone(),
                github_url: github_url.clone(),
                github_context: github_context_json,
                source: match source {
                    crate::types::AgentSource::UI => "ui".to_string(),
                    crate::types::AgentSource::Meta => "meta".to_string(),
                    crate::types::AgentSource::Pipeline => "pipeline".to_string(),
                    crate::types::AgentSource::Pool => "pool".to_string(),
                    crate::types::AgentSource::Manual => "manual".to_string(),
                },
                status: RunStatus::Running,
                started_at: now_millis,
                ended_at: None,
                last_activity: now_millis,
                initial_prompt: None,
                error_message: None,
                total_prompts: 0,
                total_tool_calls: 0,
                total_output_bytes: 0,
                total_tokens_used: None,
                total_cost_usd: None,
                can_resume: true,
                resume_data: None,
            };

            if let Err(e) = runs_db.create_run(&run).await {
                if let Some(ref logger) = self.logger {
                    let _ = logger.error("agent_manager", &format!("Failed to record run in database: {}", e), Some(agent_id.clone()), None).await;
                }
            }
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
        let output_buffer_clone = output_buffer.clone();
        let runs_db_clone = self.runs_db.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            // Helper function to extract common fields from Claude messages
            let extract_common_fields = |json: &serde_json::Value| -> (Option<String>, Option<String>, Option<String>, Option<String>) {
                let session_id = json.get("session_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let uuid = json.get("uuid").and_then(|v| v.as_str()).map(|s| s.to_string());
                let parent_tool_use_id = json.get("parent_tool_use_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let subtype = json.get("subtype").and_then(|v| v.as_str()).map(|s| s.to_string());
                (session_id, uuid, parent_tool_use_id, subtype)
            };

            // Helper function to store output in buffer (keeps last 100 outputs)
            let store_in_buffer = |output_event: AgentOutputEvent| {
                let buffer_clone = output_buffer_clone.clone();
                tokio::spawn(async move {
                    let mut buffer = buffer_clone.lock().await;
                    buffer.push(output_event);
                    // Keep only last 100 outputs
                    let buffer_len = buffer.len();
                    if buffer_len > 100 {
                        buffer.drain(0..buffer_len - 100);
                    }
                });
            };

            while let Ok(Some(line)) = lines.next_line().await {
                // Log incoming line (only if verbose logging is needed)
                // if let Some(ref logger) = self.logger {
                //     let _ = logger.debug("agent_output", &format!("Received: {}", line), Some(agent_id_clone.clone()), None).await;
                // }

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
                            let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(&json);
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
                                session_id,
                                uuid,
                                parent_tool_use_id,
                                subtype,
                            });
                        },
                        "assistant" => {
                            // Assistant message with potential tool uses
                            let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(&json);
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

                                                        let output_event = AgentOutputEvent {
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
                                                            session_id: session_id.clone(),
                                                            uuid: uuid.clone(),
                                                            parent_tool_use_id: parent_tool_use_id.clone(),
                                                            subtype: subtype.clone(),
                                                        };
                                                        store_in_buffer(output_event.clone());
                                                        let _ = app_handle_clone.emit("agent:output", output_event);
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

                                                    let output_event = AgentOutputEvent {
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
                                                        session_id: session_id.clone(),
                                                        uuid: uuid.clone(),
                                                        parent_tool_use_id: parent_tool_use_id.clone(),
                                                        subtype: subtype.clone(),
                                                    };
                                                    store_in_buffer(output_event.clone());
                                                    let _ = app_handle_clone.emit("agent:output", output_event);
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }

                            // Check stop_reason to determine if agent is waiting for input
                            // Only mark as waiting if stop_reason is "end_turn" (agent finished its turn and expects user response)
                            let stop_reason = json.get("message")
                                .and_then(|msg| msg.get("stop_reason"))
                                .and_then(|sr| sr.as_str())
                                .unwrap_or("");

                            if stop_reason == "end_turn" && !has_tool_use {
                                // Agent finished its turn without tool use - waiting for user input
                                *pending_input_clone.lock().await = true;
                                *is_processing_clone.lock().await = false;

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
                            } else if has_tool_use {
                                // Agent is processing tools
                                *pending_input_clone.lock().await = false;
                            } else {
                                // Agent is still working (e.g., stop_reason might be "tool_use" or message is partial)
                                *is_processing_clone.lock().await = true;
                                *pending_input_clone.lock().await = false;
                            }
                        },
                        "user" => {
                            // User message (tool results)
                            let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(&json);

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

                                                    let output_event = AgentOutputEvent {
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
                                                        session_id: session_id.clone(),
                                                        uuid: uuid.clone(),
                                                        parent_tool_use_id: parent_tool_use_id.clone(),
                                                        subtype: subtype.clone(),
                                                    };
                                                    store_in_buffer(output_event.clone());
                                                    let _ = app_handle_clone.emit("agent:output", output_event);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        "result" => {
                            // Final result - extract detailed cost and performance data
                            let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(&json);
                            let content = serde_json::to_string_pretty(&json).unwrap_or(line.clone());
                            let byte_size = content.len();

                            // Extract comprehensive statistics from result message
                            {
                                let mut stats = stats_clone.lock().await;

                                // Extract total_cost_usd directly from result (most accurate)
                                if let Some(total_cost) = json.get("total_cost_usd").and_then(|v| v.as_f64()) {
                                    stats.total_cost_usd = Some(
                                        stats.total_cost_usd.unwrap_or(0.0) + total_cost
                                    );
                                }

                                // Extract modelUsage for detailed per-model breakdown
                                if let Some(model_usage_obj) = json.get("modelUsage").and_then(|v| v.as_object()) {
                                    let mut model_usage_map = stats.model_usage.take().unwrap_or_default();

                                    for (model_name, model_data) in model_usage_obj {
                                        let usage_stats = crate::types::ModelUsageStats {
                                            input_tokens: model_data.get("inputTokens").and_then(|v| v.as_u64()),
                                            output_tokens: model_data.get("outputTokens").and_then(|v| v.as_u64()),
                                            cache_creation_input_tokens: model_data.get("cacheCreationInputTokens").and_then(|v| v.as_u64()),
                                            cache_read_input_tokens: model_data.get("cacheReadInputTokens").and_then(|v| v.as_u64()),
                                            cost_usd: model_data.get("costUSD").and_then(|v| v.as_f64()),
                                            context_window: model_data.get("contextWindow").and_then(|v| v.as_u64()),
                                            max_output_tokens: model_data.get("maxOutputTokens").and_then(|v| v.as_u64()),
                                        };

                                        // Accumulate or insert model usage
                                        let entry = model_usage_map.entry(model_name.clone()).or_insert_with(|| {
                                            crate::types::ModelUsageStats {
                                                input_tokens: Some(0),
                                                output_tokens: Some(0),
                                                cache_creation_input_tokens: Some(0),
                                                cache_read_input_tokens: Some(0),
                                                cost_usd: Some(0.0),
                                                context_window: usage_stats.context_window,
                                                max_output_tokens: usage_stats.max_output_tokens,
                                            }
                                        });

                                        if let Some(tokens) = usage_stats.input_tokens {
                                            entry.input_tokens = Some(entry.input_tokens.unwrap_or(0) + tokens);
                                        }
                                        if let Some(tokens) = usage_stats.output_tokens {
                                            entry.output_tokens = Some(entry.output_tokens.unwrap_or(0) + tokens);
                                        }
                                        if let Some(tokens) = usage_stats.cache_creation_input_tokens {
                                            entry.cache_creation_input_tokens = Some(entry.cache_creation_input_tokens.unwrap_or(0) + tokens);
                                        }
                                        if let Some(tokens) = usage_stats.cache_read_input_tokens {
                                            entry.cache_read_input_tokens = Some(entry.cache_read_input_tokens.unwrap_or(0) + tokens);
                                        }
                                        if let Some(cost) = usage_stats.cost_usd {
                                            entry.cost_usd = Some(entry.cost_usd.unwrap_or(0.0) + cost);
                                        }
                                    }

                                    stats.model_usage = Some(model_usage_map);
                                }

                                // Extract performance metrics
                                if let Some(duration) = json.get("duration_api_ms").and_then(|v| v.as_u64()) {
                                    stats.duration_api_ms = Some(
                                        stats.duration_api_ms.unwrap_or(0) + duration
                                    );
                                }
                                if let Some(duration) = json.get("duration_ms").and_then(|v| v.as_u64()) {
                                    stats.duration_ms = Some(
                                        stats.duration_ms.unwrap_or(0) + duration
                                    );
                                }
                                if let Some(turns) = json.get("num_turns").and_then(|v| v.as_u64()) {
                                    stats.num_turns = Some(
                                        stats.num_turns.unwrap_or(0) + turns as u32
                                    );
                                }

                                // Extract token counts from usage object (for total_tokens_used)
                                if let Some(usage) = json.get("usage") {
                                    let input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                                    let output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                                    let total_tokens = input_tokens + output_tokens;

                                    if total_tokens > 0 {
                                        stats.total_tokens_used = Some(
                                            stats.total_tokens_used.unwrap_or(0) + total_tokens as u32
                                        );
                                    }
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

                            let output_event = AgentOutputEvent {
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
                                session_id,
                                uuid,
                                parent_tool_use_id,
                                subtype,
                            };
                            store_in_buffer(output_event.clone());
                            let _ = app_handle_clone.emit("agent:output", output_event);
                        },
                        "stream_event" => {
                            // Streaming partial updates (if --include-partial-messages is enabled)
                            let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(&json);
                            let content = serde_json::to_string_pretty(&json).unwrap_or(line.clone());
                            let byte_size = content.len();

                            let _ = app_handle_clone.emit("agent:output", AgentOutputEvent {
                                agent_id: agent_id_clone.clone(),
                                output_type: "stream_event".to_string(),
                                content,
                                parsed_json: Some(json.clone()),
                                metadata: Some(OutputMetadata {
                                    language: Some("json".to_string()),
                                    line_count: None,
                                    byte_size: Some(byte_size),
                                    is_truncated: false,
                                }),
                                session_id,
                                uuid,
                                parent_tool_use_id,
                                subtype,
                            });
                        },
                        _ => {
                            // Unknown type - emit as-is
                            let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(&json);
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
                                session_id,
                                uuid,
                                parent_tool_use_id,
                                subtype,
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
                        session_id: None,
                        uuid: None,
                        parent_tool_use_id: None,
                        subtype: None,
                    };
                    let _ = app_handle_clone.emit("agent:output", event);
                }
            }

            // Process ended - check if it was expected or a crash
            let (was_stopped, final_stats) = {
                let mut agents = agents_clone.lock().await;
                if let Some(agent) = agents.get_mut(&agent_id_clone) {
                    let was_stopped = agent.info.status == AgentStatus::Stopped;
                    let stats = agent.stats.clone();
                    if !was_stopped {
                        agent.info.status = AgentStatus::Error;
                    }
                    (was_stopped, Some(stats))
                } else {
                    (false, None)
                }
            };

            // Update database - mark as crashed if not explicitly stopped
            let agent_id_for_db = agent_id_clone.clone();
            tokio::spawn(async move {
                if let Some(runs_db) = runs_db_clone {
                    if let Ok(Some(mut run)) = runs_db.get_run(&agent_id_for_db).await {
                        let now_millis = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as i64;

                        if !was_stopped {
                            run.status = RunStatus::Crashed;
                            run.error_message = Some("Process terminated unexpectedly".to_string());
                            run.can_resume = true; // Mark as resumable
                        }

                        run.ended_at = Some(now_millis);
                        run.last_activity = now_millis;

                        // Update final stats if available
                        if let Some(stats) = final_stats {
                            let stats_lock = stats.lock().await;
                            run.total_prompts = stats_lock.total_prompts;
                            run.total_tool_calls = stats_lock.total_tool_calls;
                            run.total_output_bytes = stats_lock.total_output_bytes;
                            run.total_tokens_used = stats_lock.total_tokens_used;
                            run.total_cost_usd = stats_lock.total_cost_usd;
                        }

                        let _ = runs_db.update_run(&run).await;
                    }
                }
            });

            let status = if was_stopped {
                AgentStatus::Stopped
            } else {
                AgentStatus::Error
            };

            let _ = app_handle_clone.emit(
                "agent:status",
                AgentStatusEvent {
                    agent_id: agent_id_clone,
                    status,
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
                    session_id: None,
                    uuid: None,
                    parent_tool_use_id: None,
                    subtype: None,
                };
                let _ = app_handle_clone2.emit("agent:output", event);
            }
        });

        // Call the on_agent_created callback if set
        if let Some(callback) = &self.on_agent_created {
            callback(agent_id.clone(), source.clone());
        }

        Ok(agent_id)
    }

    pub async fn send_prompt(&self, agent_id: &str, prompt: &str, app_handle: Option<tauri::AppHandle>) -> Result<(), String> {
        // Log prompt sending
        if let Some(ref logger) = self.logger {
            let _ = logger.info("agent_manager", &format!("Sending prompt to agent: {}", prompt), Some(agent_id.to_string()), None).await;
        }

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

        // Record prompt in database
        if let Some(ref runs_db) = self.runs_db {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            if let Err(e) = runs_db.record_prompt(agent_id, prompt, timestamp).await {
                if let Some(ref logger) = self.logger {
                    let _ = logger.error("agent_manager", &format!("Failed to record prompt: {}", e), Some(agent_id.to_string()), None).await;
                }
            }

            // Update run in database
            if let Ok(Some(mut run)) = runs_db.get_run(agent_id).await {
                run.last_activity = timestamp;
                run.total_prompts += 1;
                // Set initial prompt if this is the first one
                if run.total_prompts == 1 {
                    run.initial_prompt = Some(prompt.to_string());
                }
                let _ = runs_db.update_run(&run).await;
            }
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
            .map_err(|e| {
                if let Some(ref logger) = self.logger {
                    let _ = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            logger.error("agent_manager", &format!("Failed to send prompt: {}", e), Some(agent_id.to_string()), None).await
                        })
                    });
                }
                format!("Failed to send prompt: {}", e)
            })?;

        Ok(())
    }

    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), String> {
        // Get final stats before stopping
        let final_stats = {
            let agents = self.agents.lock().await;
            agents.get(agent_id)
                .map(|agent| agent.stats.clone())
        };

        let mut agents = self.agents.lock().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        if let Some(mut child) = agent.child.take() {
            let _ = child.kill().await;
        }

        // Clean up copied instruction files
        if !agent.copied_instruction_files.is_empty() {
            let working_dir = agent.info.working_dir.clone();
            let copied_files = agent.copied_instruction_files.clone();
            if let Err(e) = crate::instruction_manager::cleanup_instruction_files(
                &working_dir,
                &copied_files
            ) {
                eprintln!("Warning: Failed to cleanup instruction files: {}", e);
            }
        }

        agent.info.status = AgentStatus::Stopped;
        agent.stdin_tx = None;

        // Update run in database
        if let Some(ref runs_db) = self.runs_db {
            if let Ok(Some(mut run)) = runs_db.get_run(agent_id).await {
                let now_millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64;

                run.status = RunStatus::Stopped;
                run.ended_at = Some(now_millis);
                run.last_activity = now_millis;

                // Update stats if available
                if let Some(stats) = final_stats {
                    let stats_lock = stats.lock().await;
                    run.total_prompts = stats_lock.total_prompts;
                    run.total_tool_calls = stats_lock.total_tool_calls;
                    run.total_output_bytes = stats_lock.total_output_bytes;
                    run.total_tokens_used = stats_lock.total_tokens_used;
                    run.total_cost_usd = stats_lock.total_cost_usd;
                }

                let _ = runs_db.update_run(&run).await;
            }
        }

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

    pub async fn get_agent_outputs(&self, agent_id: &str, last_n: usize) -> Result<Vec<AgentOutputEvent>, String> {
        let agents = self.agents.lock().await;
        let agent = agents
            .get(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        let buffer = agent.output_buffer.lock().await;
        let outputs = if last_n == 0 || last_n >= buffer.len() {
            buffer.clone()
        } else {
            buffer[buffer.len() - last_n..].to_vec()
        };
        Ok(outputs)
    }
}
