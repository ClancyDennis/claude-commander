// Main pipeline execution loop

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::auto_pipeline::orchestrator::Orchestrator;
use crate::auto_pipeline::orchestrator_agent::{OrchestratorAction, OrchestratorAgent};
use crate::auto_pipeline::types::{AutoPipeline, IterationRecord};

use super::building::execute_building_step;
use super::helpers::{
    emit_decision, emit_pipeline_completed, remove_orchestrator_agent, stop_all_pipeline_agents,
    update_agent_context, with_pipeline, with_pipeline_mut,
};
use super::planning::execute_planning_step;
use super::replan::execute_replan_step_v2;
use super::verification::execute_verification_step;

/// Execute the full pipeline with iteration loop
///
/// Flow:
/// 1. Plan -> Build -> Verify
/// 2. OrchestratorAgent decides: complete | iterate | replan | give_up
/// 3. If iterate: go back to Build
/// 4. If replan: go back to Plan
/// 5. Repeat until complete or give_up or max_iterations
pub async fn execute_pipeline(
    pipeline_id: String,
    pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    orchestrator: &Orchestrator,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    // Step 1: Initial Planning (includes Resource Synthesis)
    execute_planning_step(
        &pipeline_id,
        pipelines.clone(),
        orchestrator_agents.clone(),
        orchestrator,
        agent_manager.clone(),
        app_handle.clone(),
    )
    .await?;

    // Check if pipeline was completed early during planning
    let early_complete =
        with_pipeline(&pipelines, &pipeline_id, |p| p.status == "completed").await?;

    if early_complete {
        eprintln!("[auto_pipeline] Pipeline completed during planning phase, skipping build loop");
        return Ok(());
    }

    // Main iteration loop
    loop {
        // Step 2: Building
        execute_building_step(
            &pipeline_id,
            pipelines.clone(),
            orchestrator_agents.clone(),
            agent_manager.clone(),
            app_handle.clone(),
        )
        .await?;

        // Check if pipeline was completed early during build
        let early_complete =
            with_pipeline(&pipelines, &pipeline_id, |p| p.status == "completed").await?;

        if early_complete {
            eprintln!("[auto_pipeline] Pipeline completed during build phase, cleaning up");
            remove_orchestrator_agent(&orchestrator_agents, &pipeline_id).await;
            stop_all_pipeline_agents(&pipelines, &pipeline_id, &agent_manager).await;
            return Ok(());
        }

        // Step 3: Verification - returns the OrchestratorAgent's decision
        let action = execute_verification_step(
            &pipeline_id,
            pipelines.clone(),
            orchestrator_agents.clone(),
            agent_manager.clone(),
            app_handle.clone(),
        )
        .await?;

        // Extract decision info for logging and events
        let (decision_name, reasoning, issues, suggestions) = extract_decision_info(&action);

        // Record the iteration
        with_pipeline_mut(&pipelines, &pipeline_id, |pipeline| {
            pipeline.iteration_history.push(IterationRecord {
                iteration: pipeline.current_iteration,
                decision: decision_name.to_string(),
                reasoning: reasoning.clone(),
                issues: issues.clone(),
            });
        })
        .await?;

        // Emit decision event
        emit_decision(
            &app_handle,
            &pipeline_id,
            decision_name,
            &reasoning,
            &issues,
            &suggestions,
        );

        // Handle the decision
        match action {
            OrchestratorAction::Complete { summary } => {
                return handle_complete(
                    &pipeline_id,
                    &summary,
                    &pipelines,
                    &orchestrator_agents,
                    &agent_manager,
                    &app_handle,
                )
                .await;
            }

            OrchestratorAction::Iterate { issues, suggestions } => {
                let should_continue = handle_iterate(
                    &pipeline_id,
                    &issues,
                    &suggestions,
                    &pipelines,
                    &orchestrator_agents,
                    &agent_manager,
                    &app_handle,
                )
                .await?;

                if !should_continue {
                    return Err("Max iterations reached".to_string());
                }
                continue;
            }

            OrchestratorAction::Replan {
                reason,
                issues,
                suggestions,
            } => {
                let should_continue = handle_replan(
                    &pipeline_id,
                    &reason,
                    &issues,
                    &suggestions,
                    &pipelines,
                    &orchestrator_agents,
                    &agent_manager,
                    &app_handle,
                )
                .await?;

                if !should_continue {
                    return Err("Max iterations reached".to_string());
                }

                // Execute replan
                execute_replan_step_v2(
                    &pipeline_id,
                    pipelines.clone(),
                    orchestrator_agents.clone(),
                    agent_manager.clone(),
                    app_handle.clone(),
                )
                .await?;

                continue;
            }

            OrchestratorAction::GiveUp { reason } => {
                return handle_give_up(
                    &pipeline_id,
                    &reason,
                    &pipelines,
                    &orchestrator_agents,
                    &agent_manager,
                    &app_handle,
                )
                .await;
            }

            _ => {
                return Err("Unexpected action from verification step".to_string());
            }
        }
    }
}

/// Extract decision info from an action for logging
fn extract_decision_info(action: &OrchestratorAction) -> (&str, String, Vec<String>, Vec<String>) {
    match action {
        OrchestratorAction::Complete { summary } => ("Complete", summary.clone(), vec![], vec![]),
        OrchestratorAction::Iterate { issues, suggestions } => {
            ("Iterate", String::new(), issues.clone(), suggestions.clone())
        }
        OrchestratorAction::Replan {
            reason,
            issues,
            suggestions,
        } => ("Replan", reason.clone(), issues.clone(), suggestions.clone()),
        OrchestratorAction::GiveUp { reason } => ("GiveUp", reason.clone(), vec![], vec![]),
        _ => ("Unknown", String::new(), vec![], vec![]),
    }
}

/// Handle the Complete decision
async fn handle_complete(
    pipeline_id: &str,
    summary: &str,
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: &Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: &Arc<Mutex<AgentManager>>,
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    with_pipeline_mut(pipelines, pipeline_id, |pipeline| {
        pipeline.mark_completed("complete");
    })
    .await?;

    remove_orchestrator_agent(orchestrator_agents, pipeline_id).await;
    stop_all_pipeline_agents(pipelines, pipeline_id, agent_manager).await;

    emit_pipeline_completed(
        app_handle,
        pipeline_id,
        "success",
        "complete",
        Some(json!({"summary": summary})),
    );

    Ok(())
}

/// Handle the Iterate decision - returns whether to continue
async fn handle_iterate(
    pipeline_id: &str,
    issues: &[String],
    suggestions: &[String],
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: &Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: &Arc<Mutex<AgentManager>>,
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
) -> Result<bool, String> {
    let should_continue = with_pipeline_mut(pipelines, pipeline_id, |pipeline| {
        if pipeline.at_max_iterations() {
            pipeline.mark_failed("max_iterations");
            false
        } else {
            pipeline.reset_for_iteration();
            true
        }
    })
    .await?;

    if !should_continue {
        cleanup_on_max_iterations(
            pipeline_id,
            pipelines,
            orchestrator_agents,
            agent_manager,
            app_handle,
        )
        .await;
        return Ok(false);
    }

    // Tell the orchestrator agent about the iteration
    let context = format!(
        "Iteration requested. Issues to fix:\n{}\n\nSuggestions:\n{}\n\nPlease call start_execution to implement the fixes.",
        issues.iter().map(|i| format!("- {}", i)).collect::<Vec<_>>().join("\n"),
        suggestions.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
    );
    update_agent_context(orchestrator_agents, pipeline_id, "user", &context, true).await;

    Ok(true)
}

/// Handle the Replan decision - returns whether to continue
async fn handle_replan(
    pipeline_id: &str,
    reason: &str,
    issues: &[String],
    suggestions: &[String],
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: &Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: &Arc<Mutex<AgentManager>>,
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
) -> Result<bool, String> {
    let should_continue = with_pipeline_mut(pipelines, pipeline_id, |pipeline| {
        if pipeline.at_max_iterations() {
            pipeline.mark_failed("max_iterations");
            false
        } else {
            pipeline.reset_for_replan();
            true
        }
    })
    .await?;

    if !should_continue {
        cleanup_on_max_iterations(
            pipeline_id,
            pipelines,
            orchestrator_agents,
            agent_manager,
            app_handle,
        )
        .await;
        return Ok(false);
    }

    // Tell the orchestrator agent to replan
    let context = format!(
        "Replanning required. Reason: {}\n\nIssues:\n{}\n\nSuggestions:\n{}\n\nPlease call start_planning to create a new plan.",
        reason,
        issues.iter().map(|i| format!("- {}", i)).collect::<Vec<_>>().join("\n"),
        suggestions.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
    );
    update_agent_context(orchestrator_agents, pipeline_id, "user", &context, true).await;

    Ok(true)
}

/// Handle the GiveUp decision
async fn handle_give_up(
    pipeline_id: &str,
    reason: &str,
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: &Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: &Arc<Mutex<AgentManager>>,
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    with_pipeline_mut(pipelines, pipeline_id, |pipeline| {
        pipeline.mark_failed("give_up");
    })
    .await?;

    remove_orchestrator_agent(orchestrator_agents, pipeline_id).await;
    stop_all_pipeline_agents(pipelines, pipeline_id, agent_manager).await;

    emit_pipeline_completed(
        app_handle,
        pipeline_id,
        "failed",
        "give_up",
        Some(json!({"reasoning": reason})),
    );

    Err(format!("Pipeline gave up: {}", reason))
}

/// Cleanup when max iterations reached
async fn cleanup_on_max_iterations(
    pipeline_id: &str,
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: &Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: &Arc<Mutex<AgentManager>>,
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
) {
    remove_orchestrator_agent(orchestrator_agents, pipeline_id).await;
    stop_all_pipeline_agents(pipelines, pipeline_id, agent_manager).await;

    emit_pipeline_completed(
        app_handle,
        pipeline_id,
        "failed",
        "max_iterations_reached",
        None,
    );
}
