use reqwest::Client;
use serde::Deserialize;

use crate::ai_client::error::{check_response_status, AIError};

/// Returns the latest recommended Claude model
/// Priority: claude-sonnet-4-5 > claude-sonnet-4 > claude-3-5-sonnet
pub fn get_default_claude_model() -> String {
    // Based on Anthropic's naming pattern, try latest versions first
    let candidates = vec![
        "claude-sonnet-4-5-20250929", // Latest Sonnet 4.5
        "claude-sonnet-4-20250514",   // Sonnet 4
        "claude-3-5-sonnet-20241022", // Latest 3.5
    ];

    // Return the first candidate (latest)
    candidates[0].to_string()
}

/// List available Claude models
/// Note: Anthropic doesn't have a public models endpoint yet, so this returns known models
pub async fn list_claude_models(_api_key: &str) -> Result<Vec<String>, AIError> {
    Ok(vec![
        // Claude 4.5 family
        "claude-sonnet-4-5-20250929".to_string(),
        "claude-opus-4-5-20251101".to_string(),
        "claude-haiku-4-5-20251101".to_string(),
        // Claude 4 family
        "claude-sonnet-4-20250514".to_string(),
        // Claude 3.5 family (being deprecated)
        "claude-3-5-sonnet-20241022".to_string(),
        "claude-3-5-haiku-20241022".to_string(),
        // Claude 3 family (legacy)
        "claude-3-opus-20240229".to_string(),
        "claude-3-sonnet-20240229".to_string(),
        "claude-3-haiku-20240307".to_string(),
    ])
}

/// List available OpenAI models (fetches from API)
pub async fn list_openai_models(api_key: &str) -> Result<Vec<String>, AIError> {
    let http_client = Client::new();
    let response = http_client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    let response = check_response_status(response, "OpenAI").await?;

    let models_response: OpenAIModelsResponse = response
        .json()
        .await
        .map_err(|e| AIError::ParseError(format!("Failed to parse models: {}", e)))?;

    // Filter for chat models only
    let chat_models: Vec<String> = models_response
        .data
        .into_iter()
        .filter(|m| m.id.starts_with("gpt-4") || m.id.starts_with("gpt-3.5"))
        .map(|m| m.id)
        .collect();

    Ok(chat_models)
}

// Response structure for OpenAI models list endpoint
#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelInfo {
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    created: u64,
    #[allow(dead_code)]
    owned_by: String,
}
