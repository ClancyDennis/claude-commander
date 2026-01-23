// Data shipping tools for MetaAgent

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::meta_agent::helpers::{error, shorten_id};
use crate::types::AgentSource;

/// Ship data from one agent's output to another agent as context
pub async fn ship_data_to_agent(
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
) -> Value {
    let source_id = input["source_agent_id"].as_str().unwrap_or("");
    let target_id = input["target_agent_id"].as_str().unwrap_or("");
    let prompt = input["prompt_with_context"].as_str().unwrap_or("");
    let selector = input["data_selector"].as_str().unwrap_or("last_output");

    if source_id.is_empty() || target_id.is_empty() || prompt.is_empty() {
        return error("source_agent_id, target_agent_id, and prompt_with_context are required");
    }

    // Get source agent's data
    let manager = agent_manager.lock().await;

    // Determine how much output to get based on selector
    let last_n = match selector {
        "last_output" => 1,
        "all_outputs" => 0, // 0 means all
        "final_result" => 1,
        _ => 1,
    };

    let source_data = match manager.get_agent_outputs(source_id, last_n).await {
        Ok(outputs) => extract_output_data(&outputs, selector),
        Err(e) => {
            return error(format!("Failed to get source agent output: {}", e));
        }
    };

    if source_data.is_empty() {
        return error("Source agent has no output to ship");
    }

    // Construct prompt with context
    let full_prompt = format!(
        "Context from previous agent work:\n---\n{}\n---\n\n{}",
        source_data.trim(),
        prompt
    );

    // Send to target agent
    // Note: No security_monitor for meta-agent automated prompts
    match manager
        .send_prompt(target_id, &full_prompt, Some(Arc::new(app_handle)), None)
        .await
    {
        Ok(_) => json!({
            "success": true,
            "message": format!("Shipped data from {} to {}", shorten_id(source_id), shorten_id(target_id)),
            "data_length": source_data.len()
        }),
        Err(e) => error(format!("Failed to send to target agent: {}", e)),
    }
}

/// Create a new agent that receives context from an existing agent's output
pub async fn create_chained_agent(
    input: Value,
    agent_manager: Arc<Mutex<AgentManager>>,
    app_handle: AppHandle,
) -> Value {
    let source_id = input["source_agent_id"].as_str().unwrap_or("");
    let working_dir = input["working_dir"].as_str().unwrap_or("");
    let prompt = input["prompt"].as_str().unwrap_or("");

    if source_id.is_empty() || working_dir.is_empty() || prompt.is_empty() {
        return error("source_agent_id, working_dir, and prompt are required");
    }

    // Check if working directory exists
    if !std::path::Path::new(working_dir).exists() {
        return error(format!(
            "Working directory '{}' does not exist",
            working_dir
        ));
    }

    // Get source agent's output
    let manager = agent_manager.lock().await;
    let source_data = match manager.get_agent_outputs(source_id, 0).await {
        Ok(outputs) => extract_output_data(&outputs, "all_outputs"),
        Err(e) => {
            return error(format!("Failed to get source agent output: {}", e));
        }
    };

    // Create the new agent
    match manager
        .create_agent(
            working_dir.to_string(),
            None,
            None,
            AgentSource::Meta,
            Arc::new(app_handle.clone()),
        )
        .await
    {
        Ok(new_agent_id) => {
            // Construct prompt with context from source agent
            let full_prompt = if source_data.is_empty() {
                prompt.to_string()
            } else {
                format!(
                    "Context from previous agent work:\n---\n{}\n---\n\n{}",
                    source_data.trim(),
                    prompt
                )
            };

            // Send the prompt to the new agent
            // Note: No security_monitor for meta-agent automated prompts
            if let Err(e) = manager
                .send_prompt(
                    &new_agent_id,
                    &full_prompt,
                    Some(Arc::new(app_handle)),
                    None,
                )
                .await
            {
                return json!({
                    "success": true,
                    "agent_id": new_agent_id,
                    "warning": format!("Agent created but initial prompt failed: {}", e),
                    "chained_from": source_id
                });
            }

            json!({
                "success": true,
                "agent_id": new_agent_id,
                "chained_from": source_id,
                "context_length": source_data.len(),
                "status": "created and running with context"
            })
        }
        Err(e) => error(format!("Failed to create agent: {}", e)),
    }
}

/// Extract output data from agent outputs based on selector
fn extract_output_data(outputs: &[crate::types::AgentOutputEvent], selector: &str) -> String {
    let mut data = String::new();
    for output in outputs.iter() {
        match output.output_type.as_str() {
            "text" => data.push_str(&format!("{}\n", output.content)),
            "result" if selector == "final_result" || selector == "all_outputs" => {
                data.push_str(&format!("{}\n", output.content));
            }
            _ => {
                if selector == "all_outputs" {
                    data.push_str(&format!("{}\n", output.content));
                }
            }
        }
    }
    data
}
