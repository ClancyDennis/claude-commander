//! Security event collector with ring buffer for batch processing.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::anomaly_detection::ExpectationCheckResult;
use super::pattern_matcher::PatternMatch;

/// Security-relevant event extracted from agent activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub timestamp: i64,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub event_type: SecurityEventType,
    pub content: String,
    pub metadata: SecurityEventMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_matches: Option<Vec<PatternMatch>>,
    /// Result of checking this event against session expectations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_info: Option<ExpectationCheckResult>,
}

impl SecurityEvent {
    /// Create a new security event for a user prompt
    pub fn new_user_prompt(
        agent_id: &str,
        session_id: Option<&str>,
        prompt: &str,
        working_dir: &str,
        source: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            agent_id: agent_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            event_type: SecurityEventType::UserPrompt,
            content: prompt.to_string(),
            metadata: SecurityEventMetadata {
                working_dir: working_dir.to_string(),
                parent_tool_use_id: None,
                source: source.to_string(),
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    }

    /// Create a new security event for an agent response
    pub fn new_agent_response(
        agent_id: &str,
        session_id: Option<&str>,
        response: &str,
        working_dir: &str,
        source: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            agent_id: agent_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            event_type: SecurityEventType::AgentResponse,
            content: response.to_string(),
            metadata: SecurityEventMetadata {
                working_dir: working_dir.to_string(),
                parent_tool_use_id: None,
                source: source.to_string(),
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    }

    /// Create a new security event for a tool use request
    pub fn new_tool_request(
        agent_id: &str,
        session_id: Option<&str>,
        tool_name: &str,
        tool_input: serde_json::Value,
        working_dir: &str,
        source: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            agent_id: agent_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            event_type: SecurityEventType::ToolUseRequest {
                tool_name: tool_name.to_string(),
                tool_input,
            },
            content: String::new(), // Will be populated from tool_input
            metadata: SecurityEventMetadata {
                working_dir: working_dir.to_string(),
                parent_tool_use_id: None,
                source: source.to_string(),
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    }

    /// Create a new security event for a tool use result
    pub fn new_tool_result(
        agent_id: &str,
        session_id: Option<&str>,
        tool_name: &str,
        success: bool,
        result_content: &str,
        working_dir: &str,
        source: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            agent_id: agent_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            event_type: SecurityEventType::ToolUseResult {
                tool_name: tool_name.to_string(),
                success,
            },
            content: result_content.to_string(),
            metadata: SecurityEventMetadata {
                working_dir: working_dir.to_string(),
                parent_tool_use_id: None,
                source: source.to_string(),
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    }

    /// Create a security event for a file operation
    pub fn new_file_operation(
        agent_id: &str,
        session_id: Option<&str>,
        operation: FileOpType,
        path: &str,
        working_dir: &str,
        source: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            agent_id: agent_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            event_type: SecurityEventType::FileOperation {
                operation,
                path: path.to_string(),
            },
            content: path.to_string(),
            metadata: SecurityEventMetadata {
                working_dir: working_dir.to_string(),
                parent_tool_use_id: None,
                source: source.to_string(),
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    }

    /// Create a security event for command execution
    pub fn new_command_execution(
        agent_id: &str,
        session_id: Option<&str>,
        command: &str,
        working_dir: &str,
        source: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            agent_id: agent_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            event_type: SecurityEventType::CommandExecution {
                command: command.to_string(),
            },
            content: command.to_string(),
            metadata: SecurityEventMetadata {
                working_dir: working_dir.to_string(),
                parent_tool_use_id: None,
                source: source.to_string(),
            },
            risk_score: None,
            pattern_matches: None,
            anomaly_info: None,
        }
    }
}

/// Type of security event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SecurityEventType {
    UserPrompt,
    AgentResponse,
    ToolUseRequest {
        tool_name: String,
        tool_input: serde_json::Value,
    },
    ToolUseResult {
        tool_name: String,
        success: bool,
    },
    FileOperation {
        operation: FileOpType,
        path: String,
    },
    CommandExecution {
        command: String,
    },
    NetworkRequest {
        url: String,
    },
    SystemEvent,
}

/// File operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileOpType {
    Read,
    Write,
    Edit,
    Delete,
    Create,
}

/// Additional metadata for security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventMetadata {
    pub working_dir: String,
    pub parent_tool_use_id: Option<String>,
    pub source: String, // "ui", "meta", "pipeline", "pool"
}

/// Ring buffer that collects events for batch analysis
pub struct SecurityEventCollector {
    events: VecDeque<SecurityEvent>,
    max_events: usize,
    batch_size: usize,
    batch_interval_ms: u64,
    last_batch_time: i64,
}

impl SecurityEventCollector {
    /// Create a new collector with specified limits
    pub fn new(max_events: usize, batch_size: usize, batch_interval_ms: u64) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
            batch_size,
            batch_interval_ms,
            last_batch_time: 0,
        }
    }

    /// Add a new event to the buffer
    pub fn push(&mut self, event: SecurityEvent) {
        // Remove oldest events if at capacity
        while self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get batch of events ready for analysis
    /// Returns events if either:
    /// 1. Batch size threshold reached
    /// 2. Time interval elapsed and there are pending events
    /// 3. High-risk event detected (immediate analysis)
    pub fn get_batch_if_ready(&mut self, current_time: i64) -> Option<Vec<SecurityEvent>> {
        let time_elapsed = current_time - self.last_batch_time >= self.batch_interval_ms as i64;
        let batch_full = self.events.len() >= self.batch_size;
        let has_high_risk = self
            .events
            .iter()
            .any(|e| e.risk_score.map(|s| s > 0.7).unwrap_or(false));

        if has_high_risk || batch_full || (time_elapsed && !self.events.is_empty()) {
            self.last_batch_time = current_time;
            let batch: Vec<_> = self.events.drain(..).collect();
            Some(batch)
        } else {
            None
        }
    }

    /// Get current event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if collector is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Peek at recent events without removing them
    pub fn peek_recent(&self, count: usize) -> Vec<&SecurityEvent> {
        self.events.iter().rev().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_basic_operations() {
        let mut collector = SecurityEventCollector::new(100, 10, 5000);
        assert!(collector.is_empty());

        let event = SecurityEvent::new_user_prompt(
            "agent-1",
            Some("session-1"),
            "test prompt",
            "/home/user",
            "ui",
        );

        collector.push(event);
        assert_eq!(collector.len(), 1);
        assert!(!collector.is_empty());
    }

    #[test]
    fn test_collector_batch_by_size() {
        let mut collector = SecurityEventCollector::new(100, 5, 5000);

        for i in 0..5 {
            let event = SecurityEvent::new_user_prompt(
                &format!("agent-{}", i),
                None,
                "test",
                "/home",
                "ui",
            );
            collector.push(event);
        }

        let batch = collector.get_batch_if_ready(1000);
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 5);
        assert!(collector.is_empty());
    }

    #[test]
    fn test_collector_batch_by_time() {
        let mut collector = SecurityEventCollector::new(100, 100, 5000);

        let event = SecurityEvent::new_user_prompt("agent-1", None, "test", "/home", "ui");
        collector.push(event);

        // Not enough time elapsed
        let batch = collector.get_batch_if_ready(1000);
        assert!(batch.is_none());

        // Enough time elapsed
        let batch = collector.get_batch_if_ready(6000);
        assert!(batch.is_some());
    }

    #[test]
    fn test_collector_max_capacity() {
        let mut collector = SecurityEventCollector::new(5, 10, 5000);

        for i in 0..10 {
            let event = SecurityEvent::new_user_prompt(
                &format!("agent-{}", i),
                None,
                "test",
                "/home",
                "ui",
            );
            collector.push(event);
        }

        // Should only have 5 events (max capacity)
        assert_eq!(collector.len(), 5);
    }
}
