# Mini Redis - Architecture Documentation

## Overview

A lightweight in-memory cache server implemented in Rust, featuring TTL-based expiration and LRU eviction. Exposes a REST API for key/value operations.

---

## System Architecture

```
                              MINI REDIS ARCHITECTURE

    +------------------+
    |   HTTP Client    |
    |   (REST API)     |
    +--------+---------+
             |
             | HTTP Requests
             v
    +--------+---------+
    |   Axum Router    |
    |   /set /get /del |
    |   /stats         |
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
    |  | Entry                       |    |
    |  | - value: String             |    |
    |  | - expires_at: Option<u64>   |    |
    |  | - created_at: u64           |    |
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

---

## Module Structure

```
src/
+-- main.rs                 Application entry point, server startup
|
+-- cache/
|   +-- mod.rs              Module exports
|   +-- store.rs            CacheStore: HashMap + TTL logic
|   +-- lru.rs              LRU tracker: eviction strategy
|   +-- entry.rs            CacheEntry: value + metadata
|
+-- api/
|   +-- mod.rs              Module exports
|   +-- handlers.rs         Axum route handlers
|   +-- routes.rs           Router configuration
|
+-- models/
|   +-- mod.rs              Module exports
|   +-- requests.rs         Request DTOs (SetRequest)
|   +-- responses.rs        Response DTOs (GetResponse, StatsResponse)
|
+-- error.rs                Unified error types
```

---

## Data Flow

### SET Operation

```
Client -> PUT /set
       -> SetRequest { key, value, ttl }
       -> CacheStore.set(key, value, ttl)
       -> LRU.touch(key)
       -> If full: LRU.evict_oldest()
       -> Response 200 OK
```

### GET Operation

```
Client -> GET /get/:key
       -> CacheStore.get(key)
       -> Check TTL expiration
       -> If expired: remove + return 404
       -> If found: LRU.touch(key) + return value
       -> If not found: return 404
```

### DELETE Operation

```
Client -> DELETE /del/:key
       -> CacheStore.delete(key)
       -> LRU.remove(key)
       -> Response 200 OK
```

### Background TTL Cleanup

```
Loop every 1 second:
  -> Iterate all entries
  -> Remove expired entries
  -> Update stats.evictions
```

---

## Core Components

### CacheStore

Primary data structure holding all cache entries.

```rust
struct CacheStore {
    entries: HashMap<String, CacheEntry>,
    lru: LruTracker,
    stats: CacheStats,
    max_entries: usize,
}
```

Responsibilities:
- Store and retrieve key/value pairs
- Check TTL expiration on access
- Trigger LRU eviction when capacity reached
- Track hit/miss statistics

### LruTracker

Tracks access order for LRU eviction.

```rust
struct LruTracker {
    order: VecDeque<String>,
}
```

Responsibilities:
- Move accessed keys to front (most recent)
- Return oldest key for eviction
- Remove keys when deleted

### CacheEntry

Individual cache entry with metadata.

```rust
struct CacheEntry {
    value: String,
    created_at: u64,
    expires_at: Option<u64>,
}
```

### CacheStats

Statistics for monitoring.

```rust
struct CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
}
```

---

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
    | Handler 1 |       | Handler 2 |       | Cleanup   |
    | (read)    |       | (write)   |       | Task      |
    +-----------+       +-----------+       +-----------+
```

- Multiple readers allowed (GET operations)
- Single writer at a time (SET, DELETE)
- Background cleanup task acquires write lock periodically

---

## API Endpoints

| Method | Endpoint      | Description              | Request Body                          |
|--------|---------------|--------------------------|---------------------------------------|
| PUT    | /set          | Store key/value with TTL | `{ "key": "...", "value": "...", "ttl": 30 }` |
| GET    | /get/:key     | Retrieve value by key    | -                                     |
| DELETE | /del/:key     | Delete key               | -                                     |
| GET    | /stats        | Get cache statistics     | -                                     |
| GET    | /health       | Health check             | -                                     |

---

## Configuration

| Parameter       | Default | Description                    |
|-----------------|---------|--------------------------------|
| MAX_ENTRIES     | 1000    | Maximum cache entries          |
| CLEANUP_INTERVAL| 1s      | TTL cleanup frequency          |
| DEFAULT_TTL     | 300s    | Default TTL if not specified   |
| SERVER_PORT     | 3000    | HTTP server port               |

---

## Error Handling

All errors are handled gracefully with appropriate HTTP status codes:

| Error Type      | HTTP Status | Description              |
|-----------------|-------------|--------------------------|
| Key not found   | 404         | Key does not exist       |
| Key expired     | 404         | Key TTL has expired      |
| Invalid request | 400         | Malformed request body   |
| Server error    | 500         | Internal server error    |

---

## Performance Characteristics

| Operation        | Time Complexity |
|------------------|-----------------|
| GET              | O(1) average    |
| SET              | O(1) average    |
| DELETE           | O(1) average    |
| LRU touch        | O(n) worst case |
| LRU evict        | O(1)            |
| TTL cleanup      | O(n)            |

Note: LRU touch is O(n) due to VecDeque removal. For production, consider using a LinkedHashMap.

---

## Testing Strategy

1. Unit tests for CacheStore operations
2. Unit tests for LRU eviction logic
3. Unit tests for TTL expiration
4. Integration tests for API endpoints
5. Concurrency tests for thread safety
