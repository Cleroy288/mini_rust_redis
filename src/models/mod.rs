//! Request and Response models for the cache server API
//!
//! This module defines the DTOs (Data Transfer Objects) used for
//! serializing/deserializing HTTP request and response bodies.

pub mod requests;
pub mod responses;

// Re-export commonly used types
pub use requests::SetRequest;
pub use responses::{
    DeleteResponse, ErrorResponse, GetResponse, HealthResponse, SetResponse, StatsResponse,
};
