// Message handlers for system, assistant, and user messages
//
// This module handles the core message types from Claude CLI:
// system messages, assistant responses (text and tool use), and
// user messages (tool results).

use crate::types::{AgentInputRequiredEvent, AgentStatus};

use super::event_handlers::StreamContext;
use super::output_builder::{extract_common_fields, OutputEventBuilder};
use super::statistics::{increment_tool_calls, update_output_bytes};
use super::stream_parser::{persist_output, store_in_buffer};

/// Handle system messages (init, session info, etc.)
pub(crate) async fn handle_system_message(ctx: &StreamContext, json: &serde_json::Value) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

    // Extract meaningful content from system message instead of full JSON
    let content = extract_system_content(json, &subtype);

    let byte_size = content.len();
    update_output_bytes(&ctx.stats, byte_size).await;

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("system")
        .content(&content)
        .parsed_json(Some(json.clone()))
        .session_context(session_id, uuid, parent_tool_use_id, subtype)
        .with_size_from_content()
        .build();

    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());

    // Persist to database
    persist_output(
        &ctx.runs_db,
        &ctx.agent_id,
        &ctx.pipeline_id,
        "system",
        &content,
    )
    .await;
}

/// Extract content from system message JSON
fn extract_system_content(json: &serde_json::Value, subtype: &Option<String>) -> String {
    if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
        message.to_string()
    } else if let Some(init) = json.get("init") {
        // Handle init messages - extract model/tools info
        let model = init
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let tools_count = init
            .get("tools")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        format!(
            "Session initialized with {} ({} tools available)",
            model, tools_count
        )
    } else if let Some(subtype_str) = subtype {
        format!("System: {}", subtype_str)
    } else {
        "System event".to_string()
    }
}

/// Handle assistant messages (text responses, tool use)
pub(crate) async fn handle_assistant_message(ctx: &StreamContext, json: &serde_json::Value) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);
    let mut has_tool_use = false;
    let mut last_text_output = String::new();

    if let Some(message) = json.get("message") {
        if let Some(content_array) = message.get("content").and_then(|v| v.as_array()) {
            for content_block in content_array {
                if let Some(block_type) = content_block.get("type").and_then(|v| v.as_str()) {
                    match block_type {
                        "text" => {
                            handle_text_block(
                                ctx,
                                content_block,
                                &session_id,
                                &uuid,
                                &parent_tool_use_id,
                                &subtype,
                                &mut last_text_output,
                            )
                            .await;
                        }
                        "tool_use" => {
                            has_tool_use = true;
                            handle_tool_use_block(
                                ctx,
                                content_block,
                                &session_id,
                                &uuid,
                                &parent_tool_use_id,
                                &subtype,
                            )
                            .await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Update state based on stop_reason
    update_assistant_state(ctx, json, has_tool_use, last_text_output).await;
}

/// Handle text content block within assistant message
async fn handle_text_block(
    ctx: &StreamContext,
    content_block: &serde_json::Value,
    session_id: &Option<String>,
    uuid: &Option<String>,
    parent_tool_use_id: &Option<String>,
    subtype: &Option<String>,
    last_text_output: &mut String,
) {
    if let Some(text) = content_block.get("text").and_then(|v| v.as_str()) {
        *last_text_output = text.to_string();
        let byte_size = text.len();

        update_output_bytes(&ctx.stats, byte_size).await;

        let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
            .output_type("text")
            .content(text)
            .session_context(
                session_id.clone(),
                uuid.clone(),
                parent_tool_use_id.clone(),
                subtype.clone(),
            )
            .with_size_from_content()
            .build();

        store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
        let _ = ctx
            .app_handle
            .emit("agent:output", serde_json::to_value(output_event).unwrap());

        // Persist to database
        persist_output(&ctx.runs_db, &ctx.agent_id, &ctx.pipeline_id, "text", text).await;
    }
}

/// Handle tool_use content block within assistant message
async fn handle_tool_use_block(
    ctx: &StreamContext,
    content_block: &serde_json::Value,
    session_id: &Option<String>,
    uuid: &Option<String>,
    parent_tool_use_id: &Option<String>,
    subtype: &Option<String>,
) {
    *ctx.is_processing.lock().await = true;

    increment_tool_calls(&ctx.stats).await;

    let tool_name = content_block
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let tool_input = content_block
        .get("input")
        .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
        .unwrap_or_default();
    let content = format!("Using tool: {}\nInput:\n{}", tool_name, tool_input);
    let byte_size = content.len();

    update_output_bytes(&ctx.stats, byte_size).await;

    let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
        .output_type("tool_use")
        .content(&content)
        .parsed_json(content_block.get("input").cloned())
        .session_context(
            session_id.clone(),
            uuid.clone(),
            parent_tool_use_id.clone(),
            subtype.clone(),
        )
        .with_size_from_content()
        .build();

    store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
    let _ = ctx
        .app_handle
        .emit("agent:output", serde_json::to_value(output_event).unwrap());

    // Persist to database
    persist_output(
        &ctx.runs_db,
        &ctx.agent_id,
        &ctx.pipeline_id,
        "tool_use",
        &content,
    )
    .await;
}

/// Update agent state based on assistant message stop_reason
async fn update_assistant_state(
    ctx: &StreamContext,
    json: &serde_json::Value,
    has_tool_use: bool,
    last_text_output: String,
) {
    let stop_reason = json
        .get("message")
        .and_then(|msg| msg.get("stop_reason"))
        .and_then(|sr| sr.as_str())
        .unwrap_or("");

    if stop_reason == "end_turn" && !has_tool_use {
        // Agent finished its turn without tool use - waiting for user input
        *ctx.pending_input.lock().await = true;
        *ctx.is_processing.lock().await = false;

        // Update agent status to WaitingForInput
        {
            let mut agents = ctx.agents.lock().await;
            if let Some(agent) = agents.get_mut(&ctx.agent_id) {
                agent.info.status = AgentStatus::WaitingForInput;
                agent.info.pending_input = true;
            }
        }

        // Emit input required event
        let _ = ctx.app_handle.emit(
            "agent:input_required",
            serde_json::to_value(AgentInputRequiredEvent {
                agent_id: ctx.agent_id.clone(),
                last_output: last_text_output,
            })
            .unwrap(),
        );
    } else if has_tool_use {
        *ctx.pending_input.lock().await = false;
    } else {
        *ctx.is_processing.lock().await = true;
        *ctx.pending_input.lock().await = false;
    }
}

/// Handle user messages (tool results)
pub(crate) async fn handle_user_message(ctx: &StreamContext, json: &serde_json::Value) {
    let (session_id, uuid, parent_tool_use_id, subtype) = extract_common_fields(json);

    if let Some(message) = json.get("message") {
        if let Some(content_array) = message.get("content").and_then(|v| v.as_array()) {
            for content_block in content_array {
                if let Some(block_type) = content_block.get("type").and_then(|v| v.as_str()) {
                    if block_type == "tool_result" {
                        handle_tool_result_block(
                            ctx,
                            content_block,
                            &session_id,
                            &uuid,
                            &parent_tool_use_id,
                            &subtype,
                        )
                        .await;
                    }
                }
            }
        }
    }
}

/// Handle tool_result content block within user message
async fn handle_tool_result_block(
    ctx: &StreamContext,
    content_block: &serde_json::Value,
    session_id: &Option<String>,
    uuid: &Option<String>,
    parent_tool_use_id: &Option<String>,
    subtype: &Option<String>,
) {
    if let Some(content) = content_block.get("content") {
        let result_text = if let Some(s) = content.as_str() {
            s.to_string()
        } else {
            serde_json::to_string_pretty(content).unwrap_or_default()
        };
        let is_error = content_block
            .get("is_error")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let byte_size = result_text.len();
        let output_type = if is_error { "error" } else { "tool_result" };

        update_output_bytes(&ctx.stats, byte_size).await;

        let output_event = OutputEventBuilder::new(ctx.agent_id.clone())
            .output_type(output_type)
            .content(&result_text)
            .parsed_json(if !content.is_string() {
                Some(content.clone())
            } else {
                None
            })
            .session_context(
                session_id.clone(),
                uuid.clone(),
                parent_tool_use_id.clone(),
                subtype.clone(),
            )
            .with_size_from_content()
            .build();

        store_in_buffer(output_event.clone(), ctx.output_buffer.clone());
        let _ = ctx
            .app_handle
            .emit("agent:output", serde_json::to_value(output_event).unwrap());

        // Persist to database
        persist_output(
            &ctx.runs_db,
            &ctx.agent_id,
            &ctx.pipeline_id,
            output_type,
            &result_text,
        )
        .await;
    }
}
