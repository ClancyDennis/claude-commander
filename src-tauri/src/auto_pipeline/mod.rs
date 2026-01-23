// Auto-pipeline module
//
// Implements an iterative automated development pipeline:
// 1. Skill Synthesis - Analyze task and auto-select relevant instruction files
// 2. Planning - Creates implementation plan and generates clarifying questions
// 3. Building - Implements the solution based on the plan and Q&A
// 4. Verifying - Reviews implementation and generates verification report
// 5. Orchestrator decides: complete | iterate (back to build) | replan (back to plan)

mod agent_utils;
mod orchestrator;
pub mod prompts;
pub mod steps;
mod types;

// Skill synthesis and enhanced pipeline modules
pub mod orchestrator_agent;
pub mod orchestrator_tools;
pub mod replay;
pub mod skill_matcher;
pub mod state_machine;
pub mod task_analyzer;

pub use orchestrator::{DecisionResult, Orchestrator, OrchestratorDecision, RefinedTask};
pub use orchestrator_agent::{OrchestratorAction, OrchestratorAgent};
pub use orchestrator_tools::get_orchestrator_tools;
pub use replay::{
    ChangeType, ExecutionLogEntry, ExecutionLogEntryType, FailureAnalysis, FailureCategory,
    RecommendedChange, ReplayFile, VerificationIssue, VerificationReport, VerificationStatus,
};
pub use skill_matcher::{match_instructions, match_instructions_simple, MatchResult};
pub use state_machine::{is_valid_transition, PipelineState, StateTransition};
pub use task_analyzer::{analyze_task, TaskAnalysis};
pub use types::{
    AutoPipeline, AutoPipelineStep, EnhancedAutoPipeline, IterationRecord, StepOutput, StepRole,
    StepStatus,
};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use steps::StepExecutionContext;

/// Manager for auto-pipelines
///
/// Uses Arc internally to allow multiple pipelines to run concurrently.
/// The manager lock only needs to be held briefly for pipeline creation,
/// not during execution.
pub struct AutoPipelineManager {
    /// Shared execution context wrapped in Arc for concurrent pipeline execution
    ctx: Arc<StepExecutionContext>,
}

impl AutoPipelineManager {
    /// Create a new AutoPipelineManager with default settings
    pub fn new() -> Result<Self, String> {
        let orchestrator = Orchestrator::new()?;

        Ok(Self {
            ctx: Arc::new(StepExecutionContext {
                pipelines: Arc::new(Mutex::new(HashMap::new())),
                orchestrator,
                orchestrator_agents: Arc::new(Mutex::new(HashMap::new())),
            }),
        })
    }

    /// Create a new AutoPipelineManager with custom instructions
    pub fn with_instructions(
        custom_instructions: Option<String>,
        max_iterations: Option<u8>,
    ) -> Result<Self, String> {
        let orchestrator = Orchestrator::with_instructions(custom_instructions, max_iterations)?;

        Ok(Self {
            ctx: Arc::new(StepExecutionContext {
                pipelines: Arc::new(Mutex::new(HashMap::new())),
                orchestrator,
                orchestrator_agents: Arc::new(Mutex::new(HashMap::new())),
            }),
        })
    }

    /// Get a clone of the execution context for concurrent pipeline execution
    /// This allows running multiple pipelines without holding the manager lock
    pub fn get_ctx(&self) -> Arc<StepExecutionContext> {
        Arc::clone(&self.ctx)
    }

    /// Create a new pipeline
    pub async fn create_pipeline(
        &self,
        user_request: String,
        working_dir: String,
    ) -> Result<String, String> {
        let pipeline_id = uuid::Uuid::new_v4().to_string();
        let max_iterations = self.ctx.orchestrator.max_iterations();
        let pipeline = AutoPipeline::new(
            pipeline_id.clone(),
            user_request,
            working_dir,
            max_iterations,
        );

        let mut pipelines = self.ctx.pipelines.lock().await;
        pipelines.insert(pipeline_id.clone(), pipeline);

        Ok(pipeline_id)
    }

    /// Get a pipeline by ID
    pub async fn get_pipeline(&self, pipeline_id: &str) -> Option<AutoPipeline> {
        let pipelines = self.ctx.pipelines.lock().await;
        pipelines.get(pipeline_id).cloned()
    }
}
