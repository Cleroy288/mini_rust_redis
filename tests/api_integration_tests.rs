//! Integration Tests for API Endpoints
//!
//! Tests full request/response cycle for each endpoint.
//!
//! # Requirements
//! - Validates: Requirements 4.2, 4.3, 4.4, 4.5, 4.6, 7.1, 7.2

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use mini_redis::{api::create_router, cache::CacheStore, AppState};
use serde_json::Value;
use std::thread::sleep;
use std::time::Duration;
use tower::ServiceExt;

// == Helper Functions ==

fn create_test_app() -> Router {
    let cache = CacheStore::new(100, 300);
    let state = AppState::new(cache);
    create_router(state)
}

async fn body_to_json(body: Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

// == SET Endpoint Tests ==
// Validates: Requirement 4.2

#[tokio::test]
async fn test_set_endpoint_success() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"key":"test_key","value":"test_value"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = body_to_json(response.into_body()).await;
    assert!(json.get("message").is_some());
    assert!(json["message"].as_str().unwrap().contains("test_key"));
}

#[tokio::test]
async fn test_set_endpoint_with_ttl() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"key":"ttl_key","value":"ttl_value","ttl":60}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// == GET Endpoint Tests ==
// Validates: Requirement 4.3

#[tokio::test]
async fn test_get_endpoint_success() {
    // Create state and router once
    let cache = CacheStore::new(100, 300);
    let state = AppState::new(cache);
    let app = create_router(state);

    // Set a value first
    let set_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"key":"get_key","value":"get_value"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(set_response.status(), StatusCode::OK);

    // Get the value
    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/get/get_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);
    let json = body_to_json(get_response.into_body()).await;
    assert_eq!(json["key"].as_str().unwrap(), "get_key");
    assert_eq!(json["value"].as_str().unwrap(), "get_value");
}

#[tokio::test]
async fn test_get_endpoint_not_found() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/get/nonexistent_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// == DELETE Endpoint Tests ==
// Validates: Requirement 4.4

#[tokio::test]
async fn test_delete_endpoint_success() {
    let cache = CacheStore::new(100, 300);
    let state = AppState::new(cache);
    let app = create_router(state);

    // Set a value first
    let set_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"key":"delete_key","value":"delete_value"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(set_response.status(), StatusCode::OK);

    // Delete the value
    let del_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/del/delete_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_response.status(), StatusCode::OK);

    // Verify it's gone
    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/get/delete_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_endpoint_not_found() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/del/nonexistent_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// == STATS Endpoint Tests ==
// Validates: Requirement 4.5

#[tokio::test]
async fn test_stats_endpoint() {
    let cache = CacheStore::new(100, 300);
    let state = AppState::new(cache);
    let app = create_router(state);

    // Set a value
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"key":"stats_key","value":"stats_value"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Get (hit)
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/get/stats_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Get (miss)
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/get/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Check stats
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_to_json(response.into_body()).await;

    assert_eq!(json["hits"].as_u64().unwrap(), 1);
    assert_eq!(json["misses"].as_u64().unwrap(), 1);
    assert_eq!(json["total_entries"].as_u64().unwrap(), 1);
    assert!(json.get("hit_rate").is_some());
}

// == HEALTH Endpoint Tests ==
// Validates: Requirement 4.6

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_to_json(response.into_body()).await;
    assert_eq!(json["status"].as_str().unwrap(), "healthy");
    assert!(json.get("timestamp").is_some());
}

// == Error Response Tests ==
// Validates: Requirements 7.1, 7.2

#[tokio::test]
async fn test_invalid_json_request() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"invalid json"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Axum returns 422 for JSON parsing errors by default
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn test_empty_key_request() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"key":"","value":"test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_to_json(response.into_body()).await;
    assert!(json.get("error").is_some());
}

// == TTL Expiration via API Tests ==

#[tokio::test]
async fn test_ttl_expiration_via_api() {
    let cache = CacheStore::new(100, 300);
    let state = AppState::new(cache);
    let app = create_router(state);

    // Set a value with 1 second TTL
    let set_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/set")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"key":"ttl_test","value":"expires_soon","ttl":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(set_response.status(), StatusCode::OK);

    // Verify it exists immediately
    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/get/ttl_test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    // Wait for TTL to expire
    sleep(Duration::from_millis(1100));

    // Verify it's expired
    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/get/ttl_test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}
