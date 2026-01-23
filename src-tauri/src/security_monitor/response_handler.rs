//! Response handler for executing security actions based on threat analysis.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::events::AppEventEmitter;
use crate::logger::Logger;

use super::llm_analyzer::{AnalysisResult, RecommendedAction, RiskLevel, ThreatAssessment};

/// Detailed threat information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetailEvent {
    pub event_id: String,
    pub agent_id: String,
    pub threat_type: String,
    pub severity: String,
    pub confidence: f32,
    pub explanation: String,
    pub evidence: Vec<String>,
    pub mitigations: Vec<String>,
}

impl From<&ThreatAssessment> for ThreatDetailEvent {
    fn from(threat: &ThreatAssessment) -> Self {
        Self {
            event_id: threat.event_id.clone(),
            agent_id: threat.agent_id.clone(),
            threat_type: format!("{:?}", threat.threat_type),
            severity: format!("{:?}", threat.severity),
            confidence: threat.confidence,
            explanation: threat.explanation.clone(),
            evidence: threat.evidence.clone(),
            mitigations: threat.mitigations.clone(),
        }
    }
}

/// Security alert event emitted to UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlertEvent {
    pub alert_id: String,
    /// Primary agent associated with this alert (first affected agent)
    pub agent_id: String,
    pub timestamp: i64,
    pub risk_level: String,
    pub title: String,
    pub description: String,
    pub affected_agents: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub requires_acknowledgment: bool,
    pub batch_id: String,
    /// Detailed threat assessments
    pub threats: Vec<ThreatDetailEvent>,
    /// Overall confidence score from analysis
    pub overall_confidence: f32,
}

/// Response handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseConfig {
    /// Automatically terminate agents on critical threats
    pub auto_terminate_on_critical: bool,
    /// Automatically suspend agents on high threats
    pub auto_suspend_on_high: bool,
    /// Show alerts for medium-level threats
    pub alert_on_medium: bool,
    /// Log all security events (including low/info)
    pub log_all_events: bool,
    /// Require human review before executing actions
    pub human_review_required_for_actions: bool,
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            auto_terminate_on_critical: true,
            auto_suspend_on_high: false, // Require human review by default
            alert_on_medium: true,
            log_all_events: true,
            human_review_required_for_actions: true,
        }
    }
}

/// Pending review item for human approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingReview {
    pub id: String,
    pub batch_id: String,
    pub analysis_summary: String,
    pub overall_risk_level: String,
    pub recommended_action: String,
    pub agent_id: Option<String>,
    pub created_at: i64,
}

/// Response handler that executes security actions
pub struct ResponseHandler {
    agent_manager: Arc<Mutex<AgentManager>>,
    logger: Arc<Logger>,
    app_handle: Arc<dyn AppEventEmitter>,
    config: ResponseConfig,
    pending_reviews: Arc<Mutex<Vec<PendingReview>>>,
}

impl ResponseHandler {
    /// Create a new response handler
    pub fn new(
        agent_manager: Arc<Mutex<AgentManager>>,
        logger: Arc<Logger>,
        app_handle: Arc<dyn AppEventEmitter>,
        config: ResponseConfig,
    ) -> Self {
        Self {
            agent_manager,
            logger,
            app_handle,
            config,
            pending_reviews: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Handle analysis results and execute appropriate responses
    pub async fn handle_analysis(&self, analysis: AnalysisResult) -> Result<(), String> {
        // Log the analysis
        if self.config.log_all_events {
            self.log_analysis(&analysis).await;
        }

        // Process based on risk level
        match &analysis.overall_risk_level {
            RiskLevel::Critical => {
                self.emit_alert(&analysis, true).await;

                if self.config.auto_terminate_on_critical
                    && !self.config.human_review_required_for_actions
                {
                    for action in &analysis.recommended_actions {
                        if let RecommendedAction::TerminateAgent { agent_id } = action {
                            self.terminate_agent(agent_id, &analysis.batch_id).await?;
                        }
                    }
                } else {
                    self.queue_for_review(&analysis).await;
                }
            }
            RiskLevel::High => {
                self.emit_alert(&analysis, true).await;

                if self.config.auto_suspend_on_high
                    && !self.config.human_review_required_for_actions
                {
                    for action in &analysis.recommended_actions {
                        if let RecommendedAction::SuspendAgent { agent_id } = action {
                            self.suspend_agent(agent_id, &analysis.batch_id).await?;
                        }
                    }
                } else {
                    self.queue_for_review(&analysis).await;
                }
            }
            RiskLevel::Medium => {
                if self.config.alert_on_medium {
                    self.emit_alert(&analysis, false).await;
                }
            }
            RiskLevel::Low | RiskLevel::None => {
                // Log only, no action needed
            }
        }

        Ok(())
    }

    /// Emit a security alert to the UI
    async fn emit_alert(&self, analysis: &AnalysisResult, requires_ack: bool) {
        // Collect unique affected agents directly from threats
        let affected_agents: Vec<String> = analysis
            .threats_detected
            .iter()
            .filter(|t| !t.agent_id.is_empty())
            .map(|t| t.agent_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Use first affected agent as primary, or empty string if none
        let primary_agent_id = affected_agents.first().cloned().unwrap_or_default();

        let alert = SecurityAlertEvent {
            alert_id: uuid::Uuid::new_v4().to_string(),
            agent_id: primary_agent_id,
            timestamp: chrono::Utc::now().timestamp_millis(),
            risk_level: format!("{:?}", analysis.overall_risk_level),
            title: format!("{:?} Security Threat Detected", analysis.overall_risk_level),
            description: analysis.analysis_summary.clone(),
            affected_agents,
            recommended_actions: analysis
                .recommended_actions
                .iter()
                .map(|a| format!("{:?}", a))
                .collect(),
            requires_acknowledgment: requires_ack,
            batch_id: analysis.batch_id.clone(),
            threats: analysis
                .threats_detected
                .iter()
                .map(ThreatDetailEvent::from)
                .collect(),
            overall_confidence: analysis.confidence,
        };

        if let Err(e) = self.app_handle.emit(
            "security:alert",
            serde_json::to_value(&alert).unwrap_or(serde_json::Value::Null),
        ) {
            eprintln!("Failed to emit security alert: {}", e);
        }
    }

    /// Terminate an agent due to security threat
    async fn terminate_agent(&self, agent_id: &str, batch_id: &str) -> Result<(), String> {
        let manager = self.agent_manager.lock().await;
        manager.stop_agent(agent_id).await?;

        self.logger
            .error(
                "security_monitor",
                &format!(
                    "SECURITY: Terminated agent {} due to critical threat (batch: {})",
                    agent_id, batch_id
                ),
                Some(agent_id.to_string()),
                None,
            )
            .await
            .ok();

        // Emit termination event
        let _ = self.app_handle.emit(
            "security:agent_terminated",
            serde_json::json!({
                "agent_id": agent_id,
                "batch_id": batch_id,
                "reason": "critical_security_threat"
            }),
        );

        Ok(())
    }

    /// Suspend an agent pending review
    async fn suspend_agent(&self, agent_id: &str, batch_id: &str) -> Result<(), String> {
        // For now, suspension is just logging + alerting
        // A full implementation would pause the agent's ability to receive prompts

        self.logger
            .warning(
                "security_monitor",
                &format!(
                    "SECURITY: Suspended agent {} pending review (batch: {})",
                    agent_id, batch_id
                ),
                Some(agent_id.to_string()),
                None,
            )
            .await
            .ok();

        // Emit suspension event
        let _ = self.app_handle.emit(
            "security:agent_suspended",
            serde_json::json!({
                "agent_id": agent_id,
                "batch_id": batch_id,
                "reason": "high_security_threat"
            }),
        );

        Ok(())
    }

    /// Queue analysis for human review
    async fn queue_for_review(&self, analysis: &AnalysisResult) {
        let mut pending = self.pending_reviews.lock().await;

        for action in &analysis.recommended_actions {
            let (action_str, agent_id) = match action {
                RecommendedAction::Continue => continue, // Don't queue "Continue" actions
                RecommendedAction::Alert { message } => (format!("Alert: {}", message), None),
                RecommendedAction::SuspendAgent { agent_id } => (
                    format!("Suspend agent {}", agent_id),
                    Some(agent_id.clone()),
                ),
                RecommendedAction::TerminateAgent { agent_id } => (
                    format!("Terminate agent {}", agent_id),
                    Some(agent_id.clone()),
                ),
                RecommendedAction::BlockOperation { event_id, reason } => {
                    (format!("Block {}: {}", event_id, reason), None)
                }
                RecommendedAction::RequestHumanReview { events } => {
                    (format!("Review events: {:?}", events), None)
                }
            };

            let review = PendingReview {
                id: uuid::Uuid::new_v4().to_string(),
                batch_id: analysis.batch_id.clone(),
                analysis_summary: analysis.analysis_summary.clone(),
                overall_risk_level: format!("{:?}", analysis.overall_risk_level),
                recommended_action: action_str,
                agent_id,
                created_at: chrono::Utc::now().timestamp_millis(),
            };

            pending.push(review.clone());

            // Emit pending review event
            let _ = self.app_handle.emit(
                "security:pending_review",
                serde_json::to_value(&review).unwrap_or(serde_json::Value::Null),
            );
        }
    }

    /// Log analysis results
    async fn log_analysis(&self, analysis: &AnalysisResult) {
        let metadata = serde_json::to_string(analysis).ok();

        match analysis.overall_risk_level {
            RiskLevel::Critical | RiskLevel::High => {
                self.logger
                    .error(
                        "security_monitor",
                        &analysis.analysis_summary,
                        None,
                        metadata,
                    )
                    .await
                    .ok();
            }
            RiskLevel::Medium => {
                self.logger
                    .warning(
                        "security_monitor",
                        &analysis.analysis_summary,
                        None,
                        metadata,
                    )
                    .await
                    .ok();
            }
            _ => {
                self.logger
                    .info(
                        "security_monitor",
                        &analysis.analysis_summary,
                        None,
                        metadata,
                    )
                    .await
                    .ok();
            }
        }
    }

    /// Get pending reviews
    pub async fn get_pending_reviews(&self) -> Vec<PendingReview> {
        self.pending_reviews.lock().await.clone()
    }

    /// Handle a review response from the user
    pub async fn handle_review_response(
        &self,
        review_id: &str,
        approved: bool,
    ) -> Result<(), String> {
        let mut pending = self.pending_reviews.lock().await;

        if let Some(pos) = pending.iter().position(|r| r.id == review_id) {
            let review = pending.remove(pos);

            if approved {
                // Execute the approved action
                if let Some(agent_id) = &review.agent_id {
                    if review.recommended_action.starts_with("Terminate") {
                        drop(pending); // Release lock before async call
                        self.terminate_agent(agent_id, &review.batch_id).await?;
                    } else if review.recommended_action.starts_with("Suspend") {
                        drop(pending);
                        self.suspend_agent(agent_id, &review.batch_id).await?;
                    }
                }
            }

            // Emit review completed event
            let _ = self.app_handle.emit(
                "security:review_completed",
                serde_json::json!({
                    "review_id": review_id,
                    "approved": approved
                }),
            );

            Ok(())
        } else {
            Err(format!("Review {} not found", review_id))
        }
    }

    /// Dismiss a pending review without action
    pub async fn dismiss_review(&self, review_id: &str) -> Result<(), String> {
        let mut pending = self.pending_reviews.lock().await;

        if let Some(pos) = pending.iter().position(|r| r.id == review_id) {
            pending.remove(pos);

            // Emit review dismissed event
            let _ = self.app_handle.emit(
                "security:review_dismissed",
                serde_json::json!({ "review_id": review_id }),
            );

            Ok(())
        } else {
            Err(format!("Review {} not found", review_id))
        }
    }

    /// Clear all pending reviews
    pub async fn clear_pending_reviews(&self) {
        let mut pending = self.pending_reviews.lock().await;
        pending.clear();
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ResponseConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ResponseConfig) {
        self.config = config;
    }
}
