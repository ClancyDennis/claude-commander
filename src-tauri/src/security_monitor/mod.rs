//! Security Monitor module for detecting dangerous actions and prompt injections.
//!
//! This module provides real-time security monitoring for AI agent activities using
//! a hybrid approach: fast regex-based pattern matching for known threats combined
//! with LLM-based semantic analysis for sophisticated attack detection.
//!
//! The monitor also supports prompt-seeded expectation tracking: when a user sends
//! a prompt, the LLM generates initial expectations (expected tools, paths, commands),
//! then every tool call is checked against those expectations for anomaly detection.

pub mod anomaly_detection;
pub mod collector;
mod expectation_generator;
pub mod llm_analyzer;
mod parsing_utils;
pub mod path_matching;
pub mod pattern_matcher;
pub mod response_handler;
pub mod rules;
pub mod session_expectations;

pub use collector::{
    SecurityEvent, SecurityEventCollector, SecurityEventMetadata, SecurityEventType,
};
pub use llm_analyzer::{
    AnalysisContext, AnalysisResult, LLMAnalyzer, RecommendedAction, RiskLevel, ThreatAssessment,
    ThreatType,
};
pub use pattern_matcher::{DetectionRule, PatternMatch, PatternMatcher, Severity, ThreatCategory};
pub use response_handler::{ResponseConfig, ResponseHandler, SecurityAlertEvent};
pub use anomaly_detection::ExpectationCheckResult;
pub use session_expectations::{InitialExpectations, SessionExpectations};

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use crate::agent_manager::AgentManager;
use crate::ai_client::AIClient;
use crate::events::AppEventEmitter;
use crate::logger::Logger;

/// Configuration for the security monitor
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub batch_size: usize,
    pub batch_interval_ms: u64,
    pub max_events: usize,
    pub monitoring_provider: MonitoringProvider,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: 50,
            batch_interval_ms: 5000,
            max_events: 10000,
            monitoring_provider: MonitoringProvider::default(),
        }
    }
}

/// LLM provider configuration for security monitoring
#[derive(Debug, Clone)]
pub enum MonitoringProvider {
    /// Use Claude Haiku for monitoring (fast & cheap)
    ClaudeHaiku,
    /// Use GPT-4o-mini for monitoring (fast & cheap)
    GPT4oMini,
    /// Use a custom provider/model configuration
    Custom { provider: String, model: String },
}

impl Default for MonitoringProvider {
    fn default() -> Self {
        // Prefer Claude Haiku if ANTHROPIC_API_KEY is available
        if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            MonitoringProvider::ClaudeHaiku
        } else {
            MonitoringProvider::GPT4oMini
        }
    }
}

impl MonitoringProvider {
    /// Create an AIClient for this monitoring provider
    pub fn create_client(&self) -> Result<AIClient, String> {
        match self {
            MonitoringProvider::ClaudeHaiku => {
                let api_key =
                    std::env::var("ANTHROPIC_API_KEY").map_err(|_| "ANTHROPIC_API_KEY not set")?;
                Ok(AIClient::new(crate::ai_client::Provider::Claude {
                    api_key,
                    model: "claude-3-5-haiku-latest".to_string(),
                }))
            }
            MonitoringProvider::GPT4oMini => {
                let api_key =
                    std::env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;
                Ok(AIClient::new(crate::ai_client::Provider::OpenAI {
                    api_key,
                    model: "gpt-4o-mini".to_string(),
                }))
            }
            MonitoringProvider::Custom { provider, model } => {
                match provider.to_lowercase().as_str() {
                    "claude" | "anthropic" => {
                        let api_key = std::env::var("ANTHROPIC_API_KEY")
                            .map_err(|_| "ANTHROPIC_API_KEY not set")?;
                        Ok(AIClient::new(crate::ai_client::Provider::Claude {
                            api_key,
                            model: model.clone(),
                        }))
                    }
                    "openai" | "gpt" => {
                        let api_key = std::env::var("OPENAI_API_KEY")
                            .map_err(|_| "OPENAI_API_KEY not set")?;
                        Ok(AIClient::new(crate::ai_client::Provider::OpenAI {
                            api_key,
                            model: model.clone(),
                        }))
                    }
                    _ => Err(format!("Unknown provider: {}", provider)),
                }
            }
        }
    }
}

/// Main security monitor that orchestrates all security components
pub struct SecurityMonitor {
    collector: Arc<Mutex<SecurityEventCollector>>,
    pattern_matcher: PatternMatcher,
    llm_analyzer: Option<LLMAnalyzer>,
    response_handler: ResponseHandler,
    enabled: Arc<Mutex<bool>>,
    config: SecurityConfig,
    /// Session-based expectation tracking for prompt-seeded anomaly detection
    session_expectations: Arc<Mutex<SessionExpectations>>,
}

impl SecurityMonitor {
    /// Create a new SecurityMonitor with the given configuration
    pub fn new(
        agent_manager: Arc<Mutex<AgentManager>>,
        logger: Arc<Logger>,
        app_handle: Arc<dyn AppEventEmitter>,
        config: SecurityConfig,
        response_config: ResponseConfig,
    ) -> Result<Self, String> {
        let collector = Arc::new(Mutex::new(SecurityEventCollector::new(
            config.max_events,
            config.batch_size,
            config.batch_interval_ms,
        )));

        // Load default detection rules
        let rules = rules::get_default_rules();
        let pattern_matcher = PatternMatcher::new(rules)?;

        // Try to create LLM analyzer (optional - will work without it)
        let llm_analyzer = match config.monitoring_provider.create_client() {
            Ok(client) => {
                println!("✓ Security monitor LLM analyzer initialized");
                Some(LLMAnalyzer::new(client))
            }
            Err(e) => {
                eprintln!(
                    "⚠ Security monitor: LLM analyzer not available ({}). Using pattern matching only.",
                    e
                );
                None
            }
        };

        let response_handler =
            ResponseHandler::new(agent_manager, logger, app_handle, response_config);

        // Initialize session expectations tracker
        let session_expectations = Arc::new(Mutex::new(SessionExpectations::new()));

        Ok(Self {
            collector,
            pattern_matcher,
            llm_analyzer,
            response_handler,
            enabled: Arc::new(Mutex::new(config.enabled)),
            config,
            session_expectations,
        })
    }

    /// Called when user sends a prompt - seeds expectations for anomaly detection
    ///
    /// This analyzes the prompt using the LLM to predict expected tools, paths,
    /// and behaviors. Subsequent tool calls will be checked against these expectations.
    pub async fn on_user_prompt(&self, agent_id: &str, working_dir: &str, prompt: &str) {
        if !*self.enabled.lock().await {
            return;
        }

        let mut expectations = self.session_expectations.lock().await;

        // Try LLM-based expectation seeding if available
        if let Some(ref llm) = self.llm_analyzer {
            match expectations
                .seed_from_prompt(agent_id, working_dir, prompt, llm)
                .await
            {
                Ok(()) => {
                    println!(
                        "Security: Seeded expectations for agent {} from prompt",
                        agent_id
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Security: Failed to seed LLM expectations for {}: {}. Using defaults.",
                        agent_id, e
                    );
                    // Fall back to default expectations
                    expectations.seed_default(agent_id, working_dir, prompt);
                }
            }
        } else {
            // No LLM available, use default expectations
            expectations.seed_default(agent_id, working_dir, prompt);
        }
    }

    /// Process an incoming security event
    pub async fn process_event(&self, mut event: SecurityEvent) {
        if !*self.enabled.lock().await {
            return;
        }

        // Fast path: Pattern matching (synchronous)
        let pattern_matches = self.pattern_matcher.check(&event);

        // Calculate risk score based on pattern matches
        let mut risk_score: f32 = if pattern_matches.is_empty() {
            0.0
        } else {
            pattern_matches
                .iter()
                .map(|m| match m.severity {
                    Severity::Critical => 1.0,
                    Severity::High => 0.8,
                    Severity::Medium => 0.5,
                    Severity::Low => 0.3,
                    Severity::Info => 0.1,
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0)
        };

        // Store pattern matches in event metadata
        if !pattern_matches.is_empty() {
            event.pattern_matches = Some(pattern_matches);
        }

        // Expectation checking (fast, ~1ms per event, no LLM)
        let expectation_result = {
            let mut expectations = self.session_expectations.lock().await;
            expectations.check_and_update(&event.agent_id, &event)
        };

        // If anomaly detected, update risk score and store anomaly info
        if expectation_result.is_anomaly {
            // Combine risk scores (take max of pattern-based and expectation-based)
            risk_score = risk_score.max(expectation_result.severity.to_score());
            event.anomaly_info = Some(expectation_result);
        }

        // Set risk score
        event.risk_score = Some(risk_score);

        // Add event to collector
        let mut collector = self.collector.lock().await;
        collector.push(event);
    }

    /// Remove expectations for an agent (call when agent is stopped)
    pub async fn remove_agent_expectations(&self, agent_id: &str) {
        let mut expectations = self.session_expectations.lock().await;
        expectations.remove_session(agent_id);
    }

    /// Start the background analysis loop
    pub fn start_background_analysis(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(self.config.batch_interval_ms));

            loop {
                interval.tick().await;

                if !*self.enabled.lock().await {
                    continue;
                }

                let current_time = chrono::Utc::now().timestamp_millis();

                // Get batch if ready
                let batch = {
                    let mut collector = self.collector.lock().await;
                    collector.get_batch_if_ready(current_time)
                };

                if let Some(events) = batch {
                    if events.is_empty() {
                        continue;
                    }

                    // Collect all pattern matches from events
                    let all_pattern_matches: Vec<PatternMatch> = events
                        .iter()
                        .filter_map(|e| e.pattern_matches.clone())
                        .flatten()
                        .collect();

                    // Determine if we need LLM analysis
                    let has_suspicious_patterns = !all_pattern_matches.is_empty();
                    let large_batch = events.len() >= 10;
                    let should_analyze_with_llm =
                        self.llm_analyzer.is_some() && (has_suspicious_patterns || large_batch);

                    if should_analyze_with_llm {
                        // Build context
                        let context = AnalysisContext {
                            working_dir: events
                                .first()
                                .map(|e| e.metadata.working_dir.clone())
                                .unwrap_or_default(),
                            agent_source: events
                                .first()
                                .map(|e| e.metadata.source.clone())
                                .unwrap_or_default(),
                            time_range_start: events
                                .first()
                                .map(|e| {
                                    chrono::DateTime::from_timestamp_millis(e.timestamp)
                                        .map(|dt| dt.to_rfc3339())
                                        .unwrap_or_default()
                                })
                                .unwrap_or_default(),
                            time_range_end: events
                                .last()
                                .map(|e| {
                                    chrono::DateTime::from_timestamp_millis(e.timestamp)
                                        .map(|dt| dt.to_rfc3339())
                                        .unwrap_or_default()
                                })
                                .unwrap_or_default(),
                        };

                        // Run LLM analysis
                        if let Some(analyzer) = &self.llm_analyzer {
                            match analyzer
                                .analyze_batch(events, all_pattern_matches, context)
                                .await
                            {
                                Ok(analysis) => {
                                    if let Err(e) =
                                        self.response_handler.handle_analysis(analysis).await
                                    {
                                        eprintln!(
                                            "Security monitor: Failed to handle analysis: {}",
                                            e
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Security monitor: LLM analysis failed: {}", e);
                                }
                            }
                        }
                    } else if has_suspicious_patterns {
                        // Pattern-only analysis (no LLM)
                        let analysis =
                            self.create_pattern_only_analysis(&events, all_pattern_matches);
                        if let Err(e) = self.response_handler.handle_analysis(analysis).await {
                            eprintln!("Security monitor: Failed to handle analysis: {}", e);
                        }
                    }
                }
            }
        })
    }

    /// Create an analysis result from pattern matches only (when LLM is not available)
    fn create_pattern_only_analysis(
        &self,
        events: &[SecurityEvent],
        pattern_matches: Vec<PatternMatch>,
    ) -> AnalysisResult {
        // Build a map of event_id -> agent_id for quick lookup
        let event_agent_map: std::collections::HashMap<&str, &str> = events
            .iter()
            .map(|e| (e.id.as_str(), e.agent_id.as_str()))
            .collect();

        let threats: Vec<ThreatAssessment> = pattern_matches
            .iter()
            .map(|pm| {
                let event_id = pm.event_id.clone().unwrap_or_default();
                // Look up agent_id from the event, fallback to empty string
                let agent_id = event_agent_map
                    .get(event_id.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                ThreatAssessment {
                    event_id,
                    agent_id,
                    threat_type: match pm.category {
                        ThreatCategory::PromptInjection => ThreatType::PromptInjection,
                        ThreatCategory::DataExfiltration => ThreatType::DataExfiltration,
                        ThreatCategory::UnauthorizedFileAccess => ThreatType::UnauthorizedAccess,
                        ThreatCategory::DangerousCommand => ThreatType::MaliciousCodeExecution,
                        ThreatCategory::PrivilegeEscalation => ThreatType::PrivilegeEscalation,
                        ThreatCategory::SystemTampering => ThreatType::SystemManipulation,
                        ThreatCategory::NetworkAbuse => ThreatType::DataExfiltration,
                    },
                    severity: pm.severity.clone(),
                    confidence: pm.confidence,
                    explanation: format!("Pattern '{}' matched: {}", pm.rule_name, pm.matched_text),
                    evidence: vec![pm.matched_text.clone()],
                    mitigations: vec!["Review the flagged content".to_string()],
                }
            })
            .collect();

        let overall_risk = threats
            .iter()
            .map(|t| &t.severity)
            .max()
            .cloned()
            .map(|s| match s {
                Severity::Critical => RiskLevel::Critical,
                Severity::High => RiskLevel::High,
                Severity::Medium => RiskLevel::Medium,
                Severity::Low => RiskLevel::Low,
                Severity::Info => RiskLevel::None,
            })
            .unwrap_or(RiskLevel::None);

        let recommended_actions = if overall_risk >= RiskLevel::High {
            vec![RecommendedAction::Alert {
                message: format!("{} suspicious patterns detected", pattern_matches.len()),
            }]
        } else {
            vec![RecommendedAction::Continue]
        };

        AnalysisResult {
            batch_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            threats_detected: threats,
            overall_risk_level: overall_risk,
            recommended_actions,
            analysis_summary: format!(
                "Pattern matching detected {} potential threats in {} events (LLM analysis not available)",
                pattern_matches.len(),
                events.len()
            ),
            confidence: 0.7, // Lower confidence without LLM confirmation
        }
    }

    /// Enable or disable the monitor
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().await = enabled;
    }

    /// Check if monitor is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }

    /// Get current configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Get response handler for external access (e.g., handling review responses)
    pub fn get_response_handler(&self) -> &ResponseHandler {
        &self.response_handler
    }
}
