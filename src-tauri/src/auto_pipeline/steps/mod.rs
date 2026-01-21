// Step execution module for auto-pipeline

mod building;
mod helpers;
mod pipeline_loop;
mod planning;
mod replan;
mod verification;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;

use super::orchestrator::{DecisionResult, Orchestrator};
use super::orchestrator_agent::OrchestratorAgent;
use super::types::{AutoPipeline, StepStatus};

pub use helpers::stop_all_pipeline_agents;

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
        helpers::update_step_status(
            &self.pipelines,
            pipeline_id,
            step_number,
            status,
            app_handle,
        )
        .await;
    }

    /// Stop all agents associated with a pipeline
    pub async fn stop_all_pipeline_agents(
        &self,
        pipeline_id: &str,
        agent_manager: &Arc<Mutex<AgentManager>>,
    ) {
        helpers::stop_all_pipeline_agents(&self.pipelines, pipeline_id, agent_manager).await;
    }

    /// Execute the planning step using the OrchestratorAgent
    pub async fn execute_planning_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        planning::execute_planning_step(
            pipeline_id,
            self.pipelines.clone(),
            self.orchestrator_agents.clone(),
            &self.orchestrator,
            agent_manager,
            app_handle,
        )
        .await
    }

    /// Execute the building step using the OrchestratorAgent
    pub async fn execute_building_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        building::execute_building_step(
            pipeline_id,
            self.pipelines.clone(),
            self.orchestrator_agents.clone(),
            agent_manager,
            app_handle,
        )
        .await
    }

    /// Execute the verification step using the OrchestratorAgent
    pub async fn execute_verification_step(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<super::orchestrator_agent::OrchestratorAction, String> {
        verification::execute_verification_step(
            pipeline_id,
            self.pipelines.clone(),
            self.orchestrator_agents.clone(),
            agent_manager,
            app_handle,
        )
        .await
    }

    /// Execute the replan step when orchestrator decides to go back to planning (legacy)
    pub async fn execute_replan_step(
        &self,
        pipeline_id: &str,
        decision: &DecisionResult,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        replan::execute_replan_step(
            pipeline_id,
            self.pipelines.clone(),
            &self.orchestrator,
            decision,
            agent_manager,
            app_handle,
        )
        .await
    }

    /// Execute replan step using the OrchestratorAgent (v2)
    pub async fn execute_replan_step_v2(
        &self,
        pipeline_id: &str,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        replan::execute_replan_step_v2(
            pipeline_id,
            self.pipelines.clone(),
            self.orchestrator_agents.clone(),
            agent_manager,
            app_handle,
        )
        .await
    }

    /// Execute the full pipeline with iteration loop
    pub async fn execute_pipeline(
        &self,
        pipeline_id: String,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<(), String> {
        pipeline_loop::execute_pipeline(
            pipeline_id,
            self.pipelines.clone(),
            self.orchestrator_agents.clone(),
            &self.orchestrator,
            agent_manager,
            app_handle,
        )
        .await
    }
}
