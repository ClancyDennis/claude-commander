// Configuration status Tauri commands

use crate::ai_client::models;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ModelConfig {
    pub name: String,
    pub value: Option<String>,
    pub is_default: bool,
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
    pub available_openai_models: Vec<String>,
    pub config_path: String,
    pub is_first_run: bool,
}

fn mask_api_key(key: &str) -> String {
    if key.len() < 8 {
        return "****".to_string();
    }
    let prefix = &key[..4];
    let suffix = &key[key.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

#[tauri::command]
pub async fn get_config_status() -> Result<ConfigStatus, String> {
    // Determine active provider
    let provider = if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        "Anthropic".to_string()
    } else if std::env::var("OPENAI_API_KEY").is_ok() {
        "OpenAI".to_string()
    } else {
        "None".to_string()
    };

    // Build API key statuses
    let mut api_keys = Vec::new();

    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        api_keys.push(ApiKeyStatus {
            provider: "Anthropic".to_string(),
            is_configured: !key.is_empty(),
            key_preview: mask_api_key(&key),
        });
    } else {
        api_keys.push(ApiKeyStatus {
            provider: "Anthropic".to_string(),
            is_configured: false,
            key_preview: "Not configured".to_string(),
        });
    }

    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        api_keys.push(ApiKeyStatus {
            provider: "OpenAI".to_string(),
            is_configured: !key.is_empty(),
            key_preview: mask_api_key(&key),
        });
    } else {
        api_keys.push(ApiKeyStatus {
            provider: "OpenAI".to_string(),
            is_configured: false,
            key_preview: "Not configured".to_string(),
        });
    }

    // Build model configurations
    let models_config = vec![
        ModelConfig {
            name: "ANTHROPIC_MODEL".to_string(),
            value: std::env::var("ANTHROPIC_MODEL").ok(),
            is_default: std::env::var("ANTHROPIC_MODEL").is_err(),
        },
        ModelConfig {
            name: "SECURITY_MODEL".to_string(),
            value: std::env::var("SECURITY_MODEL").ok(),
            is_default: std::env::var("SECURITY_MODEL").is_err(),
        },
        ModelConfig {
            name: "LIGHT_TASK_MODEL".to_string(),
            value: std::env::var("LIGHT_TASK_MODEL").ok(),
            is_default: std::env::var("LIGHT_TASK_MODEL").is_err(),
        },
        ModelConfig {
            name: "OPENAI_MODEL".to_string(),
            value: std::env::var("OPENAI_MODEL").ok(),
            is_default: std::env::var("OPENAI_MODEL").is_err(),
        },
        ModelConfig {
            name: "CLAUDE_CODE_API_KEY_MODE".to_string(),
            value: std::env::var("CLAUDE_CODE_API_KEY_MODE").ok(),
            is_default: std::env::var("CLAUDE_CODE_API_KEY_MODE").is_err(),
        },
    ];

    // Available Claude models (current generation only)
    let available_claude_models = vec![
        "claude-sonnet-4-5-20250929".to_string(),
        "claude-opus-4-5-20251101".to_string(),
        "claude-haiku-4-5-20251101".to_string(),
        "claude-sonnet-4-20250514".to_string(),
    ];

    // Fetch OpenAI models if API key is configured
    let available_openai_models = if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        if !api_key.is_empty() {
            match models::list_openai_models(&api_key).await {
                Ok(models) => models,
                Err(e) => {
                    eprintln!("Failed to fetch OpenAI models: {}", e);
                    // Return current models as fallback
                    vec![
                        "gpt-5.2".to_string(),
                        "gpt-5".to_string(),
                        "gpt-5-mini".to_string(),
                        "gpt-5-nano".to_string(),
                    ]
                }
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Determine config path (where .env should be placed)
    let config_dir = dirs::config_dir().map(|d| d.join("claude-commander"));

    let config_path = config_dir
        .as_ref()
        .map(|d| d.join(".env"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.config/claude-commander/.env".to_string());

    // Determine if this is first run (no .env file exists in config directory)
    let is_first_run = config_dir
        .as_ref()
        .map(|d| !d.join(".env").exists())
        .unwrap_or(true);

    Ok(ConfigStatus {
        provider,
        api_keys,
        models: models_config,
        available_claude_models,
        available_openai_models,
        config_path,
        is_first_run,
    })
}

#[tauri::command]
pub async fn open_config_directory() -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("claude-commander"))
        .ok_or("Could not determine config directory")?;

    // Ensure directory exists
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

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
    let config_dir = dirs::config_dir()
        .map(|d| d.join("claude-commander"))
        .ok_or("Could not determine config directory")?;

    // Ensure directory exists
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let env_path = config_dir.join(".env");

    // Only create if it doesn't exist
    if !env_path.exists() {
        // Create empty placeholder with comment
        let placeholder = "# Claude Commander Configuration\n# Copy settings from env.example and add your API keys\n";
        std::fs::write(&env_path, placeholder)
            .map_err(|e| format!("Failed to create .env placeholder: {}", e))?;
    }

    Ok(())
}

// ============================================================================
// Configuration Update Commands
// ============================================================================

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

/// Allowlist of editable configuration keys
const ALLOWED_CONFIG_KEYS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "GITHUB_TOKEN",
    "ANTHROPIC_MODEL",
    "SECURITY_MODEL",
    "LIGHT_TASK_MODEL",
    "OPENAI_MODEL",
    "CLAUDE_CODE_API_KEY_MODE",
];

/// Keys that require app restart to take full effect
const RESTART_REQUIRED_KEYS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
];

/// Parse .env file content into a vector of lines, preserving comments and blank lines
fn parse_env_content(content: &str) -> Vec<(Option<String>, String)> {
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

/// Write .env file with proper formatting
fn write_env_file(path: &Path, lines: &[(Option<String>, String)]) -> Result<(), String> {
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

    std::fs::write(path, &content)
        .map_err(|e| format!("Failed to write .env file: {}", e))?;

    // Set file permissions to 0600 on Unix (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn update_config_value(key: String, value: String) -> Result<ConfigUpdateResult, String> {
    // Validate key is in allowlist
    if !ALLOWED_CONFIG_KEYS.contains(&key.as_str()) {
        return Err(format!("Configuration key '{}' is not editable", key));
    }

    // Get config directory path
    let config_dir = dirs::config_dir()
        .map(|d| d.join("claude-commander"))
        .ok_or("Could not determine config directory")?;

    // Ensure directory exists
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let env_path = config_dir.join(".env");

    // Read existing content or create empty
    let existing_content = if env_path.exists() {
        std::fs::read_to_string(&env_path)
            .map_err(|e| format!("Failed to read .env file: {}", e))?
    } else {
        "# Claude Commander Configuration\n".to_string()
    };

    // Parse existing content
    let mut lines = parse_env_content(&existing_content);

    // Find if key already exists
    let existing_idx = lines.iter().position(|(k, _)| k.as_deref() == Some(&key));

    let value_trimmed = value.trim();

    if value_trimmed.is_empty() {
        // Remove the key if value is empty
        if let Some(idx) = existing_idx {
            lines.remove(idx);
        }
        // Also remove from runtime environment
        std::env::remove_var(&key);
    } else {
        // Format the new line
        let new_line = format!("{}={}", key, value_trimmed);

        if let Some(idx) = existing_idx {
            // Update existing line
            lines[idx] = (Some(key.clone()), new_line);
        } else {
            // Add new line at the end
            lines.push((Some(key.clone()), new_line));
        }

        // Update runtime environment
        std::env::set_var(&key, value_trimmed);
    }

    // Write back to file
    write_env_file(&env_path, &lines)?;

    let requires_restart = RESTART_REQUIRED_KEYS.contains(&key.as_str());

    Ok(ConfigUpdateResult {
        success: true,
        message: if requires_restart {
            "Configuration saved. Restart the app for API key changes to take full effect.".to_string()
        } else {
            "Configuration saved successfully.".to_string()
        },
        requires_restart,
    })
}

#[tauri::command]
pub async fn update_config_batch(updates: Vec<ConfigUpdate>) -> Result<ConfigUpdateResult, String> {
    // Validate all keys first
    for update in &updates {
        if !ALLOWED_CONFIG_KEYS.contains(&update.key.as_str()) {
            return Err(format!("Configuration key '{}' is not editable", update.key));
        }
    }

    // Get config directory path
    let config_dir = dirs::config_dir()
        .map(|d| d.join("claude-commander"))
        .ok_or("Could not determine config directory")?;

    // Ensure directory exists
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let env_path = config_dir.join(".env");

    // Read existing content or create empty
    let existing_content = if env_path.exists() {
        std::fs::read_to_string(&env_path)
            .map_err(|e| format!("Failed to read .env file: {}", e))?
    } else {
        "# Claude Commander Configuration\n".to_string()
    };

    // Parse existing content
    let mut lines = parse_env_content(&existing_content);

    let mut any_restart_required = false;

    for update in &updates {
        let key = &update.key;
        let value_trimmed = update.value.trim();

        // Check if restart required
        if RESTART_REQUIRED_KEYS.contains(&key.as_str()) && !value_trimmed.is_empty() {
            any_restart_required = true;
        }

        // Find if key already exists
        let existing_idx = lines.iter().position(|(k, _)| k.as_deref() == Some(key.as_str()));

        if value_trimmed.is_empty() {
            // Remove the key if value is empty
            if let Some(idx) = existing_idx {
                lines.remove(idx);
            }
            std::env::remove_var(key);
        } else {
            let new_line = format!("{}={}", key, value_trimmed);

            if let Some(idx) = existing_idx {
                lines[idx] = (Some(key.clone()), new_line);
            } else {
                lines.push((Some(key.clone()), new_line));
            }

            std::env::set_var(key, value_trimmed);
        }
    }

    // Write back to file
    write_env_file(&env_path, &lines)?;

    Ok(ConfigUpdateResult {
        success: true,
        message: if any_restart_required {
            "Configuration saved. Restart the app for API key changes to take full effect.".to_string()
        } else {
            "Configuration saved successfully.".to_string()
        },
        requires_restart: any_restart_required,
    })
}

#[derive(Debug, Serialize)]
pub struct ApiKeyValidationResult {
    pub valid: bool,
    pub message: String,
}

/// Validate an API key by making a lightweight API call
#[tauri::command]
pub async fn validate_api_key(provider: String, api_key: String) -> Result<ApiKeyValidationResult, String> {
    let api_key = api_key.trim();

    if api_key.is_empty() {
        return Ok(ApiKeyValidationResult {
            valid: false,
            message: "API key is empty".to_string(),
        });
    }

    let http_client = Client::new();

    match provider.to_lowercase().as_str() {
        "anthropic" => {
            // Validate Anthropic key by checking message count endpoint (lightweight)
            // Since there's no dedicated validation endpoint, we'll make a minimal messages request
            // that will fail gracefully but tell us if the key is valid
            let response = http_client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(r#"{"model":"claude-3-haiku-20240307","max_tokens":1,"messages":[{"role":"user","content":"Hi"}]}"#)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let status = response.status();

            if status.is_success() || status.as_u16() == 200 {
                Ok(ApiKeyValidationResult {
                    valid: true,
                    message: "API key is valid".to_string(),
                })
            } else if status.as_u16() == 401 {
                Ok(ApiKeyValidationResult {
                    valid: false,
                    message: "Invalid API key".to_string(),
                })
            } else if status.as_u16() == 400 {
                // Bad request but key was accepted - key is valid
                Ok(ApiKeyValidationResult {
                    valid: true,
                    message: "API key is valid".to_string(),
                })
            } else {
                let body = response.text().await.unwrap_or_default();
                Ok(ApiKeyValidationResult {
                    valid: false,
                    message: format!("Validation failed: {} - {}", status, body),
                })
            }
        }
        "openai" => {
            // Validate OpenAI key by listing models (lightweight call)
            let response = http_client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let status = response.status();

            if status.is_success() {
                Ok(ApiKeyValidationResult {
                    valid: true,
                    message: "API key is valid".to_string(),
                })
            } else if status.as_u16() == 401 {
                Ok(ApiKeyValidationResult {
                    valid: false,
                    message: "Invalid API key".to_string(),
                })
            } else {
                let body = response.text().await.unwrap_or_default();
                Ok(ApiKeyValidationResult {
                    valid: false,
                    message: format!("Validation failed: {} - {}", status, body),
                })
            }
        }
        "github" => {
            // Validate GitHub token by getting authenticated user
            let response = http_client
                .get("https://api.github.com/user")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("User-Agent", "Claude-Commander")
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let status = response.status();

            if status.is_success() {
                Ok(ApiKeyValidationResult {
                    valid: true,
                    message: "GitHub token is valid".to_string(),
                })
            } else if status.as_u16() == 401 {
                Ok(ApiKeyValidationResult {
                    valid: false,
                    message: "Invalid GitHub token".to_string(),
                })
            } else {
                let body = response.text().await.unwrap_or_default();
                Ok(ApiKeyValidationResult {
                    valid: false,
                    message: format!("Validation failed: {} - {}", status, body),
                })
            }
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}
