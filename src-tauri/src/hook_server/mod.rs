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

// Re-export the handler functions for internal router use
use elevated_commands::{handle_elevated_request, handle_elevated_status, handle_scope_check};
use tool_tracking::handle_hook;

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
    let state = Arc::new(HookServerState {
        agent_manager,
        app_handle,
        pending_tools: Arc::new(Mutex::new(HashMap::new())),
        security_monitor,
        pending_elevated,
        approved_scopes,
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
