// Tool implementations for MetaAgent

pub mod agent_tools;
pub mod fs_tools;
pub mod interaction_tools;
pub mod memory_tools;
pub mod search_tools;
pub mod todo_tools;

// Re-export interaction tool types for use in MetaAgent
pub use interaction_tools::{PendingQuestion, SleepState};

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;

use super::action_logger::emit_action;
use super::context_tracker::ContextInfo;

// ============================================================================
// Iteration Context
// ============================================================================

/// Context about the current iteration state - passed to tools so they can
/// include this info in results, helping the meta-agent know its limits.
#[derive(Debug, Clone)]
pub struct IterationContext {
    /// Current iteration number (1-based)
    pub current: usize,
    /// Maximum allowed iterations before forced stop
    pub max: usize,
    /// Context usage information (optional)
    pub context_info: Option<ContextInfo>,
}

impl IterationContext {
    pub fn new(current: usize, max: usize) -> Self {
        Self {
            current,
            max,
            context_info: None,
        }
    }

    /// Create with context info
    pub fn with_context_info(current: usize, max: usize, context_info: ContextInfo) -> Self {
        Self {
            current,
            max,
            context_info: Some(context_info),
        }
    }

    /// Remaining iterations before limit
    pub fn remaining(&self) -> usize {
        self.max.saturating_sub(self.current)
    }
}

// ============================================================================
// Tool Execution Result
// ============================================================================

/// Result of executing a tool - determines how the tool loop should proceed
#[derive(Debug)]
pub enum ToolExecutionResult {
    /// Normal tool result - continue the tool loop
    Continue(Value),
    /// Task is complete - exit the tool loop with this final message
    Complete(String),
    /// Sleep completed - continue but reset iteration counter
    SleepComplete(Value),
}

impl ToolExecutionResult {
    /// Get the underlying value for logging/events
    pub fn to_value(&self) -> Value {
        match self {
            ToolExecutionResult::Continue(v) => v.clone(),
            ToolExecutionResult::Complete(msg) => json!({
                "success": true,
                "completed": true,
                "message": msg
            }),
            ToolExecutionResult::SleepComplete(v) => v.clone(),
        }
    }

    /// Check if this result indicates Sleep completed (for iteration reset)
    pub fn is_sleep_complete(&self) -> bool {
        matches!(self, ToolExecutionResult::SleepComplete(_))
    }
}

// ============================================================================
// Tool Execution
// ============================================================================

/// Execute a tool by name
///
/// Returns a ToolExecutionResult indicating whether to continue or complete.
/// The iteration_ctx provides info about remaining iterations so tools can include
/// this in their results, helping the meta-agent manage its work within limits.
#[allow(clippy::too_many_arguments)]
pub async fn execute_tool(
    tool_name: &str,
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
    sleep_state: Arc<Mutex<SleepState>>,
    pending_question: Arc<Mutex<Option<PendingQuestion>>>,
    _queue_status_fn: impl Fn() -> crate::types::QueueStatus,
    iteration_ctx: IterationContext,
) -> ToolExecutionResult {
    let result = match tool_name {
        // =====================================================================
        // Agent Management Tools
        // =====================================================================
        "CreateWorkerAgent" => {
            let val =
                agent_tools::create_worker_agent(input.clone(), agent_manager, app_handle.clone())
                    .await;
            ToolExecutionResult::Continue(val)
        }
        "SendPromptToWorker" => {
            let val = agent_tools::send_prompt_to_worker(
                input.clone(),
                agent_manager,
                app_handle.clone(),
            )
            .await;
            ToolExecutionResult::Continue(val)
        }
        "StopWorkerAgent" => {
            let val = agent_tools::stop_worker_agent(input.clone(), agent_manager).await;
            ToolExecutionResult::Continue(val)
        }
        "ListWorkerAgents" => {
            let val = agent_tools::list_worker_agents(agent_manager).await;
            ToolExecutionResult::Continue(val)
        }
        "GetAgentOutput" => {
            let val = agent_tools::get_agent_output(
                input.clone(),
                agent_manager.clone(),
                app_handle.clone(),
            )
            .await;
            ToolExecutionResult::Continue(val)
        }
        "GetAgentTodoList" => {
            let val = agent_tools::get_agent_todo_list(input.clone(), agent_manager.clone()).await;
            ToolExecutionResult::Continue(val)
        }
        "Search" => {
            // Get runs_db from agent_manager
            let manager = agent_manager.lock().await;
            let runs_db = match &manager.runs_db {
                Some(db) => db.clone(),
                None => {
                    drop(manager);
                    return ToolExecutionResult::Continue(json!({
                        "success": false,
                        "error": "Run history database is not available"
                    }));
                }
            };
            drop(manager);
            let val = search_tools::search(input.clone(), runs_db).await;
            ToolExecutionResult::Continue(val)
        }

        // =====================================================================
        // Filesystem Tools
        // =====================================================================
        "ListDirectory" => {
            let val = fs_tools::list_directory(input.clone()).await;
            ToolExecutionResult::Continue(val)
        }

        // =====================================================================
        // Todo Tools
        // =====================================================================
        "UpdateMetaTodoList" => {
            let val = todo_tools::update_meta_todo_list(input.clone(), app_handle.clone()).await;
            ToolExecutionResult::Continue(val)
        }

        // =====================================================================
        // Memory Tools
        // =====================================================================
        "UpdateMemory" => {
            let val = memory_tools::update_memory(input.clone()).await;
            ToolExecutionResult::Continue(val)
        }

        // =====================================================================
        // User Interaction Tools
        // =====================================================================
        "Sleep" => {
            let mut val = interaction_tools::sleep_tool(
                input.clone(),
                &app_handle,
                sleep_state,
                agent_manager,
            )
            .await;
            // Add iteration reset info to sleep result
            if let Some(obj) = val.as_object_mut() {
                obj.insert("iteration_reset".to_string(), json!(true));
                obj.insert(
                    "note".to_string(),
                    json!("Your iteration counter has been reset. You now have full iterations available again."),
                );
            }
            // Return SleepComplete to signal iteration reset
            ToolExecutionResult::SleepComplete(val)
        }
        "UpdateUser" => {
            let val = interaction_tools::update_user(input.clone(), &app_handle).await;
            ToolExecutionResult::Continue(val)
        }
        "AskUserQuestion" => {
            let val =
                interaction_tools::ask_user_question(input.clone(), &app_handle, pending_question)
                    .await;
            ToolExecutionResult::Continue(val)
        }
        "CompleteTask" => {
            // Special handling - extract message and return Complete variant
            let message = input["message"]
                .as_str()
                .unwrap_or("Task completed.")
                .to_string();
            let status = input["status"].as_str().unwrap_or("success");

            // Log the completion
            eprintln!(
                "[MetaAgent] CompleteTask called with status '{}': {}",
                status,
                if message.len() > 100 {
                    format!("{}...", &message[..100])
                } else {
                    message.clone()
                }
            );

            ToolExecutionResult::Complete(message)
        }

        // =====================================================================
        // Unknown Tool
        // =====================================================================
        _ => ToolExecutionResult::Continue(json!({
            "success": false,
            "error": format!("Unknown tool: {}", tool_name)
        })),
    };

    // Emit commander:action event for the action log sidebar
    emit_action(tool_name, &input, &result.to_value(), &app_handle);

    // Add iteration context to the result (except for CompleteTask which exits anyway)
    add_iteration_info(result, iteration_ctx)
}

/// Add iteration and context info to a tool result so the meta-agent knows its limits
fn add_iteration_info(result: ToolExecutionResult, ctx: IterationContext) -> ToolExecutionResult {
    match result {
        ToolExecutionResult::Complete(msg) => {
            // CompleteTask doesn't need iteration info - we're done
            ToolExecutionResult::Complete(msg)
        }
        ToolExecutionResult::SleepComplete(mut val) => {
            // Sleep already has iteration_reset info, add the new limits
            if let Some(obj) = val.as_object_mut() {
                obj.insert("iterations_remaining".to_string(), json!(ctx.max));
                obj.insert("max_iterations".to_string(), json!(ctx.max));
                // Add context info if available
                add_context_info_to_obj(obj, &ctx.context_info);
            }
            ToolExecutionResult::SleepComplete(val)
        }
        ToolExecutionResult::Continue(mut val) => {
            // Add iteration info to help meta-agent plan
            if let Some(obj) = val.as_object_mut() {
                obj.insert("iterations_remaining".to_string(), json!(ctx.remaining()));
                obj.insert("max_iterations".to_string(), json!(ctx.max));
                // Warn if running low
                if ctx.remaining() <= 5 {
                    obj.insert(
                        "iteration_warning".to_string(),
                        json!(format!(
                            "Only {} iterations remaining! Consider using Sleep to reset the counter, or call CompleteTask soon.",
                            ctx.remaining()
                        )),
                    );
                }
                // Add context info if available
                add_context_info_to_obj(obj, &ctx.context_info);
            }
            ToolExecutionResult::Continue(val)
        }
    }
}

/// Add context usage info to a JSON object
fn add_context_info_to_obj(
    obj: &mut serde_json::Map<String, Value>,
    context_info: &Option<ContextInfo>,
) {
    if let Some(info) = context_info {
        obj.insert(
            "context_usage_percent".to_string(),
            json!(format!("{:.1}%", info.usage_percent)),
        );

        // Add warning if context is filling up
        if let Some(warning) = info.warning_message() {
            obj.insert("context_warning".to_string(), json!(warning));
        }
    }
}
