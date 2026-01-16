// Agent runs database models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::types::AgentSource;

/// Run status - tracks the lifecycle of an agent run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Running,      // Currently active
    Completed,    // Finished successfully
    Stopped,      // Manually stopped
    Crashed,      // Process crashed or failed
    WaitingInput, // Waiting for user input
}

impl RunStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            RunStatus::Running => "running",
            RunStatus::Completed => "completed",
            RunStatus::Stopped => "stopped",
            RunStatus::Crashed => "crashed",
            RunStatus::WaitingInput => "waiting_input",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "running" => RunStatus::Running,
            "completed" => RunStatus::Completed,
            "stopped" => RunStatus::Stopped,
            "crashed" => RunStatus::Crashed,
            "waiting_input" => RunStatus::WaitingInput,
            _ => RunStatus::Crashed,
        }
    }
}

/// A record of an agent run - stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: Option<i64>,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub working_dir: String,
    pub github_url: Option<String>,
    pub github_context: Option<String>, // JSON serialized
    pub source: String,
    pub status: RunStatus,
    pub started_at: i64,         // Unix timestamp in milliseconds
    pub ended_at: Option<i64>,   // Unix timestamp in milliseconds
    pub last_activity: i64,      // Unix timestamp in milliseconds
    pub initial_prompt: Option<String>,
    pub error_message: Option<String>,

    // Statistics
    pub total_prompts: u32,
    pub total_tool_calls: u32,
    pub total_output_bytes: u64,
    pub total_tokens_used: Option<u32>,
    pub total_cost_usd: Option<f64>,
    pub model_usage: Option<String>, // JSON serialized model usage breakdown

    // Recovery information
    pub can_resume: bool,
    pub resume_data: Option<String>, // JSON serialized state for recovery
}

/// Query filters for searching runs
#[derive(Debug, Clone, Default)]
pub struct RunQueryFilters {
    pub status: Option<RunStatus>,
    pub working_dir: Option<String>,
    pub source: Option<AgentSource>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Model cost breakdown - detailed token usage per model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cost_usd: f64,
}

/// Cost summary across all agent runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_cost_usd: f64,
    pub total_sessions: usize,
    pub total_tokens: u64,
    pub total_prompts: u32,
    pub total_tool_calls: u32,
    pub session_records: Vec<SessionCostRecord>,
    pub cost_by_model: HashMap<String, f64>,
    pub cost_by_working_dir: HashMap<String, f64>,
}

/// Individual session cost record for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCostRecord {
    pub session_id: String,
    pub agent_id: String,
    pub working_dir: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub total_cost_usd: f64,
    pub total_tokens: u64,
    pub total_prompts: u32,
    pub total_tool_calls: u32,
    pub model_usage: Option<HashMap<String, ModelCostBreakdown>>,
}

/// Date range cost summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeCostSummary {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub total_cost_usd: f64,
    pub session_count: usize,
    pub daily_costs: Vec<DailyCost>,
}

/// Daily cost aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    pub date: String,
    pub cost_usd: f64,
    pub session_count: usize,
}

/// Statistics about all runs
#[derive(Debug, Serialize, Deserialize)]
pub struct RunStats {
    pub total_runs: i64,
    pub by_status: Vec<(String, i64)>,
    pub by_source: Vec<(String, i64)>,
    pub total_cost_usd: f64,
    pub resumable_runs: i64,
}

/// Database statistics including size and record counts
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub db_size_bytes: u64,
    pub db_size_formatted: String,
    pub total_runs: i64,
    pub total_prompts: i64,
    pub runs_by_status: Vec<(String, i64)>,
    pub runs_by_source: Vec<(String, i64)>,
    pub total_cost_usd: f64,
}

/// Format bytes into human readable size
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let base = 1024_f64;
    let exp = (bytes as f64).log(base).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);

    let size = bytes as f64 / base.powi(exp as i32);

    format!("{:.2} {}", size, UNITS[exp])
}
