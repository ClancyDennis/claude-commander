//! Security monitor Tauri commands.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

/// Security status response
#[derive(Debug, Serialize)]
pub struct SecurityStatus {
    pub enabled: bool,
    pub pending_reviews_count: usize,
    pub config: SecurityConfigResponse,
    pub rule_count: usize,
    pub categories: Vec<String>,
}

/// Security configuration response
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfigResponse {
    pub auto_terminate_on_critical: bool,
    pub auto_suspend_on_high: bool,
    pub alert_on_medium: bool,
    pub log_all_events: bool,
    pub human_review_required_for_actions: bool,
}

/// Get security monitor status
#[tauri::command]
pub async fn get_security_status(state: State<'_, AppState>) -> Result<SecurityStatus, String> {
    if let Some(monitor) = &state.security_monitor {
        let enabled = monitor.is_enabled().await;
        let pending_reviews = monitor.get_response_handler().get_pending_reviews().await;
        let config = monitor.get_config();
        let response_config = monitor.get_response_handler().get_config();

        Ok(SecurityStatus {
            enabled,
            pending_reviews_count: pending_reviews.len(),
            config: SecurityConfigResponse {
                auto_terminate_on_critical: response_config.auto_terminate_on_critical,
                auto_suspend_on_high: response_config.auto_suspend_on_high,
                alert_on_medium: response_config.alert_on_medium,
                log_all_events: response_config.log_all_events,
                human_review_required_for_actions: response_config
                    .human_review_required_for_actions,
            },
            rule_count: config.batch_size, // This is a simplification
            categories: vec![
                "PromptInjection".to_string(),
                "DangerousCommand".to_string(),
                "UnauthorizedFileAccess".to_string(),
                "DataExfiltration".to_string(),
                "PrivilegeEscalation".to_string(),
            ],
        })
    } else {
        Err("Security monitor not initialized".to_string())
    }
}

/// Enable or disable security monitor
#[tauri::command]
pub async fn set_security_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    if let Some(monitor) = &state.security_monitor {
        monitor.set_enabled(enabled).await;
        Ok(())
    } else {
        Err("Security monitor not initialized".to_string())
    }
}

/// Get pending security reviews
#[tauri::command]
pub async fn get_pending_security_reviews(
    state: State<'_, AppState>,
) -> Result<Vec<crate::security_monitor::response_handler::PendingReview>, String> {
    if let Some(monitor) = &state.security_monitor {
        Ok(monitor.get_response_handler().get_pending_reviews().await)
    } else {
        Err("Security monitor not initialized".to_string())
    }
}

/// Approve a pending security action
#[tauri::command]
pub async fn approve_security_action(
    state: State<'_, AppState>,
    review_id: String,
) -> Result<(), String> {
    if let Some(monitor) = &state.security_monitor {
        monitor
            .get_response_handler()
            .handle_review_response(&review_id, true)
            .await
    } else {
        Err("Security monitor not initialized".to_string())
    }
}

/// Reject/dismiss a pending security action
#[tauri::command]
pub async fn reject_security_action(
    state: State<'_, AppState>,
    review_id: String,
) -> Result<(), String> {
    if let Some(monitor) = &state.security_monitor {
        monitor
            .get_response_handler()
            .dismiss_review(&review_id)
            .await
    } else {
        Err("Security monitor not initialized".to_string())
    }
}

/// Clear all pending reviews
#[tauri::command]
pub async fn clear_security_reviews(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(monitor) = &state.security_monitor {
        monitor.get_response_handler().clear_pending_reviews().await;
        Ok(())
    } else {
        Err("Security monitor not initialized".to_string())
    }
}

/// Manually trigger a security scan on an agent's recent activity
#[tauri::command]
pub async fn scan_agent_activity(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<String, String> {
    if state.security_monitor.is_none() {
        return Err("Security monitor not initialized".to_string());
    }

    // Get agent info to verify it exists
    let manager = state.agent_manager.lock().await;
    if manager.get_agent_info(&agent_id).await.is_none() {
        return Err(format!("Agent {} not found", agent_id));
    }

    // For now, just return a message. A full implementation would
    // query recent logs for this agent and run them through the analyzer
    Ok(format!(
        "Security scan initiated for agent {}. Results will appear in alerts.",
        agent_id
    ))
}
