// Agent utility functions for auto-pipeline

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::types::AgentStatus;

use super::types::StepOutput;

/// Extract JSON from markdown code blocks (```json ... ``` or ``` ... ```)
pub fn extract_json_from_markdown(text: &str) -> String {
    // Check for ```json block
    if let Some(start_idx) = text.find("```json") {
        if let Some(content_start) = text[start_idx..].find('\n') {
            let content_start = start_idx + content_start + 1;
            if let Some(end_idx) = text[content_start..].find("```") {
                return text[content_start..content_start + end_idx].trim().to_string();
            }
        }
    }

    // Check for generic ``` block
    if let Some(start_idx) = text.find("```") {
        if let Some(content_start) = text[start_idx..].find('\n') {
            let content_start = start_idx + content_start + 1;
            if let Some(end_idx) = text[content_start..].find("```") {
                return text[content_start..content_start + end_idx].trim().to_string();
            }
        }
    }

    text.to_string()
}

/// Wait for an agent to complete (reach WaitingForInput, Stopped, or Error status)
pub async fn wait_for_agent_completion(
    agent_id: &str,
    agent_manager: Arc<Mutex<AgentManager>>,
) -> Result<(), String> {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let manager = agent_manager.lock().await;
        let agents_list = manager.list_agents().await;

        if let Some(agent) = agents_list.iter().find(|a| a.id == agent_id) {
            match agent.status {
                AgentStatus::WaitingForInput => break,
                AgentStatus::Stopped => break,
                AgentStatus::Error => {
                    return Err("Agent encountered an error".to_string());
                }
                _ => continue,
            }
        } else {
            return Err("Agent not found".to_string());
        }
    }

    Ok(())
}

/// Extract output from an agent's result
pub async fn extract_agent_output(
    agent_id: &str,
    agent_manager: Arc<Mutex<AgentManager>>,
) -> Result<StepOutput, String> {
    let manager = agent_manager.lock().await;
    let outputs = manager.get_agent_outputs(agent_id, 100).await?;

    // Try to find a result message first
    if let Some(result_output) = outputs.iter().rev().find(|o| o.output_type == "result") {
        let mut last_text = result_output
            .parsed_json
            .as_ref()
            .and_then(|json| json.get("result").and_then(|r| r.as_str()))
            .map(String::from)
            .unwrap_or_else(|| result_output.content.clone());

        last_text = extract_json_from_markdown(&last_text);
        let structured_data = serde_json::from_str(&last_text).ok();

        return Ok(StepOutput {
            raw_text: last_text,
            structured_data,
        });
    }

    // Fallback to text output
    let mut last_text = outputs
        .iter()
        .rev()
        .find(|o| o.output_type == "text")
        .map(|o| o.content.clone())
        .unwrap_or_default();

    last_text = extract_json_from_markdown(&last_text);
    let structured_data = serde_json::from_str(&last_text).ok();

    Ok(StepOutput {
        raw_text: last_text,
        structured_data,
    })
}
