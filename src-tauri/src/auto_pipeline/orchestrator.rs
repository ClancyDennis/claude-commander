// Orchestrator - LLM-based decision maker for the auto pipeline
//
// Responsibilities:
// a) Refine the original task to be more specific
// b) Generate answers to clarifying questions
// c) Decide on verification results (pass/fail/iterate/replan)
// d) Apply custom user preferences via system prompt

use crate::ai_client::{AIClient, ContentBlock, Message};
use serde::{Deserialize, Serialize};

use super::prompts::{
    DEFAULT_CUSTOM_INSTRUCTIONS, QNA_GENERATION_PROMPT, TASK_REFINEMENT_PROMPT,
    VERIFICATION_DECISION_PROMPT,
};

/// Orchestrator decision after verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OrchestratorDecision {
    /// Task completed successfully
    Complete,
    /// Need to iterate on the build (minor fixes needed)
    Iterate,
    /// Need to go back to planning (fundamental approach issue)
    Replan,
    /// Give up - max iterations reached or unrecoverable
    GiveUp,
}

/// Detailed decision with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    pub decision: OrchestratorDecision,
    pub reasoning: String,
    pub issues_to_fix: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Refined task output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedTask {
    pub original_request: String,
    pub refined_request: String,
    pub clarifications: Vec<String>,
    pub scope_boundaries: Vec<String>,
    pub success_criteria: Vec<String>,
}

/// The Orchestrator handles all LLM-based decisions for the pipeline
pub struct Orchestrator {
    ai_client: AIClient,
    /// Custom instructions/preferences as a free-form string for the system prompt
    /// Example: "Don't lint. Test API endpoints. Never modify src/config/. Max 3 iterations."
    custom_instructions: Option<String>,
    /// Maximum iterations before giving up
    max_iterations: u8,
}

impl Orchestrator {
    /// Create a new Orchestrator with default settings
    pub fn new() -> Result<Self, String> {
        let ai_client = AIClient::from_env()
            .map_err(|e| format!("Failed to create AI client for orchestrator: {}", e))?;

        Ok(Self {
            ai_client,
            custom_instructions: None,
            max_iterations: 5,
        })
    }

    /// Create an Orchestrator with custom instructions
    ///
    /// # Arguments
    /// * `custom_instructions` - Free-form instructions like "Don't lint. Test API endpoints. Never modify src/config/."
    /// * `max_iterations` - Maximum iterations before giving up (default 5)
    pub fn with_instructions(
        custom_instructions: Option<String>,
        max_iterations: Option<u8>,
    ) -> Result<Self, String> {
        let ai_client = AIClient::from_env()
            .map_err(|e| format!("Failed to create AI client for orchestrator: {}", e))?;

        Ok(Self {
            ai_client,
            custom_instructions,
            max_iterations: max_iterations.unwrap_or(5),
        })
    }

    /// Get max iterations
    pub fn max_iterations(&self) -> u8 {
        self.max_iterations
    }

    /// Get custom instructions
    pub fn custom_instructions(&self) -> Option<&str> {
        self.custom_instructions.as_deref()
    }

    /// Format custom instructions for inclusion in prompts
    /// Falls back to DEFAULT_CUSTOM_INSTRUCTIONS if none provided
    fn format_custom_instructions(&self) -> String {
        match &self.custom_instructions {
            Some(instructions) => format!("\nCUSTOM INSTRUCTIONS:\n{}\n", instructions),
            None => DEFAULT_CUSTOM_INSTRUCTIONS.to_string(),
        }
    }

    // =========================================================================
    // A) TASK REFINEMENT - Make the original request more specific
    // =========================================================================

    /// Refine a user request to be more specific and actionable
    pub async fn refine_task(
        &self,
        user_request: &str,
        working_dir: &str,
        codebase_context: Option<&str>,
    ) -> Result<RefinedTask, String> {
        let context_section = codebase_context
            .map(|c| format!("\nCODEBASE CONTEXT:\n{}\n", c))
            .unwrap_or_default();

        let custom_instructions = self.format_custom_instructions();

        let prompt = TASK_REFINEMENT_PROMPT
            .replace("{user_request}", user_request)
            .replace("{working_dir}", working_dir)
            .replace("{context_section}", &context_section)
            .replace("{custom_instructions}", &custom_instructions);

        let response = self.send_message(&prompt).await?;
        let json_text = extract_json_from_text(&response);

        serde_json::from_str(&json_text).map_err(|e| {
            format!(
                "Failed to parse refined task: {}. Response: {}",
                e, response
            )
        })
    }

    // =========================================================================
    // B) Q&A ANSWER GENERATION
    // =========================================================================

    /// Generate answers to clarifying questions based on the task context
    pub async fn generate_answers(
        &self,
        user_request: &str,
        refined_request: Option<&str>,
        questions: &[String],
    ) -> Result<Vec<String>, String> {
        if questions.is_empty() {
            return Ok(Vec::new());
        }

        let refined = refined_request.unwrap_or(user_request);
        let custom_instructions = self.format_custom_instructions();

        let questions_list = questions
            .iter()
            .enumerate()
            .map(|(i, q)| format!("{}. {}", i + 1, q))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = QNA_GENERATION_PROMPT
            .replace("{user_request}", user_request)
            .replace("{refined_request}", refined)
            .replace("{custom_instructions}", &custom_instructions)
            .replace("{questions_list}", &questions_list);

        let response = self.send_message(&prompt).await?;
        let json_text = extract_json_from_text(&response);

        serde_json::from_str(&json_text)
            .map_err(|e| format!("Failed to parse answers: {}. Response: {}", e, response))
    }

    // =========================================================================
    // C) VERIFICATION DECISION
    // =========================================================================

    /// Decide what to do based on verification results
    pub async fn decide_on_verification(
        &self,
        user_request: &str,
        plan: &str,
        qna: &str,
        build_output: &str,
        verification_output: &str,
        iteration_count: u8,
        previous_issues: Option<&[String]>,
    ) -> Result<DecisionResult, String> {
        let custom_instructions = self.format_custom_instructions();
        let max_iterations = self.max_iterations;

        let previous_issues_section = previous_issues
            .map(|issues| {
                format!(
                    "\nPREVIOUS ITERATION ISSUES (for thrash detection):\n{}\n",
                    issues
                        .iter()
                        .map(|i| format!("- {}", i))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .unwrap_or_default();

        let prompt = VERIFICATION_DECISION_PROMPT
            .replace("{user_request}", user_request)
            .replace("{plan}", plan)
            .replace("{qna}", qna)
            .replace("{build_output}", build_output)
            .replace("{verification_output}", verification_output)
            .replace("{iteration_count}", &iteration_count.to_string())
            .replace("{max_iterations}", &max_iterations.to_string())
            .replace("{previous_issues_section}", &previous_issues_section)
            .replace("{custom_instructions}", &custom_instructions);

        let response = self.send_message(&prompt).await?;
        let json_text = extract_json_from_text(&response);

        serde_json::from_str(&json_text)
            .map_err(|e| format!("Failed to parse decision: {}. Response: {}", e, response))
    }

    // =========================================================================
    // HELPER METHODS
    // =========================================================================

    /// Send a message to the AI and extract text response
    async fn send_message(&self, prompt: &str) -> Result<String, String> {
        let response = self
            .ai_client
            .send_message(vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }])
            .await
            .map_err(|e| format!("AI request failed: {}", e))?;

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

        Ok(text)
    }
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
    fn test_extract_json_from_markdown() {
        let text = r#"Here's the JSON:
```json
{"key": "value"}
```
Done!"#;
        assert_eq!(extract_json_from_text(text), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_array() {
        let text = r#"The answers are: ["one", "two", "three"]"#;
        assert_eq!(extract_json_from_text(text), r#"["one", "two", "three"]"#);
    }
}
