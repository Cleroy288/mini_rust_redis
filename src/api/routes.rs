//! API Routes
//!
//! Configures the Axum router with all cache server endpoints.
//!
//! # Requirements
//! - Validates: Requirement 4.1

use axum::{
    routing::{delete, get, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use super::handlers::{
    delete_handler, get_handler, health_handler, set_handler, stats_handler, AppState,
};

/// Creates the main router with all endpoints configured.
///
/// # Endpoints
/// - `PUT /set` - Store a key-value pair
/// - `GET /get/:key` - Retrieve a value by key
/// - `DELETE /del/:key` - Delete a key
/// - `GET /stats` - Get cache statistics
/// - `GET /health` - Health check endpoint
///
/// # Middleware
/// - CORS: Allows any origin (configurable for production)
/// - Tracing: Logs all requests for debugging
///
/// # Requirements
/// - Validates: Requirement 4.1
pub fn create_router(state: AppState) -> Router {
    // Configure CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router with all endpoints
    Router::new()
        .route("/set", put(set_handler))
        .route("/get/:key", get(get_handler))
        .route("/del/:key", delete(delete_handler))
        .route("/stats", get(stats_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheStore;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    fn create_test_app() -> Router {
        let cache = CacheStore::new(100, 300);
        let state = AppState::new(cache);
        create_router(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stats_endpoint() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_set_endpoint() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/set")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"key":"test","value":"hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/get/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
