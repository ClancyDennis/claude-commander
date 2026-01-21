//! Shared utilities for AI-based generator modules.
//!
//! This module provides common functions used across skill_generator,
//! subagent_generator, and claudemd_generator for extracting JSON from
//! AI responses, sanitizing names, and extracting text from response blocks.

use crate::ai_client::ContentBlock;

/// Extract JSON from an AI response that may be wrapped in markdown code blocks.
///
/// Handles three formats:
/// 1. Raw JSON starting with `{`
/// 2. JSON wrapped in ```json ... ``` code blocks
/// 3. JSON embedded somewhere in the text (finds first `{` to last `}`)
pub fn extract_json_from_response(response: &str) -> Result<&str, String> {
    let trimmed = response.trim();

    // Case 1: Raw JSON starting with {
    if trimmed.starts_with('{') {
        return Ok(trimmed);
    }

    // Case 2: JSON wrapped in ```json code block
    if let Some(start_idx) = response.find("```json") {
        let content_start = response[start_idx..]
            .find('\n')
            .map(|i| start_idx + i + 1)
            .unwrap_or(start_idx + "```json".len());

        let end_idx = response[content_start..]
            .find("```")
            .ok_or_else(|| "Could not find closing ``` for JSON block".to_string())?;

        return Ok(response[content_start..content_start + end_idx].trim());
    }

    // Case 3: JSON embedded in text
    let start = response
        .find('{')
        .ok_or_else(|| "No JSON found in AI response".to_string())?;
    let end = response
        .rfind('}')
        .ok_or_else(|| "Could not find valid JSON in AI response".to_string())?;

    Ok(&response[start..=end])
}

/// Sanitize a name to kebab-case format suitable for file/directory names.
///
/// Transforms the input by:
/// 1. Converting to lowercase
/// 2. Replacing whitespace, underscores, and special characters with hyphens
/// 3. Collapsing consecutive hyphens into single hyphens
/// 4. Removing leading/trailing hyphens
pub fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract text content from AI response content blocks.
///
/// Iterates through content blocks and concatenates all Text variants
/// into a single string.
pub fn extract_text_from_content_blocks(content_blocks: &[ContentBlock]) -> String {
    let mut result = String::new();
    for block in content_blocks {
        if let ContentBlock::Text { text } = block {
            result.push_str(text);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // JSON extraction tests
    #[test]
    fn test_extract_json_raw() {
        let response = r#"{"name": "test"}"#;
        let result = extract_json_from_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"name": "test"}"#);
    }

    #[test]
    fn test_extract_json_raw_with_whitespace() {
        let response = r#"  {"name": "test"}  "#;
        let result = extract_json_from_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"name": "test"}"#);
    }

    #[test]
    fn test_extract_json_markdown_wrapped() {
        let response = "Here's the JSON:\n\n```json\n{\"name\": \"test\"}\n```\n\nDone!";
        let result = extract_json_from_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"name": "test"}"#);
    }

    #[test]
    fn test_extract_json_embedded() {
        let response = "Some text before {\"name\": \"test\"} some text after";
        let result = extract_json_from_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"name": "test"}"#);
    }

    #[test]
    fn test_extract_json_no_json() {
        let response = "No JSON here at all";
        let result = extract_json_from_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No JSON found"));
    }

    #[test]
    fn test_extract_json_unclosed_code_block() {
        let response = "```json\n{\"name\": \"test\"}\n";
        let result = extract_json_from_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("closing ```"));
    }

    // Name sanitization tests
    #[test]
    fn test_sanitize_name_spaces() {
        assert_eq!(sanitize_name("Code Reviewer"), "code-reviewer");
    }

    #[test]
    fn test_sanitize_name_underscores() {
        assert_eq!(sanitize_name("my_cool_agent"), "my-cool-agent");
    }

    #[test]
    fn test_sanitize_name_multiple_dashes() {
        assert_eq!(
            sanitize_name("test--multiple---dashes"),
            "test-multiple-dashes"
        );
    }

    #[test]
    fn test_sanitize_name_special_chars() {
        assert_eq!(sanitize_name("Special!@#Characters"), "special-characters");
    }

    #[test]
    fn test_sanitize_name_leading_trailing() {
        assert_eq!(sanitize_name("-leading-trailing-"), "leading-trailing");
    }

    #[test]
    fn test_sanitize_name_mixed_case() {
        assert_eq!(sanitize_name("MyAgent"), "myagent");
    }

    // Content block extraction tests
    #[test]
    fn test_extract_text_single_block() {
        let blocks = vec![ContentBlock::Text {
            text: "Hello".to_string(),
        }];
        assert_eq!(extract_text_from_content_blocks(&blocks), "Hello");
    }

    #[test]
    fn test_extract_text_multiple_blocks() {
        let blocks = vec![
            ContentBlock::Text {
                text: "Hello ".to_string(),
            },
            ContentBlock::Text {
                text: "World".to_string(),
            },
        ];
        assert_eq!(extract_text_from_content_blocks(&blocks), "Hello World");
    }

    #[test]
    fn test_extract_text_mixed_blocks() {
        use serde_json::json;
        let blocks = vec![
            ContentBlock::Text {
                text: "Hello".to_string(),
            },
            ContentBlock::ToolUse {
                id: "1".to_string(),
                name: "test".to_string(),
                input: json!({}),
            },
            ContentBlock::Text {
                text: " World".to_string(),
            },
        ];
        assert_eq!(extract_text_from_content_blocks(&blocks), "Hello World");
    }

    #[test]
    fn test_extract_text_empty() {
        let blocks: Vec<ContentBlock> = vec![];
        assert_eq!(extract_text_from_content_blocks(&blocks), "");
    }
}
