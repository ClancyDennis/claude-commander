//! Builder pattern for SecurityMonitor initialization.
//!
//! This module provides a fluent builder API for constructing SecurityMonitor
//! instances, separating the concerns of component creation from the main
//! SecurityMonitor struct.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::ai_client::AIClient;
use crate::events::AppEventEmitter;
use crate::logger::Logger;

use super::llm_analyzer::LLMAnalyzer;
use super::pattern_matcher::PatternMatcher;
use super::response_handler::{ResponseConfig, ResponseHandler, RetryConfig};
use super::rules;
use super::session_expectations::SessionExpectations;
use super::{MonitoringProvider, SecurityConfig, SecurityMonitor};

/// Builder for constructing SecurityMonitor instances.
///
/// # Example
/// ```ignore
/// let monitor = SecurityMonitorBuilder::new()
///     .with_config(config)
///     .with_response_config(response_config)
///     .with_agent_manager(agent_manager)
///     .with_logger(logger)
///     .with_app_handle(app_handle)
///     .build()?;
/// ```
pub struct SecurityMonitorBuilder {
    config: Option<SecurityConfig>,
    response_config: Option<ResponseConfig>,
    agent_manager: Option<Arc<Mutex<AgentManager>>>,
    logger: Option<Arc<Logger>>,
    app_handle: Option<Arc<dyn AppEventEmitter>>,
    custom_rules: Option<Vec<super::pattern_matcher::DetectionRule>>,
    custom_ai_client: Option<AIClient>,
}

impl Default for SecurityMonitorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityMonitorBuilder {
    /// Create a new SecurityMonitorBuilder with default settings.
    pub fn new() -> Self {
        Self {
            config: None,
            response_config: None,
            agent_manager: None,
            logger: None,
            app_handle: None,
            custom_rules: None,
            custom_ai_client: None,
        }
    }

    /// Set the security configuration.
    pub fn with_config(mut self, config: SecurityConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the response handler configuration.
    pub fn with_response_config(mut self, config: ResponseConfig) -> Self {
        self.response_config = Some(config);
        self
    }

    /// Set the agent manager for agent control operations.
    pub fn with_agent_manager(mut self, manager: Arc<Mutex<AgentManager>>) -> Self {
        self.agent_manager = Some(manager);
        self
    }

    /// Set the logger for security event logging.
    pub fn with_logger(mut self, logger: Arc<Logger>) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Set the app handle for emitting events to the UI.
    pub fn with_app_handle(mut self, handle: Arc<dyn AppEventEmitter>) -> Self {
        self.app_handle = Some(handle);
        self
    }

    /// Set custom detection rules instead of using defaults.
    pub fn with_custom_rules(mut self, rules: Vec<super::pattern_matcher::DetectionRule>) -> Self {
        self.custom_rules = Some(rules);
        self
    }

    /// Set a custom AI client for LLM analysis.
    pub fn with_ai_client(mut self, client: AIClient) -> Self {
        self.custom_ai_client = Some(client);
        self
    }

    /// Build the SecurityMonitor instance.
    ///
    /// # Errors
    /// Returns an error if required components are missing or initialization fails.
    pub fn build(self) -> Result<SecurityMonitor, String> {
        // Get required components
        let agent_manager = self.agent_manager.ok_or("Agent manager is required")?;
        let logger = self.logger.ok_or("Logger is required")?;
        let app_handle = self.app_handle.ok_or("App handle is required")?;

        // Use provided configs or defaults
        let config = self.config.unwrap_or_default();
        let response_config = self.response_config.unwrap_or_default();

        // Build components
        let collector = Self::build_collector(&config);
        let pattern_matcher = Self::build_pattern_matcher(self.custom_rules)?;
        let llm_analyzer = Self::build_llm_analyzer(&config, self.custom_ai_client);
        let response_handler =
            Self::build_response_handler(agent_manager, logger, app_handle, response_config);
        let session_expectations = Self::build_session_expectations();

        Ok(SecurityMonitor {
            collector,
            pattern_matcher,
            llm_analyzer,
            response_handler,
            enabled: Arc::new(Mutex::new(config.enabled)),
            config,
            session_expectations,
        })
    }

    /// Build the event collector component.
    fn build_collector(
        config: &SecurityConfig,
    ) -> Arc<Mutex<super::collector::SecurityEventCollector>> {
        Arc::new(Mutex::new(super::collector::SecurityEventCollector::new(
            config.max_events,
            config.batch_size,
            config.batch_interval_ms,
        )))
    }

    /// Build the pattern matcher component.
    fn build_pattern_matcher(
        custom_rules: Option<Vec<super::pattern_matcher::DetectionRule>>,
    ) -> Result<PatternMatcher, String> {
        let rules = custom_rules.unwrap_or_else(rules::get_default_rules);
        PatternMatcher::new(rules)
    }

    /// Build the LLM analyzer component (optional).
    fn build_llm_analyzer(
        config: &SecurityConfig,
        custom_client: Option<AIClient>,
    ) -> Option<LLMAnalyzer> {
        // If custom client provided, use it
        if let Some(client) = custom_client {
            println!("Security monitor: Using custom AI client for LLM analyzer");
            return Some(LLMAnalyzer::new(client));
        }

        // Try to create LLM analyzer from config
        match config.monitoring_provider.create_client() {
            Ok(client) => {
                println!("Security monitor: LLM analyzer initialized");
                Some(LLMAnalyzer::new(client))
            }
            Err(e) => {
                eprintln!(
                    "Security monitor: LLM analyzer not available ({}). Using pattern matching only.",
                    e
                );
                None
            }
        }
    }

    /// Build the response handler component.
    fn build_response_handler(
        agent_manager: Arc<Mutex<AgentManager>>,
        logger: Arc<Logger>,
        app_handle: Arc<dyn AppEventEmitter>,
        config: ResponseConfig,
    ) -> ResponseHandler {
        ResponseHandler::new(agent_manager, logger, app_handle, config)
    }

    /// Build the session expectations tracker.
    fn build_session_expectations() -> Arc<Mutex<SessionExpectations>> {
        Arc::new(Mutex::new(SessionExpectations::new()))
    }
}

/// Factory functions for creating common SecurityMonitor configurations.
pub mod presets {
    use super::*;

    /// Create a SecurityMonitor with default settings.
    pub fn default_monitor(
        agent_manager: Arc<Mutex<AgentManager>>,
        logger: Arc<Logger>,
        app_handle: Arc<dyn AppEventEmitter>,
    ) -> Result<SecurityMonitor, String> {
        SecurityMonitorBuilder::new()
            .with_agent_manager(agent_manager)
            .with_logger(logger)
            .with_app_handle(app_handle)
            .build()
    }

    /// Create a SecurityMonitor with strict settings (auto-terminate on critical).
    pub fn strict_monitor(
        agent_manager: Arc<Mutex<AgentManager>>,
        logger: Arc<Logger>,
        app_handle: Arc<dyn AppEventEmitter>,
    ) -> Result<SecurityMonitor, String> {
        let response_config = ResponseConfig {
            auto_terminate_on_critical: true,
            auto_suspend_on_high: true,
            alert_on_medium: true,
            log_all_events: true,
            human_review_required_for_actions: false,
            retry_config: RetryConfig::default(),
        };

        SecurityMonitorBuilder::new()
            .with_response_config(response_config)
            .with_agent_manager(agent_manager)
            .with_logger(logger)
            .with_app_handle(app_handle)
            .build()
    }

    /// Create a SecurityMonitor that requires human review for all actions.
    pub fn human_review_monitor(
        agent_manager: Arc<Mutex<AgentManager>>,
        logger: Arc<Logger>,
        app_handle: Arc<dyn AppEventEmitter>,
    ) -> Result<SecurityMonitor, String> {
        let response_config = ResponseConfig {
            auto_terminate_on_critical: false,
            auto_suspend_on_high: false,
            alert_on_medium: true,
            log_all_events: true,
            human_review_required_for_actions: true,
            retry_config: RetryConfig::default(),
        };

        SecurityMonitorBuilder::new()
            .with_response_config(response_config)
            .with_agent_manager(agent_manager)
            .with_logger(logger)
            .with_app_handle(app_handle)
            .build()
    }

    /// Create a SecurityMonitor with a specific LLM provider.
    pub fn monitor_with_provider(
        agent_manager: Arc<Mutex<AgentManager>>,
        logger: Arc<Logger>,
        app_handle: Arc<dyn AppEventEmitter>,
        provider: MonitoringProvider,
    ) -> Result<SecurityMonitor, String> {
        let config = SecurityConfig {
            monitoring_provider: provider,
            ..Default::default()
        };

        SecurityMonitorBuilder::new()
            .with_config(config)
            .with_agent_manager(agent_manager)
            .with_logger(logger)
            .with_app_handle(app_handle)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests require mocking the dependencies
    // These tests verify the builder API works correctly

    #[test]
    fn test_builder_default() {
        let builder = SecurityMonitorBuilder::new();
        assert!(builder.config.is_none());
        assert!(builder.response_config.is_none());
    }

    #[test]
    fn test_builder_with_config() {
        let config = SecurityConfig {
            enabled: false,
            ..Default::default()
        };

        let builder = SecurityMonitorBuilder::new().with_config(config);
        assert!(builder.config.is_some());
        assert!(!builder.config.unwrap().enabled);
    }

    #[test]
    fn test_builder_missing_required() {
        let result = SecurityMonitorBuilder::new().build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Agent manager is required"));
    }
}
