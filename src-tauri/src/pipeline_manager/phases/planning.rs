use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::pipeline_manager::types::{PhaseStatus, Pipeline};

/// Phase 1: Planning - Create execution plan for the pipeline
///
/// This phase captures the user request and prepares it for human review.
/// The actual planning/decomposition happens in the orchestrator during implementation.
pub async fn execute_planning_phase(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
) -> Result<(), String> {
    let user_request = {
        let pl = pipelines.lock().await;
        let pipeline = pl.get(pipeline_id).ok_or("Pipeline not found")?;
        pipeline.user_request.clone()
    };

    println!("Planning phase for request: {}", user_request);

    // Mark as waiting for human review checkpoint
    // Human will review the request before implementation begins
    let mut pl = pipelines.lock().await;
    if let Some(p) = pl.get_mut(pipeline_id) {
        if let Some(phase) = p.current_phase_mut() {
            phase.status = PhaseStatus::WaitingCheckpoint;
        }
    }

    Ok(())
}
