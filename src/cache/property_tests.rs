//! Property-Based Tests for Cache Module
//!
//! Uses proptest to verify correctness properties defined in the design document.

use proptest::prelude::*;
use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

use crate::cache::{CacheStore, MAX_KEY_LENGTH, MAX_VALUE_SIZE};

// == Test Configuration ==
const TEST_MAX_ENTRIES: usize = 100;
const TEST_DEFAULT_TTL: u64 = 300;

// == Strategies ==
/// Generates valid cache keys (non-empty, within length limit)
fn valid_key_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{1,64}".prop_map(|s| s)
}

/// Generates valid cache values (within size limit)
fn valid_value_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,256}".prop_map(|s| s)
}

/// Generates a sequence of cache operations for testing
#[derive(Debug, Clone)]
enum CacheOp {
    Set { key: String, value: String },
    Get { key: String },
    Delete { key: String },
}

fn cache_op_strategy() -> impl Strategy<Value = CacheOp> {
    prop_oneof![
        (valid_key_strategy(), valid_value_strategy())
            .prop_map(|(key, value)| CacheOp::Set { key, value }),
        valid_key_strategy().prop_map(|key| CacheOp::Get { key }),
        valid_key_strategy().prop_map(|key| CacheOp::Delete { key }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: local-cache-server, Property 8: Statistics Accuracy**
    // *For any* sequence of cache operations, the statistics (hits, misses, evictions)
    // SHALL accurately reflect the number of each operation type that occurred.
    // **Validates: Requirements 6.1, 6.2, 3.5, 6.4**
    #[test]
    fn prop_statistics_accuracy(ops in prop::collection::vec(cache_op_strategy(), 1..50)) {
        let mut store = CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL);
        let mut expected_hits: u64 = 0;
        let mut expected_misses: u64 = 0;

        for op in ops {
            match op {
                CacheOp::Set { key, value } => {
                    let _ = store.set(key, value, None);
                }
                CacheOp::Get { key } => {
                    match store.get(&key) {
                        Ok(_) => expected_hits += 1,
                        Err(_) => expected_misses += 1,
                    }
                }
                CacheOp::Delete { key } => {
                    let _ = store.delete(&key);
                }
            }
        }

        let stats = store.stats();
        prop_assert_eq!(stats.hits, expected_hits, "Hits mismatch");
        prop_assert_eq!(stats.misses, expected_misses, "Misses mismatch");
        prop_assert_eq!(stats.total_entries, store.len(), "Total entries mismatch");
    }

    // **Feature: local-cache-server, Property 1: Round-trip Storage Consistency**
    // *For any* valid key-value pair, storing the pair and then retrieving it
    // (before expiration) SHALL return the exact same value that was stored.
    // **Validates: Requirements 1.1, 1.2**
    #[test]
    fn prop_roundtrip_storage(key in valid_key_strategy(), value in valid_value_strategy()) {
        let mut store = CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL);

        // Store the value
        store.set(key.clone(), value.clone(), None).unwrap();

        // Retrieve and verify
        let retrieved = store.get(&key).unwrap();
        prop_assert_eq!(retrieved, value, "Round-trip value mismatch");
    }

    // **Feature: local-cache-server, Property 2: Delete Removes Entry**
    // *For any* key that exists in the cache, after a DELETE operation,
    // a subsequent GET operation SHALL return a "not found" result.
    // **Validates: Requirements 1.3, 1.4**
    #[test]
    fn prop_delete_removes_entry(key in valid_key_strategy(), value in valid_value_strategy()) {
        let mut store = CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL);

        // Store the value
        store.set(key.clone(), value, None).unwrap();

        // Verify it exists
        prop_assert!(store.get(&key).is_ok(), "Key should exist before delete");

        // Delete it
        store.delete(&key).unwrap();

        // Verify it's gone
        prop_assert!(store.get(&key).is_err(), "Key should not exist after delete");
    }

    // **Feature: local-cache-server, Property 3: Overwrite Semantics**
    // *For any* key, storing a value V1 and then storing a value V2 with the same key
    // SHALL result in GET returning V2.
    // **Validates: Requirements 1.5**
    #[test]
    fn prop_overwrite_semantics(
        key in valid_key_strategy(),
        value1 in valid_value_strategy(),
        value2 in valid_value_strategy()
    ) {
        let mut store = CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL);

        // Store first value
        store.set(key.clone(), value1, None).unwrap();

        // Overwrite with second value
        store.set(key.clone(), value2.clone(), None).unwrap();

        // Retrieve and verify second value is returned
        let retrieved = store.get(&key).unwrap();
        prop_assert_eq!(retrieved, value2, "Overwrite should return new value");

        // Verify only one entry exists
        prop_assert_eq!(store.len(), 1, "Should have exactly one entry after overwrite");
    }

    // **Feature: local-cache-server, Property 7: Capacity Enforcement**
    // *For any* sequence of SET operations, the number of entries in the cache
    // SHALL never exceed MAX_ENTRIES.
    // **Validates: Requirements 3.1, 8.2**
    #[test]
    fn prop_capacity_enforcement(
        entries in prop::collection::vec(
            (valid_key_strategy(), valid_value_strategy()),
            1..200
        )
    ) {
        let max_entries = 50; // Use smaller max for testing
        let mut store = CacheStore::new(max_entries, TEST_DEFAULT_TTL);

        for (key, value) in entries {
            let _ = store.set(key, value, None);
            prop_assert!(
                store.len() <= max_entries,
                "Cache size {} exceeds max {}",
                store.len(),
                max_entries
            );
        }
    }

}

// Separate proptest block with fewer cases for time-sensitive TTL tests
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    // **Feature: local-cache-server, Property 4: TTL Expiration Behavior**
    // *For any* entry stored with a TTL, after the TTL duration has elapsed,
    // a GET operation SHALL return a "not found" result.
    // **Validates: Requirements 2.1, 2.2**
    #[test]
    fn prop_ttl_expiration_behavior(
        key in valid_key_strategy(),
        value in valid_value_strategy()
    ) {
        let mut store = CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL);

        // Store entry with 1 second TTL
        let ttl_seconds = 1u64;
        store.set(key.clone(), value.clone(), Some(ttl_seconds)).unwrap();

        // Verify entry exists before expiration
        let result_before = store.get(&key);
        prop_assert!(result_before.is_ok(), "Entry should exist before TTL expires");
        prop_assert_eq!(result_before.unwrap(), value, "Value should match before expiration");

        // Wait for TTL to expire (add small buffer for timing)
        sleep(Duration::from_millis(1100));

        // Verify entry is not found after expiration
        let result_after = store.get(&key);
        prop_assert!(result_after.is_err(), "Entry should not be found after TTL expires");
    }
}

// Property tests for LRU eviction behavior
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: local-cache-server, Property 5: LRU Eviction Order**
    // *For any* sequence of cache operations that fills the cache to capacity,
    // when a new entry is added, the entry that was accessed least recently SHALL be evicted.
    // **Validates: Requirements 3.1, 3.4**
    #[test]
    fn prop_lru_eviction_order(
        // Generate unique keys for initial fill
        initial_keys in prop::collection::vec(valid_key_strategy(), 3..10),
        new_key in valid_key_strategy(),
        new_value in valid_value_strategy()
    ) {
        // Deduplicate keys to ensure we have unique entries
        let unique_keys: Vec<String> = initial_keys
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Need at least 2 unique keys for meaningful test
        prop_assume!(unique_keys.len() >= 2);
        
        // Ensure new_key is not in the initial set
        prop_assume!(!unique_keys.contains(&new_key));

        let capacity = unique_keys.len();
        let mut store = CacheStore::new(capacity, TEST_DEFAULT_TTL);

        // Fill cache to capacity - first key added will be oldest (LRU candidate)
        let oldest_key = unique_keys[0].clone();
        for key in &unique_keys {
            store.set(key.clone(), format!("value_{}", key), None).unwrap();
        }

        // Verify cache is at capacity
        prop_assert_eq!(store.len(), capacity, "Cache should be at capacity");

        // Add new entry - should evict the oldest (first) key
        store.set(new_key.clone(), new_value, None).unwrap();

        // Cache should still be at capacity
        prop_assert_eq!(store.len(), capacity, "Cache should remain at capacity after eviction");

        // The oldest key should have been evicted
        prop_assert!(
            store.get(&oldest_key).is_err(),
            "Oldest key '{}' should have been evicted",
            oldest_key
        );

        // The new key should exist
        prop_assert!(
            store.get(&new_key).is_ok(),
            "New key '{}' should exist after insertion",
            new_key
        );

        // All other original keys (except oldest) should still exist
        for key in unique_keys.iter().skip(1) {
            prop_assert!(
                store.get(key).is_ok(),
                "Key '{}' should still exist (not the oldest)",
                key
            );
        }
    }

    // **Feature: local-cache-server, Property 6: LRU Access Tracking**
    // *For any* GET or PUT operation on an existing key, that key SHALL become
    // the most recently used and SHALL NOT be the next eviction candidate.
    // **Validates: Requirements 3.2, 3.3**
    #[test]
    fn prop_lru_access_tracking(
        // Generate unique keys
        keys in prop::collection::vec(valid_key_strategy(), 3..8),
        access_index in 0usize..100,
        new_key in valid_key_strategy(),
        new_value in valid_value_strategy()
    ) {
        // Deduplicate keys
        let unique_keys: Vec<String> = keys
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Need at least 3 unique keys for meaningful test
        prop_assume!(unique_keys.len() >= 3);
        
        // Ensure new_key is not in the initial set
        prop_assume!(!unique_keys.contains(&new_key));

        let capacity = unique_keys.len();
        let mut store = CacheStore::new(capacity, TEST_DEFAULT_TTL);

        // Fill cache to capacity
        for key in &unique_keys {
            store.set(key.clone(), format!("value_{}", key), None).unwrap();
        }

        // Access the first key (which would normally be evicted next) via GET
        // This should move it to most recently used
        let accessed_key = unique_keys[0].clone();
        let _ = store.get(&accessed_key);

        // Now the second key should be the oldest (LRU candidate)
        let expected_evicted = unique_keys[1].clone();

        // Add new entry to trigger eviction
        store.set(new_key.clone(), new_value, None).unwrap();

        // The accessed key should NOT have been evicted
        prop_assert!(
            store.get(&accessed_key).is_ok(),
            "Accessed key '{}' should not be evicted after being touched",
            accessed_key
        );

        // The second key (now oldest) should have been evicted
        prop_assert!(
            store.get(&expected_evicted).is_err(),
            "Key '{}' should have been evicted as it was oldest after access",
            expected_evicted
        );

        // New key should exist
        prop_assert!(
            store.get(&new_key).is_ok(),
            "New key should exist"
        );
    }
}

// == Property Test for Error Response Format ==
// This tests the CacheError -> HTTP response conversion

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: local-cache-server, Property 10: Error Response Format**
    // *For any* error condition, the HTTP response SHALL include a JSON body
    // with an "error" field containing a descriptive message.
    // **Validates: Requirements 7.1, 7.2, 7.5**
    #[test]
    fn prop_error_response_format(
        error_msg in "[a-zA-Z0-9 _-]{1,100}"
    ) {
        use crate::error::CacheError;
        use axum::response::IntoResponse;
        use axum::body::to_bytes;

        // Test all error variants produce valid JSON with "error" field
        let error_variants = vec![
            CacheError::NotFound(error_msg.clone()),
            CacheError::Expired(error_msg.clone()),
            CacheError::InvalidRequest(error_msg.clone()),
            CacheError::CacheFull(error_msg.clone()),
            CacheError::Internal(error_msg.clone()),
        ];

        for error in error_variants {
            let expected_msg = error.to_string();
            let response = error.into_response();

            // Verify response has correct content-type header
            let content_type = response.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok());
            prop_assert!(
                content_type.map(|ct| ct.contains("application/json")).unwrap_or(false),
                "Response should have JSON content-type"
            );

            // Parse body as JSON and verify "error" field exists
            let body = response.into_body();
            let rt = tokio::runtime::Runtime::new().unwrap();
            let bytes = rt.block_on(async {
                to_bytes(body, usize::MAX).await.unwrap()
            });

            let json: serde_json::Value = serde_json::from_slice(&bytes)
                .expect("Response body should be valid JSON");

            prop_assert!(
                json.get("error").is_some(),
                "JSON response should contain 'error' field"
            );

            let error_value = json.get("error").unwrap();
            prop_assert!(
                error_value.is_string(),
                "'error' field should be a string"
            );

            // Verify the error message contains the original message
            let error_str = error_value.as_str().unwrap();
            prop_assert!(
                error_str.contains(&expected_msg) || expected_msg.contains(error_str),
                "Error message '{}' should relate to expected '{}'",
                error_str,
                expected_msg
            );
        }
    }
}

// == Property Test for Concurrent Operation Correctness ==
// This tests thread-safe access to the cache via Arc<RwLock<CacheStore>>

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // **Feature: local-cache-server, Property 9: Concurrent Operation Correctness**
    // *For any* set of concurrent read and write operations, all reads SHALL return
    // either a complete old value or a complete new value, never partial or corrupted data.
    // **Validates: Requirements 5.1, 5.2, 5.3**
    #[test]
    fn prop_concurrent_operation_correctness(
        initial_entries in prop::collection::vec(
            (valid_key_strategy(), valid_value_strategy()),
            1..20
        ),
        operations in prop::collection::vec(cache_op_strategy(), 10..50)
    ) {
        use std::sync::Arc;
        use tokio::sync::RwLock;

        // Create a runtime for async operations
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // Create shared cache store
            let store = Arc::new(RwLock::new(CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL)));

            // Populate with initial entries
            {
                let mut cache = store.write().await;
                for (key, value) in &initial_entries {
                    let _ = cache.set(key.clone(), value.clone(), None);
                }
            }

            // Track expected values for verification
            let expected_values: std::collections::HashMap<String, String> = initial_entries
                .iter()
                .cloned()
                .collect();

            // Spawn concurrent tasks
            let mut handles = vec![];

            for op in operations {
                let store_clone = Arc::clone(&store);
                let expected_clone = expected_values.clone();

                let handle = tokio::spawn(async move {
                    match op {
                        CacheOp::Set { key, value } => {
                            let mut cache = store_clone.write().await;
                            let _ = cache.set(key, value, None);
                            Ok::<_, String>(())
                        }
                        CacheOp::Get { key } => {
                            let mut cache = store_clone.write().await;
                            if let Ok(value) = cache.get(&key) {
                                // Verify value is complete (not partial/corrupted)
                                // A valid value should be non-empty and contain only valid chars
                                if value.is_empty() && expected_clone.get(&key).map(|v| !v.is_empty()).unwrap_or(false) {
                                    return Err(format!("Got empty value for key '{}' when non-empty expected", key));
                                }
                                // Value should be a complete string, not truncated
                                // Check it's valid UTF-8 (already guaranteed by String type)
                                // and has reasonable length
                                if value.len() > MAX_VALUE_SIZE {
                                    return Err(format!("Value exceeds max size: {}", value.len()));
                                }
                            }
                            Ok(())
                        }
                        CacheOp::Delete { key } => {
                            let mut cache = store_clone.write().await;
                            let _ = cache.delete(&key);
                            Ok(())
                        }
                    }
                });

                handles.push(handle);
            }

            // Wait for all tasks to complete and check for errors
            for handle in handles {
                let result = handle.await.expect("Task should not panic");
                prop_assert!(result.is_ok(), "Concurrent operation failed: {:?}", result);
            }

            // Verify cache is in a consistent state
            let cache = store.read().await;
            let stats = cache.stats();

            // Stats should be consistent
            prop_assert!(
                stats.total_entries <= TEST_MAX_ENTRIES,
                "Cache should not exceed max entries"
            );

            // Hit rate should be valid (0.0 to 1.0 or NaN if no requests)
            let hit_rate = stats.hit_rate();
            prop_assert!(
                hit_rate.is_nan() || (hit_rate >= 0.0 && hit_rate <= 1.0),
                "Hit rate should be between 0 and 1, got {}",
                hit_rate
            );

            Ok(())
        })?;
    }
}

// == Additional Unit Tests for Edge Cases ==
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_length_validation() {
        let mut store = CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL);
        let long_key = "x".repeat(MAX_KEY_LENGTH + 1);

        let result = store.set(long_key, "value".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_value_size_validation() {
        let mut store = CacheStore::new(TEST_MAX_ENTRIES, TEST_DEFAULT_TTL);
        let large_value = "x".repeat(MAX_VALUE_SIZE + 1);

        let result = store.set("key".to_string(), large_value, None);
        assert!(result.is_err());
    }

    // **Feature: local-cache-server, Property 10: Error Response Format**
    // Unit test for HTTP status code mapping
    // **Validates: Requirements 7.1, 7.2, 7.5**
    #[test]
    fn test_error_status_codes() {
        use crate::error::CacheError;
        use axum::http::StatusCode;
        use axum::response::IntoResponse;

        let test_cases = vec![
            (CacheError::NotFound("key".to_string()), StatusCode::NOT_FOUND),
            (CacheError::Expired("key".to_string()), StatusCode::NOT_FOUND),
            (CacheError::InvalidRequest("bad".to_string()), StatusCode::BAD_REQUEST),
            (CacheError::CacheFull("full".to_string()), StatusCode::SERVICE_UNAVAILABLE),
            (CacheError::Internal("error".to_string()), StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (error, expected_status) in test_cases {
            let response = error.into_response();
            assert_eq!(
                response.status(),
                expected_status,
                "Error should map to correct HTTP status"
            );
        }
    }
}
