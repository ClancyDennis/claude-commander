// Instruction analysis commands - AI-assisted instruction file editing

use crate::ai_client::{ContentBlock, Message};
use crate::AppState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionCategory {
    Clarity,
    Structure,
    Completeness,
    Specificity,
    Formatting,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionSuggestion {
    pub id: String,
    pub category: SuggestionCategory,
    pub title: String,
    pub description: String,
    pub original_text: Option<String>,
    pub suggested_text: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionAnalysisResult {
    pub quality_score: u8,
    pub quality_summary: String,
    pub issues: Vec<InstructionIssue>,
    pub suggestions: Vec<InstructionSuggestion>,
    pub improved_content: Option<String>,
}

// Internal type for parsing AI response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AIAnalysisResponse {
    quality_score: u8,
    quality_summary: String,
    issues: Vec<AIIssue>,
    suggestions: Vec<AISuggestion>,
    improved_content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AIIssue {
    severity: String,
    title: String,
    description: String,
    line_start: Option<u32>,
    line_end: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AISuggestion {
    category: String,
    title: String,
    description: String,
    original_text: Option<String>,
    suggested_text: String,
    line_start: Option<u32>,
    line_end: Option<u32>,
}

// ============================================================================
// Commands
// ============================================================================

/// Analyze instruction content using AI
#[tauri::command]
pub async fn analyze_instruction_content(
    content: String,
    context: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<InstructionAnalysisResult, String> {
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    // Get the AI client from meta_agent
    let meta_agent = state.meta_agent.lock().await;
    let ai_client = meta_agent.get_ai_client();

    // Build the analysis prompt
    let system_prompt = build_analysis_system_prompt();
    let user_prompt = build_analysis_user_prompt(&content, context.as_deref());

    let messages = vec![Message {
        role: "user".to_string(),
        content: format!("{}\n\n{}", system_prompt, user_prompt),
    }];

    // Send to AI
    let response = ai_client
        .send_message(messages)
        .await
        .map_err(|e| format!("AI analysis failed: {}", e))?;

    // Parse the response into structured result
    parse_analysis_response(&response.content)
}

/// Apply selected suggestions to generate improved content
#[tauri::command]
pub async fn apply_instruction_suggestions(
    original_content: String,
    accepted_suggestion_ids: Vec<String>,
    analysis_result: InstructionAnalysisResult,
) -> Result<String, String> {
    // If all suggestions are accepted and we have improved_content, use it directly
    if let Some(ref improved) = analysis_result.improved_content {
        let all_accepted = analysis_result
            .suggestions
            .iter()
            .all(|s| accepted_suggestion_ids.contains(&s.id));

        if all_accepted && !analysis_result.suggestions.is_empty() {
            return Ok(improved.clone());
        }
    }

    // If no suggestions accepted, return original
    if accepted_suggestion_ids.is_empty() {
        return Ok(original_content);
    }

    // Otherwise, apply selected suggestions incrementally
    let mut content = original_content;

    // Filter and sort suggestions by line number (reverse) to avoid offset issues
    let mut suggestions_to_apply: Vec<_> = analysis_result
        .suggestions
        .iter()
        .filter(|s| accepted_suggestion_ids.contains(&s.id))
        .filter(|s| s.original_text.is_some()) // Only apply suggestions that have original text
        .collect();

    suggestions_to_apply.sort_by(|a, b| {
        b.line_start
            .unwrap_or(u32::MAX)
            .cmp(&a.line_start.unwrap_or(u32::MAX))
    });

    for suggestion in suggestions_to_apply {
        if let Some(ref original) = suggestion.original_text {
            // Simple text replacement
            content = content.replace(original, &suggestion.suggested_text);
        }
    }

    Ok(content)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn build_analysis_system_prompt() -> String {
    r#"You are an expert at analyzing and improving instruction files for AI agents.
Your task is to analyze the provided instruction content and provide:

1. A quality score (1-10) with brief justification
2. Any issues or problems (with severity: critical, warning, info)
3. Specific improvement suggestions
4. An improved version of the full content

Focus on:
- Clarity and specificity of instructions
- Completeness (does it cover edge cases?)
- Structure and organization
- Actionability (can an AI follow these instructions?)
- Avoiding ambiguity

Respond ONLY with valid JSON matching this schema (no markdown code blocks, just raw JSON):
{
  "qualityScore": number (1-10),
  "qualitySummary": "brief assessment string",
  "issues": [
    {
      "severity": "critical" | "warning" | "info",
      "title": "short title",
      "description": "detailed description",
      "lineStart": number | null,
      "lineEnd": number | null
    }
  ],
  "suggestions": [
    {
      "category": "clarity" | "structure" | "completeness" | "specificity" | "formatting" | "other",
      "title": "short title",
      "description": "what to improve and why",
      "originalText": "exact text to replace" | null,
      "suggestedText": "improved text",
      "lineStart": number | null,
      "lineEnd": number | null
    }
  ],
  "improvedContent": "full improved version of the instruction"
}"#
    .to_string()
}

fn build_analysis_user_prompt(content: &str, context: Option<&str>) -> String {
    let context_section = context
        .map(|c| format!("\n\nContext about this instruction:\n{}", c))
        .unwrap_or_default();

    format!(
        "Please analyze the following instruction content and provide improvements:{}

---
INSTRUCTION CONTENT:
---
{}
---

Remember: Respond with raw JSON only, no markdown formatting.",
        context_section, content
    )
}

fn parse_analysis_response(content: &[ContentBlock]) -> Result<InstructionAnalysisResult, String> {
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
    // Handle potential markdown code blocks
    let json_str = extract_json(&text);

    let ai_response: AIAnalysisResponse = serde_json::from_str(json_str.trim()).map_err(|e| {
        format!(
            "Failed to parse AI response as JSON: {}. Response was: {}",
            e,
            &text[..text.len().min(500)]
        )
    })?;

    // Convert to our result type with generated IDs
    Ok(InstructionAnalysisResult {
        quality_score: ai_response.quality_score.clamp(1, 10),
        quality_summary: ai_response.quality_summary,
        issues: ai_response
            .issues
            .into_iter()
            .map(|issue| InstructionIssue {
                id: Uuid::new_v4().to_string(),
                severity: parse_severity(&issue.severity),
                title: issue.title,
                description: issue.description,
                line_start: issue.line_start,
                line_end: issue.line_end,
            })
            .collect(),
        suggestions: ai_response
            .suggestions
            .into_iter()
            .map(|suggestion| InstructionSuggestion {
                id: Uuid::new_v4().to_string(),
                category: parse_category(&suggestion.category),
                title: suggestion.title,
                description: suggestion.description,
                original_text: suggestion.original_text,
                suggested_text: suggestion.suggested_text,
                line_start: suggestion.line_start,
                line_end: suggestion.line_end,
            })
            .collect(),
        improved_content: ai_response.improved_content,
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
        // Skip the language identifier line if present
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

fn parse_severity(s: &str) -> IssueSeverity {
    match s.to_lowercase().as_str() {
        "critical" => IssueSeverity::Critical,
        "warning" => IssueSeverity::Warning,
        "info" => IssueSeverity::Info,
        _ => IssueSeverity::Info,
    }
}

fn parse_category(s: &str) -> SuggestionCategory {
    match s.to_lowercase().as_str() {
        "clarity" => SuggestionCategory::Clarity,
        "structure" => SuggestionCategory::Structure,
        "completeness" => SuggestionCategory::Completeness,
        "specificity" => SuggestionCategory::Specificity,
        "formatting" => SuggestionCategory::Formatting,
        _ => SuggestionCategory::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_raw() {
        let text = r#"{"qualityScore": 5, "qualitySummary": "test"}"#;
        let result = extract_json(text);
        assert!(result.contains("qualityScore"));
    }

    #[test]
    fn test_extract_json_code_block() {
        let text = r#"Here's the analysis:
```json
{"qualityScore": 5, "qualitySummary": "test"}
```"#;
        let result = extract_json(text);
        assert!(result.contains("qualityScore"));
    }

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("critical"), IssueSeverity::Critical);
        assert_eq!(parse_severity("WARNING"), IssueSeverity::Warning);
        assert_eq!(parse_severity("info"), IssueSeverity::Info);
        assert_eq!(parse_severity("unknown"), IssueSeverity::Info);
    }

    #[test]
    fn test_parse_category() {
        assert_eq!(parse_category("clarity"), SuggestionCategory::Clarity);
        assert_eq!(parse_category("STRUCTURE"), SuggestionCategory::Structure);
        assert_eq!(parse_category("unknown"), SuggestionCategory::Other);
    }
}
