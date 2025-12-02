# Design Document: Local Cache Server

## Overview

This document describes the technical design for a lightweight in-memory cache server with TTL-based expiration and LRU eviction policies. The system is implemented in Rust using async/await patterns with Tokio runtime and exposes a REST API via Axum framework.

The design prioritizes:
- O(1) time complexity for core operations
- Thread-safe concurrent access
- Clean modular architecture
- Enterprise-level error handling

## Architecture

```
                              MINI REDIS ARCHITECTURE

    +------------------+
    |   HTTP Client    |
    |   (REST API)     |
    +--------+---------+
             |
             | HTTP Requests (JSON)
             v
    +--------+---------+
    |   Axum Router    |
    |   /set /get /del |
    |   /stats /health |
    +--------+---------+
             |
             v
    +--------+---------+
    |   API Handlers   |
    |   handlers.rs    |
    +--------+---------+
             |
             v
    +--------+----------------------------+
    |        Cache Store                  |
    |   Arc<RwLock<CacheStore>>           |
    +-------------------------------------+
    |                                     |
    |  +-------------+  +-------------+   |
    |  | HashMap     |  | LRU Tracker |   |
    |  | key -> Entry|  | VecDeque    |   |
    |  +-------------+  +-------------+   |
    |                                     |
    |  +-----------------------------+    |
    |  | CacheEntry                  |    |
    |  | - value: String             |    |
    |  | - expires_at: Option<u64>   |    |
    |  | - created_at: u64           |    |
    |  +-----------------------------+    |
    |                                     |
    |  +-----------------------------+    |
    |  | CacheStats                  |    |
    |  | - hits: u64                 |    |
    |  | - misses: u64               |    |
    |  | - evictions: u64            |    |
    |  +-----------------------------+    |
    |                                     |
    +-------------------------------------+
             ^
             |
    +--------+---------+
    |  Background Task |
    |  TTL Cleanup     |
    |  (tokio::spawn)  |
    +------------------+
```

## Components and Interfaces

### 1. Cache Entry (`src/cache/entry.rs`)

Represents a single cached value with metadata.

```rust
pub struct CacheEntry {
    pub value: String,
    pub created_at: u64,      // Unix timestamp ms
    pub expires_at: Option<u64>, // Unix timestamp ms, None = no expiry
}

impl CacheEntry {
    pub fn new(value: String, ttl_seconds: Option<u64>) -> Self;
    pub fn is_expired(&self) -> bool;
    pub fn ttl_remaining(&self) -> Option<u64>;
}
```

### 2. LRU Tracker (`src/cache/lru.rs`)

Tracks access order for LRU eviction using a VecDeque.

```rust
pub struct LruTracker {
    order: VecDeque<String>,
}

impl LruTracker {
    pub fn new() -> Self;
    pub fn touch(&mut self, key: &str);      // Move to front
    pub fn remove(&mut self, key: &str);     // Remove key
    pub fn evict_oldest(&mut self) -> Option<String>; // Pop back
    pub fn len(&self) -> usize;
}
```

### 3. Cache Store (`src/cache/store.rs`)

Main cache engine combining HashMap storage with LRU tracking.

```rust
pub struct CacheStore {
    entries: HashMap<String, CacheEntry>,
    lru: LruTracker,
    stats: CacheStats,
    max_entries: usize,
    default_ttl: u64,
}

impl CacheStore {
    pub fn new(max_entries: usize, default_ttl: u64) -> Self;
    pub fn set(&mut self, key: String, value: String, ttl: Option<u64>) -> Result<()>;
    pub fn get(&mut self, key: &str) -> Result<String>;
    pub fn delete(&mut self, key: &str) -> Result<()>;
    pub fn stats(&self) -> CacheStats;
    pub fn cleanup_expired(&mut self) -> usize;
    pub fn len(&self) -> usize;
}
```

### 4. Cache Statistics (`src/cache/stats.rs`)

Tracks cache performance metrics.

```rust
#[derive(Clone, Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_entries: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64;
}
```

### 5. API Handlers (`src/api/handlers.rs`)

HTTP request handlers for each endpoint.

```rust
pub async fn set_handler(
    State(state): State<AppState>,
    Json(req): Json<SetRequest>,
) -> Result<Json<SetResponse>, CacheError>;

pub async fn get_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<GetResponse>, CacheError>;

pub async fn delete_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<DeleteResponse>, CacheError>;

pub async fn stats_handler(
    State(state): State<AppState>,
) -> Json<StatsResponse>;

pub async fn health_handler() -> Json<HealthResponse>;
```

### 6. Request/Response Models (`src/models/`)

```rust
// Request DTOs
#[derive(Deserialize)]
pub struct SetRequest {
    pub key: String,
    pub value: String,
    pub ttl: Option<u64>,
}

// Response DTOs
#[derive(Serialize)]
pub struct GetResponse {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_entries: usize,
    pub hit_rate: f64,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
}
```

### 7. Application State

```rust
pub struct AppState {
    pub cache: Arc<RwLock<CacheStore>>,
}
```

## Data Models

### Cache Entry Lifecycle

```
┌─────────────┐     SET      ┌─────────────┐
│   Created   │ ──────────── │   Active    │
└─────────────┘              └──────┬──────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    v               v               v
             ┌──────────┐    ┌──────────┐    ┌──────────┐
             │  DELETE  │    │ TTL Exp  │    │ LRU Evict│
             └──────────┘    └──────────┘    └──────────┘
                    │               │               │
                    v               v               v
             ┌─────────────────────────────────────────┐
             │              Removed                    │
             └─────────────────────────────────────────┘
```

### Memory Layout

| Component | Size per Entry | Notes |
|-----------|---------------|-------|
| HashMap entry | ~80 bytes | Key + pointer overhead |
| CacheEntry | ~56 bytes | value String + timestamps |
| LRU entry | ~24 bytes | String in VecDeque |
| **Total** | ~160 bytes | + value.len() |

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the prework analysis, the following correctness properties must be verified:

### Property 1: Round-trip Storage Consistency

*For any* valid key-value pair, storing the pair and then retrieving it (before expiration) SHALL return the exact same value that was stored.

**Validates: Requirements 1.1, 1.2**

### Property 2: Delete Removes Entry

*For any* key that exists in the cache, after a DELETE operation, a subsequent GET operation SHALL return a "not found" result.

**Validates: Requirements 1.3, 1.4**

### Property 3: Overwrite Semantics

*For any* key, storing a value V1 and then storing a value V2 with the same key SHALL result in GET returning V2.

**Validates: Requirements 1.5**

### Property 4: TTL Expiration Behavior

*For any* entry stored with a TTL, after the TTL duration has elapsed, a GET operation SHALL return a "not found" result.

**Validates: Requirements 2.1, 2.2**

### Property 5: LRU Eviction Order

*For any* sequence of cache operations that fills the cache to capacity, when a new entry is added, the entry that was accessed least recently SHALL be evicted.

**Validates: Requirements 3.1, 3.4**

### Property 6: LRU Access Tracking

*For any* GET or PUT operation on an existing key, that key SHALL become the most recently used and SHALL NOT be the next eviction candidate.

**Validates: Requirements 3.2, 3.3**

### Property 7: Capacity Enforcement

*For any* sequence of SET operations, the number of entries in the cache SHALL never exceed MAX_ENTRIES.

**Validates: Requirements 3.1, 8.2**

### Property 8: Statistics Accuracy

*For any* sequence of cache operations, the statistics (hits, misses, evictions) SHALL accurately reflect the number of each operation type that occurred.

**Validates: Requirements 6.1, 6.2, 3.5, 6.4**

### Property 9: Concurrent Operation Correctness

*For any* set of concurrent read and write operations, all reads SHALL return either a complete old value or a complete new value, never partial or corrupted data.

**Validates: Requirements 5.1, 5.2, 5.3**

### Property 10: Error Response Format

*For any* error condition, the HTTP response SHALL include a JSON body with an "error" field containing a descriptive message.

**Validates: Requirements 7.1, 7.2, 7.5**

## Error Handling

### Error Types

```rust
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Key expired: {0}")]
    Expired(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Cache full: {0}")]
    CacheFull(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

### HTTP Status Mapping

| Error Type | HTTP Status | Description |
|------------|-------------|-------------|
| NotFound | 404 | Key does not exist |
| Expired | 404 | Key TTL has expired |
| InvalidRequest | 400 | Malformed request |
| CacheFull | 503 | Eviction failed |
| Internal | 500 | Server error |

## Testing Strategy

### Dual Testing Approach

The testing strategy combines unit tests for specific examples and property-based tests for universal correctness guarantees.

### Property-Based Testing Library

**Library:** `proptest` (Rust)

Property-based tests will be configured to run a minimum of 100 iterations per property.

### Unit Tests

Unit tests cover:
- CacheEntry creation and expiration logic
- LruTracker touch, remove, and eviction operations
- CacheStore CRUD operations
- API endpoint request/response handling
- Error conversion and HTTP status codes

### Property-Based Tests

Each correctness property from the design will be implemented as a property-based test:

1. **Round-trip test**: Generate random key-value pairs, store and retrieve
2. **Delete test**: Generate random keys, store, delete, verify not found
3. **Overwrite test**: Generate key with two values, verify second returned
4. **TTL test**: Generate entries with short TTL, verify expiration
5. **LRU order test**: Fill cache, verify eviction order matches access order
6. **LRU tracking test**: Access entries, verify they move to front
7. **Capacity test**: Generate many entries, verify count never exceeds max
8. **Statistics test**: Perform operations, verify stats match
9. **Concurrency test**: Spawn concurrent operations, verify no corruption
10. **Error format test**: Trigger errors, verify JSON structure

### Test Annotations

Each property-based test MUST include a comment referencing the correctness property:
```rust
// **Feature: local-cache-server, Property 1: Round-trip Storage Consistency**
#[test]
fn prop_roundtrip_storage() { ... }
```

### Integration Tests

Integration tests verify end-to-end API behavior:
- Full request/response cycle for each endpoint
- Concurrent client simulation
- TTL expiration via API
- Statistics accuracy after operations

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MAX_ENTRIES` | 1000 | Maximum cache entries |
| `DEFAULT_TTL` | 300 | Default TTL in seconds |
| `SERVER_PORT` | 3000 | HTTP server port |
| `CLEANUP_INTERVAL` | 1 | Cleanup frequency in seconds |
| `RUST_LOG` | info | Log level |

### Configuration Loading

```rust
pub struct Config {
    pub max_entries: usize,
    pub default_ttl: u64,
    pub server_port: u16,
    pub cleanup_interval: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            max_entries: env::var("MAX_ENTRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            // ... other fields
        }
    }
}
```

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| GET | O(1) average | HashMap lookup + LRU touch |
| SET | O(1) average | HashMap insert + LRU touch |
| DELETE | O(1) average | HashMap remove + LRU remove |
| LRU touch | O(n) worst | VecDeque removal |
| LRU evict | O(1) | VecDeque pop_back |
| TTL cleanup | O(n) | Iterate all entries |

**Note:** LRU touch is O(n) due to VecDeque linear search. For production with high throughput, consider using a LinkedHashMap or doubly-linked list with HashMap for O(1) operations.

## Concurrency Model

```
                    +---------------------------+
                    |  Arc<RwLock<CacheStore>>  |
                    +---------------------------+
                              |
          +-------------------+-------------------+
          |                   |                   |
          v                   v                   v
    +-----------+       +-----------+       +-----------+
    | Reader 1  |       | Reader N  |       | Writer    |
    | (GET)     |       | (GET)     |       | (SET/DEL) |
    +-----------+       +-----------+       +-----------+
```

- **RwLock** allows multiple concurrent readers
- Writers acquire exclusive lock
- Background cleanup task acquires write lock briefly
- Lock contention minimized by short critical sections
