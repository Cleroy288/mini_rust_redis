# Implementation Plan

## Phase 1: Core Cache Infrastructure

- [x] 1. Complete cache module foundation
  - [x] 1.1 Create src/cache/mod.rs with module exports
    - Export CacheEntry, LruTracker, CacheStore, CacheStats
    - Define public constants (MAX_KEY_LENGTH, MAX_VALUE_SIZE)
    - _Requirements: 1.1, 3.1_
  - [x] 1.2 Create src/cache/stats.rs for cache statistics
    - Define CacheStats struct with hits, misses, evictions, total_entries
    - Implement hit_rate() calculation method
    - Add Serialize derive for JSON responses
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  - [x] 1.3 Write property test for statistics accuracy
    - **Property 8: Statistics Accuracy**
    - **Validates: Requirements 6.1, 6.2, 3.5, 6.4**
  - [x] 1.4 Create src/cache/store.rs implementing CacheStore
    - Define CacheStore struct with HashMap, LruTracker, CacheStats, max_entries, default_ttl
    - Implement new() constructor
    - Implement set() with TTL handling and LRU eviction
    - Implement get() with expiration check and LRU touch
    - Implement delete() with LRU removal
    - Implement stats() returning current statistics
    - Implement cleanup_expired() for background task
    - Implement len() for entry count
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 3.1, 3.2, 3.3_
  - [x] 1.5 Write property test for round-trip storage
    - **Property 1: Round-trip Storage Consistency**
    - **Validates: Requirements 1.1, 1.2**
  - [x] 1.6 Write property test for delete removes entry
    - **Property 2: Delete Removes Entry**
    - **Validates: Requirements 1.3, 1.4**
  - [x] 1.7 Write property test for overwrite semantics
    - **Property 3: Overwrite Semantics**
    - **Validates: Requirements 1.5**
  - [x] 1.8 Write property test for capacity enforcement
    - **Property 7: Capacity Enforcement**
    - **Validates: Requirements 3.1, 8.2**

- [x] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: TTL and LRU Verification

- [x] 3. TTL expiration implementation
  - [x] 3.1 Enhance CacheEntry with TTL validation
    - Add is_expired() boundary condition handling
    - Add ttl_remaining() for stats/debugging
    - _Requirements: 2.1, 2.2_
  - [x] 3.2 Write property test for TTL expiration
    - **Property 4: TTL Expiration Behavior**
    - **Validates: Requirements 2.1, 2.2**

- [x] 4. LRU eviction verification
  - [x] 4.1 Verify LruTracker implementation completeness
    - Ensure touch() moves key to front correctly
    - Ensure evict_oldest() returns correct key
    - Ensure remove() handles non-existent keys
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - [x] 4.2 Write property test for LRU eviction order
    - **Property 5: LRU Eviction Order**
    - **Validates: Requirements 3.1, 3.4**
  - [x] 4.3 Write property test for LRU access tracking
    - **Property 6: LRU Access Tracking**
    - **Validates: Requirements 3.2, 3.3**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: API Layer

- [x] 6. Request/Response models
  - [x] 6.1 Create src/models/mod.rs with module exports
    - Export requests and responses modules
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6_
  - [x] 6.2 Create src/models/requests.rs
    - Define SetRequest with key, value, ttl fields
    - Add Deserialize derive and validation
    - _Requirements: 4.2_
  - [x] 6.3 Create src/models/responses.rs
    - Define GetResponse with key, value fields
    - Define SetResponse with success message
    - Define DeleteResponse with success message
    - Define StatsResponse with all statistics fields
    - Define HealthResponse with status and timestamp
    - Define ErrorResponse with error field
    - Add Serialize derive to all
    - _Requirements: 4.3, 4.4, 4.5, 4.6, 7.5_

- [x] 7. API handlers implementation
  - [x] 7.1 Create src/api/mod.rs with module exports
    - Export handlers and routes modules
    - _Requirements: 4.1_
  - [x] 7.2 Create src/api/handlers.rs
    - Implement set_handler for PUT /set
    - Implement get_handler for GET /get/:key
    - Implement delete_handler for DELETE /del/:key
    - Implement stats_handler for GET /stats
    - Implement health_handler for GET /health
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6_
  - [x] 7.3 Create src/api/routes.rs
    - Configure Axum router with all endpoints
    - Add CORS middleware
    - Add tracing middleware
    - _Requirements: 4.1_
  - [x] 7.4 Write property test for error response format
    - **Property 10: Error Response Format**
    - **Validates: Requirements 7.1, 7.2, 7.5**

- [x] 8. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: Server Integration

- [x] 9. Application state and configuration
  - [x] 9.1 Create src/config.rs for configuration management
    - Define Config struct with all parameters
    - Implement from_env() loading from environment variables
    - Set sensible defaults
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  - [x] 9.2 Define AppState in main.rs
    - Create AppState with Arc<RwLock<CacheStore>>
    - Initialize from Config
    - _Requirements: 5.1, 5.2, 5.3_

- [x] 10. Background cleanup task
  - [x] 10.1 Implement TTL cleanup task
    - Create async cleanup loop with tokio::spawn
    - Run at CLEANUP_INTERVAL frequency
    - Acquire write lock and call cleanup_expired()
    - Log cleanup statistics
    - _Requirements: 2.3, 2.5, 8.5_

- [x] 11. Server startup
  - [x] 11.1 Complete main.rs server implementation
    - Initialize tracing subscriber
    - Load configuration from environment
    - Create CacheStore with config values
    - Start background cleanup task
    - Create Axum app with routes and state
    - Start server on configured port
    - Handle graceful shutdown
    - _Requirements: 4.1, 8.4_

- [x] 12. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: Concurrency and Integration Testing

- [-] 13. Concurrency verification
  - [x] 13.1 Write property test for concurrent correctness
    - **Property 9: Concurrent Operation Correctness**
    - **Validates: Requirements 5.1, 5.2, 5.3**
  - [x] 13.2 Write integration tests for API endpoints
    - Test full request/response cycle for each endpoint
    - Test error responses for invalid requests
    - Test TTL expiration via API
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6, 7.1, 7.2_

- [x] 14. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: Documentation

- [-] 15. Update documentation
  - [x] 15.1 Update README.md with usage instructions
    - Add project description
    - Add build and run instructions
    - Add API documentation with curl examples
    - Add configuration options
    - _Requirements: All_
  - [x] 15.2 Update doc/ARCHITECTURE.md if needed
    - Ensure diagrams match implementation
    - Update any changed interfaces
    - _Requirements: All_
  - [x] 15.3 Update doc/IMPLEMENTATION_PLAN.md
    - Mark completed tasks
    - Add any lessons learned
    - _Requirements: All_
