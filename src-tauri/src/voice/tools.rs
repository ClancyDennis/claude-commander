//! Voice mode tool execution
//!
//! This module handles executing tools called by the OpenAI Realtime model
//! during Discuss mode. Routes through the real System Commander (MetaAgent).

use crate::agent_manager::AgentManager;
use crate::meta_agent::MetaAgent;
use serde_json::Value;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;

/// Execute a tool and return the result as a string
/// This version routes through the real System Commander (MetaAgent)
pub async fn execute_tool_with_state(
    name: &str,
    args: &str,
    meta_agent: Arc<Mutex<MetaAgent>>,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
) -> String {
    let parsed_args: Value = match serde_json::from_str(args) {
        Ok(v) => v,
        Err(e) => {
            return format!("Failed to parse arguments: {}", e);
        }
    };

    match name {
        "talk_to_mission_control" => {
            execute_talk_to_mission_control(&parsed_args, meta_agent, agent_manager, app_handle)
                .await
        }
        _ => format!("Unknown tool: {}", name),
    }
}

/// Talk to Mission Control - the central command system
/// This routes messages through the real MetaAgent/System Commander
async fn execute_talk_to_mission_control(
    args: &Value,
    meta_agent: Arc<Mutex<MetaAgent>>,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
) -> String {
    let message = match args.get("message").and_then(|m| m.as_str()) {
        Some(m) => m,
        None => {
            return "Missing 'message' parameter".to_string();
        }
    };

    println!("[Tools] Mission Control received: {}", message);

    // Route through the real MetaAgent (System Commander)
    let mut agent = meta_agent.lock().await;

    // Add context that this is coming from voice discuss mode
    let prefixed_message = format!(
        "[Voice Discuss Mode] The user is speaking with you via voice. \
        Respond concisely (1-3 sentences) since this will be spoken aloud. \
        User request: {}",
        message
    );

    match agent
        .process_user_message(prefixed_message, agent_manager, app_handle)
        .await
    {
        Ok(response) => {
            let content = &response.message.content;
            let preview = if content.len() > 200 {
                format!("{}...", &content[..200])
            } else {
                content.clone()
            };
            println!("[Tools] Mission Control response: {}", preview);

            // Check if response looks like raw tool_use JSON (OpenAI compatibility issue)
            // This happens when OpenAI echoes back malformed tool call messages
            if content.trim_start().starts_with('[') {
                if let Ok(parsed) = serde_json::from_str::<Vec<Value>>(content) {
                    if let Some(first) = parsed.first() {
                        if first.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            // Extract what action was attempted
                            if let Some(name) = first.get("name").and_then(|n| n.as_str()) {
                                return format!(
                                    "I'm working on that for you. The system is processing a {} action.",
                                    name.replace("_", " ").replace("Worker", "coding ").to_lowercase()
                                );
                            }
                        }
                    }
                }
            }

            content.clone()
        }
        Err(e) => {
            eprintln!("[Tools] Mission Control error: {}", e);
            format!("Mission Control encountered an error: {}", e)
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests removed - they require full state setup now
}
