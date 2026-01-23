//! Path matching utilities for security monitoring.
//!
//! This module provides path normalization, glob pattern matching,
//! and path scope checking for the session expectations system.

use serde::{Deserialize, Serialize};

/// Defines the allowed path scope for an agent session
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

impl PathScope {
    /// Check if a path is within this scope
    pub fn contains(&self, path: &str, working_dir: &str) -> bool {
        let normalized = normalize_path(path);

        match self {
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
                    if matches_glob_pattern(&normalized, pattern, working_dir) {
                        return true;
                    }
                }
                // Also allow paths within working directory
                let wd_normalized = normalize_path(working_dir);
                normalized.starts_with(&wd_normalized)
            }
            PathScope::Unrestricted => true,
        }
    }
}

/// Configuration for forbidden paths
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForbiddenPathConfig {
    /// Paths that are ALWAYS forbidden regardless of expectations
    pub paths: Vec<String>,
}

impl Default for ForbiddenPathConfig {
    fn default() -> Self {
        Self {
            paths: vec![
                "/etc/shadow".to_string(),
                "/etc/passwd".to_string(),
                "~/.ssh/id_rsa".to_string(),
                "~/.ssh/id_ed25519".to_string(),
                "~/.aws/credentials".to_string(),
                "~/.gnupg/".to_string(),
            ],
        }
    }
}

impl ForbiddenPathConfig {
    /// Check if a path is in the forbidden list
    pub fn is_forbidden(&self, path: &str) -> bool {
        let normalized = normalize_path(path);
        for forbidden in &self.paths {
            let forbidden_normalized = normalize_path(forbidden);
            if normalized.starts_with(&forbidden_normalized) || normalized == forbidden_normalized {
                return true;
            }
        }
        false
    }
}

/// Extract a path from tool input JSON
pub fn extract_path_from_input(tool_input: &serde_json::Value) -> Option<String> {
    // Try common path fields
    tool_input
        .get("file_path")
        .or_else(|| tool_input.get("path"))
        .or_else(|| tool_input.get("directory"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Normalize a path for consistent comparison
pub fn normalize_path(path: &str) -> String {
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

/// Match a path against a glob pattern
///
/// Supports:
/// - `**` for recursive matching
/// - `*` for single-level matching
/// - Exact path matching
pub fn matches_glob_pattern(path: &str, pattern: &str, working_dir: &str) -> bool {
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

            // Handle suffix with wildcards (e.g., "*.rs")
            let path_ends_ok = if suffix.is_empty() {
                true
            } else if suffix.starts_with('*') {
                // Extract extension pattern (e.g., "*.rs" -> ".rs")
                let ext = &suffix[1..];
                path.ends_with(ext)
            } else {
                path.ends_with(suffix)
            };

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

/// Truncate a string to a maximum length
pub(crate) fn truncate_string(s: &str, max_len: usize) -> String {
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
        let scope = PathScope::WorkingDirOnly("/home/user/project".to_string());

        assert!(scope.contains("/home/user/project", "/home/user/project"));
        assert!(!scope.contains("/home/user/project/subdir", "/home/user/project"));
        assert!(!scope.contains("/etc/passwd", "/home/user/project"));
    }

    #[test]
    fn test_path_scope_with_children() {
        let scope = PathScope::WorkingDirAndChildren("/home/user/project".to_string());

        assert!(scope.contains("/home/user/project", "/home/user/project"));
        assert!(scope.contains("/home/user/project/subdir", "/home/user/project"));
        assert!(scope.contains("/home/user/project/src/main.rs", "/home/user/project"));
        assert!(!scope.contains("/etc/passwd", "/home/user/project"));
        assert!(!scope.contains("/home/user/other", "/home/user/project"));
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
        let config = ForbiddenPathConfig::default();

        assert!(config.is_forbidden("/etc/shadow"));
        assert!(config.is_forbidden("/etc/passwd"));
        assert!(config.is_forbidden("~/.ssh/id_rsa"));
        assert!(!config.is_forbidden("/home/user/project/file.txt"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/home/user/"), "/home/user");
        assert_eq!(normalize_path("/home/user"), "/home/user");
        assert_eq!(normalize_path("~/test"), "~/test");
    }

    #[test]
    fn test_extract_path() {
        let input = serde_json::json!({"file_path": "/tmp/test.txt"});
        assert_eq!(extract_path_from_input(&input), Some("/tmp/test.txt".to_string()));

        let input = serde_json::json!({"path": "/tmp/other.txt"});
        assert_eq!(extract_path_from_input(&input), Some("/tmp/other.txt".to_string()));

        let input = serde_json::json!({"command": "ls"});
        assert_eq!(extract_path_from_input(&input), None);
    }
}
