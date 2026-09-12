use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::fmt;

/// Application-wide error type replacing `Result<T, String>`.
#[derive(Debug)]
pub enum AppError {
    /// File system / I/O errors.
    Io(std::io::Error),
    /// JSON serialization/deserialization errors.
    Json(serde_json::Error),
    /// A required resource was not found (file, session, etc.).
    NotFound(String),
    /// Invalid input from the user or frontend.
    InvalidInput(String),
    /// LLM service errors (connection, timeout, bad response).
    Llm(String),
    /// Internal/unexpected errors.
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::Llm(msg) => write!(f, "LLM error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl AppError {
    /// Check if the formatted error message contains the given substring.
    /// Provided for backward compatibility with existing test assertions.
    pub fn contains(&self, needle: &str) -> bool {
        self.to_string().contains(needle)
    }
}

// ── Conversions from standard error types ──────────────────────────────────

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Internal(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Internal(s.to_string())
    }
}

// ── Axum IntoResponse ─────────────────────────────────────────────────────

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
            AppError::Json(e) => (StatusCode::BAD_REQUEST, format!("{}", e)),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Llm(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, message).into_response()
    }
}

// ── Helper traits for String-based error contexts ──────────────────────────

/// Extension trait to add context to `Result<T, String>` before converting to `AppError`.
pub trait ErrorContext<T> {
    fn with_context(self, ctx: &str) -> std::result::Result<T, AppError>;
}

impl<T> ErrorContext<T> for std::result::Result<T, String> {
    fn with_context(self, ctx: &str) -> std::result::Result<T, AppError> {
        self.map_err(|e| AppError::Internal(format!("{}: {}", ctx, e)))
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

// ── Compatibility: allow `?` on String errors in functions returning AppError ──
// This enables incremental migration: existing code using `.map_err(|e| ...)` still works.

pub type Result<T> = std::result::Result<T, AppError>;
