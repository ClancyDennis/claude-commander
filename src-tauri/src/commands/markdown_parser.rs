// Markdown and JSON parsing utilities for instruction wizard
//
// These functions extract content from AI responses that may be wrapped
// in markdown code blocks or contain other formatting.

/// Extract markdown content from text that may be wrapped in code blocks.
///
/// Attempts to extract content from:
/// 1. ` ```markdown ... ``` ` blocks
/// 2. ` ```md ... ``` ` blocks
/// 3. Generic ` ``` ... ``` ` blocks
/// 4. Returns trimmed input if no code blocks found
///
/// # Example
/// ```
/// use claude_commander_lib::commands::markdown_parser::extract_markdown_content;
///
/// let text = "```markdown\n# My Title\nContent here\n```";
/// let content = extract_markdown_content(text);
/// assert!(content.contains("My Title"));
/// ```
pub fn extract_markdown_content(text: &str) -> &str {
    // Try to find markdown in code blocks first
    if let Some(start) = text.find("```markdown") {
        if let Some(end) = text[start + 11..].find("```") {
            return &text[start + 11..start + 11 + end];
        }
    }

    if let Some(start) = text.find("```md") {
        if let Some(end) = text[start + 5..].find("```") {
            return &text[start + 5..start + 5 + end];
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

    // Return as-is if no code blocks
    text.trim()
}

/// Extract JSON from text that may be wrapped in code blocks.
///
/// Attempts to extract content from:
/// 1. ` ```json ... ``` ` blocks
/// 2. Generic ` ``` ... ``` ` blocks
/// 3. Raw JSON starting with `{` and ending with `}`
/// 4. Returns original text if no JSON found
///
/// # Example
/// ```
/// use claude_commander_lib::commands::markdown_parser::extract_json;
///
/// let text = r#"Here's the JSON:
/// ```json
/// {"key": "value"}
/// ```"#;
/// let json = extract_json(text);
/// assert!(json.contains("key"));
/// ```
pub fn extract_json(text: &str) -> &str {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown_content_with_markdown_block() {
        let text = "Some text before\n```markdown\n# Title\nContent\n```\nAfter";
        let result = extract_markdown_content(text);
        assert!(result.contains("# Title"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_extract_markdown_content_with_md_block() {
        let text = "```md\n# Title\n```";
        let result = extract_markdown_content(text);
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_extract_markdown_content_with_generic_block() {
        let text = "```\n# Title\nContent\n```";
        let result = extract_markdown_content(text);
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_extract_markdown_content_no_block() {
        let text = "  # Title\nContent  ";
        let result = extract_markdown_content(text);
        assert_eq!(result, "# Title\nContent");
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

    #[test]
    fn test_extract_json_generic_block() {
        let text = r#"```
{"key": "value"}
```"#;
        let result = extract_json(text);
        assert!(result.contains("key"));
    }

    #[test]
    fn test_extract_json_embedded_in_text() {
        let text = r#"Some text before {"key": "value"} some text after"#;
        let result = extract_json(text);
        assert_eq!(result, r#"{"key": "value"}"#);
    }
}
