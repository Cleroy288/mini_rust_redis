//! Background Tasks Module
//!
//! Contains background tasks that run periodically during server operation.
//!
//! # Tasks
//! - TTL Cleanup: Removes expired cache entries at configured intervals
//!
//! # Requirements
//! - Validates: Requirements 2.3, 2.5, 8.5

mod cleanup;

pub use cleanup::spawn_cleanup_task;
