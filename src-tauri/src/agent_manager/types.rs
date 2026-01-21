// Agent manager types

use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::types::{AgentInfo, AgentOutputEvent, AgentStatistics};

/// Represents a running agent process with its associated state
pub struct AgentProcess {
    pub info: AgentInfo,
    pub child: Option<Child>,
    pub stdin_tx: Option<tokio::sync::mpsc::Sender<String>>,
    pub last_activity: Arc<Mutex<Instant>>,
    pub is_processing: Arc<Mutex<bool>>,
    pub pending_input: Arc<Mutex<bool>>,
    pub stats: Arc<Mutex<AgentStatistics>>,
    pub output_buffer: Arc<Mutex<Vec<AgentOutputEvent>>>,
    pub generated_skill_names: Vec<String>,
}
