//! Request DTOs for the cache server API
//!
//! Defines the structure of incoming HTTP request bodies.

use serde::Deserialize;

/// Request body for the SET operation (PUT /set)
///
/// # Fields
/// - `key`: The cache key to store the value under
/// - `value`: The value to store
/// - `ttl`: Optional TTL in seconds (uses default if not specified)
///
/// # Requirements
/// - Validates: Requirement 4.2
#[derive(Debug, Clone, Deserialize)]
pub struct SetRequest {
    /// The cache key
    pub key: String,
    /// The value to store
    pub value: String,
    /// Optional TTL in seconds
    #[serde(default)]
    pub ttl: Option<u64>,
}

impl SetRequest {
    /// Validates the request data
    ///
    /// Returns an error message if validation fails, None if valid.
    pub fn validate(&self) -> Option<String> {
        if self.key.is_empty() {
            return Some("Key cannot be empty".to_string());
        }
        if self.key.len() > 256 {
            return Some("Key exceeds maximum length of 256 characters".to_string());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_request_deserialize() {
        let json = r#"{"key": "test", "value": "hello"}"#;
        let req: SetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key, "test");
        assert_eq!(req.value, "hello");
        assert!(req.ttl.is_none());
    }

    #[test]
    fn test_set_request_with_ttl() {
        let json = r#"{"key": "test", "value": "hello", "ttl": 60}"#;
        let req: SetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ttl, Some(60));
    }

    #[test]
    fn test_validate_empty_key() {
        let req = SetRequest {
            key: "".to_string(),
            value: "test".to_string(),
            ttl: None,
        };
        assert!(req.validate().is_some());
    }

    #[test]
    fn test_validate_valid_request() {
        let req = SetRequest {
            key: "valid_key".to_string(),
            value: "test".to_string(),
            ttl: Some(60),
        };
        assert!(req.validate().is_none());
    }
}
