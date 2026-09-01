# Part 14: Troubleshooting Guide

---

# Chapter 70: Common Build Errors and Fixes

This chapter covers the most common Rust build errors and how to fix them.

## Lifetime Errors

### Error: missing lifetime specifier

```rust
// ERROR
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[0..i];
        }
    }
    s
}
```

**Fix:** Add explicit lifetime:

```rust
fn first_word<'a>(s: &'a str) -> &'a str {
    // ...
}
```

### Error: cannot borrow as mutable because it is also borrowed as immutable

```rust
// ERROR
let mut data = vec![1, 2, 3];
let first = &data[0];  // Immutable borrow
data.push(4);          // Mutable borrow while immutable borrow exists
```

**Fix:** Clone the data or restructure the code:

```rust
let mut data = vec![1, 2, 3];
let first = data[0];  // Copy the value
data.push(4);
println!("{}", first);
```

## Type Errors

### Error: expected struct, found enum

```rust
// ERROR
let result: Result<String> = Ok("hello".to_string());
```

**Fix:** Specify both type parameters:

```rust
let result: Result<String, Error> = Ok("hello".to_string());
```

### Error: mismatched types

```rust
// ERROR
let x: i32 = "hello";
```

**Fix:** Convert the type:

```rust
let x: i32 = "hello".parse().unwrap();
```

## Move Errors

### Error: use of moved value

```rust
// ERROR
let s = String::from("hello");
let s2 = s;
println!("{}", s);  // Error: s was moved to s2
```

**Fix:** Clone the value or use references:

```rust
let s = String::from("hello");
let s2 = s.clone();  // Clone
println!("{}", s);
```

## Trait Object Errors

### Error: the size for values of type cannot be known at compilation time

```rust
// ERROR
fn process(item: dyn SiteScraper) {
    // ...
}
```

**Fix:** Use a reference or Box:

```rust
fn process(item: &dyn SiteScraper) {
    // ...
}

fn process(item: Box<dyn SiteScraper>) {
    // ...
}
```

## Async Errors

### Error: async non-`Send` future

```rust
// ERROR
async fn process() {
    let data = Rc::new(42);  // Rc is not Send
    tokio::spawn(async move {
        println!("{}", data);
    });
}
```

**Fix:** Use Arc instead of Rc:

```rust
async fn process() {
    let data = Arc::new(42);  // Arc is Send
    tokio::spawn(async move {
        println!("{}", data);
    });
}
```

## 📝 Practice Exercises

1. **Lifetime Errors:** Write 3 functions that produce lifetime errors and fix them.

2. **Move Errors:** Write code that demonstrates move semantics and fix the errors.

3. **Trait Objects:** Create a trait object collection and fix any compilation errors.

---

# Chapter 71: Runtime Debugging Techniques

When things go wrong at runtime, you need debugging tools.

## println! Debugging

```rust
fn debug_function(input: &str) {
    println!("Input: {:?}", input);
    let parsed = parse_input(input);
    println!("Parsed: {:?}", parsed);
    let result = process_parsed(&parsed);
    println!("Result: {:?}", result);
}
```

## tracing Debug Logging

```rust
use tracing::debug;

async fn debug_function(input: &str) {
    debug!(input = %input, "Processing input");
    let parsed = parse_input(input);
    debug!(parsed = ?parsed, "Input parsed");
    let result = process_parsed(&parsed).await;
    debug!(result = ?result, "Processing complete");
}
```

## Conditional Compilation

```rust
#[cfg(debug_assertions)]
fn debug_only_function() {
    println!("Debug mode only");
}

#[cfg(not(debug_assertions))]
fn debug_only_function() {
    // No-op in release mode
}
```

## Debug Assertions

```rust
fn calculate_score(values: &[i32]) -> f64 {
    let sum: i32 = values.iter().sum();
    let count = values.len();
    
    debug_assert!(count > 0, "values must not be empty");
    debug_assert!(sum >= 0, "sum must be non-negative");
    
    sum as f64 / count as f64
}
```

## Remote Debugging

For production issues, use structured logging and metrics:

```rust
// Add context to errors
async fn process_request(url: &str) -> Result<(), AppError> {
    let request_id = Uuid::new_v4();
    tracing::info!(request_id = %request_id, url = %url, "Processing request");
    
    match fetch_and_export(url).await {
        Ok(result) => {
            tracing::info!(request_id = %request_id, "Request complete");
            Ok(result)
        }
        Err(e) => {
            tracing::error!(request_id = %request_id, error = %e, "Request failed");
            Err(e)
        }
    }
}
```

## 📝 Practice Exercises

1. **Debug Logging:** Add debug logging to 5 functions and verify the output.

2. **Debug Assertions:** Add debug assertions to validate function inputs.

3. **Error Context:** Add request IDs to all error messages for tracing.

---

# Chapter 72: Memory Leak Investigation

Rust prevents most memory leaks, but some patterns can still cause them.

## Common Leak Patterns

### Circular References with Rc

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    parent: Option<Rc<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}

// This creates a circular reference:
// parent -> child -> parent
```

**Fix:** Use `Weak` references:

```rust
use std::rc::Weak;

struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}
```

### Unbounded Caches

```rust
// BAD: Unbounded cache grows forever
let mut cache: HashMap<String, String> = HashMap::new();

// GOOD: Bounded cache with LRU eviction
use lru::LruCache;
let mut cache: LruCache<String, String> = LruCache::new(1000);
```

### Forgotten Drop

```rust
// BAD: Resource not dropped
fn process() {
    let resource = Resource::new();
    // ... do work ...
    // resource is dropped at end of function, but what if we return early?
    if error {
        return Err(e);  // resource is dropped here, which is correct
    }
    Ok(())
}

// GOOD: Explicit cleanup
fn process() -> Result<(), Error> {
    let resource = Resource::new();
    let result = do_work(&resource);
    resource.cleanup()?;  // Explicit cleanup
    result
}
```

## Memory Profiling

```bash
# Using Valgrind
valgrind --leak-check=full ./fichub

# Using heaptrack
heaptrack ./fichub

# Using Rust's built-in profiling
RUSTFLAGS="-Z instrument-coverage" cargo build
```

## Monitoring Memory Usage

```rust
use std::alloc::{GlobalAlloc, Layout, System};

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}
```

## 📝 Practice Exercises

1. **Circular References:** Create a circular reference and fix it using Weak.

2. **Cache Monitoring:** Implement a bounded cache and monitor its size.

3. **Memory Profiling:** Use valgrind to profile FicHub's memory usage.

---

# Chapter 73: Performance Profiling

This chapter covers tools and techniques for profiling Rust applications.

## cargo-flamegraph

```bash
# Install
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench my_benchmark

# Open flamegraph.svg in a browser
```

## cargo-bench

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("generate_url_id", |b| {
        b.iter(|| generate_url_id(black_box(42), black_box("story_123")))
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

## perf on Linux

```bash
# Record performance data
perf record -g ./fichub

# Analyze
perf report

# Flamegraph from perf data
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

## tokio-console

Real-time Tokio task monitoring:

```rust
// Add to Cargo.toml
// [dependencies]
// console-subscriber = "0.1"

fn main() {
    console_subscriber::init();
    
    // Your application code...
}
```

## Common Performance Issues

### Too Many Allocations

```rust
// BAD: Allocating in a loop
for i in 0..10000 {
    let s = format!("item_{}", i);  // Allocation per iteration
    process(&s);
}

// GOOD: Reuse buffer
let mut s = String::new();
for i in 0..10000 {
    s.clear();
    s.push_str("item_");
    s.push_str(&i.to_string());
    process(&s);
}
```

### Excessive Cloning

```rust
// BAD: Cloning large data
let data = vec![0u8; 1024 * 1024];
let data_clone = data.clone();  // 1MB copy

// GOOD: Use references or Arc
let data = Arc::new(vec![0u8; 1024 * 1024]);
let data_ref = data.clone();  // Arc clone is cheap
```

## 📝 Practice Exercises

1. **Flamegraph:** Generate a flamegraph of FicHub handling requests.

2. **Benchmarking:** Write benchmarks for the EPUB generation function.

3. **tokio-console:** Set up tokio-console and monitor task execution.

---

# Chapter 74: Database Troubleshooting

Database issues are common in production. This chapter covers troubleshooting techniques.

## Connection Issues

### Connection Refused

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check if port is open
ss -tlnp | grep 5432

# Test connection
psql -h localhost -U fichub -d fichub -c "SELECT 1;"
```

### Authentication Failed

```bash
# Check pg_hba.conf
sudo cat /etc/postgresql/16/main/pg_hba.conf | grep fichub

# Update authentication method
sudo nano /etc/postgresql/16/main/pg_hba.conf
# Change "peer" to "md5"

# Restart PostgreSQL
sudo systemctl restart postgresql
```

### Too Many Connections

```sql
-- Check current connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub';

-- Kill idle connections
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'fichub'
AND state = 'idle'
AND query_start < now() - interval '10 minutes';
```

## Slow Queries

### Find Slow Queries

```sql
-- Enable slow query logging
ALTER SYSTEM SET log_min_duration_statement = 1000;  -- Log queries > 1s
SELECT pg_reload_conf();

-- Check pg_stat_statements
CREATE EXTENSION pg_stat_statements;
SELECT query, calls, mean_exec_time, total_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

### Analyze Slow Queries

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM fic_info WHERE words > 10000;
```

## Lock Issues

### Check for Locks

```sql
SELECT
    blocked_locks.pid AS blocked_pid,
    blocked_activity.usename AS blocked_user,
    blocking_locks.pid AS blocking_pid,
    blocking_activity.usename AS blocking_user,
    blocked_activity.query AS blocked_statement,
    blocking_activity.query AS current_statement_in_blocking_process
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks
    ON blocking_locks.locktype = blocked_locks.locktype
    AND blocking_locks.relation = blocked_locks.relation
    AND blocking_locks.pid != blocked_locks.pid
JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;
```

### Kill Blocking Queries

```sql
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE pid = <blocking_pid>;
```

## 📝 Practice Exercises

1. **Connection Debugging:** Simulate a connection refused error and fix it.

2. **Slow Query Analysis:** Find and optimize a slow query in FicHub.

3. **Lock Debugging:** Simulate a lock contention scenario and resolve it.

---

# Chapter 75: Network and Connection Issues

Network issues can cause intermittent failures. This chapter covers debugging techniques.

## Connection Timeouts

```rust
// Configure timeouts
let client = reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(10))
    .timeout(Duration::from_secs(30))
    .build()?;
```

## DNS Resolution Issues

```bash
# Test DNS resolution
nslookup archiveofourown.org

# Check /etc/hosts
cat /etc/hosts | grep archiveofourown

# Use alternative DNS
echo "nameserver 8.8.8.8" > /tmp/resolv.conf
```

## TLS Issues

```rust
// Use rustls for better cross-platform TLS
let client = reqwest::Client::builder()
    .use_rustls_tls()
    .build()?;
```

## Connection Pool Exhaustion

```rust
// Monitor pool usage
let stats = pool.stats();
if stats.waiting() > 0 {
    tracing::warn!(
        active = stats.active_connections(),
        idle = stats.idle_connections(),
        waiting = stats.waiting(),
        "Connection pool under pressure"
    );
}
```

## Retry Logic

```rust
async fn fetch_with_retry(
    client: &reqwest::Client,
    url: &str,
    max_retries: u32,
) -> Result<String, reqwest::Error> {
    let mut delay = Duration::from_secs(1);
    
    for attempt in 0..max_retries {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => {
                return response.text().await;
            }
            Ok(response) if response.status().is_server_error() => {
                // Server error — retry
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Ok(response) => {
                // Client error — don't retry
                return Err(response.error_for_status().unwrap_err());
            }
            Err(e) if e.is_timeout() || e.is_connect() => {
                // Network error — retry
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    
    Err(reqwest::Error::new(reqwest::error::Kind::Request, "max retries exceeded"))
}
```

## 📝 Practice Exercises

1. **Timeout Handling:** Implement different timeout strategies for different endpoints.

2. **Retry Logic:** Implement retry with exponential backoff for all external calls.

3. **Connection Monitoring:** Add monitoring for all external connections (database, Redis, HTTP).

