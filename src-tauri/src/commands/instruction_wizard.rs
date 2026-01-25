// Instruction Wizard commands - AI-assisted instruction creation and testing
//
// This module provides Tauri commands for the instruction wizard workflow:
// - Generate instruction drafts from user goals
// - Create test agents to validate instructions
// - Analyze test results for issues
// - Enhance instructions based on test feedback
//
// Helper functions have been extracted to:
// - validation_patterns.rs: Pattern constants for output analysis
// - markdown_parser.rs: Content extraction utilities
// - findings_analyzer.rs: Output analysis and recommendations
// - instruction_generator.rs: Prompt building and parsing

use crate::ai_client::{ContentBlock, Message};
use crate::types::AgentSource;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// Re-export types from extracted modules for external use
pub use super::findings_analyzer::{FindingType, TestFinding};
pub use super::instruction_generator::InstructionDraft;

// Import from extracted modules
use super::findings_analyzer::{analyze_output_for_findings, generate_recommendations};
use super::instruction_generator::{
    build_draft_system_prompt, build_draft_user_prompt, build_enhancement_system_prompt,
    build_enhancement_user_prompt, build_smart_test_instruction, parse_draft_response,
};
use super::markdown_parser::extract_markdown_content;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestAgentSession {
    pub session_id: String,
    pub agent_id: String,
    pub status: TestSessionStatus,
    pub started_at: i64,
    pub temp_instruction_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TestSessionStatus {
    Running,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestAnalysisResult {
    pub status: TestSessionStatus,
    pub duration_ms: i64,
    pub findings: Vec<TestFinding>,
    pub recommendations: Vec<String>,
    pub raw_output_summary: String,
}

// ============================================================================
// Commands
// ============================================================================

/// Generate an instruction draft from a user's goal description
#[tauri::command]
pub async fn generate_instruction_draft(
    goal_description: String,
    context: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<InstructionDraft, String> {
    if goal_description.trim().is_empty() {
        return Err("Goal description cannot be empty".to_string());
    }

    // Get the AI client from meta_agent
    let meta_agent = state.meta_agent.lock().await;
    let ai_client = meta_agent.get_ai_client();

    // Build the draft generation prompt
    let system_prompt = build_draft_system_prompt();
    let user_prompt = build_draft_user_prompt(&goal_description, context.as_deref());

    let messages = vec![Message {
        role: "user".to_string(),
        content: format!("{}\n\n{}", system_prompt, user_prompt),
    }];

    // Send to AI
    let response = ai_client
        .send_message(messages)
        .await
        .map_err(|e| format!("AI draft generation failed: {}", e))?;

    // Parse the response into structured result
    parse_draft_response(&response.content)
}

/// Create a test agent session with the draft instruction
#[tauri::command]
pub async fn create_test_agent(
    instruction_content: String,
    test_prompt: String,
    potential_requirements: Vec<String>,
    working_dir: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<TestAgentSession, String> {
    use std::fs;

    if instruction_content.trim().is_empty() {
        return Err("Instruction content cannot be empty".to_string());
    }

    // Create a temporary instruction file
    let session_id = Uuid::new_v4().to_string();
    let temp_filename = format!("wizard_test_{}.md", &session_id[..8]);

    // Determine instructions directory
    let instructions_dir = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join(".instructions");

    fs::create_dir_all(&instructions_dir)
        .map_err(|e| format!("Failed to create instructions directory: {}", e))?;

    let temp_file_path = instructions_dir.join(&temp_filename);

    // Build smart test instruction that includes setup verification
    let smart_instruction =
        build_smart_test_instruction(&instruction_content, &potential_requirements);

    // Write the smart instruction content to temp file
    fs::write(&temp_file_path, &smart_instruction)
        .map_err(|e| format!("Failed to write temp instruction file: {}", e))?;

    // Create the test agent using the AgentManager
    let agent_id = {
        let manager = state.agent_manager.lock().await;
        manager
            .create_agent_with_pipeline(
                working_dir.clone(),
                None, // No GitHub URL for test
                Some(vec![temp_filename.clone()]),
                Vec::new(), // No pre-generated skills
                AgentSource::TestWizard,
                Arc::new(app_handle.clone()),
                None,                                        // No pipeline ID
                Some(format!("Test: {}", &session_id[..8])), // Title
            )
            .await
            .map_err(|e| format!("Failed to create test agent: {}", e))?
    };

    // Send the test prompt to the agent
    {
        let manager = state.agent_manager.lock().await;
        manager
            .send_prompt(
                &agent_id,
                &test_prompt,
                Some(Arc::new(app_handle)),
                state.security_monitor.clone(),
            )
            .await
            .map_err(|e| format!("Failed to send test prompt: {}", e))?;
    }

    let session = TestAgentSession {
        session_id: session_id.clone(),
        agent_id,
        status: TestSessionStatus::Running,
        started_at: chrono::Utc::now().timestamp_millis(),
        temp_instruction_file: Some(temp_file_path.to_string_lossy().to_string()),
    };

    Ok(session)
}

/// Analyze test results from a running/completed agent
#[tauri::command]
pub async fn analyze_test_results(
    agent_id: String,
    started_at: i64,
    state: tauri::State<'_, AppState>,
) -> Result<TestAnalysisResult, String> {
    // Get agent outputs from the agent manager (handle case where agent may be gone)
    let outputs = {
        let manager = state.agent_manager.lock().await;
        manager
            .get_agent_outputs(&agent_id, 0)
            .await
            .unwrap_or_default()
    };

    // Combine all outputs for analysis
    let combined_output: String = outputs
        .iter()
        .map(|o| o.content.clone())
        .collect::<Vec<_>>()
        .join("\n");

    // Analyze the output for findings
    let findings = analyze_output_for_findings(&combined_output);
    let recommendations = generate_recommendations(&findings);

    // Calculate duration
    let duration_ms = chrono::Utc::now().timestamp_millis() - started_at;

    // Determine final status based on agent info
    let status = determine_test_status(&state, &agent_id, &findings).await;

    // Create summary
    let raw_output_summary = if combined_output.len() > 2000 {
        format!("{}...", &combined_output[..2000])
    } else {
        combined_output
    };

    Ok(TestAnalysisResult {
        status,
        duration_ms,
        findings,
        recommendations,
        raw_output_summary,
    })
}

/// Stop a running test agent and clean up
#[tauri::command]
pub async fn stop_test_agent(
    agent_id: String,
    temp_instruction_file: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use std::fs;

    // Stop the agent
    {
        let manager = state.agent_manager.lock().await;
        manager
            .stop_agent(&agent_id)
            .await
            .map_err(|e| format!("Failed to stop agent: {}", e))?;
    }

    // Clean up temp file
    if let Some(ref temp_path) = temp_instruction_file {
        let _ = fs::remove_file(temp_path);
    }

    Ok(())
}

/// Enhance instruction content based on test agent execution transcript
#[tauri::command]
pub async fn enhance_instruction_from_test(
    original_instruction: String,
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    if original_instruction.trim().is_empty() {
        return Err("Original instruction content cannot be empty".to_string());
    }

    // Get agent outputs (the full transcript)
    let outputs = {
        let manager = state.agent_manager.lock().await;
        manager
            .get_agent_outputs(&agent_id, 0)
            .await
            .unwrap_or_default()
    };

    if outputs.is_empty() {
        return Err("No test output available to enhance instruction".to_string());
    }

    // Combine all outputs into transcript
    let transcript: String = outputs
        .iter()
        .map(|o| o.content.clone())
        .collect::<Vec<_>>()
        .join("\n");

    // Get the AI client
    let meta_agent = state.meta_agent.lock().await;
    let ai_client = meta_agent.get_ai_client();

    // Build enhancement prompt
    let system_prompt = build_enhancement_system_prompt();
    let user_prompt = build_enhancement_user_prompt(&original_instruction, &transcript);

    let messages = vec![Message {
        role: "user".to_string(),
        content: format!("{}\n\n{}", system_prompt, user_prompt),
    }];

    // Send to AI
    let response = ai_client
        .send_message(messages)
        .await
        .map_err(|e| format!("AI enhancement failed: {}", e))?;

    // Extract the enhanced instruction from response
    let enhanced = response
        .content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");

    // Extract markdown content if wrapped in code blocks
    let final_content = extract_markdown_content(&enhanced);

    Ok(final_content.to_string())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Determine the test session status based on agent state and findings
async fn determine_test_status(
    state: &tauri::State<'_, AppState>,
    agent_id: &str,
    findings: &[TestFinding],
) -> TestSessionStatus {
    let manager = state.agent_manager.lock().await;

    if let Some(info) = manager.get_agent_info(agent_id).await {
        match info.status {
            crate::types::AgentStatus::Running | crate::types::AgentStatus::Processing => {
                TestSessionStatus::Running
            }
            crate::types::AgentStatus::Error => TestSessionStatus::Failed,
            crate::types::AgentStatus::Stopped => {
                // Agent was stopped - check if it was a normal stop or had issues
                if findings.iter().any(|f| f.severity == "critical") {
                    TestSessionStatus::Failed
                } else {
                    TestSessionStatus::Completed
                }
            }
            _ => {
                if findings.iter().any(|f| f.severity == "critical") {
                    TestSessionStatus::Failed
                } else {
                    TestSessionStatus::Completed
                }
            }
        }
    } else {
        // Agent not found - likely already cleaned up, assume completed
        if findings.iter().any(|f| f.severity == "critical") {
            TestSessionStatus::Failed
        } else {
            TestSessionStatus::Completed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_serialization() {
        let status = TestSessionStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"running\"");
    }

    #[test]
    fn test_test_agent_session_serialization() {
        let session = TestAgentSession {
            session_id: "test-123".to_string(),
            agent_id: "agent-456".to_string(),
            status: TestSessionStatus::Completed,
            started_at: 1234567890,
            temp_instruction_file: Some("/tmp/test.md".to_string()),
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("sessionId"));
        assert!(json.contains("agentId"));
    }
}
