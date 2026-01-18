// Auto-pipeline type definitions

use serde::{Deserialize, Serialize};

use super::replay::ReplayFile;
use super::skill_matcher::MatchResult;
use super::state_machine::{PipelineState, StateTransition};
use super::task_analyzer::TaskAnalysis;

/// Role of a pipeline step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepRole {
    Planning,
    Building,
    Verifying,
}

/// Status of a pipeline step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Output from a pipeline step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    pub raw_text: String,
    pub structured_data: Option<serde_json::Value>,
    #[serde(default)]
    pub agent_outputs: Vec<crate::types::AgentOutputEvent>,
}

/// A single step in the auto-pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPipelineStep {
    pub step_number: u8,
    pub role: StepRole,
    pub agent_id: Option<String>,
    pub status: StepStatus,
    pub output: Option<StepOutput>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    /// Count of tools used in this step (tracked in real-time via frontend)
    #[serde(default)]
    pub tool_count: u32,
}

impl AutoPipelineStep {
    /// Create a new step with the given role
    pub fn new(step_number: u8, role: StepRole) -> Self {
        Self {
            step_number,
            role,
            agent_id: None,
            status: StepStatus::Pending,
            output: None,
            started_at: None,
            completed_at: None,
            tool_count: 0,
        }
    }

    /// Reset the step to pending state
    pub fn reset(&mut self) {
        self.agent_id = None;
        self.status = StepStatus::Pending;
        self.output = None;
        self.started_at = None;
        self.completed_at = None;
        self.tool_count = 0;
    }
}

/// Record of a single iteration in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    pub iteration: u8,
    pub decision: String,
    pub reasoning: String,
    pub issues: Vec<String>,
}

/// The main auto-pipeline state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPipeline {
    pub id: String,
    pub user_request: String,
    pub refined_request: Option<String>,
    pub working_dir: String,
    pub status: String,
    pub steps: [AutoPipelineStep; 3],
    pub created_at: String,
    pub completed_at: Option<String>,
    pub questions: Vec<String>,
    pub answers: Vec<String>,
    // Iteration tracking
    pub current_iteration: u8,
    pub max_iterations: u8,
    pub iteration_history: Vec<IterationRecord>,
    pub final_decision: Option<String>,
}

impl AutoPipeline {
    /// Create a new pipeline with the given parameters
    pub fn new(id: String, user_request: String, working_dir: String, max_iterations: u8) -> Self {
        Self {
            id,
            user_request,
            refined_request: None,
            working_dir,
            status: "running".to_string(),
            steps: [
                AutoPipelineStep::new(1, StepRole::Planning),
                AutoPipelineStep::new(2, StepRole::Building),
                AutoPipelineStep::new(3, StepRole::Verifying),
            ],
            questions: Vec::new(),
            answers: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            current_iteration: 1,
            max_iterations,
            iteration_history: Vec::new(),
            final_decision: None,
        }
    }

    /// Reset steps for a new iteration (keeps planning, resets build/verify)
    pub fn reset_for_iteration(&mut self) {
        self.current_iteration += 1;
        self.steps[1].reset();
        self.steps[2].reset();
    }

    /// Reset all steps for replanning
    pub fn reset_for_replan(&mut self) {
        self.current_iteration += 1;
        for step in &mut self.steps {
            step.reset();
        }
    }

    /// Mark the pipeline as completed with a final decision
    pub fn mark_completed(&mut self, decision: &str) {
        self.status = "completed".to_string();
        self.final_decision = Some(decision.to_string());
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mark the pipeline as failed with a final decision
    pub fn mark_failed(&mut self, decision: &str) {
        self.status = "failed".to_string();
        self.final_decision = Some(decision.to_string());
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Check if the pipeline has reached max iterations
    /// Note: max_iterations of 0 means unlimited iterations (never returns true)
    pub fn at_max_iterations(&self) -> bool {
        self.max_iterations > 0 && self.current_iteration > self.max_iterations
    }

    /// Get Q&A formatted as string
    pub fn format_qna(&self) -> String {
        self.questions
            .iter()
            .zip(&self.answers)
            .map(|(q, a)| format!("Q: {}\nA: {}", q, a))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get plan output text (from planning step)
    pub fn get_plan_output(&self) -> Option<&str> {
        self.steps[0].output.as_ref().map(|o| o.raw_text.as_str())
    }

    /// Get build output text (from building step)
    pub fn get_build_output(&self) -> Option<&str> {
        self.steps[1].output.as_ref().map(|o| o.raw_text.as_str())
    }

    /// Get verification output text (from verification step)
    pub fn get_verification_output(&self) -> Option<&str> {
        self.steps[2].output.as_ref().map(|o| o.raw_text.as_str())
    }
}

// ============================================================================
// Enhanced Auto-Pipeline - Extended version with skill synthesis
// ============================================================================

/// Enhanced auto-pipeline with skill synthesis and state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAutoPipeline {
    // Core identification
    pub id: String,
    pub user_request: String,
    pub refined_request: Option<String>,
    pub working_dir: String,

    // State machine tracking
    pub state: PipelineState,
    pub state_history: Vec<StateTransition>,

    // Skill synthesis results
    pub task_analysis: Option<TaskAnalysis>,
    pub matched_instructions: Vec<MatchResult>,
    pub generated_skills: Vec<String>,

    // Legacy step tracking (for compatibility)
    pub steps: [AutoPipelineStep; 3],

    // Q&A
    pub questions: Vec<String>,
    pub answers: Vec<String>,

    // Iteration tracking
    pub current_iteration: u8,
    pub max_iterations: u8,
    pub iteration_history: Vec<IterationRecord>,

    // Replay context (for failed runs)
    pub replay_context: Option<ReplayFile>,

    // Timestamps and status
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub final_decision: Option<String>,
}

impl EnhancedAutoPipeline {
    /// Create a new enhanced pipeline
    pub fn new(id: String, user_request: String, working_dir: String, max_iterations: u8) -> Self {
        Self {
            id,
            user_request,
            refined_request: None,
            working_dir,
            state: PipelineState::ReceivedTask,
            state_history: Vec::new(),
            task_analysis: None,
            matched_instructions: Vec::new(),
            generated_skills: Vec::new(),
            steps: [
                AutoPipelineStep::new(1, StepRole::Planning),
                AutoPipelineStep::new(2, StepRole::Building),
                AutoPipelineStep::new(3, StepRole::Verifying),
            ],
            questions: Vec::new(),
            answers: Vec::new(),
            current_iteration: 1,
            max_iterations,
            iteration_history: Vec::new(),
            replay_context: None,
            status: "running".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            final_decision: None,
        }
    }

    /// Create from a replay file (for restart)
    pub fn from_replay(replay: ReplayFile, max_iterations: u8) -> Self {
        let mut pipeline = Self::new(
            uuid::Uuid::new_v4().to_string(),
            replay.original_request.clone(),
            replay.working_dir.clone(),
            max_iterations,
        );
        pipeline.refined_request = replay.refined_request.clone();
        pipeline.replay_context = Some(replay);
        pipeline
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: PipelineState, reason: String) {
        let transition = StateTransition::new(self.state.clone(), new_state.clone(), reason);
        self.state_history.push(transition);
        self.state = new_state;
    }

    /// Set task analysis results
    pub fn set_task_analysis(&mut self, analysis: TaskAnalysis) {
        self.task_analysis = Some(analysis);
    }

    /// Set matched instructions
    pub fn set_matched_instructions(&mut self, matches: Vec<MatchResult>) {
        self.matched_instructions = matches;
    }

    /// Add a generated skill name
    pub fn add_generated_skill(&mut self, skill_name: String) {
        self.generated_skills.push(skill_name);
    }

    /// Reset steps for a new iteration (keeps planning, resets build/verify)
    pub fn reset_for_iteration(&mut self) {
        self.current_iteration += 1;
        self.steps[1].reset();
        self.steps[2].reset();
    }

    /// Reset all steps for replanning
    pub fn reset_for_replan(&mut self) {
        self.current_iteration += 1;
        for step in &mut self.steps {
            step.reset();
        }
    }

    /// Mark the pipeline as completed
    pub fn mark_completed(&mut self, decision: &str) {
        self.status = "completed".to_string();
        self.final_decision = Some(decision.to_string());
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
        self.transition_to(PipelineState::Completed, decision.to_string());
    }

    /// Mark the pipeline as failed
    pub fn mark_failed(&mut self, decision: &str) {
        self.status = "failed".to_string();
        self.final_decision = Some(decision.to_string());
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
        self.transition_to(PipelineState::Failed, decision.to_string());
    }

    /// Check if at max iterations
    /// Note: max_iterations of 0 means unlimited iterations (never returns true)
    pub fn at_max_iterations(&self) -> bool {
        self.max_iterations > 0 && self.current_iteration > self.max_iterations
    }

    /// Format Q&A for prompts
    pub fn format_qna(&self) -> String {
        self.questions
            .iter()
            .zip(&self.answers)
            .map(|(q, a)| format!("Q: {}\nA: {}", q, a))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get plan output
    pub fn get_plan_output(&self) -> Option<&str> {
        self.steps[0].output.as_ref().map(|o| o.raw_text.as_str())
    }

    /// Get build output
    pub fn get_build_output(&self) -> Option<&str> {
        self.steps[1].output.as_ref().map(|o| o.raw_text.as_str())
    }

    /// Get verification output
    pub fn get_verification_output(&self) -> Option<&str> {
        self.steps[2].output.as_ref().map(|o| o.raw_text.as_str())
    }

    /// Convert to legacy AutoPipeline (for backward compatibility)
    pub fn to_legacy(&self) -> AutoPipeline {
        AutoPipeline {
            id: self.id.clone(),
            user_request: self.user_request.clone(),
            refined_request: self.refined_request.clone(),
            working_dir: self.working_dir.clone(),
            status: self.status.clone(),
            steps: self.steps.clone(),
            created_at: self.created_at.clone(),
            completed_at: self.completed_at.clone(),
            questions: self.questions.clone(),
            answers: self.answers.clone(),
            current_iteration: self.current_iteration,
            max_iterations: self.max_iterations,
            iteration_history: self.iteration_history.clone(),
            final_decision: self.final_decision.clone(),
        }
    }
}
