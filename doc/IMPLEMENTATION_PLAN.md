# Mini Redis - Implementation Plan

## Overview

This document outlines the step-by-step implementation plan for the Mini Redis cache server.

**Status: ✅ COMPLETED**

---

## Phase 1: Project Setup and Core Data Structures

### Task 1.1: Project Structure
- [x] Create folder structure (src/cache, src/api, src/models, doc)
- [x] Configure Cargo.toml with dependencies
- [x] Create ARCHITECTURE.md

### Task 1.2: Error Module
- [x] Create src/error.rs with CacheError enum
- [x] Implement From traits for error conversion
- [x] Define Result type alias

### Task 1.3: Cache Entry
- [x] Create src/cache/entry.rs
- [x] Define CacheEntry struct (value, created_at, expires_at)
- [x] Implement is_expired() method
- [x] Add unit tests

### Task 1.4: LRU Tracker
- [x] Create src/cache/lru.rs
- [x] Define LruTracker struct with VecDeque
- [x] Implement touch(key) - move to front
- [x] Implement evict_oldest() - return oldest key
- [x] Implement remove(key) - remove specific key
- [x] Add unit tests

### Task 1.5: Cache Store
- [x] Create src/cache/store.rs
- [x] Define CacheStore struct (HashMap + LruTracker + Stats)
- [x] Implement set(key, value, ttl)
- [x] Implement get(key) with TTL check
- [x] Implement delete(key)
- [x] Implement stats()
- [x] Implement cleanup_expired()
- [x] Add unit tests

---

## Phase 2: API Layer

### Task 2.1: Request/Response Models
- [x] Create src/models/requests.rs (SetRequest)
- [x] Create src/models/responses.rs (GetResponse, StatsResponse, ErrorResponse)
- [x] Add Serde derive macros

### Task 2.2: API Handlers
- [x] Create src/api/handlers.rs
- [x] Implement set_handler (PUT /set)
- [x] Implement get_handler (GET /get/:key)
- [x] Implement delete_handler (DELETE /del/:key)
- [x] Implement stats_handler (GET /stats)
- [x] Implement health_handler (GET /health)

### Task 2.3: Router Configuration
- [x] Create src/api/routes.rs
- [x] Configure Axum router with all endpoints
- [x] Add CORS middleware
- [x] Add tracing middleware

---

## Phase 3: Server Integration

### Task 3.1: Application State
- [x] Define AppState with Arc<RwLock<CacheStore>>
- [x] Initialize state in main.rs

### Task 3.2: Background Cleanup Task
- [x] Implement TTL cleanup task using tokio::spawn
- [x] Run cleanup at configurable interval
- [x] Acquire write lock, remove expired entries

### Task 3.3: Server Startup
- [x] Configure tracing subscriber
- [x] Initialize cache store
- [x] Start background cleanup task
- [x] Start Axum server on configurable port

---

## Phase 4: Testing and Documentation

### Task 4.1: Unit Tests
- [x] Test CacheEntry expiration logic
- [x] Test LruTracker operations
- [x] Test CacheStore CRUD operations
- [x] Test TTL expiration
- [x] Test LRU eviction

### Task 4.2: Property-Based Tests
- [x] Property 1: Round-trip Storage Consistency
- [x] Property 2: Delete Removes Entry
- [x] Property 3: Overwrite Semantics
- [x] Property 4: TTL Expiration Behavior
- [x] Property 5: LRU Eviction Order
- [x] Property 6: LRU Access Tracking
- [x] Property 7: Capacity Enforcement
- [x] Property 8: Statistics Accuracy
- [x] Property 9: Concurrent Operation Correctness
- [x] Property 10: Error Response Format

### Task 4.3: Integration Tests
- [x] Test API endpoints with reqwest
- [x] Test concurrent access
- [x] Test TTL expiration via API

### Task 4.4: Documentation
- [x] Update README.md with usage instructions
- [x] Add API documentation
- [x] Add example curl commands
- [x] Update ARCHITECTURE.md

---

## File Creation Order

1. src/error.rs ✅
2. src/cache/entry.rs ✅
3. src/cache/lru.rs ✅
4. src/cache/store.rs ✅
5. src/cache/stats.rs ✅
6. src/cache/mod.rs ✅
7. src/cache/property_tests.rs ✅
8. src/models/requests.rs ✅
9. src/models/responses.rs ✅
10. src/models/mod.rs ✅
11. src/api/handlers.rs ✅
12. src/api/routes.rs ✅
13. src/api/mod.rs ✅
14. src/tasks/cleanup.rs ✅
15. src/tasks/mod.rs ✅
16. src/config.rs ✅
17. src/lib.rs ✅
18. src/main.rs ✅
19. tests/api_integration_tests.rs ✅
20. README.md ✅

---

## Estimated vs Actual Time

| Phase   | Description                    | Estimated | Actual   |
|---------|--------------------------------|-----------|----------|
| Phase 1 | Core data structures           | 2 hours   | ~2 hours |
| Phase 2 | API layer                      | 1.5 hours | ~1.5 hours |
| Phase 3 | Server integration             | 1 hour    | ~1 hour  |
| Phase 4 | Testing and documentation      | 1.5 hours | ~2 hours |
| Total   |                                | 6 hours   | ~6.5 hours |

---

## Dependencies Summary

```toml
[dependencies]
tokio = { version = "1.40", features = ["full"] }
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
proptest = "1.4"
reqwest = { version = "0.11", features = ["json"] }
```

---

## Success Criteria

1. ✅ All API endpoints functional
2. ✅ TTL expiration working correctly
3. ✅ LRU eviction when capacity reached
4. ✅ Thread-safe concurrent access
5. ✅ All unit tests passing
6. ✅ All property-based tests passing
7. ✅ No panics under any condition
8. ✅ Clean, modular code structure

---

## Lessons Learned

### Architecture Decisions

1. **RwLock over Mutex**: Using `Arc<RwLock<CacheStore>>` allows multiple concurrent readers while serializing writes. This significantly improves read throughput for cache-heavy workloads.

2. **VecDeque for LRU**: The VecDeque-based LRU tracker provides O(1) eviction but O(n) touch operations. For production systems with high write throughput, consider using a LinkedHashMap or doubly-linked list with HashMap for O(1) operations across the board.

3. **Modular Task Organization**: Separating the cleanup task into `src/tasks/cleanup.rs` keeps the main.rs clean and makes the background task logic testable and maintainable.

### Property-Based Testing Insights

1. **Proptest Value**: Property-based testing with `proptest` caught edge cases that unit tests missed, particularly around TTL boundary conditions and LRU eviction ordering.

2. **Test Isolation**: Each property test needs careful setup to avoid interference. Using fresh CacheStore instances per test ensures deterministic behavior.

3. **Concurrency Testing**: Testing concurrent operations requires careful synchronization. Using `Arc<RwLock<>>` in tests mirrors production behavior and validates thread safety.

### Implementation Challenges

1. **TTL Precision**: Using millisecond timestamps (u64) provides sufficient precision for TTL calculations. The `is_expired()` check must handle the boundary condition where `expires_at == current_time`.

2. **Statistics Consistency**: Maintaining accurate statistics during concurrent operations requires updating stats within the same lock scope as the cache operation.

3. **Error Response Consistency**: Implementing `IntoResponse` for `CacheError` ensures all error paths return properly formatted JSON responses with appropriate HTTP status codes.

### Best Practices Applied

1. **Configuration via Environment**: All tunable parameters (MAX_ENTRIES, DEFAULT_TTL, SERVER_PORT, CLEANUP_INTERVAL) are configurable via environment variables with sensible defaults.

2. **Structured Logging**: Using `tracing` with structured fields enables effective debugging and monitoring in production.

3. **Graceful Shutdown**: The server handles shutdown signals properly, allowing in-flight requests to complete.

### Future Improvements

1. **Persistence**: Add optional disk persistence for cache entries to survive restarts.

2. **Clustering**: Implement distributed caching with consistent hashing for horizontal scaling.

3. **Metrics Export**: Add Prometheus metrics endpoint for production monitoring.

4. **Rate Limiting**: Add per-client rate limiting to prevent abuse.

5. **Authentication**: Add optional API key authentication for secure deployments.
