// Subagent Generator - Convert instruction files to Claude Code subagents
//
// This module mirrors the skill_generator pattern but produces subagent
// configuration files (.claude/agents/{name}.md) with YAML frontmatter.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::ai_client::AIClient;
use crate::utils::generator::{
    extract_json_from_response, extract_text_from_content_blocks, sanitize_name,
};

/// Metadata about a generated subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSubagent {
    pub agent_name: String,
    pub agent_path: String,
    pub source_file: String,
    pub generated_at: String,
}

/// Content of a subagent (parsed from AI response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentContent {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disallowed_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
}

const SUBAGENT_GENERATION_PROMPT: &str = r###"You are a subagent generator for Claude Code. Convert the following instruction document into a specialized subagent configuration.

A subagent is an autonomous AI assistant that handles specific types of tasks. Each subagent runs in its own context window with a custom system prompt and specific tool access.

## Input Document

<document>
{instruction_content}
</document>

## Output Format

Generate a JSON object with the following structure:

```json
{
  "name": "kebab-case-name",
  "description": "One sentence describing WHEN Claude should delegate to this subagent",
  "tools": ["Read", "Grep", "Glob", "Bash"],
  "model": "sonnet",
  "system_prompt": "The full system prompt as markdown..."
}
```

## Guidelines

### name
- Use kebab-case (lowercase letters and hyphens only)
- Descriptive and concise (e.g., "code-reviewer", "debugger", "data-analyst")
- Should reflect the agent's specialty

### description (Required)
- Single sentence explaining WHEN Claude should delegate to this subagent
- Claude uses this description to decide when to automatically delegate tasks
- Include "Use proactively" if the agent should be used automatically
- Examples:
  - "Expert code reviewer. Use proactively after writing or modifying code."
  - "Debugging specialist for errors and test failures. Use when encountering issues."
  - "Database query specialist for SQL operations. Use for data analysis tasks."

### tools
- List of tools the subagent can use
- Common tools: Read, Glob, Grep, Bash, Edit, Write, WebFetch, WebSearch
- Use fewer tools for safer, more focused agents
- Omit entirely for full tool access (inherits from parent)
- Read-only agents: ["Read", "Glob", "Grep", "Bash"]
- Full-featured agents: omit the field

### model
- "sonnet" - balanced capability and speed (default, recommended for most tasks)
- "haiku" - fast and cheap, good for simple lookups and quick tasks
- "opus" - most capable, for complex reasoning tasks
- "inherit" - use the same model as the parent conversation

### system_prompt
- Clear, focused instructions for the subagent
- Structure it with:
  1. Role introduction (who the agent is)
  2. When invoked workflow (numbered steps)
  3. Checklist or guidelines
  4. Output format expectations
- Keep it concise but complete

## Important

- Return ONLY the JSON object, no additional text or markdown formatting
- Ensure all JSON strings are properly escaped (especially newlines in system_prompt)
- Focus the subagent on a specific task domain - don't make it too general
- Make the description actionable so Claude knows exactly when to delegate

Now generate the subagent configuration:"###;

/// Generate a Claude Code subagent from an instruction file using AI
pub async fn generate_subagent_from_instruction(
    instruction_file_path: &str,
    working_dir: &str,
    ai_client: &AIClient,
) -> Result<GeneratedSubagent, String> {
    // 1. Read instruction file
    let instruction_content = fs::read_to_string(instruction_file_path)
        .map_err(|e| format!("Failed to read instruction file: {}", e))?;

    // Extract filename for fallback name
    let file_name = Path::new(instruction_file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?;

    // 2. Construct prompt
    let prompt = SUBAGENT_GENERATION_PROMPT.replace("{instruction_content}", &instruction_content);

    // 3. Call AI to generate subagent content
    let messages = vec![crate::ai_client::Message {
        role: "user".to_string(),
        content: prompt.clone(),
    }];

    let response = ai_client
        .send_message(messages)
        .await
        .map_err(|e| format!("AI generation failed: {}", e))?;

    // Extract text content from response
    let ai_response = extract_text_from_content_blocks(&response.content);

    // 4. Parse AI response into SubagentContent
    let subagent_content = parse_subagent_response(&ai_response, file_name)?;

    // 5. Create subagent file
    let agent_path = create_subagent_file(working_dir, &subagent_content)?;

    // 6. Return metadata
    Ok(GeneratedSubagent {
        agent_name: subagent_content.name.clone(),
        agent_path,
        source_file: instruction_file_path.to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn parse_subagent_response(
    ai_response: &str,
    fallback_name: &str,
) -> Result<SubagentContent, String> {
    // Extract JSON from response (AI might wrap it in markdown)
    let json_str = extract_json_from_response(ai_response)?;

    let mut subagent_content: SubagentContent = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse AI response as JSON: {}", e))?;

    // Validate and sanitize name
    if subagent_content.name.is_empty() {
        subagent_content.name = sanitize_name(fallback_name);
    } else {
        subagent_content.name = sanitize_name(&subagent_content.name);
    }

    // Validate required fields
    if subagent_content.description.trim().is_empty() {
        return Err("Generated description is empty".to_string());
    }

    if subagent_content.system_prompt.trim().is_empty() {
        return Err("Generated system_prompt is empty".to_string());
    }

    Ok(subagent_content)
}

fn create_subagent_file(
    working_dir: &str,
    subagent_content: &SubagentContent,
) -> Result<String, String> {
    let agents_dir = Path::new(working_dir).join(".claude").join("agents");

    // Create agents directory
    fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("Failed to create agents directory: {}", e))?;

    // Build YAML frontmatter
    let mut frontmatter = String::new();
    frontmatter.push_str("---\n");
    frontmatter.push_str(&format!("name: {}\n", subagent_content.name));
    frontmatter.push_str(&format!("description: {}\n", subagent_content.description));

    if let Some(ref tools) = subagent_content.tools {
        if !tools.is_empty() {
            frontmatter.push_str(&format!("tools: {}\n", tools.join(", ")));
        }
    }

    if let Some(ref disallowed) = subagent_content.disallowed_tools {
        if !disallowed.is_empty() {
            frontmatter.push_str(&format!("disallowedTools: {}\n", disallowed.join(", ")));
        }
    }

    if let Some(ref model) = subagent_content.model {
        frontmatter.push_str(&format!("model: {}\n", model));
    }

    if let Some(ref mode) = subagent_content.permission_mode {
        frontmatter.push_str(&format!("permissionMode: {}\n", mode));
    }

    if let Some(ref skills) = subagent_content.skills {
        if !skills.is_empty() {
            frontmatter.push_str(&format!("skills: {}\n", skills.join(", ")));
        }
    }

    frontmatter.push_str("---\n\n");

    // Combine frontmatter and system prompt
    let file_content = format!("{}{}", frontmatter, subagent_content.system_prompt);

    // Write the file
    let agent_file = agents_dir.join(format!("{}.md", subagent_content.name));
    fs::write(&agent_file, file_content)
        .map_err(|e| format!("Failed to write subagent file: {}", e))?;

    Ok(agent_file.to_string_lossy().to_string())
}

/// List all generated subagents in a working directory
pub fn list_generated_subagents(working_dir: &str) -> Result<Vec<GeneratedSubagent>, String> {
    let agents_dir = Path::new(working_dir).join(".claude").join("agents");

    if !agents_dir.exists() {
        return Ok(Vec::new());
    }

    let mut subagents = Vec::new();

    let entries =
        fs::read_dir(&agents_dir).map_err(|e| format!("Failed to read agents directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            let agent_name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let metadata =
                fs::metadata(&path).map_err(|e| format!("Failed to read metadata: {}", e))?;

            let modified = metadata
                .modified()
                .map_err(|e| format!("Failed to get modified time: {}", e))?;

            let generated_at = {
                use std::time::UNIX_EPOCH;
                let duration = modified
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("Invalid modified time: {}", e))?;
                let secs = duration.as_secs();
                let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
                    .ok_or_else(|| "Failed to convert timestamp".to_string())?;
                datetime.to_rfc3339()
            };

            subagents.push(GeneratedSubagent {
                agent_name,
                agent_path: path.to_string_lossy().to_string(),
                source_file: "unknown".to_string(),
                generated_at,
            });
        }
    }

    subagents.sort_by(|a, b| a.agent_name.cmp(&b.agent_name));

    Ok(subagents)
}

/// Get the content of a subagent by parsing the markdown file
pub fn get_subagent_content(
    agent_name: &str,
    working_dir: &str,
) -> Result<SubagentContent, String> {
    let agent_file = Path::new(working_dir)
        .join(".claude")
        .join("agents")
        .join(format!("{}.md", agent_name));

    if !agent_file.exists() {
        return Err(format!("Subagent '{}' not found", agent_name));
    }

    let content = fs::read_to_string(&agent_file)
        .map_err(|e| format!("Failed to read subagent file: {}", e))?;

    parse_subagent_file(&content, agent_name)
}

fn parse_subagent_file(content: &str, fallback_name: &str) -> Result<SubagentContent, String> {
    // Check for YAML frontmatter
    if !content.starts_with("---") {
        return Err("Subagent file missing YAML frontmatter".to_string());
    }

    // Find the end of frontmatter
    let after_first_marker = &content[3..];
    let end_pos = after_first_marker
        .find("---")
        .ok_or("Could not find end of YAML frontmatter")?;

    let frontmatter = &after_first_marker[..end_pos].trim();
    let system_prompt = after_first_marker[end_pos + 3..].trim().to_string();

    // Parse YAML frontmatter manually (simple key: value parsing)
    let mut name = fallback_name.to_string();
    let mut description = String::new();
    let mut tools: Option<Vec<String>> = None;
    let mut disallowed_tools: Option<Vec<String>> = None;
    let mut model: Option<String> = None;
    let mut permission_mode: Option<String> = None;
    let mut skills: Option<Vec<String>> = None;

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "name" => name = value.to_string(),
                "description" => description = value.to_string(),
                "tools" => {
                    tools = Some(
                        value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    );
                }
                "disallowedTools" => {
                    disallowed_tools = Some(
                        value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    );
                }
                "model" => model = Some(value.to_string()),
                "permissionMode" => permission_mode = Some(value.to_string()),
                "skills" => {
                    skills = Some(
                        value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    );
                }
                _ => {} // Ignore unknown fields
            }
        }
    }

    Ok(SubagentContent {
        name,
        description,
        system_prompt,
        tools,
        disallowed_tools,
        model,
        permission_mode,
        skills,
    })
}

/// Delete a generated subagent
pub fn delete_generated_subagent(agent_name: &str, working_dir: &str) -> Result<(), String> {
    let agent_file = Path::new(working_dir)
        .join(".claude")
        .join("agents")
        .join(format!("{}.md", agent_name));

    if !agent_file.exists() {
        return Err(format!("Subagent '{}' not found", agent_name));
    }

    fs::remove_file(&agent_file).map_err(|e| format!("Failed to delete subagent: {}", e))?;

    Ok(())
}

/// Clean up generated subagents tracked in the list
pub fn cleanup_generated_subagents(
    working_dir: &str,
    generated_agent_names: &[String],
) -> Result<(), String> {
    for agent_name in generated_agent_names {
        if let Err(e) = delete_generated_subagent(agent_name, working_dir) {
            eprintln!("Warning: Failed to cleanup subagent {}: {}", agent_name, e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Code Reviewer"), "code-reviewer");
        assert_eq!(sanitize_name("my_cool_agent"), "my-cool-agent");
        assert_eq!(
            sanitize_name("test--multiple---dashes"),
            "test-multiple-dashes"
        );
        assert_eq!(sanitize_name("Special!@#Characters"), "special-characters");
    }

    #[test]
    fn test_parse_subagent_response_json() {
        let response = r#"{"name":"test-agent","description":"Test agent","system_prompt":"You are a test agent.","tools":["Read"],"model":"sonnet"}"#;
        let result = parse_subagent_response(response, "fallback");
        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.description, "Test agent");
        assert_eq!(agent.model, Some("sonnet".to_string()));
    }

    #[test]
    fn test_parse_subagent_file() {
        let content = r#"---
name: code-reviewer
description: Expert code reviewer
tools: Read, Glob, Grep
model: sonnet
---

You are a senior code reviewer."#;

        let result = parse_subagent_file(content, "fallback");
        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name, "code-reviewer");
        assert_eq!(agent.description, "Expert code reviewer");
        assert_eq!(
            agent.tools,
            Some(vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Grep".to_string()
            ])
        );
        assert_eq!(agent.model, Some("sonnet".to_string()));
        assert_eq!(agent.system_prompt, "You are a senior code reviewer.");
    }
}
