// Orchestrator Tools - Tool definitions for the persistent orchestrator agent
//
// The orchestrator runs as a tool-calling loop throughout the entire pipeline,
// maintaining context and making decisions based on accumulated knowledge.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Tool definition for the AI API
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Result of executing a tool
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(tool_use_id: String, content: String) -> Self {
        Self {
            tool_use_id,
            content,
            is_error: false,
        }
    }

    pub fn error(tool_use_id: String, error: String) -> Self {
        Self {
            tool_use_id,
            content: error,
            is_error: true,
        }
    }
}

use super::state_machine::PipelineState;

/// All available tools for the orchestrator (all phases)
pub fn get_orchestrator_tools() -> Vec<ToolDefinition> {
    vec![
        // Phase A: Skill Synthesis Tools
        tool_read_instruction_file(),
        tool_create_skill(),

        // Phase B: Planning Tools
        tool_start_planning(),
        tool_approve_plan(),

        // Phase C: Execution Tools
        tool_start_execution(),

        // Phase D: Verification & Decision Tools
        tool_start_verification(),
        tool_complete(),
        tool_iterate(),
        tool_replan(),
        tool_give_up(),
    ]
}

/// Get tools available for a specific phase of the pipeline
/// This enforces the workflow by only providing tools valid for the current state
/// Note: give_up is intentionally NOT available in most phases to prevent the LLM from bailing out
pub fn get_tools_for_state(state: &PipelineState) -> Vec<ToolDefinition> {
    match state {
        // Phase A: Skill & Subagent Synthesis - read instructions, create skills/subagents/CLAUDE.md, then start planning
        PipelineState::ReceivedTask
        | PipelineState::AnalyzingTask
        | PipelineState::SelectingInstructions
        | PipelineState::GeneratingSkills => {
            vec![
                tool_read_instruction_file(),
                tool_create_skill(),
                tool_create_subagent(),
                tool_generate_claudemd(),
                tool_start_planning(),
            ]
        }

        // Phase B: Planning - after plan is generated, must approve or replan
        // approve_plan transitions to ReadyForExecution
        PipelineState::Planning | PipelineState::PlanReady | PipelineState::PlanRevisionRequired => {
            vec![
                tool_approve_plan(),
                tool_replan(),
            ]
        }

        // Phase C: Ready for Execution - must start execution
        PipelineState::ReadyForExecution => {
            vec![
                tool_start_execution(),
            ]
        }

        // During/after execution - must verify
        PipelineState::Executing => {
            vec![
                tool_start_verification(),
            ]
        }

        // Phase D: Verification - decide outcome
        PipelineState::Verifying => {
            vec![
                tool_complete(),
                tool_iterate(),
                tool_replan(),
            ]
        }

        // Verification passed - complete or iterate
        PipelineState::VerificationPassed => {
            vec![
                tool_complete(),
                tool_iterate(),
            ]
        }

        // Verification failed - iterate or replan (give_up only here after repeated failures)
        PipelineState::VerificationFailed => {
            vec![
                tool_iterate(),
                tool_replan(),
                tool_give_up(),
            ]
        }

        // Terminal states - no tools
        PipelineState::Completed | PipelineState::Failed | PipelineState::GaveUp => {
            vec![]
        }
    }
}

// ============================================================================
// Phase A: Skill Synthesis Tools
// ============================================================================

fn tool_read_instruction_file() -> ToolDefinition {
    ToolDefinition {
        name: "read_instruction_file".to_string(),
        description: "Read the content of an instruction file from .grove-instructions/ to decide if it should be converted to a skill. Use this to inspect instruction files before deciding whether to create a skill from them.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The relative path of the instruction file within .grove-instructions/"
                }
            },
            "required": ["file_path"]
        }),
    }
}

fn tool_create_skill() -> ToolDefinition {
    ToolDefinition {
        name: "create_skill".to_string(),
        description: "Generate a Claude Code skill from an instruction file. The skill will be created in .claude/skills/ and available for use by agents. Only create skills that are relevant to the current task.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "instruction_file_path": {
                    "type": "string",
                    "description": "The relative path of the instruction file to convert to a skill"
                },
                "rationale": {
                    "type": "string",
                    "description": "Brief explanation of why this skill is needed for the task"
                }
            },
            "required": ["instruction_file_path", "rationale"]
        }),
    }
}

fn tool_create_subagent() -> ToolDefinition {
    ToolDefinition {
        name: "create_subagent".to_string(),
        description: "Generate a Claude Code subagent from an instruction file. The subagent will be created in .claude/agents/ and available for task delegation. Subagents are autonomous AI assistants with their own context and specialized capabilities.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "instruction_file_path": {
                    "type": "string",
                    "description": "The relative path of the instruction file to convert to a subagent"
                },
                "rationale": {
                    "type": "string",
                    "description": "Brief explanation of why this subagent is needed for the task"
                }
            },
            "required": ["instruction_file_path", "rationale"]
        }),
    }
}

fn tool_generate_claudemd() -> ToolDefinition {
    ToolDefinition {
        name: "generate_claudemd".to_string(),
        description: "Generate or update a CLAUDE.md file from instruction files. CLAUDE.md provides project-specific context and guidelines that Claude automatically reads when working in the directory. Use this to create persistent project memory with coding standards, commands, and conventions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "instruction_file_paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Array of instruction file paths to include in the CLAUDE.md generation"
                },
                "rationale": {
                    "type": "string",
                    "description": "Brief explanation of what project context should be captured"
                }
            },
            "required": ["instruction_file_paths", "rationale"]
        }),
    }
}

// ============================================================================
// Phase B: Planning Tools
// ============================================================================

fn tool_start_planning() -> ToolDefinition {
    ToolDefinition {
        name: "start_planning".to_string(),
        description: "Transition to the planning phase. Call this after you have created all necessary skills. A planning agent will be spawned to create an implementation plan.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "summary": {
                    "type": "string",
                    "description": "Brief summary of the skills created and why they were chosen"
                }
            },
            "required": ["summary"]
        }),
    }
}

fn tool_approve_plan() -> ToolDefinition {
    ToolDefinition {
        name: "approve_plan".to_string(),
        description: "Approve the current plan and proceed to execution. Only call this after reviewing the plan output.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "assessment": {
                    "type": "string",
                    "description": "Brief assessment of why the plan is adequate"
                }
            },
            "required": ["assessment"]
        }),
    }
}

// ============================================================================
// Phase C: Execution Tools
// ============================================================================

fn tool_start_execution() -> ToolDefinition {
    ToolDefinition {
        name: "start_execution".to_string(),
        description: "Start the execution phase. A build agent will implement the approved plan.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "notes": {
                    "type": "string",
                    "description": "Any notes or guidance for the build agent"
                }
            },
            "required": []
        }),
    }
}

// ============================================================================
// Phase D: Verification & Decision Tools
// ============================================================================

fn tool_start_verification() -> ToolDefinition {
    ToolDefinition {
        name: "start_verification".to_string(),
        description: "Start the verification phase. A verification agent will review the implementation.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "focus_areas": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Specific areas to focus verification on"
                }
            },
            "required": []
        }),
    }
}

fn tool_complete() -> ToolDefinition {
    ToolDefinition {
        name: "complete".to_string(),
        description: "Mark the pipeline as successfully completed. Use this when verification passes and the task is done.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "summary": {
                    "type": "string",
                    "description": "Summary of what was accomplished"
                }
            },
            "required": ["summary"]
        }),
    }
}

fn tool_iterate() -> ToolDefinition {
    ToolDefinition {
        name: "iterate".to_string(),
        description: "Go back to execution to fix issues found in verification. Use when the plan is sound but implementation needs fixes.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "issues": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of issues to fix in the next iteration"
                },
                "suggestions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Suggestions for fixing the issues"
                }
            },
            "required": ["issues"]
        }),
    }
}

fn tool_replan() -> ToolDefinition {
    ToolDefinition {
        name: "replan".to_string(),
        description: "Go back to planning phase. Use when the current plan is fundamentally flawed and needs to be revised.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "reason": {
                    "type": "string",
                    "description": "Why replanning is needed"
                },
                "issues": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Issues with the current plan"
                },
                "suggestions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Suggestions for the new plan"
                }
            },
            "required": ["reason", "issues"]
        }),
    }
}

fn tool_give_up() -> ToolDefinition {
    ToolDefinition {
        name: "give_up".to_string(),
        description: "Abandon the pipeline. Use only when the task cannot be completed after multiple attempts or is fundamentally impossible.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "reason": {
                    "type": "string",
                    "description": "Why the task cannot be completed"
                }
            },
            "required": ["reason"]
        }),
    }
}

// ============================================================================
// Tool Input Parsing
// ============================================================================

/// Parsed input for read_instruction_file tool
#[derive(Debug, Deserialize)]
pub struct ReadInstructionFileInput {
    pub file_path: String,
}

/// Parsed input for create_skill tool
#[derive(Debug, Deserialize)]
pub struct CreateSkillInput {
    pub instruction_file_path: String,
    pub rationale: String,
}

/// Parsed input for create_subagent tool
#[derive(Debug, Deserialize)]
pub struct CreateSubagentInput {
    pub instruction_file_path: String,
    pub rationale: String,
}

/// Parsed input for generate_claudemd tool
#[derive(Debug, Deserialize)]
pub struct GenerateClaudeMdInput {
    pub instruction_file_paths: Vec<String>,
    pub rationale: String,
}

/// Parsed input for start_planning tool
#[derive(Debug, Deserialize)]
pub struct StartPlanningInput {
    pub summary: String,
}

/// Parsed input for approve_plan tool
#[derive(Debug, Deserialize)]
pub struct ApprovePlanInput {
    pub assessment: String,
}

/// Parsed input for start_execution tool
#[derive(Debug, Deserialize)]
pub struct StartExecutionInput {
    #[serde(default)]
    pub notes: Option<String>,
}

/// Parsed input for start_verification tool
#[derive(Debug, Deserialize)]
pub struct StartVerificationInput {
    #[serde(default)]
    pub focus_areas: Vec<String>,
}

/// Parsed input for complete tool
#[derive(Debug, Deserialize)]
pub struct CompleteInput {
    pub summary: String,
}

/// Parsed input for iterate tool
#[derive(Debug, Deserialize)]
pub struct IterateInput {
    pub issues: Vec<String>,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// Parsed input for replan tool
#[derive(Debug, Deserialize)]
pub struct ReplanInput {
    pub reason: String,
    pub issues: Vec<String>,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// Parsed input for give_up tool
#[derive(Debug, Deserialize)]
pub struct GiveUpInput {
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions_are_valid() {
        let tools = get_orchestrator_tools();
        assert!(!tools.is_empty());

        for tool in &tools {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            // Verify schema is valid JSON
            assert!(tool.input_schema.is_object());
        }
    }

    #[test]
    fn test_parse_read_instruction_file_input() {
        let input: ReadInstructionFileInput = serde_json::from_str(
            r#"{"file_path": "api-guidelines.md"}"#
        ).unwrap();
        assert_eq!(input.file_path, "api-guidelines.md");
    }

    #[test]
    fn test_parse_create_skill_input() {
        let input: CreateSkillInput = serde_json::from_str(
            r#"{"instruction_file_path": "api-guidelines.md", "rationale": "needed for API design"}"#
        ).unwrap();
        assert_eq!(input.instruction_file_path, "api-guidelines.md");
        assert_eq!(input.rationale, "needed for API design");
    }
}
