// Auto-pipeline module
//
// Implements an iterative automated development pipeline:
// 1. Planning - Creates implementation plan and generates clarifying questions
// 2. Building - Implements the solution based on the plan and Q&A
// 3. Verifying - Reviews implementation and generates verification report
// 4. Orchestrator decides: complete | iterate (back to build) | replan (back to plan)

mod agent_utils;
mod orchestrator;
mod prompts;
mod steps;
mod types;

pub use orchestrator::{DecisionResult, Orchestrator, OrchestratorDecision, RefinedTask};
pub use types::{AutoPipeline, AutoPipelineStep, IterationRecord, StepOutput, StepRole, StepStatus};

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;

use steps::StepExecutionContext;

/// Manager for auto-pipelines
pub struct AutoPipelineManager {
    ctx: StepExecutionContext,
}

impl AutoPipelineManager {
    /// Create a new AutoPipelineManager with default settings
    pub fn new() -> Result<Self, String> {
        let orchestrator = Orchestrator::new()?;

        Ok(Self {
            ctx: StepExecutionContext {
                pipelines: Arc::new(Mutex::new(HashMap::new())),
                orchestrator,
            },
        })
    }

    /// Create a new AutoPipelineManager with custom instructions
    pub fn with_instructions(
        custom_instructions: Option<String>,
        max_iterations: Option<u8>,
    ) -> Result<Self, String> {
        let orchestrator = Orchestrator::with_instructions(custom_instructions, max_iterations)?;

        Ok(Self {
            ctx: StepExecutionContext {
                pipelines: Arc::new(Mutex::new(HashMap::new())),
                orchestrator,
            },
        })
    }

    /// Create a new pipeline
    pub async fn create_pipeline(
        &self,
        user_request: String,
        working_dir: String,
    ) -> Result<String, String> {
        let pipeline_id = uuid::Uuid::new_v4().to_string();
        let max_iterations = self.ctx.orchestrator.max_iterations();
        let pipeline = AutoPipeline::new(pipeline_id.clone(), user_request, working_dir, max_iterations);

        let mut pipelines = self.ctx.pipelines.lock().await;
        pipelines.insert(pipeline_id.clone(), pipeline);

        Ok(pipeline_id)
    }

    /// Execute the pipeline with iteration loop
    ///
    /// Flow:
    /// 1. Plan -> Build -> Verify
    /// 2. Orchestrator decides: complete | iterate | replan | give_up
    /// 3. If iterate: go back to Build
    /// 4. If replan: go back to Plan
    /// 5. Repeat until complete or give_up or max_iterations
    pub async fn execute_pipeline(
        &self,
        pipeline_id: String,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        // Step 1: Initial Planning
        self.ctx
            .execute_planning_step(&pipeline_id, agent_manager.clone(), app_handle.clone())
            .await?;

        // Main iteration loop
        loop {
            // Step 2: Building
            self.ctx
                .execute_building_step(&pipeline_id, agent_manager.clone(), app_handle.clone())
                .await?;

            // Step 3: Verification
            self.ctx
                .execute_verification_step(&pipeline_id, agent_manager.clone(), app_handle.clone())
                .await?;

            // Step 4: Orchestrator decides what to do next
            let decision = self.get_orchestrator_decision(&pipeline_id).await?;

            // Record the iteration
            {
                let mut pipelines = self.ctx.pipelines.lock().await;
                if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                    pipeline.iteration_history.push(IterationRecord {
                        iteration: pipeline.current_iteration,
                        decision: format!("{:?}", decision.decision),
                        reasoning: decision.reasoning.clone(),
                        issues: decision.issues_to_fix.clone(),
                    });
                }
            }

            // Emit decision event
            let _ = app_handle.emit(
                "auto_pipeline:decision",
                json!({
                    "pipeline_id": pipeline_id,
                    "decision": format!("{:?}", decision.decision),
                    "reasoning": decision.reasoning,
                    "issues": decision.issues_to_fix,
                    "suggestions": decision.suggestions,
                }),
            );

            match decision.decision {
                OrchestratorDecision::Complete => {
                    // Success! Mark as completed
                    {
                        let mut pipelines = self.ctx.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                            pipeline.mark_completed("complete");
                        }
                    }

                    let _ = app_handle.emit(
                        "auto_pipeline:completed",
                        json!({
                            "pipeline_id": pipeline_id,
                            "status": "success",
                            "decision": "complete",
                        }),
                    );

                    return Ok(());
                }

                OrchestratorDecision::Iterate => {
                    // Check iteration limit
                    let should_continue = {
                        let mut pipelines = self.ctx.pipelines.lock().await;
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

                    // Continue loop - will go back to Build step
                    continue;
                }

                OrchestratorDecision::Replan => {
                    // Check iteration limit
                    let should_continue = {
                        let mut pipelines = self.ctx.pipelines.lock().await;
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

                    // Execute replan step
                    self.ctx
                        .execute_replan_step(
                            &pipeline_id,
                            &decision,
                            agent_manager.clone(),
                            app_handle.clone(),
                        )
                        .await?;

                    // Continue loop - will go to Build step
                    continue;
                }

                OrchestratorDecision::GiveUp => {
                    // Cannot complete - mark as failed
                    {
                        let mut pipelines = self.ctx.pipelines.lock().await;
                        if let Some(pipeline) = pipelines.get_mut(&pipeline_id) {
                            pipeline.mark_failed("give_up");
                        }
                    }

                    let _ = app_handle.emit(
                        "auto_pipeline:completed",
                        json!({
                            "pipeline_id": pipeline_id,
                            "status": "failed",
                            "decision": "give_up",
                            "reasoning": decision.reasoning,
                        }),
                    );

                    return Err(format!("Pipeline gave up: {}", decision.reasoning));
                }
            }
        }
    }

    /// Get orchestrator decision based on current pipeline state
    async fn get_orchestrator_decision(&self, pipeline_id: &str) -> Result<DecisionResult, String> {
        let (user_request, plan, qna, build_output, verification_output, iteration, previous_issues) = {
            let pipelines = self.ctx.pipelines.lock().await;
            let pipeline = pipelines.get(pipeline_id).ok_or("Pipeline not found")?;

            let plan = pipeline.get_plan_output().unwrap_or_default().to_string();
            let qna = pipeline.format_qna();
            let build_output = pipeline.get_build_output().unwrap_or_default().to_string();
            let verification_output = pipeline.get_verification_output().unwrap_or_default().to_string();

            let previous_issues: Vec<String> = pipeline
                .iteration_history
                .iter()
                .flat_map(|h| h.issues.clone())
                .collect();

            (
                pipeline.user_request.clone(),
                plan,
                qna,
                build_output,
                verification_output,
                pipeline.current_iteration,
                previous_issues,
            )
        };

        let prev_issues = if previous_issues.is_empty() {
            None
        } else {
            Some(previous_issues.as_slice())
        };

        self.ctx.orchestrator
            .decide_on_verification(
                &user_request,
                &plan,
                &qna,
                &build_output,
                &verification_output,
                iteration,
                prev_issues,
            )
            .await
    }

    /// Get a pipeline by ID
    pub async fn get_pipeline(&self, pipeline_id: &str) -> Option<AutoPipeline> {
        let pipelines = self.ctx.pipelines.lock().await;
        pipelines.get(pipeline_id).cloned()
    }
}
