// Replan step execution

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::auto_pipeline::agent_utils::{extract_agent_output, wait_for_agent_completion};
use crate::auto_pipeline::orchestrator::{DecisionResult, Orchestrator};
use crate::auto_pipeline::orchestrator_agent::{OrchestratorAction, OrchestratorAgent};
use crate::auto_pipeline::prompts::REPLAN_PROMPT_TEMPLATE;
use crate::auto_pipeline::types::{AutoPipeline, StepOutput, StepStatus};
use crate::types::AgentSource;

use super::helpers::{
    emit_step_completed, emit_step_event, stop_step_agent, store_orchestrator_agent,
    take_orchestrator_agent, update_step_status, with_pipeline, with_pipeline_mut,
};

/// Execute the replan step when orchestrator decides to go back to planning (legacy mode)
pub async fn execute_replan_step(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator: &Orchestrator,
    decision: &DecisionResult,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    eprintln!(
        "[auto_pipeline] execute_replan_step starting for pipeline={}",
        pipeline_id
    );

    // Stop the verification agent before starting the replan agent
    stop_step_agent(&pipelines, pipeline_id, 3, &agent_manager).await;

    eprintln!("[auto_pipeline] execute_replan_step: previous agent stopped, continuing");

    let (user_request, working_dir, previous_plan, qna, build_output, verification_output) =
        with_pipeline(&pipelines, pipeline_id, |pipeline| {
            let previous_plan = pipeline.steps[0]
                .output
                .as_ref()
                .map(|o| o.raw_text.clone())
                .unwrap_or_default();

            let qna = pipeline.format_qna();

            let build_output = pipeline.steps[1]
                .output
                .as_ref()
                .map(|o| o.raw_text.clone())
                .unwrap_or_default();

            let verification_output = pipeline.steps[2]
                .output
                .as_ref()
                .map(|o| o.raw_text.clone())
                .unwrap_or_default();

            (
                pipeline.user_request.clone(),
                pipeline.working_dir.clone(),
                previous_plan,
                qna,
                build_output,
                verification_output,
            )
        })
        .await?;

    update_step_status(&pipelines, pipeline_id, 1, StepStatus::Running, &app_handle).await;

    let issues_to_fix = decision.issues_to_fix.join("\n- ");
    let suggestions = decision.suggestions.join("\n- ");

    let replan_prompt = REPLAN_PROMPT_TEMPLATE
        .replace("{user_request}", &user_request)
        .replace("{working_dir}", &working_dir)
        .replace("{previous_plan}", &previous_plan)
        .replace("{qna}", &qna)
        .replace("{build_output}", &build_output)
        .replace("{verification_output}", &verification_output)
        .replace("{issues_to_fix}", &issues_to_fix)
        .replace("{suggestions}", &suggestions);

    let agent_id = {
        let manager = agent_manager.lock().await;
        manager
            .create_agent(
                working_dir.clone(),
                None,
                None,
                AgentSource::Pipeline,
                app_handle.clone(),
            )
            .await?
    };

    with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
        pipeline.steps[0].agent_id = Some(agent_id.clone());
        pipeline.steps[0].started_at = Some(chrono::Utc::now().to_rfc3339());
    })
    .await?;

    {
        let manager = agent_manager.lock().await;
        manager
            .send_prompt(&agent_id, &replan_prompt, Some(app_handle.clone()))
            .await?;
    }

    wait_for_agent_completion(&agent_id, agent_manager.clone()).await?;

    let output = extract_agent_output(&agent_id, agent_manager.clone()).await?;

    // Parse new questions if any
    let structured_data: serde_json::Value =
        serde_json::from_str(&output.raw_text).unwrap_or(json!({}));

    let questions: Vec<String> = structured_data
        .get("questions")
        .and_then(|q| q.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Generate new answers using orchestrator
    let user_req = with_pipeline(&pipelines, pipeline_id, |p| p.user_request.clone()).await?;

    // Emit tool start for Q&A (Replan)
    emit_step_event(
        &app_handle,
        "orchestrator:tool_start",
        json!({
            "tool_name": "generate_answers",
            "tool_input": {
                "question_count": questions.len(),
                "context": "replanning"
            },
            "current_state": "Replanning",
            "iteration": 1
        }),
    );

    let answers = orchestrator
        .generate_answers(&user_req, None, &questions)
        .await?;

    // Emit tool complete
    emit_step_event(
        &app_handle,
        "orchestrator:tool_complete",
        json!({
            "tool_name": "generate_answers",
            "is_error": false,
            "summary": format!("Generated {} answers for replan", answers.len()),
            "current_state": "Replanning"
        }),
    );

    with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
        pipeline.steps[0].output = Some(output.clone());
        pipeline.steps[0].status = StepStatus::Completed;
        pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
        pipeline.questions = questions;
        pipeline.answers = answers;
    })
    .await?;

    emit_step_completed(
        &app_handle,
        pipeline_id,
        1,
        Some(json!({
            "step_type": "replan",
            "output": output.structured_data,
        })),
    );

    Ok(())
}

/// Execute replan step using the OrchestratorAgent (v2 mode)
///
/// The agent already has the replan context added by the main loop.
/// This runs the planning loop until a new plan is approved.
pub async fn execute_replan_step_v2(
    pipeline_id: &str,
    pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    orchestrator_agents: Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
) -> Result<(), String> {
    eprintln!(
        "[auto_pipeline] execute_replan_step_v2 starting for pipeline={}",
        pipeline_id
    );

    // Stop the verification agent
    stop_step_agent(&pipelines, pipeline_id, 3, &agent_manager).await;

    update_step_status(&pipelines, pipeline_id, 1, StepStatus::Running, &app_handle).await;

    // Get the stored orchestrator agent (context already added by main loop)
    let mut orchestrator_agent = take_orchestrator_agent(&orchestrator_agents, pipeline_id).await?;

    // Run until planning completes
    loop {
        match orchestrator_agent.run_until_action().await? {
            OrchestratorAction::ApprovePlan { .. } | OrchestratorAction::StartExecution { .. } => {
                eprintln!("[auto_pipeline] Replan complete, new plan approved");
                break;
            }

            OrchestratorAction::Complete { summary } => {
                eprintln!(
                    "[auto_pipeline] OrchestratorAgent completed during replan: {}",
                    summary
                );
                break;
            }

            OrchestratorAction::GiveUp { reason } => {
                return Err(format!("Orchestrator gave up during replan: {}", reason));
            }

            OrchestratorAction::StartPlanning { .. } | OrchestratorAction::Replan { .. } => {
                continue;
            }

            _ => continue,
        }
    }

    // Extract new plan
    let plan = orchestrator_agent.current_plan.clone();

    // Store agent back
    store_orchestrator_agent(&orchestrator_agents, pipeline_id, orchestrator_agent).await;

    // Update pipeline
    with_pipeline_mut(&pipelines, pipeline_id, |pipeline| {
        pipeline.refined_request = Some(plan.clone());
        pipeline.steps[0].output = Some(StepOutput {
            raw_text: plan,
            structured_data: None,
            agent_outputs: vec![],
        });
        pipeline.steps[0].status = StepStatus::Completed;
        pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
    })
    .await?;

    emit_step_completed(
        &app_handle,
        pipeline_id,
        1,
        Some(json!({"step_type": "replan"})),
    );

    Ok(())
}
