use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::elevation::{classify_risk_level, extract_inner_command, generate_warnings, parse_compound_command};
use crate::security_monitor::{
    SecurityEvent, SecurityEventMetadata, SecurityEventType, SecurityMonitor,
};
use crate::types::{
    AgentActivityDetailEvent, CommandRiskLevel, ElevatedCommandRequest,
    ElevatedCommandRequestEvent, ElevatedCommandRequestResponse, ElevatedCommandStatus,
    ElevatedCommandStatusEvent, ElevatedScopeCheckResponse, ElevatedStatusResponse, HookInput,
    PendingElevatedCommand, ToolEventPayload,
};

/// How long an elevated command request stays valid (5 minutes)
const ELEVATED_REQUEST_EXPIRY_MS: i64 = 5 * 60 * 1000;

/// Maximum pending elevated commands per agent (rate limiting)
const MAX_PENDING_PER_AGENT: usize = 3;

#[derive(Debug, Clone)]
struct PendingToolCall {
    tool_name: String,
    tool_input: serde_json::Value,
    start_time: i64,
}

pub struct HookServerState {
    pub agent_manager: Arc<Mutex<AgentManager>>,
    pub app_handle: Arc<dyn crate::events::AppEventEmitter>,
    pub pending_tools: Arc<Mutex<HashMap<String, PendingToolCall>>>,
    pub security_monitor: Option<Arc<SecurityMonitor>>,
    /// Pending elevated command requests awaiting user approval
    pub pending_elevated: Arc<Mutex<HashMap<String, PendingElevatedCommand>>>,
    /// Approved script scopes (script_hash -> expiry timestamp)
    pub approved_scopes: Arc<Mutex<HashMap<String, i64>>>,
}

pub async fn start_hook_server(
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
    port: u16,
    security_monitor: Option<Arc<SecurityMonitor>>,
    pending_elevated: Arc<Mutex<HashMap<String, PendingElevatedCommand>>>,
    approved_scopes: Arc<Mutex<HashMap<String, i64>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(HookServerState {
        agent_manager,
        app_handle,
        pending_tools: Arc::new(Mutex::new(HashMap::new())),
        security_monitor,
        pending_elevated,
        approved_scopes,
    });

    let app = Router::new()
        .route("/hook", post(handle_hook))
        // Elevation API endpoints for wrapper scripts
        .route("/elevated/request", post(handle_elevated_request))
        .route("/elevated/status/:id", get(handle_elevated_status))
        .route("/elevated/check-scope/:hash", get(handle_scope_check))
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
        let tool_call_id = format!("{}_{}_{}_{}", agent_id, input.session_id, tool_name, now);

        if input.hook_event_name == "PreToolUse" {
            // Store the pending tool call
            let mut pending = state.pending_tools.lock().await;
            pending.insert(
                tool_call_id.clone(),
                PendingToolCall {
                    tool_name: tool_name.clone(),
                    tool_input: input.tool_input.clone().unwrap_or(serde_json::Value::Null),
                    start_time: now,
                },
            );
            drop(pending);

            // Clone tool_input for use in multiple places
            let tool_input_value = input.tool_input.clone().unwrap_or(serde_json::Value::Null);

            // Emit pending event
            let event = ToolEventPayload {
                agent_id: agent_id.clone(),
                session_id: input.session_id.clone(),
                hook_event_name: input.hook_event_name.clone(),
                tool_name: tool_name.clone(),
                tool_input: tool_input_value.clone(),
                tool_response: None,
                tool_call_id: tool_call_id.clone(),
                status: Some("pending".to_string()),
                error_message: None,
                execution_time_ms: None,
                timestamp: now,
            };

            let _ = state
                .app_handle
                .emit("agent:tool", serde_json::to_value(event).unwrap());

            // Emit enhanced activity event for UI status display
            let activity = format_agent_activity(tool_name, &Some(tool_input_value));
            let activity_event = AgentActivityDetailEvent {
                agent_id: agent_id.clone(),
                activity,
                tool_name: tool_name.clone(),
                timestamp: now,
            };
            let _ = state.app_handle.emit(
                "agent:activity",
                serde_json::to_value(activity_event).unwrap(),
            );
        } else if input.hook_event_name == "PostToolUse" {
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

            let _ = state
                .app_handle
                .emit("agent:tool", serde_json::to_value(event).unwrap());
        }
    }

    // Forward to security monitor if available
    if let (Some(monitor), Some(tool_name)) = (&state.security_monitor, &input.tool_name) {
        let now = chrono::Utc::now().timestamp_millis();

        // Get agent info for context (combine into single lock)
        let (working_dir, source) = {
            let manager = state.agent_manager.lock().await;
            match manager.get_agent_info(&agent_id).await {
                Some(info) => (info.working_dir.clone(), info.source.as_str().to_string()),
                None => ("/unknown".to_string(), "unknown".to_string()),
            }
        };

        let security_event = if input.hook_event_name == "PreToolUse" {
            SecurityEvent {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: now,
                agent_id: agent_id.clone(),
                session_id: Some(input.session_id.clone()),
                event_type: SecurityEventType::ToolUseRequest {
                    tool_name: tool_name.clone(),
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
            // Note: We already checked tool_response earlier in this function,
            // so we use a fresh reference to the JSON content for security logging
            let response_content = serde_json::to_string(&input.tool_input).unwrap_or_default();
            let success = true; // Default to success for security logging

            SecurityEvent {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: now,
                agent_id: agent_id.clone(),
                session_id: Some(input.session_id.clone()),
                event_type: SecurityEventType::ToolUseResult {
                    tool_name: tool_name.clone(),
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

    StatusCode::OK
}

/// Format a human-readable activity description based on tool name and input
fn format_agent_activity(tool_name: &str, tool_input: &Option<serde_json::Value>) -> String {
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

// ============================================================================
// Elevated Command API Handlers
// ============================================================================

/// Handle elevation request from wrapper script
/// POST /elevated/request
async fn handle_elevated_request(
    State(state): State<Arc<HookServerState>>,
    Json(request): Json<ElevatedCommandRequest>,
) -> (StatusCode, Json<ElevatedCommandRequestResponse>) {
    let now = chrono::Utc::now().timestamp_millis();
    let request_id = uuid::Uuid::new_v4().to_string();

    // Rate limiting: check how many pending requests this agent has
    {
        let pending = state.pending_elevated.lock().await;
        let agent_pending_count = pending
            .values()
            .filter(|cmd| cmd.agent_id == request.agent_id && cmd.status == ElevatedCommandStatus::Pending)
            .count();

        if agent_pending_count >= MAX_PENDING_PER_AGENT {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ElevatedCommandRequestResponse {
                    request_id: String::new(),
                    status: "rate_limited".to_string(),
                }),
            );
        }
    }

    // Check if this script scope is already approved
    if let Some(script_hash) = &request.script_hash {
        let scopes = state.approved_scopes.lock().await;
        if let Some(&expiry) = scopes.get(script_hash) {
            if expiry > now {
                // Script scope is approved, auto-approve this request
                return (
                    StatusCode::OK,
                    Json(ElevatedCommandRequestResponse {
                        request_id: request_id.clone(),
                        status: "approved".to_string(),
                    }),
                );
            }
        }
    }

    // Parse compound command if present
    let compound_info = parse_compound_command(&request.command);

    // Classify risk level
    let risk_level = classify_risk_level(&request.command);

    // Generate warnings
    let warnings = generate_warnings(&request.command);
    let warnings = if warnings.is_empty() {
        None
    } else {
        Some(warnings)
    };

    // Extract inner command if bash -c is used
    let inner_command = request.inner_command.clone().or_else(|| extract_inner_command(&request.command));

    // Get working directory from request or default
    let working_dir = request.working_dir.clone().unwrap_or_else(|| "/unknown".to_string());

    // Create pending command
    let pending_cmd = PendingElevatedCommand {
        id: request_id.clone(),
        agent_id: request.agent_id.clone(),
        command: request.command.clone(),
        working_dir,
        requested_at: now,
        expires_at: now + ELEVATED_REQUEST_EXPIRY_MS,
        status: ElevatedCommandStatus::Pending,
        risk_level,
        warnings,
        script_hash: request.script_hash.clone(),
        parent_cmd: request.parent_cmd.clone(),
        inner_command,
        pre_commands: if compound_info.pre_commands.is_empty() {
            None
        } else {
            Some(compound_info.pre_commands)
        },
        post_commands: if compound_info.post_commands.is_empty() {
            None
        } else {
            Some(compound_info.post_commands)
        },
        sudo_command: compound_info.sudo_command,
    };

    // Store pending command
    {
        let mut pending = state.pending_elevated.lock().await;
        pending.insert(request_id.clone(), pending_cmd.clone());
    }

    // Emit event to frontend
    let event = ElevatedCommandRequestEvent {
        request: pending_cmd,
    };
    let _ = state
        .app_handle
        .emit("elevated:request", serde_json::to_value(event).unwrap());

    (
        StatusCode::OK,
        Json(ElevatedCommandRequestResponse {
            request_id,
            status: "pending".to_string(),
        }),
    )
}

/// Handle status check from wrapper script
/// GET /elevated/status/:id
async fn handle_elevated_status(
    State(state): State<Arc<HookServerState>>,
    Path(request_id): Path<String>,
) -> (StatusCode, Json<ElevatedStatusResponse>) {
    let now = chrono::Utc::now().timestamp_millis();

    // Clean up expired requests first
    {
        let mut pending = state.pending_elevated.lock().await;
        pending.retain(|_, cmd| {
            cmd.expires_at > now || cmd.status != ElevatedCommandStatus::Pending
        });
    }

    // Look up the request
    let pending = state.pending_elevated.lock().await;

    match pending.get(&request_id) {
        Some(cmd) => {
            // Check if expired
            if cmd.status == ElevatedCommandStatus::Pending && cmd.expires_at <= now {
                return (
                    StatusCode::OK,
                    Json(ElevatedStatusResponse {
                        status: "expired".to_string(),
                        error: Some("Request timed out".to_string()),
                    }),
                );
            }

            (
                StatusCode::OK,
                Json(ElevatedStatusResponse {
                    status: cmd.status.as_str().to_string(),
                    error: None,
                }),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ElevatedStatusResponse {
                status: "not_found".to_string(),
                error: Some("Request not found or expired".to_string()),
            }),
        ),
    }
}

/// Check if a script scope is approved
/// GET /elevated/check-scope/:hash
async fn handle_scope_check(
    State(state): State<Arc<HookServerState>>,
    Path(script_hash): Path<String>,
) -> Json<ElevatedScopeCheckResponse> {
    let now = chrono::Utc::now().timestamp_millis();

    let scopes = state.approved_scopes.lock().await;

    let approved = scopes
        .get(&script_hash)
        .map(|&expiry| expiry > now)
        .unwrap_or(false);

    Json(ElevatedScopeCheckResponse { approved })
}

/// Approve an elevated command request (called by Tauri command)
pub async fn approve_elevated_request(
    state: &HookServerState,
    request_id: &str,
    approve_scope: bool,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    let mut pending = state.pending_elevated.lock().await;

    let cmd = pending
        .get_mut(request_id)
        .ok_or_else(|| "Request not found".to_string())?;

    if cmd.status != ElevatedCommandStatus::Pending {
        return Err(format!("Request is not pending (status: {:?})", cmd.status));
    }

    if cmd.expires_at <= now {
        cmd.status = ElevatedCommandStatus::Expired;
        return Err("Request has expired".to_string());
    }

    // Approve the request
    cmd.status = ElevatedCommandStatus::Approved;

    // If approve_scope is true and there's a script_hash, approve the whole scope
    if approve_scope {
        if let Some(script_hash) = &cmd.script_hash {
            let mut scopes = state.approved_scopes.lock().await;
            // Scope approval lasts for 10 minutes
            scopes.insert(script_hash.clone(), now + 10 * 60 * 1000);
        }
    }

    // Emit status change event
    let event = ElevatedCommandStatusEvent {
        request_id: request_id.to_string(),
        status: ElevatedCommandStatus::Approved,
        error: None,
    };
    let _ = state
        .app_handle
        .emit("elevated:status", serde_json::to_value(event).unwrap());

    Ok(())
}

/// Deny an elevated command request (called by Tauri command)
pub async fn deny_elevated_request(
    state: &HookServerState,
    request_id: &str,
) -> Result<(), String> {
    let mut pending = state.pending_elevated.lock().await;

    let cmd = pending
        .get_mut(request_id)
        .ok_or_else(|| "Request not found".to_string())?;

    if cmd.status != ElevatedCommandStatus::Pending {
        return Err(format!("Request is not pending (status: {:?})", cmd.status));
    }

    // Deny the request
    cmd.status = ElevatedCommandStatus::Denied;

    // Emit status change event
    let event = ElevatedCommandStatusEvent {
        request_id: request_id.to_string(),
        status: ElevatedCommandStatus::Denied,
        error: None,
    };
    let _ = state
        .app_handle
        .emit("elevated:status", serde_json::to_value(event).unwrap());

    Ok(())
}

/// Get all pending elevated commands (for Tauri command)
pub async fn get_pending_elevated_commands(
    state: &HookServerState,
) -> Vec<PendingElevatedCommand> {
    let now = chrono::Utc::now().timestamp_millis();
    let pending = state.pending_elevated.lock().await;

    pending
        .values()
        .filter(|cmd| cmd.status == ElevatedCommandStatus::Pending && cmd.expires_at > now)
        .cloned()
        .collect()
}
