//! Response DTOs for the cache server API
//!
//! Defines the structure of outgoing HTTP response bodies.

use serde::Serialize;

/// Response body for the GET operation (GET /get/:key)
///
/// # Requirements
/// - Validates: Requirement 4.3
#[derive(Debug, Clone, Serialize)]
pub struct GetResponse {
    /// The requested key
    pub key: String,
    /// The stored value
    pub value: String,
}

impl GetResponse {
    /// Creates a new GetResponse
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Response body for the SET operation (PUT /set)
///
/// # Requirements
/// - Validates: Requirement 4.2
#[derive(Debug, Clone, Serialize)]
pub struct SetResponse {
    /// Success message
    pub message: String,
    /// The key that was set
    pub key: String,
}

impl SetResponse {
    /// Creates a new SetResponse
    pub fn new(key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            message: format!("Key '{}' set successfully", key),
            key,
        }
    }
}

/// Response body for the DELETE operation (DELETE /del/:key)
///
/// # Requirements
/// - Validates: Requirement 4.4
#[derive(Debug, Clone, Serialize)]
pub struct DeleteResponse {
    /// Success message
    pub message: String,
    /// The key that was deleted
    pub key: String,
}

impl DeleteResponse {
    /// Creates a new DeleteResponse
    pub fn new(key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            message: format!("Key '{}' deleted successfully", key),
            key,
        }
    }
}

/// Response body for the stats endpoint (GET /stats)
///
/// # Requirements
/// - Validates: Requirement 4.5, 6.4
#[derive(Debug, Clone, Serialize)]
pub struct StatsResponse {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of evictions
    pub evictions: u64,
    /// Current number of entries in cache
    pub total_entries: usize,
    /// Hit rate (hits / (hits + misses))
    pub hit_rate: f64,
}

impl StatsResponse {
    /// Creates a new StatsResponse from cache statistics
    pub fn new(hits: u64, misses: u64, evictions: u64, total_entries: usize) -> Self {
        let total_requests = hits + misses;
        let hit_rate = if total_requests > 0 {
            hits as f64 / total_requests as f64
        } else {
            0.0
        };
        Self {
            hits,
            misses,
            evictions,
            total_entries,
            hit_rate,
        }
    }
}

/// Response body for the health endpoint (GET /health)
///
/// # Requirements
/// - Validates: Requirement 4.6
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    /// Health status (e.g., "healthy")
    pub status: String,
    /// Current timestamp in ISO 8601 format
    pub timestamp: String,
}

impl HealthResponse {
    /// Creates a new HealthResponse with current timestamp
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Error response body for all error conditions
///
/// # Requirements
/// - Validates: Requirement 7.5
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    pub error: String,
}

impl ErrorResponse {
    /// Creates a new ErrorResponse
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_response_serialize() {
        let resp = GetResponse::new("test_key", "test_value");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("test_key"));
        assert!(json.contains("test_value"));
    }

    #[test]
    fn test_set_response_serialize() {
        let resp = SetResponse::new("my_key");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("my_key"));
        assert!(json.contains("successfully"));
    }

    #[test]
    fn test_delete_response_serialize() {
        let resp = DeleteResponse::new("deleted_key");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("deleted_key"));
        assert!(json.contains("deleted"));
    }

    #[test]
    fn test_stats_response_hit_rate() {
        let resp = StatsResponse::new(80, 20, 5, 100);
        assert!((resp.hit_rate - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_stats_response_zero_requests() {
        let resp = StatsResponse::new(0, 0, 0, 0);
        assert_eq!(resp.hit_rate, 0.0);
    }

    #[test]
    fn test_health_response_serialize() {
        let resp = HealthResponse::healthy();
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_error_response_serialize() {
        let resp = ErrorResponse::new("Something went wrong");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Something went wrong"));
    }
}
