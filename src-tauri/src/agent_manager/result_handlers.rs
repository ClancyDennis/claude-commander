// Result and miscellaneous event handlers
//
// This module handles result messages (task completion), stream events,
// unknown message types, plain text output, and process end handling.

use crate::agent_runs_db::RunStatus;
use crate::types::{AgentInputRequiredEvent, AgentStatsEvent, AgentStatus, AgentStatusEvent};
use crate::utils::time::now_millis;

use super::event_handlers::StreamContext;
use super::output_builder::{extract_common_fields, OutputEventBuilder};
use super::statistics::{update_from_result, update_output_bytes};
use super::stream_parser::{persist_output, store_in_buffer};

/// Handle result messages (task completion)
pub(crate) async fn handle_result_message(ctx: &StreamContext, json: &serde_json::Value) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

    // Check if this is a successful completion
    let is_success = subtype.as_deref() == Some("success");

    // Extract meaningful content from result message instead of full JSON
    let content = extract_result_content(json, &subtype);

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

    store_in_buffer(output_event.clone(), ctx.output_buffer.clone()).await;
    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());

    // Persist to database
    persist_output(
        &ctx.runs_db,
        &ctx.agent_id,
        &ctx.pipeline_id,
        "result",
        &content,
    )
    .await;

    // Handle successful completion - update state
    if is_success {
        handle_success_completion(ctx).await;
    }
}

/// Extract human-readable content from result JSON
fn extract_result_content(json: &serde_json::Value, subtype: &Option<String>) -> String {
    if let Some(result) = json.get("result") {
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
    }
}

/// Handle successful task completion - update state and emit events
async fn handle_success_completion(ctx: &StreamContext) {
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

/// Handle stream events
pub(crate) async fn handle_stream_event(ctx: &StreamContext, json: &serde_json::Value) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

    // Extract meaningful content from stream event instead of full JSON
    let event_type = json
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("stream");
    let content = extract_stream_content(json, event_type);

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

/// Extract content from stream event JSON
fn extract_stream_content(json: &serde_json::Value, event_type: &str) -> String {
    if let Some(data) = json.get("data") {
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
    }
}

/// Handle unknown message types
pub(crate) async fn handle_unknown_message(
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

/// Handle plain text (non-JSON) output
pub(crate) async fn handle_plain_text(ctx: &StreamContext, line: &str) {
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
    persist_output(&ctx.runs_db, &ctx.agent_id, &ctx.pipeline_id, "text", line).await;
}

/// Handle process end (completion or crash)
pub(crate) async fn handle_process_end(ctx: &StreamContext) {
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
    update_database_on_end(ctx, was_stopped, final_stats).await;

    // Emit status event
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

/// Update database when process ends
async fn update_database_on_end(
    ctx: &StreamContext,
    was_stopped: bool,
    final_stats: Option<std::sync::Arc<tokio::sync::Mutex<crate::types::AgentStatistics>>>,
) {
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
}
