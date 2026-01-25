// Tool implementations for MetaAgent

pub mod agent_tools;
pub mod data_tools;
pub mod fs_tools;
pub mod quick_actions;
pub mod ui_tools;

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;

use super::action_logger::emit_action;

/// Execute a tool by name
pub async fn execute_tool(
    tool_name: &str,
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
    queue_status_fn: impl Fn() -> crate::types::QueueStatus,
) -> Value {
    let result = match tool_name {
        "CreateWorkerAgent" => {
            agent_tools::create_worker_agent(input.clone(), agent_manager, app_handle.clone()).await
        }
        "SendPromptToWorker" => {
            agent_tools::send_prompt_to_worker(input.clone(), agent_manager, app_handle.clone())
                .await
        }
        "StopWorkerAgent" => agent_tools::stop_worker_agent(input.clone(), agent_manager).await,
        "ListWorkerAgents" => agent_tools::list_worker_agents(agent_manager).await,
        "GetAgentOutput" => {
            agent_tools::get_agent_output(input.clone(), agent_manager.clone(), app_handle.clone())
                .await
        }
        "GetAgentTodoList" => {
            agent_tools::get_agent_todo_list(input.clone(), agent_manager.clone()).await
        }
        "SearchRunHistory" => agent_tools::search_run_history(input.clone(), agent_manager).await,
        "NavigateToAgent" => ui_tools::navigate_to_agent(input.clone(), app_handle.clone()).await,
        "ToggleToolPanel" => ui_tools::toggle_tool_panel(input.clone(), app_handle.clone()).await,
        "ShowNotification" => ui_tools::show_notification(input.clone(), app_handle.clone()).await,
        "ListDirectory" => fs_tools::list_directory(input.clone()).await,
        "ShipDataToAgent" => {
            data_tools::ship_data_to_agent(input.clone(), agent_manager, app_handle.clone()).await
        }
        "CreateChainedAgent" => {
            data_tools::create_chained_agent(input.clone(), agent_manager, app_handle.clone()).await
        }
        "QuickAction" => {
            quick_actions::quick_action(
                input.clone(),
                agent_manager,
                app_handle.clone(),
                queue_status_fn,
            )
            .await
        }
        _ => json!({
            "success": false,
            "error": format!("Unknown tool: {}", tool_name)
        }),
    };

    // Emit commander:action event for the action log sidebar
    emit_action(tool_name, &input, &result, &app_handle);

    result
}
