// Main pipeline execution loop

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::auto_pipeline::orchestrator::Orchestrator;
use crate::auto_pipeline::orchestrator_agent::{OrchestratorAction, OrchestratorAgent};
use crate::auto_pipeline::types::AutoPipeline;

use super::helpers::{
    emit_pipeline_completed, stop_all_pipeline_agents, with_pipeline, with_pipeline_mut,
};

/// Execute the full pipeline with orchestrator managing everything internally
///
/// Flow:
/// 1. Create OrchestratorAgent
/// 2. Hand off to orchestrator - it manages planning, execution, verification, iteration, and replanning
/// 3. Wait for final result (Complete or GiveUp)
pub async fn execute_pipeline(
    pipeline_id: String,
    pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    _orchestrator_agents: Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    orchestrator: &Orchestrator,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    let (user_request, working_dir) = with_pipeline(&pipelines, &pipeline_id, |p| {
        (p.user_request.clone(), p.working_dir.clone())
    })
    .await?;

    let custom_instructions = orchestrator.custom_instructions().map(String::from);
    let max_iterations = orchestrator.max_iterations();

    // Create the OrchestratorAgent
    let mut orchestrator_agent = OrchestratorAgent::with_agent_manager(
        working_dir,
        user_request,
        custom_instructions,
        max_iterations,
        agent_manager.clone(),
        app_handle.clone(),
        pipeline_id.clone(),
    )?;

    eprintln!("[auto_pipeline] Handing off to OrchestratorAgent for complete workflow execution");

    // Hand off to orchestrator - it handles EVERYTHING internally
    // (planning, execution, verification, iteration, replanning, decisions)
    let result = orchestrator_agent.run_to_completion().await?;

    // Clean up agents
    stop_all_pipeline_agents(&pipelines, &pipeline_id, &agent_manager).await;

    // Handle final result
    match result {
        OrchestratorAction::Complete { summary } => {
            with_pipeline_mut(&pipelines, &pipeline_id, |pipeline| {
                pipeline.mark_completed(&summary);
            })
            .await?;

            emit_pipeline_completed(
                &app_handle,
                &pipeline_id,
                "success",
                "complete",
                Some(json!({"summary": summary})),
            );

            eprintln!(
                "[auto_pipeline] Pipeline {} completed successfully",
                pipeline_id
            );
            Ok(())
        }
        OrchestratorAction::GiveUp { reason } => {
            with_pipeline_mut(&pipelines, &pipeline_id, |pipeline| {
                pipeline.mark_failed("give_up");
            })
            .await?;

            emit_pipeline_completed(
                &app_handle,
                &pipeline_id,
                "failed",
                "give_up",
                Some(json!({"reason": reason})),
            );

            eprintln!(
                "[auto_pipeline] Pipeline {} gave up: {}",
                pipeline_id, reason
            );
            Err(format!("Orchestrator gave up: {}", reason))
        }
        _ => {
            eprintln!(
                "[auto_pipeline] Pipeline {} returned unexpected action",
                pipeline_id
            );
            Err("Orchestrator returned unexpected action".to_string())
        }
    }
}
