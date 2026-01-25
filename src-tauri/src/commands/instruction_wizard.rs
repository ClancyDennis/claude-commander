// Instruction Wizard commands - AI-assisted instruction creation and testing

use crate::ai_client::{ContentBlock, Message};
use crate::types::AgentSource;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionDraft {
    pub content: String,
    pub suggested_filename: String,
    pub suggested_test_prompt: String,
    pub potential_requirements: Vec<String>,
    pub complexity: String, // "simple", "moderate", "complex"
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFinding {
    pub id: String,
    pub finding_type: FindingType,
    pub severity: String, // "critical", "warning", "info"
    pub title: String,
    pub description: String,
    pub resolution_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FindingType {
    MissingTool,
    AuthRequired,
    PermissionDenied,
    EnvironmentSetup,
    InstructionAmbiguity,
    SuccessPattern,
    Other,
}

// Internal type for parsing AI draft response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AIDraftResponse {
    instruction_content: String,
    suggested_filename: String,
    test_prompt: String,
    potential_requirements: Vec<String>,
    complexity: String,
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

    // Write the instruction content to temp file
    fs::write(&temp_file_path, &instruction_content)
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
        manager.get_agent_outputs(&agent_id, 0).await.unwrap_or_default()
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
    let status = {
        let manager = state.agent_manager.lock().await;
        if let Some(info) = manager.get_agent_info(&agent_id).await {
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
    };

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

// ============================================================================
// Helper Functions
// ============================================================================

fn build_draft_system_prompt() -> String {
    r#"You are an expert at writing instruction files for AI coding agents like Claude Code.
Your task is to generate a clear, actionable instruction document based on the user's goal.

The instruction should include:
1. A clear title and description
2. Setup steps (authentication, API keys, tool installation if needed)
3. Usage instructions showing how the agent should perform the task
4. Example commands or code patterns
5. Common issues and troubleshooting tips

Focus on:
- Being specific about required credentials, tokens, or API keys
- Including exact commands or steps to set up dependencies
- Providing practical usage examples
- Anticipating common issues and how to resolve them

Respond ONLY with valid JSON matching this schema (no markdown code blocks, just raw JSON):
{
  "instructionContent": "Full markdown instruction content...",
  "suggestedFilename": "descriptive-name.md",
  "testPrompt": "A simple prompt to test this instruction works, e.g. 'List my Google Drive files'",
  "potentialRequirements": ["List of things that may need setup, e.g. 'Google OAuth credentials', 'gh CLI installed'"],
  "complexity": "simple" | "moderate" | "complex"
}"#
    .to_string()
}

fn build_draft_user_prompt(goal: &str, context: Option<&str>) -> String {
    let context_section = context
        .map(|c| format!("\n\nAdditional context provided by the user:\n{}", c))
        .unwrap_or_default();

    format!(
        "Generate an instruction document for the following goal:{}

User's Goal: {}

Remember: Respond with raw JSON only, no markdown formatting.",
        context_section, goal
    )
}

fn parse_draft_response(content: &[ContentBlock]) -> Result<InstructionDraft, String> {
    // Extract text content from response
    let text = content
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

    // Try to parse JSON from the response
    let json_str = extract_json(&text);

    let ai_response: AIDraftResponse = serde_json::from_str(json_str.trim()).map_err(|e| {
        format!(
            "Failed to parse AI response as JSON: {}. Response was: {}",
            e,
            &text[..text.len().min(500)]
        )
    })?;

    Ok(InstructionDraft {
        content: ai_response.instruction_content,
        suggested_filename: ai_response.suggested_filename,
        suggested_test_prompt: ai_response.test_prompt,
        potential_requirements: ai_response.potential_requirements,
        complexity: ai_response.complexity,
    })
}

fn extract_json(text: &str) -> &str {
    // Try to find JSON in markdown code blocks first
    if let Some(start) = text.find("```json") {
        if let Some(end) = text[start + 7..].find("```") {
            return &text[start + 7..start + 7 + end];
        }
    }

    // Try generic code blocks
    if let Some(start) = text.find("```") {
        let after_start = start + 3;
        let content_start = text[after_start..]
            .find('\n')
            .map(|n| after_start + n + 1)
            .unwrap_or(after_start);

        if let Some(end) = text[content_start..].find("```") {
            return &text[content_start..content_start + end];
        }
    }

    // Try to find raw JSON (starts with {)
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return &text[start..=end];
            }
        }
    }

    text
}

// Output analysis patterns
const MISSING_TOOL_PATTERNS: &[&str] = &[
    "command not found",
    "not recognized",
    "Cannot find module",
    "No such file or directory",
    "is not installed",
    "ModuleNotFoundError",
    "ImportError",
    "npm ERR!",
    "pip: command not found",
];

const AUTH_PATTERNS: &[&str] = &[
    "authentication",
    "unauthorized",
    "401",
    "403",
    "credentials",
    "token",
    "api key",
    "apikey",
    "access denied",
    "login required",
    "OAuth",
    "GOOGLE_APPLICATION_CREDENTIALS",
    "GH_TOKEN",
    "ANTHROPIC_API_KEY",
];

const PERMISSION_PATTERNS: &[&str] = &[
    "permission denied",
    "EACCES",
    "Operation not permitted",
    "Access is denied",
];

const SUCCESS_PATTERNS: &[&str] = &[
    "successfully",
    "completed",
    "done",
    "created",
    "saved",
    "connected",
    "authenticated",
];

fn analyze_output_for_findings(output: &str) -> Vec<TestFinding> {
    let mut findings = Vec::new();
    let output_lower = output.to_lowercase();

    // Check for missing tools
    for pattern in MISSING_TOOL_PATTERNS {
        if output_lower.contains(&pattern.to_lowercase()) {
            findings.push(TestFinding {
                id: Uuid::new_v4().to_string(),
                finding_type: FindingType::MissingTool,
                severity: "critical".to_string(),
                title: "Missing Tool or Dependency".to_string(),
                description: format!(
                    "The output contains '{}' which suggests a required tool or dependency is not installed.",
                    pattern
                ),
                resolution_hint: Some(
                    "Check if required tools are installed and in PATH. You may need to install dependencies."
                        .to_string(),
                ),
            });
            break; // Only report once per category
        }
    }

    // Check for auth issues
    for pattern in AUTH_PATTERNS {
        if output_lower.contains(&pattern.to_lowercase()) {
            // Don't flag success patterns that mention auth
            if output_lower.contains("authenticated successfully")
                || output_lower.contains("login successful")
            {
                continue;
            }
            findings.push(TestFinding {
                id: Uuid::new_v4().to_string(),
                finding_type: FindingType::AuthRequired,
                severity: "critical".to_string(),
                title: "Authentication Required".to_string(),
                description: format!(
                    "The output mentions '{}' which suggests authentication or API credentials are needed.",
                    pattern
                ),
                resolution_hint: Some(
                    "Set up required API keys or credentials. Check environment variables or config files."
                        .to_string(),
                ),
            });
            break;
        }
    }

    // Check for permission issues
    for pattern in PERMISSION_PATTERNS {
        if output_lower.contains(&pattern.to_lowercase()) {
            findings.push(TestFinding {
                id: Uuid::new_v4().to_string(),
                finding_type: FindingType::PermissionDenied,
                severity: "warning".to_string(),
                title: "Permission Issue".to_string(),
                description: format!(
                    "The output contains '{}' which indicates a permission problem.",
                    pattern
                ),
                resolution_hint: Some(
                    "Check file/directory permissions or run with appropriate privileges."
                        .to_string(),
                ),
            });
            break;
        }
    }

    // Check for success patterns (only if no critical issues)
    if !findings.iter().any(|f| f.severity == "critical") {
        for pattern in SUCCESS_PATTERNS {
            if output_lower.contains(&pattern.to_lowercase()) {
                findings.push(TestFinding {
                    id: Uuid::new_v4().to_string(),
                    finding_type: FindingType::SuccessPattern,
                    severity: "info".to_string(),
                    title: "Success Indicator".to_string(),
                    description: format!("Found success indicator: '{}'", pattern),
                    resolution_hint: None,
                });
                break;
            }
        }
    }

    findings
}

fn generate_recommendations(findings: &[TestFinding]) -> Vec<String> {
    let mut recommendations = Vec::new();

    let has_missing_tool = findings
        .iter()
        .any(|f| f.finding_type == FindingType::MissingTool);
    let has_auth_issue = findings
        .iter()
        .any(|f| f.finding_type == FindingType::AuthRequired);
    let has_success = findings
        .iter()
        .any(|f| f.finding_type == FindingType::SuccessPattern);

    if has_missing_tool {
        recommendations.push("Add installation instructions to the instruction file".to_string());
        recommendations
            .push("Consider listing required tools in a 'Prerequisites' section".to_string());
    }

    if has_auth_issue {
        recommendations
            .push("Include detailed authentication setup steps in the instruction".to_string());
        recommendations.push(
            "Consider adding environment variable requirements to a 'Setup' section".to_string(),
        );
        recommendations.push(
            "You may need to complete authentication setup before the instruction can be used"
                .to_string(),
        );
    }

    if has_success && !has_missing_tool && !has_auth_issue {
        recommendations
            .push("The instruction appears to work! Consider adding more test cases.".to_string());
    }

    if findings.is_empty() {
        recommendations.push(
            "No specific issues detected. Review the output to verify the test completed as expected."
                .to_string(),
        );
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_output_missing_tool() {
        let output = "Error: command not found: kubectl";
        let findings = analyze_output_for_findings(output);
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::MissingTool));
    }

    #[test]
    fn test_analyze_output_auth_required() {
        let output = "Error 401: Unauthorized - Please provide valid API credentials";
        let findings = analyze_output_for_findings(output);
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::AuthRequired));
    }

    #[test]
    fn test_analyze_output_success() {
        let output = "File created successfully!";
        let findings = analyze_output_for_findings(output);
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::SuccessPattern));
    }

    #[test]
    fn test_extract_json_raw() {
        let text = r#"{"instructionContent": "test", "suggestedFilename": "test.md"}"#;
        let result = extract_json(text);
        assert!(result.contains("instructionContent"));
    }

    #[test]
    fn test_extract_json_code_block() {
        let text = r#"Here's the JSON:
```json
{"instructionContent": "test", "suggestedFilename": "test.md"}
```"#;
        let result = extract_json(text);
        assert!(result.contains("instructionContent"));
    }
}
