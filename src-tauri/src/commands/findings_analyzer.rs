// Findings analyzer for instruction wizard test output
//
// Analyzes agent test output to identify issues and generate recommendations.
// Consolidates the pattern matching logic that was previously spread across
// multiple similar for-loops.

use super::validation_patterns::{
    AUTH_PATTERNS, AUTH_SUCCESS_EXCLUSIONS, MISSING_TOOL_PATTERNS, PERMISSION_PATTERNS,
    SUCCESS_PATTERNS,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of finding identified in test output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FindingType {
    MissingTool,
    AuthRequired,
    PermissionDenied,
    EnvironmentSetup,
    InstructionAmbiguity,
    SuccessPattern,
    Other,
}

/// A single finding from analyzing test output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFinding {
    pub id: String,
    pub finding_type: FindingType,
    pub severity: String, // "critical", "warning", "info"
    pub title: String,
    pub description: String,
    pub resolution_hint: Option<String>,
}

/// Configuration for a pattern-based check
struct PatternCheck<'a> {
    patterns: &'a [&'a str],
    finding_type: FindingType,
    severity: &'a str,
    title: &'a str,
    description_template: &'a str,
    resolution_hint: Option<&'a str>,
    /// Patterns that, if present, should skip this finding
    exclusions: Option<&'a [&'a str]>,
}

/// Check output against a set of patterns and return a finding if matched
fn check_patterns(output_lower: &str, check: &PatternCheck) -> Option<TestFinding> {
    // Check exclusions first
    if let Some(exclusions) = check.exclusions {
        for exclusion in exclusions.iter() {
            if output_lower.contains(&exclusion.to_lowercase()) {
                return None;
            }
        }
    }

    // Check for pattern matches
    for pattern in check.patterns {
        if output_lower.contains(&pattern.to_lowercase()) {
            return Some(TestFinding {
                id: Uuid::new_v4().to_string(),
                finding_type: check.finding_type.clone(),
                severity: check.severity.to_string(),
                title: check.title.to_string(),
                description: check.description_template.replace("{}", pattern),
                resolution_hint: check.resolution_hint.map(String::from),
            });
        }
    }

    None
}

/// Analyze test output for common issues and patterns.
///
/// Checks for:
/// - Missing tools or dependencies
/// - Authentication/credential issues
/// - Permission problems
/// - Success indicators
///
/// Returns a list of findings with severity and resolution hints.
pub fn analyze_output_for_findings(output: &str) -> Vec<TestFinding> {
    let mut findings = Vec::new();
    let output_lower = output.to_lowercase();

    // Define all pattern checks
    let checks = [
        PatternCheck {
            patterns: MISSING_TOOL_PATTERNS,
            finding_type: FindingType::MissingTool,
            severity: "critical",
            title: "Missing Tool or Dependency",
            description_template: "The output contains '{}' which suggests a required tool or dependency is not installed.",
            resolution_hint: Some("Check if required tools are installed and in PATH. You may need to install dependencies."),
            exclusions: None,
        },
        PatternCheck {
            patterns: AUTH_PATTERNS,
            finding_type: FindingType::AuthRequired,
            severity: "critical",
            title: "Authentication Required",
            description_template: "The output mentions '{}' which suggests authentication or API credentials are needed.",
            resolution_hint: Some("Set up required API keys or credentials. Check environment variables or config files."),
            exclusions: Some(AUTH_SUCCESS_EXCLUSIONS),
        },
        PatternCheck {
            patterns: PERMISSION_PATTERNS,
            finding_type: FindingType::PermissionDenied,
            severity: "warning",
            title: "Permission Issue",
            description_template: "The output contains '{}' which indicates a permission problem.",
            resolution_hint: Some("Check file/directory permissions or run with appropriate privileges."),
            exclusions: None,
        },
    ];

    // Run each check (only report first match per category)
    for check in &checks {
        if let Some(finding) = check_patterns(&output_lower, check) {
            findings.push(finding);
        }
    }

    // Check for success patterns only if no critical issues found
    if !findings.iter().any(|f| f.severity == "critical") {
        let success_check = PatternCheck {
            patterns: SUCCESS_PATTERNS,
            finding_type: FindingType::SuccessPattern,
            severity: "info",
            title: "Success Indicator",
            description_template: "Found success indicator: '{}'",
            resolution_hint: None,
            exclusions: None,
        };

        if let Some(finding) = check_patterns(&output_lower, &success_check) {
            findings.push(finding);
        }
    }

    findings
}

/// Generate recommendations based on findings.
///
/// Provides actionable suggestions based on the types of issues found.
pub fn generate_recommendations(findings: &[TestFinding]) -> Vec<String> {
    let mut recommendations = Vec::new();

    let has_missing_tool = findings
        .iter()
        .any(|f| f.finding_type == FindingType::MissingTool);
    let has_auth_issue = findings
        .iter()
        .any(|f| f.finding_type == FindingType::AuthRequired);
    let has_success = findings
        .iter()
        .any(|f| f.finding_type == FindingType::SuccessPattern);

    if has_missing_tool {
        recommendations.push("Add installation instructions to the instruction file".to_string());
        recommendations
            .push("Consider listing required tools in a 'Prerequisites' section".to_string());
    }

    if has_auth_issue {
        recommendations
            .push("Include detailed authentication setup steps in the instruction".to_string());
        recommendations.push(
            "Consider adding environment variable requirements to a 'Setup' section".to_string(),
        );
        recommendations.push(
            "You may need to complete authentication setup before the instruction can be used"
                .to_string(),
        );
    }

    if has_success && !has_missing_tool && !has_auth_issue {
        recommendations
            .push("The instruction appears to work! Consider adding more test cases.".to_string());
    }

    if findings.is_empty() {
        recommendations.push(
            "No specific issues detected. Review the output to verify the test completed as expected."
                .to_string(),
        );
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_output_missing_tool() {
        let output = "Error: command not found: kubectl";
        let findings = analyze_output_for_findings(output);
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::MissingTool));
    }

    #[test]
    fn test_analyze_output_auth_required() {
        let output = "Error 401: Unauthorized - Please provide valid API credentials";
        let findings = analyze_output_for_findings(output);
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::AuthRequired));
    }

    #[test]
    fn test_analyze_output_auth_success_excluded() {
        let output = "User authenticated successfully with OAuth";
        let findings = analyze_output_for_findings(output);
        // Should not flag as auth required since it says "authenticated successfully"
        assert!(!findings
            .iter()
            .any(|f| f.finding_type == FindingType::AuthRequired));
    }

    #[test]
    fn test_analyze_output_success() {
        let output = "File created successfully!";
        let findings = analyze_output_for_findings(output);
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::SuccessPattern));
    }

    #[test]
    fn test_analyze_output_permission_denied() {
        let output = "Error: permission denied while accessing /etc/passwd";
        let findings = analyze_output_for_findings(output);
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::PermissionDenied));
    }

    #[test]
    fn test_generate_recommendations_with_auth_issue() {
        let findings = vec![TestFinding {
            id: "test".to_string(),
            finding_type: FindingType::AuthRequired,
            severity: "critical".to_string(),
            title: "Auth Required".to_string(),
            description: "Test".to_string(),
            resolution_hint: None,
        }];
        let recommendations = generate_recommendations(&findings);
        assert!(recommendations.iter().any(|r| r.contains("authentication")));
    }

    #[test]
    fn test_generate_recommendations_empty_findings() {
        let findings = vec![];
        let recommendations = generate_recommendations(&findings);
        assert!(recommendations
            .iter()
            .any(|r| r.contains("No specific issues")));
    }
}
