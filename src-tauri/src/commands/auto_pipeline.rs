// Auto-pipeline related Tauri commands

use crate::auto_pipeline::AutoPipeline;
use crate::AppState;
use std::sync::Arc;

#[tauri::command]
pub async fn create_auto_pipeline(
    user_request: String,
    working_dir: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.auto_pipeline_manager.lock().await;
    manager.create_pipeline(user_request, working_dir).await
}

#[tauri::command]
pub async fn start_auto_pipeline(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let manager = state.auto_pipeline_manager.clone();
    let agent_manager = state.agent_manager.clone();

    // Clone for spawned task
    let pipeline_id_clone = pipeline_id.clone();

    // Spawn async execution
    tokio::spawn(async move {
        let mgr = manager.lock().await;
        let _ = mgr.execute_pipeline(pipeline_id_clone, agent_manager, Arc::new(app_handle)).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn get_auto_pipeline(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AutoPipeline, String> {
    let manager = state.auto_pipeline_manager.lock().await;
    manager.get_pipeline(&pipeline_id).await
        .ok_or_else(|| "Pipeline not found".to_string())
}
