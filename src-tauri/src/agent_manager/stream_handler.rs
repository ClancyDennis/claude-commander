// Stream handler for parsing Claude CLI stdout/stderr

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::agent_runs_db::{AgentOutputRecord, AgentRunsDB, RunStatus};
use crate::types::{
    AgentInputRequiredEvent, AgentOutputEvent, AgentStatistics, AgentStatsEvent, AgentStatus,
    AgentStatusEvent,
};
use crate::utils::time::now_millis;

use super::output_builder::{extract_common_fields, OutputEventBuilder};
use super::statistics::{increment_tool_calls, update_from_result, update_output_bytes};
use super::types::AgentProcess;

/// Context for stream handling
pub struct StreamContext {
    pub agent_id: String,
    pub agents: Arc<Mutex<HashMap<String, AgentProcess>>>,
    pub session_map: Arc<Mutex<HashMap<String, String>>>,
    pub app_handle: Arc<dyn crate::events::AppEventEmitter>,
    pub last_activity: Arc<Mutex<Instant>>,
    pub is_processing: Arc<Mutex<bool>>,
    pub pending_input: Arc<Mutex<bool>>,
    pub stats: Arc<Mutex<AgentStatistics>>,
    pub output_buffer: Arc<Mutex<Vec<AgentOutputEvent>>>,
    pub runs_db: Option<Arc<AgentRunsDB>>,
    pub pipeline_id: Option<String>,
}

/// Helper to persist agent outputs to the database
async fn persist_output(ctx: &StreamContext, output_type: &str, content: &str) {
    if let Some(ref runs_db) = ctx.runs_db {
        let record = AgentOutputRecord {
            id: None,
            agent_id: ctx.agent_id.clone(),
            pipeline_id: ctx.pipeline_id.clone(),
            output_type: output_type.to_string(),
            content: content.to_string(),
            metadata: None,
            timestamp: now_millis(),
        };

        eprintln!(
            "[persist_output] Inserting output: agent_id={}, pipeline_id={:?}, output_type={}",
            record.agent_id, record.pipeline_id, record.output_type
        );

        // Write synchronously to avoid race conditions
        match runs_db.insert_agent_output(&record).await {
            Ok(id) => {
                eprintln!(
                    "[persist_output] Successfully inserted output with id={}",
                    id
                );
            }
            Err(e) => {
                eprintln!("[persist_output] ERROR inserting output: {}", e);
            }
        }
    } else {
        eprintln!(
            "[persist_output] WARNING: No runs_db available for agent_id={}",
            ctx.agent_id
        );
    }
}

/// Spawn the stdout stream handler task
pub fn spawn_stdout_handler(stdout: ChildStdout, ctx: StreamContext) {
    tokio::spawn(async move {
        handle_stdout_stream(stdout, ctx).await;
    });
}

/// Main stdout stream handling logic
async fn handle_stdout_stream(stdout: ChildStdout, ctx: StreamContext) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    // Helper to store output in buffer (keeps last 100 outputs)
    let store_in_buffer = |output_event: AgentOutputEvent,
                           buffer: Arc<Mutex<Vec<AgentOutputEvent>>>| {
        tokio::spawn(async move {
            let mut buffer = buffer.lock().await;
            buffer.push(output_event);
            let buffer_len = buffer.len();
            if buffer_len > 100 {
                buffer.drain(0..buffer_len - 100);
            }
        });
    };

    while let Ok(Some(line)) = lines.next_line().await {
        // Update activity timestamp
        *ctx.last_activity.lock().await = Instant::now();

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            // Extract session_id if present
            if let Some(session_id) = json.get("session_id").and_then(|v| v.as_str()) {
                let mut map = ctx.session_map.lock().await;
                map.insert(session_id.to_string(), ctx.agent_id.clone());

                let mut agents = ctx.agents.lock().await;
                if let Some(agent) = agents.get_mut(&ctx.agent_id) {
                    agent.info.session_id = Some(session_id.to_string());
                }
            }

            // Parse stream-json format
            let msg_type = json
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            match msg_type {
                "system" => {
                    handle_system_message(&ctx, &json, &line, &store_in_buffer).await;
                }
                "assistant" => {
                    handle_assistant_message(&ctx, &json, &store_in_buffer).await;
                }
                "user" => {
                    handle_user_message(&ctx, &json, &store_in_buffer).await;
                }
                "result" => {
                    handle_result_message(&ctx, &json, &line, &store_in_buffer).await;
                }
                "stream_event" => {
                    handle_stream_event(&ctx, &json, &line).await;
                }
                _ => {
                    handle_unknown_message(&ctx, &json, &line, msg_type).await;
                }
            }
        } else {
            // Not JSON, emit as plain text
            handle_plain_text(&ctx, &line).await;
        }
    }

    // Process ended - handle completion
    handle_process_end(&ctx).await;
}

async fn handle_system_message<F>(
    ctx: &StreamContext,
    json: &serde_json::Value,
    _line: &str,
    _store_in_buffer: &F,
) where
    F: Fn(AgentOutputEvent, Arc<Mutex<Vec<AgentOutputEvent>>>),
{
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

    // Extract meaningful content from system message instead of full JSON
    let content = if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        message.to_string()
    } else if let Some(init) = json.get("init") {
        // Handle init messages - extract model/tools info
        let model = init
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let tools_count = init
            .get("tools")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        format!(
            "Session initialized with {} ({} tools available)",
            model, tools_count
        )
    } else if let Some(subtype_str) = &subtype {
        format!("System: {}", subtype_str)
    } else {
        "System event".to_string()
    };

    let byte_size = content.len();
    update_output_bytes(&ctx.stats, byte_size).await;

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("system")
        .content(&content)
        .parsed_json(Some(json.clone()))
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());

    // Persist to database
    persist_output(ctx, "system", &content).await;
}

async fn handle_assistant_message<F>(
    ctx: &StreamContext,
    json: &serde_json::Value,
    store_in_buffer: &F,
) where
    F: Fn(AgentOutputEvent, Arc<Mutex<Vec<AgentOutputEvent>>>),
{
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);
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

                                update_output_bytes(&ctx.stats, byte_size).await;

                                let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
                                    .output_type("text")
                                    .content(text)
                                    .session_context(
                                        session_id.clone(),
                                        uuid.clone(),
                                        parent_tool_use_id.clone(),
                                        subtype.clone(),
                                    )
                                    .with_size_from_content()
                                    .build();

                                store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
                                let _ = ctx.app_handle.emit(
                                    "agent:output",
                                    serde_json::to_value(output_event).unwrap(),
                                );

                                // Persist to database
                                persist_output(ctx, "text", text).await;
                            }
                        }
                        "tool_use" => {
                            has_tool_use = true;
                            *ctx.is_processing.lock().await = true;

                            increment_tool_calls(&ctx.stats).await;

                            let tool_name = content_block
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let tool_input = content_block
                                .get("input")
                                .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
                                .unwrap_or_default();
                            let content =
                                format!("🔧 Using tool: {}\nInput:\n{}", tool_name, tool_input);
                            let byte_size = content.len();

                            update_output_bytes(&ctx.stats, byte_size).await;

                            let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
                                .output_type("tool_use")
                                .content(&content)
                                .parsed_json(content_block.get("input").cloned())
                                .session_context(
                                    session_id.clone(),
                                    uuid.clone(),
                                    parent_tool_use_id.clone(),
                                    subtype.clone(),
                                )
                                .with_size_from_content()
                                .build();

                            store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
                            let _ = ctx
                                .app_handle
                                .emit("agent:output", serde_json::to_value(output_event).unwrap());

                            // Persist to database
                            persist_output(ctx, "tool_use", &content).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Check stop_reason to determine if agent is waiting for input
    let stop_reason = json
        .get("message")
        .and_then(|msg| msg.get("stop_reason"))
        .and_then(|sr| sr.as_str())
        .unwrap_or("");

    if stop_reason == "end_turn" && !has_tool_use {
        // Agent finished its turn without tool use - waiting for user input
        *ctx.pending_input.lock().await = true;
        *ctx.is_processing.lock().await = false;

        // Update agent status to WaitingForInput
        {
            let mut agents = ctx.agents.lock().await;
            if let Some(agent) = agents.get_mut(&ctx.agent_id) {
                agent.info.status = AgentStatus::WaitingForInput;
                agent.info.pending_input = true;
            }
        }

        // Emit input required event
        let _ = ctx.app_handle.emit(
            "agent:input_required",
            serde_json::to_value(AgentInputRequiredEvent {
                agent_id: ctx.agent_id.clone(),
                last_output: last_text_output,
            })
            .unwrap(),
        );
    } else if has_tool_use {
        *ctx.pending_input.lock().await = false;
    } else {
        *ctx.is_processing.lock().await = true;
        *ctx.pending_input.lock().await = false;
    }
}

async fn handle_user_message<F>(ctx: &StreamContext, json: &serde_json::Value, store_in_buffer: &F)
where
    F: Fn(AgentOutputEvent, Arc<Mutex<Vec<AgentOutputEvent>>>),
{
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

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
                            let is_error = content_block
                                .get("is_error")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let byte_size = result_text.len();
                            let output_type = if is_error { "error" } else { "tool_result" };

                            update_output_bytes(&ctx.stats, byte_size).await;

                            let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
                                .output_type(output_type)
                                .content(&result_text)
                                .parsed_json(if !content.is_string() {
                                    Some(content.clone())
                                } else {
                                    None
                                })
                                .session_context(
                                    session_id.clone(),
                                    uuid.clone(),
                                    parent_tool_use_id.clone(),
                                    subtype.clone(),
                                )
                                .with_size_from_content()
                                .build();

                            store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
                            let _ = ctx
                                .app_handle
                                .emit("agent:output", serde_json::to_value(output_event).unwrap());

                            // Persist to database
                            persist_output(ctx, output_type, &result_text).await;
                        }
                    }
                }
            }
        }
    }
}

async fn handle_result_message<F>(
    ctx: &StreamContext,
    json: &serde_json::Value,
    _line: &str,
    store_in_buffer: &F,
) where
    F: Fn(AgentOutputEvent, Arc<Mutex<Vec<AgentOutputEvent>>>),
{
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

    // Check if this is a successful completion
    let is_success = subtype.as_deref() == Some("success");

    // Extract meaningful content from result message instead of full JSON
    let content = if let Some(result) = json.get("result") {
        if let Some(result_str) = result.as_str() {
            // Result is a string
            result_str.to_string()
        } else if let Some(summary) = result.get("summary").and_then(|v| v.as_str()) {
            // Result has a summary field
            summary.to_string()
        } else if let Some(message) = result.get("message").and_then(|v| v.as_str()) {
            // Result has a message field
            message.to_string()
        } else {
            // Result is an object without clear text - format key info
            let cost = json
                .get("cost_usd")
                .and_then(|v| v.as_f64())
                .map(|c| format!(" (${:.4})", c))
                .unwrap_or_default();
            let duration = json
                .get("duration_ms")
                .and_then(|v| v.as_u64())
                .map(|d| format!(" in {:.1}s", d as f64 / 1000.0))
                .unwrap_or_default();
            format!("Task completed{}{}", duration, cost)
        }
    } else if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
        format!("Error: {}", error)
    } else {
        // Fallback based on subtype
        match subtype.as_deref() {
            Some("success") => "Task completed successfully".to_string(),
            Some("error") => "Task failed".to_string(),
            _ => "Result".to_string(),
        }
    };

    let byte_size = content.len();

    // Update statistics from result message
    update_from_result(&ctx.stats, json, byte_size).await;

    // Emit updated stats
    let stats_snapshot = ctx.stats.lock().await.clone();
    let _ = ctx.app_handle.emit(
        "agent:stats",
        serde_json::to_value(AgentStatsEvent {
            agent_id: ctx.agent_id.clone(),
            stats: stats_snapshot,
        })
        .unwrap(),
    );

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("result")
        .content(&content)
        .parsed_json(Some(json.clone()))
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());

    // Persist to database
    persist_output(ctx, "result", &content).await;

    // Check if this is a successful completion
    if is_success {
        *ctx.pending_input.lock().await = true;
        *ctx.is_processing.lock().await = false;

        // Update agent status to WaitingForInput
        {
            let mut agents = ctx.agents.lock().await;
            if let Some(agent) = agents.get_mut(&ctx.agent_id) {
                agent.info.status = AgentStatus::WaitingForInput;
                agent.info.pending_input = true;
            }
        }

        // Emit input required event
        let _ = ctx.app_handle.emit(
            "agent:input_required",
            serde_json::to_value(AgentInputRequiredEvent {
                agent_id: ctx.agent_id.clone(),
                last_output: String::new(),
            })
            .unwrap(),
        );
    }
}

async fn handle_stream_event(ctx: &StreamContext, json: &serde_json::Value, _line: &str) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

    // Extract meaningful content from stream event instead of full JSON
    let event_type = json
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("stream");
    let content = if let Some(data) = json.get("data") {
        if let Some(text) = data.as_str() {
            text.to_string()
        } else if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
            msg.to_string()
        } else if let Some(status) = data.get("status").and_then(|v| v.as_str()) {
            format!("{}: {}", event_type, status)
        } else {
            format!("Stream: {}", event_type)
        }
    } else if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        message.to_string()
    } else {
        format!("Stream: {}", event_type)
    };

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("stream_event")
        .content(content)
        .parsed_json(Some(json.clone()))
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());
}

async fn handle_unknown_message(
    ctx: &StreamContext,
    json: &serde_json::Value,
    line: &str,
    msg_type: &str,
) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);
    let byte_size = line.len();

    update_output_bytes(&ctx.stats, byte_size).await;

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type(msg_type)
        .content(line)
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());
}

async fn handle_plain_text(ctx: &StreamContext, line: &str) {
    let byte_size = line.len();

    update_output_bytes(&ctx.stats, byte_size).await;

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("text")
        .content(line)
        .with_size_from_content()
        .build();

    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());

    // Persist to database
    persist_output(ctx, "text", line).await;
}

async fn handle_process_end(ctx: &StreamContext) {
    // Process ended - check if it was expected or a crash
    let (was_stopped, final_stats) = {
        let mut agents = ctx.agents.lock().await;
        if let Some(agent) = agents.get_mut(&ctx.agent_id) {
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
    if let Some(runs_db) = ctx.runs_db.clone() {
        let agent_id = ctx.agent_id.clone();
        tokio::spawn(async move {
            if let Ok(Some(mut run)) = runs_db.get_run(&agent_id).await {
                let now = now_millis();

                if !was_stopped {
                    run.status = RunStatus::Crashed;
                    run.error_message = Some("Process terminated unexpectedly".to_string());
                    run.can_resume = true;
                }

                run.ended_at = Some(now);
                run.last_activity = now;

                // Update final stats if available
                if let Some(stats) = final_stats {
                    let stats_lock = stats.lock().await;
                    run.total_prompts = stats_lock.total_prompts;
                    run.total_tool_calls = stats_lock.total_tool_calls;
                    run.total_output_bytes = stats_lock.total_output_bytes;
                    run.total_tokens_used = stats_lock.total_tokens_used;
                    run.total_cost_usd = stats_lock.total_cost_usd;

                    if let Some(ref model_usage) = stats_lock.model_usage {
                        run.model_usage = serde_json::to_string(&model_usage).ok();
                    }
                }

                let _ = runs_db.update_run(&run).await;
            }
        });
    }

    let status = if was_stopped {
        AgentStatus::Stopped
    } else {
        AgentStatus::Error
    };

    let _ = ctx.app_handle.emit(
        "agent:status",
        serde_json::to_value(AgentStatusEvent {
            agent_id: ctx.agent_id.clone(),
            status,
            info: None,
        })
        .unwrap(),
    );
}

/// Spawn the stderr stream handler task
pub fn spawn_stderr_handler(
    stderr: tokio::process::ChildStderr,
    agent_id: String,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
    runs_db: Option<Arc<AgentRunsDB>>,
    pipeline_id: Option<String>,
) {
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let output_event = OutputEventBuilder::new(agent_id.clone())
                .output_type("error")
                .content(&line)
                .with_size_from_content()
                .build();

            let _ = app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());

            // Persist error to database
            if let Some(ref db) = runs_db {
                let record = AgentOutputRecord {
                    id: None,
                    agent_id: agent_id.clone(),
                    pipeline_id: pipeline_id.clone(),
                    output_type: "error".to_string(),
                    content: line.clone(),
                    metadata: None,
                    timestamp: now_millis(),
                };
                let db_clone = db.clone();
                tokio::spawn(async move {
                    let _ = db_clone.insert_agent_output(&record).await;
                });
            }
        }
    });
}
