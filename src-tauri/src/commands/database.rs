// Database/Runs related Tauri commands

use crate::agent_runs_db::{AgentRun, RunQueryFilters, RunStats, RunStatus, DatabaseStats};
use crate::types::AgentSource;
use crate::AppState;

#[tauri::command]
pub async fn get_database_stats(
    state: tauri::State<'_, AppState>,
) -> Result<DatabaseStats, String> {
    state.agent_runs_db
        .get_database_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cost_database_stats(
    state: tauri::State<'_, AppState>,
) -> Result<DatabaseStats, String> {
    state.agent_runs_db.get_database_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_runs(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentRun>, String> {
    state.agent_runs_db
        .query_runs(RunQueryFilters::default())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_run_by_id(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<AgentRun>, String> {
    state.agent_runs_db
        .get_run(&agent_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn query_runs(
    status: Option<String>,
    working_dir: Option<String>,
    source: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentRun>, String> {
    let run_status = status.and_then(|s| match s.as_str() {
        "running" => Some(RunStatus::Running),
        "completed" => Some(RunStatus::Completed),
        "stopped" => Some(RunStatus::Stopped),
        "crashed" => Some(RunStatus::Crashed),
        "waiting_input" => Some(RunStatus::WaitingInput),
        _ => None,
    });

    let agent_source = source.and_then(|s| match s.as_str() {
        "ui" => Some(AgentSource::UI),
        "meta" => Some(AgentSource::Meta),
        "pipeline" => Some(AgentSource::Pipeline),
        "pool" => Some(AgentSource::Pool),
        "manual" => Some(AgentSource::Manual),
        _ => None,
    });

    let filters = RunQueryFilters {
        status: run_status,
        working_dir,
        source: agent_source,
        date_from: None,
        date_to: None,
        limit,
        offset,
    };

    state.agent_runs_db
        .query_runs(filters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_resumable_runs(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentRun>, String> {
    state.agent_runs_db
        .get_resumable_runs()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_run_prompts(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    state.agent_runs_db
        .get_prompts(&agent_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_run_stats(
    state: tauri::State<'_, AppState>,
) -> Result<RunStats, String> {
    state.agent_runs_db
        .get_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_old_runs(
    days_to_keep: i64,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    state.agent_runs_db
        .cleanup_old_runs(days_to_keep)
        .await
        .map_err(|e| e.to_string())
}
