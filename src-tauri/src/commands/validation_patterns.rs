// Validation patterns for instruction wizard test output analysis
//
// These patterns are used to identify common issues in agent test outputs.
// Extracted from instruction_wizard.rs to reduce duplication and improve maintainability.

/// Patterns that indicate a required tool or dependency is missing
pub const MISSING_TOOL_PATTERNS: &[&str] = &[
    "command not found",
    "not recognized",
    "Cannot find module",
    "No such file or directory",
    "is not installed",
    "ModuleNotFoundError",
    "ImportError",
    "npm ERR!",
    "pip: command not found",
];

/// Patterns that indicate authentication or credentials are required
pub const AUTH_PATTERNS: &[&str] = &[
    "authentication",
    "unauthorized",
    "401",
    "403",
    "credentials",
    "token",
    "api key",
    "apikey",
    "access denied",
    "login required",
    "OAuth",
    "GOOGLE_APPLICATION_CREDENTIALS",
    "GH_TOKEN",
    "ANTHROPIC_API_KEY",
];

/// Patterns that indicate permission issues
pub const PERMISSION_PATTERNS: &[&str] = &[
    "permission denied",
    "EACCES",
    "Operation not permitted",
    "Access is denied",
];

/// Patterns that indicate successful execution
pub const SUCCESS_PATTERNS: &[&str] = &[
    "successfully",
    "completed",
    "done",
    "created",
    "saved",
    "connected",
    "authenticated",
];

/// Patterns that should be excluded when detecting auth issues
/// (these indicate auth succeeded, not failed)
pub const AUTH_SUCCESS_EXCLUSIONS: &[&str] = &["authenticated successfully", "login successful"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns_not_empty() {
        assert!(!MISSING_TOOL_PATTERNS.is_empty());
        assert!(!AUTH_PATTERNS.is_empty());
        assert!(!PERMISSION_PATTERNS.is_empty());
        assert!(!SUCCESS_PATTERNS.is_empty());
    }

    #[test]
    fn test_patterns_are_non_empty() {
        // Verify that all patterns are non-empty strings
        for pattern in MISSING_TOOL_PATTERNS {
            assert!(!pattern.is_empty());
        }
        for pattern in PERMISSION_PATTERNS {
            assert!(!pattern.is_empty());
        }
        for pattern in AUTH_PATTERNS {
            assert!(!pattern.is_empty());
        }
        for pattern in SUCCESS_PATTERNS {
            assert!(!pattern.is_empty());
        }
    }
}
