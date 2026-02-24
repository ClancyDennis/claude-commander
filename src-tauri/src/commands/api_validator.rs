// API key validation module
//
// This module provides validation logic for various API providers
// (Anthropic, OpenAI, GitHub) by making lightweight API calls.

use crate::error::{ApiError, AppError};
use reqwest::Client;
use serde::Serialize;

/// Result of API key validation
#[derive(Debug, Serialize)]
pub struct ApiKeyValidationResult {
    pub valid: bool,
    pub message: String,
}

impl ApiKeyValidationResult {
    /// Create a successful validation result
    pub fn valid() -> Self {
        Self {
            valid: true,
            message: "API key is valid".to_string(),
        }
    }

    /// Create a successful validation result with custom message
    pub fn valid_with_message(message: impl Into<String>) -> Self {
        Self {
            valid: true,
            message: message.into(),
        }
    }

    /// Create an invalid result
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            valid: false,
            message: message.into(),
        }
    }
}

/// Supported API providers for validation
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Gemini,
    GitHub,
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "anthropic" => Ok(Self::Anthropic),
            "openai" => Ok(Self::OpenAI),
            "gemini" | "google" => Ok(Self::Gemini),
            "github" => Ok(Self::GitHub),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

/// Validate an API key for a specific provider
pub async fn validate_api_key(
    provider: Provider,
    api_key: &str,
) -> Result<ApiKeyValidationResult, AppError> {
    let api_key = api_key.trim();

    if api_key.is_empty() {
        return Ok(ApiKeyValidationResult::invalid("API key is empty"));
    }

    let http_client = Client::new();

    match provider {
        Provider::Anthropic => validate_anthropic_key(&http_client, api_key).await,
        Provider::OpenAI => validate_openai_key(&http_client, api_key).await,
        Provider::Gemini => validate_gemini_key(&http_client, api_key).await,
        Provider::GitHub => validate_github_token(&http_client, api_key).await,
    }
}

/// Validate Anthropic API key by making a minimal messages request
async fn validate_anthropic_key(
    client: &Client,
    api_key: &str,
) -> Result<ApiKeyValidationResult, AppError> {
    // Make a minimal messages request - the key validity is determined
    // by the response status, not whether the request succeeds
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(r#"{"model":"claude-3-haiku-20240307","max_tokens":1,"messages":[{"role":"user","content":"Hi"}]}"#)
        .send()
        .await
        .map_err(|e| AppError::Api(ApiError::Network(format!("Network error: {}", e))))?;

    let status = response.status();

    match status.as_u16() {
        200 => Ok(ApiKeyValidationResult::valid()),
        400 => {
            // Bad request but key was accepted - key is valid
            Ok(ApiKeyValidationResult::valid())
        }
        401 => Ok(ApiKeyValidationResult::invalid("Invalid API key")),
        _ => {
            let body = response.text().await.unwrap_or_default();
            Ok(ApiKeyValidationResult::invalid(format!(
                "Validation failed: {} - {}",
                status, body
            )))
        }
    }
}

/// Validate OpenAI API key by listing models (lightweight call)
async fn validate_openai_key(
    client: &Client,
    api_key: &str,
) -> Result<ApiKeyValidationResult, AppError> {
    let response = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| AppError::Api(ApiError::Network(format!("Network error: {}", e))))?;

    let status = response.status();

    match status.as_u16() {
        200 => Ok(ApiKeyValidationResult::valid()),
        401 => Ok(ApiKeyValidationResult::invalid("Invalid API key")),
        _ => {
            let body = response.text().await.unwrap_or_default();
            Ok(ApiKeyValidationResult::invalid(format!(
                "Validation failed: {} - {}",
                status, body
            )))
        }
    }
}

/// Validate Gemini API key by listing models
async fn validate_gemini_key(
    client: &Client,
    api_key: &str,
) -> Result<ApiKeyValidationResult, AppError> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Api(ApiError::Network(format!("Network error: {}", e))))?;

    let status = response.status();

    match status.as_u16() {
        200 => Ok(ApiKeyValidationResult::valid_with_message(
            "Gemini API key is valid",
        )),
        400 | 403 => Ok(ApiKeyValidationResult::invalid("Invalid Gemini API key")),
        _ => {
            let body = response.text().await.unwrap_or_default();
            Ok(ApiKeyValidationResult::invalid(format!(
                "Validation failed: {} - {}",
                status, body
            )))
        }
    }
}

/// Validate GitHub token by getting authenticated user
async fn validate_github_token(
    client: &Client,
    api_key: &str,
) -> Result<ApiKeyValidationResult, AppError> {
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("User-Agent", "Claude-Commander")
        .send()
        .await
        .map_err(|e| AppError::Api(ApiError::Network(format!("Network error: {}", e))))?;

    let status = response.status();

    match status.as_u16() {
        200 => Ok(ApiKeyValidationResult::valid_with_message(
            "GitHub token is valid",
        )),
        401 => Ok(ApiKeyValidationResult::invalid("Invalid GitHub token")),
        _ => {
            let body = response.text().await.unwrap_or_default();
            Ok(ApiKeyValidationResult::invalid(format!(
                "Validation failed: {} - {}",
                status, body
            )))
        }
    }
}

/// Validate an API key by provider name string
/// This is a convenience wrapper for the Tauri command interface
pub async fn validate_api_key_by_name(
    provider: &str,
    api_key: &str,
) -> Result<ApiKeyValidationResult, String> {
    let provider: Provider = provider.parse()?;

    validate_api_key(provider, api_key)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_str() {
        assert!(matches!(
            "anthropic".parse::<Provider>(),
            Ok(Provider::Anthropic)
        ));
        assert!(matches!(
            "ANTHROPIC".parse::<Provider>(),
            Ok(Provider::Anthropic)
        ));
        assert!(matches!("openai".parse::<Provider>(), Ok(Provider::OpenAI)));
        assert!(matches!("github".parse::<Provider>(), Ok(Provider::GitHub)));
        assert!("unknown".parse::<Provider>().is_err());
    }

    #[test]
    fn test_validation_result_constructors() {
        let valid = ApiKeyValidationResult::valid();
        assert!(valid.valid);
        assert_eq!(valid.message, "API key is valid");

        let invalid = ApiKeyValidationResult::invalid("test error");
        assert!(!invalid.valid);
        assert_eq!(invalid.message, "test error");
    }
}
