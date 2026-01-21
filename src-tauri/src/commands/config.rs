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

    Ok(ConfigStatus {
        provider,
        api_keys,
        models: models_config,
        available_claude_models,
        available_openai_models,
    })
}
