//! Cache Module
//!
//! Provides in-memory caching with TTL expiration and LRU eviction.

mod entry;
mod lru;
mod stats;
mod store;

#[cfg(test)]
mod property_tests;

// Re-export public types
pub use entry::CacheEntry;
pub use lru::LruTracker;
pub use stats::CacheStats;
pub use store::CacheStore;

// == Public Constants ==
/// Maximum allowed key length in bytes
pub const MAX_KEY_LENGTH: usize = 256;

/// Maximum allowed value size in bytes
pub const MAX_VALUE_SIZE: usize = 1024 * 1024; // 1 MB
