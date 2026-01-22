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
    eprintln!("[get_auto_pipeline] Called for pipeline_id={}", pipeline_id);

    let manager = state.auto_pipeline_manager.as_ref()
        .ok_or_else(|| "Auto-pipeline unavailable: No API key configured".to_string())?;
    let manager = manager.lock().await;
    let mut pipeline = manager.get_pipeline(&pipeline_id).await
        .ok_or_else(|| "Pipeline not found".to_string())?;

    eprintln!("[get_auto_pipeline] Pipeline status={:?}, steps={}", pipeline.status, pipeline.steps.len());

    // Fetch ALL outputs for this pipeline from the database
    {
        use crate::agent_runs_db::EventQueryFilters;
        let runs_db = &state.agent_runs_db;

        // Query all outputs for this pipeline (not filtered by agent_id)
        let filters = EventQueryFilters {
            pipeline_id: Some(pipeline_id.clone()),
            agent_id: None, // Get ALL agents
            since_timestamp: None,
            until_timestamp: None,
            limit: None,
            offset: None,
        };

        match runs_db.query_agent_outputs(filters).await {
            Ok(db_outputs) => {
                eprintln!("[get_auto_pipeline] Found {} total outputs in database for pipeline", db_outputs.len());

                // Convert AgentOutputRecord to AgentOutputEvent
                let all_outputs: Vec<crate::types::AgentOutputEvent> = db_outputs.into_iter().map(|record| {
                    let metadata_value: Option<serde_json::Value> = record.metadata.as_ref()
                        .and_then(|m| serde_json::from_str(m).ok());

                    let metadata = metadata_value.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok());

                    crate::types::AgentOutputEvent {
                        agent_id: record.agent_id,
                        output_type: record.output_type,
                        content: record.content,
                        parsed_json: None,
                        metadata,
                        session_id: None,
                        uuid: None,
                        parent_tool_use_id: None,
                        subtype: None,
                        timestamp: Some(record.timestamp),
                    }
                }).collect();

                // Distribute outputs to steps based on agent_id
                for step in &mut pipeline.steps {
                    if let Some(ref agent_id) = step.agent_id {
                        let step_outputs: Vec<_> = all_outputs.iter()
                            .filter(|o| o.agent_id == *agent_id)
                            .cloned()
                            .collect();

                        if !step_outputs.is_empty() {
                            eprintln!("[get_auto_pipeline] Assigning {} outputs to step {:?}", step_outputs.len(), step.role);
                            if let Some(ref mut output) = step.output {
                                output.agent_outputs = step_outputs;
                            } else {
                                // Create output if it doesn't exist
                                eprintln!("[get_auto_pipeline] Creating output for step {:?} with {} agent outputs", step.role, step_outputs.len());
                                step.output = Some(crate::auto_pipeline::StepOutput {
                                    raw_text: String::new(),
                                    structured_data: None,
                                    agent_outputs: step_outputs,
                                });
                            }
                        }
                    }
                }

                // Store all outputs in the pipeline for display (includes orchestrator outputs)
                // This allows the UI to show everything that happened
                eprintln!("[get_auto_pipeline] Returning pipeline with {} total outputs", all_outputs.len());
            }
            Err(e) => {
                eprintln!("[get_auto_pipeline] ERROR querying outputs: {}", e);
            }
        }
    }

    Ok(pipeline)
}
