// Skill Matcher - Match instruction files to task requirements
//
// This module scores and selects instruction files based on their relevance
// to a task analysis. It reads instruction file content and uses AI to
// determine which instructions are most relevant.

use super::task_analyzer::TaskAnalysis;
use crate::ai_client::{AIClient, ContentBlock, Message};
use crate::instruction_manager::{get_instruction_file_content, InstructionFileInfo};
use serde::{Deserialize, Serialize};

/// Result of matching an instruction file to a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Path to the instruction file
    pub instruction_path: String,
    /// Relevance score from 0.0 to 1.0
    pub relevance_score: f32,
    /// Keywords that matched between task and instruction
    pub matched_keywords: Vec<String>,
    /// Domains that matched
    pub matched_domains: Vec<String>,
    /// Brief rationale for the match
    pub rationale: String,
}

/// Threshold for including an instruction (0.3 = 30% relevance)
const RELEVANCE_THRESHOLD: f32 = 0.3;

/// Maximum number of instructions to include
const MAX_INSTRUCTIONS: usize = 10;

const INSTRUCTION_MATCHING_PROMPT: &str = r#"You are selecting relevant instruction files for a task.

TASK ANALYSIS:
```json
{task_analysis_json}
```

INSTRUCTION FILES TO EVALUATE:
{instruction_summaries}

For each instruction file, evaluate its relevance to the task based on:
1. Keyword overlap with task domains, technologies, actions, entities
2. How directly applicable the instruction content is to the task
3. Whether the instruction provides necessary guidance for completing the task

Return a JSON array with relevance scores (0.0 to 1.0):
```json
[
  {
    "instruction_path": "path/to/file.md",
    "relevance_score": 0.85,
    "matched_keywords": ["api", "rest", "endpoint"],
    "matched_domains": ["backend", "api"],
    "rationale": "Brief explanation of why this instruction is relevant"
  }
]
```

IMPORTANT:
- Only include instructions with relevance_score > 0.3
- Be selective - only include truly relevant instructions
- Return an empty array [] if no instructions are relevant
- Maximum 10 instructions in the response
"#;

/// Match instruction files to a task analysis
/// Returns a list of relevant instructions sorted by relevance score
pub async fn match_instructions(
    ai_client: &AIClient,
    task_analysis: &TaskAnalysis,
    available_instructions: &[InstructionFileInfo],
) -> Result<Vec<MatchResult>, String> {
    if available_instructions.is_empty() {
        return Ok(Vec::new());
    }

    // Build instruction summaries (name + first 500 chars of content)
    let mut instruction_summaries = String::new();
    for instruction in available_instructions {
        let content = match get_instruction_file_content(&instruction.path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files we can't read
        };

        // Truncate content for the prompt
        let preview: String = content.chars().take(500).collect();
        let preview = if content.len() > 500 {
            format!("{}...", preview)
        } else {
            preview
        };

        instruction_summaries.push_str(&format!(
            "\n---\nFILE: {}\nPATH: {}\nCONTENT PREVIEW:\n{}\n",
            instruction.name, instruction.relative_path, preview
        ));
    }

    let task_analysis_json = serde_json::to_string_pretty(task_analysis)
        .map_err(|e| format!("Failed to serialize task analysis: {}", e))?;

    let prompt = INSTRUCTION_MATCHING_PROMPT
        .replace("{task_analysis_json}", &task_analysis_json)
        .replace("{instruction_summaries}", &instruction_summaries);

    let response = ai_client
        .send_message(vec![Message {
            role: "user".to_string(),
            content: prompt,
        }])
        .await
        .map_err(|e| format!("AI request failed: {}", e))?;

    // Extract text from response
    let text = response
        .content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Parse JSON from response
    let json_text = extract_json_from_text(&text);

    let mut matches: Vec<MatchResult> = serde_json::from_str(&json_text)
        .map_err(|e| format!("Failed to parse match results: {}. Response: {}", e, text))?;

    // Filter by threshold and limit count
    matches.retain(|m| m.relevance_score >= RELEVANCE_THRESHOLD);
    matches.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(MAX_INSTRUCTIONS);

    // Resolve relative paths to absolute paths
    for m in &mut matches {
        // Find the matching instruction to get absolute path
        if let Some(instruction) = available_instructions
            .iter()
            .find(|i| i.relative_path == m.instruction_path || i.name == m.instruction_path)
        {
            m.instruction_path = instruction.path.clone();
        }
    }

    Ok(matches)
}

/// Simple keyword-based matching (fallback if AI fails)
pub fn match_instructions_simple(
    task_analysis: &TaskAnalysis,
    available_instructions: &[InstructionFileInfo],
) -> Result<Vec<MatchResult>, String> {
    let task_terms: Vec<String> = task_analysis
        .all_terms()
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let mut matches = Vec::new();

    for instruction in available_instructions {
        let content = match get_instruction_file_content(&instruction.path) {
            Ok(c) => c.to_lowercase(),
            Err(_) => continue,
        };

        let name_lower = instruction.name.to_lowercase();

        // Count matching terms
        let mut matched_keywords = Vec::new();
        for term in &task_terms {
            if content.contains(term) || name_lower.contains(term) {
                matched_keywords.push(term.clone());
            }
        }

        if !matched_keywords.is_empty() {
            // Calculate simple relevance score
            let score = (matched_keywords.len() as f32 / task_terms.len() as f32).min(1.0);

            if score >= RELEVANCE_THRESHOLD {
                matches.push(MatchResult {
                    instruction_path: instruction.path.clone(),
                    relevance_score: score,
                    matched_keywords: matched_keywords.clone(),
                    matched_domains: task_analysis
                        .domains
                        .iter()
                        .filter(|d| matched_keywords.contains(&d.to_lowercase()))
                        .cloned()
                        .collect(),
                    rationale: format!(
                        "Matched {} keywords: {}",
                        matched_keywords.len(),
                        matched_keywords.join(", ")
                    ),
                });
            }
        }
    }

    // Sort by relevance
    matches.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(MAX_INSTRUCTIONS);

    Ok(matches)
}

/// Extract JSON from text that might contain markdown code blocks
fn extract_json_from_text(text: &str) -> String {
    // Try ```json ... ```
    if let Some(start) = text.find("```json") {
        if let Some(content_start) = text[start..].find('\n') {
            let content_start = start + content_start + 1;
            if let Some(end) = text[content_start..].find("```") {
                return text[content_start..content_start + end].trim().to_string();
            }
        }
    }

    // Try ``` ... ```
    if let Some(start) = text.find("```") {
        if let Some(content_start) = text[start..].find('\n') {
            let content_start = start + content_start + 1;
            if let Some(end) = text[content_start..].find("```") {
                return text[content_start..content_start + end].trim().to_string();
            }
        }
    }

    // Try to find JSON array directly
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return text[start..=end].to_string();
        }
    }

    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_result_serialization() {
        let result = MatchResult {
            instruction_path: "test.md".to_string(),
            relevance_score: 0.75,
            matched_keywords: vec!["api".to_string(), "rest".to_string()],
            matched_domains: vec!["backend".to_string()],
            rationale: "Test match".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test.md"));
        assert!(json.contains("0.75"));
    }

    #[test]
    fn test_extract_json_array() {
        let text = r#"Here are the matches:
```json
[{"instruction_path": "test.md", "relevance_score": 0.8, "matched_keywords": [], "matched_domains": [], "rationale": "test"}]
```
"#;
        let json = extract_json_from_text(text);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }
}
