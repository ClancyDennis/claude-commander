// Event handlers module - coordinates message handling
//
// This module defines the StreamContext and re-exports handlers from
// specialized sub-modules for different message types.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::agent_runs_db::AgentRunsDB;
use crate::types::{AgentOutputEvent, AgentStatistics};

use super::types::AgentProcess;

// Re-export handlers from sub-modules
pub(crate) use super::message_handlers::{
    handle_assistant_message, handle_system_message, handle_user_message,
};
pub(crate) use super::result_handlers::{
    handle_plain_text, handle_process_end, handle_result_message, handle_stream_event,
    handle_unknown_message,
};

/// Context for stream handling - passed to all event handlers
///
/// This struct contains all the shared state and resources needed by
/// event handlers to process messages from the Claude CLI process.
pub struct StreamContext {
    /// Unique identifier for this agent
    pub agent_id: String,

    /// Map of all active agents, keyed by agent_id
    pub agents: Arc<Mutex<HashMap<String, AgentProcess>>>,

    /// Maps session IDs to agent IDs for lookup
    pub session_map: Arc<Mutex<HashMap<String, String>>>,

    /// Handle for emitting events to the frontend
    pub app_handle: Arc<dyn crate::events::AppEventEmitter>,

    /// Timestamp of last activity for this agent
    pub last_activity: Arc<Mutex<Instant>>,

    /// Whether the agent is currently processing
    pub is_processing: Arc<Mutex<bool>>,

    /// Whether the agent is waiting for user input
    pub pending_input: Arc<Mutex<bool>>,

    /// Statistics for this agent session
    pub stats: Arc<Mutex<AgentStatistics>>,

    /// Buffer of recent output events
    pub output_buffer: Arc<Mutex<Vec<AgentOutputEvent>>>,

    /// Database for persisting agent runs
    pub runs_db: Option<Arc<AgentRunsDB>>,

    /// Optional pipeline ID if this agent is part of a pipeline
    pub pipeline_id: Option<String>,
}
