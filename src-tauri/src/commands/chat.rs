// Chat/Meta-agent related Tauri commands

use serde::Serialize;

use crate::agent_runs_db::{ConversationQueryFilters, MetaConversationRecord};
use crate::meta_agent::CommanderPersonality;
use crate::types::{ChatMessage, ChatResponse, ImageAttachment};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct SystemPromptResponse {
    pub prompt: String,
    pub source: String,
}

#[tauri::command]
pub async fn send_chat_message(
    message: String,
    image: Option<ImageAttachment>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    // CRITICAL: Interrupt sleep BEFORE acquiring meta_agent lock
    // This allows user messages to wake up a sleeping meta-agent
    {
        let mut sleep_state = state.meta_sleep_state.lock().await;
        if sleep_state.is_sleeping {
            if let Some(cancel_tx) = sleep_state.cancel_tx.take() {
                eprintln!("[send_chat_message] Interrupting sleep with user message");
                let _ = cancel_tx.send(message.clone());
                // Don't return - the meta-agent will process the interrupted message
            }
        }
    }

    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent
        .process_user_message_with_image(message, image, state.agent_manager.clone(), app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_chat_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    let meta_agent = state.meta_agent.lock().await;
    Ok(meta_agent.get_chat_messages())
}

#[tauri::command]
pub async fn clear_chat_history(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent.clear_conversation_history();
    Ok(())
}

#[tauri::command]
pub async fn process_agent_results(
    agent_id: String,
    results_only: Option<bool>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    let results_only = results_only.unwrap_or(false);

    // Get agent outputs
    let manager = state.agent_manager.lock().await;
    let outputs = manager.get_agent_outputs(&agent_id, 0).await?;

    // Get agent info
    let agents = manager.list_agents().await;
    let agent_info = agents.iter().find(|a| a.id == agent_id);
    let agent_name = agent_info
        .map(|a| a.working_dir.clone())
        .unwrap_or_else(|| agent_id.clone());

    // Format outputs
    let mut formatted_output = if results_only {
        format!("Final results from agent in {}:\n\n", agent_name)
    } else {
        format!("Full output from agent in {}:\n\n", agent_name)
    };

    for output in outputs.iter() {
        match output.output_type.as_str() {
            "text" => {
                formatted_output.push_str(&format!("Assistant: {}\n\n", output.content));
            }
            "tool_use" => {
                // Skip tool uses if results_only mode
                if results_only {
                    continue;
                }
                // Extract tool name from content
                let tool_name = if output.content.contains("Using tool:") {
                    output.content.lines().next().unwrap_or("Unknown tool")
                } else {
                    "Using tool"
                };
                formatted_output.push_str(&format!("{}\n", tool_name));
            }
            "tool_result" => {
                // Skip tool results if results_only mode
                if results_only {
                    continue;
                }
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
                        if let Some(input_tokens) =
                            usage.get("input_tokens").and_then(|v| v.as_u64())
                        {
                            if let Some(output_tokens) =
                                usage.get("output_tokens").and_then(|v| v.as_u64())
                            {
                                formatted_output.push_str(&format!(
                                    "Tokens: {} input, {} output\n",
                                    input_tokens, output_tokens
                                ));
                            }
                        }
                    }
                }
                formatted_output.push('\n');
            }
            _ => {}
        }
    }

    drop(manager);

    // Process the formatted output as a user message through the meta agent
    let mut meta_agent = state.meta_agent.lock().await;
    let response = meta_agent
        .process_user_message(formatted_output, state.agent_manager.clone(), app_handle)
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

#[tauri::command]
pub async fn set_commander_personality(
    personality: CommanderPersonality,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    eprintln!(
        "[set_commander_personality] Updating personality: strictness={}, tone={}",
        personality.strictness, personality.tone
    );

    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent.set_personality(personality).await
}

#[tauri::command]
pub async fn get_commander_system_prompt(
    state: tauri::State<'_, AppState>,
) -> Result<SystemPromptResponse, String> {
    let meta_agent = state.meta_agent.lock().await;
    let (prompt, personalized) = meta_agent.get_system_prompt_snapshot();

    Ok(SystemPromptResponse {
        prompt,
        source: if personalized {
            "personalized".to_string()
        } else {
            "base".to_string()
        },
    })
}

#[tauri::command]
pub async fn reset_commander_personality(state: tauri::State<'_, AppState>) -> Result<(), String> {
    eprintln!("[reset_commander_personality] Clearing personality and cached prompt");

    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent.clear_personality()
}

#[tauri::command]
pub async fn answer_meta_agent_question(
    question_id: String,
    answer: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    eprintln!(
        "[answer_meta_agent_question] Answering question {}: {}",
        question_id,
        if answer.len() > 50 {
            format!("{}...", &answer[..50])
        } else {
            answer.clone()
        }
    );

    // Use the shared pending_meta_question directly from AppState
    // This avoids deadlock since we don't need to lock meta_agent
    let mut pending = state.pending_meta_question.lock().await;
    if let Some(pq) = pending.take() {
        if pq.question_id == question_id {
            // Send the answer through the oneshot channel
            pq.response_tx
                .send(answer)
                .map_err(|_| "Failed to send answer: channel closed".to_string())?;
            Ok(())
        } else {
            // Put it back if question_id doesn't match
            *pending = Some(pq);
            Err(format!(
                "Question ID mismatch: expected current pending question, got {}",
                question_id
            ))
        }
    } else {
        Err("No pending question to answer".to_string())
    }
}

// =========================================================================
// Conversation Persistence Commands
// =========================================================================

#[tauri::command]
pub async fn list_conversations(
    include_archived: Option<bool>,
    search_text: Option<String>,
    limit: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<MetaConversationRecord>, String> {
    let filters = ConversationQueryFilters {
        include_archived: include_archived.unwrap_or(false),
        search_text,
        limit,
        offset: None,
    };

    state
        .agent_runs_db
        .list_meta_conversations(filters)
        .await
        .map_err(|e| format!("Failed to list conversations: {}", e))
}

#[tauri::command]
pub async fn load_conversation(
    conversation_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent.load_conversation(&conversation_id).await
}

#[tauri::command]
pub async fn new_conversation(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent.start_new_conversation().await
}

#[tauri::command]
pub async fn delete_conversation(
    conversation_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // If this is the current conversation, clear it
    {
        let meta_agent = state.meta_agent.lock().await;
        if meta_agent.get_current_conversation_id() == Some(&conversation_id) {
            drop(meta_agent);
            let mut meta_agent = state.meta_agent.lock().await;
            meta_agent.clear_conversation_history();
        }
    }

    state
        .agent_runs_db
        .delete_meta_conversation(&conversation_id)
        .await
        .map_err(|e| format!("Failed to delete conversation: {}", e))
}

#[tauri::command]
pub async fn rename_conversation(
    conversation_id: String,
    new_title: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .agent_runs_db
        .rename_meta_conversation(&conversation_id, &new_title)
        .await
        .map_err(|e| format!("Failed to rename conversation: {}", e))
}

#[tauri::command]
pub async fn get_current_conversation_id(
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    let meta_agent = state.meta_agent.lock().await;
    Ok(meta_agent
        .get_current_conversation_id()
        .map(|s| s.to_string()))
}
