use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::orchestrator::{TaskOrchestrator, WorkflowStatus};
use crate::pipeline_manager::types::Pipeline;

/// Phase 2: Implementation - Create and execute workflow
pub async fn execute_implementation_phase(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
    orchestrator: Arc<Mutex<TaskOrchestrator>>,
) -> Result<(), String> {
    println!("Starting implementation phase");

    let user_request = {
        let pl = pipelines.lock().await;
        let pipeline = pl.get(pipeline_id).ok_or("Pipeline not found")?;
        pipeline.user_request.clone()
    };

    // Create workflow (simplified - would normally parse plan)
    let orch = orchestrator.lock().await;
    let workflow_id = orch
        .create_workflow_from_request(&user_request)
        .await
        .unwrap_or_else(|_| {
            // If auto-creation fails, return empty string
            String::new()
        });

    if workflow_id.is_empty() {
        return Err("Failed to create workflow".to_string());
    }

    // Execute workflow
    orch.execute_workflow(&workflow_id).await?;
    drop(orch);

    // Store workflow_id
    let mut pl = pipelines.lock().await;
    if let Some(p) = pl.get_mut(pipeline_id) {
        p.workflow_id = Some(workflow_id);
    }

    Ok(())
}

/// Check if phase tasks are complete
pub async fn check_phase_tasks_complete(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
    orchestrator: Arc<Mutex<TaskOrchestrator>>,
) -> bool {
    let workflow_id = {
        let pl = pipelines.lock().await;
        let pipeline = match pl.get(pipeline_id) {
            Some(p) => p,
            None => return false,
        };
        match &pipeline.workflow_id {
            Some(id) => id.clone(),
            None => return true, // No workflow = tasks complete
        }
    };

    let orch = orchestrator.lock().await;
    let workflow = match orch.get_workflow(&workflow_id).await {
        Some(w) => w,
        None => return false,
    };

    workflow.status == WorkflowStatus::Completed
}
