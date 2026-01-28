// Unified error types for the application
//
// This module provides a structured error type that replaces
// ad-hoc `Result<T, String>` usage throughout the codebase.

use std::fmt;

/// Application-wide error type with structured variants for different error categories.
#[derive(Debug)]
pub enum AppError {
    /// Database operation errors (SQLite, connection issues)
    Database(DatabaseError),

    /// File system and IO errors
    Io(IoError),

    /// Configuration errors (missing env vars, invalid values)
    Config(ConfigError),

    /// API and network errors
    Api(ApiError),

    /// Validation errors (invalid input, failed checks)
    Validation(ValidationError),

    /// Internal logic errors (unexpected states, invariant violations)
    Internal(String),
}

/// Database-specific errors
#[derive(Debug)]
pub enum DatabaseError {
    /// SQLite operation failed
    Sqlite(rusqlite::Error),
    /// Connection pool exhausted or unavailable
    ConnectionUnavailable,
    /// Query returned unexpected results
    UnexpectedResult(String),
    /// Migration or schema error
    Schema(String),
}

/// IO and filesystem errors
#[derive(Debug)]
pub enum IoError {
    /// Standard IO error
    Std(std::io::Error),
    /// File not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Path operation failed
    PathError(String),
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    /// Required environment variable missing
    MissingEnvVar(String),
    /// Invalid configuration value
    InvalidValue { key: String, message: String },
    /// Configuration file error
    FileError(String),
}

/// API and network errors
#[derive(Debug)]
pub enum ApiError {
    /// Network request failed
    Network(String),
    /// HTTP error with status code
    Http { status: u16, message: String },
    /// Failed to parse API response
    ParseError(String),
    /// API key invalid or missing
    AuthenticationFailed(String),
    /// Rate limit exceeded
    RateLimited,
    /// Request timeout
    Timeout,
}

/// Validation errors for user input or data
#[derive(Debug)]
pub enum ValidationError {
    /// Required field missing
    MissingField(String),
    /// Field value invalid
    InvalidField { field: String, message: String },
    /// Data constraint violated
    ConstraintViolation(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Config(e) => write!(f, "Config error: {}", e),
            AppError::Api(e) => write!(f, "API error: {}", e),
            AppError::Validation(e) => write!(f, "Validation error: {}", e),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::Sqlite(e) => write!(f, "{}", e),
            DatabaseError::ConnectionUnavailable => write!(f, "Database connection unavailable"),
            DatabaseError::UnexpectedResult(msg) => write!(f, "Unexpected result: {}", msg),
            DatabaseError::Schema(msg) => write!(f, "Schema error: {}", msg),
        }
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::Std(e) => write!(f, "{}", e),
            IoError::NotFound(path) => write!(f, "File not found: {}", path),
            IoError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            IoError::PathError(msg) => write!(f, "Path error: {}", msg),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => {
                write!(f, "Missing environment variable: {}", var)
            }
            ConfigError::InvalidValue { key, message } => {
                write!(f, "Invalid config value for '{}': {}", key, message)
            }
            ConfigError::FileError(msg) => write!(f, "Config file error: {}", msg),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Http { status, message } => {
                write!(f, "HTTP {} error: {}", status, message)
            }
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            ApiError::RateLimited => write!(f, "Rate limit exceeded"),
            ApiError::Timeout => write!(f, "Request timeout"),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ValidationError::InvalidField { field, message } => {
                write!(f, "Invalid field '{}': {}", field, message)
            }
            ValidationError::ConstraintViolation(msg) => {
                write!(f, "Constraint violation: {}", msg)
            }
        }
    }
}

impl std::error::Error for AppError {}
impl std::error::Error for DatabaseError {}
impl std::error::Error for IoError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for ApiError {}
impl std::error::Error for ValidationError {}

// From implementations for automatic error conversion

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(DatabaseError::Sqlite(err))
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        DatabaseError::Sqlite(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(IoError::Std(err))
    }
}

impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        IoError::Std(err)
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        match err {
            std::env::VarError::NotPresent => {
                AppError::Config(ConfigError::MissingEnvVar("unknown".to_string()))
            }
            std::env::VarError::NotUnicode(_) => AppError::Config(ConfigError::InvalidValue {
                key: "unknown".to_string(),
                message: "Value is not valid Unicode".to_string(),
            }),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::Api(ApiError::Timeout)
        } else if err.is_connect() {
            AppError::Api(ApiError::Network(err.to_string()))
        } else if let Some(status) = err.status() {
            AppError::Api(ApiError::Http {
                status: status.as_u16(),
                message: err.to_string(),
            })
        } else {
            AppError::Api(ApiError::Network(err.to_string()))
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Api(ApiError::ParseError(err.to_string()))
    }
}

// Convenience conversion from String (for backwards compatibility during migration)
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Internal(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Internal(msg.to_string())
    }
}

// Conversion to String for Tauri command compatibility
impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

/// Type alias for Results using AppError
pub type AppResult<T> = Result<T, AppError>;

/// Helper trait for converting Result<T, E> to Result<T, AppError>
pub trait IntoAppResult<T> {
    fn into_app_result(self) -> AppResult<T>;
}

impl<T, E: Into<AppError>> IntoAppResult<T> for Result<T, E> {
    fn into_app_result(self) -> AppResult<T> {
        self.map_err(Into::into)
    }
}

/// Extension trait for adding context to errors
pub trait ResultExt<T> {
    /// Add context to an error, converting it to AppError::Internal
    fn context(self, msg: &str) -> AppResult<T>;

    /// Add context with a closure for lazy evaluation
    fn with_context<F: FnOnce() -> String>(self, f: F) -> AppResult<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> AppResult<T> {
        self.map_err(|e| AppError::Internal(format!("{}: {}", msg, e)))
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> AppResult<T> {
        self.map_err(|e| AppError::Internal(format!("{}: {}", f(), e)))
    }
}

/// Extension trait for logging errors that are intentionally discarded.
/// Use this instead of `.ok()` when you want visibility into what errors are being ignored.
pub trait LogOnError<T> {
    /// Convert Result to Option, logging the error at WARN level if present
    fn log_err(self, context: &str) -> Option<T>;

    /// Convert Result to Option, logging at DEBUG level (for expected/routine errors)
    fn log_err_debug(self, context: &str) -> Option<T>;
}

impl<T, E: std::fmt::Display> LogOnError<T> for Result<T, E> {
    fn log_err(self, context: &str) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("[WARN] {}: {}", context, e);
                None
            }
        }
    }

    fn log_err_debug(self, context: &str) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                // Debug level - only log in debug builds
                // Debug level - only log in debug builds
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG] {}: {}", context, e);
                // Suppress unused warnings in release builds
                let _ = (context, e);
                None
            }
        }
    }
}

/// Extension trait for converting Option to AppResult with a custom error message.
/// Use this instead of `.ok_or_else(|| "message".to_string())` for cleaner code.
pub trait OptionExt<T> {
    /// Convert Option to AppResult with an Internal error
    fn ok_or_app(self, msg: impl Into<String>) -> AppResult<T>;

    /// Convert Option to AppResult with a Validation error (for user input)
    fn ok_or_validation(self, field: impl Into<String>) -> AppResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_app(self, msg: impl Into<String>) -> AppResult<T> {
        self.ok_or_else(|| AppError::Internal(msg.into()))
    }

    fn ok_or_validation(self, field: impl Into<String>) -> AppResult<T> {
        self.ok_or_else(|| AppError::Validation(ValidationError::MissingField(field.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::Config(ConfigError::MissingEnvVar("API_KEY".to_string()));
        assert!(err.to_string().contains("API_KEY"));

        let err = AppError::Api(ApiError::Http {
            status: 401,
            message: "Unauthorized".to_string(),
        });
        assert!(err.to_string().contains("401"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    fn test_string_conversion() {
        let err = AppError::Internal("test error".to_string());
        let s: String = err.into();
        assert!(s.contains("test error"));
    }
}
