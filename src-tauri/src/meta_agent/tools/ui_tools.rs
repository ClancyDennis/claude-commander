// UI control tools for MetaAgent

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::meta_agent::helpers::{error, get_optional_bool};

/// Navigate to an agent in the UI
pub async fn navigate_to_agent(input: Value, app_handle: AppHandle) -> Value {
    let agent_id = input["agent_id"].as_str().unwrap_or("");

    if agent_id.is_empty() {
        return error("agent_id is required");
    }

    match app_handle.emit("agent:navigate", json!({ "agent_id": agent_id })) {
        Ok(_) => json!({
            "success": true,
            "message": "Navigation event emitted"
        }),
        Err(e) => error(format!("Failed to emit navigation event: {}", e)),
    }
}

/// Toggle the tool panel visibility
pub async fn toggle_tool_panel(input: Value, app_handle: AppHandle) -> Value {
    let show = get_optional_bool(&input, "show", true);

    match app_handle.emit("tool-panel:toggle", json!({ "show": show })) {
        Ok(_) => json!({
            "success": true,
            "message": if show { "Tool panel shown" } else { "Tool panel hidden" }
        }),
        Err(e) => error(format!("Failed to toggle tool panel: {}", e)),
    }
}

/// Show a notification to the user
pub async fn show_notification(input: Value, app_handle: AppHandle) -> Value {
    let message = input["message"].as_str().unwrap_or("");
    let notification_type = input["type"].as_str().unwrap_or("info");

    if message.is_empty() {
        return error("message is required");
    }

    match app_handle.emit(
        "notification:show",
        json!({
            "message": message,
            "type": notification_type
        }),
    ) {
        Ok(_) => json!({
            "success": true,
            "message": "Notification emitted"
        }),
        Err(e) => error(format!("Failed to show notification: {}", e)),
    }
}
