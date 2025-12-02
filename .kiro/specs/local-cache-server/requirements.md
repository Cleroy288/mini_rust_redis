# Requirements Document

## Introduction

This document specifies the requirements for a lightweight in-memory cache server implemented in Rust. The system provides Redis-like functionality with TTL (Time-To-Live) based expiration and LRU (Least Recently Used) eviction policies, exposed through a REST API. This project demonstrates systems programming, memory data structures, concurrency, HTTP API design, and clean async Rust architecture.

## Glossary

- **Cache Server**: The in-memory key-value storage system that handles client requests
- **TTL (Time-To-Live)**: The duration in seconds after which a cache entry automatically expires
- **LRU (Least Recently Used)**: An eviction policy that removes the least recently accessed entries when capacity is reached
- **Cache Entry**: A single key-value pair stored in the cache with associated metadata (creation time, expiration time)
- **Eviction**: The process of removing entries from the cache to free up space
- **Cache Hit**: A successful retrieval of a value from the cache
- **Cache Miss**: A failed retrieval attempt when the key does not exist or has expired

## Requirements

### Requirement 1: Key-Value Storage

**User Story:** As a client application, I want to store and retrieve key-value pairs, so that I can cache data for fast access.

#### Acceptance Criteria

1. WHEN a client sends a PUT request with a key, value, and optional TTL THEN the Cache Server SHALL store the entry and return a success response
2. WHEN a client sends a GET request for an existing, non-expired key THEN the Cache Server SHALL return the stored value
3. WHEN a client sends a DELETE request for an existing key THEN the Cache Server SHALL remove the entry and return a success response
4. WHEN a client sends a GET request for a non-existent key THEN the Cache Server SHALL return a 404 Not Found response
5. WHEN storing a new entry with a key that already exists THEN the Cache Server SHALL overwrite the existing value and reset the TTL

### Requirement 2: TTL-Based Expiration

**User Story:** As a client application, I want cache entries to automatically expire after a specified time, so that stale data is removed without manual intervention.

#### Acceptance Criteria

1. WHEN a client stores an entry with a TTL value THEN the Cache Server SHALL mark the entry to expire after the specified number of seconds
2. WHEN a client retrieves an entry that has exceeded its TTL THEN the Cache Server SHALL return a 404 Not Found response and remove the expired entry
3. WHILE the Cache Server is running THEN the Cache Server SHALL execute a background cleanup task that removes expired entries periodically
4. WHEN a client stores an entry without specifying a TTL THEN the Cache Server SHALL use a configurable default TTL value
5. WHEN the background cleanup task runs THEN the Cache Server SHALL remove all entries whose TTL has expired

### Requirement 3: LRU Eviction Policy

**User Story:** As a system administrator, I want the cache to automatically evict least recently used entries when capacity is reached, so that memory usage remains bounded.

#### Acceptance Criteria

1. WHEN the cache reaches maximum capacity and a new entry is added THEN the Cache Server SHALL evict the least recently used entry to make space
2. WHEN a client retrieves an existing entry THEN the Cache Server SHALL mark that entry as most recently used
3. WHEN a client updates an existing entry THEN the Cache Server SHALL mark that entry as most recently used
4. WHEN multiple entries need eviction THEN the Cache Server SHALL evict entries in order from least to most recently used
5. WHEN an entry is evicted due to LRU policy THEN the Cache Server SHALL increment the eviction counter in statistics

### Requirement 4: REST API Interface

**User Story:** As a client application, I want to interact with the cache through a REST API, so that I can integrate with any programming language or platform.

#### Acceptance Criteria

1. WHEN the server starts THEN the Cache Server SHALL listen for HTTP requests on a configurable port
2. WHEN a client sends a PUT request to /set with JSON body containing key, value, and optional ttl THEN the Cache Server SHALL process the store operation
3. WHEN a client sends a GET request to /get/:key THEN the Cache Server SHALL process the retrieval operation
4. WHEN a client sends a DELETE request to /del/:key THEN the Cache Server SHALL process the deletion operation
5. WHEN a client sends a GET request to /stats THEN the Cache Server SHALL return cache statistics including hits, misses, and evictions
6. WHEN a client sends a GET request to /health THEN the Cache Server SHALL return a health status response

### Requirement 5: Concurrent Access

**User Story:** As a client application, I want to access the cache from multiple threads simultaneously, so that the cache can handle high-throughput workloads.

#### Acceptance Criteria

1. WHEN multiple clients send concurrent read requests THEN the Cache Server SHALL process all requests without data corruption
2. WHEN multiple clients send concurrent write requests THEN the Cache Server SHALL serialize writes to maintain data consistency
3. WHEN a read and write operation occur simultaneously THEN the Cache Server SHALL ensure the read returns either the old or new value, never partial data
4. WHEN the background cleanup task runs THEN the Cache Server SHALL acquire appropriate locks without blocking client requests excessively

### Requirement 6: Cache Statistics

**User Story:** As a system administrator, I want to monitor cache performance metrics, so that I can optimize cache configuration and troubleshoot issues.

#### Acceptance Criteria

1. WHEN a cache hit occurs THEN the Cache Server SHALL increment the hit counter
2. WHEN a cache miss occurs THEN the Cache Server SHALL increment the miss counter
3. WHEN an entry is evicted due to LRU policy THEN the Cache Server SHALL increment the eviction counter
4. WHEN the /stats endpoint is called THEN the Cache Server SHALL return current values for hits, misses, evictions, and total entries
5. WHEN calculating hit rate THEN the Cache Server SHALL compute hits divided by total requests (hits plus misses)

### Requirement 7: Error Handling

**User Story:** As a client application, I want clear error responses when operations fail, so that I can handle errors appropriately.

#### Acceptance Criteria

1. WHEN a request contains invalid JSON THEN the Cache Server SHALL return a 400 Bad Request response with an error message
2. WHEN a requested key is not found THEN the Cache Server SHALL return a 404 Not Found response with an error message
3. WHEN an internal error occurs THEN the Cache Server SHALL return a 500 Internal Server Error response with an error message
4. WHEN the cache is full and eviction fails THEN the Cache Server SHALL return a 503 Service Unavailable response
5. WHEN an error response is returned THEN the Cache Server SHALL include a JSON body with an "error" field containing the message

### Requirement 8: Configuration

**User Story:** As a system administrator, I want to configure cache parameters, so that I can tune the cache for different deployment scenarios.

#### Acceptance Criteria

1. WHEN the server starts THEN the Cache Server SHALL read configuration from environment variables or defaults
2. WHEN MAX_ENTRIES is configured THEN the Cache Server SHALL limit the cache to that number of entries
3. WHEN DEFAULT_TTL is configured THEN the Cache Server SHALL use that value for entries without explicit TTL
4. WHEN SERVER_PORT is configured THEN the Cache Server SHALL listen on that port
5. WHEN CLEANUP_INTERVAL is configured THEN the Cache Server SHALL run the background cleanup task at that frequency
