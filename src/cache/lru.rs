//! LRU Tracker Module
//!
//! Implements Least Recently Used tracking for cache eviction.

use std::collections::VecDeque;

// == LRU Tracker ==
/// Tracks access order for LRU eviction strategy.
///
/// Keys are stored in a VecDeque where:
/// - Front = Most recently used
/// - Back = Least recently used
#[derive(Debug, Default)]
pub struct LruTracker {
    /// Order of keys by access time
    order: VecDeque<String>,
}

impl LruTracker {
    // == Constructor ==
    /// Creates a new empty LRU tracker.
    pub fn new() -> Self {
        Self {
            order: VecDeque::new(),
        }
    }

    // == Touch ==
    /// Marks a key as recently used (moves to front).
    ///
    /// If key exists, removes it first then adds to front.
    /// If key is new, just adds to front.
    pub fn touch(&mut self, key: &str) {
        // Remove existing occurrence
        self.remove(key);
        // Add to front (most recent)
        self.order.push_front(key.to_string());
    }

    // == Remove ==
    /// Removes a key from the tracker.
    pub fn remove(&mut self, key: &str) {
        self.order.retain(|k| k != key);
    }

    // == Evict Oldest ==
    /// Returns and removes the least recently used key.
    ///
    /// Returns None if tracker is empty.
    pub fn evict_oldest(&mut self) -> Option<String> {
        self.order.pop_back()
    }

    // == Peek Oldest ==
    /// Returns the least recently used key without removing it.
    #[allow(dead_code)]
    pub fn peek_oldest(&self) -> Option<&String> {
        self.order.back()
    }

    // == Length ==
    /// Returns the number of tracked keys.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    // == Is Empty ==
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    // == Contains ==
    /// Checks if a key is being tracked.
    #[allow(dead_code)]
    pub fn contains(&self, key: &str) -> bool {
        self.order.iter().any(|k| k == key)
    }
}

// == Unit Tests ==
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_new() {
        let lru = LruTracker::new();
        assert!(lru.is_empty());
        assert_eq!(lru.len(), 0);
    }

    #[test]
    fn test_lru_touch_new_key() {
        let mut lru = LruTracker::new();

        lru.touch("key1");
        lru.touch("key2");
        lru.touch("key3");

        assert_eq!(lru.len(), 3);
        // key1 is oldest (added first)
        assert_eq!(lru.peek_oldest(), Some(&"key1".to_string()));
    }

    #[test]
    fn test_lru_touch_existing_key() {
        let mut lru = LruTracker::new();

        lru.touch("key1");
        lru.touch("key2");
        lru.touch("key3");

        // Touch key1 again - should move to front
        lru.touch("key1");

        assert_eq!(lru.len(), 3);
        // key2 is now oldest
        assert_eq!(lru.peek_oldest(), Some(&"key2".to_string()));
    }

    #[test]
    fn test_lru_evict_oldest() {
        let mut lru = LruTracker::new();

        lru.touch("key1");
        lru.touch("key2");
        lru.touch("key3");

        let evicted = lru.evict_oldest();
        assert_eq!(evicted, Some("key1".to_string()));
        assert_eq!(lru.len(), 2);

        let evicted = lru.evict_oldest();
        assert_eq!(evicted, Some("key2".to_string()));
        assert_eq!(lru.len(), 1);
    }

    #[test]
    fn test_lru_evict_empty() {
        let mut lru = LruTracker::new();
        assert_eq!(lru.evict_oldest(), None);
    }

    #[test]
    fn test_lru_remove() {
        let mut lru = LruTracker::new();

        lru.touch("key1");
        lru.touch("key2");
        lru.touch("key3");

        lru.remove("key2");

        assert_eq!(lru.len(), 2);
        assert!(!lru.contains("key2"));
        assert!(lru.contains("key1"));
        assert!(lru.contains("key3"));
    }

    #[test]
    fn test_lru_order_after_multiple_touches() {
        let mut lru = LruTracker::new();

        // Add keys
        lru.touch("a");
        lru.touch("b");
        lru.touch("c");

        // Access in different order
        lru.touch("a");
        lru.touch("c");
        lru.touch("b");

        // Eviction order should be: a, c, b (oldest to newest)
        // But since we touched them: b is most recent, then c, then a
        // So oldest is now: a (wait, we touched a, then c, then b)
        // Order after touches: front=[b, c, a]=back
        // So oldest is 'a'... wait no.
        // Let me trace:
        // touch(a): [a]
        // touch(b): [b, a]
        // touch(c): [c, b, a]
        // touch(a): remove a, add front: [a, c, b]
        // touch(c): remove c, add front: [c, a, b]
        // touch(b): remove b, add front: [b, c, a]
        // So back (oldest) = 'a'

        assert_eq!(lru.evict_oldest(), Some("a".to_string()));
        assert_eq!(lru.evict_oldest(), Some("c".to_string()));
        assert_eq!(lru.evict_oldest(), Some("b".to_string()));
    }

    #[test]
    fn test_lru_remove_nonexistent_key() {
        let mut lru = LruTracker::new();

        lru.touch("key1");
        lru.touch("key2");

        // Remove a key that doesn't exist - should not panic or affect existing keys
        lru.remove("nonexistent");

        assert_eq!(lru.len(), 2);
        assert!(lru.contains("key1"));
        assert!(lru.contains("key2"));
    }

    #[test]
    fn test_lru_touch_same_key_multiple_times() {
        let mut lru = LruTracker::new();

        // Touch the same key multiple times
        lru.touch("key1");
        lru.touch("key1");
        lru.touch("key1");

        // Should only have one entry
        assert_eq!(lru.len(), 1);
        assert_eq!(lru.evict_oldest(), Some("key1".to_string()));
        assert!(lru.is_empty());
    }

    #[test]
    fn test_lru_evict_oldest_returns_correct_key() {
        let mut lru = LruTracker::new();

        // Add keys in order: a, b, c, d
        lru.touch("a");
        lru.touch("b");
        lru.touch("c");
        lru.touch("d");

        // Oldest should be 'a' (first added, never touched again)
        assert_eq!(lru.peek_oldest(), Some(&"a".to_string()));
        assert_eq!(lru.evict_oldest(), Some("a".to_string()));

        // Now oldest should be 'b'
        assert_eq!(lru.peek_oldest(), Some(&"b".to_string()));
    }

    #[test]
    fn test_lru_touch_moves_to_front() {
        let mut lru = LruTracker::new();

        lru.touch("a");
        lru.touch("b");
        lru.touch("c");

        // 'a' is oldest
        assert_eq!(lru.peek_oldest(), Some(&"a".to_string()));

        // Touch 'a' to move it to front
        lru.touch("a");

        // Now 'b' should be oldest
        assert_eq!(lru.peek_oldest(), Some(&"b".to_string()));

        // Verify 'a' is not evicted first
        assert_eq!(lru.evict_oldest(), Some("b".to_string()));
        assert_eq!(lru.evict_oldest(), Some("c".to_string()));
        assert_eq!(lru.evict_oldest(), Some("a".to_string()));
    }
}
