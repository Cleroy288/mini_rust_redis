//! API Module
//!
//! HTTP handlers and routing for the cache server REST API.
//!
//! # Endpoints
//! - `PUT /set` - Store a key-value pair
//! - `GET /get/:key` - Retrieve a value by key
//! - `DELETE /del/:key` - Delete a key
//! - `GET /stats` - Get cache statistics
//! - `GET /health` - Health check endpoint
//!
//! # Requirements
//! - Validates: Requirement 4.1

pub mod handlers;
pub mod routes;

pub use handlers::*;
pub use routes::create_router;
