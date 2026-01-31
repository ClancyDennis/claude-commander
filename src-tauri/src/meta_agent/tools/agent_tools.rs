// Agent management tools for MetaAgent

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::meta_agent::helpers::{error, get_optional_bool, get_optional_u64};
use crate::types::AgentSource;

/// Create a new worker agent
pub async fn create_worker_agent(
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
) -> Value {
    let working_dir = input["working_dir"].as_str().unwrap_or("");
    if working_dir.is_empty() {
        return error("Validation failed: working_dir is required. Use the ListDirectory tool to explore the filesystem and find a valid directory, or ask the user for a working directory path.");
    }

    // Check if the directory exists
    if !std::path::Path::new(working_dir).exists() {
        return error(format!(
            "Validation failed: Directory '{}' does not exist. Use the ListDirectory tool to explore available directories (e.g., ListDirectory with path '~' or '/home'), or ask the user for a valid path.",
            working_dir
        ));
    }

    let github_url = input["github_url"].as_str().map(|s| s.to_string());

    let manager = agent_manager.lock().await;
    match manager
        .create_agent(
            working_dir.to_string(),
            github_url,
            None,
            AgentSource::Meta,
            Arc::new(app_handle.clone()),
        )
        .await
    {
        Ok(agent_id) => {
            drop(manager);

            // Send initial prompt if provided
            // Note: No security_monitor for meta-agent automated prompts
            if let Some(initial_prompt) = input["initial_prompt"].as_str() {
                let manager = agent_manager.lock().await;
                if let Err(e) = manager
                    .send_prompt(
                        &agent_id,
                        initial_prompt,
                        Some(Arc::new(app_handle.clone())),
                        None,
                    )
                    .await
                {
                    return json!({
                        "success": true,
                        "agent_id": agent_id,
                        "warning": format!("Agent created but initial prompt failed: {}", e)
                    });
                }
                drop(manager);
            }

            // Navigate to agent if requested
            if get_optional_bool(&input, "navigate", false) {
                app_handle
                    .emit("agent:navigate", json!({ "agent_id": agent_id }))
                    .ok();
            }

            json!({
                "success": true,
                "agent_id": agent_id,
                "status": "created and running"
            })
        }
        Err(e) => error(format!("Failed to create agent: {}", e)),
    }
}

/// Send a prompt to an existing worker agent
pub async fn send_prompt_to_worker(
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
) -> Value {
    let agent_id = input["agent_id"].as_str().unwrap_or("");
    let prompt = input["prompt"].as_str().unwrap_or("");

    if agent_id.is_empty() || prompt.is_empty() {
        return error("agent_id and prompt are required");
    }

    let manager = agent_manager.lock().await;
    // Note: No security_monitor for meta-agent automated prompts
    match manager
        .send_prompt(agent_id, prompt, Some(Arc::new(app_handle)), None)
        .await
    {
        Ok(_) => json!({
            "success": true,
            "message": "Prompt sent successfully"
        }),
        Err(e) => error(format!("Failed to send prompt: {}", e)),
    }
}

/// Stop a worker agent
pub async fn stop_worker_agent(input: Value, agent_manager: Arc<Mutex<AgentManager>>) -> Value {
    let agent_id = input["agent_id"].as_str().unwrap_or("");

    if agent_id.is_empty() {
        return error("agent_id is required");
    }

    let manager = agent_manager.lock().await;
    match manager.stop_agent(agent_id).await {
        Ok(_) => json!({
            "success": true,
            "message": "Agent stopped successfully"
        }),
        Err(e) => error(format!("Failed to stop agent: {}", e)),
    }
}

/// List all worker agents
pub async fn list_worker_agents(agent_manager: Arc<Mutex<AgentManager>>) -> Value {
    let manager = agent_manager.lock().await;
    let agents = manager.list_agents().await;
    json!({
        "success": true,
        "agents": agents
    })
}

/// Get output from an agent
pub async fn get_agent_output(
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    _app_handle: AppHandle,
) -> Value {
    let agent_id = input["agent_id"].as_str().unwrap_or("");
    let last_n = get_optional_u64(&input, "last_n", 10) as usize;
    let filter_type = input["filter_type"].as_str().unwrap_or("text");

    if agent_id.is_empty() {
        return error("agent_id is required");
    }

    // Validate filter_type
    let valid_types = [
        "result",
        "text",
        "tool_use",
        "tool_result",
        "error",
        "most_recent",
        "all",
    ];
    if !valid_types.contains(&filter_type) {
        return error(format!(
            "Invalid filter_type '{}'. Must be one of: {}",
            filter_type,
            valid_types.join(", ")
        ));
    }

    let manager = agent_manager.lock().await;
    // Fetch all outputs first (pass 0 to get everything), then filter and apply last_n
    match manager.get_agent_outputs(agent_id, 0).await {
        Ok(outputs) => {
            // Apply filter BEFORE last_n limit (matching UI behavior)
            let filtered_outputs: Vec<_> = outputs
                .into_iter()
                .filter(|output| match filter_type {
                    "all" | "most_recent" => output.output_type != "system",
                    _ => output.output_type == filter_type,
                })
                .collect();

            // For most_recent, take only the last item; otherwise apply last_n limit AFTER filtering
            let filtered_len = filtered_outputs.len();
            let limited_outputs: Vec<_> = if filter_type == "most_recent" {
                filtered_outputs.into_iter().rev().take(1).collect()
            } else if last_n == 0 || last_n >= filtered_len {
                filtered_outputs
            } else {
                filtered_outputs
                    .into_iter()
                    .skip(filtered_len.saturating_sub(last_n))
                    .collect()
            };

            // Format outputs as readable text
            let formatted_output = format_agent_outputs(&limited_outputs);

            // Get agent info for working directory
            let agent_info = manager
                .list_agents()
                .await
                .into_iter()
                .find(|a| a.id == agent_id);
            let agent_name = agent_info
                .map(|a| a.working_dir)
                .unwrap_or_else(|| agent_id.to_string());

            let type_label = if filter_type == "all" {
                "total"
            } else {
                filter_type
            };

            json!({
                "success": true,
                "agent_id": agent_id,
                "filter_type": filter_type,
                "output_count": limited_outputs.len(),
                "outputs": formatted_output,
                "summary": format!("Retrieved {} {} outputs from agent in {}", limited_outputs.len(), type_label, agent_name)
            })
        }
        Err(e) => error(format!("Failed to get agent output: {}", e)),
    }
}

/// Get the todo/task list for an agent
pub async fn get_agent_todo_list(input: Value, agent_manager: Arc<Mutex<AgentManager>>) -> Value {
    let agent_id = input["agent_id"].as_str();

    // If no agent_id provided, get all agent todos
    if agent_id.is_none() || agent_id.unwrap().is_empty() {
        let all_todos = crate::hook_server::get_all_agent_todos().await;

        if all_todos.is_empty() {
            return json!({
                "success": true,
                "message": "No agent todo lists found. Agents may not have created task lists yet.",
                "agents": []
            });
        }

        // Get agent info for working directories
        let manager = agent_manager.lock().await;
        let agent_list = manager.list_agents().await;
        drop(manager);

        let mut agent_todos_with_info: Vec<Value> = Vec::new();
        for (id, todos) in all_todos {
            let working_dir = agent_list
                .iter()
                .find(|a| a.id == id)
                .map(|a| a.working_dir.clone())
                .unwrap_or_else(|| id.clone());

            let completed = todos.iter().filter(|t| t.status == "completed").count();
            let in_progress = todos.iter().filter(|t| t.status == "in_progress").count();
            let pending = todos.iter().filter(|t| t.status == "pending").count();
            let total = todos.len();
            let progress_pct = if total > 0 {
                (completed * 100) / total
            } else {
                0
            };

            let current_task = todos
                .iter()
                .find(|t| t.status == "in_progress")
                .map(|t| t.active_form.as_deref().unwrap_or(&t.content));

            agent_todos_with_info.push(json!({
                "agent_id": id,
                "working_dir": working_dir,
                "progress": {
                    "total": total,
                    "completed": completed,
                    "in_progress": in_progress,
                    "pending": pending,
                    "percent": progress_pct
                },
                "current_task": current_task,
                "todos": todos.iter().map(|t| json!({
                    "content": t.content,
                    "status": t.status,
                    "active_form": t.active_form
                })).collect::<Vec<_>>()
            }));
        }

        return json!({
            "success": true,
            "agent_count": agent_todos_with_info.len(),
            "agents": agent_todos_with_info
        });
    }

    // Get todos for specific agent
    let agent_id = agent_id.unwrap();
    match crate::hook_server::get_agent_todos(agent_id).await {
        Some(todos) => {
            let completed = todos.iter().filter(|t| t.status == "completed").count();
            let in_progress = todos.iter().filter(|t| t.status == "in_progress").count();
            let pending = todos.iter().filter(|t| t.status == "pending").count();
            let total = todos.len();
            let progress_pct = if total > 0 {
                (completed * 100) / total
            } else {
                0
            };

            let current_task = todos
                .iter()
                .find(|t| t.status == "in_progress")
                .map(|t| t.active_form.as_deref().unwrap_or(&t.content));

            // Get agent info
            let manager = agent_manager.lock().await;
            let agent_info = manager
                .list_agents()
                .await
                .into_iter()
                .find(|a| a.id == agent_id);
            let working_dir = agent_info
                .map(|a| a.working_dir)
                .unwrap_or_else(|| agent_id.to_string());

            json!({
                "success": true,
                "agent_id": agent_id,
                "working_dir": working_dir,
                "progress": {
                    "total": total,
                    "completed": completed,
                    "in_progress": in_progress,
                    "pending": pending,
                    "percent": progress_pct
                },
                "current_task": current_task,
                "todos": todos.iter().map(|t| json!({
                    "content": t.content,
                    "status": t.status,
                    "active_form": t.active_form
                })).collect::<Vec<_>>()
            })
        }
        None => json!({
            "success": true,
            "agent_id": agent_id,
            "message": "No todo list found for this agent. The agent may not have created a task list yet.",
            "todos": []
        }),
    }
}

/// Format agent outputs as readable text
fn format_agent_outputs(outputs: &[crate::types::AgentOutputEvent]) -> String {
    let mut formatted = String::new();

    for output in outputs.iter() {
        match output.output_type.as_str() {
            "text" => {
                formatted.push_str(&format!("Assistant: {}\n\n", output.content));
            }
            "tool_use" => {
                formatted.push_str(&format!(
                    "🔧 Using tool: {}\n",
                    output
                        .parsed_json
                        .as_ref()
                        .and_then(|j| j.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown")
                ));
            }
            "tool_result" => {
                formatted.push_str(&format!("Tool result: {}\n\n", output.content));
            }
            "result" => {
                formatted.push_str("\n--- Final Results ---\n");
                if let Some(parsed) = &output.parsed_json {
                    if let Some(cost) = parsed.get("total_cost_usd").and_then(|v| v.as_f64()) {
                        formatted.push_str(&format!("Cost: ${:.4}\n", cost));
                    }
                    if let Some(usage) = parsed.get("usage") {
                        if let Some(input_tokens) =
                            usage.get("input_tokens").and_then(|v| v.as_u64())
                        {
                            if let Some(output_tokens) =
                                usage.get("output_tokens").and_then(|v| v.as_u64())
                            {
                                formatted.push_str(&format!(
                                    "\nTokens: {} input, {} output\n",
                                    input_tokens, output_tokens
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    formatted
}
