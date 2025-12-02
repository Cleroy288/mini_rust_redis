//! Configuration Module
//!
//! Handles loading and managing server configuration from environment variables.

use std::env;

/// Server configuration parameters.
///
/// All values can be configured via environment variables with sensible defaults.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of entries the cache can hold
    pub max_entries: usize,
    /// Default TTL in seconds for entries without explicit TTL
    pub default_ttl: u64,
    /// HTTP server port
    pub server_port: u16,
    /// Background cleanup task interval in seconds
    pub cleanup_interval: u64,
}

impl Config {
    /// Creates a new Config by loading values from environment variables.
    ///
    /// # Environment Variables
    /// - `MAX_ENTRIES` - Maximum cache entries (default: 1000)
    /// - `DEFAULT_TTL` - Default TTL in seconds (default: 300)
    /// - `SERVER_PORT` - HTTP server port (default: 3000)
    /// - `CLEANUP_INTERVAL` - Cleanup frequency in seconds (default: 1)
    pub fn from_env() -> Self {
        Self {
            max_entries: env::var("MAX_ENTRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            default_ttl: env::var("DEFAULT_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            cleanup_interval: env::var("CLEANUP_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: 300,
            server_port: 3000,
            cleanup_interval: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.default_ttl, 300);
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.cleanup_interval, 1);
    }

    #[test]
    fn test_config_from_env_defaults() {
        // Clear any existing env vars to test defaults
        env::remove_var("MAX_ENTRIES");
        env::remove_var("DEFAULT_TTL");
        env::remove_var("SERVER_PORT");
        env::remove_var("CLEANUP_INTERVAL");

        let config = Config::from_env();
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.default_ttl, 300);
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.cleanup_interval, 1);
    }
}
