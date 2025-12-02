//! Mini Redis - A lightweight in-memory cache server
//!
//! Provides Redis-like functionality with TTL expiration and LRU eviction.
//!
//! # Requirements
//! - Validates: Requirements 4.1, 8.4

mod api;
mod cache;
mod config;
mod error;
mod models;
mod tasks;

use std::net::SocketAddr;

use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{create_router, AppState};
use config::Config;
use tasks::spawn_cleanup_task;

/// Main entry point for the Mini Redis cache server.
///
/// # Startup Sequence
/// 1. Initialize tracing subscriber for logging
/// 2. Load configuration from environment variables
/// 3. Create cache store with configured parameters
/// 4. Start background TTL cleanup task
/// 5. Create Axum router with all endpoints
/// 6. Start HTTP server on configured port
/// 7. Handle graceful shutdown on SIGINT/SIGTERM
///
/// # Requirements
/// - Validates: Requirements 4.1, 8.4
#[tokio::main]
async fn main() {
    // Initialize tracing subscriber with env filter
    // Defaults to "info" level, can be overridden with RUST_LOG env var
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mini_redis=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Mini Redis Cache Server");

    // Load configuration from environment variables
    let config = Config::from_env();
    info!(
        "Configuration loaded: max_entries={}, default_ttl={}s, port={}, cleanup_interval={}s",
        config.max_entries, config.default_ttl, config.server_port, config.cleanup_interval
    );

    // Create application state with cache store
    let state = AppState::from_config(&config);
    info!("Cache store initialized");

    // Start background cleanup task
    let cleanup_handle = spawn_cleanup_task(state.cache.clone(), config.cleanup_interval);
    info!("Background cleanup task started");

    // Create router with all endpoints
    let app = create_router(state);

    // Bind to configured port
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Server listening on http://{}", addr);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(cleanup_handle))
        .await
        .unwrap();

    info!("Server shutdown complete");
}

/// Waits for shutdown signal (Ctrl+C or SIGTERM).
///
/// On shutdown signal, aborts the cleanup task and allows graceful shutdown.
async fn shutdown_signal(cleanup_handle: tokio::task::JoinHandle<()>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating shutdown...");
        }
    }

    // Abort the cleanup task
    cleanup_handle.abort();
    warn!("Cleanup task aborted");
}
