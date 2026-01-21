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
    let manager = state.auto_pipeline_manager.as_ref()
        .ok_or_else(|| "Auto-pipeline unavailable: No API key configured. Set OPENAI_API_KEY or ANTHROPIC_API_KEY in .env".to_string())?;
    let manager = manager.lock().await;
    manager.create_pipeline(user_request, working_dir).await
}

#[tauri::command]
pub async fn start_auto_pipeline(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::Emitter;

    let manager = state.auto_pipeline_manager.as_ref()
        .ok_or_else(|| "Auto-pipeline unavailable: No API key configured. Set OPENAI_API_KEY or ANTHROPIC_API_KEY in .env".to_string())?;

    let agent_manager = state.agent_manager.clone();

    // Clone for spawned task
    let pipeline_id_clone = pipeline_id.clone();

    // Get the execution context and emit started event, then release the lock
    // This allows multiple pipelines to run concurrently
    let ctx = {
        let mgr = manager.lock().await;

        // Emit pipeline started event so the UI can switch to view it
        if let Some(pipeline) = mgr.get_pipeline(&pipeline_id).await {
            let _ = app_handle.emit("auto_pipeline:started", pipeline);
        } else {
            // Fallback if somehow pipeline is missing (shouldn't happen)
            let _ = app_handle.emit("auto_pipeline:started", serde_json::json!({
                "pipeline_id": pipeline_id,
            }));
        }

        // Get the shared context - this is Arc-wrapped so we can use it after dropping the lock
        mgr.get_ctx()
    };
    // Lock is now released - other pipelines can start

    // Spawn async execution using the context directly
    tokio::spawn(async move {
        let _ = ctx.execute_pipeline(pipeline_id_clone, agent_manager, Arc::new(app_handle)).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn get_auto_pipeline(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AutoPipeline, String> {
    let manager = state.auto_pipeline_manager.as_ref()
        .ok_or_else(|| "Auto-pipeline unavailable: No API key configured".to_string())?;
    let manager = manager.lock().await;
    manager.get_pipeline(&pipeline_id).await
        .ok_or_else(|| "Pipeline not found".to_string())
}
