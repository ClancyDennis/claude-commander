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

    if agent_id.is_empty() {
        return error("agent_id is required");
    }

    let manager = agent_manager.lock().await;
    match manager.get_agent_outputs(agent_id, last_n).await {
        Ok(outputs) => {
            // Format outputs as readable text
            let formatted_output = format_agent_outputs(&outputs);

            // Get agent info for working directory
            let agent_info = manager
                .list_agents()
                .await
                .into_iter()
                .find(|a| a.id == agent_id);
            let agent_name = agent_info
                .map(|a| a.working_dir)
                .unwrap_or_else(|| agent_id.to_string());

            json!({
                "success": true,
                "agent_id": agent_id,
                "output_count": outputs.len(),
                "outputs": formatted_output,
                "summary": format!("Retrieved {} outputs from agent in {}", outputs.len(), agent_name)
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

/// Search through historical agent runs
pub async fn search_run_history(input: Value, agent_manager: Arc<Mutex<AgentManager>>) -> Value {
    use chrono::{Duration, Utc};
    use crate::agent_runs_db::{RunQueryFilters, RunStatus};

    let manager = agent_manager.lock().await;
    let runs_db = match &manager.runs_db {
        Some(db) => db.clone(),
        None => {
            return error("Run history database is not available");
        }
    };
    drop(manager);

    // Build query filters from input
    let days_back = input["days_back"].as_i64().unwrap_or(30);
    let limit = input["limit"].as_u64().unwrap_or(20) as usize;

    let status = input["status"].as_str().map(RunStatus::parse);
    // working_dir filtering done in post-processing for partial match support
    let working_dir_filter = input["working_dir"].as_str().map(|s| s.to_lowercase());
    let source = input["source"].as_str().and_then(|s| {
        match s {
            "ui" => Some(AgentSource::UI),
            "meta" => Some(AgentSource::Meta),
            "pipeline" => Some(AgentSource::Pipeline),
            "pool" => Some(AgentSource::Pool),
            "manual" => Some(AgentSource::Manual),
            _ => None,
        }
    });

    let date_from = Some(Utc::now() - Duration::days(days_back));

    let filters = RunQueryFilters {
        status,
        working_dir: None, // Do partial matching in post-processing
        source,
        date_from,
        date_to: None,
        limit: Some(limit + 100), // Get extra for post-filtering
        offset: None,
    };

    // Query the database
    let runs = match runs_db.query_runs(filters).await {
        Ok(runs) => runs,
        Err(e) => {
            return error(format!("Failed to query run history: {}", e));
        }
    };

    // Post-filter by keyword, working_dir partial match, and resumable
    let keyword = input["keyword"].as_str().map(|s| s.to_lowercase());
    let resumable_only = input["resumable_only"].as_bool().unwrap_or(false);

    let filtered_runs: Vec<_> = runs
        .into_iter()
        .filter(|run| {
            // Filter by working_dir (partial match)
            if let Some(ref dir_filter) = working_dir_filter {
                if !run.working_dir.to_lowercase().contains(dir_filter) {
                    return false;
                }
            }
            // Filter by keyword in initial_prompt
            if let Some(ref kw) = keyword {
                if let Some(ref prompt) = run.initial_prompt {
                    if !prompt.to_lowercase().contains(kw) {
                        return false;
                    }
                } else {
                    return false; // No prompt to search
                }
            }
            // Filter by resumable
            if resumable_only && !run.can_resume {
                return false;
            }
            true
        })
        .take(limit)
        .collect();

    // Format results for the AI
    let formatted_runs: Vec<Value> = filtered_runs
        .iter()
        .map(|run| {
            // Calculate duration if ended
            let duration_mins = run.ended_at.map(|ended| {
                ((ended - run.started_at) / 60000) as i64 // Convert ms to minutes
            });

            // Format timestamp as readable string
            let started_at = chrono::DateTime::from_timestamp_millis(run.started_at)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Truncate initial_prompt for display
            let initial_prompt: Option<String> = run.initial_prompt.as_ref().map(|p: &String| {
                if p.len() > 100 {
                    format!("{}...", &p[..97])
                } else {
                    p.clone()
                }
            });

            json!({
                "agent_id": run.agent_id,
                "working_dir": run.working_dir,
                "status": run.status.to_str(),
                "source": run.source,
                "started_at": started_at,
                "duration_mins": duration_mins,
                "initial_prompt": initial_prompt,
                "total_prompts": run.total_prompts,
                "total_tool_calls": run.total_tool_calls,
                "cost_usd": run.total_cost_usd,
                "can_resume": run.can_resume
            })
        })
        .collect();

    json!({
        "success": true,
        "total_found": formatted_runs.len(),
        "runs": formatted_runs
    })
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
