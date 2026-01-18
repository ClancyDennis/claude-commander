// Agent management tools for MetaAgent

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::meta_agent::helpers::{error, get_optional_bool, get_optional_u64, shorten_id};
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
            if let Some(initial_prompt) = input["initial_prompt"].as_str() {
                let manager = agent_manager.lock().await;
                if let Err(e) = manager
                    .send_prompt(&agent_id, initial_prompt, Some(Arc::new(app_handle.clone())))
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
    match manager
        .send_prompt(agent_id, prompt, Some(Arc::new(app_handle)))
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
                        if let Some(input_tokens) = usage.get("input_tokens").and_then(|v| v.as_u64())
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
