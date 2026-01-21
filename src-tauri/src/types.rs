use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Stopped,
    Error,
    WaitingForInput,
    Idle,
    Processing,
}

impl AgentStatus {
    /// Returns the status as a lowercase string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentStatus::Running => "running",
            AgentStatus::Stopped => "stopped",
            AgentStatus::Error => "error",
            AgentStatus::WaitingForInput => "waitingforinput",
            AgentStatus::Idle => "idle",
            AgentStatus::Processing => "processing",
        }
    }
}

impl fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AgentSource {
    UI,           // Created via NewAgentDialog
    Meta,         // Created by meta agent
    Pipeline,     // Created by orchestration pipeline
    Pool,         // Created by pool (legacy)
    Manual,       // Created via API/command
}

impl AgentSource {
    /// Returns the source as a lowercase string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentSource::UI => "ui",
            AgentSource::Meta => "meta",
            AgentSource::Pipeline => "pipeline",
            AgentSource::Pool => "pool",
            AgentSource::Manual => "manual",
        }
    }
}

impl fmt::Display for AgentSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubContext {
    pub repository_url: String,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub working_dir: String,
    pub status: AgentStatus,
    pub session_id: Option<String>,
    pub last_activity: Option<i64>, // Unix timestamp in milliseconds
    pub is_processing: bool,
    pub pending_input: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_context: Option<GitHubContext>,
    pub source: AgentSource,  // Track origin of agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pooled: Option<bool>,  // Whether tracked by pool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,  // Optional display title (e.g., pipeline stage)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutputEvent {
    pub agent_id: String,
    pub output_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OutputMetadata>,

    // Enhanced fields from Claude stream-json format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<usize>,
    pub is_truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusEvent {
    pub agent_id: String,
    pub status: AgentStatus,
    pub info: Option<AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInputRequiredEvent {
    pub agent_id: String,
    pub last_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActivityEvent {
    pub agent_id: String,
    pub is_processing: bool,
    pub pending_input: bool,
    pub last_activity: i64, // Unix timestamp in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEventPayload {
    pub agent_id: String,
    pub session_id: String,
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<serde_json::Value>,

    // Enhanced fields for Phase 3
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // "pending", "success", "failed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>,
    pub timestamp: i64, // Unix timestamp in milliseconds
}

#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub session_id: String,
    pub hook_event_name: String,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatistics {
    pub agent_id: String,
    pub total_prompts: u32,
    pub total_tool_calls: u32,
    pub total_output_bytes: u64,
    pub session_start: String,
    pub last_activity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens_used: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,

    // Enhanced statistics from final result message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_usage: Option<std::collections::HashMap<String, ModelUsageStats>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_api_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_turns: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatsEvent {
    pub agent_id: String,
    pub stats: AgentStatistics,
}

// Chat-related types for meta-agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,  // "user" or "assistant"
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    pub timestamp: i64,  // Unix timestamp in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub usage: ChatUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAgentToolCallEvent {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAgentThinkingEvent {
    pub is_thinking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallStatistics {
    pub agent_id: String,
    pub total_calls: u32,
    pub successful_calls: u32,
    pub failed_calls: u32,
    pub pending_calls: u32,
    pub average_execution_time_ms: f64,
    pub calls_by_tool: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionFileInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub file_type: String,
    pub size: u64,
    pub modified: String,
}

// ============================================================================
// System Commander UX Types
// ============================================================================

/// Action log entry for the commander action sidebar
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommanderAction {
    pub action_type: String,
    pub description: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub success: bool,
}

/// Status of a queued agent result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentResultStatus {
    Pending,      // Waiting in queue
    Processing,   // Currently being analyzed
    Processed,    // Done
}

/// A queued agent result waiting to be processed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedAgentResult {
    pub agent_id: String,
    pub working_dir: String,
    pub output: String,
    pub timestamp: i64,
    pub status: AgentResultStatus,
}

impl QueuedAgentResult {
    /// Create a summary for queue display
    pub fn summary(&self) -> QueueItemSummary {
        QueueItemSummary {
            agent_id: self.agent_id.clone(),
            working_dir: self.working_dir.clone(),
            timestamp: self.timestamp,
        }
    }
}

/// Summary of a queue item for display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemSummary {
    pub agent_id: String,
    pub working_dir: String,
    pub timestamp: i64,
}

/// Overall queue status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStatus {
    pub pending: usize,
    pub items: Vec<QueueItemSummary>,
}

/// Event emitted when queue is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultQueueUpdatedEvent {
    pub queue_status: QueueStatus,
}

/// Enhanced agent activity event with current task info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentActivityDetailEvent {
    pub agent_id: String,
    pub activity: String,           // Human-readable activity: "Reading src/main.rs..."
    pub tool_name: String,          // The tool being used
    pub timestamp: i64,
}

/// Progress information for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProgress {
    pub stage: String,              // "planning", "executing", "verifying"
    pub message: String,            // Human-readable status
}

/// Enhanced status update with activity details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatusUpdateEvent {
    pub agent_id: String,
    pub status: AgentStatus,
    pub working_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_activity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<AgentProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool: Option<String>,
    pub tool_count: u32,
}
