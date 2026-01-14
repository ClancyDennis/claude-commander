use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::types::{HookInput, ToolEventPayload};

#[derive(Debug, Clone)]
struct PendingToolCall {
    tool_name: String,
    tool_input: serde_json::Value,
    start_time: i64,
}

pub struct HookServerState {
    pub agent_manager: Arc<Mutex<AgentManager>>,
    pub app_handle: tauri::AppHandle,
    pub pending_tools: Arc<Mutex<HashMap<String, PendingToolCall>>>,
}

pub async fn start_hook_server(
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: tauri::AppHandle,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(HookServerState {
        agent_manager,
        app_handle,
        pending_tools: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = Router::new()
        .route("/hook", post(handle_hook))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Hook server listening on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_hook(
    State(state): State<Arc<HookServerState>>,
    Json(input): Json<HookInput>,
) -> StatusCode {
    // Find agent by session_id
    let agent_manager = state.agent_manager.lock().await;
    let agent_id = match agent_manager.get_agent_by_session(&input.session_id).await {
        Some(id) => id,
        None => {
            // Session not mapped yet, just acknowledge
            return StatusCode::OK;
        }
    };
    drop(agent_manager); // Release lock early

    // Only emit tool events for PreToolUse and PostToolUse
    if let Some(tool_name) = &input.tool_name {
        let now = chrono::Utc::now().timestamp_millis();

        // Generate a unique ID for this tool call (agent_id + session_id + tool_name + timestamp)
        let tool_call_id = format!("{}_{}_{}_{}",
            agent_id,
            input.session_id,
            tool_name,
            now
        );

        if input.hook_event_name == "PreToolUse" {
            // Store the pending tool call
            let mut pending = state.pending_tools.lock().await;
            pending.insert(tool_call_id.clone(), PendingToolCall {
                tool_name: tool_name.clone(),
                tool_input: input.tool_input.clone().unwrap_or(serde_json::Value::Null),
                start_time: now,
            });
            drop(pending);

            // Emit pending event
            let event = ToolEventPayload {
                agent_id: agent_id.clone(),
                session_id: input.session_id.clone(),
                hook_event_name: input.hook_event_name.clone(),
                tool_name: tool_name.clone(),
                tool_input: input.tool_input.unwrap_or(serde_json::Value::Null),
                tool_response: None,
                tool_call_id: tool_call_id.clone(),
                status: Some("pending".to_string()),
                error_message: None,
                execution_time_ms: None,
                timestamp: now,
            };

            let _ = state.app_handle.emit("agent:tool", event);
        } else if input.hook_event_name == "PostToolUse" {
            // Try to find the matching PreToolUse
            let mut pending = state.pending_tools.lock().await;

            // Find the most recent matching pending call for this tool
            let matching_key = pending.iter()
                .filter(|(k, v)| k.starts_with(&format!("{}_{}", agent_id, input.session_id))
                    && v.tool_name == *tool_name)
                .map(|(k, _)| k.clone())
                .max(); // Get the most recent one

            let (execution_time_ms, start_time, stored_input) = if let Some(key) = &matching_key {
                if let Some(pending_call) = pending.remove(key) {
                    let exec_time = (now - pending_call.start_time) as u64;
                    (Some(exec_time), pending_call.start_time, pending_call.tool_input)
                } else {
                    (None, now, input.tool_input.clone().unwrap_or(serde_json::Value::Null))
                }
            } else {
                (None, now, input.tool_input.clone().unwrap_or(serde_json::Value::Null))
            };
            drop(pending);

            // Determine status from response
            let (status, error_message) = if let Some(response) = &input.tool_response {
                // Check if response indicates an error
                if response.get("error").is_some() {
                    (
                        "failed".to_string(),
                        response.get("error")
                            .and_then(|e| e.as_str())
                            .map(|s| s.to_string())
                    )
                } else {
                    ("success".to_string(), None)
                }
            } else {
                ("success".to_string(), None)
            };

            // Emit completed event
            let event = ToolEventPayload {
                agent_id: agent_id.clone(),
                session_id: input.session_id.clone(),
                hook_event_name: input.hook_event_name.clone(),
                tool_name: tool_name.clone(),
                tool_input: stored_input,
                tool_response: input.tool_response,
                tool_call_id: matching_key.unwrap_or_else(|| tool_call_id.clone()),
                status: Some(status),
                error_message,
                execution_time_ms,
                timestamp: start_time,
            };

            let _ = state.app_handle.emit("agent:tool", event);
        }
    }

    StatusCode::OK
}
