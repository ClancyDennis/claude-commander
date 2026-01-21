use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::orchestrator::WorkflowStatus;

/// Fusion strategy for Best-of-N verification
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FusionStrategy {
    MajorityVote,       // Most common result wins
    WeightedConsensus,  // Weighted by confidence/quality scores
    MetaAgentReview,    // Meta-agent picks best
    FirstCorrect,       // First agent that passes validation
}

/// Checkpoint types for phase gates
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum CheckpointType {
    None, // No checkpoint, proceed immediately
    HumanReview, // Wait for user approval
    AutomaticValidation {
        command: String, // e.g., "cargo test", "npm run lint"
        working_dir: String,
    },
    BestOfN {
        n: usize, // Run verification with N agents
        strategy: FusionStrategy,
    },
    Conditional {
        condition: String,               // e.g., "all_tests_passed"
        on_success: Box<CheckpointType>, // Checkpoint if condition true
        on_failure: Box<CheckpointType>, // Checkpoint if condition false
    },
}

/// Phase status
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum PhaseStatus {
    Pending,           // Not started yet
    Running,           // Tasks executing
    WaitingCheckpoint, // Tasks done, waiting for checkpoint approval
    CheckpointFailed,  // Checkpoint validation failed
    Completed,         // Phase completed successfully
    Skipped,           // Phase skipped due to conditional
}

/// Checkpoint execution result
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CheckpointResult {
    pub passed: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: SystemTime,
}

impl CheckpointResult {
    /// Create a successful checkpoint result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            passed: true,
            message: message.into(),
            details: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a failed checkpoint result
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            passed: false,
            message: message.into(),
            details: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Add details to the checkpoint result
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// A phase in the pipeline with tasks and checkpoint
#[derive(Clone, Serialize, Deserialize)]
pub struct Phase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_ids: Vec<String>, // Task IDs from workflow
    pub checkpoint: CheckpointType,
    pub status: PhaseStatus,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub checkpoint_result: Option<CheckpointResult>,
}

impl Phase {
    /// Mark the phase as completed
    pub fn mark_completed(&mut self) {
        self.status = PhaseStatus::Completed;
        self.completed_at = Some(SystemTime::now());
    }

    /// Mark the phase as waiting for checkpoint
    pub fn mark_waiting_checkpoint(&mut self) {
        self.status = PhaseStatus::WaitingCheckpoint;
    }

    /// Mark the phase as failed
    pub fn mark_failed(&mut self, result: CheckpointResult) {
        self.status = PhaseStatus::CheckpointFailed;
        self.checkpoint_result = Some(result);
    }

    /// Start the phase
    pub fn start(&mut self) {
        self.status = PhaseStatus::Running;
        self.started_at = Some(SystemTime::now());
    }
}

/// Pipeline execution state using phases
#[derive(Clone, Serialize)]
pub struct Pipeline {
    pub id: String,
    pub user_request: String,
    pub phases: Vec<Phase>,
    pub current_phase_index: usize,
    pub workflow_id: Option<String>,
    pub status: WorkflowStatus,
    pub created_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub error_message: Option<String>,
    pub allow_phase_skip: bool, // Can user skip failed phases?
}

impl Pipeline {
    /// Get current phase (if any)
    pub fn current_phase(&self) -> Option<&Phase> {
        self.phases.get(self.current_phase_index)
    }

    /// Get current phase mutably (if any)
    pub fn current_phase_mut(&mut self) -> Option<&mut Phase> {
        self.phases.get_mut(self.current_phase_index)
    }

    /// Advance to the next phase
    pub fn advance_phase(&mut self) {
        self.current_phase_index += 1;
    }

    /// Mark pipeline as failed
    pub fn mark_failed(&mut self, message: impl Into<String>) {
        self.status = WorkflowStatus::Failed;
        self.error_message = Some(message.into());
    }

    /// Mark pipeline as completed
    pub fn mark_completed(&mut self) {
        self.status = WorkflowStatus::Completed;
        self.completed_at = Some(SystemTime::now());
    }

    /// Check if pipeline is finished (completed or failed)
    pub fn is_finished(&self) -> bool {
        self.status == WorkflowStatus::Completed || self.status == WorkflowStatus::Failed
    }

    /// Check if all phases are done
    pub fn all_phases_done(&self) -> bool {
        self.current_phase_index >= self.phases.len()
    }
}
