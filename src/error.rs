//! Error types for the cache server
//!
//! Provides unified error handling using thiserror.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

// == Cache Error Enum ==
/// Unified error type for the cache server.
#[derive(Error, Debug)]
pub enum CacheError {
    /// Key not found in cache
    #[error("Key not found: {0}")]
    NotFound(String),

    /// Key has expired
    #[error("Key expired: {0}")]
    Expired(String),

    /// Invalid request data
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Cache is full and eviction failed
    #[error("Cache full: {0}")]
    CacheFull(String),

    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
}

// == IntoResponse Implementation ==
impl IntoResponse for CacheError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            CacheError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            CacheError::Expired(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            CacheError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            CacheError::CacheFull(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            CacheError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

// == Result Type Alias ==
/// Convenience Result type for the cache server.
pub type Result<T> = std::result::Result<T, CacheError>;
