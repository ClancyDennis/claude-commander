// Context tracking for managing conversation token usage
//
// This module uses tiktoken for accurate token counting and tracks
// the context state throughout the conversation.

use std::sync::OnceLock;
use tiktoken_rs::CoreBPE;

use super::context_config::ContextConfig;

/// Global tokenizer instance (cl100k_base encoding for Claude/GPT-4)
static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();

/// Get or initialize the tokenizer
fn get_tokenizer() -> &'static CoreBPE {
    TOKENIZER.get_or_init(|| {
        tiktoken_rs::cl100k_base().expect("Failed to initialize cl100k_base tokenizer")
    })
}

/// Context state indicating how full the context is
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextState {
    /// Context usage is normal (< 75%)
    Normal,
    /// Context is filling up (75-90%)
    Warning,
    /// Context is nearly full (90-95%)
    Critical,
    /// Context is at overflow risk (> 95%)
    Overflow,
}

impl ContextState {
    /// Get a human-readable description of the state
    pub fn description(&self) -> &'static str {
        match self {
            ContextState::Normal => "normal",
            ContextState::Warning => "filling up",
            ContextState::Critical => "nearly full",
            ContextState::Overflow => "at overflow risk",
        }
    }

    /// Whether this state should trigger a warning in tool results
    pub fn should_warn(&self) -> bool {
        matches!(
            self,
            ContextState::Warning | ContextState::Critical | ContextState::Overflow
        )
    }
}

/// Tracks context token usage throughout the conversation
#[derive(Debug)]
pub struct ContextTracker {
    /// Configuration for context limits
    config: ContextConfig,
    /// Current estimated token count
    current_tokens: usize,
    /// Token count for system prompt (cached)
    system_prompt_tokens: usize,
}

impl ContextTracker {
    /// Create a new context tracker with the given configuration
    pub fn new(config: ContextConfig) -> Self {
        Self {
            config,
            current_tokens: 0,
            system_prompt_tokens: 0,
        }
    }

    /// Count tokens in a string using tiktoken
    pub fn count_tokens(text: &str) -> usize {
        let tokenizer = get_tokenizer();
        tokenizer.encode_with_special_tokens(text).len()
    }

    /// Set the system prompt and cache its token count
    pub fn set_system_prompt(&mut self, prompt: &str) {
        self.system_prompt_tokens = Self::count_tokens(prompt);
        self.current_tokens = self.system_prompt_tokens;
    }

    /// Record token usage from an API response
    pub fn record_usage(&mut self, input_tokens: u32, output_tokens: u32) {
        // Use the actual input tokens from the API response as the authoritative source
        // This is more accurate than our estimation
        self.current_tokens = (input_tokens + output_tokens) as usize;
    }

    /// Add tokens for a message being added to history
    pub fn add_message_tokens(&mut self, content: &str) {
        let tokens = Self::count_tokens(content);
        // Add overhead for message structure (role, etc.) ~4 tokens
        self.current_tokens += tokens + 4;
    }

    /// Reset token count after compaction, accounting for summary
    pub fn reset_after_compaction(
        &mut self,
        summary_tokens: usize,
        remaining_history_tokens: usize,
    ) {
        self.current_tokens = self.system_prompt_tokens + summary_tokens + remaining_history_tokens;
    }

    /// Get the current context state
    pub fn get_state(&self) -> ContextState {
        let available = self.config.available_tokens();
        let usage_ratio = self.current_tokens as f64 / available as f64;

        if usage_ratio >= 0.95 {
            ContextState::Overflow
        } else if usage_ratio >= self.config.critical_threshold_pct {
            ContextState::Critical
        } else if usage_ratio >= self.config.warning_threshold_pct {
            ContextState::Warning
        } else {
            ContextState::Normal
        }
    }

    /// Get the current token count
    pub fn current_tokens(&self) -> usize {
        self.current_tokens
    }

    /// Get the available token budget
    pub fn available_tokens(&self) -> usize {
        self.config.available_tokens()
    }

    /// Get context usage as a percentage
    pub fn usage_percent(&self) -> f64 {
        let available = self.config.available_tokens();
        if available == 0 {
            return 100.0;
        }
        (self.current_tokens as f64 / available as f64) * 100.0
    }

    /// Get remaining tokens before hitting the available limit
    pub fn remaining_tokens(&self) -> usize {
        self.config
            .available_tokens()
            .saturating_sub(self.current_tokens)
    }

    /// Check if compaction is needed (at critical threshold or above)
    pub fn needs_compaction(&self) -> bool {
        matches!(
            self.get_state(),
            ContextState::Critical | ContextState::Overflow
        )
    }

    /// Check if emergency compaction is needed (at overflow threshold)
    pub fn needs_emergency_compaction(&self) -> bool {
        matches!(self.get_state(), ContextState::Overflow)
    }

    /// Get the configuration
    pub fn config(&self) -> &ContextConfig {
        &self.config
    }

    /// Get context info for tool results
    pub fn get_context_info(&self) -> ContextInfo {
        ContextInfo {
            usage_percent: self.usage_percent(),
            current_tokens: self.current_tokens(),
            available_tokens: self.available_tokens(),
            remaining_tokens: self.remaining_tokens(),
            state: self.get_state(),
        }
    }
}

/// Context information for tool results
#[derive(Debug, Clone)]
pub struct ContextInfo {
    pub usage_percent: f64,
    pub current_tokens: usize,
    pub available_tokens: usize,
    pub remaining_tokens: usize,
    pub state: ContextState,
}

impl ContextInfo {
    /// Generate a warning message if applicable
    pub fn warning_message(&self) -> Option<String> {
        match self.state {
            ContextState::Normal => None,
            ContextState::Warning => Some(format!(
                "Context is {:.0}% full ({} of {} tokens). Consider completing or summarizing soon.",
                self.usage_percent, self.current_tokens, self.available_tokens
            )),
            ContextState::Critical => Some(format!(
                "Context is {:.0}% full ({} of {} tokens). Compaction will occur at next idle moment.",
                self.usage_percent, self.current_tokens, self.available_tokens
            )),
            ContextState::Overflow => Some(format!(
                "Context is {:.0}% full - OVERFLOW IMMINENT. Emergency compaction will occur.",
                self.usage_percent
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let tokens = ContextTracker::count_tokens("Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 10); // Should be about 4 tokens
    }

    #[test]
    fn test_context_states() {
        let config = ContextConfig::for_claude();
        let mut tracker = ContextTracker::new(config);

        assert_eq!(tracker.get_state(), ContextState::Normal);

        // Simulate high usage
        tracker.current_tokens = (tracker.available_tokens() as f64 * 0.80) as usize;
        assert_eq!(tracker.get_state(), ContextState::Warning);

        tracker.current_tokens = (tracker.available_tokens() as f64 * 0.92) as usize;
        assert_eq!(tracker.get_state(), ContextState::Critical);

        tracker.current_tokens = (tracker.available_tokens() as f64 * 0.96) as usize;
        assert_eq!(tracker.get_state(), ContextState::Overflow);
    }

    #[test]
    fn test_usage_percent() {
        let config = ContextConfig::for_claude();
        let mut tracker = ContextTracker::new(config);

        tracker.current_tokens = tracker.available_tokens() / 2;
        let percent = tracker.usage_percent();
        assert!((percent - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_needs_compaction() {
        let config = ContextConfig::for_claude();
        let mut tracker = ContextTracker::new(config);

        assert!(!tracker.needs_compaction());

        tracker.current_tokens = (tracker.available_tokens() as f64 * 0.92) as usize;
        assert!(tracker.needs_compaction());
    }
}
