//! Anomaly detection types and utilities for security monitoring.
//!
//! This module provides types for representing anomalies, their severity,
//! and helper functions for detecting potentially dangerous operations.

use serde::{Deserialize, Serialize};

/// Result of checking an event against expectations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpectationCheckResult {
    pub is_anomaly: bool,
    pub anomaly_type: Option<AnomalyType>,
    pub severity: AnomalySeverity,
    pub explanation: String,
    pub expected_context: Option<String>,
}

impl ExpectationCheckResult {
    /// Create a result indicating no anomaly was detected
    pub fn ok() -> Self {
        Self {
            is_anomaly: false,
            anomaly_type: None,
            severity: AnomalySeverity::Info,
            explanation: String::new(),
            expected_context: None,
        }
    }

    /// Create a result indicating an anomaly was detected
    pub fn anomaly(
        anomaly_type: AnomalyType,
        severity: AnomalySeverity,
        explanation: String,
        expected_context: Option<String>,
    ) -> Self {
        Self {
            is_anomaly: true,
            anomaly_type: Some(anomaly_type),
            severity,
            explanation,
            expected_context,
        }
    }
}

/// Types of anomalies that can be detected
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    /// Tool was not expected for this task
    UnexpectedTool,
    /// Path is outside the expected scope
    PathOutOfScope,
    /// Network activity was not expected
    UnexpectedNetwork,
    /// Destructive operation was not expected
    UnexpectedDestructive,
    /// Suspicious bash command detected
    SuspiciousBashCommand,
    /// Path is in the always-forbidden list
    ForbiddenPath,
}

/// Severity levels for detected anomalies
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AnomalySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl AnomalySeverity {
    /// Convert severity to a numeric score (0.0 - 1.0)
    pub fn to_score(&self) -> f32 {
        match self {
            AnomalySeverity::Info => 0.1,
            AnomalySeverity::Low => 0.3,
            AnomalySeverity::Medium => 0.5,
            AnomalySeverity::High => 0.7,
            AnomalySeverity::Critical => 0.9,
        }
    }
}

/// Check if a tool is a network-related tool
pub fn is_network_tool(tool_name: &str) -> bool {
    matches!(tool_name, "WebFetch" | "WebSearch")
}

/// Check if a tool operation is destructive
pub fn is_destructive_tool(tool_name: &str, tool_input: &serde_json::Value) -> bool {
    match tool_name {
        "Write" => true, // Creating/overwriting files
        "Edit" => false, // Edit is generally safe (modifies existing)
        "Bash" => {
            // Check for destructive bash commands
            if let Some(cmd) = tool_input.get("command").and_then(|v| v.as_str()) {
                is_destructive_bash_command(cmd)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if a bash command is potentially destructive
pub fn is_destructive_bash_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();
    cmd_lower.contains("rm ")
        || cmd_lower.contains("rm\t")
        || cmd_lower.contains("rmdir")
        || cmd_lower.contains("mv ")
        || cmd_lower.contains("dd ")
        || cmd_lower.contains("> /")
        || cmd_lower.contains("truncate")
}

/// Categorize the risk level of a tool
pub fn categorize_tool_risk(tool_name: &str, high_risk_tools: &[String]) -> AnomalySeverity {
    if high_risk_tools.contains(&tool_name.to_string()) {
        AnomalySeverity::High
    } else {
        categorize_tool_risk_default(tool_name)
    }
}

/// Categorize tool risk using default high-risk classifications
pub fn categorize_tool_risk_default(tool_name: &str) -> AnomalySeverity {
    match tool_name {
        "Bash" => AnomalySeverity::High,
        "WebFetch" | "WebSearch" => AnomalySeverity::Medium,
        "Write" => AnomalySeverity::Medium,
        _ => AnomalySeverity::Low,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expectation_check_result_ok() {
        let result = ExpectationCheckResult::ok();
        assert!(!result.is_anomaly);
        assert!(result.anomaly_type.is_none());
    }

    #[test]
    fn test_expectation_check_result_anomaly() {
        let result = ExpectationCheckResult::anomaly(
            AnomalyType::UnexpectedTool,
            AnomalySeverity::High,
            "Unexpected tool usage".to_string(),
            Some("Expected: Read, Glob".to_string()),
        );
        assert!(result.is_anomaly);
        assert_eq!(result.anomaly_type, Some(AnomalyType::UnexpectedTool));
        assert_eq!(result.severity, AnomalySeverity::High);
    }

    #[test]
    fn test_anomaly_severity_to_score() {
        assert_eq!(AnomalySeverity::Info.to_score(), 0.1);
        assert_eq!(AnomalySeverity::Low.to_score(), 0.3);
        assert_eq!(AnomalySeverity::Medium.to_score(), 0.5);
        assert_eq!(AnomalySeverity::High.to_score(), 0.7);
        assert_eq!(AnomalySeverity::Critical.to_score(), 0.9);
    }

    #[test]
    fn test_is_network_tool() {
        assert!(is_network_tool("WebFetch"));
        assert!(is_network_tool("WebSearch"));
        assert!(!is_network_tool("Read"));
        assert!(!is_network_tool("Bash"));
    }

    #[test]
    fn test_is_destructive_tool() {
        let rm_input = serde_json::json!({"command": "rm -rf /tmp/test"});
        assert!(is_destructive_tool("Bash", &rm_input));

        let ls_input = serde_json::json!({"command": "ls -la"});
        assert!(!is_destructive_tool("Bash", &ls_input));

        let write_input = serde_json::json!({"file_path": "/tmp/test.txt"});
        assert!(is_destructive_tool("Write", &write_input));

        let edit_input = serde_json::json!({"file_path": "/tmp/test.txt"});
        assert!(!is_destructive_tool("Edit", &edit_input));
    }

    #[test]
    fn test_is_destructive_bash_command() {
        assert!(is_destructive_bash_command("rm -rf /tmp"));
        assert!(is_destructive_bash_command("rmdir foo"));
        assert!(is_destructive_bash_command("mv file1 file2"));
        assert!(is_destructive_bash_command("dd if=/dev/zero of=/tmp/file"));
        assert!(is_destructive_bash_command("echo foo > /etc/test"));
        assert!(is_destructive_bash_command("truncate -s 0 file"));

        assert!(!is_destructive_bash_command("ls -la"));
        assert!(!is_destructive_bash_command("cat file.txt"));
        assert!(!is_destructive_bash_command("grep pattern file"));
    }

    #[test]
    fn test_categorize_tool_risk() {
        let high_risk = vec!["CustomDanger".to_string()];

        assert_eq!(categorize_tool_risk("CustomDanger", &high_risk), AnomalySeverity::High);
        assert_eq!(categorize_tool_risk("Bash", &high_risk), AnomalySeverity::High);
        assert_eq!(categorize_tool_risk("WebFetch", &high_risk), AnomalySeverity::Medium);
        assert_eq!(categorize_tool_risk("Read", &high_risk), AnomalySeverity::Low);
    }

    #[test]
    fn test_categorize_tool_risk_default() {
        assert_eq!(categorize_tool_risk_default("Bash"), AnomalySeverity::High);
        assert_eq!(categorize_tool_risk_default("WebFetch"), AnomalySeverity::Medium);
        assert_eq!(categorize_tool_risk_default("WebSearch"), AnomalySeverity::Medium);
        assert_eq!(categorize_tool_risk_default("Write"), AnomalySeverity::Medium);
        assert_eq!(categorize_tool_risk_default("Read"), AnomalySeverity::Low);
        assert_eq!(categorize_tool_risk_default("Glob"), AnomalySeverity::Low);
    }
}
