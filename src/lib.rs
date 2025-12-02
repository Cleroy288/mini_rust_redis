//! Mini Redis - A lightweight in-memory cache server
//!
//! Provides Redis-like functionality with TTL expiration and LRU eviction.

pub mod api;
pub mod cache;
pub mod config;
pub mod error;
pub mod models;
pub mod tasks;

pub use api::AppState;
pub use config::Config;
pub use tasks::spawn_cleanup_task;
