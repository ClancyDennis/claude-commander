// Chat/Meta-agent related Tauri commands

use crate::types::{ChatMessage, ChatResponse};
use crate::AppState;

#[tauri::command]
pub async fn send_chat_message(
    message: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent
        .process_user_message(message, state.agent_manager.clone(), app_handle)
        .await
}

#[tauri::command]
pub async fn get_chat_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    let meta_agent = state.meta_agent.lock().await;
    Ok(meta_agent.get_chat_messages())
}

#[tauri::command]
pub async fn clear_chat_history(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent.clear_conversation_history();
    Ok(())
}

#[tauri::command]
pub async fn process_agent_results(
    agent_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    // Get agent outputs
    let manager = state.agent_manager.lock().await;
    let outputs = manager.get_agent_outputs(&agent_id, 0).await?;

    // Get agent info
    let agents = manager.list_agents().await;
    let agent_info = agents.iter().find(|a| a.id == agent_id);
    let agent_name = agent_info.map(|a| a.working_dir.clone()).unwrap_or_else(|| agent_id.clone());

    // Format outputs
    let mut formatted_output = format!("Results from agent in {}:\n\n", agent_name);

    for output in outputs.iter() {
        match output.output_type.as_str() {
            "text" => {
                formatted_output.push_str(&format!("Assistant: {}\n\n", output.content));
            }
            "tool_use" => {
                // Extract tool name from content
                let tool_name = if output.content.contains("Using tool:") {
                    output.content.lines().next().unwrap_or("Unknown tool")
                } else {
                    "Using tool"
                };
                formatted_output.push_str(&format!("{}\n", tool_name));
            }
            "tool_result" => {
                // Truncate long tool results
                let truncated = if output.content.len() > 500 {
                    format!("{}...[truncated]", &output.content[..500])
                } else {
                    output.content.clone()
                };
                formatted_output.push_str(&format!("Result: {}\n\n", truncated));
            }
            "result" => {
                formatted_output.push_str("\n--- Final Results ---\n");
                if let Some(parsed) = &output.parsed_json {
                    if let Some(cost) = parsed.get("total_cost_usd").and_then(|v| v.as_f64()) {
                        formatted_output.push_str(&format!("Cost: ${:.4}\n", cost));
                    }
                    if let Some(usage) = parsed.get("usage") {
                        if let Some(input_tokens) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                            if let Some(output_tokens) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                                formatted_output.push_str(&format!("Tokens: {} input, {} output\n", input_tokens, output_tokens));
                            }
                        }
                    }
                }
                formatted_output.push_str("\n");
            }
            _ => {}
        }
    }

    drop(manager);

    // Process the formatted output as a user message through the meta agent
    let mut meta_agent = state.meta_agent.lock().await;
    let response = meta_agent.process_user_message(
        formatted_output,
        state.agent_manager.clone(),
        app_handle
    ).await?;

    Ok(response)
}
