// Stream handler for parsing Claude CLI stdout/stderr

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::agent_runs_db::{AgentRunsDB, RunStatus};
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
    let store_in_buffer = |output_event: AgentOutputEvent, buffer: Arc<Mutex<Vec<AgentOutputEvent>>>| {
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
    line: &str,
    _store_in_buffer: &F,
) where
    F: Fn(AgentOutputEvent, Arc<Mutex<Vec<AgentOutputEvent>>>),
{
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);
    let content = serde_json::to_string_pretty(json).unwrap_or_else(|_| line.to_string());
    let byte_size = content.len();

    update_output_bytes(&ctx.stats, byte_size).await;

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("system")
        .content(content)
        .parsed_json(Some(json.clone()))
        .language(Some("json".to_string()))
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());
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
                                let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());
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
                                .content(content)
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
                            let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());
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
            }).unwrap(),
        );
    } else if has_tool_use {
        *ctx.pending_input.lock().await = false;
    } else {
        *ctx.is_processing.lock().await = true;
        *ctx.pending_input.lock().await = false;
    }
}

async fn handle_user_message<F>(
    ctx: &StreamContext,
    json: &serde_json::Value,
    store_in_buffer: &F,
) where
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

                            update_output_bytes(&ctx.stats, byte_size).await;

                            let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
                                .output_type(if is_error { "error" } else { "tool_result" })
                                .content(result_text)
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
                            let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());
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
    line: &str,
    store_in_buffer: &F,
) where
    F: Fn(AgentOutputEvent, Arc<Mutex<Vec<AgentOutputEvent>>>),
{
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);
    let content = serde_json::to_string_pretty(json).unwrap_or_else(|_| line.to_string());
    let byte_size = content.len();

    // Check if this is a successful completion
    let is_success = subtype.as_deref() == Some("success");

    // Update statistics from result message
    update_from_result(&ctx.stats, json, byte_size).await;

    // Emit updated stats
    let stats_snapshot = ctx.stats.lock().await.clone();
    let _ = ctx.app_handle.emit(
        "agent:stats",
        serde_json::to_value(AgentStatsEvent {
            agent_id: ctx.agent_id.clone(),
            stats: stats_snapshot,
        }).unwrap(),
    );

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("result")
        .content(content)
        .parsed_json(Some(json.clone()))
        .language(Some("json".to_string()))
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
    let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());

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
            }).unwrap(),
        );
    }
}

async fn handle_stream_event(ctx: &StreamContext, json: &serde_json::Value, line: &str) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);
    let content = serde_json::to_string_pretty(json).unwrap_or_else(|_| line.to_string());

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("stream_event")
        .content(content)
        .parsed_json(Some(json.clone()))
        .language(Some("json".to_string()))
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());
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

    let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());
}

async fn handle_plain_text(ctx: &StreamContext, line: &str) {
    let byte_size = line.len();

    update_output_bytes(&ctx.stats, byte_size).await;

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("text")
        .content(line)
        .with_size_from_content()
        .build();

    let _ = ctx.app_handle.emit("agent:output", serde_json::to_value(output_event).unwrap());
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
        }).unwrap(),
    );
}

/// Spawn the stderr stream handler task
pub fn spawn_stderr_handler(
    stderr: tokio::process::ChildStderr,
    agent_id: String,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
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
        }
    });
}
