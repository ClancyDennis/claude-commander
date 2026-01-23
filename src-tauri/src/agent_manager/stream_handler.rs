// Stream handler for parsing Claude CLI stdout/stderr
//
// This module manages the main stream reading loop and coordinates
// message dispatching to appropriate handlers.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::agent_runs_db::{AgentOutputRecord, AgentRunsDB};
use crate::types::{AgentOutputEvent, AgentStatistics};
use crate::utils::time::now_millis;

use super::event_handlers::{
    handle_assistant_message, handle_plain_text, handle_process_end, handle_result_message,
    handle_stream_event, handle_system_message, handle_unknown_message, handle_user_message,
};
use super::output_builder::OutputEventBuilder;
use super::types::AgentProcess;

// Re-export StreamContext for use by mod.rs
pub use super::event_handlers::StreamContext;

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

            // Parse stream-json format and dispatch to appropriate handler
            let msg_type = json
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            dispatch_message(&ctx, &json, &line, msg_type).await;
        } else {
            // Not JSON, emit as plain text
            handle_plain_text(&ctx, &line).await;
        }
    }

    // Process ended - handle completion
    handle_process_end(&ctx).await;
}

/// Dispatch message to appropriate handler based on type
async fn dispatch_message(
    ctx: &StreamContext,
    json: &serde_json::Value,
    line: &str,
    msg_type: &str,
) {
    match msg_type {
        "system" => {
            handle_system_message(ctx, json).await;
        }
        "assistant" => {
            handle_assistant_message(ctx, json).await;
        }
        "user" => {
            handle_user_message(ctx, json).await;
        }
        "result" => {
            handle_result_message(ctx, json).await;
        }
        "stream_event" => {
            handle_stream_event(ctx, json).await;
        }
        _ => {
            handle_unknown_message(ctx, json, line, msg_type).await;
        }
    }
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

/// Create a new StreamContext for handlers
///
/// This is a convenience function for creating the context struct
/// that gets passed to all stream handlers.
pub fn create_stream_context(
    agent_id: String,
    agents: Arc<Mutex<HashMap<String, AgentProcess>>>,
    session_map: Arc<Mutex<HashMap<String, String>>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
    last_activity: Arc<Mutex<Instant>>,
    is_processing: Arc<Mutex<bool>>,
    pending_input: Arc<Mutex<bool>>,
    stats: Arc<Mutex<AgentStatistics>>,
    output_buffer: Arc<Mutex<Vec<AgentOutputEvent>>>,
    runs_db: Option<Arc<AgentRunsDB>>,
    pipeline_id: Option<String>,
) -> StreamContext {
    StreamContext {
        agent_id,
        agents,
        session_map,
        app_handle,
        last_activity,
        is_processing,
        pending_input,
        stats,
        output_buffer,
        runs_db,
        pipeline_id,
    }
}
