// Context configuration for managing token limits and compaction thresholds
//
// This module provides provider-specific context limits and configurable
// thresholds for context compaction.

/// Configuration for context management
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContextConfig {
    /// Maximum context tokens for the model
    pub max_context_tokens: usize,
    /// Tokens reserved for system prompt
    pub system_prompt_reserve: usize,
    /// Tokens reserved for output generation
    pub output_reserve: usize,
    /// Warning threshold as percentage (0.0 - 1.0)
    pub warning_threshold_pct: f64,
    /// Critical threshold as percentage (0.0 - 1.0)
    pub critical_threshold_pct: f64,
    /// Maximum characters for tool output before truncation
    pub max_tool_output_chars: usize,
    /// Number of recent messages to always preserve during compaction
    pub preserve_recent_messages: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self::for_claude()
    }
}

impl ContextConfig {
    /// Create config for Claude models (200k context)
    pub fn for_claude() -> Self {
        Self {
            max_context_tokens: 200_000,
            system_prompt_reserve: 4_000,
            output_reserve: 4_096,
            warning_threshold_pct: 0.75,
            critical_threshold_pct: 0.90,
            max_tool_output_chars: 10_000,
            preserve_recent_messages: 6, // 3 user-assistant turns
        }
    }

    /// Create config for OpenAI GPT-4 models (128k context)
    #[allow(dead_code)]
    pub fn for_openai_gpt4() -> Self {
        Self {
            max_context_tokens: 128_000,
            system_prompt_reserve: 4_000,
            output_reserve: 4_096,
            warning_threshold_pct: 0.75,
            critical_threshold_pct: 0.90,
            max_tool_output_chars: 10_000,
            preserve_recent_messages: 6,
        }
    }

    /// Create config for OpenAI GPT-5 models (assumed larger context)
    pub fn for_openai_gpt5() -> Self {
        Self {
            max_context_tokens: 256_000,
            system_prompt_reserve: 4_000,
            output_reserve: 4_096,
            warning_threshold_pct: 0.75,
            critical_threshold_pct: 0.90,
            max_tool_output_chars: 10_000,
            preserve_recent_messages: 6,
        }
    }

    /// Create config based on provider name
    pub fn for_provider(provider: &str) -> Self {
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => Self::for_claude(),
            "openai" => Self::for_openai_gpt5(), // Default to GPT-5 limits
            _ => Self::for_claude(),             // Fallback to Claude limits
        }
    }

    /// Calculate available tokens for conversation (excluding reserves)
    pub fn available_tokens(&self) -> usize {
        self.max_context_tokens
            .saturating_sub(self.system_prompt_reserve)
            .saturating_sub(self.output_reserve)
    }

    /// Calculate the warning threshold in tokens
    #[allow(dead_code)]
    pub fn warning_threshold_tokens(&self) -> usize {
        (self.available_tokens() as f64 * self.warning_threshold_pct) as usize
    }

    /// Calculate the critical threshold in tokens
    #[allow(dead_code)]
    pub fn critical_threshold_tokens(&self) -> usize {
        (self.available_tokens() as f64 * self.critical_threshold_pct) as usize
    }

    /// Calculate the overflow threshold (95%)
    #[allow(dead_code)]
    pub fn overflow_threshold_tokens(&self) -> usize {
        (self.available_tokens() as f64 * 0.95) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_config() {
        let config = ContextConfig::for_claude();
        assert_eq!(config.max_context_tokens, 200_000);
        assert!(config.available_tokens() < config.max_context_tokens);
    }

    #[test]
    fn test_thresholds() {
        let config = ContextConfig::for_claude();
        let available = config.available_tokens();

        assert!(config.warning_threshold_tokens() < config.critical_threshold_tokens());
        assert!(config.critical_threshold_tokens() < available);
        assert!(config.overflow_threshold_tokens() < available);
    }

    #[test]
    fn test_provider_selection() {
        let claude = ContextConfig::for_provider("Anthropic");
        assert_eq!(claude.max_context_tokens, 200_000);

        let openai = ContextConfig::for_provider("openai");
        assert_eq!(openai.max_context_tokens, 256_000);
    }
}
