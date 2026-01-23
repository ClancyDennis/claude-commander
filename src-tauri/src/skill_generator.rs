use crate::ai_client::AIClient;
use crate::utils::generator::{
    extract_json_from_response, extract_text_from_content_blocks, sanitize_name,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSkill {
    pub skill_name: String,
    pub skill_path: String,
    pub source_file: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContent {
    pub skill_name: String,
    pub skill_md: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_md: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples_md: Option<String>,
    #[serde(default)]
    pub scripts: Vec<SkillScript>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillScript {
    pub name: String,
    pub content: String,
    pub language: String,
}

const SKILL_GENERATION_PROMPT: &str = r###"You are a skill generator for Claude Code. Convert the following instruction document into a well-structured Claude Code skill.

## Input Document

<document>
{instruction_content}
</document>

## Output Format

Generate a JSON object with the following structure:

```json
{
  "skill_name": "kebab-case-name",
  "skill_md": "Content for SKILL.md...",
  "reference_md": "Content for reference.md...",
  "examples_md": "Content for examples.md...",
  "scripts": []
}
```

## Guidelines

### skill_name
- Use kebab-case (lowercase with hyphens)
- Descriptive and concise (e.g., "api-design-guidelines")
- Should reflect the main topic of the instruction

### SKILL.md (Required)
- Start with a # header with the skill name (human-readable, Title Case)
- Add a brief 1-2 sentence description
- Include "## When to use" section explaining when this skill applies
- List "## Key Concepts" or "## Overview" with main topics
- Keep it concise (under 500 words)
- Focus on WHEN and WHY to use this skill
- Use clear markdown formatting

### reference.md (Optional)
- Extract detailed guidelines, specifications, rules
- Organize into clear sections with ## headers
- Include technical details, parameters, configurations
- Can be longer and more comprehensive
- Use markdown formatting (tables, code blocks, lists)
- If the instruction document is short or doesn't have detailed technical info, you can omit this

### examples.md (Optional)
- Provide 2-5 concrete usage examples
- Show before/after code samples where applicable
- Include common patterns and edge cases
- Make examples realistic and practical
- Use markdown code blocks with language tags
- If examples aren't applicable, you can omit this

### scripts (Optional)
- Only if the instruction document explicitly suggests automation or includes scripts
- Keep scripts simple and focused
- Include clear comments
- Most instruction documents won't need scripts, so this will usually be empty

## Important
- Return ONLY the JSON object, no additional text or markdown formatting
- Ensure all JSON strings are properly escaped
- If a field is optional and not applicable, omit it or set to null
- Make the skill_md engaging and helpful, not just a dry summary

Now generate the skill content:"###;

/// Maximum number of retries for JSON parsing errors
const MAX_JSON_RETRIES: usize = 2;

/// Generate a Claude Code skill from an instruction file using AI
pub async fn generate_skill_from_instruction(
    instruction_file_path: &str,
    working_dir: &str,
    ai_client: &AIClient,
) -> Result<GeneratedSkill, String> {
    // 1. Read instruction file
    let instruction_content = fs::read_to_string(instruction_file_path)
        .map_err(|e| format!("Failed to read instruction file: {}", e))?;

    // Extract filename for default skill name
    let file_name = Path::new(instruction_file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?;

    // 2. Construct prompt
    let prompt = SKILL_GENERATION_PROMPT.replace("{instruction_content}", &instruction_content);

    // Track attempts for retry logic
    let mut last_error: Option<String> = None;
    let mut last_response: Option<String> = None;

    for attempt in 0..=MAX_JSON_RETRIES {
        // 3. Call AI to generate skill content
        let messages = if attempt == 0 {
            // First attempt: just the prompt
            vec![crate::ai_client::Message {
                role: "user".to_string(),
                content: prompt.clone(),
            }]
        } else {
            // Retry attempt: include the error feedback
            vec![
                crate::ai_client::Message {
                    role: "user".to_string(),
                    content: prompt.clone(),
                },
                crate::ai_client::Message {
                    role: "assistant".to_string(),
                    content: last_response.clone().unwrap_or_default(),
                },
                crate::ai_client::Message {
                    role: "user".to_string(),
                    content: format!(
                        "Your response had a JSON parsing error: {}\n\n\
                        Please provide a corrected response with valid JSON. \
                        Make sure all strings are properly escaped and the JSON is complete.",
                        last_error.as_ref().unwrap_or(&"Unknown error".to_string())
                    ),
                },
            ]
        };

        let response = ai_client
            .send_message(messages)
            .await
            .map_err(|e| format!("AI generation failed: {}", e))?;

        // Extract text content from response
        let ai_response = extract_text_from_content_blocks(&response.content);

        // 4. Parse AI response into SkillContent
        match parse_skill_response(&ai_response, file_name) {
            Ok(skill_content) => {
                // 5. Create skill directory structure and write files
                let skill_path = create_skill_directory(working_dir, &skill_content)?;

                // 6. Return metadata
                return Ok(GeneratedSkill {
                    skill_name: skill_content.skill_name.clone(),
                    skill_path,
                    source_file: instruction_file_path.to_string(),
                    generated_at: chrono::Utc::now().to_rfc3339(),
                });
            }
            Err(e) => {
                // Check if this is a JSON parsing error that might be recoverable
                if e.contains("Failed to parse AI response as JSON")
                    || e.contains("No JSON found")
                    || e.contains("Could not find")
                {
                    if attempt < MAX_JSON_RETRIES {
                        eprintln!(
                            "  JSON parsing failed (attempt {}/{}), retrying...",
                            attempt + 1,
                            MAX_JSON_RETRIES + 1
                        );
                        last_error = Some(e);
                        last_response = Some(ai_response);
                        continue;
                    }
                }
                // Either not a JSON error or we've exhausted retries
                // Include a snippet of the response for debugging
                let response_preview = if ai_response.len() > 200 {
                    format!("{}...", &ai_response[..200])
                } else {
                    ai_response.clone()
                };
                return Err(format!("{}\n\nResponse preview:\n{}", e, response_preview));
            }
        }
    }

    // Should not reach here, but just in case
    Err(last_error.unwrap_or_else(|| "Unknown error during skill generation".to_string()))
}

fn parse_skill_response(ai_response: &str, fallback_name: &str) -> Result<SkillContent, String> {
    // Extract JSON from response (AI might wrap it in markdown)
    let json_str = extract_json_from_response(ai_response)?;

    let mut skill_content: SkillContent = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse AI response as JSON: {}", e))?;

    // Validate and sanitize skill_name
    if skill_content.skill_name.is_empty() {
        skill_content.skill_name = sanitize_name(fallback_name);
    } else {
        skill_content.skill_name = sanitize_name(&skill_content.skill_name);
    }

    // Validate required fields
    if skill_content.skill_md.trim().is_empty() {
        return Err("Generated skill_md is empty".to_string());
    }

    Ok(skill_content)
}

fn create_skill_directory(
    working_dir: &str,
    skill_content: &SkillContent,
) -> Result<String, String> {
    let skills_dir = Path::new(working_dir).join(".claude").join("skills");
    let skill_dir = skills_dir.join(&skill_content.skill_name);

    // Create skill directory
    fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to create skill directory: {}", e))?;

    // Write SKILL.md
    fs::write(skill_dir.join("SKILL.md"), &skill_content.skill_md)
        .map_err(|e| format!("Failed to write SKILL.md: {}", e))?;

    // Write reference.md if present
    if let Some(ref reference) = skill_content.reference_md {
        if !reference.trim().is_empty() {
            fs::write(skill_dir.join("reference.md"), reference)
                .map_err(|e| format!("Failed to write reference.md: {}", e))?;
        }
    }

    // Write examples.md if present
    if let Some(ref examples) = skill_content.examples_md {
        if !examples.trim().is_empty() {
            fs::write(skill_dir.join("examples.md"), examples)
                .map_err(|e| format!("Failed to write examples.md: {}", e))?;
        }
    }

    // Write scripts if present
    if !skill_content.scripts.is_empty() {
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir)
            .map_err(|e| format!("Failed to create scripts directory: {}", e))?;

        for script in &skill_content.scripts {
            let ext = match script.language.as_str() {
                "python" => "py",
                "bash" | "shell" => "sh",
                "javascript" => "js",
                "typescript" => "ts",
                _ => "txt",
            };
            let script_path = scripts_dir.join(format!("{}.{}", script.name, ext));
            fs::write(&script_path, &script.content)
                .map_err(|e| format!("Failed to write script {}: {}", script.name, e))?;

            // Make shell scripts executable
            #[cfg(unix)]
            if ext == "sh" {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path).unwrap().permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms).ok();
            }
        }
    }

    Ok(skill_dir.to_string_lossy().to_string())
}

/// List all generated skills in a working directory
pub fn list_generated_skills(working_dir: &str) -> Result<Vec<GeneratedSkill>, String> {
    let skills_dir = Path::new(working_dir).join(".claude").join("skills");

    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    let entries =
        fs::read_dir(&skills_dir).map_err(|e| format!("Failed to read skills directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                let skill_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Try to read metadata (we don't store source_file, so it's unknown)
                let metadata = fs::metadata(&skill_md_path)
                    .map_err(|e| format!("Failed to read metadata: {}", e))?;

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

                skills.push(GeneratedSkill {
                    skill_name: skill_name.clone(),
                    skill_path: path.to_string_lossy().to_string(),
                    source_file: "unknown".to_string(),
                    generated_at,
                });
            }
        }
    }

    skills.sort_by(|a, b| a.skill_name.cmp(&b.skill_name));

    Ok(skills)
}

/// Delete a generated skill
pub fn delete_generated_skill(skill_name: &str, working_dir: &str) -> Result<(), String> {
    let skill_dir = Path::new(working_dir)
        .join(".claude")
        .join("skills")
        .join(skill_name);

    if !skill_dir.exists() {
        return Err(format!("Skill '{}' not found", skill_name));
    }

    fs::remove_dir_all(&skill_dir).map_err(|e| format!("Failed to delete skill: {}", e))?;

    Ok(())
}

/// Get the content of a skill
pub fn get_skill_content(skill_name: &str, working_dir: &str) -> Result<SkillContent, String> {
    let skill_dir = Path::new(working_dir)
        .join(".claude")
        .join("skills")
        .join(skill_name);

    if !skill_dir.exists() {
        return Err(format!("Skill '{}' not found", skill_name));
    }

    let skill_md = fs::read_to_string(skill_dir.join("SKILL.md"))
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

    let reference_md = skill_dir
        .join("reference.md")
        .exists()
        .then(|| fs::read_to_string(skill_dir.join("reference.md")).ok())
        .flatten();

    let examples_md = skill_dir
        .join("examples.md")
        .exists()
        .then(|| fs::read_to_string(skill_dir.join("examples.md")).ok())
        .flatten();

    let scripts = Vec::new(); // TODO: Read scripts if needed

    Ok(SkillContent {
        skill_name: skill_name.to_string(),
        skill_md,
        reference_md,
        examples_md,
        scripts,
    })
}

/// Clean up generated skills tracked in the list
pub fn cleanup_generated_skills(
    working_dir: &str,
    generated_skill_names: &[String],
) -> Result<(), String> {
    for skill_name in generated_skill_names {
        if let Err(e) = delete_generated_skill(skill_name, working_dir) {
            eprintln!("Warning: Failed to cleanup skill {}: {}", skill_name, e);
        }
    }
    Ok(())
}

/// Create a basic skill by copying the instruction file content directly.
/// This is used as a fallback when AI generation fails (e.g., API key issues).
/// The skill will be functional but won't have the enhanced structure that AI provides.
pub fn create_fallback_skill_from_instruction(
    instruction_file_path: &str,
    working_dir: &str,
) -> Result<GeneratedSkill, String> {
    // Read instruction file
    let instruction_content = fs::read_to_string(instruction_file_path)
        .map_err(|e| format!("Failed to read instruction file: {}", e))?;

    // Extract filename for skill name
    let file_name = Path::new(instruction_file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?;

    let skill_name = sanitize_name(file_name);

    // Create a basic SKILL.md with the instruction content
    let skill_md = format!(
        "# {}\n\n{}\n",
        file_name.replace('_', " ").replace('-', " "),
        instruction_content
    );

    let skill_content = SkillContent {
        skill_name: skill_name.clone(),
        skill_md,
        reference_md: None,
        examples_md: None,
        scripts: Vec::new(),
    };

    // Create skill directory and write files
    let skill_path = create_skill_directory(working_dir, &skill_content)?;

    Ok(GeneratedSkill {
        skill_name,
        skill_path,
        source_file: instruction_file_path.to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Check if an error is an API authentication error
pub fn is_auth_error(error: &str) -> bool {
    error.contains("authentication_error")
        || error.contains("x-api-key")
        || error.contains("api_key")
        || error.contains("unauthorized")
        || error.contains("401")
}

/// Check if a skill already exists for a given instruction file
/// Returns the skill name if it exists, None otherwise
pub fn find_existing_skill_for_instruction(
    instruction_file: &str,
    working_dir: &str,
) -> Option<String> {
    use crate::utils::generator::sanitize_name;

    // Get the expected skill name from the instruction file name
    let file_stem = Path::new(instruction_file)
        .file_stem()
        .and_then(|s| s.to_str())?;

    let expected_skill_name = sanitize_name(file_stem);

    // Check if this skill already exists
    let skill_dir = Path::new(working_dir)
        .join(".claude")
        .join("skills")
        .join(&expected_skill_name);

    if skill_dir.exists() && skill_dir.join("SKILL.md").exists() {
        Some(expected_skill_name)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("API Guidelines"), "api-guidelines");
        assert_eq!(sanitize_name("my_cool_skill"), "my-cool-skill");
        assert_eq!(
            sanitize_name("test--multiple---dashes"),
            "test-multiple-dashes"
        );
        assert_eq!(sanitize_name("Special!@#Characters"), "special-characters");
    }

    #[test]
    fn test_parse_skill_response_json() {
        let response = r###"{"skill_name":"test-skill","skill_md":"# Test\nContent","reference_md":null,"examples_md":null,"scripts":[]}"###;
        let result = parse_skill_response(response, "fallback");
        assert!(result.is_ok());
        let skill = result.unwrap();
        assert_eq!(skill.skill_name, "test-skill");
    }

    #[test]
    fn test_parse_skill_response_markdown_wrapped() {
        let response = r###"
Here's the skill:

```json
{
  "skill_name": "test-skill",
  "skill_md": "# Test\nContent",
  "scripts": []
}
```
"###;
        let result = parse_skill_response(response, "fallback");
        assert!(result.is_ok());
        let skill = result.unwrap();
        assert_eq!(skill.skill_name, "test-skill");
    }
}
