//! Tool tracking for Claude agent hooks
//!
//! Handles PreToolUse and PostToolUse events from Claude agents,
//! tracking tool call duration and emitting events to the frontend.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::security_monitor::{SecurityEvent, SecurityEventMetadata, SecurityEventType};
use crate::types::{AgentActivityDetailEvent, HookInput, ToolEventPayload};

use super::{AgentTodoItem, HookServerState};

/// Query parameters for the hook endpoint
#[derive(Debug, Deserialize)]
pub(crate) struct HookQueryParams {
    /// Agent ID passed directly in URL to avoid session mapping race condition
    pub agent_id: Option<String>,
}

/// Represents a pending tool call awaiting completion
#[derive(Debug, Clone)]
pub(crate) struct PendingToolCall {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub start_time: i64,
}

/// Handle incoming hook events from Claude agents
///
/// This endpoint receives PreToolUse and PostToolUse events,
/// tracking tool execution and emitting events to the frontend.
pub(crate) async fn handle_hook(
    State(state): State<Arc<HookServerState>>,
    Query(params): Query<HookQueryParams>,
    Json(input): Json<HookInput>,
) -> StatusCode {
    // Find agent by session_id, with fallback to agent_id from query params
    // This handles the race condition where hooks arrive before session is mapped
    let agent_manager = state.agent_manager.lock().await;
    let agent_id = match agent_manager.get_agent_by_session(&input.session_id).await {
        Some(id) => id,
        None => {
            // Session not mapped yet, try using agent_id from query params
            match params.agent_id {
                Some(id) => {
                    // Also register this session mapping for future lookups
                    let mut session_map = agent_manager.session_to_agent.lock().await;
                    session_map.insert(input.session_id.clone(), id.clone());
                    id
                }
                None => {
                    // No agent_id available, just acknowledge
                    return StatusCode::OK;
                }
            }
        }
    };
    drop(agent_manager); // Release lock early

    // Only emit tool events for PreToolUse and PostToolUse
    if let Some(tool_name) = &input.tool_name {
        let now = chrono::Utc::now().timestamp_millis();

        // Generate a unique ID for this tool call (agent_id + session_id + tool_name + timestamp)
        let tool_call_id = format!("{}_{}_{}_{}", agent_id, input.session_id, tool_name, now);

        if input.hook_event_name == "PreToolUse" {
            handle_pre_tool_use(&state, &agent_id, &input, tool_name, &tool_call_id, now).await;
        } else if input.hook_event_name == "PostToolUse" {
            handle_post_tool_use(&state, &agent_id, &input, tool_name, &tool_call_id, now).await;
        }
    }

    // Forward to security monitor if available
    if let (Some(monitor), Some(tool_name)) = (&state.security_monitor, &input.tool_name) {
        forward_to_security_monitor(&state, monitor, &agent_id, &input, tool_name).await;
    }

    StatusCode::OK
}

/// Handle PreToolUse event - store pending call and emit event
async fn handle_pre_tool_use(
    state: &HookServerState,
    agent_id: &str,
    input: &HookInput,
    tool_name: &str,
    tool_call_id: &str,
    now: i64,
) {
    // Store the pending tool call
    let mut pending = state.pending_tools.lock().await;
    pending.insert(
        tool_call_id.to_string(),
        PendingToolCall {
            tool_name: tool_name.to_string(),
            tool_input: input.tool_input.clone().unwrap_or(serde_json::Value::Null),
            start_time: now,
        },
    );
    drop(pending);

    // Clone tool_input for use in multiple places
    let tool_input_value = input.tool_input.clone().unwrap_or(serde_json::Value::Null);

    // If this is a TodoWrite call, capture the todo list
    if tool_name == "TodoWrite" {
        if let Some(todos_value) = tool_input_value.get("todos") {
            if let Some(todos_array) = todos_value.as_array() {
                let todo_items: Vec<AgentTodoItem> = todos_array
                    .iter()
                    .filter_map(|item| {
                        let content = item.get("content")?.as_str()?.to_string();
                        let status = item
                            .get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("pending")
                            .to_string();
                        let active_form = item
                            .get("activeForm")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string());
                        Some(AgentTodoItem {
                            content,
                            status,
                            active_form,
                        })
                    })
                    .collect();

                // Store the todo list for this agent
                let mut agent_todos = state.agent_todos.lock().await;
                agent_todos.insert(agent_id.to_string(), todo_items);
            }
        }
    }

    // Emit pending event
    let event = ToolEventPayload {
        agent_id: agent_id.to_string(),
        session_id: input.session_id.clone(),
        hook_event_name: input.hook_event_name.clone(),
        tool_name: tool_name.to_string(),
        tool_input: tool_input_value.clone(),
        tool_response: None,
        tool_call_id: tool_call_id.to_string(),
        status: Some("pending".to_string()),
        error_message: None,
        execution_time_ms: None,
        timestamp: now,
    };

    let _ = state
        .app_handle
        .emit("agent:tool", serde_json::to_value(event).unwrap());

    // Emit enhanced activity event for UI status display
    // Use "agent:activity:detail" channel to avoid collision with AgentActivityEvent
    // which has different fields (is_processing, pending_input, last_activity)
    let activity = format_agent_activity(tool_name, &Some(tool_input_value));
    let activity_event = AgentActivityDetailEvent {
        agent_id: agent_id.to_string(),
        activity,
        tool_name: tool_name.to_string(),
        timestamp: now,
    };
    let _ = state.app_handle.emit(
        "agent:activity:detail",
        serde_json::to_value(activity_event).unwrap(),
    );
}

/// Handle PostToolUse event - find matching pre-call, calculate duration, emit event
async fn handle_post_tool_use(
    state: &HookServerState,
    agent_id: &str,
    input: &HookInput,
    tool_name: &str,
    tool_call_id: &str,
    now: i64,
) {
    // Try to find the matching PreToolUse
    let mut pending = state.pending_tools.lock().await;

    // Find the most recent matching pending call for this tool
    let matching_key = pending
        .iter()
        .filter(|(k, v)| {
            k.starts_with(&format!("{}_{}", agent_id, input.session_id))
                && v.tool_name == *tool_name
        })
        .map(|(k, _)| k.clone())
        .max(); // Get the most recent one

    let (execution_time_ms, start_time, stored_input) = if let Some(key) = &matching_key {
        if let Some(pending_call) = pending.remove(key) {
            let exec_time = (now - pending_call.start_time) as u64;
            (
                Some(exec_time),
                pending_call.start_time,
                pending_call.tool_input,
            )
        } else {
            (
                None,
                now,
                input.tool_input.clone().unwrap_or(serde_json::Value::Null),
            )
        }
    } else {
        (
            None,
            now,
            input.tool_input.clone().unwrap_or(serde_json::Value::Null),
        )
    };
    drop(pending);

    // Determine status from response
    let (status, error_message) = if let Some(response) = &input.tool_response {
        // Check if response indicates an error
        if response.get("error").is_some() {
            (
                "failed".to_string(),
                response
                    .get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string()),
            )
        } else {
            ("success".to_string(), None)
        }
    } else {
        ("success".to_string(), None)
    };

    // Emit completed event
    let event = ToolEventPayload {
        agent_id: agent_id.to_string(),
        session_id: input.session_id.clone(),
        hook_event_name: input.hook_event_name.clone(),
        tool_name: tool_name.to_string(),
        tool_input: stored_input,
        tool_response: input.tool_response.clone(),
        tool_call_id: matching_key.unwrap_or_else(|| tool_call_id.to_string()),
        status: Some(status),
        error_message,
        execution_time_ms,
        timestamp: start_time,
    };

    let _ = state
        .app_handle
        .emit("agent:tool", serde_json::to_value(event).unwrap());
}

/// Forward tool events to the security monitor for analysis
async fn forward_to_security_monitor(
    state: &HookServerState,
    monitor: &crate::security_monitor::SecurityMonitor,
    agent_id: &str,
    input: &HookInput,
    tool_name: &str,
) {
    let now = chrono::Utc::now().timestamp_millis();

    // Get agent info for context (combine into single lock)
    let (working_dir, source) = {
        let manager = state.agent_manager.lock().await;
        match manager.get_agent_info(agent_id).await {
            Some(info) => (info.working_dir.clone(), info.source.as_str().to_string()),
            None => ("/unknown".to_string(), "unknown".to_string()),
        }
    };

    let security_event = if input.hook_event_name == "PreToolUse" {
        SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            agent_id: agent_id.to_string(),
            session_id: Some(input.session_id.clone()),
            event_type: SecurityEventType::ToolUseRequest {
                tool_name: tool_name.to_string(),
                tool_input: input.tool_input.clone().unwrap_or(serde_json::Value::Null),
            },
            content: serde_json::to_string(&input.tool_input).unwrap_or_default(),
            metadata: SecurityEventMetadata {
                working_dir,
                parent_tool_use_id: None,
                source,
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    } else {
        // For PostToolUse, determine success by checking for error in response
        let response_content = serde_json::to_string(&input.tool_input).unwrap_or_default();
        let success = true; // Default to success for security logging

        SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            agent_id: agent_id.to_string(),
            session_id: Some(input.session_id.clone()),
            event_type: SecurityEventType::ToolUseResult {
                tool_name: tool_name.to_string(),
                success,
            },
            content: response_content,
            metadata: SecurityEventMetadata {
                working_dir,
                parent_tool_use_id: None,
                source,
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    };

    monitor.process_event(security_event).await;
}

/// Format a human-readable activity description based on tool name and input
pub(crate) fn format_agent_activity(
    tool_name: &str,
    tool_input: &Option<serde_json::Value>,
) -> String {
    let input = tool_input.as_ref();

    match tool_name {
        "Read" => {
            let file_path = input
                .and_then(|i| i.get("file_path"))
                .and_then(|p| p.as_str())
                .unwrap_or("file");
            // Shorten long paths
            let short_path = if file_path.len() > 40 {
                format!("...{}", &file_path[file_path.len() - 37..])
            } else {
                file_path.to_string()
            };
            format!("Reading {}", short_path)
        }
        "Write" => {
            let file_path = input
                .and_then(|i| i.get("file_path"))
                .and_then(|p| p.as_str())
                .unwrap_or("file");
            let short_path = if file_path.len() > 40 {
                format!("...{}", &file_path[file_path.len() - 37..])
            } else {
                file_path.to_string()
            };
            format!("Writing {}", short_path)
        }
        "Edit" => {
            let file_path = input
                .and_then(|i| i.get("file_path"))
                .and_then(|p| p.as_str())
                .unwrap_or("file");
            let short_path = if file_path.len() > 40 {
                format!("...{}", &file_path[file_path.len() - 37..])
            } else {
                file_path.to_string()
            };
            format!("Editing {}", short_path)
        }
        "Bash" => {
            let command = input
                .and_then(|i| i.get("command"))
                .and_then(|c| c.as_str())
                .unwrap_or("command");
            // Truncate long commands
            let short_cmd = if command.len() > 35 {
                format!("{}...", &command[..32])
            } else {
                command.to_string()
            };
            format!("Running: {}", short_cmd)
        }
        "Grep" => {
            let pattern = input
                .and_then(|i| i.get("pattern"))
                .and_then(|p| p.as_str())
                .unwrap_or("pattern");
            format!("Searching for '{}'", pattern)
        }
        "Glob" => {
            let pattern = input
                .and_then(|i| i.get("pattern"))
                .and_then(|p| p.as_str())
                .unwrap_or("*");
            format!("Finding files: {}", pattern)
        }
        "Task" => {
            let description = input
                .and_then(|i| i.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or("task");
            format!("Running task: {}", description)
        }
        "WebFetch" => {
            let url = input
                .and_then(|i| i.get("url"))
                .and_then(|u| u.as_str())
                .unwrap_or("URL");
            // Extract domain from URL
            let domain = url
                .split("//")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .unwrap_or(url);
            format!("Fetching {}", domain)
        }
        "WebSearch" => {
            let query = input
                .and_then(|i| i.get("query"))
                .and_then(|q| q.as_str())
                .unwrap_or("query");
            let short_query = if query.len() > 30 {
                format!("{}...", &query[..27])
            } else {
                query.to_string()
            };
            format!("Searching: {}", short_query)
        }
        "TodoWrite" => "Updating task list".to_string(),
        "AskUserQuestion" => "Asking question".to_string(),
        _ => format!("Using {}", tool_name),
    }
}
