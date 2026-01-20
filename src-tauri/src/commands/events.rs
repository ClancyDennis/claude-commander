// Orchestrator event persistence Tauri commands
//
// Commands for persisting and querying orchestrator events (tool calls,
// state changes, decisions) and agent outputs for the hybrid persistence model.

use crate::agent_runs_db::{
    AgentOutputRecord, EventQueryFilters, OrchestratorDecisionRecord,
    OrchestratorStateChangeRecord, OrchestratorToolCallRecord, PipelineHistoryBundle,
};
use crate::AppState;

// ============================================================================
// Persistence Commands (fire-and-forget from frontend)
// ============================================================================

#[tauri::command]
pub async fn persist_tool_call(
    tool_call: OrchestratorToolCallRecord,
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
    state
        .agent_runs_db
        .insert_tool_call(&tool_call)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn persist_state_change(
    state_change: OrchestratorStateChangeRecord,
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
    state
        .agent_runs_db
        .insert_state_change(&state_change)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn persist_decision(
    decision: OrchestratorDecisionRecord,
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
    state
        .agent_runs_db
        .insert_decision(&decision)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn persist_agent_output(
    output: AgentOutputRecord,
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
    state
        .agent_runs_db
        .insert_agent_output(&output)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Query Commands (for loading history on page reload)
// ============================================================================

#[tauri::command]
pub async fn get_orchestrator_tool_calls(
    pipeline_id: Option<String>,
    agent_id: Option<String>,
    since_timestamp: Option<i64>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<OrchestratorToolCallRecord>, String> {
    let filters = EventQueryFilters {
        pipeline_id,
        agent_id,
        since_timestamp,
        until_timestamp: None,
        limit: limit.or(Some(200)),
        offset,
    };

    state
        .agent_runs_db
        .query_tool_calls(filters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_orchestrator_state_changes(
    pipeline_id: Option<String>,
    since_timestamp: Option<i64>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<OrchestratorStateChangeRecord>, String> {
    let filters = EventQueryFilters {
        pipeline_id,
        agent_id: None,
        since_timestamp,
        until_timestamp: None,
        limit: limit.or(Some(100)),
        offset,
    };

    state
        .agent_runs_db
        .query_state_changes(filters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_orchestrator_decisions(
    pipeline_id: Option<String>,
    since_timestamp: Option<i64>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<OrchestratorDecisionRecord>, String> {
    let filters = EventQueryFilters {
        pipeline_id,
        agent_id: None,
        since_timestamp,
        until_timestamp: None,
        limit: limit.or(Some(50)),
        offset,
    };

    state
        .agent_runs_db
        .query_decisions(filters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_output_history(
    agent_id: String,
    pipeline_id: Option<String>,
    since_timestamp: Option<i64>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentOutputRecord>, String> {
    let filters = EventQueryFilters {
        pipeline_id,
        agent_id: Some(agent_id),
        since_timestamp,
        until_timestamp: None,
        limit: limit.or(Some(500)),
        offset,
    };

    state
        .agent_runs_db
        .query_agent_outputs(filters)
        .await
        .map_err(|e| e.to_string())
}

/// Load all historical data for a pipeline in one call (efficient for restoring UI)
#[tauri::command]
pub async fn get_pipeline_history(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<PipelineHistoryBundle, String> {
    state
        .agent_runs_db
        .get_pipeline_history(&pipeline_id)
        .await
        .map_err(|e| e.to_string())
}

/// Clear all events for a pipeline (useful for restarting)
#[tauri::command]
pub async fn clear_pipeline_events(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .agent_runs_db
        .clear_pipeline_events(&pipeline_id)
        .await
        .map_err(|e| e.to_string())
}
