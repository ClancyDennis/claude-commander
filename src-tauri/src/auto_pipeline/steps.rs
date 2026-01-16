// Step execution functions for auto-pipeline

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::types::AgentSource;

use super::agent_utils::{extract_agent_output, wait_for_agent_completion};
use super::orchestrator::{DecisionResult, Orchestrator};
use super::prompts::{
    BUILDER_PROMPT_TEMPLATE, PLANNING_PROMPT_TEMPLATE, REPLAN_PROMPT_TEMPLATE,
    VERIFIER_PROMPT_TEMPLATE,
};
use super::types::{AutoPipeline, StepStatus};

/// Context needed for step execution
pub struct StepExecutionContext {
    pub pipelines: Arc<Mutex<HashMap<String, AutoPipeline>>>,
    pub orchestrator: Orchestrator,
}

impl StepExecutionContext {
    /// Update step status for a pipeline
    pub async fn update_step_status(&self, pipeline_id: &str, step_number: u8, status: StepStatus) {
        let mut pipelines = self.pipelines.lock().await;
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            let index = (step_number - 1) as usize;
            pipeline.steps[index].status = status;
        }
    }

    /// Execute the planning step
    pub async fn execute_planning_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        let (user_request, working_dir) = {
            let pipelines = self.pipelines.lock().await;
            let pipeline = pipelines.get(pipeline_id).ok_or("Pipeline not found")?;
            (pipeline.user_request.clone(), pipeline.working_dir.clone())
        };

        self.update_step_status(pipeline_id, 1, StepStatus::Running).await;

        // Use orchestrator to refine the task before planning
        let refined = self.orchestrator.refine_task(&user_request, &working_dir, None).await?;

        // Store refined request in pipeline
        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.refined_request = Some(refined.refined_request.clone());
            }
        }

        // Use refined request for planning
        let planning_prompt = PLANNING_PROMPT_TEMPLATE
            .replace("{user_request}", &refined.refined_request)
            .replace("{working_dir}", &working_dir);

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
                .send_prompt(&agent_id, &planning_prompt, Some(app_handle.clone()))
                .await?;
        }

        wait_for_agent_completion(&agent_id, agent_manager.clone()).await?;

        let output = extract_agent_output(&agent_id, agent_manager.clone()).await?;

        let structured_data: serde_json::Value = serde_json::from_str(&output.raw_text)
            .map_err(|e| format!("Failed to parse planning output as JSON: {}", e))?;

        let questions: Vec<String> = structured_data
            .get("questions")
            .and_then(|q| q.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Get refined request for better Q&A context
        let refined_request = {
            let pipelines = self.pipelines.lock().await;
            pipelines.get(pipeline_id).and_then(|p| p.refined_request.clone())
        };
        let answers = self.orchestrator.generate_answers(&user_request, refined_request.as_deref(), &questions).await?;

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
                "output": output.structured_data,
            }),
        );

        Ok(())
    }

    /// Execute the building step
    pub async fn execute_building_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        let (user_request, working_dir, plan, qna) = {
            let pipelines = self.pipelines.lock().await;
            let pipeline = pipelines.get(pipeline_id).ok_or("Pipeline not found")?;

            let plan = pipeline.steps[0]
                .output
                .as_ref()
                .ok_or("Planning step not completed")?
                .raw_text
                .clone();

            let qna = pipeline.format_qna();

            (
                pipeline.user_request.clone(),
                pipeline.working_dir.clone(),
                plan,
                qna,
            )
        };

        self.update_step_status(pipeline_id, 2, StepStatus::Running).await;

        let builder_prompt = BUILDER_PROMPT_TEMPLATE
            .replace("{user_request}", &user_request)
            .replace("{plan}", &plan)
            .replace("{qna}", &qna)
            .replace("{working_dir}", &working_dir);

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
                pipeline.steps[1].agent_id = Some(agent_id.clone());
                pipeline.steps[1].started_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        {
            let manager = agent_manager.lock().await;
            manager
                .send_prompt(&agent_id, &builder_prompt, Some(app_handle.clone()))
                .await?;
        }

        wait_for_agent_completion(&agent_id, agent_manager.clone()).await?;

        let output = extract_agent_output(&agent_id, agent_manager).await?;

        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.steps[1].output = Some(output.clone());
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

    /// Execute the verification step
    pub async fn execute_verification_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        let (user_request, working_dir, plan, qna, implementation) = {
            let pipelines = self.pipelines.lock().await;
            let pipeline = pipelines.get(pipeline_id).ok_or("Pipeline not found")?;

            let plan = pipeline.steps[0]
                .output
                .as_ref()
                .ok_or("Planning step not completed")?
                .raw_text
                .clone();

            let implementation = pipeline.steps[1]
                .output
                .as_ref()
                .ok_or("Building step not completed")?
                .raw_text
                .clone();

            let qna = pipeline.format_qna();

            (
                pipeline.user_request.clone(),
                pipeline.working_dir.clone(),
                plan,
                qna,
                implementation,
            )
        };

        self.update_step_status(pipeline_id, 3, StepStatus::Running).await;

        let verifier_prompt = VERIFIER_PROMPT_TEMPLATE
            .replace("{user_request}", &user_request)
            .replace("{plan}", &plan)
            .replace("{qna}", &qna)
            .replace("{implementation}", &implementation);

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
                pipeline.steps[2].agent_id = Some(agent_id.clone());
                pipeline.steps[2].started_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        {
            let manager = agent_manager.lock().await;
            manager
                .send_prompt(&agent_id, &verifier_prompt, Some(app_handle.clone()))
                .await?;
        }

        wait_for_agent_completion(&agent_id, agent_manager.clone()).await?;

        let output = extract_agent_output(&agent_id, agent_manager).await?;

        {
            let mut pipelines = self.pipelines.lock().await;
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                pipeline.steps[2].output = Some(output.clone());
                pipeline.steps[2].status = StepStatus::Completed;
                pipeline.steps[2].completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        let _ = app_handle.emit(
            "auto_pipeline:step_completed",
            json!({
                "pipeline_id": pipeline_id,
                "step_number": 3,
                "verification_report": output.structured_data,
            }),
        );

        Ok(())
    }

    /// Execute the replan step when orchestrator decides to go back to planning
    pub async fn execute_replan_step(
        &self,
        pipeline_id: &str,
        decision: &DecisionResult,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
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

        self.update_step_status(pipeline_id, 1, StepStatus::Running).await;

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
        let answers = self.orchestrator.generate_answers(&user_req, None, &questions).await?;

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
}
