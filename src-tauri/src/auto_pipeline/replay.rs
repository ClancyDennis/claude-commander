// Replay - Structures for capturing failed run context
//
// When a pipeline fails verification, a ReplayFile is created containing
// all the context needed to restart the pipeline with knowledge of what
// went wrong.

use serde::{Deserialize, Serialize};

/// Complete context from a failed pipeline run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFile {
    /// ID of the original pipeline
    pub pipeline_id: String,
    /// Original user request
    pub original_request: String,
    /// Refined request (if any)
    pub refined_request: Option<String>,
    /// Working directory
    pub working_dir: String,
    /// Names of skills that were generated
    pub generated_skills: Vec<String>,
    /// The plan that was executed
    pub plan: Option<String>,
    /// Q&A from the planning phase
    pub questions_and_answers: Vec<(String, String)>,
    /// Execution log entries
    pub execution_log: Vec<ExecutionLogEntry>,
    /// Verification report
    pub verification_report: Option<VerificationReport>,
    /// Analysis of why the run failed
    pub failure_analysis: FailureAnalysis,
    /// Recommended changes for the next attempt
    pub recommended_changes: Vec<RecommendedChange>,
    /// Notes from the orchestrator
    pub orchestrator_notes: String,
    /// When the replay file was created
    pub created_at: String,
    /// Number of iterations completed before failure
    pub iterations_completed: u8,
}

impl ReplayFile {
    /// Create a new replay file with basic info
    pub fn new(pipeline_id: String, original_request: String, working_dir: String) -> Self {
        Self {
            pipeline_id,
            original_request,
            refined_request: None,
            working_dir,
            generated_skills: Vec::new(),
            plan: None,
            questions_and_answers: Vec::new(),
            execution_log: Vec::new(),
            verification_report: None,
            failure_analysis: FailureAnalysis::default(),
            recommended_changes: Vec::new(),
            orchestrator_notes: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            iterations_completed: 0,
        }
    }

    /// Format the replay context for inclusion in a prompt
    pub fn format_for_prompt(&self) -> String {
        let mut output = String::new();

        output.push_str("## Previous Run Context (Replay)\n\n");
        output.push_str(&format!(
            "**Original Request:** {}\n\n",
            self.original_request
        ));

        if let Some(ref refined) = self.refined_request {
            output.push_str(&format!("**Refined Request:** {}\n\n", refined));
        }

        if !self.generated_skills.is_empty() {
            output.push_str("**Skills Used:**\n");
            for skill in &self.generated_skills {
                output.push_str(&format!("- {}\n", skill));
            }
            output.push('\n');
        }

        if let Some(ref plan) = self.plan {
            output.push_str(&format!("**Previous Plan:**\n```\n{}\n```\n\n", plan));
        }

        output.push_str("**Failure Analysis:**\n");
        output.push_str(&format!(
            "- Primary Cause: {}\n",
            self.failure_analysis.primary_cause
        ));
        for factor in &self.failure_analysis.contributing_factors {
            output.push_str(&format!("- Contributing Factor: {}\n", factor));
        }
        output.push('\n');

        if !self.recommended_changes.is_empty() {
            output.push_str("**Recommended Changes:**\n");
            for change in &self.recommended_changes {
                output.push_str(&format!(
                    "- [{}] {}\n",
                    change.change_type, change.description
                ));
            }
            output.push('\n');
        }

        if !self.orchestrator_notes.is_empty() {
            output.push_str(&format!(
                "**Orchestrator Notes:** {}\n\n",
                self.orchestrator_notes
            ));
        }

        output.push_str(&format!(
            "**Iterations Completed:** {}\n",
            self.iterations_completed
        ));

        output
    }
}

/// Entry in the execution log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    /// When this entry was recorded
    pub timestamp: String,
    /// Type of log entry
    pub entry_type: ExecutionLogEntryType,
    /// Agent ID (if applicable)
    pub agent_id: Option<String>,
    /// Step number (if applicable)
    pub step_number: Option<u32>,
    /// Tool name (if applicable)
    pub tool_name: Option<String>,
    /// Input to the tool/step
    pub input: Option<String>,
    /// Output from the tool/step
    pub output: Option<String>,
    /// Whether it succeeded
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl ExecutionLogEntry {
    /// Create a step started entry
    pub fn step_started(step_number: u32, agent_id: Option<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            entry_type: ExecutionLogEntryType::StepStarted,
            agent_id,
            step_number: Some(step_number),
            tool_name: None,
            input: None,
            output: None,
            success: true,
            error: None,
            duration_ms: None,
        }
    }

    /// Create a step completed entry
    pub fn step_completed(step_number: u32, output: String, duration_ms: u64) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            entry_type: ExecutionLogEntryType::StepCompleted,
            agent_id: None,
            step_number: Some(step_number),
            tool_name: None,
            input: None,
            output: Some(output),
            success: true,
            error: None,
            duration_ms: Some(duration_ms),
        }
    }

    /// Create an error entry
    pub fn error(step_number: Option<u32>, error: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            entry_type: ExecutionLogEntryType::Error,
            agent_id: None,
            step_number,
            tool_name: None,
            input: None,
            output: None,
            success: false,
            error: Some(error),
            duration_ms: None,
        }
    }
}

/// Type of execution log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionLogEntryType {
    /// A step started
    StepStarted,
    /// A step completed
    StepCompleted,
    /// A tool was called
    ToolCall,
    /// A file was modified
    FileModified,
    /// An error occurred
    Error,
    /// A decision was made
    Decision,
    /// A checkpoint was reached
    Checkpoint,
}

/// Verification report from the verification agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    /// Overall status
    pub overall_status: VerificationStatus,
    /// Issues found
    pub issues: Vec<VerificationIssue>,
    /// Files that were reviewed
    pub files_reviewed: Vec<String>,
    /// Summary of the verification
    pub summary: String,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Verification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Success,
    Partial,
    Failed,
}

/// An issue found during verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Description of the issue
    pub description: String,
    /// File where the issue was found (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<u32>,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Severity of a verification issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

/// Analysis of why a pipeline run failed
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FailureAnalysis {
    /// Primary cause of the failure
    pub primary_cause: String,
    /// Contributing factors
    pub contributing_factors: Vec<String>,
    /// Step where the failure occurred (if identifiable)
    pub failed_at_step: Option<u32>,
    /// Whether this failure is recoverable
    pub recoverable: bool,
    /// Category of the failure
    pub failure_category: FailureCategory,
}

/// Category of failure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    /// Planning was inadequate
    PlanningIssue,
    /// Implementation had bugs
    ImplementationBug,
    /// Missing skill/capability
    MissingCapability,
    /// External dependency issue
    ExternalDependency,
    /// Configuration/environment issue
    EnvironmentIssue,
    /// Unknown or unclassified
    #[default]
    Unknown,
}

/// A recommended change for the next attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Description of the change
    pub description: String,
    /// Priority (1 = highest)
    pub priority: u8,
}

/// Type of recommended change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// Add a new skill
    AddSkill,
    /// Modify the plan
    ModifyPlan,
    /// Change the approach entirely
    ChangeApproach,
    /// Request human input
    RequestHumanInput,
    /// Fix a specific issue
    FixIssue,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::AddSkill => write!(f, "Add Skill"),
            ChangeType::ModifyPlan => write!(f, "Modify Plan"),
            ChangeType::ChangeApproach => write!(f, "Change Approach"),
            ChangeType::RequestHumanInput => write!(f, "Request Human Input"),
            ChangeType::FixIssue => write!(f, "Fix Issue"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_file_creation() {
        let replay = ReplayFile::new(
            "test-123".to_string(),
            "Build a web app".to_string(),
            "/tmp/test".to_string(),
        );

        assert_eq!(replay.pipeline_id, "test-123");
        assert_eq!(replay.original_request, "Build a web app");
        assert!(replay.generated_skills.is_empty());
    }

    #[test]
    fn test_execution_log_entry() {
        let entry = ExecutionLogEntry::step_started(1, Some("agent-1".to_string()));
        assert_eq!(entry.entry_type, ExecutionLogEntryType::StepStarted);
        assert_eq!(entry.step_number, Some(1));
        assert!(entry.success);
    }

    #[test]
    fn test_format_for_prompt() {
        let mut replay = ReplayFile::new(
            "test-123".to_string(),
            "Build a web app".to_string(),
            "/tmp/test".to_string(),
        );
        replay.failure_analysis.primary_cause = "Missing dependency".to_string();
        replay.recommended_changes.push(RecommendedChange {
            change_type: ChangeType::AddSkill,
            description: "Add npm skill".to_string(),
            priority: 1,
        });

        let prompt = replay.format_for_prompt();
        assert!(prompt.contains("Build a web app"));
        assert!(prompt.contains("Missing dependency"));
        assert!(prompt.contains("Add npm skill"));
    }
}
