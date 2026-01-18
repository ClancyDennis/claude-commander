// Step execution functions for auto-pipeline

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::types::AgentSource;

use super::agent_utils::{extract_agent_output, wait_for_agent_completion};
use super::orchestrator::{DecisionResult, Orchestrator};
use super::orchestrator_agent::OrchestratorAgent;
use super::prompts::REPLAN_PROMPT_TEMPLATE;
use super::orchestrator_agent::OrchestratorAction;
use super::types::{AutoPipeline, IterationRecord, StepStatus};

/// Context needed for step execution
pub struct StepExecutionContext {
    pub pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    pub orchestrator: Orchestrator,
    /// Persistent orchestrator agents per pipeline (for new mode)
    pub orchestrator_agents: Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
}

impl StepExecutionContext {
    /// Update step status for a pipeline and emit event
    pub async fn update_step_status(
        &self,
        pipeline_id: &str,
        step_number: u8,
        status: StepStatus,
        app_handle: &Arc<dyn crate::events::AppEventEmitter>,
    ) {
        let mut pipelines = self.pipelines.lock().await;
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            let index = (step_number - 1) as usize;
            pipeline.steps[index].status = status.clone();

            // Emit step status change event
            let _ = app_handle.emit(
                "auto_pipeline:step_status",
                json!({
                    "pipeline_id": pipeline_id,
                    "step_number": step_number,
                    "status": format!("{:?}", status),
                }),
            );
        }
    }

    /// Stop the agent from a previous step if it exists
    async fn stop_previous_step_agent(
        &self,
        pipeline_id: &str,
        step_number: u8,
        agent_manager: &Arc<Mutex<AgentManager>>,
    ) {
        eprintln!("[auto_pipeline] stop_previous_step_agent called for pipeline={}, step={}", pipeline_id, step_number);

        let agent_id = {
            let pipelines = self.pipelines.lock().await;
            pipelines
                .get(pipeline_id)
                .and_then(|p| p.steps.get((step_number - 1) as usize))
                .and_then(|s| s.agent_id.clone())
        };

        if let Some(ref agent_id) = agent_id {
            eprintln!("[auto_pipeline] Stopping agent {} from step {}", agent_id, step_number);
            let manager = agent_manager.lock().await;
            let _ = manager.stop_agent(agent_id).await;
            eprintln!("[auto_pipeline] Agent {} stopped", agent_id);
        } else {
            eprintln!("[auto_pipeline] No agent to stop for step {}", step_number);
        }
    }

    /// Stop all agents associated with a pipeline
    pub async fn stop_all_pipeline_agents(
        &self,
        pipeline_id: &str,
        agent_manager: &Arc<Mutex<AgentManager>>,
    ) {
        eprintln!("[auto_pipeline] stop_all_pipeline_agents called for pipeline={}", pipeline_id);

        let agent_ids: Vec<String> = {
            let pipelines = self.pipelines.lock().await;
            pipelines
                .get(pipeline_id)
                .map(|p| {
                    p.steps
                        .iter()
                        .filter_map(|s| s.agent_id.clone())
                        .collect()
                })
                .unwrap_or_default()
        };

        eprintln!("[auto_pipeline] Found {} agents to stop: {:?}", agent_ids.len(), agent_ids);

        let manager = agent_manager.lock().await;
        for agent_id in agent_ids {
            eprintln!("[auto_pipeline] Stopping agent {}", agent_id);
            let _ = manager.stop_agent(&agent_id).await;
            eprintln!("[auto_pipeline] Agent {} stopped", agent_id);
        }

        eprintln!("[auto_pipeline] stop_all_pipeline_agents completed");
    }

    /// Execute the planning step using the OrchestratorAgent
    ///
    /// This runs the Resource Synthesis phase (reads instructions, creates skills)
    /// and then the Planning phase (spawns planning agent, reviews plan).
    pub async fn execute_planning_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        eprintln!("[auto_pipeline] execute_planning_step starting for pipeline={}", pipeline_id);

        let (user_request, working_dir) = {
            let pipelines = self.pipelines.lock().await;
            let pipeline = pipelines.get(pipeline_id).ok_or("Pipeline not found")?;
            (pipeline.user_request.clone(), pipeline.working_dir.clone())
        };

        self.update_step_status(pipeline_id, 1, StepStatus::Running, &app_handle).await;

        // Get custom instructions and max iterations from the legacy orchestrator
        let custom_instructions = self.orchestrator.custom_instructions().map(String::from);
        let max_iterations = self.orchestrator.max_iterations();

        // Create the OrchestratorAgent with full agent manager integration
        // This agent will handle Resource Synthesis (reading instructions, creating skills)
        // and Planning (spawning planning agent, reviewing plan)
        let mut orchestrator_agent = OrchestratorAgent::with_agent_manager(
            working_dir.clone(),
            user_request.clone(),
            custom_instructions,
            max_iterations,
            agent_manager.clone(),
            app_handle.clone(),
        )?;

        eprintln!("[auto_pipeline] OrchestratorAgent created, starting tool loop");

        // Run the orchestrator until it reaches the end of planning
        // The agent will:
        // 1. Analyze the task (ReceivedTask -> AnalyzingTask)
        // 2. Read instruction files and select relevant ones (SelectingInstructions)
        // 3. Create skills/subagents if needed (GeneratingSkills)
        // 4. Call start_planning which spawns the planning agent (Planning)
        // 5. Review the plan and either approve or replan (PlanReady)
        //
        // The loop continues until the agent calls approve_plan (which we handle)
        // or a terminal decision (complete, iterate, replan, give_up)

        loop {
            match orchestrator_agent.run_until_action().await? {
                OrchestratorAction::Complete { summary } => {
                    // Rare case: agent decides task is complete during planning
                    eprintln!("[auto_pipeline] OrchestratorAgent completed during planning: {}", summary);

                    // Store the agent for potential reuse
                    {
                        let mut agents = self.orchestrator_agents.lock().await;
                        agents.insert(pipeline_id.to_string(), orchestrator_agent);
                    }

                    // Update pipeline - mark as completed so execute_pipeline knows to skip build loop
                    {
                        let mut pipelines = self.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                            pipeline.steps[0].status = StepStatus::Completed;
                            pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
                            pipeline.mark_completed(&summary);  // Mark entire pipeline as completed
                        }
                    }

                    let _ = app_handle.emit(
                        "auto_pipeline:step_completed",
                        json!({
                            "pipeline_id": pipeline_id,
                            "step_number": 1,
                            "early_complete": true,
                            "summary": summary,
                        }),
                    );

                    // Also emit pipeline completed event
                    let _ = app_handle.emit(
                        "auto_pipeline:completed",
                        json!({
                            "pipeline_id": pipeline_id,
                            "status": "success",
                            "decision": "complete",
                            "summary": summary,
                        }),
                    );

                    return Ok(());
                }

                OrchestratorAction::GiveUp { reason } => {
                    eprintln!("[auto_pipeline] OrchestratorAgent gave up during planning: {}", reason);
                    return Err(format!("Orchestrator gave up during planning: {}", reason));
                }

                OrchestratorAction::Iterate { .. } | OrchestratorAction::Replan { .. } => {
                    // During planning, these actions mean the agent wants to revise its approach
                    // The agent will handle this internally by continuing the loop
                    eprintln!("[auto_pipeline] OrchestratorAgent requested iteration/replan during planning phase");
                    continue;
                }

                OrchestratorAction::StartPlanning { .. } |
                OrchestratorAction::ApprovePlan { .. } |
                OrchestratorAction::StartExecution { .. } |
                OrchestratorAction::StartVerification { .. } => {
                    // These are internal state transitions handled by the tool loop
                    // If we get here, the planning phase is complete
                    eprintln!("[auto_pipeline] Planning phase complete, agent ready for execution");
                    break;
                }

                OrchestratorAction::Continue => {
                    // Agent is still working through synthesis/planning
                    continue;
                }
            }
        }

        // Extract outputs from the orchestrator agent
        let plan = orchestrator_agent.current_plan.clone();
        let _qna = orchestrator_agent.current_qna.clone();
        let generated_skills = orchestrator_agent.generated_skills().to_vec();
        let generated_subagents = orchestrator_agent.generated_subagents().to_vec();

        eprintln!(
            "[auto_pipeline] Planning complete. Skills: {:?}, Subagents: {:?}",
            generated_skills, generated_subagents
        );

        // Store the orchestrator agent for use in subsequent steps
        {
            let mut agents = self.orchestrator_agents.lock().await;
            agents.insert(pipeline_id.to_string(), orchestrator_agent);
        }

        // Update pipeline with planning outputs
        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.refined_request = Some(plan.clone());
                pipeline.steps[0].output = Some(super::types::StepOutput {
                    raw_text: plan,
                    structured_data: None,
                    agent_outputs: vec![],
                });
                pipeline.steps[0].status = StepStatus::Completed;
                pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        let _ = app_handle.emit(
            "auto_pipeline:step_completed",
            json!({
                "pipeline_id": pipeline_id,
                "step_number": 1,
                "generated_skills": generated_skills,
                "generated_subagents": generated_subagents,
            }),
        );

        Ok(())
    }

    /// Execute the building step using the OrchestratorAgent
    ///
    /// This uses the stored OrchestratorAgent to run the execution phase.
    /// The agent will call start_execution which spawns the build agent.
    pub async fn execute_building_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        eprintln!("[auto_pipeline] execute_building_step starting for pipeline={}", pipeline_id);

        // Stop the planning agent before starting the building agent
        self.stop_previous_step_agent(pipeline_id, 1, &agent_manager).await;

        eprintln!("[auto_pipeline] execute_building_step: previous agent stopped, continuing");

        self.update_step_status(pipeline_id, 2, StepStatus::Running, &app_handle).await;

        // Get the stored orchestrator agent
        let mut orchestrator_agent = {
            let mut agents = self.orchestrator_agents.lock().await;
            agents.remove(pipeline_id).ok_or("OrchestratorAgent not found for pipeline")?
        };

        // Tell the agent to start execution
        // The agent already has the plan stored from the planning phase
        orchestrator_agent.add_context(
            "user",
            "The plan has been approved. Please call start_execution to begin implementing the plan.",
        );

        // Run until execution completes (agent will call start_verification when done)
        loop {
            match orchestrator_agent.run_until_action().await? {
                OrchestratorAction::StartVerification { .. } => {
                    // Execution complete, ready for verification
                    eprintln!("[auto_pipeline] Build phase complete, ready for verification");
                    break;
                }

                OrchestratorAction::Complete { summary } => {
                    // Rare: agent decides task is complete during build
                    eprintln!("[auto_pipeline] OrchestratorAgent completed during build: {}", summary);

                    // Store agent back
                    {
                        let mut agents = self.orchestrator_agents.lock().await;
                        agents.insert(pipeline_id.to_string(), orchestrator_agent);
                    }

                    // Mark pipeline as completed
                    {
                        let mut pipelines = self.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                            pipeline.steps[1].status = StepStatus::Completed;
                            pipeline.steps[1].completed_at = Some(chrono::Utc::now().to_rfc3339());
                            pipeline.mark_completed(&summary);  // Mark entire pipeline as completed
                        }
                    }

                    // Emit pipeline completed event
                    let _ = app_handle.emit(
                        "auto_pipeline:completed",
                        json!({
                            "pipeline_id": pipeline_id,
                            "status": "success",
                            "decision": "complete",
                            "summary": summary,
                        }),
                    );

                    return Ok(());
                }

                OrchestratorAction::GiveUp { reason } => {
                    eprintln!("[auto_pipeline] OrchestratorAgent gave up during build: {}", reason);
                    return Err(format!("Orchestrator gave up during build: {}", reason));
                }

                OrchestratorAction::Iterate { .. } | OrchestratorAction::Replan { .. } => {
                    // During build, these signal issues - continue loop
                    eprintln!("[auto_pipeline] OrchestratorAgent requested iteration/replan during build");
                    continue;
                }

                OrchestratorAction::StartPlanning { .. } |
                OrchestratorAction::ApprovePlan { .. } |
                OrchestratorAction::StartExecution { .. } => {
                    // Internal transitions, continue
                    continue;
                }

                OrchestratorAction::Continue => {
                    continue;
                }
            }
        }

        // Extract build output
        let implementation = orchestrator_agent.current_implementation.clone();

        // Store agent back for verification step
        {
            let mut agents = self.orchestrator_agents.lock().await;
            agents.insert(pipeline_id.to_string(), orchestrator_agent);
        }

        // Update pipeline
        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.steps[1].output = Some(super::types::StepOutput {
                    raw_text: implementation,
                    structured_data: None,
                    agent_outputs: vec![],
                });
                pipeline.steps[1].status = StepStatus::Completed;
                pipeline.steps[1].completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        let _ = app_handle.emit(
            "auto_pipeline:step_completed",
            json!({
                "pipeline_id": pipeline_id,
                "step_number": 2,
            }),
        );

        Ok(())
    }

    /// Execute the verification step using the OrchestratorAgent
    ///
    /// This uses the stored OrchestratorAgent to run verification.
    /// The agent will call start_verification which spawns the verification agent,
    /// then make a decision (complete, iterate, replan, give_up).
    ///
    /// Returns the OrchestratorAction decision for the caller to handle.
    pub async fn execute_verification_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<OrchestratorAction, String> {
        eprintln!("[auto_pipeline] execute_verification_step starting for pipeline={}", pipeline_id);

        // Stop the building agent before starting the verification agent
        self.stop_previous_step_agent(pipeline_id, 2, &agent_manager).await;

        eprintln!("[auto_pipeline] execute_verification_step: previous agent stopped, continuing");

        self.update_step_status(pipeline_id, 3, StepStatus::Running, &app_handle).await;

        // Get the stored orchestrator agent
        let mut orchestrator_agent = {
            let mut agents = self.orchestrator_agents.lock().await;
            agents.remove(pipeline_id).ok_or("OrchestratorAgent not found for pipeline")?
        };

        // Tell the agent to start verification
        orchestrator_agent.add_context(
            "user",
            "The implementation is complete. Please call start_verification to verify the implementation.",
        );

        // Run until we get a terminal decision
        let decision = loop {
            match orchestrator_agent.run_until_action().await? {
                action @ (OrchestratorAction::Complete { .. } |
                          OrchestratorAction::Iterate { .. } |
                          OrchestratorAction::Replan { .. } |
                          OrchestratorAction::GiveUp { .. }) => {
                    eprintln!("[auto_pipeline] Verification complete, got decision: {:?}",
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

        // Store agent back for potential replan/iterate iterations
        {
            let mut agents = self.orchestrator_agents.lock().await;
            agents.insert(pipeline_id.to_string(), orchestrator_agent);
        }

        // Update pipeline
        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.steps[2].output = Some(super::types::StepOutput {
                    raw_text: "Verification completed by OrchestratorAgent".to_string(),
                    structured_data: None,
                    agent_outputs: vec![],
                });
                pipeline.steps[2].status = StepStatus::Completed;
                pipeline.steps[2].completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        let _ = app_handle.emit(
            "auto_pipeline:step_completed",
            json!({
                "pipeline_id": pipeline_id,
                "step_number": 3,
            }),
        );

        Ok(decision)
    }

    /// Execute the replan step when orchestrator decides to go back to planning
    pub async fn execute_replan_step(
        &self,
        pipeline_id: &str,
        decision: &DecisionResult,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        eprintln!("[auto_pipeline] execute_replan_step starting for pipeline={}", pipeline_id);

        // Stop the verification agent before starting the replan agent
        self.stop_previous_step_agent(pipeline_id, 3, &agent_manager).await;

        eprintln!("[auto_pipeline] execute_replan_step: previous agent stopped, continuing");

        let (user_request, working_dir, previous_plan, qna, build_output, verification_output) = {
            let pipelines = self.pipelines.lock().await;
            let pipeline = pipelines.get(pipeline_id).ok_or("Pipeline not found")?;

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
        };

        self.update_step_status(pipeline_id, 1, StepStatus::Running, &app_handle).await;

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

        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.steps[0].agent_id = Some(agent_id.clone());
                pipeline.steps[0].started_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

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
        let user_req = {
            let pipelines = self.pipelines.lock().await;
            pipelines
                .get(pipeline_id)
                .map(|p| p.user_request.clone())
                .unwrap_or_default()
        };

        // Emit tool start for Q&A (Replan)
        let _ = app_handle.emit("orchestrator:tool_start", json!({
            "tool_name": "generate_answers",
            "tool_input": {
                "question_count": questions.len(),
                "context": "replanning"
            },
            "current_state": "Replanning",
            "iteration": 1 // Ideally track actual iteration
        }));

        let answers = self.orchestrator.generate_answers(&user_req, None, &questions).await?;

        // Emit tool complete
        let _ = app_handle.emit("orchestrator:tool_complete", json!({
            "tool_name": "generate_answers",
            "is_error": false,
            "summary": format!("Generated {} answers for replan", answers.len()),
            "current_state": "Replanning"
        }));

        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.steps[0].output = Some(output.clone());
                pipeline.steps[0].status = StepStatus::Completed;
                pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
                pipeline.questions = questions;
                pipeline.answers = answers;
            }
        }

        let _ = app_handle.emit(
            "auto_pipeline:step_completed",
            json!({
                "pipeline_id": pipeline_id,
                "step_number": 1,
                "step_type": "replan",
                "output": output.structured_data,
            }),
        );

        Ok(())
    }

    /// Execute replan step using the OrchestratorAgent
    ///
    /// The agent already has the replan context added by mod.rs.
    /// This runs the planning loop until a new plan is approved.
    pub async fn execute_replan_step_v2(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        eprintln!("[auto_pipeline] execute_replan_step_v2 starting for pipeline={}", pipeline_id);

        // Stop the verification agent
        self.stop_previous_step_agent(pipeline_id, 3, &agent_manager).await;

        self.update_step_status(pipeline_id, 1, StepStatus::Running, &app_handle).await;

        // Get the stored orchestrator agent (context already added by mod.rs)
        let mut orchestrator_agent = {
            let mut agents = self.orchestrator_agents.lock().await;
            agents.remove(pipeline_id).ok_or("OrchestratorAgent not found for pipeline")?
        };

        // Run until planning completes
        loop {
            match orchestrator_agent.run_until_action().await? {
                OrchestratorAction::ApprovePlan { .. } |
                OrchestratorAction::StartExecution { .. } => {
                    eprintln!("[auto_pipeline] Replan complete, new plan approved");
                    break;
                }

                OrchestratorAction::Complete { summary } => {
                    eprintln!("[auto_pipeline] OrchestratorAgent completed during replan: {}", summary);
                    break;
                }

                OrchestratorAction::GiveUp { reason } => {
                    return Err(format!("Orchestrator gave up during replan: {}", reason));
                }

                OrchestratorAction::StartPlanning { .. } |
                OrchestratorAction::Replan { .. } => {
                    // Internal transitions during replanning
                    continue;
                }

                _ => continue,
            }
        }

        // Extract new plan
        let plan = orchestrator_agent.current_plan.clone();

        // Store agent back
        {
            let mut agents = self.orchestrator_agents.lock().await;
            agents.insert(pipeline_id.to_string(), orchestrator_agent);
        }

        // Update pipeline
        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.refined_request = Some(plan.clone());
                pipeline.steps[0].output = Some(super::types::StepOutput {
                    raw_text: plan,
                    structured_data: None,
                    agent_outputs: vec![],
                });
                pipeline.steps[0].status = StepStatus::Completed;
                pipeline.steps[0].completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        let _ = app_handle.emit(
            "auto_pipeline:step_completed",
            json!({
                "pipeline_id": pipeline_id,
                "step_number": 1,
                "step_type": "replan",
            }),
        );

        Ok(())
    }

    /// Execute the full pipeline with iteration loop
    ///
    /// This method can be called on Arc<StepExecutionContext> to enable
    /// concurrent pipeline execution without holding the manager lock.
    ///
    /// Flow:
    /// 1. Plan -> Build -> Verify
    /// 2. OrchestratorAgent decides: complete | iterate | replan | give_up
    /// 3. If iterate: go back to Build
    /// 4. If replan: go back to Plan
    /// 5. Repeat until complete or give_up or max_iterations
    pub async fn execute_pipeline(
        &self,
        pipeline_id: String,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        // Step 1: Initial Planning (includes Resource Synthesis)
        self.execute_planning_step(&pipeline_id, agent_manager.clone(), app_handle.clone())
            .await?;

        // Check if pipeline was completed early during planning (e.g., simple tasks)
        {
            let pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get(&pipeline_id) {
                if pipeline.status == "completed" {
                    eprintln!("[auto_pipeline] Pipeline completed during planning phase, skipping build loop");
                    return Ok(());
                }
            }
        }

        // Main iteration loop
        loop {
            // Step 2: Building
            self.execute_building_step(&pipeline_id, agent_manager.clone(), app_handle.clone())
                .await?;

            // Check if pipeline was completed early during build
            {
                let pipelines = self.pipelines.lock().await;
                if let Some(pipeline) = pipelines.get(&pipeline_id) {
                    if pipeline.status == "completed" {
                        eprintln!("[auto_pipeline] Pipeline completed during build phase, cleaning up");
                        // Clean up orchestrator agent
                        drop(pipelines);
                        {
                            let mut agents = self.orchestrator_agents.lock().await;
                            agents.remove(&pipeline_id);
                        }
                        self.stop_all_pipeline_agents(&pipeline_id, &agent_manager)
                            .await;
                        return Ok(());
                    }
                }
            }

            // Step 3: Verification - returns the OrchestratorAgent's decision
            let action = self
                .execute_verification_step(&pipeline_id, agent_manager.clone(), app_handle.clone())
                .await?;

            // Extract decision info for logging and events
            let (decision_name, reasoning, issues, suggestions) = match &action {
                OrchestratorAction::Complete { summary } => {
                    ("Complete", summary.clone(), vec![], vec![])
                }
                OrchestratorAction::Iterate { issues, suggestions } => {
                    ("Iterate", String::new(), issues.clone(), suggestions.clone())
                }
                OrchestratorAction::Replan {
                    reason,
                    issues,
                    suggestions,
                } => ("Replan", reason.clone(), issues.clone(), suggestions.clone()),
                OrchestratorAction::GiveUp { reason } => {
                    ("GiveUp", reason.clone(), vec![], vec![])
                }
                _ => ("Unknown", String::new(), vec![], vec![]),
            };

            // Record the iteration
            {
                let mut pipelines = self.pipelines.lock().await;
                if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                    pipeline.iteration_history.push(IterationRecord {
                        iteration: pipeline.current_iteration,
                        decision: decision_name.to_string(),
                        reasoning: reasoning.clone(),
                        issues: issues.clone(),
                    });
                }
            }

            // Emit decision event
            let _ = app_handle.emit(
                "auto_pipeline:decision",
                json!({
                    "pipeline_id": pipeline_id,
                    "decision": decision_name,
                    "reasoning": reasoning,
                    "issues": issues,
                    "suggestions": suggestions,
                }),
            );

            match action {
                OrchestratorAction::Complete { summary } => {
                    // Success! Mark as completed and cleanup agents
                    {
                        let mut pipelines = self.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                            pipeline.mark_completed("complete");
                        }
                    }

                    // Clean up the orchestrator agent
                    {
                        let mut agents = self.orchestrator_agents.lock().await;
                        agents.remove(&pipeline_id);
                    }

                    // Stop all pipeline agents
                    self.stop_all_pipeline_agents(&pipeline_id, &agent_manager)
                        .await;

                    let _ = app_handle.emit(
                        "auto_pipeline:completed",
                        json!({
                            "pipeline_id": pipeline_id,
                            "status": "success",
                            "decision": "complete",
                            "summary": summary,
                        }),
                    );

                    return Ok(());
                }

                OrchestratorAction::Iterate { issues, suggestions } => {
                    // Check iteration limit
                    let should_continue = {
                        let mut pipelines = self.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                            if pipeline.at_max_iterations() {
                                pipeline.mark_failed("max_iterations");
                                false
                            } else {
                                pipeline.reset_for_iteration();
                                true
                            }
                        } else {
                            false
                        }
                    };

                    if !should_continue {
                        // Clean up
                        {
                            let mut agents = self.orchestrator_agents.lock().await;
                            agents.remove(&pipeline_id);
                        }
                        self.stop_all_pipeline_agents(&pipeline_id, &agent_manager)
                            .await;

                        let _ = app_handle.emit(
                            "auto_pipeline:completed",
                            json!({
                                "pipeline_id": pipeline_id,
                                "status": "failed",
                                "decision": "max_iterations_reached",
                            }),
                        );
                        return Err("Max iterations reached".to_string());
                    }

                    // Tell the orchestrator agent about the iteration
                    {
                        let mut agents = self.orchestrator_agents.lock().await;
                        if let Some(agent) = agents.get_mut(&pipeline_id) {
                            let context = format!(
                                "Iteration requested. Issues to fix:\n{}\n\nSuggestions:\n{}\n\nPlease call start_execution to implement the fixes.",
                                issues.iter().map(|i| format!("- {}", i)).collect::<Vec<_>>().join("\n"),
                                suggestions.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
                            );
                            agent.add_context("user", &context);
                            agent.increment_iteration();
                        }
                    }

                    // Continue loop - will go back to Build step
                    continue;
                }

                OrchestratorAction::Replan {
                    reason,
                    issues,
                    suggestions,
                } => {
                    // Check iteration limit
                    let should_continue = {
                        let mut pipelines = self.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                            if pipeline.at_max_iterations() {
                                pipeline.mark_failed("max_iterations");
                                false
                            } else {
                                pipeline.reset_for_replan();
                                true
                            }
                        } else {
                            false
                        }
                    };

                    if !should_continue {
                        // Clean up
                        {
                            let mut agents = self.orchestrator_agents.lock().await;
                            agents.remove(&pipeline_id);
                        }
                        self.stop_all_pipeline_agents(&pipeline_id, &agent_manager)
                            .await;

                        let _ = app_handle.emit(
                            "auto_pipeline:completed",
                            json!({
                                "pipeline_id": pipeline_id,
                                "status": "failed",
                                "decision": "max_iterations_reached",
                            }),
                        );
                        return Err("Max iterations reached".to_string());
                    }

                    // Tell the orchestrator agent to replan
                    {
                        let mut agents = self.orchestrator_agents.lock().await;
                        if let Some(agent) = agents.get_mut(&pipeline_id) {
                            let context = format!(
                                "Replanning required. Reason: {}\n\nIssues:\n{}\n\nSuggestions:\n{}\n\nPlease call start_planning to create a new plan.",
                                reason,
                                issues.iter().map(|i| format!("- {}", i)).collect::<Vec<_>>().join("\n"),
                                suggestions.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
                            );
                            agent.add_context("user", &context);
                            agent.increment_iteration();
                        }
                    }

                    // Execute replan through orchestrator agent
                    self.execute_replan_step_v2(&pipeline_id, agent_manager.clone(), app_handle.clone())
                        .await?;

                    // Continue loop - will go to Build step
                    continue;
                }

                OrchestratorAction::GiveUp { reason } => {
                    // Cannot complete - mark as failed and cleanup agents
                    {
                        let mut pipelines = self.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                            pipeline.mark_failed("give_up");
                        }
                    }

                    // Clean up
                    {
                        let mut agents = self.orchestrator_agents.lock().await;
                        agents.remove(&pipeline_id);
                    }
                    self.stop_all_pipeline_agents(&pipeline_id, &agent_manager)
                        .await;

                    let _ = app_handle.emit(
                        "auto_pipeline:completed",
                        json!({
                            "pipeline_id": pipeline_id,
                            "status": "failed",
                            "decision": "give_up",
                            "reasoning": reason,
                        }),
                    );

                    return Err(format!("Pipeline gave up: {}", reason));
                }

                _ => {
                    // Unexpected action
                    return Err("Unexpected action from verification step".to_string());
                }
            }
        }
    }
}
