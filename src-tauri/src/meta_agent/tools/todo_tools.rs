// Todo list tools for MetaAgent

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::meta_agent::helpers::error;
use crate::types::{MetaTodoItem, MetaTodoStatus, MetaTodoUpdatedEvent};

/// Update the meta-agent's todo list
pub async fn update_meta_todo_list(input: Value, app_handle: AppHandle) -> Value {
    let todos_input = match input["todos"].as_array() {
        Some(arr) => arr,
        None => {
            return error("todos array is required");
        }
    };

    // Parse todos from input
    let todos: Vec<MetaTodoItem> = todos_input
        .iter()
        .filter_map(|item| {
            let content = item["content"].as_str()?.to_string();
            let status = match item["status"].as_str()? {
                "pending" => MetaTodoStatus::Pending,
                "in_progress" => MetaTodoStatus::InProgress,
                "completed" => MetaTodoStatus::Completed,
                _ => MetaTodoStatus::Pending,
            };
            let active_form = item["activeForm"].as_str().map(|s| s.to_string());

            Some(MetaTodoItem {
                content,
                status,
                active_form,
            })
        })
        .collect();

    let timestamp = chrono::Utc::now().timestamp_millis();

    // Calculate statistics
    let total = todos.len();
    let completed = todos
        .iter()
        .filter(|t| t.status == MetaTodoStatus::Completed)
        .count();
    let in_progress = todos
        .iter()
        .filter(|t| t.status == MetaTodoStatus::InProgress)
        .count();

    // Emit event to frontend
    let event = MetaTodoUpdatedEvent {
        todos: todos.clone(),
        timestamp,
    };

    match app_handle.emit("meta-agent:todos", event) {
        Ok(_) => json!({
            "success": true,
            "message": format!("Updated todo list with {} items", total),
            "todo_count": total,
            "completed_count": completed,
            "in_progress_count": in_progress
        }),
        Err(e) => error(format!("Failed to emit todo update: {}", e)),
    }
}
