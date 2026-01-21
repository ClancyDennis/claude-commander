// State Machine - Pipeline state tracking and transitions
//
// Defines the states a pipeline can be in and valid transitions between them.

use serde::{Deserialize, Serialize};

/// Extended state machine for the enhanced pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineState {
    // Phase A: Skill Synthesis
    /// Task received, ready to start
    ReceivedTask,
    /// Analyzing task to extract requirements
    AnalyzingTask,
    /// Selecting relevant instruction files
    SelectingInstructions,
    /// Generating skills from selected instructions
    GeneratingSkills,

    // Phase B: Planning Loop
    /// Planning agent is working
    Planning,
    /// Plan created, ready for review
    PlanReady,
    /// Plan needs revision (skills missing or issues found)
    PlanRevisionRequired,

    // Phase C: Execution
    /// Ready to start execution
    ReadyForExecution,
    /// Build agent is executing
    Executing,

    // Phase D: Verification
    /// Verification agent is reviewing
    Verifying,
    /// Verification passed
    VerificationPassed,
    /// Verification failed
    VerificationFailed,

    // Terminal states
    /// Pipeline completed successfully
    Completed,
    /// Pipeline failed
    Failed,
    /// Pipeline gave up after max iterations
    GaveUp,
}

impl PipelineState {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            PipelineState::Completed | PipelineState::Failed | PipelineState::GaveUp
        )
    }

    /// Check if this is a skill synthesis state
    pub fn is_skill_synthesis(&self) -> bool {
        matches!(
            self,
            PipelineState::ReceivedTask
                | PipelineState::AnalyzingTask
                | PipelineState::SelectingInstructions
                | PipelineState::GeneratingSkills
        )
    }

    /// Get the phase name for this state
    pub fn phase_name(&self) -> &'static str {
        match self {
            PipelineState::ReceivedTask
            | PipelineState::AnalyzingTask
            | PipelineState::SelectingInstructions
            | PipelineState::GeneratingSkills => "Skill Synthesis",
            PipelineState::Planning
            | PipelineState::PlanReady
            | PipelineState::PlanRevisionRequired => "Planning",
            PipelineState::ReadyForExecution | PipelineState::Executing => "Execution",
            PipelineState::Verifying
            | PipelineState::VerificationPassed
            | PipelineState::VerificationFailed => "Verification",
            PipelineState::Completed | PipelineState::Failed | PipelineState::GaveUp => "Terminal",
        }
    }
}

impl std::fmt::Display for PipelineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineState::ReceivedTask => write!(f, "Received Task"),
            PipelineState::AnalyzingTask => write!(f, "Analyzing Task"),
            PipelineState::SelectingInstructions => write!(f, "Selecting Instructions"),
            PipelineState::GeneratingSkills => write!(f, "Generating Skills"),
            PipelineState::Planning => write!(f, "Planning"),
            PipelineState::PlanReady => write!(f, "Plan Ready"),
            PipelineState::PlanRevisionRequired => write!(f, "Plan Revision Required"),
            PipelineState::ReadyForExecution => write!(f, "Ready for Execution"),
            PipelineState::Executing => write!(f, "Executing"),
            PipelineState::Verifying => write!(f, "Verifying"),
            PipelineState::VerificationPassed => write!(f, "Verification Passed"),
            PipelineState::VerificationFailed => write!(f, "Verification Failed"),
            PipelineState::Completed => write!(f, "Completed"),
            PipelineState::Failed => write!(f, "Failed"),
            PipelineState::GaveUp => write!(f, "Gave Up"),
        }
    }
}

/// Record of a state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// State before transition
    pub from_state: PipelineState,
    /// State after transition
    pub to_state: PipelineState,
    /// Reason for the transition
    pub reason: String,
    /// When the transition occurred
    pub timestamp: String,
}

impl StateTransition {
    pub fn new(from: PipelineState, to: PipelineState, reason: String) -> Self {
        Self {
            from_state: from,
            to_state: to,
            reason,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Valid state transitions
pub fn is_valid_transition(from: &PipelineState, to: &PipelineState) -> bool {
    use PipelineState::*;

    matches!(
        (from, to),
        // Skill synthesis flow
        (ReceivedTask, AnalyzingTask)
            | (AnalyzingTask, SelectingInstructions)
            | (SelectingInstructions, GeneratingSkills)
            | (GeneratingSkills, Planning)
            // Skip skill synthesis if no instructions
            | (ReceivedTask, Planning)
            | (SelectingInstructions, Planning)
            // Planning flow
            | (Planning, PlanReady)
            | (PlanReady, ReadyForExecution)
            | (PlanReady, PlanRevisionRequired)
            | (PlanRevisionRequired, Planning)
            // Replan from planning states (keeps skills)
            | (Planning, Planning)  // Replan during planning
            | (PlanReady, Planning) // Replan after plan generated
            // Execution flow
            | (ReadyForExecution, Executing)
            | (Executing, Verifying)
            // Verification flow - direct transitions (tools handle state directly)
            | (Verifying, Completed)           // Complete successfully
            | (Verifying, Planning)            // Replan (keeps skills)
            | (Verifying, ReadyForExecution)   // Iterate (fix implementation)
            // Intermediate states (for detailed tracking if needed)
            | (Verifying, VerificationPassed)
            | (Verifying, VerificationFailed)
            | (VerificationPassed, Completed)
            | (VerificationFailed, Planning) // Replan
            | (VerificationFailed, Executing) // Iterate
            | (VerificationFailed, GaveUp)
            // Direct failures
            | (AnalyzingTask, Failed)
            | (SelectingInstructions, Failed)
            | (GeneratingSkills, Failed)
            | (Planning, Failed)
            | (Executing, Failed)
            | (Verifying, Failed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_states() {
        assert!(PipelineState::Completed.is_terminal());
        assert!(PipelineState::Failed.is_terminal());
        assert!(PipelineState::GaveUp.is_terminal());
        assert!(!PipelineState::Executing.is_terminal());
    }

    #[test]
    fn test_skill_synthesis_states() {
        assert!(PipelineState::ReceivedTask.is_skill_synthesis());
        assert!(PipelineState::AnalyzingTask.is_skill_synthesis());
        assert!(!PipelineState::Planning.is_skill_synthesis());
    }

    #[test]
    fn test_valid_transitions() {
        assert!(is_valid_transition(
            &PipelineState::ReceivedTask,
            &PipelineState::AnalyzingTask
        ));
        assert!(is_valid_transition(
            &PipelineState::VerificationFailed,
            &PipelineState::Planning
        ));
        assert!(!is_valid_transition(
            &PipelineState::Completed,
            &PipelineState::Planning
        ));
    }

    #[test]
    fn test_phase_names() {
        assert_eq!(PipelineState::AnalyzingTask.phase_name(), "Skill Synthesis");
        assert_eq!(PipelineState::Planning.phase_name(), "Planning");
        assert_eq!(PipelineState::Executing.phase_name(), "Execution");
        assert_eq!(PipelineState::Verifying.phase_name(), "Verification");
    }
}
