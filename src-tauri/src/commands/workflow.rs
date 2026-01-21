// Workflow/Orchestrator related Tauri commands

use crate::orchestrator::Workflow;
use crate::thread_controller::{ThreadConfig, ThreadStats};
use crate::AppState;

#[tauri::command]
pub async fn create_workflow_from_request(
    request: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let orchestrator = state.orchestrator.lock().await;
    orchestrator.create_workflow_from_request(&request).await
}

#[tauri::command]
pub async fn execute_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let orchestrator = state.orchestrator.lock().await;
    orchestrator.execute_workflow(&workflow_id).await
}

#[tauri::command]
pub async fn get_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Workflow, String> {
    let orchestrator = state.orchestrator.lock().await;
    orchestrator
        .get_workflow(&workflow_id)
        .await
        .ok_or_else(|| "Workflow not found".to_string())
}

#[tauri::command]
pub async fn list_workflows(state: tauri::State<'_, AppState>) -> Result<Vec<Workflow>, String> {
    let orchestrator = state.orchestrator.lock().await;
    Ok(orchestrator.list_workflows().await)
}

#[tauri::command]
pub async fn get_thread_config(state: tauri::State<'_, AppState>) -> Result<ThreadConfig, String> {
    let controller = state.thread_controller.lock().await;
    Ok(controller.get_config().await)
}

#[tauri::command]
pub async fn update_thread_config(
    config: ThreadConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let controller = state.thread_controller.lock().await;
    controller.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn get_thread_stats(state: tauri::State<'_, AppState>) -> Result<ThreadStats, String> {
    let controller = state.thread_controller.lock().await;
    Ok(controller.get_stats().await)
}

#[tauri::command]
pub async fn emergency_shutdown_threads(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let controller = state.thread_controller.lock().await;
    controller.emergency_shutdown().await
}
