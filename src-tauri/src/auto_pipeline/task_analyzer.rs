// Task Analyzer - Parse user requests to extract structured requirements
//
// This module analyzes a user's task request to understand what domains,
// technologies, and actions are involved. This information is then used
// to automatically select relevant instruction files.

use crate::ai_client::{AIClient, ContentBlock, Message};
use serde::{Deserialize, Serialize};

/// Structured analysis of a user's task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalysis {
    /// Domains involved (e.g., "api", "database", "frontend", "devops")
    pub domains: Vec<String>,
    /// Technologies mentioned or implied (e.g., "rust", "react", "postgres")
    pub technologies: Vec<String>,
    /// Actions required (e.g., "create", "modify", "delete", "test", "deploy")
    pub actions: Vec<String>,
    /// Entities/concepts mentioned (e.g., "user", "product", "endpoint")
    pub entities: Vec<String>,
    /// Keywords for matching to instructions
    pub keywords: Vec<String>,
    /// Estimated complexity: "simple", "moderate", "complex"
    pub complexity_estimate: String,
}

impl TaskAnalysis {
    /// Create an empty task analysis
    pub fn empty() -> Self {
        Self {
            domains: Vec::new(),
            technologies: Vec::new(),
            actions: Vec::new(),
            entities: Vec::new(),
            keywords: Vec::new(),
            complexity_estimate: "moderate".to_string(),
        }
    }

    /// Get all terms for matching (combines all fields)
    pub fn all_terms(&self) -> Vec<&str> {
        let mut terms: Vec<&str> = Vec::new();
        terms.extend(self.domains.iter().map(|s| s.as_str()));
        terms.extend(self.technologies.iter().map(|s| s.as_str()));
        terms.extend(self.actions.iter().map(|s| s.as_str()));
        terms.extend(self.entities.iter().map(|s| s.as_str()));
        terms.extend(self.keywords.iter().map(|s| s.as_str()));
        terms
    }
}

const TASK_ANALYSIS_PROMPT: &str = r#"Analyze this task request and extract structured information for matching against instruction files.

TASK REQUEST:
{user_request}

WORKING DIRECTORY:
{working_dir}

Extract the following (be thorough but precise):

1. **domains**: Technical domains involved. Examples: "api", "database", "frontend", "backend", "devops", "testing", "security", "data-processing", "file-io", "networking"

2. **technologies**: Programming languages, frameworks, tools mentioned or implied. Examples: "rust", "python", "react", "postgres", "docker", "aws", "git"

3. **actions**: What needs to be done. Examples: "create", "modify", "delete", "read", "write", "test", "deploy", "configure", "parse", "generate", "fetch", "compare"

4. **entities**: Key concepts, objects, or components mentioned. Examples: "user", "file", "report", "endpoint", "database", "config", "schema"

5. **keywords**: Other relevant terms that might match instruction file content. Include synonyms and related terms.

6. **complexity_estimate**: "simple" (single file, straightforward), "moderate" (few files, some logic), or "complex" (many files, significant logic)

Return ONLY a JSON object with these fields:
```json
{
  "domains": ["string"],
  "technologies": ["string"],
  "actions": ["string"],
  "entities": ["string"],
  "keywords": ["string"],
  "complexity_estimate": "string"
}
```
"#;

/// Analyze a task request to extract structured requirements
pub async fn analyze_task(
    ai_client: &AIClient,
    user_request: &str,
    working_dir: &str,
) -> Result<TaskAnalysis, String> {
    let prompt = TASK_ANALYSIS_PROMPT
        .replace("{user_request}", user_request)
        .replace("{working_dir}", working_dir);

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

    serde_json::from_str(&json_text)
        .map_err(|e| format!("Failed to parse task analysis: {}. Response: {}", e, text))
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

    // Try to find JSON object directly
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return text[start..=end].to_string();
        }
    }

    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_analysis_all_terms() {
        let analysis = TaskAnalysis {
            domains: vec!["api".to_string(), "database".to_string()],
            technologies: vec!["rust".to_string()],
            actions: vec!["create".to_string()],
            entities: vec!["user".to_string()],
            keywords: vec!["endpoint".to_string()],
            complexity_estimate: "moderate".to_string(),
        };

        let terms = analysis.all_terms();
        assert!(terms.contains(&"api"));
        assert!(terms.contains(&"rust"));
        assert!(terms.contains(&"create"));
        assert!(terms.contains(&"user"));
        assert!(terms.contains(&"endpoint"));
    }

    #[test]
    fn test_extract_json_from_markdown() {
        let text = r#"Here's the analysis:
```json
{"domains": ["api"], "technologies": ["rust"], "actions": ["create"], "entities": [], "keywords": [], "complexity_estimate": "simple"}
```
Done!"#;
        let json = extract_json_from_text(text);
        assert!(json.contains("\"domains\""));
        assert!(json.contains("[\"api\"]"));
    }

    #[test]
    fn test_extract_json_direct() {
        let text = r#"{"domains": ["api"], "technologies": [], "actions": [], "entities": [], "keywords": [], "complexity_estimate": "simple"}"#;
        let json = extract_json_from_text(text);
        assert_eq!(json, text);
    }
}
