//! Parsing utilities for LLM analyzer responses.
//!
//! This module contains helper functions to parse JSON responses from
//! the LLM into strongly-typed analysis results.

use crate::ai_client::{AIResponse, ContentBlock};

use super::llm_analyzer::{
    AnalysisResult, RecommendedAction, RiskLevel, ThreatAssessment, ThreatType,
};
use super::pattern_matcher::Severity;

/// Parse the LLM response into an AnalysisResult
pub(crate) fn parse_analysis_response(
    response: AIResponse,
    batch_id: String,
    timestamp: i64,
) -> Result<AnalysisResult, String> {
    // Look for tool use in response
    for content in response.content {
        if let ContentBlock::ToolUse { name, input, .. } = content {
            if name == "report_threat_analysis" {
                return parse_tool_input(input, batch_id, timestamp);
            }
        }
    }

    // If no tool use, return a default "no threats" result
    Ok(AnalysisResult {
        batch_id,
        timestamp,
        threats_detected: vec![],
        overall_risk_level: RiskLevel::None,
        recommended_actions: vec![RecommendedAction::Continue],
        analysis_summary: "No security threats detected in this batch.".to_string(),
        confidence: 0.5, // Lower confidence when LLM didn't use the tool
    })
}

/// Parse the tool input into an AnalysisResult
pub(crate) fn parse_tool_input(
    input: serde_json::Value,
    batch_id: String,
    timestamp: i64,
) -> Result<AnalysisResult, String> {
    // Parse threats_detected
    let threats_detected: Vec<ThreatAssessment> = input
        .get("threats_detected")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(parse_threat_assessment)
                .collect()
        })
        .unwrap_or_default();

    // Parse overall_risk_level
    let overall_risk_level = input
        .get("overall_risk_level")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "Critical" => RiskLevel::Critical,
            "High" => RiskLevel::High,
            "Medium" => RiskLevel::Medium,
            "Low" => RiskLevel::Low,
            _ => RiskLevel::None,
        })
        .unwrap_or(RiskLevel::None);

    // Parse recommended_actions
    let recommended_actions: Vec<RecommendedAction> = input
        .get("recommended_actions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(parse_recommended_action)
                .collect()
        })
        .unwrap_or_else(|| vec![RecommendedAction::Continue]);

    // Parse analysis_summary
    let analysis_summary = input
        .get("analysis_summary")
        .and_then(|v| v.as_str())
        .unwrap_or("Analysis complete.")
        .to_string();

    // Parse confidence
    let confidence = input
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.8);

    Ok(AnalysisResult {
        batch_id,
        timestamp,
        threats_detected,
        overall_risk_level,
        recommended_actions,
        analysis_summary,
        confidence,
    })
}

/// Parse a single threat assessment from JSON
pub(crate) fn parse_threat_assessment(value: &serde_json::Value) -> Option<ThreatAssessment> {
    let event_id = value.get("event_id")?.as_str()?.to_string();

    let threat_type = value
        .get("threat_type")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "PromptInjection" => ThreatType::PromptInjection,
            "JailbreakAttempt" => ThreatType::JailbreakAttempt,
            "DataExfiltration" => ThreatType::DataExfiltration,
            "UnauthorizedAccess" => ThreatType::UnauthorizedAccess,
            "MaliciousCodeExecution" => ThreatType::MaliciousCodeExecution,
            "PrivilegeEscalation" => ThreatType::PrivilegeEscalation,
            "SystemManipulation" => ThreatType::SystemManipulation,
            "SocialEngineering" => ThreatType::SocialEngineering,
            "ChainedAttack" => ThreatType::ChainedAttack,
            _ => ThreatType::Unknown,
        })?;

    let severity = value
        .get("severity")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "Critical" => Severity::Critical,
            "High" => Severity::High,
            "Medium" => Severity::Medium,
            "Low" => Severity::Low,
            _ => Severity::Info,
        })?;

    let confidence = value
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.8);

    let explanation = value
        .get("explanation")
        .and_then(|v| v.as_str())
        .unwrap_or("No explanation provided")
        .to_string();

    let evidence: Vec<String> = value
        .get("evidence")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mitigations: Vec<String> = value
        .get("mitigations")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Try to get agent_id from JSON (LLM might provide it), fallback to empty string
    // The caller should populate this from the events if empty
    let agent_id = value
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(ThreatAssessment {
        event_id,
        agent_id,
        threat_type,
        severity,
        confidence,
        explanation,
        evidence,
        mitigations,
    })
}

/// Parse a recommended action from JSON
pub(crate) fn parse_recommended_action(value: &serde_json::Value) -> Option<RecommendedAction> {
    let action_type = value.get("action_type")?.as_str()?;

    Some(match action_type {
        "Continue" => RecommendedAction::Continue,
        "Alert" => RecommendedAction::Alert {
            message: value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Security alert")
                .to_string(),
        },
        "SuspendAgent" => RecommendedAction::SuspendAgent {
            agent_id: value.get("agent_id")?.as_str()?.to_string(),
        },
        "TerminateAgent" => RecommendedAction::TerminateAgent {
            agent_id: value.get("agent_id")?.as_str()?.to_string(),
        },
        "BlockOperation" => RecommendedAction::BlockOperation {
            event_id: value.get("event_id")?.as_str()?.to_string(),
            reason: value
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("Security concern")
                .to_string(),
        },
        "RequestHumanReview" => RecommendedAction::RequestHumanReview {
            events: value
                .get("events")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|e| e.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        },
        _ => RecommendedAction::Continue,
    })
}
