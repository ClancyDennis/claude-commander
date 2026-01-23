// Action logging for MetaAgent (commander action sidebar)

use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::types::CommanderAction;

use super::helpers::{shorten_id, shorten_message, shorten_path};

/// Emit a commander action event for the action log sidebar
pub fn emit_action(tool_name: &str, input: &Value, result: &Value, app_handle: &AppHandle) {
    let description = format_action_description(tool_name, input, result);
    let agent_id = extract_agent_id(input);
    let success = result["success"].as_bool().unwrap_or(true);
    let timestamp = chrono::Utc::now().timestamp_millis();

    let action = CommanderAction {
        action_type: tool_name.to_string(),
        description,
        timestamp,
        agent_id,
        success,
    };

    let _ = app_handle.emit("commander:action", action);
}

/// Format a human-readable description of an action
pub fn format_action_description(tool_name: &str, input: &Value, result: &Value) -> String {
    match tool_name {
        "CreateWorkerAgent" => {
            let dir = input["working_dir"].as_str().unwrap_or("unknown");
            let short_dir = shorten_path(dir, 30);
            if let Some(agent_id) = result["agent_id"].as_str() {
                format!("Created agent {} in {}", shorten_id(agent_id), short_dir)
            } else {
                format!("Created agent in {}", short_dir)
            }
        }
        "SendPromptToWorker" => {
            let agent_id = input["agent_id"].as_str().unwrap_or("?");
            format!("Sent task to agent {}", shorten_id(agent_id))
        }
        "StopWorkerAgent" => {
            let agent_id = input["agent_id"].as_str().unwrap_or("?");
            format!("Stopped agent {}", shorten_id(agent_id))
        }
        "ListWorkerAgents" => {
            let count = result["agents"].as_array().map(|a| a.len()).unwrap_or(0);
            format!("Listed {} agents", count)
        }
        "GetAgentOutput" => {
            let agent_id = input["agent_id"].as_str().unwrap_or("?");
            format!("Retrieved output from agent {}", shorten_id(agent_id))
        }
        "NavigateToAgent" => {
            let agent_id = input["agent_id"].as_str().unwrap_or("?");
            format!("Navigated to agent {}", shorten_id(agent_id))
        }
        "ToggleToolPanel" => {
            let show = input["show"].as_bool().unwrap_or(true);
            if show {
                "Showed tool panel".to_string()
            } else {
                "Hid tool panel".to_string()
            }
        }
        "ShowNotification" => {
            let msg = input["message"].as_str().unwrap_or("");
            format!("Notification: {}", shorten_message(msg, 30))
        }
        "ListDirectory" => {
            let path = input["path"].as_str().unwrap_or("?");
            format!("Listed directory {}", path)
        }
        "ShipDataToAgent" => {
            let source = input["source_agent_id"].as_str().unwrap_or("?");
            let target = input["target_agent_id"].as_str().unwrap_or("?");
            format!(
                "Shipped data {} → {}",
                shorten_id(source),
                shorten_id(target)
            )
        }
        "CreateChainedAgent" => {
            let source = input["source_agent_id"].as_str().unwrap_or("?");
            format!("Created chained agent from {}", shorten_id(source))
        }
        "QuickAction" => {
            let action = input["action"].as_str().unwrap_or("unknown");
            format!("Quick action: {}", action)
        }
        _ => format!("Executed {}", tool_name),
    }
}

/// Extract agent_id from tool input if present
pub fn extract_agent_id(input: &Value) -> Option<String> {
    input["agent_id"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| input["source_agent_id"].as_str().map(|s| s.to_string()))
        .or_else(|| input["target_agent_id"].as_str().map(|s| s.to_string()))
}
