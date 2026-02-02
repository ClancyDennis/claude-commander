// Configuration loading utilities
//
// This module handles environment variable loading, parsing, and caching
// to reduce redundant lookups and centralize configuration access.

use crate::error::{AppError, ConfigError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Cached environment variables for frequently accessed config keys
static ENV_CACHE: OnceLock<EnvCache> = OnceLock::new();

/// Environment variable names used throughout the application
pub mod env_keys {
    pub const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
    pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
    pub const GITHUB_TOKEN: &str = "GITHUB_TOKEN";
    pub const PRIMARY_MODEL: &str = "PRIMARY_MODEL";
    pub const SECURITY_MODEL: &str = "SECURITY_MODEL";
    pub const LIGHT_TASK_MODEL: &str = "LIGHT_TASK_MODEL";
    pub const CLAUDE_CODE_API_KEY_MODE: &str = "CLAUDE_CODE_API_KEY_MODE";
    pub const CLAUDE_CODE_MODEL: &str = "CLAUDE_CODE_MODEL";
}

/// Allowlist of editable configuration keys
pub const ALLOWED_CONFIG_KEYS: &[&str] = &[
    env_keys::ANTHROPIC_API_KEY,
    env_keys::OPENAI_API_KEY,
    env_keys::GITHUB_TOKEN,
    env_keys::PRIMARY_MODEL,
    env_keys::SECURITY_MODEL,
    env_keys::LIGHT_TASK_MODEL,
    env_keys::CLAUDE_CODE_API_KEY_MODE,
    env_keys::CLAUDE_CODE_MODEL,
];

/// Keys that require app restart to take full effect
pub const RESTART_REQUIRED_KEYS: &[&str] = &[env_keys::ANTHROPIC_API_KEY, env_keys::OPENAI_API_KEY];

/// Claude model aliases (auto-update to latest snapshots)
pub const CLAUDE_MODEL_ALIASES: &[&str] =
    &["claude-sonnet-4-5", "claude-opus-4-5", "claude-haiku-4-5"];

/// Claude pinned model versions
pub const CLAUDE_PINNED_MODELS: &[&str] = &[
    "claude-sonnet-4-5-20250929",
    "claude-opus-4-5-20251101",
    "claude-haiku-4-5-20251101",
    "claude-sonnet-4-20250514",
];

/// All available Claude models (aliases + pinned versions)
pub const AVAILABLE_CLAUDE_MODELS: &[&str] = &[
    // Aliases (auto-update to latest)
    "claude-sonnet-4-5",
    "claude-opus-4-5",
    "claude-haiku-4-5",
    // Pinned versions
    "claude-sonnet-4-5-20250929",
    "claude-opus-4-5-20251101",
    "claude-haiku-4-5-20251101",
    "claude-sonnet-4-20250514",
];

/// Claude Code CLI model options (short aliases + full names)
pub const CLAUDE_CODE_MODEL_OPTIONS: &[&str] = &[
    "auto",   // Use Claude Code's default (latest)
    "sonnet", // Short alias for latest sonnet
    "opus",   // Short alias for latest opus
    "haiku",  // Short alias for latest haiku
];

/// Fallback OpenAI models when API is unavailable
pub const FALLBACK_OPENAI_MODELS: &[&str] =
    &["gpt-5.2", "gpt-5.1", "gpt-5", "gpt-5-mini", "gpt-5-nano"];

/// Cached snapshot of environment variables
#[derive(Debug)]
pub struct EnvCache {
    values: HashMap<String, Option<String>>,
}

impl EnvCache {
    /// Create a new cache by reading all relevant environment variables
    fn new() -> Self {
        let mut values = HashMap::new();

        for &key in ALLOWED_CONFIG_KEYS {
            values.insert(key.to_string(), std::env::var(key).ok());
        }

        Self { values }
    }

    /// Get a cached value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key).and_then(|v| v.as_ref())
    }

    /// Check if a key is configured (has a non-empty value)
    pub fn is_configured(&self, key: &str) -> bool {
        self.values
            .get(key)
            .and_then(|v| v.as_ref())
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }
}

/// Get or initialize the environment cache
/// Note: This cache is a snapshot at initialization time.
/// For runtime updates, use `refresh_env_cache()` or direct `std::env::var()` calls.
pub fn get_env_cache() -> &'static EnvCache {
    ENV_CACHE.get_or_init(EnvCache::new)
}

/// Batch load multiple environment variables efficiently
pub fn load_env_vars(keys: &[&str]) -> HashMap<String, Option<String>> {
    keys.iter()
        .map(|&key| (key.to_string(), std::env::var(key).ok()))
        .collect()
}

/// Load a single environment variable with error context
pub fn load_env_var(key: &str) -> Result<String, AppError> {
    std::env::var(key).map_err(|_| AppError::Config(ConfigError::MissingEnvVar(key.to_string())))
}

/// Load an optional environment variable
pub fn load_env_var_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

/// Determine the active AI provider based on configured API keys
pub fn determine_active_provider() -> String {
    if std::env::var(env_keys::ANTHROPIC_API_KEY)
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        "Anthropic".to_string()
    } else if std::env::var(env_keys::OPENAI_API_KEY)
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        "OpenAI".to_string()
    } else {
        "None".to_string()
    }
}

/// Get the configuration directory path
pub fn get_config_dir() -> Result<PathBuf, AppError> {
    dirs::config_dir()
        .map(|d| d.join("claude-commander"))
        .ok_or_else(|| {
            AppError::Config(ConfigError::FileError(
                "Could not determine config directory".to_string(),
            ))
        })
}

/// Get the .env file path within the config directory
pub fn get_env_file_path() -> Result<PathBuf, AppError> {
    get_config_dir().map(|d| d.join(".env"))
}

/// Ensure the config directory exists, creating it if necessary
pub fn ensure_config_dir() -> Result<PathBuf, AppError> {
    let config_dir = get_config_dir()?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| {
            AppError::Config(ConfigError::FileError(format!(
                "Failed to create config directory: {}",
                e
            )))
        })?;
    }

    Ok(config_dir)
}

/// Check if this is the first run (no .env file exists)
pub fn is_first_run() -> bool {
    get_config_dir()
        .map(|d| !d.join(".env").exists())
        .unwrap_or(true)
}

/// Mask an API key for display (show first 4 and last 4 characters)
pub fn mask_api_key(key: &str) -> String {
    if key.len() < 8 {
        return "****".to_string();
    }
    let prefix = &key[..4];
    let suffix = &key[key.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

/// Parse .env file content into a vector of lines, preserving comments and blank lines
pub fn parse_env_content(content: &str) -> Vec<(Option<String>, String)> {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                // Comment or blank line - no key
                (None, line.to_string())
            } else if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                (Some(key), line.to_string())
            } else {
                // Line without = sign, treat as comment-like
                (None, line.to_string())
            }
        })
        .collect()
}

/// Write .env file with proper formatting and permissions
pub fn write_env_file(path: &Path, lines: &[(Option<String>, String)]) -> Result<(), AppError> {
    let content: String = lines
        .iter()
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    // Add trailing newline
    let content = if content.ends_with('\n') {
        content
    } else {
        format!("{}\n", content)
    };

    std::fs::write(path, &content).map_err(|e| {
        AppError::Config(ConfigError::FileError(format!(
            "Failed to write .env file: {}",
            e
        )))
    })?;

    // Set file permissions to 0600 on Unix (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms).map_err(|e| {
            AppError::Config(ConfigError::FileError(format!(
                "Failed to set file permissions: {}",
                e
            )))
        })?;
    }

    Ok(())
}

/// Read existing .env content or return default header
pub fn read_env_content(path: &Path) -> Result<String, AppError> {
    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| {
            AppError::Config(ConfigError::FileError(format!(
                "Failed to read .env file: {}",
                e
            )))
        })
    } else {
        Ok("# Claude Commander Configuration\n".to_string())
    }
}

/// Update a single configuration value in the environment and file
pub fn update_single_config(
    lines: &mut Vec<(Option<String>, String)>,
    key: &str,
    value: &str,
) -> bool {
    let value_trimmed = value.trim();
    let existing_idx = lines.iter().position(|(k, _)| k.as_deref() == Some(key));

    if value_trimmed.is_empty() {
        // Remove the key if value is empty
        if let Some(idx) = existing_idx {
            lines.remove(idx);
        }
        std::env::remove_var(key);
        false
    } else {
        let new_line = format!("{}={}", key, value_trimmed);

        if let Some(idx) = existing_idx {
            lines[idx] = (Some(key.to_string()), new_line);
        } else {
            lines.push((Some(key.to_string()), new_line));
        }

        std::env::set_var(key, value_trimmed);
        RESTART_REQUIRED_KEYS.contains(&key)
    }
}

/// Validate that a key is in the allowed configuration keys list
pub fn validate_config_key(key: &str) -> Result<(), AppError> {
    if !ALLOWED_CONFIG_KEYS.contains(&key) {
        return Err(AppError::Config(ConfigError::InvalidValue {
            key: key.to_string(),
            message: format!("Configuration key '{}' is not editable", key),
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(mask_api_key("short"), "****");
        assert_eq!(mask_api_key("12345678"), "1234...5678");
    }

    #[test]
    fn test_parse_env_content() {
        let content = "# Comment\nKEY1=value1\n\nKEY2=value2";
        let parsed = parse_env_content(content);

        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], (None, "# Comment".to_string()));
        assert_eq!(
            parsed[1],
            (Some("KEY1".to_string()), "KEY1=value1".to_string())
        );
        assert_eq!(parsed[2], (None, "".to_string()));
        assert_eq!(
            parsed[3],
            (Some("KEY2".to_string()), "KEY2=value2".to_string())
        );
    }

    #[test]
    fn test_validate_config_key() {
        assert!(validate_config_key("ANTHROPIC_API_KEY").is_ok());
        assert!(validate_config_key("INVALID_KEY").is_err());
    }
}
