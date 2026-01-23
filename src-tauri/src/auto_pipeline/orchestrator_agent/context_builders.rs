// Context Builders
//
// Utilities for building context sections and AI communication.

use chrono::Local;

use crate::ai_client::{
    AIClient, ContentBlock, RichContentBlock, RichMessage, RichMessageContent, Tool,
};
use crate::skill_generator::get_skill_content;
use crate::subagent_generator::get_subagent_content;

use super::types::{ContentBlockValue, ConversationContent, ConversationMessage};

/// Build system context information (OS, date, time, architecture)
/// This helps the AI give platform-specific and time-aware responses.
pub fn build_system_context() -> String {
    let now = Local::now();
    format!(
        r#"## System Information
- Operating System: {}
- OS Family: {}
- Architecture: {}
- Date: {}
- Time: {}"#,
        std::env::consts::OS,
        std::env::consts::FAMILY,
        std::env::consts::ARCH,
        now.format("%Y-%m-%d"),
        now.format("%H:%M %Z"),
    )
}

/// Build a comprehensive skills section with full skill content (not just names).
/// This loads SKILL.md, reference.md, and examples.md for each generated skill.
pub fn build_full_skills_section(generated_skills: &[String], working_dir: &str) -> String {
    if generated_skills.is_empty() {
        return String::new();
    }

    let mut sections = Vec::new();
    sections.push("## AVAILABLE SKILLS\n".to_string());
    sections.push(
        "The following skills have been generated. Use the information below when implementing.\n"
            .to_string(),
    );

    for skill_name in generated_skills {
        match get_skill_content(skill_name, working_dir) {
            Ok(content) => {
                sections.push(format!("\n### Skill: {}\n", skill_name));
                sections.push(content.skill_md.clone());

                if let Some(ref reference) = content.reference_md {
                    if !reference.trim().is_empty() {
                        sections.push("\n#### Reference Details\n".to_string());
                        sections.push(reference.clone());
                    }
                }

                if let Some(ref examples) = content.examples_md {
                    if !examples.trim().is_empty() {
                        sections.push("\n#### Examples\n".to_string());
                        sections.push(examples.clone());
                    }
                }
            }
            Err(e) => {
                sections.push(format!(
                    "\n### Skill: {} (failed to load: {})\n",
                    skill_name, e
                ));
            }
        }
    }

    sections.join("\n")
}

/// Build a comprehensive subagents section with full subagent content.
/// This loads the YAML frontmatter and system prompt for each generated subagent.
pub fn build_full_subagents_section(generated_subagents: &[String], working_dir: &str) -> String {
    if generated_subagents.is_empty() {
        return String::new();
    }

    let mut sections = Vec::new();
    sections.push("## AVAILABLE SUBAGENTS\n".to_string());
    sections.push(
        "The following subagents have been generated and can be used for task delegation.\n"
            .to_string(),
    );

    for agent_name in generated_subagents {
        match get_subagent_content(agent_name, working_dir) {
            Ok(content) => {
                sections.push(format!("\n### Subagent: {}\n", agent_name));
                sections.push(format!("**Description**: {}\n", content.description));

                if let Some(ref tools) = content.tools {
                    sections.push(format!("**Tools**: {}\n", tools.join(", ")));
                }

                if let Some(ref model) = content.model {
                    sections.push(format!("**Model**: {}\n", model));
                }

                sections.push("\n**System Prompt**:\n".to_string());
                sections.push(content.system_prompt.clone());
            }
            Err(e) => {
                sections.push(format!(
                    "\n### Subagent: {} (failed to load: {})\n",
                    agent_name, e
                ));
            }
        }
    }

    sections.join("\n")
}

/// Send messages to AI and get response
/// Uses rich message format to properly preserve tool_use and tool_result blocks
pub async fn send_to_ai(
    ai_client: &AIClient,
    messages: &[ConversationMessage],
    tools: &[Tool],
) -> Result<Vec<ContentBlockValue>, String> {
    // Convert our messages to rich message format for proper multi-turn tool conversations
    let api_messages: Vec<RichMessage> = messages
        .iter()
        .map(|m| {
            let content = match &m.content {
                ConversationContent::Text(s) => RichMessageContent::Text(s.clone()),
                ConversationContent::Blocks(blocks) => {
                    // Convert ContentBlockValue to RichContentBlock
                    let rich_blocks: Vec<RichContentBlock> = blocks
                        .iter()
                        .map(|block| match block {
                            ContentBlockValue::Text { text } => {
                                RichContentBlock::Text { text: text.clone() }
                            }
                            ContentBlockValue::ToolUse { id, name, input } => {
                                RichContentBlock::ToolUse {
                                    id: id.clone(),
                                    name: name.clone(),
                                    input: input.clone(),
                                }
                            }
                            ContentBlockValue::ToolResult {
                                tool_use_id,
                                content,
                                is_error,
                            } => RichContentBlock::ToolResult {
                                tool_use_id: tool_use_id.clone(),
                                content: content.clone(),
                                is_error: *is_error,
                            },
                        })
                        .collect();
                    RichMessageContent::Blocks(rich_blocks)
                }
            };
            RichMessage {
                role: m.role.clone(),
                content,
            }
        })
        .collect();

    let response = ai_client
        .send_rich_message_with_tools(api_messages, tools.to_vec())
        .await
        .map_err(|e| format!("AI request failed: {}", e))?;

    // Convert response content blocks to our format
    let blocks: Vec<ContentBlockValue> = response
        .content
        .into_iter()
        .map(|block| match block {
            ContentBlock::Text { text } => ContentBlockValue::Text { text },
            ContentBlock::ToolUse { id, name, input } => {
                ContentBlockValue::ToolUse { id, name, input }
            }
        })
        .collect();

    Ok(blocks)
}
