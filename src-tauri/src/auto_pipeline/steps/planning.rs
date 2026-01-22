// Planning step execution

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::auto_pipeline::orchestrator::Orchestrator;
use crate::auto_pipeline::orchestrator_agent::{OrchestratorAction, OrchestratorAgent};
use crate::auto_pipeline::types::{AutoPipeline, StepOutput, StepStatus};

use super::helpers::{
    emit_pipeline_completed, emit_step_completed, store_orchestrator_agent,
    update_step_status, with_pipeline, with_pipeline_mut,
};

/// Execute the planning step using the OrchestratorAgent
///
/// This runs the Resource Synthesis phase (reads instructions, creates skills)
/// and then the Planning phase (spawns planning agent, reviews plan).
pub async fn execute_planning_step(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    orchestrator: &Orchestrator,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    eprintln!(
        "[auto_pipeline] execute_planning_step starting for pipeline={}",
        pipeline_id
    );

    let (user_request, working_dir) = with_pipeline(&pipelines, pipeline_id, |p| {
        (p.user_request.clone(), p.working_dir.clone())
    })
    .await?;

    update_step_status(&pipelines, pipeline_id, 1, StepStatus::Running, &app_handle).await;

    // Get custom instructions and max iterations from the orchestrator
    let custom_instructions = orchestrator.custom_instructions().map(String::from);
    let max_iterations = orchestrator.max_iterations();

    // Create the OrchestratorAgent with full agent manager integration
    let mut orchestrator_agent = OrchestratorAgent::with_agent_manager(
        working_dir.clone(),
        user_request.clone(),
        custom_instructions,
        max_iterations,
        agent_manager.clone(),
        app_handle.clone(),
        pipeline_id.to_string(),
    )?;

    eprintln!("[auto_pipeline] OrchestratorAgent created, starting tool loop");

    // Run the orchestrator until it reaches the end of planning (with max iteration safety check)
    let mut loop_count = 0;
    const MAX_PLANNING_LOOPS: u32 = 100;

    loop {
        loop_count += 1;
        if loop_count > MAX_PLANNING_LOOPS {
            return Err(format!(
                "Planning step exceeded maximum iterations ({}). Agent may be stuck in a loop.",
                MAX_PLANNING_LOOPS
            ));
        }

        eprintln!("[execute_planning_step] Loop iteration {}, current state={:?}", loop_count, orchestrator_agent.current_state);

        // Check if we've transitioned to ReadyForExecution - that means planning is done
        if matches!(orchestrator_agent.current_state, crate::auto_pipeline::state_machine::PipelineState::ReadyForExecution) {
            eprintln!("[execute_planning_step] Orchestrator reached ReadyForExecution state, planning complete");
            break;
        }

        let action = orchestrator_agent.run_until_action().await?;
        eprintln!("[execute_planning_step] Received action: {:?}, current state after action: {:?}", action, orchestrator_agent.current_state);

        // Check state again after run_until_action returns, as it may have transitioned
        if matches!(orchestrator_agent.current_state, crate::auto_pipeline::state_machine::PipelineState::ReadyForExecution) {
            eprintln!("[execute_planning_step] Orchestrator reached ReadyForExecution state after action, planning complete");
            break;
        }

        match action {
            OrchestratorAction::Complete { summary } => {
                eprintln!(
                    "[auto_pipeline] OrchestratorAgent completed during planning: {}",
                    summary
                );

                // Extract agent IDs from the orchestrator before storing
                let planning_agent_id = orchestrator_agent.spawned_agents[0].clone();
                let building_agent_id = orchestrator_agent.spawned_agents[1].clone();
                let verification_agent_id = orchestrator_agent.spawned_agents[2].clone();

                eprintln!("[execute_planning_step] Early completion - storing agent IDs: planning={:?}, building={:?}, verification={:?}",
                    planning_agent_id, building_agent_id, verification_agent_id);

                // Store the agent for potential reuse
                store_orchestrator_agent(&orchestrator_agents, pipeline_id, orchestrator_agent)
                    .await;

                // Update pipeline - mark all steps with their agent IDs and complete the pipeline
                with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
                    // Store agent IDs in each step
                    if let Some(ref agent_id) = planning_agent_id {
                        pipeline.steps[0].agent_id = Some(agent_id.clone());
                        eprintln!("[execute_planning_step] Stored planning agent_id={}", agent_id);
                    }
                    if let Some(ref agent_id) = building_agent_id {
                        pipeline.steps[1].agent_id = Some(agent_id.clone());
                        eprintln!("[execute_planning_step] Stored building agent_id={}", agent_id);
                    }
                    if let Some(ref agent_id) = verification_agent_id {
                        pipeline.steps[2].agent_id = Some(agent_id.clone());
                        eprintln!("[execute_planning_step] Stored verification agent_id={}", agent_id);
                    }

                    // Mark all steps as completed
                    pipeline.steps[0].status = StepStatus::Completed;
                    pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
                    pipeline.steps[1].status = StepStatus::Completed;
                    pipeline.steps[1].completed_at = Some(chrono::Utc::now().to_rfc3339());
                    pipeline.steps[2].status = StepStatus::Completed;
                    pipeline.steps[2].completed_at = Some(chrono::Utc::now().to_rfc3339());
                    pipeline.mark_completed(&summary);
                })
                .await?;

                emit_step_completed(
                    &app_handle,
                    pipeline_id,
                    1,
                    Some(json!({
                        "early_complete": true,
                        "summary": summary.clone(),
                    })),
                );

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
                    "[auto_pipeline] OrchestratorAgent gave up during planning: {}",
                    reason
                );
                return Err(format!("Orchestrator gave up during planning: {}", reason));
            }

            OrchestratorAction::Iterate { .. } | OrchestratorAction::Replan { .. } => {
                eprintln!(
                    "[auto_pipeline] OrchestratorAgent requested iteration/replan during planning phase"
                );
                continue;
            }

            OrchestratorAction::StartPlanning { .. }
            | OrchestratorAction::ApprovePlan { .. }
            | OrchestratorAction::StartExecution { .. }
            | OrchestratorAction::StartVerification { .. } => {
                eprintln!("[auto_pipeline] Planning phase complete, agent ready for execution");
                break;
            }

            OrchestratorAction::Continue => {
                continue;
            }
        }
    }

    // Extract outputs from the orchestrator agent
    let plan = orchestrator_agent.current_plan.clone();
    let generated_skills = orchestrator_agent.generated_skills().to_vec();
    let generated_subagents = orchestrator_agent.generated_subagents().to_vec();
    let planning_outputs = orchestrator_agent.planning_agent_outputs.clone();
    let planning_agent_id = orchestrator_agent.spawned_agents[0].clone();

    eprintln!(
        "[auto_pipeline] Planning complete. Skills: {:?}, Subagents: {:?}, Outputs: {}, Agent: {:?}",
        generated_skills, generated_subagents, planning_outputs.len(), planning_agent_id
    );

    // Store the orchestrator agent for use in subsequent steps
    store_orchestrator_agent(&orchestrator_agents, pipeline_id, orchestrator_agent).await;

    // Store the spawned agent ID in the pipeline step for later cleanup
    if let Some(ref agent_id) = planning_agent_id {
        eprintln!("[execute_planning_step] Storing agent_id={} in pipeline step", agent_id);
        with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
            pipeline.steps[0].agent_id = Some(agent_id.clone());
        }).await?;
    } else {
        eprintln!("[execute_planning_step] WARNING: No planning_agent_id to store!");
    }

    // Update pipeline with planning outputs
    with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
        pipeline.refined_request = Some(plan.clone());
        pipeline.steps[0].output = Some(StepOutput {
            raw_text: plan,
            structured_data: None,
            agent_outputs: planning_outputs,
        });
        pipeline.steps[0].status = StepStatus::Completed;
        pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
    })
    .await?;

    emit_step_completed(
        &app_handle,
        pipeline_id,
        1,
        Some(json!({
            "generated_skills": generated_skills,
            "generated_subagents": generated_subagents,
        })),
    );

    Ok(())
}
