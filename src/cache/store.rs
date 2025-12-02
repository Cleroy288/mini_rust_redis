//! Cache Store Module
//!
//! Main cache engine combining HashMap storage with LRU tracking and TTL expiration.

use std::collections::HashMap;

use crate::cache::{CacheEntry, CacheStats, LruTracker, MAX_KEY_LENGTH, MAX_VALUE_SIZE};
use crate::error::{CacheError, Result};

// == Cache Store ==
/// Main cache storage with LRU eviction and TTL support.
#[derive(Debug)]
pub struct CacheStore {
    /// Key-value storage
    entries: HashMap<String, CacheEntry>,
    /// LRU access tracker
    lru: LruTracker,
    /// Performance statistics
    stats: CacheStats,
    /// Maximum number of entries allowed
    max_entries: usize,
    /// Default TTL in seconds for entries without explicit TTL
    default_ttl: u64,
}

impl CacheStore {
    // == Constructor ==
    /// Creates a new CacheStore with specified capacity and default TTL.
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of entries the cache can hold
    /// * `default_ttl` - Default TTL in seconds for entries without explicit TTL
    pub fn new(max_entries: usize, default_ttl: u64) -> Self {
        Self {
            entries: HashMap::new(),
            lru: LruTracker::new(),
            stats: CacheStats::new(),
            max_entries,
            default_ttl,
        }
    }

    // == Set ==
    /// Stores a key-value pair with optional TTL.
    ///
    /// If the key already exists, the value is overwritten and TTL is reset.
    /// If the cache is at capacity, the least recently used entry is evicted.
    ///
    /// # Arguments
    /// * `key` - The key to store
    /// * `value` - The value to store
    /// * `ttl` - Optional TTL in seconds (uses default_ttl if None)
    pub fn set(&mut self, key: String, value: String, ttl: Option<u64>) -> Result<()> {
        // Validate key length
        if key.len() > MAX_KEY_LENGTH {
            return Err(CacheError::InvalidRequest(format!(
                "Key exceeds maximum length of {} bytes",
                MAX_KEY_LENGTH
            )));
        }

        // Validate value size
        if value.len() > MAX_VALUE_SIZE {
            return Err(CacheError::InvalidRequest(format!(
                "Value exceeds maximum size of {} bytes",
                MAX_VALUE_SIZE
            )));
        }

        // Check if key already exists (overwrite case)
        let is_overwrite = self.entries.contains_key(&key);

        // If not overwriting and at capacity, evict oldest entry
        if !is_overwrite && self.entries.len() >= self.max_entries {
            if let Some(evicted_key) = self.lru.evict_oldest() {
                self.entries.remove(&evicted_key);
                self.stats.record_eviction();
            } else {
                return Err(CacheError::CacheFull(
                    "Cache is full and eviction failed".to_string(),
                ));
            }
        }

        // Use provided TTL or default
        let effective_ttl = Some(ttl.unwrap_or(self.default_ttl));

        // Create and store entry
        let entry = CacheEntry::new(value, effective_ttl);
        self.entries.insert(key.clone(), entry);

        // Update LRU tracker (touch moves to front)
        self.lru.touch(&key);

        // Update stats
        self.stats.set_total_entries(self.entries.len());

        Ok(())
    }

    // == Get ==
    /// Retrieves a value by key.
    ///
    /// Returns the value if found and not expired.
    /// Expired entries are removed and counted as misses.
    ///
    /// # Arguments
    /// * `key` - The key to retrieve
    pub fn get(&mut self, key: &str) -> Result<String> {
        // Check if entry exists
        if let Some(entry) = self.entries.get(key) {
            // Check if expired
            if entry.is_expired() {
                // Remove expired entry
                self.entries.remove(key);
                self.lru.remove(key);
                self.stats.set_total_entries(self.entries.len());
                self.stats.record_miss();
                return Err(CacheError::Expired(key.to_string()));
            }

            // Entry exists and is valid - record hit and update LRU
            let value = entry.value.clone();
            self.stats.record_hit();
            self.lru.touch(key);
            Ok(value)
        } else {
            // Entry doesn't exist
            self.stats.record_miss();
            Err(CacheError::NotFound(key.to_string()))
        }
    }

    // == Delete ==
    /// Removes an entry by key.
    ///
    /// # Arguments
    /// * `key` - The key to delete
    pub fn delete(&mut self, key: &str) -> Result<()> {
        if self.entries.remove(key).is_some() {
            self.lru.remove(key);
            self.stats.set_total_entries(self.entries.len());
            Ok(())
        } else {
            Err(CacheError::NotFound(key.to_string()))
        }
    }

    // == Stats ==
    /// Returns current cache statistics.
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.stats.clone();
        stats.set_total_entries(self.entries.len());
        stats
    }

    // == Cleanup Expired ==
    /// Removes all expired entries from the cache.
    ///
    /// Returns the number of entries removed.
    pub fn cleanup_expired(&mut self) -> usize {
        let expired_keys: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();

        for key in expired_keys {
            self.entries.remove(&key);
            self.lru.remove(&key);
        }

        self.stats.set_total_entries(self.entries.len());
        count
    }

    // == Length ==
    /// Returns the current number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    // == Is Empty ==
    /// Returns true if the cache is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}


// == Unit Tests ==
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_store_new() {
        let store = CacheStore::new(100, 300);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_store_set_and_get() {
        let mut store = CacheStore::new(100, 300);

        store.set("key1".to_string(), "value1".to_string(), None).unwrap();
        let value = store.get("key1").unwrap();

        assert_eq!(value, "value1");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_store_get_nonexistent() {
        let mut store = CacheStore::new(100, 300);

        let result = store.get("nonexistent");
        assert!(matches!(result, Err(CacheError::NotFound(_))));
    }

    #[test]
    fn test_store_delete() {
        let mut store = CacheStore::new(100, 300);

        store.set("key1".to_string(), "value1".to_string(), None).unwrap();
        store.delete("key1").unwrap();

        assert!(store.is_empty());
        assert!(matches!(store.get("key1"), Err(CacheError::NotFound(_))));
    }

    #[test]
    fn test_store_delete_nonexistent() {
        let mut store = CacheStore::new(100, 300);

        let result = store.delete("nonexistent");
        assert!(matches!(result, Err(CacheError::NotFound(_))));
    }

    #[test]
    fn test_store_overwrite() {
        let mut store = CacheStore::new(100, 300);

        store.set("key1".to_string(), "value1".to_string(), None).unwrap();
        store.set("key1".to_string(), "value2".to_string(), None).unwrap();

        let value = store.get("key1").unwrap();
        assert_eq!(value, "value2");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_store_ttl_expiration() {
        let mut store = CacheStore::new(100, 300);

        // Set with 1 second TTL
        store.set("key1".to_string(), "value1".to_string(), Some(1)).unwrap();

        // Should be accessible immediately
        assert!(store.get("key1").is_ok());

        // Wait for expiration
        sleep(Duration::from_millis(1100));

        // Should be expired now
        let result = store.get("key1");
        assert!(matches!(result, Err(CacheError::Expired(_))));
    }

    #[test]
    fn test_store_lru_eviction() {
        let mut store = CacheStore::new(3, 300);

        store.set("key1".to_string(), "value1".to_string(), None).unwrap();
        store.set("key2".to_string(), "value2".to_string(), None).unwrap();
        store.set("key3".to_string(), "value3".to_string(), None).unwrap();

        // Cache is full, adding key4 should evict key1 (oldest)
        store.set("key4".to_string(), "value4".to_string(), None).unwrap();

        assert_eq!(store.len(), 3);
        assert!(matches!(store.get("key1"), Err(CacheError::NotFound(_))));
        assert!(store.get("key2").is_ok());
        assert!(store.get("key3").is_ok());
        assert!(store.get("key4").is_ok());
    }

    #[test]
    fn test_store_lru_touch_on_get() {
        let mut store = CacheStore::new(3, 300);

        store.set("key1".to_string(), "value1".to_string(), None).unwrap();
        store.set("key2".to_string(), "value2".to_string(), None).unwrap();
        store.set("key3".to_string(), "value3".to_string(), None).unwrap();

        // Access key1 to make it most recently used
        store.get("key1").unwrap();

        // Adding key4 should evict key2 (now oldest)
        store.set("key4".to_string(), "value4".to_string(), None).unwrap();

        assert!(store.get("key1").is_ok());
        assert!(matches!(store.get("key2"), Err(CacheError::NotFound(_))));
    }

    #[test]
    fn test_store_stats() {
        let mut store = CacheStore::new(100, 300);

        store.set("key1".to_string(), "value1".to_string(), None).unwrap();
        store.get("key1").unwrap(); // hit
        let _ = store.get("nonexistent"); // miss

        let stats = store.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_entries, 1);
    }

    #[test]
    fn test_store_cleanup_expired() {
        let mut store = CacheStore::new(100, 300);

        store.set("key1".to_string(), "value1".to_string(), Some(1)).unwrap();
        store.set("key2".to_string(), "value2".to_string(), Some(10)).unwrap();

        // Wait for key1 to expire
        sleep(Duration::from_millis(1100));

        let removed = store.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(store.len(), 1);
        assert!(store.get("key2").is_ok());
    }

    #[test]
    fn test_store_key_too_long() {
        let mut store = CacheStore::new(100, 300);
        let long_key = "x".repeat(MAX_KEY_LENGTH + 1);

        let result = store.set(long_key, "value".to_string(), None);
        assert!(matches!(result, Err(CacheError::InvalidRequest(_))));
    }

    #[test]
    fn test_store_value_too_large() {
        let mut store = CacheStore::new(100, 300);
        let large_value = "x".repeat(MAX_VALUE_SIZE + 1);

        let result = store.set("key".to_string(), large_value, None);
        assert!(matches!(result, Err(CacheError::InvalidRequest(_))));
    }
}
