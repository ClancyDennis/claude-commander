// Resource Tools
//
// Tools for reading and creating resources: read_instruction_file, create_skill,
// create_subagent, generate_claudemd.

use serde_json::Value;

use crate::auto_pipeline::orchestrator_tools::{
    CreateSkillInput, CreateSubagentInput, GenerateClaudeMdInput, ReadInstructionFileInput,
    ToolResult,
};
use crate::claudemd_generator::generate_claudemd_from_instructions;
use crate::instruction_manager::get_instruction_file_content;
use crate::skill_generator::generate_skill_from_instruction;
use crate::subagent_generator::generate_subagent_from_instruction;

use super::super::OrchestratorAgent;

impl OrchestratorAgent {
    /// Read instruction file tool implementation
    pub(crate) async fn tool_read_instruction_file(&self, input: &Value) -> ToolResult {
        let parsed: ReadInstructionFileInput = match serde_json::from_value(input.clone()) {
            Ok(p) => p,
            Err(e) => return ToolResult::error("".to_string(), format!("Invalid input: {}", e)),
        };

        // Find the instruction file
        let file_info = self
            .instruction_files
            .iter()
            .find(|f| f.relative_path == parsed.file_path || f.name == parsed.file_path);

        let file_path = match file_info {
            Some(info) => &info.path,
            None => {
                return ToolResult::error(
                    "".to_string(),
                    format!("Instruction file not found: {}", parsed.file_path),
                )
            }
        };

        match get_instruction_file_content(file_path) {
            Ok(content) => ToolResult::success("".to_string(), content),
            Err(e) => ToolResult::error("".to_string(), format!("Failed to read file: {}", e)),
        }
    }

    /// Create skill tool implementation
    pub(crate) async fn tool_create_skill(&mut self, input: &Value) -> ToolResult {
        let parsed: CreateSkillInput = match serde_json::from_value(input.clone()) {
            Ok(p) => p,
            Err(e) => return ToolResult::error("".to_string(), format!("Invalid input: {}", e)),
        };

        // Check if we've hit the skill limit
        if self.generated_skills.len() >= 10 {
            return ToolResult::error(
                "".to_string(),
                "Maximum of 10 skills already created.".to_string(),
            );
        }

        // Find the instruction file
        let file_info = self.instruction_files.iter().find(|f| {
            f.relative_path == parsed.instruction_file_path || f.name == parsed.instruction_file_path
        });

        let file_path = match file_info {
            Some(info) => info.path.clone(),
            None => {
                return ToolResult::error(
                    "".to_string(),
                    format!(
                        "Instruction file not found: {}",
                        parsed.instruction_file_path
                    ),
                )
            }
        };

        // Generate the skill
        match generate_skill_from_instruction(&file_path, &self.working_dir, &self.ai_client).await
        {
            Ok(skill) => {
                self.generated_skills.push(skill.skill_name.clone());
                ToolResult::success(
                    "".to_string(),
                    format!(
                        "Skill '{}' created successfully at {}. Rationale: {}",
                        skill.skill_name, skill.skill_path, parsed.rationale
                    ),
                )
            }
            Err(e) => ToolResult::error("".to_string(), format!("Failed to create skill: {}", e)),
        }
    }

    /// Create subagent tool implementation
    pub(crate) async fn tool_create_subagent(&mut self, input: &Value) -> ToolResult {
        let parsed: CreateSubagentInput = match serde_json::from_value(input.clone()) {
            Ok(p) => p,
            Err(e) => return ToolResult::error("".to_string(), format!("Invalid input: {}", e)),
        };

        // Check if we've hit the subagent limit (same as skills)
        if self.generated_subagents.len() >= 10 {
            return ToolResult::error(
                "".to_string(),
                "Maximum of 10 subagents already created.".to_string(),
            );
        }

        // Find the instruction file
        let file_info = self.instruction_files.iter().find(|f| {
            f.relative_path == parsed.instruction_file_path || f.name == parsed.instruction_file_path
        });

        let file_path = match file_info {
            Some(info) => info.path.clone(),
            None => {
                return ToolResult::error(
                    "".to_string(),
                    format!(
                        "Instruction file not found: {}",
                        parsed.instruction_file_path
                    ),
                )
            }
        };

        // Generate the subagent
        match generate_subagent_from_instruction(&file_path, &self.working_dir, &self.ai_client)
            .await
        {
            Ok(subagent) => {
                self.generated_subagents.push(subagent.agent_name.clone());
                ToolResult::success(
                    "".to_string(),
                    format!(
                        "Subagent '{}' created successfully at {}. Rationale: {}",
                        subagent.agent_name, subagent.agent_path, parsed.rationale
                    ),
                )
            }
            Err(e) => {
                ToolResult::error("".to_string(), format!("Failed to create subagent: {}", e))
            }
        }
    }

    /// Generate CLAUDE.md tool implementation
    pub(crate) async fn tool_generate_claudemd(&mut self, input: &Value) -> ToolResult {
        let parsed: GenerateClaudeMdInput = match serde_json::from_value(input.clone()) {
            Ok(p) => p,
            Err(e) => return ToolResult::error("".to_string(), format!("Invalid input: {}", e)),
        };

        // Check if we've already generated CLAUDE.md
        if self.claudemd_generated {
            return ToolResult::error(
                "".to_string(),
                "CLAUDE.md has already been generated. Only one CLAUDE.md file per project."
                    .to_string(),
            );
        }

        // Resolve all instruction file paths
        let mut resolved_paths = Vec::new();
        for path in &parsed.instruction_file_paths {
            let file_info = self
                .instruction_files
                .iter()
                .find(|f| f.relative_path == *path || f.name == *path);

            match file_info {
                Some(info) => resolved_paths.push(info.path.clone()),
                None => {
                    return ToolResult::error(
                        "".to_string(),
                        format!("Instruction file not found: {}", path),
                    )
                }
            }
        }

        if resolved_paths.is_empty() {
            return ToolResult::error(
                "".to_string(),
                "At least one instruction file path is required.".to_string(),
            );
        }

        // Generate the CLAUDE.md
        match generate_claudemd_from_instructions(&resolved_paths, &self.working_dir, &self.ai_client)
            .await
        {
            Ok(result) => {
                self.claudemd_generated = true;
                ToolResult::success(
                    "".to_string(),
                    format!(
                        "CLAUDE.md generated successfully at {}. Sources: {}. Rationale: {}",
                        result.file_path,
                        result.source_files.join(", "),
                        parsed.rationale
                    ),
                )
            }
            Err(e) => {
                ToolResult::error("".to_string(), format!("Failed to generate CLAUDE.md: {}", e))
            }
        }
    }
}
