// Session-based expectation tracking for security monitoring
//
// When a user sends a prompt, we use LLM to generate initial expectations
// (expected tools, paths, commands), then check every tool call against
// those expectations and refine them as the session progresses.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::collector::{SecurityEvent, SecurityEventType};
use super::llm_analyzer::LLMAnalyzer;

/// Tracks expected behavior patterns for each agent session
pub struct SessionExpectations {
    /// Per-session expectations (agent_id -> expectations)
    sessions: HashMap<String, AgentSessionState>,
    /// Configuration for expectation checking
    config: ExpectationConfig,
}

/// Configuration for expectation checking
#[derive(Clone)]
pub struct ExpectationConfig {
    /// Enable prompt-seeded expectations
    pub enabled: bool,
    /// Paths that are ALWAYS forbidden regardless of expectations
    pub always_forbidden_paths: Vec<String>,
    /// Tools that require explicit expectation (high risk)
    pub high_risk_tools: Vec<String>,
    /// Whether to expand expectations based on observed behavior
    pub adaptive_expansion: bool,
}

impl Default for ExpectationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            always_forbidden_paths: vec![
                "/etc/shadow".to_string(),
                "/etc/passwd".to_string(),
                "~/.ssh/id_rsa".to_string(),
                "~/.ssh/id_ed25519".to_string(),
                "~/.aws/credentials".to_string(),
                "~/.gnupg/".to_string(),
            ],
            high_risk_tools: vec![
                "Bash".to_string(),
                "WebFetch".to_string(),
                "Write".to_string(),
            ],
            adaptive_expansion: true,
        }
    }
}

/// State for a single agent session
pub struct AgentSessionState {
    pub agent_id: String,
    pub working_dir: String,
    pub started_at: i64,
    pub original_prompt: String,

    // Initial expectations (from LLM analysis of prompt)
    pub initial_expectations: InitialExpectations,

    // Observed behavior (accumulated from tool calls)
    pub observed_tools: HashSet<String>,
    pub observed_paths: HashSet<String>,
    pub observed_bash_commands: Vec<String>,
    pub network_activity_seen: bool,
    pub destructive_ops_seen: bool,

    // Combined expectations (initial + observed if adaptive)
    pub allowed_tools: HashSet<String>,
    pub allowed_path_scope: PathScope,
    pub network_allowed: bool,
    pub destructive_allowed: bool,
}

/// LLM-generated expectations from analyzing the user prompt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitialExpectations {
    pub expected_tools: HashSet<String>,
    pub expected_path_patterns: Vec<String>,
    pub network_likely: bool,
    pub destructive_likely: bool,
    pub bash_patterns: Vec<String>,
    pub confidence: f32,
    pub reasoning: String,
}

impl Default for InitialExpectations {
    fn default() -> Self {
        Self {
            // Default to common safe tools
            expected_tools: ["Read", "Glob", "Grep", "Edit", "Write", "Task", "TodoWrite"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            expected_path_patterns: vec!["**/*".to_string()], // Allow all by default
            network_likely: false,
            destructive_likely: false,
            bash_patterns: vec![],
            confidence: 0.5,
            reasoning: "Default expectations (no LLM analysis)".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PathScope {
    /// Only working directory (strict)
    WorkingDirOnly(String),
    /// Working dir + subdirs
    WorkingDirAndChildren(String),
    /// Glob patterns from expectations
    SpecificPatterns(Vec<String>),
    /// Broad task with system access
    Unrestricted,
}

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
    pub fn ok() -> Self {
        Self {
            is_anomaly: false,
            anomaly_type: None,
            severity: AnomalySeverity::Info,
            explanation: String::new(),
            expected_context: None,
        }
    }

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    UnexpectedTool,
    PathOutOfScope,
    UnexpectedNetwork,
    UnexpectedDestructive,
    SuspiciousBashCommand,
    ForbiddenPath,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AnomalySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl AnomalySeverity {
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

impl SessionExpectations {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            config: ExpectationConfig::default(),
        }
    }

    pub fn with_config(config: ExpectationConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
        }
    }

    /// Initialize session with LLM-generated expectations from prompt
    pub async fn seed_from_prompt(
        &mut self,
        agent_id: &str,
        working_dir: &str,
        prompt: &str,
        llm: &LLMAnalyzer,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        // Call LLM to generate initial expectations
        let initial = llm.generate_expectations(prompt, working_dir).await?;

        let state = AgentSessionState {
            agent_id: agent_id.to_string(),
            working_dir: working_dir.to_string(),
            started_at: chrono::Utc::now().timestamp_millis(),
            original_prompt: prompt.to_string(),

            // Initialize from LLM expectations
            allowed_tools: initial.expected_tools.clone(),
            allowed_path_scope: PathScope::SpecificPatterns(initial.expected_path_patterns.clone()),
            network_allowed: initial.network_likely,
            destructive_allowed: initial.destructive_likely,
            initial_expectations: initial,

            // Empty observed (will fill as agent runs)
            observed_tools: HashSet::new(),
            observed_paths: HashSet::new(),
            observed_bash_commands: Vec::new(),
            network_activity_seen: false,
            destructive_ops_seen: false,
        };

        self.sessions.insert(agent_id.to_string(), state);
        Ok(())
    }

    /// Seed with default expectations (when LLM is not available)
    pub fn seed_default(&mut self, agent_id: &str, working_dir: &str, prompt: &str) {
        let initial = InitialExpectations::default();

        let state = AgentSessionState {
            agent_id: agent_id.to_string(),
            working_dir: working_dir.to_string(),
            started_at: chrono::Utc::now().timestamp_millis(),
            original_prompt: prompt.to_string(),

            allowed_tools: initial.expected_tools.clone(),
            allowed_path_scope: PathScope::WorkingDirAndChildren(working_dir.to_string()),
            network_allowed: initial.network_likely,
            destructive_allowed: initial.destructive_likely,
            initial_expectations: initial,

            observed_tools: HashSet::new(),
            observed_paths: HashSet::new(),
            observed_bash_commands: Vec::new(),
            network_activity_seen: false,
            destructive_ops_seen: false,
        };

        self.sessions.insert(agent_id.to_string(), state);
    }

    /// Check event against expectations AND update observed behavior
    pub fn check_and_update(
        &mut self,
        agent_id: &str,
        event: &SecurityEvent,
    ) -> ExpectationCheckResult {
        if !self.config.enabled {
            return ExpectationCheckResult::ok();
        }

        // Extract forbidden paths from config before borrowing sessions
        let forbidden_paths = self.config.always_forbidden_paths.clone();

        let Some(state) = self.sessions.get(agent_id) else {
            // No expectations seeded - can't check
            return ExpectationCheckResult::ok();
        };

        let result = match &event.event_type {
            SecurityEventType::ToolUseRequest {
                tool_name,
                tool_input,
            } => Self::check_tool_use_static(&forbidden_paths, state, tool_name, tool_input),
            _ => ExpectationCheckResult::ok(),
        };

        // Update observed behavior (expand expectations based on actual usage)
        if self.config.adaptive_expansion {
            self.update_observed(agent_id, event);
        }

        result
    }

    /// Check a tool use event against expectations (static version to avoid borrow issues)
    fn check_tool_use_static(
        forbidden_paths: &[String],
        state: &AgentSessionState,
        tool_name: &str,
        tool_input: &serde_json::Value,
    ) -> ExpectationCheckResult {
        // Check 1: Always-forbidden paths (highest priority)
        if let Some(path) = extract_path_from_input(tool_input) {
            if Self::is_forbidden_path_static(forbidden_paths, &path) {
                return ExpectationCheckResult::anomaly(
                    AnomalyType::ForbiddenPath,
                    AnomalySeverity::Critical,
                    format!("Accessing forbidden path: {}", path),
                    Some("This path is always forbidden regardless of task".to_string()),
                );
            }
        }

        // Check 2: Tool in allowed set?
        let tool_ok = state.allowed_tools.contains(tool_name);

        // Check 3: Path in scope?
        let path = extract_path_from_input(tool_input);
        let path_ok = path
            .as_ref()
            .map(|p| state.is_path_in_scope(p))
            .unwrap_or(true);

        // Check 4: Network allowed?
        let network_ok = !is_network_tool(tool_name) || state.network_allowed;

        // Check 5: Destructive allowed?
        let destructive_ok =
            !is_destructive_tool(tool_name, tool_input) || state.destructive_allowed;

        // Build result based on checks
        if !tool_ok {
            let severity = categorize_tool_risk_static(tool_name);
            ExpectationCheckResult::anomaly(
                AnomalyType::UnexpectedTool,
                severity,
                format!("Tool '{}' not expected for this task", tool_name),
                Some(format!(
                    "Expected tools: {:?}",
                    state.initial_expectations.expected_tools
                )),
            )
        } else if !path_ok {
            ExpectationCheckResult::anomaly(
                AnomalyType::PathOutOfScope,
                AnomalySeverity::High,
                format!("Path '{}' outside expected scope", path.unwrap()),
                Some(format!(
                    "Expected patterns: {:?}",
                    state.initial_expectations.expected_path_patterns
                )),
            )
        } else if !network_ok {
            ExpectationCheckResult::anomaly(
                AnomalyType::UnexpectedNetwork,
                AnomalySeverity::Medium,
                "Network activity not expected for this task".to_string(),
                Some(format!(
                    "Task: {}",
                    truncate_string(&state.original_prompt, 100)
                )),
            )
        } else if !destructive_ok {
            ExpectationCheckResult::anomaly(
                AnomalyType::UnexpectedDestructive,
                AnomalySeverity::High,
                "Destructive operation not expected for this task".to_string(),
                Some(format!(
                    "Task: {}",
                    truncate_string(&state.original_prompt, 100)
                )),
            )
        } else {
            ExpectationCheckResult::ok()
        }
    }

    /// Check if a path is in the always-forbidden list (static version)
    fn is_forbidden_path_static(forbidden_paths: &[String], path: &str) -> bool {
        let normalized = normalize_path(path);
        for forbidden in forbidden_paths {
            let forbidden_normalized = normalize_path(forbidden);
            if normalized.starts_with(&forbidden_normalized) || normalized == forbidden_normalized {
                return true;
            }
        }
        false
    }

    /// Update observed behavior and potentially expand allowed scope
    fn update_observed(&mut self, agent_id: &str, event: &SecurityEvent) {
        let Some(state) = self.sessions.get_mut(agent_id) else {
            return;
        };

        if let SecurityEventType::ToolUseRequest {
            tool_name,
            tool_input,
        } = &event.event_type
        {
            // Track what we've seen
            state.observed_tools.insert(tool_name.clone());

            if let Some(path) = extract_path_from_input(tool_input) {
                state.observed_paths.insert(path);
            }

            if is_network_tool(tool_name) {
                state.network_activity_seen = true;
                // Once network is used, allow it going forward
                state.network_allowed = true;
            }

            if is_destructive_tool(tool_name, tool_input) {
                state.destructive_ops_seen = true;
            }

            // Expand allowed tools to include observed (for future checks)
            // This allows natural task evolution while still catching sudden shifts
            state.allowed_tools.insert(tool_name.clone());
        }
    }

    /// Remove session when agent is stopped
    pub fn remove_session(&mut self, agent_id: &str) {
        self.sessions.remove(agent_id);
    }

    /// Get session state for an agent (for debugging/display)
    pub fn get_session(&self, agent_id: &str) -> Option<&AgentSessionState> {
        self.sessions.get(agent_id)
    }
}

impl AgentSessionState {
    /// Check if a path is within the allowed scope
    pub fn is_path_in_scope(&self, path: &str) -> bool {
        let normalized = normalize_path(path);

        match &self.allowed_path_scope {
            PathScope::WorkingDirOnly(wd) => {
                let wd_normalized = normalize_path(wd);
                normalized == wd_normalized
            }
            PathScope::WorkingDirAndChildren(wd) => {
                let wd_normalized = normalize_path(wd);
                normalized.starts_with(&wd_normalized)
            }
            PathScope::SpecificPatterns(patterns) => {
                // Check if path matches any of the glob patterns
                for pattern in patterns {
                    if matches_glob_pattern(&normalized, pattern, &self.working_dir) {
                        return true;
                    }
                }
                // Also allow paths within working directory
                let wd_normalized = normalize_path(&self.working_dir);
                normalized.starts_with(&wd_normalized)
            }
            PathScope::Unrestricted => true,
        }
    }
}

// Helper functions

fn extract_path_from_input(tool_input: &serde_json::Value) -> Option<String> {
    // Try common path fields
    tool_input
        .get("file_path")
        .or_else(|| tool_input.get("path"))
        .or_else(|| tool_input.get("directory"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn is_network_tool(tool_name: &str) -> bool {
    matches!(tool_name, "WebFetch" | "WebSearch")
}

fn is_destructive_tool(tool_name: &str, tool_input: &serde_json::Value) -> bool {
    match tool_name {
        "Write" => true, // Creating/overwriting files
        "Edit" => false, // Edit is generally safe (modifies existing)
        "Bash" => {
            // Check for destructive bash commands
            if let Some(cmd) = tool_input.get("command").and_then(|v| v.as_str()) {
                let cmd_lower = cmd.to_lowercase();
                cmd_lower.contains("rm ")
                    || cmd_lower.contains("rm\t")
                    || cmd_lower.contains("rmdir")
                    || cmd_lower.contains("mv ")
                    || cmd_lower.contains("dd ")
                    || cmd_lower.contains("> /")
                    || cmd_lower.contains("truncate")
            } else {
                false
            }
        }
        _ => false,
    }
}

fn categorize_tool_risk(tool_name: &str, config: &ExpectationConfig) -> AnomalySeverity {
    if config.high_risk_tools.contains(&tool_name.to_string()) {
        AnomalySeverity::High
    } else {
        categorize_tool_risk_static(tool_name)
    }
}

/// Categorize tool risk without config (uses default high-risk list)
fn categorize_tool_risk_static(tool_name: &str) -> AnomalySeverity {
    match tool_name {
        "Bash" => AnomalySeverity::High,
        "WebFetch" | "WebSearch" => AnomalySeverity::Medium,
        "Write" => AnomalySeverity::Medium,
        _ => AnomalySeverity::Low,
    }
}

fn normalize_path(path: &str) -> String {
    // Expand ~ to home directory pattern
    let expanded = if path.starts_with("~/") {
        // Keep as pattern for matching
        path.to_string()
    } else {
        path.to_string()
    };

    // Remove trailing slashes for consistency
    expanded.trim_end_matches('/').to_string()
}

fn matches_glob_pattern(path: &str, pattern: &str, working_dir: &str) -> bool {
    // Simple glob matching
    // For production, consider using the `glob` or `globset` crate

    let pattern_normalized = if pattern.starts_with('/') || pattern.starts_with("~/") {
        pattern.to_string()
    } else {
        // Relative pattern - prepend working directory
        format!("{}/{}", working_dir.trim_end_matches('/'), pattern)
    };

    // Handle ** (recursive)
    if pattern_normalized.contains("**") {
        let parts: Vec<&str> = pattern_normalized.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            let path_starts_ok = prefix.is_empty() || path.starts_with(prefix);
            let path_ends_ok = suffix.is_empty() || path.ends_with(suffix);

            return path_starts_ok && path_ends_ok;
        }
    }

    // Handle * (single level)
    if pattern_normalized.contains('*') {
        let parts: Vec<&str> = pattern_normalized.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];

            return path.starts_with(prefix) && path.ends_with(suffix);
        }
    }

    // Exact match
    path == pattern_normalized
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_scope_working_dir_only() {
        let state = AgentSessionState {
            agent_id: "test".to_string(),
            working_dir: "/home/user/project".to_string(),
            started_at: 0,
            original_prompt: "test".to_string(),
            initial_expectations: InitialExpectations::default(),
            observed_tools: HashSet::new(),
            observed_paths: HashSet::new(),
            observed_bash_commands: Vec::new(),
            network_activity_seen: false,
            destructive_ops_seen: false,
            allowed_tools: HashSet::new(),
            allowed_path_scope: PathScope::WorkingDirOnly("/home/user/project".to_string()),
            network_allowed: false,
            destructive_allowed: false,
        };

        assert!(state.is_path_in_scope("/home/user/project"));
        assert!(!state.is_path_in_scope("/home/user/project/subdir"));
        assert!(!state.is_path_in_scope("/etc/passwd"));
    }

    #[test]
    fn test_path_scope_with_children() {
        let state = AgentSessionState {
            agent_id: "test".to_string(),
            working_dir: "/home/user/project".to_string(),
            started_at: 0,
            original_prompt: "test".to_string(),
            initial_expectations: InitialExpectations::default(),
            observed_tools: HashSet::new(),
            observed_paths: HashSet::new(),
            observed_bash_commands: Vec::new(),
            network_activity_seen: false,
            destructive_ops_seen: false,
            allowed_tools: HashSet::new(),
            allowed_path_scope: PathScope::WorkingDirAndChildren("/home/user/project".to_string()),
            network_allowed: false,
            destructive_allowed: false,
        };

        assert!(state.is_path_in_scope("/home/user/project"));
        assert!(state.is_path_in_scope("/home/user/project/subdir"));
        assert!(state.is_path_in_scope("/home/user/project/src/main.rs"));
        assert!(!state.is_path_in_scope("/etc/passwd"));
        assert!(!state.is_path_in_scope("/home/user/other"));
    }

    #[test]
    fn test_glob_pattern_matching() {
        assert!(matches_glob_pattern(
            "/home/user/project/README.md",
            "*.md",
            "/home/user/project"
        ));
        assert!(matches_glob_pattern(
            "/home/user/project/src/main.rs",
            "src/**/*.rs",
            "/home/user/project"
        ));
        assert!(!matches_glob_pattern(
            "/etc/passwd",
            "*.md",
            "/home/user/project"
        ));
    }

    #[test]
    fn test_forbidden_paths() {
        let expectations = SessionExpectations::new();
        assert!(expectations.is_forbidden_path("/etc/shadow"));
        assert!(expectations.is_forbidden_path("/etc/passwd"));
        assert!(expectations.is_forbidden_path("~/.ssh/id_rsa"));
        assert!(!expectations.is_forbidden_path("/home/user/project/file.txt"));
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
}
