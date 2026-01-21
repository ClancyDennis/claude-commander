use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AIError {
    HttpError(reqwest::Error),
    ApiError(String),
    ParseError(String),
    ConfigError(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AIError::HttpError(e) => write!(f, "HTTP error: {}", e),
            AIError::ApiError(e) => write!(f, "API error: {}", e),
            AIError::ParseError(e) => write!(f, "Parse error: {}", e),
            AIError::ConfigError(e) => write!(f, "Config error: {}", e),
        }
    }
}

impl Error for AIError {}

impl From<reqwest::Error> for AIError {
    fn from(err: reqwest::Error) -> Self {
        AIError::HttpError(err)
    }
}

/// Helper to check HTTP response status and return a standardized error
pub async fn check_response_status(
    response: reqwest::Response,
    provider_name: &str,
) -> Result<reqwest::Response, AIError> {
    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AIError::ApiError(format!(
            "{} API error: {}",
            provider_name, error_text
        )));
    }
    Ok(response)
}
