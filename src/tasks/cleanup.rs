//! TTL Cleanup Task
//!
//! Background task that periodically removes expired cache entries.
//!
//! # Requirements
//! - Validates: Requirements 2.3, 2.5, 8.5

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info};

use crate::cache::CacheStore;

/// Spawns a background task that periodically cleans up expired cache entries.
///
/// The task runs in an infinite loop, sleeping for the specified interval
/// between cleanup runs. It acquires a write lock on the cache store to
/// remove expired entries.
///
/// # Arguments
/// * `cache` - Arc<RwLock<CacheStore>> shared reference to the cache
/// * `cleanup_interval_secs` - Interval in seconds between cleanup runs
///
/// # Returns
/// A JoinHandle for the spawned task, which can be used to abort the task
/// during graceful shutdown.
///
/// # Requirements
/// - Validates: Requirements 2.3, 2.5, 8.5
///
/// # Example
/// ```ignore
/// let cache = Arc::new(RwLock::new(CacheStore::new(1000, 300)));
/// let cleanup_handle = spawn_cleanup_task(cache.clone(), 1);
/// // Later, during shutdown:
/// cleanup_handle.abort();
/// ```
pub fn spawn_cleanup_task(
    cache: Arc<RwLock<CacheStore>>,
    cleanup_interval_secs: u64,
) -> JoinHandle<()> {
    let interval = Duration::from_secs(cleanup_interval_secs);

    tokio::spawn(async move {
        info!(
            "Starting TTL cleanup task with interval of {} seconds",
            cleanup_interval_secs
        );

        loop {
            // Sleep for the configured interval
            tokio::time::sleep(interval).await;

            // Acquire write lock and cleanup expired entries
            let removed = {
                let mut cache_guard = cache.write().await;
                cache_guard.cleanup_expired()
            };

            // Log cleanup statistics
            if removed > 0 {
                info!("TTL cleanup: removed {} expired entries", removed);
            } else {
                debug!("TTL cleanup: no expired entries found");
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cleanup_task_removes_expired_entries() {
        let cache = Arc::new(RwLock::new(CacheStore::new(100, 300)));

        // Add an entry with very short TTL
        {
            let mut cache_guard = cache.write().await;
            cache_guard
                .set("expire_soon".to_string(), "value".to_string(), Some(1))
                .unwrap();
        }

        // Spawn cleanup task with 1 second interval
        let handle = spawn_cleanup_task(cache.clone(), 1);

        // Wait for entry to expire and cleanup to run
        tokio::time::sleep(Duration::from_millis(2500)).await;

        // Verify entry was removed
        {
            let mut cache_guard = cache.write().await;
            let result = cache_guard.get("expire_soon");
            assert!(result.is_err(), "Expired entry should have been cleaned up");
        }

        // Abort the cleanup task
        handle.abort();
    }

    #[tokio::test]
    async fn test_cleanup_task_preserves_valid_entries() {
        let cache = Arc::new(RwLock::new(CacheStore::new(100, 300)));

        // Add an entry with long TTL
        {
            let mut cache_guard = cache.write().await;
            cache_guard
                .set("long_lived".to_string(), "value".to_string(), Some(3600))
                .unwrap();
        }

        // Spawn cleanup task
        let handle = spawn_cleanup_task(cache.clone(), 1);

        // Wait for cleanup to run
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Verify entry still exists
        {
            let mut cache_guard = cache.write().await;
            let result = cache_guard.get("long_lived");
            assert!(result.is_ok(), "Valid entry should not be removed");
            assert_eq!(result.unwrap(), "value");
        }

        // Abort the cleanup task
        handle.abort();
    }

    #[tokio::test]
    async fn test_cleanup_task_can_be_aborted() {
        let cache = Arc::new(RwLock::new(CacheStore::new(100, 300)));

        let handle = spawn_cleanup_task(cache, 1);

        // Abort immediately
        handle.abort();

        // Wait a bit and verify task is finished
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(handle.is_finished(), "Task should be finished after abort");
    }
}
