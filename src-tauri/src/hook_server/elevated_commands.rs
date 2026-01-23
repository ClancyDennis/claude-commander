//! Elevated command handling for the hook server
//!
//! This module handles elevation requests from wrapper scripts,
//! allowing users to approve or deny commands that require elevated privileges.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::elevation::{
    classify_risk_level, extract_inner_command, generate_warnings, parse_compound_command,
};
use crate::types::{
    ElevatedCommandRequest, ElevatedCommandRequestEvent, ElevatedCommandRequestResponse,
    ElevatedCommandStatus, ElevatedCommandStatusEvent, ElevatedScopeCheckResponse,
    ElevatedStatusResponse, PendingElevatedCommand,
};

use super::HookServerState;

/// How long an elevated command request stays valid (5 minutes)
pub(crate) const ELEVATED_REQUEST_EXPIRY_MS: i64 = 5 * 60 * 1000;

/// Maximum pending elevated commands per agent (rate limiting)
pub(crate) const MAX_PENDING_PER_AGENT: usize = 3;

/// Handle elevation request from wrapper script
/// POST /elevated/request
pub(crate) async fn handle_elevated_request(
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
            .filter(|cmd| {
                cmd.agent_id == request.agent_id && cmd.status == ElevatedCommandStatus::Pending
            })
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
    let inner_command = request
        .inner_command
        .clone()
        .or_else(|| extract_inner_command(&request.command));

    // Get working directory from request or default
    let working_dir = request
        .working_dir
        .clone()
        .unwrap_or_else(|| "/unknown".to_string());

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
pub(crate) async fn handle_elevated_status(
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
pub(crate) async fn handle_scope_check(
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
pub async fn deny_elevated_request(state: &HookServerState, request_id: &str) -> Result<(), String> {
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
pub async fn get_pending_elevated_commands(state: &HookServerState) -> Vec<PendingElevatedCommand> {
    let now = chrono::Utc::now().timestamp_millis();
    let pending = state.pending_elevated.lock().await;

    pending
        .values()
        .filter(|cmd| cmd.status == ElevatedCommandStatus::Pending && cmd.expires_at > now)
        .cloned()
        .collect()
}
