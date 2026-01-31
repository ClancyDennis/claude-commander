// Configuration status Tauri commands
//
// This module provides the Tauri command entry points for configuration
// management. Business logic is delegated to submodules:
// - config_loader: Environment variable loading, parsing, and file operations
// - api_validator: API key validation for various providers

use crate::ai_client::models;
use crate::commands::api_validator::{self, ApiKeyValidationResult};
use crate::commands::config_loader::{
    self, env_keys, mask_api_key, AVAILABLE_CLAUDE_MODELS, CLAUDE_CODE_MODEL_OPTIONS,
    CLAUDE_MODEL_ALIASES, FALLBACK_OPENAI_MODELS, META_AGENT_PROVIDERS,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ClaudeCodeStatus {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub authenticated: bool,
    pub auth_type: Option<String>,
    pub subscription_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelConfig {
    pub name: String,
    pub value: Option<String>,
    pub is_default: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyStatus {
    pub provider: String,
    pub is_configured: bool,
    pub key_preview: String,
}

#[derive(Debug, Serialize)]
pub struct ConfigStatus {
    pub provider: String,
    pub api_keys: Vec<ApiKeyStatus>,
    pub models: Vec<ModelConfig>,
    pub available_claude_models: Vec<String>,
    pub claude_model_aliases: Vec<String>,
    pub claude_code_model_options: Vec<String>,
    pub meta_agent_providers: Vec<String>,
    pub available_openai_models: Vec<String>,
    pub config_path: String,
    pub is_first_run: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConfigUpdate {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct ConfigUpdateResult {
    pub success: bool,
    pub message: String,
    pub requires_restart: bool,
}

// ============================================================================
// Tauri Commands - Status & Information
// ============================================================================

#[tauri::command]
pub async fn check_claude_code_installed() -> Result<ClaudeCodeStatus, String> {
    use crate::agent_manager::claude_cli::find_claude_cli;

    // Check installation
    let (installed, path, version) = match find_claude_cli() {
        Ok(p) => {
            let v = get_claude_version(&p).await.ok();
            (true, Some(p), v)
        }
        Err(_) => (false, None, None),
    };

    // Check authentication by reading ~/.claude/.credentials.json
    let (authenticated, auth_type, subscription_type) = check_claude_auth();

    Ok(ClaudeCodeStatus {
        installed,
        path,
        version,
        authenticated,
        auth_type,
        subscription_type,
    })
}

/// Check if Claude Code is authenticated by reading the credentials file
fn check_claude_auth() -> (bool, Option<String>, Option<String>) {
    if let Some(home) = dirs::home_dir() {
        let credentials_path = home.join(".claude").join(".credentials.json");

        if credentials_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&credentials_path) {
                // Parse the JSON to check for valid tokens
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Check for OAuth credentials
                    if let Some(oauth) = json.get("claudeAiOauth") {
                        if oauth.get("accessToken").is_some() {
                            // Check if token has expired
                            let is_valid = if let Some(expires_at) = oauth.get("expiresAt") {
                                if let Some(exp) = expires_at.as_i64() {
                                    let now = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_millis() as i64)
                                        .unwrap_or(0);
                                    exp > now
                                } else {
                                    true // No expiry info, assume valid
                                }
                            } else {
                                true
                            };

                            if is_valid {
                                let sub_type = oauth
                                    .get("subscriptionType")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());

                                return (true, Some("oauth".to_string()), sub_type);
                            }
                        }
                    }
                }
            }
        }
    }

    (false, None, None)
}

/// Get the Claude CLI version by running `claude --version`
async fn get_claude_version(cli_path: &str) -> Result<String, String> {
    let output = tokio::process::Command::new(cli_path)
        .arg("--version")
        .output()
        .await
        .map_err(|e| e.to_string())?;

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_config_status() -> Result<ConfigStatus, String> {
    // Determine active provider
    let provider = config_loader::determine_active_provider();

    // Build API key statuses
    let api_keys = build_api_key_statuses();

    // Build model configurations
    let models = build_model_configs();

    // Available Claude models (aliases + pinned versions)
    let available_claude_models: Vec<String> = AVAILABLE_CLAUDE_MODELS
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Claude model aliases only (for display/grouping)
    let claude_model_aliases: Vec<String> =
        CLAUDE_MODEL_ALIASES.iter().map(|s| s.to_string()).collect();

    // Claude Code CLI model options
    let claude_code_model_options: Vec<String> = CLAUDE_CODE_MODEL_OPTIONS
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Meta agent provider options
    let meta_agent_providers: Vec<String> =
        META_AGENT_PROVIDERS.iter().map(|s| s.to_string()).collect();

    // Fetch OpenAI models if API key is configured
    let available_openai_models = fetch_openai_models().await;

    // Determine config path
    let config_path = config_loader::get_env_file_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "~/.config/claude-commander/.env".to_string());

    // Check if first run
    let is_first_run = config_loader::is_first_run();

    Ok(ConfigStatus {
        provider,
        api_keys,
        models,
        available_claude_models,
        claude_model_aliases,
        claude_code_model_options,
        meta_agent_providers,
        available_openai_models,
        config_path,
        is_first_run,
    })
}

#[tauri::command]
pub async fn open_config_directory() -> Result<(), String> {
    let config_dir = config_loader::ensure_config_dir().map_err(|e| e.to_string())?;

    // Open the directory using the system's default file manager
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn create_env_placeholder() -> Result<(), String> {
    let config_dir = config_loader::ensure_config_dir().map_err(|e| e.to_string())?;
    let env_path = config_dir.join(".env");

    // Only create if it doesn't exist
    if !env_path.exists() {
        let placeholder =
            "# Claude Commander Configuration\n# Copy settings from env.example and add your API keys\n";
        std::fs::write(&env_path, placeholder)
            .map_err(|e| format!("Failed to create .env placeholder: {}", e))?;
    }

    Ok(())
}

// ============================================================================
// Tauri Commands - Configuration Updates
// ============================================================================

#[tauri::command]
pub async fn update_config_value(key: String, value: String) -> Result<ConfigUpdateResult, String> {
    // Use the unified update function with a single item
    update_config_internal(&[(key, value)])
}

#[tauri::command]
pub async fn update_config_batch(updates: Vec<ConfigUpdate>) -> Result<ConfigUpdateResult, String> {
    // Convert to tuples and use unified update function
    let updates: Vec<(String, String)> = updates.into_iter().map(|u| (u.key, u.value)).collect();

    update_config_internal(&updates)
}

// ============================================================================
// Tauri Commands - API Key Validation
// ============================================================================

#[tauri::command]
pub async fn validate_api_key(
    provider: String,
    api_key: String,
) -> Result<ApiKeyValidationResult, String> {
    api_validator::validate_api_key_by_name(&provider, &api_key).await
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build API key status list for all providers
fn build_api_key_statuses() -> Vec<ApiKeyStatus> {
    let providers = [
        ("Anthropic", env_keys::ANTHROPIC_API_KEY),
        ("OpenAI", env_keys::OPENAI_API_KEY),
    ];

    providers
        .iter()
        .map(|(name, key)| {
            if let Ok(api_key) = std::env::var(key) {
                ApiKeyStatus {
                    provider: name.to_string(),
                    is_configured: !api_key.is_empty(),
                    key_preview: mask_api_key(&api_key),
                }
            } else {
                ApiKeyStatus {
                    provider: name.to_string(),
                    is_configured: false,
                    key_preview: "Not configured".to_string(),
                }
            }
        })
        .collect()
}

/// Build model configuration list
fn build_model_configs() -> Vec<ModelConfig> {
    let model_keys = [
        env_keys::META_AGENT_PROVIDER,
        env_keys::ANTHROPIC_MODEL,
        env_keys::SECURITY_MODEL,
        env_keys::LIGHT_TASK_MODEL,
        env_keys::CLAUDE_CODE_MODEL,
        env_keys::OPENAI_MODEL,
        env_keys::CLAUDE_CODE_API_KEY_MODE,
    ];

    model_keys
        .iter()
        .map(|&key| {
            let value = std::env::var(key).ok();
            let is_default = value.is_none();
            let default_value = get_default_for_model_key(key);
            ModelConfig {
                name: key.to_string(),
                value,
                is_default,
                default_value,
            }
        })
        .collect()
}

/// Get the default value for a model configuration key
fn get_default_for_model_key(key: &str) -> Option<String> {
    match key {
        env_keys::META_AGENT_PROVIDER => Some("auto".to_string()),
        env_keys::ANTHROPIC_MODEL => Some(models::get_default_claude_model()),
        env_keys::SECURITY_MODEL => Some(models::get_default_claude_model()), // Falls back to main model
        env_keys::LIGHT_TASK_MODEL => Some("claude-haiku-4-5".to_string()), // Faster/cheaper model (alias)
        env_keys::CLAUDE_CODE_MODEL => Some("auto".to_string()), // Use Claude Code's default
        env_keys::OPENAI_MODEL => Some("gpt-4o".to_string()),
        _ => None, // Settings like CLAUDE_CODE_API_KEY_MODE don't have model defaults
    }
}

/// Fetch available OpenAI models, falling back to defaults on error
async fn fetch_openai_models() -> Vec<String> {
    if let Ok(api_key) = std::env::var(env_keys::OPENAI_API_KEY) {
        if !api_key.is_empty() {
            match models::list_openai_models(&api_key).await {
                Ok(models) => return models,
                Err(e) => {
                    eprintln!("Failed to fetch OpenAI models: {}", e);
                }
            }
        }
    }

    // Return fallback models
    FALLBACK_OPENAI_MODELS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Unified configuration update function
/// Handles both single and batch updates with consistent logic
fn update_config_internal(updates: &[(String, String)]) -> Result<ConfigUpdateResult, String> {
    // Validate all keys first
    for (key, _) in updates {
        config_loader::validate_config_key(key).map_err(|e| e.to_string())?;
    }

    // Ensure config directory exists
    let config_dir = config_loader::ensure_config_dir().map_err(|e| e.to_string())?;
    let env_path = config_dir.join(".env");

    // Read existing content
    let existing_content = config_loader::read_env_content(&env_path).map_err(|e| e.to_string())?;

    // Parse existing content
    let mut lines = config_loader::parse_env_content(&existing_content);

    // Apply updates and track if restart is required
    let mut any_restart_required = false;
    for (key, value) in updates {
        let restart_needed = config_loader::update_single_config(&mut lines, key, value);
        if restart_needed {
            any_restart_required = true;
        }
    }

    // Write back to file
    config_loader::write_env_file(&env_path, &lines).map_err(|e| e.to_string())?;

    Ok(ConfigUpdateResult {
        success: true,
        message: if any_restart_required {
            "Configuration saved. Restart the app for API key changes to take full effect."
                .to_string()
        } else {
            "Configuration saved successfully.".to_string()
        },
        requires_restart: any_restart_required,
    })
}
