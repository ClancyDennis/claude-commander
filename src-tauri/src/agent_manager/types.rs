// Agent manager types

use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
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
    /// Path to the hooks config file (for cleanup)
    pub settings_path: Option<PathBuf>,
    /// JoinHandle for stdin handler task (for cleanup)
    pub stdin_handle: Option<JoinHandle<()>>,
    /// JoinHandle for stdout stream handler task (for cleanup)
    pub stdout_handle: Option<JoinHandle<()>>,
    /// JoinHandle for stderr stream handler task (for cleanup)
    pub stderr_handle: Option<JoinHandle<()>>,
}
