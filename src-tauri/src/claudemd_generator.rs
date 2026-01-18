// CLAUDE.md Generator - Generate project-level CLAUDE.md files from instruction documents
//
// CLAUDE.md files provide project-specific context and instructions that are
// automatically read by Claude Code and the Claude Agent SDK when they run
// in a directory. They serve as persistent "memory" for your project.
//
// Location options:
// - Project-level: `CLAUDE.md` or `.claude/CLAUDE.md` in your working directory
// - User-level: `~/.claude/CLAUDE.md` for global instructions across all projects

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use crate::ai_client::AIClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedClaudeMd {
    pub file_path: String,
    pub source_files: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMdContent {
    pub content: String,
    #[serde(default)]
    pub sections: Vec<ClaudeMdSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMdSection {
    pub title: String,
    pub content: String,
}

const CLAUDEMD_GENERATION_PROMPT: &str = r###"You are a CLAUDE.md file generator for Claude Code. Convert the following instruction document(s) into a well-structured CLAUDE.md file that will provide project-specific context and guidelines.

## Input Document(s)

<documents>
{instruction_content}
</documents>

## Output Format

Generate a JSON object with the following structure:

```json
{
  "content": "The full CLAUDE.md content as markdown...",
  "sections": [
    {"title": "Section Name", "content": "Section content..."}
  ]
}
```

## Guidelines

### Purpose of CLAUDE.md
CLAUDE.md files serve as persistent "memory" for Claude when working on a project. They should contain:
- Coding guidelines and standards
- Project-specific context and conventions
- Common commands or workflows
- API conventions and patterns
- Testing requirements
- File structure guidelines
- Environment setup notes

### Content Structure
The markdown should include:

1. **# Project Guidelines** - Main header
2. **## Code Style** - Coding standards, formatting preferences
3. **## Architecture** - Project structure, patterns used
4. **## Commands** - Build, test, deploy commands
5. **## API Conventions** - If applicable, API design patterns
6. **## Testing** - Testing requirements and commands
7. **## Environment** - Setup, dependencies, configuration

### Important Rules
- Keep content concise and actionable
- Focus on things Claude needs to know to work effectively
- Include specific commands when available
- Avoid duplicating what's in code comments
- Use clear markdown formatting with headers, lists, and code blocks
- Don't include sensitive information (API keys, credentials)

### Example Output
```markdown
# Project Guidelines

## Code Style
- Use TypeScript strict mode
- Prefer functional components in React
- Always include JSDoc comments for public APIs
- Use kebab-case for file names

## Architecture
- Frontend: React + TypeScript
- Backend: Rust with Tauri
- State management: Zustand

## Commands
- Build: `npm run build`
- Dev server: `npm run dev`
- Type check: `npm run typecheck`
- Tests: `npm test`

## Testing
- Run `npm test` before committing
- Maintain >80% code coverage
- Use jest for unit tests
```

## Important
- Return ONLY the JSON object, no additional text or markdown wrapper
- Ensure the content field contains valid markdown
- Make the output practical and focused on what Claude needs to know
"###;

/// Generate a CLAUDE.md file from one or more instruction files
pub async fn generate_claudemd_from_instructions(
    instruction_file_paths: &[String],
    working_dir: &str,
    ai_client: &AIClient,
) -> Result<GeneratedClaudeMd, String> {
    // 1. Read all instruction files
    let mut combined_content = String::new();
    for (i, path) in instruction_file_paths.iter().enumerate() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read instruction file {}: {}", path, e))?;

        let file_name = Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        combined_content.push_str(&format!("\n### Source: {}\n\n{}\n", file_name, content));

        if i < instruction_file_paths.len() - 1 {
            combined_content.push_str("\n---\n");
        }
    }

    // 2. Construct prompt
    let prompt = CLAUDEMD_GENERATION_PROMPT.replace("{instruction_content}", &combined_content);

    // 3. Call AI to generate CLAUDE.md content
    let messages = vec![crate::ai_client::Message {
        role: "user".to_string(),
        content: prompt.clone(),
    }];

    let response = ai_client
        .send_message(messages)
        .await
        .map_err(|e| format!("AI generation failed: {}", e))?;

    // Extract text content from response
    let mut ai_response = String::new();
    for block in response.content {
        if let crate::ai_client::ContentBlock::Text { text } = block {
            ai_response.push_str(&text);
        }
    }

    // 4. Parse AI response into ClaudeMdContent
    let claudemd_content = parse_claudemd_response(&ai_response)?;

    // 5. Write CLAUDE.md file
    let file_path = write_claudemd_file(working_dir, &claudemd_content)?;

    // 6. Return metadata
    Ok(GeneratedClaudeMd {
        file_path,
        source_files: instruction_file_paths.to_vec(),
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn parse_claudemd_response(ai_response: &str) -> Result<ClaudeMdContent, String> {
    // Try to extract JSON from response (AI might wrap it in markdown)
    let json_str = if ai_response.trim().starts_with('{') {
        ai_response.trim()
    } else {
        // Try to extract from markdown code block
        if let Some(start_idx) = ai_response.find("```json") {
            let content_start = if let Some(newline_idx) = ai_response[start_idx..].find('\n') {
                start_idx + newline_idx + 1
            } else {
                start_idx + "```json".len()
            };

            if let Some(end_idx) = ai_response[content_start..].find("```") {
                let json_end = content_start + end_idx;
                ai_response[content_start..json_end].trim()
            } else {
                return Err("Could not find closing ``` for JSON block".to_string());
            }
        } else if let Some(start) = ai_response.find('{') {
            if let Some(end) = ai_response.rfind('}') {
                &ai_response[start..=end]
            } else {
                return Err("Could not find valid JSON in AI response".to_string());
            }
        } else {
            return Err("No JSON found in AI response".to_string());
        }
    };

    let claudemd_content: ClaudeMdContent = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse AI response as JSON: {}", e))?;

    // Validate required fields
    if claudemd_content.content.trim().is_empty() {
        return Err("Generated content is empty".to_string());
    }

    Ok(claudemd_content)
}

fn write_claudemd_file(working_dir: &str, content: &ClaudeMdContent) -> Result<String, String> {
    // Write to .claude/CLAUDE.md (preferred location for generated files)
    let claude_dir = Path::new(working_dir).join(".claude");

    // Create .claude directory if it doesn't exist
    fs::create_dir_all(&claude_dir)
        .map_err(|e| format!("Failed to create .claude directory: {}", e))?;

    let file_path = claude_dir.join("CLAUDE.md");

    // Write the content
    fs::write(&file_path, &content.content)
        .map_err(|e| format!("Failed to write CLAUDE.md: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Check if a CLAUDE.md file exists in the working directory
pub fn claudemd_exists(working_dir: &str) -> bool {
    let root_path = Path::new(working_dir).join("CLAUDE.md");
    let claude_dir_path = Path::new(working_dir).join(".claude").join("CLAUDE.md");

    root_path.exists() || claude_dir_path.exists()
}

/// Get the path to the CLAUDE.md file if it exists
pub fn get_claudemd_path(working_dir: &str) -> Option<String> {
    let claude_dir_path = Path::new(working_dir).join(".claude").join("CLAUDE.md");
    if claude_dir_path.exists() {
        return Some(claude_dir_path.to_string_lossy().to_string());
    }

    let root_path = Path::new(working_dir).join("CLAUDE.md");
    if root_path.exists() {
        return Some(root_path.to_string_lossy().to_string());
    }

    None
}

/// Read the content of the CLAUDE.md file
pub fn get_claudemd_content(working_dir: &str) -> Result<String, String> {
    let path = get_claudemd_path(working_dir)
        .ok_or_else(|| "No CLAUDE.md file found".to_string())?;

    fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read CLAUDE.md: {}", e))
}

/// Delete the CLAUDE.md file
pub fn delete_claudemd(working_dir: &str) -> Result<(), String> {
    let path = get_claudemd_path(working_dir)
        .ok_or_else(|| "No CLAUDE.md file found".to_string())?;

    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete CLAUDE.md: {}", e))
}

/// Append content to an existing CLAUDE.md file
pub fn append_to_claudemd(working_dir: &str, new_content: &str) -> Result<String, String> {
    let path = get_claudemd_path(working_dir);

    let file_path = match path {
        Some(p) => {
            // Read existing content and append
            let existing = fs::read_to_string(&p)
                .map_err(|e| format!("Failed to read existing CLAUDE.md: {}", e))?;

            let combined = format!("{}\n\n{}", existing.trim(), new_content);

            fs::write(&p, combined)
                .map_err(|e| format!("Failed to write CLAUDE.md: {}", e))?;

            p
        }
        None => {
            // Create new file
            let claude_dir = Path::new(working_dir).join(".claude");
            fs::create_dir_all(&claude_dir)
                .map_err(|e| format!("Failed to create .claude directory: {}", e))?;

            let file_path = claude_dir.join("CLAUDE.md");
            fs::write(&file_path, new_content)
                .map_err(|e| format!("Failed to write CLAUDE.md: {}", e))?;

            file_path.to_string_lossy().to_string()
        }
    };

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claudemd_response_json() {
        let response = "{\"content\": \"# Project Guidelines\", \"sections\": []}";
        let result = parse_claudemd_response(response);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.content.contains("Project Guidelines"));
    }

    #[test]
    fn test_parse_claudemd_response_markdown_wrapped() {
        let response = "
Here's the CLAUDE.md:

```json
{
  \"content\": \"# Project Guidelines\",
  \"sections\": []
}
```
";
        let result = parse_claudemd_response(response);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.content.contains("Project Guidelines"));
    }

    #[test]
    fn test_claudemd_exists_false() {
        let temp_dir = std::env::temp_dir().join("test_claudemd_exists");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        assert!(!claudemd_exists(temp_dir.to_str().unwrap()));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
