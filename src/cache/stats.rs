//! Cache Statistics Module
//!
//! Tracks cache performance metrics including hits, misses, and evictions.

use serde::Serialize;

// == Cache Stats ==
/// Tracks cache performance metrics.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CacheStats {
    /// Number of successful cache retrievals
    pub hits: u64,
    /// Number of failed cache retrievals (key not found or expired)
    pub misses: u64,
    /// Number of entries evicted due to LRU policy
    pub evictions: u64,
    /// Current number of entries in the cache
    pub total_entries: usize,
}

impl CacheStats {
    // == Constructor ==
    /// Creates a new CacheStats with all counters at zero.
    pub fn new() -> Self {
        Self::default()
    }

    // == Hit Rate ==
    /// Calculates the cache hit rate.
    ///
    /// Returns hits / (hits + misses), or 0.0 if no requests have been made.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    // == Record Hit ==
    /// Increments the hit counter.
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    // == Record Miss ==
    /// Increments the miss counter.
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    // == Record Eviction ==
    /// Increments the eviction counter.
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    // == Update Entry Count ==
    /// Updates the total entries count.
    pub fn set_total_entries(&mut self, count: usize) {
        self.total_entries = count;
    }
}

// == Unit Tests ==
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_new() {
        let stats = CacheStats::new();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_hit_rate_no_requests() {
        let stats = CacheStats::new();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate_all_hits() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hit_rate(), 1.0);
    }

    #[test]
    fn test_hit_rate_all_misses() {
        let mut stats = CacheStats::new();
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate_mixed() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_miss();
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[test]
    fn test_record_eviction() {
        let mut stats = CacheStats::new();
        stats.record_eviction();
        stats.record_eviction();
        assert_eq!(stats.evictions, 2);
    }

    #[test]
    fn test_set_total_entries() {
        let mut stats = CacheStats::new();
        stats.set_total_entries(42);
        assert_eq!(stats.total_entries, 42);
    }
}
