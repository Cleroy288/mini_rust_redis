//! Cache Entry Module
//!
//! Defines the structure for individual cache entries with TTL support.

use std::time::{SystemTime, UNIX_EPOCH};

// == Cache Entry ==
/// Represents a single cache entry with value and metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The stored value
    pub value: String,
    /// Creation timestamp (Unix milliseconds)
    pub created_at: u64,
    /// Expiration timestamp (Unix milliseconds), None = no expiration
    pub expires_at: Option<u64>,
}

impl CacheEntry {
    // == Constructor ==
    /// Creates a new cache entry with optional TTL.
    ///
    /// # Arguments
    /// * `value` - The value to store
    /// * `ttl_seconds` - Optional TTL in seconds
    pub fn new(value: String, ttl_seconds: Option<u64>) -> Self {
        let now = current_timestamp_ms();
        let expires_at = ttl_seconds.map(|ttl| now + (ttl * 1000));

        Self {
            value,
            created_at: now,
            expires_at,
        }
    }

    // == Is Expired ==
    /// Checks if the entry has expired.
    ///
    /// Boundary condition: An entry is considered expired when the current time
    /// is greater than or equal to the expiration time. This ensures that once
    /// the TTL duration has fully elapsed, the entry is immediately expired.
    ///
    /// # Returns
    /// - `true` if the entry has a TTL and the current time >= expiration time
    /// - `false` if the entry has no TTL (never expires) or TTL hasn't elapsed
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => current_timestamp_ms() >= expires,
            None => false,
        }
    }

    // == Time To Live ==
    /// Returns remaining TTL in milliseconds, or None if no expiration is set.
    ///
    /// This method is useful for debugging and statistics purposes.
    ///
    /// # Returns
    /// - `Some(0)` if the entry has expired (TTL elapsed)
    /// - `Some(remaining_ms)` if the entry has TTL and hasn't expired
    /// - `None` if the entry has no TTL (never expires)
    pub fn ttl_remaining_ms(&self) -> Option<u64> {
        self.expires_at.map(|expires| {
            let now = current_timestamp_ms();
            if expires > now {
                expires - now
            } else {
                0
            }
        })
    }

    /// Returns remaining TTL in seconds, or None if no expiration is set.
    ///
    /// This is a convenience method that returns TTL in seconds for API responses.
    ///
    /// # Returns
    /// - `Some(0)` if the entry has expired (TTL elapsed)
    /// - `Some(remaining_seconds)` if the entry has TTL and hasn't expired
    /// - `None` if the entry has no TTL (never expires)
    pub fn ttl_remaining(&self) -> Option<u64> {
        self.ttl_remaining_ms().map(|ms| ms / 1000)
    }
}

// == Utility Functions ==
/// Returns current Unix timestamp in milliseconds.
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

// == Unit Tests ==
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_entry_creation_no_ttl() {
        let entry = CacheEntry::new("test_value".to_string(), None);

        assert_eq!(entry.value, "test_value");
        assert!(entry.expires_at.is_none());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_entry_creation_with_ttl() {
        let entry = CacheEntry::new("test_value".to_string(), Some(60));

        assert_eq!(entry.value, "test_value");
        assert!(entry.expires_at.is_some());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_entry_expiration() {
        // Create entry with 1 second TTL
        let entry = CacheEntry::new("test_value".to_string(), Some(1));

        assert!(!entry.is_expired());

        // Wait for expiration
        sleep(Duration::from_millis(1100));

        assert!(entry.is_expired());
    }

    #[test]
    fn test_ttl_remaining_seconds() {
        let entry = CacheEntry::new("test_value".to_string(), Some(10));

        let remaining = entry.ttl_remaining().unwrap();
        assert!(remaining <= 10);
        assert!(remaining >= 9);
    }

    #[test]
    fn test_ttl_remaining_ms() {
        let entry = CacheEntry::new("test_value".to_string(), Some(10));

        let remaining_ms = entry.ttl_remaining_ms().unwrap();
        assert!(remaining_ms <= 10_000);
        assert!(remaining_ms >= 9_000);
    }

    #[test]
    fn test_ttl_remaining_no_expiration() {
        let entry = CacheEntry::new("test_value".to_string(), None);

        assert!(entry.ttl_remaining().is_none());
        assert!(entry.ttl_remaining_ms().is_none());
    }

    #[test]
    fn test_ttl_remaining_expired() {
        // Create entry with very short TTL
        let entry = CacheEntry::new("test_value".to_string(), Some(1));

        // Wait for expiration
        sleep(Duration::from_millis(1100));

        // TTL remaining should be 0 when expired
        assert_eq!(entry.ttl_remaining().unwrap(), 0);
        assert_eq!(entry.ttl_remaining_ms().unwrap(), 0);
    }

    #[test]
    fn test_expiration_boundary_condition() {
        // Create an entry with a known expiration time
        let now = current_timestamp_ms();
        let entry = CacheEntry {
            value: "test".to_string(),
            created_at: now,
            expires_at: Some(now), // Expires exactly at creation time
        };

        // Entry should be expired when current time >= expires_at
        assert!(entry.is_expired(), "Entry should be expired at boundary");
    }
}
