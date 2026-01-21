// Building step execution

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::auto_pipeline::orchestrator_agent::{OrchestratorAction, OrchestratorAgent};
use crate::auto_pipeline::types::{AutoPipeline, StepOutput, StepStatus};

use super::helpers::{
    emit_pipeline_completed, emit_step_completed, stop_step_agent, store_orchestrator_agent,
    take_orchestrator_agent, update_step_status, with_pipeline_mut,
};

/// Execute the building step using the OrchestratorAgent
///
/// This uses the stored OrchestratorAgent to run the execution phase.
/// The agent will call start_execution which spawns the build agent.
pub async fn execute_building_step(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    eprintln!(
        "[auto_pipeline] execute_building_step starting for pipeline={}",
        pipeline_id
    );

    // Stop the planning agent before starting the building agent
    stop_step_agent(&pipelines, pipeline_id, 1, &agent_manager).await;

    eprintln!("[auto_pipeline] execute_building_step: previous agent stopped, continuing");

    update_step_status(&pipelines, pipeline_id, 2, StepStatus::Running, &app_handle).await;

    // Get the stored orchestrator agent
    let mut orchestrator_agent = take_orchestrator_agent(&orchestrator_agents, pipeline_id).await?;

    // Tell the agent to start execution
    orchestrator_agent.add_context(
        "user",
        "The plan has been approved. Please call start_execution to begin implementing the plan.",
    );

    // Run until execution completes
    loop {
        match orchestrator_agent.run_until_action().await? {
            OrchestratorAction::StartVerification { .. } => {
                eprintln!("[auto_pipeline] Build phase complete, ready for verification");
                break;
            }

            OrchestratorAction::Complete { summary } => {
                eprintln!(
                    "[auto_pipeline] OrchestratorAgent completed during build: {}",
                    summary
                );

                // Store agent back
                store_orchestrator_agent(&orchestrator_agents, pipeline_id, orchestrator_agent)
                    .await;

                // Mark pipeline as completed
                with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
                    pipeline.steps[1].status = StepStatus::Completed;
                    pipeline.steps[1].completed_at = Some(chrono::Utc::now().to_rfc3339());
                    pipeline.mark_completed(&summary);
                })
                .await?;

                emit_pipeline_completed(
                    &app_handle,
                    pipeline_id,
                    "success",
                    "complete",
                    Some(json!({"summary": summary})),
                );

                return Ok(());
            }

            OrchestratorAction::GiveUp { reason } => {
                eprintln!(
                    "[auto_pipeline] OrchestratorAgent gave up during build: {}",
                    reason
                );
                return Err(format!("Orchestrator gave up during build: {}", reason));
            }

            OrchestratorAction::Iterate { .. } | OrchestratorAction::Replan { .. } => {
                eprintln!(
                    "[auto_pipeline] OrchestratorAgent requested iteration/replan during build"
                );
                continue;
            }

            OrchestratorAction::StartPlanning { .. }
            | OrchestratorAction::ApprovePlan { .. }
            | OrchestratorAction::StartExecution { .. } => {
                continue;
            }

            OrchestratorAction::Continue => {
                continue;
            }
        }
    }

    // Extract build output and agent outputs
    let implementation = orchestrator_agent.current_implementation.clone();
    let building_outputs = orchestrator_agent.building_agent_outputs.clone();
    let building_agent_id = orchestrator_agent.spawned_agents[1].clone();

    eprintln!(
        "[auto_pipeline] Building complete. Outputs: {}, Agent: {:?}",
        building_outputs.len(), building_agent_id
    );

    // Store agent back for verification step
    store_orchestrator_agent(&orchestrator_agents, pipeline_id, orchestrator_agent).await;

    // Store the spawned agent ID in the pipeline step for later cleanup
    if let Some(agent_id) = building_agent_id {
        with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
            pipeline.steps[1].agent_id = Some(agent_id);
        }).await?;
    }

    // Update pipeline
    with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
        pipeline.steps[1].output = Some(StepOutput {
            raw_text: implementation,
            structured_data: None,
            agent_outputs: building_outputs,
        });
        pipeline.steps[1].status = StepStatus::Completed;
        pipeline.steps[1].completed_at = Some(chrono::Utc::now().to_rfc3339());
    })
    .await?;

    emit_step_completed(&app_handle, pipeline_id, 2, None);

    Ok(())
}
