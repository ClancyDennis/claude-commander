// Cost tracking related Tauri commands

use crate::agent_runs_db::{CostSummary, DateRangeCostSummary, DailyCost};
use crate::AppState;

#[tauri::command]
pub async fn get_cost_summary(
    state: tauri::State<'_, AppState>,
) -> Result<CostSummary, String> {
    state.agent_runs_db.get_cost_summary().await
}

#[tauri::command]
pub async fn get_cost_by_date_range(
    start_date: Option<String>,
    end_date: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<DateRangeCostSummary, String> {
    use chrono::DateTime;

    let start = start_date
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let end = end_date
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    state.agent_runs_db.get_date_range_summary(start, end).await
}

#[tauri::command]
pub async fn get_current_month_cost(
    state: tauri::State<'_, AppState>,
) -> Result<f64, String> {
    state.agent_runs_db.get_current_month_cost().await
}

#[tauri::command]
pub async fn get_today_cost(
    state: tauri::State<'_, AppState>,
) -> Result<f64, String> {
    state.agent_runs_db.get_today_cost().await
}

#[tauri::command]
pub async fn clear_cost_history(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.agent_runs_db.clear_cost_history().await
}

#[tauri::command]
pub async fn get_cost_by_working_dir(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<(String, f64)>, String> {
    state.agent_runs_db
        .get_cost_by_working_dir()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_daily_costs(
    days: i64,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DailyCost>, String> {
    state.agent_runs_db
        .get_daily_costs(days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_runs_current_month_cost(
    state: tauri::State<'_, AppState>,
) -> Result<f64, String> {
    state.agent_runs_db
        .get_current_month_cost()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_runs_today_cost(
    state: tauri::State<'_, AppState>,
) -> Result<f64, String> {
    state.agent_runs_db
        .get_today_cost()
        .await
        .map_err(|e| e.to_string())
}
