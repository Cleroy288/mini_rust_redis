//! API Handlers
//!
//! HTTP request handlers for each cache server endpoint.
//!
//! # Requirements
//! - Validates: Requirements 4.2, 4.3, 4.4, 4.5, 4.6

use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::cache::CacheStore;
use crate::error::{CacheError, Result};
use crate::models::{
    DeleteResponse, GetResponse, HealthResponse, SetRequest, SetResponse, StatsResponse,
};

/// Application state shared across all handlers.
///
/// Contains the cache store wrapped in Arc<RwLock<>> for thread-safe access.
///
/// # Requirements
/// - Validates: Requirements 5.1, 5.2, 5.3
#[derive(Clone)]
pub struct AppState {
    /// Thread-safe cache store
    pub cache: Arc<RwLock<CacheStore>>,
}

impl AppState {
    /// Creates a new AppState with the given cache store.
    pub fn new(cache: CacheStore) -> Self {
        Self {
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// Creates a new AppState from configuration.
    ///
    /// Initializes the cache store with parameters from the Config.
    pub fn from_config(config: &crate::config::Config) -> Self {
        let cache = CacheStore::new(config.max_entries, config.default_ttl);
        Self::new(cache)
    }
}

/// Handler for PUT /set
///
/// Stores a key-value pair in the cache with optional TTL.
///
/// # Requirements
/// - Validates: Requirement 4.2
pub async fn set_handler(
    State(state): State<AppState>,
    Json(req): Json<SetRequest>,
) -> Result<Json<SetResponse>> {
    // Validate request
    if let Some(error_msg) = req.validate() {
        return Err(CacheError::InvalidRequest(error_msg));
    }

    // Acquire write lock and set the value
    let mut cache = state.cache.write().await;
    cache.set(req.key.clone(), req.value, req.ttl)?;

    Ok(Json(SetResponse::new(req.key)))
}


/// Handler for GET /get/:key
///
/// Retrieves a value from the cache by key.
///
/// # Requirements
/// - Validates: Requirement 4.3
pub async fn get_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<GetResponse>> {
    // Acquire write lock (needed for LRU touch and stats update)
    let mut cache = state.cache.write().await;
    let value = cache.get(&key)?;

    Ok(Json(GetResponse::new(key, value)))
}

/// Handler for DELETE /del/:key
///
/// Deletes a key from the cache.
///
/// # Requirements
/// - Validates: Requirement 4.4
pub async fn delete_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<DeleteResponse>> {
    // Acquire write lock
    let mut cache = state.cache.write().await;
    cache.delete(&key)?;

    Ok(Json(DeleteResponse::new(key)))
}

/// Handler for GET /stats
///
/// Returns current cache statistics.
///
/// # Requirements
/// - Validates: Requirement 4.5
pub async fn stats_handler(State(state): State<AppState>) -> Json<StatsResponse> {
    // Acquire read lock for stats
    let cache = state.cache.read().await;
    let stats = cache.stats();

    Json(StatsResponse::new(
        stats.hits,
        stats.misses,
        stats.evictions,
        stats.total_entries,
    ))
}

/// Handler for GET /health
///
/// Returns health status of the server.
///
/// # Requirements
/// - Validates: Requirement 4.6
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse::healthy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_and_get_handler() {
        let state = AppState::new(CacheStore::new(100, 300));

        // Set a value
        let req = SetRequest {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
            ttl: None,
        };
        let result = set_handler(State(state.clone()), Json(req)).await;
        assert!(result.is_ok());

        // Get the value
        let result = get_handler(State(state.clone()), Path("test_key".to_string())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.value, "test_value");
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let state = AppState::new(CacheStore::new(100, 300));

        let result = get_handler(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_handler() {
        let state = AppState::new(CacheStore::new(100, 300));

        // Set a value first
        let req = SetRequest {
            key: "to_delete".to_string(),
            value: "value".to_string(),
            ttl: None,
        };
        set_handler(State(state.clone()), Json(req)).await.unwrap();

        // Delete it
        let result = delete_handler(State(state.clone()), Path("to_delete".to_string())).await;
        assert!(result.is_ok());

        // Verify it's gone
        let result = get_handler(State(state), Path("to_delete".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stats_handler() {
        let state = AppState::new(CacheStore::new(100, 300));

        let response = stats_handler(State(state)).await;
        assert_eq!(response.hits, 0);
        assert_eq!(response.misses, 0);
    }

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await;
        assert_eq!(response.status, "healthy");
    }

    #[tokio::test]
    async fn test_set_invalid_request() {
        let state = AppState::new(CacheStore::new(100, 300));

        let req = SetRequest {
            key: "".to_string(), // Empty key is invalid
            value: "value".to_string(),
            ttl: None,
        };
        let result = set_handler(State(state), Json(req)).await;
        assert!(result.is_err());
    }
}
