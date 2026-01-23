// Verification step execution

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::auto_pipeline::orchestrator_agent::{OrchestratorAction, OrchestratorAgent};
use crate::auto_pipeline::types::{AutoPipeline, StepOutput, StepStatus};

use super::helpers::{
    emit_step_completed, stop_step_agent, store_orchestrator_agent, take_orchestrator_agent,
    update_step_status, with_pipeline_mut,
};

/// Execute the verification step using the OrchestratorAgent
///
/// This uses the stored OrchestratorAgent to run verification.
/// The agent will call start_verification which spawns the verification agent,
/// then make a decision (complete, iterate, replan, give_up).
///
/// Returns the OrchestratorAction decision for the caller to handle.
pub async fn execute_verification_step(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
) -> Result<OrchestratorAction, String> {
    eprintln!(
        "[auto_pipeline] execute_verification_step starting for pipeline={}",
        pipeline_id
    );

    // Stop the building agent before starting the verification agent
    stop_step_agent(&pipelines, pipeline_id, 2, &agent_manager).await;

    eprintln!("[auto_pipeline] execute_verification_step: previous agent stopped, continuing");

    update_step_status(&pipelines, pipeline_id, 3, StepStatus::Running, &app_handle).await;

    // Get the stored orchestrator agent
    let mut orchestrator_agent = take_orchestrator_agent(&orchestrator_agents, pipeline_id).await?;

    // Tell the agent to start verification
    orchestrator_agent.add_context(
        "user",
        "The implementation is complete. Please call start_verification to verify the implementation.",
    );

    // Run until we get a terminal decision (with max iteration safety check)
    let mut loop_count = 0;
    const MAX_VERIFICATION_LOOPS: u32 = 100;

    let decision = loop {
        loop_count += 1;
        if loop_count > MAX_VERIFICATION_LOOPS {
            return Err(format!(
                "Verification step exceeded maximum iterations ({}). Agent may be stuck in a loop.",
                MAX_VERIFICATION_LOOPS
            ));
        }

        match orchestrator_agent.run_until_action().await? {
            action @ (OrchestratorAction::Complete { .. }
            | OrchestratorAction::Iterate { .. }
            | OrchestratorAction::Replan { .. }
            | OrchestratorAction::GiveUp { .. }) => {
                eprintln!(
                    "[auto_pipeline] Verification complete, got decision: {:?}",
                    match &action {
                        OrchestratorAction::Complete { .. } => "Complete",
                        OrchestratorAction::Iterate { .. } => "Iterate",
                        OrchestratorAction::Replan { .. } => "Replan",
                        OrchestratorAction::GiveUp { .. } => "GiveUp",
                        _ => "Unknown",
                    }
                );
                break action;
            }

            OrchestratorAction::StartVerification { .. } => continue,
            _ => continue,
        }
    };

    // Extract verification outputs before storing agent
    let verification_outputs = orchestrator_agent.verification_agent_outputs.clone();
    let verification_agent_id = orchestrator_agent.spawned_agents[2].clone();

    eprintln!(
        "[auto_pipeline] Verification complete. Outputs: {}, Agent: {:?}",
        verification_outputs.len(),
        verification_agent_id
    );

    // Store agent back for potential replan/iterate iterations
    store_orchestrator_agent(&orchestrator_agents, pipeline_id, orchestrator_agent).await;

    // Store the spawned agent ID in the pipeline step for later cleanup
    if let Some(agent_id) = verification_agent_id {
        with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
            pipeline.steps[2].agent_id = Some(agent_id);
        })
        .await?;
    }

    // Update pipeline
    with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
        pipeline.steps[2].output = Some(StepOutput {
            raw_text: "Verification completed by OrchestratorAgent".to_string(),
            structured_data: None,
            agent_outputs: verification_outputs,
        });
        pipeline.steps[2].status = StepStatus::Completed;
        pipeline.steps[2].completed_at = Some(chrono::Utc::now().to_rfc3339());
    })
    .await?;

    emit_step_completed(&app_handle, pipeline_id, 3, None);

    Ok(decision)
}
