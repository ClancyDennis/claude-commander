// Gemini Advisor Module
//
// Provides an external AI advisor (default: Gemini) that the orchestrator can consult
// after each pipeline step completes. The advisor reviews the step output and provides
// guidance on next steps, potential issues, and strategic recommendations.

use crate::ai_client::{AIClient, Message};

/// Advice returned by the advisor after reviewing a step
#[derive(Debug, Clone)]
pub struct AdvisorAdvice {
    /// The advisor's recommendation text
    pub recommendation: String,
    /// The model that provided the advice
    pub model: String,
    /// Token usage for this consultation
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// The advisor that wraps an AI client for consultation
pub struct Advisor {
    ai_client: AIClient,
}

impl Advisor {
    /// Try to create an advisor from environment configuration.
    /// Returns None if advisor is disabled or no API key is configured.
    pub fn from_env() -> Option<Self> {
        // Check if advisor is enabled
        let enabled = std::env::var("ADVISOR_ENABLED")
            .ok()
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if !enabled {
            eprintln!("[ADVISOR] Advisor is disabled (set ADVISOR_ENABLED=true to enable)");
            return None;
        }

        match AIClient::advisor_from_env() {
            Ok(client) => {
                eprintln!(
                    "[ADVISOR] Initialized with provider={}, model={}",
                    client.get_provider_name(),
                    client.get_model_name()
                );
                Some(Self { ai_client: client })
            }
            Err(e) => {
                eprintln!("[ADVISOR] Failed to initialize: {}", e);
                None
            }
        }
    }

    /// Consult the advisor after a pipeline step completes.
    ///
    /// Provides the advisor with context about:
    /// - The original user request
    /// - The current pipeline phase
    /// - The output from the completed step
    /// - The available next actions
    ///
    /// Returns advice on what the orchestrator should do next.
    pub async fn consult(
        &self,
        user_request: &str,
        phase_name: &str,
        step_output: &str,
        available_actions: &[&str],
        iteration: u8,
        max_iterations: u8,
    ) -> Result<AdvisorAdvice, String> {
        let actions_list = available_actions
            .iter()
            .map(|a| format!("- {}", a))
            .collect::<Vec<_>>()
            .join("\n");

        let system_prompt = format!(
            r#"You are a strategic advisor for an AI orchestration pipeline. Your role is to review
the output of each pipeline step and provide clear, concise guidance on what should happen next.

You are advising an orchestrator AI that manages a multi-phase pipeline:
1. **Skill Synthesis** - Reading instructions and generating skills
2. **Planning** - Creating an implementation plan
3. **Execution** - Building the implementation
4. **Verification** - Reviewing the implementation quality

Current iteration: {} of {} maximum.

Be direct and actionable. Focus on:
- Whether the step output indicates success or issues
- What the orchestrator should do next
- Any risks or concerns to watch for
- Strategic suggestions for improvement

Keep your response under 500 words. Do not repeat the step output back."#,
            iteration, max_iterations
        );

        let user_message = format!(
            r#"## Original User Request
{}

## Current Phase: {}

## Step Output
{}

## Available Next Actions
{}

Based on the step output above, what should the orchestrator do next? Provide your recommendation."#,
            user_request, phase_name, step_output, actions_list
        );

        let messages = vec![
            Message {
                role: "user".to_string(),
                content: format!("System instructions:\n\n{}", system_prompt),
            },
            Message {
                role: "assistant".to_string(),
                content: "I understand. I'll review the pipeline step output and provide strategic advice on next steps.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_message,
            },
        ];

        let response = self
            .ai_client
            .send_message(messages)
            .await
            .map_err(|e| format!("Advisor consultation failed: {}", e))?;

        // Extract text from response
        let mut recommendation = String::new();
        for block in &response.content {
            if let crate::ai_client::ContentBlock::Text { text } = block {
                if !recommendation.is_empty() {
                    recommendation.push('\n');
                }
                recommendation.push_str(text);
            }
        }

        if recommendation.is_empty() {
            recommendation = "No specific recommendation provided.".to_string();
        }

        Ok(AdvisorAdvice {
            recommendation,
            model: response.model,
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
        })
    }

    /// Get the provider and model name for display
    pub fn model_info(&self) -> (&str, &str) {
        (
            self.ai_client.get_provider_name(),
            self.ai_client.get_model_name(),
        )
    }
}
