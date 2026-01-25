mod elevated_commands;
mod tool_tracking;

use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::security_monitor::SecurityMonitor;
use crate::types::PendingElevatedCommand;

// Re-export public items from submodules
pub use elevated_commands::{
    approve_elevated_request, deny_elevated_request, get_pending_elevated_commands,
};

/// Type alias for agent todo storage
type AgentTodoStorage = Arc<Mutex<HashMap<String, Vec<AgentTodoItem>>>>;

/// Global storage for agent todo lists (accessible by meta-agent tools)
static AGENT_TODOS: std::sync::OnceLock<AgentTodoStorage> = std::sync::OnceLock::new();

/// Initialize the global todo storage (called from start_hook_server)
pub fn init_agent_todos(todos: Arc<Mutex<HashMap<String, Vec<AgentTodoItem>>>>) {
    let _ = AGENT_TODOS.set(todos);
}

/// Get the todo list for a specific agent
pub async fn get_agent_todos(agent_id: &str) -> Option<Vec<AgentTodoItem>> {
    let todos = AGENT_TODOS.get()?;
    let guard = todos.lock().await;
    guard.get(agent_id).cloned()
}

/// Get all agent todo lists
pub async fn get_all_agent_todos() -> HashMap<String, Vec<AgentTodoItem>> {
    if let Some(todos) = AGENT_TODOS.get() {
        todos.lock().await.clone()
    } else {
        HashMap::new()
    }
}

// Re-export the handler functions for internal router use
use elevated_commands::{handle_elevated_request, handle_elevated_status, handle_scope_check};
use tool_tracking::handle_hook;

/// A single task/todo item from an agent's TodoWrite call
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentTodoItem {
    pub content: String,
    pub status: String, // "pending", "in_progress", "completed"
    pub active_form: Option<String>,
}

/// Shared state for the hook server
pub struct HookServerState {
    pub(crate) agent_manager: Arc<Mutex<AgentManager>>,
    pub(crate) app_handle: Arc<dyn crate::events::AppEventEmitter>,
    pub(crate) pending_tools: Arc<Mutex<HashMap<String, tool_tracking::PendingToolCall>>>,
    pub(crate) security_monitor: Option<Arc<SecurityMonitor>>,
    /// Pending elevated command requests awaiting user approval
    pub(crate) pending_elevated: Arc<Mutex<HashMap<String, PendingElevatedCommand>>>,
    /// Approved script scopes (script_hash -> expiry timestamp)
    pub(crate) approved_scopes: Arc<Mutex<HashMap<String, i64>>>,
    /// Latest todo list per agent (agent_id -> todo items)
    pub(crate) agent_todos: Arc<Mutex<HashMap<String, Vec<AgentTodoItem>>>>,
}

/// Start the hook server on the specified port
///
/// This server handles:
/// - Tool use hooks from Claude agents (PreToolUse, PostToolUse)
/// - Elevated command approval requests from wrapper scripts
pub async fn start_hook_server(
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: Arc<dyn crate::events::AppEventEmitter>,
    port: u16,
    security_monitor: Option<Arc<SecurityMonitor>>,
    pending_elevated: Arc<Mutex<HashMap<String, PendingElevatedCommand>>>,
    approved_scopes: Arc<Mutex<HashMap<String, i64>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let agent_todos = Arc::new(Mutex::new(HashMap::new()));

    // Initialize global todo storage for meta-agent access
    init_agent_todos(agent_todos.clone());

    let state = Arc::new(HookServerState {
        agent_manager,
        app_handle,
        pending_tools: Arc::new(Mutex::new(HashMap::new())),
        security_monitor,
        pending_elevated,
        approved_scopes,
        agent_todos,
    });

    let app = Router::new()
        // Tool tracking endpoint
        .route("/hook", post(handle_hook))
        // Elevation API endpoints for wrapper scripts
        .route("/elevated/request", post(handle_elevated_request))
        .route("/elevated/status/:id", get(handle_elevated_status))
        .route("/elevated/check-scope/:hash", get(handle_scope_check))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Hook server listening on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
