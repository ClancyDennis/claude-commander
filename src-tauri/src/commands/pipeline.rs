// Pipeline-related Tauri commands

use crate::pipeline_manager::{Pipeline, PipelineConfig};
use crate::verification::{VerificationConfig, VerificationResult};
use crate::AppState;

#[tauri::command]
pub async fn create_pipeline(
    user_request: String,
    config: Option<PipelineConfig>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.pipeline_manager.lock().await;
    manager.create_pipeline(user_request, config).await
}

#[tauri::command]
pub async fn start_pipeline(
    pipeline_id: String,
    user_request: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.pipeline_manager.lock().await;
    manager.start_pipeline(&pipeline_id, user_request).await
}

#[tauri::command]
pub async fn get_pipeline(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Pipeline, String> {
    let manager = state.pipeline_manager.lock().await;
    manager.get_pipeline(&pipeline_id).await
        .ok_or_else(|| "Pipeline not found".to_string())
}

#[tauri::command]
pub async fn list_pipelines(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Pipeline>, String> {
    let manager = state.pipeline_manager.lock().await;
    Ok(manager.list_pipelines().await)
}

#[tauri::command]
pub async fn approve_pipeline_checkpoint(
    pipeline_id: String,
    phase_index: usize,
    approved: bool,
    comment: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.pipeline_manager.lock().await;
    manager.approve_checkpoint(&pipeline_id, phase_index, approved, comment).await
}

#[tauri::command]
pub async fn get_pipeline_config(
    state: tauri::State<'_, AppState>,
) -> Result<PipelineConfig, String> {
    let manager = state.pipeline_manager.lock().await;
    Ok(manager.get_config().await)
}

#[tauri::command]
pub async fn update_pipeline_config(
    config: PipelineConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.pipeline_manager.lock().await;
    manager.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn run_best_of_n_verification(
    prompt: String,
    config: VerificationConfig,
    state: tauri::State<'_, AppState>,
) -> Result<VerificationResult, String> {
    let engine = state.verification_engine.lock().await;
    engine.best_of_n(&prompt, config).await
}
