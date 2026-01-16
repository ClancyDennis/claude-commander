// Auto-pipeline type definitions

use serde::{Deserialize, Serialize};

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
        }
    }

    /// Reset the step to pending state
    pub fn reset(&mut self) {
        self.agent_id = None;
        self.status = StepStatus::Pending;
        self.output = None;
        self.started_at = None;
        self.completed_at = None;
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
    pub fn at_max_iterations(&self) -> bool {
        self.current_iteration >= self.max_iterations
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
