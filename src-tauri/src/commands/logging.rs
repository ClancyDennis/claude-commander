// Logging related Tauri commands

use crate::logger::{LogEntry, LogLevel, LogStats};
use crate::AppState;

#[tauri::command]
pub async fn query_logs(
    level: Option<String>,
    component: Option<String>,
    agent_id: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_level = level.and_then(|l| match l.as_str() {
        "debug" => Some(LogLevel::Debug),
        "info" => Some(LogLevel::Info),
        "warning" => Some(LogLevel::Warning),
        "error" => Some(LogLevel::Error),
        _ => None,
    });

    state.logger
        .query(log_level, component, agent_id, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recent_logs(
    limit: usize,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LogEntry>, String> {
    state.logger
        .recent(limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_log_stats(
    state: tauri::State<'_, AppState>,
) -> Result<LogStats, String> {
    state.logger
        .stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_old_logs(
    days_to_keep: i64,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    state.logger
        .cleanup(days_to_keep)
        .await
        .map_err(|e| e.to_string())
}
