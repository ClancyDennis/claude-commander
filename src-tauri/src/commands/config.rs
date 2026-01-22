// Configuration status Tauri commands

use serde::Serialize;
use crate::ai_client::models;

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
    let suffix = &key[key.len()-4..];
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
    ];

    // Available Claude models (static list since Anthropic doesn't have a models endpoint)
    let available_claude_models = vec![
        "claude-sonnet-4-5-20250929".to_string(),
        "claude-sonnet-4-20250514".to_string(),
        "claude-opus-4-5-20251101".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        "claude-3-5-sonnet-20240620".to_string(),
        "claude-3-opus-20240229".to_string(),
        "claude-3-sonnet-20240229".to_string(),
        "claude-3-haiku-20240307".to_string(),
    ];

    // Fetch OpenAI models if API key is configured
    let available_openai_models = if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        if !api_key.is_empty() {
            match models::list_openai_models(&api_key).await {
                Ok(models) => models,
                Err(e) => {
                    eprintln!("Failed to fetch OpenAI models: {}", e);
                    // Return common models as fallback
                    vec![
                        "gpt-4o".to_string(),
                        "gpt-4o-mini".to_string(),
                        "gpt-4-turbo".to_string(),
                        "gpt-4".to_string(),
                        "gpt-3.5-turbo".to_string(),
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
    let config_dir = dirs::config_dir()
        .map(|d| d.join("claude-commander"));

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
