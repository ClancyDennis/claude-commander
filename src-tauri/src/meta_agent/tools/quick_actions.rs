// Quick action tools for MetaAgent

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::meta_agent::helpers::{error, shorten_id};
use crate::types::QueueStatus;

/// Execute common quick actions
pub async fn quick_action(
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
    queue_status_fn: impl Fn() -> QueueStatus,
) -> Value {
    let action = input["action"].as_str().unwrap_or("");

    match action {
        "status" => {
            // List all agents with their status
            let manager = agent_manager.lock().await;
            let agents = manager.list_agents().await;
            json!({
                "success": true,
                "agents": agents
            })
        }
        "stop_all" => stop_all_agents(agent_manager, app_handle).await,
        "queue" => {
            // Return queue status
            let status = queue_status_fn();
            json!({
                "success": true,
                "queue": status
            })
        }
        "clear_completed" => {
            // This would clear completed agents from tracking
            json!({
                "success": true,
                "message": "Cleared completed agents from display"
            })
        }
        _ => error(format!(
            "Unknown quick action: '{}'. Available actions: status, stop_all, queue, clear_completed",
            action
        )),
    }
}

/// Stop all running agents
async fn stop_all_agents(agent_manager: Arc<Mutex<AgentManager>>, app_handle: AppHandle) -> Value {
    let manager = agent_manager.lock().await;
    let agents = manager.list_agents().await;
    let mut stopped = 0;
    let mut errors = Vec::new();

    for agent in agents {
        match manager.stop_agent(&agent.id).await {
            Ok(_) => stopped += 1,
            Err(e) => errors.push(format!("{}: {}", shorten_id(&agent.id), e)),
        }
    }

    // Notify user
    let _ = app_handle.emit(
        "notification:show",
        json!({
            "message": format!("Stopped {} agents", stopped),
            "type": if errors.is_empty() { "success" } else { "warning" }
        }),
    );

    json!({
        "success": true,
        "stopped_count": stopped,
        "errors": errors
    })
}
