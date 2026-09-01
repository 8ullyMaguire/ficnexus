# Part 8: Testing and Polish

---

# Chapter 34: Backend Testing

Testing is crucial for production software. This chapter covers unit tests, integration tests, and testing patterns in Rust.

## Unit Tests

Unit tests verify individual functions in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_url_id_deterministic() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(1, "story_123");
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_generate_url_id_different_source() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(2, "story_123");
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_generate_url_id_length() {
        let id = generate_url_id(42, "abc123");
        assert_eq!(id.len(), 12);
    }
    
    #[test]
    fn test_generate_url_id_hex_chars() {
        let id = generate_url_id(7, "test_url");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
```

### Test Organization

Tests are organized in modules within each source file:

```rust
// At the end of src/scrape/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_name() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests in a specific module
cargo test scrape::tests

# Run a specific test
cargo test test_generate_url_id_deterministic

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests sequentially
cargo test -- --test-threads=1
```

## Integration Tests

Integration tests verify the interaction between components:

```rust
// tests/api_tests.rs
use reqwest;
use serde_json::json;

#[tokio::test]
async fn test_epub_endpoint() {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:3000/api/v0/epub")
        .query(&[("q", "https://archiveofourown.org/works/123456")])
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["err"], 0);
    assert!(body["url_id"].is_string());
}

#[tokio::test]
async fn test_search_endpoint() {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:3000/api/v0/search")
        .query(&[("q", "harry potter")])
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["err"], 0);
    assert!(body["results"].is_array());
}
```

## Test Fixtures

```rust
// tests/fixtures.rs
use sqlx::PgPool;

pub async fn setup_test_db() -> PgPool {
    let pool = sqlx::PgPool::connect("postgres://localhost/fichub_test")
        .await
        .unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();
    
    pool
}

pub async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query("DELETE FROM fic_info").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM fic_tags").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM tags").execute(pool).await.unwrap();
}
```

## Test Patterns

### Table-Driven Tests

```rust
#[test]
fn test_escape_xml() {
    let cases = vec![
        ("hello", "hello"),
        ("<b>bold</b>", "&lt;b&gt;bold&lt;/b&gt;"),
        ("a & b", "a &amp; b"),
        ("\"quoted\"", "&quot;quoted&quot;"),
    ];
    
    for (input, expected) in cases {
        assert_eq!(escape_xml(input), expected, "Failed for input: {}", input);
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_url_id_is_always_hex(s in "[a-z0-9]+", id in 0i64..10000) {
        let url_id = generate_url_id(id, &s);
        prop_assert!(url_id.chars().all(|c| c.is_ascii_hexdigit()));
        prop_assert_eq!(url_id.len(), 12);
    }
    
    #[test]
    fn test_url_id_is_deterministic(s in "[a-z0-9]+", id in 0i64..10000) {
        let id1 = generate_url_id(id, &s);
        let id2 = generate_url_id(id, &s);
        prop_assert_eq!(id1, id2);
    }
}
```

## 📝 Practice Exercises

1. **Unit Tests:** Write unit tests for the `escape_xml` function covering all edge cases.

2. **Integration Tests:** Write integration tests for the search endpoint with different filter combinations.

3. **Property Tests:** Write property tests for the `generate_url_id` function.

4. **Test Coverage:** Use `cargo-tarpaulin` to measure test coverage and identify untested code.

---

# Chapter 35: Production Hardening

Production systems need to be robust, secure, and observable. This chapter covers security best practices, monitoring, and operational concerns.

## Security Checklist

### Input Validation

```rust
fn validate_url(url: &str) -> Result<(), AppError> {
    // Check URL length
    if url.len() > 2048 {
        return Err(AppError::BadRequest(-1, "URL too long".into()));
    }
    
    // Check URL scheme
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::BadRequest(-1, "Invalid URL scheme".into()));
    }
    
    // Check for supported domains
    let supported_domains = [
        "archiveofourown.org",
        "fanfiction.net",
        "fictionpress.com",
        "forums.spacebattles.com",
        "forums.sufficientvelocity.com",
        "forum.questionablequesting.com",
    ];
    
    let url_lower = url.to_lowercase();
    if !supported_domains.iter().any(|d| url_lower.contains(d)) {
        return Err(AppError::BadRequest(-1, "Unsupported URL".into()));
    }
    
    Ok(())
}
```

### SQL Injection Prevention

Always use parameterized queries:

```rust
// DANGEROUS
let query = format!("SELECT * FROM fic_info WHERE title LIKE '%{}%'", user_input);

// SAFE
let query = "SELECT * FROM fic_info WHERE title LIKE $1";
sqlx::query(query)
    .bind(format!("%{}%", user_input))
    .fetch_all(&pool)
    .await?;
```

### Rate Limiting

```rust
// Apply rate limits to all endpoints
async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    
    match state.rate_limiter.check(ip, req.uri().path()).await? {
        RateLimitResult::Allowed => Ok(next.run(req).await),
        RateLimitResult::Wait(secs) => Err(AppError::RateLimited(secs)),
        RateLimitResult::Blocked => Err(AppError::BadRequest(-403, "blocked".into())),
    }
}
```

## Monitoring

### Structured Logging

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,fichub=debug".into()),
    )
    .with_target(false)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

### Metrics with Prometheus

```rust
use prometheus::{Encoder, IntCounter, Registry, TextEncoder};

lazy_static! {
    static ref REQUEST_COUNT: IntCounter = IntCounter::new(
        "fichub_requests_total",
        "Total number of requests"
    ).unwrap();
    
    static ref EXPORT_COUNT: IntCounter = IntCounter::new(
        "fichub_exports_total",
        "Total number of exports"
    ).unwrap();
}
```

## Graceful Shutdown

```rust
async fn run_server(config: Config) {
    let app = build_router(state).await;
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    // Handle shutdown signals
    let shutdown_signal = async {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutdown signal received");
    };
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
    
    // Cleanup
    tracing::info!("Shutting down...");
    // Close database pool, Redis connection, etc.
}
```

## 📝 Practice Exercises

1. **Security Audit:** Review FicHub's codebase for security vulnerabilities. What would you fix?

2. **Monitoring Dashboard:** Set up a Grafana dashboard with FicHub metrics.

3. **Alerting:** Configure alerts for error rate spikes and high latency.

---

# Chapter 36: What's Next?

Congratulations! You've built a complete, production-ready backend server in Rust. Let's recap what you've learned and look at what's next.

## What You've Learned

- **Rust fundamentals** — Ownership, borrowing, traits, generics
- **Web development** — Axum, HTTP, JSON, routing, middleware
- **Database** — PostgreSQL, SQLx, migrations, connection pooling
- **Web scraping** — HTML parsing, CSS selectors, the registry pattern
- **File generation** — EPUB, HTML bundles, ZIP archives
- **Caching** — Disk cache, cache invalidation, semaphores
- **Rate limiting** — Token buckets, Redis, Lua scripts
- **Recommendations** — Collaborative filtering, co-occurrence, scoring
- **Tags** — Tag resolution, voting, curator tools
- **Search** — Full-text search, dynamic query building
- **OPDS** — E-book catalog feeds
- **Deployment** — Docker, cross-compilation, systemd

## Possible Extensions

1. **User Authentication** — Add user accounts and personalized features
2. **Bookmarks** — Let users save and organize their favourite stories
3. **Reading Lists** — Create curated reading lists
4. **Notifications** — Alert users when new chapters are published
5. **Mobile App** — Build a native mobile app using the API
6. **AI Recommendations** — Use machine learning for better recommendations
7. **Multi-language Support** — Add internationalization
8. **Accessibility** — Improve accessibility for screen readers
9. **API Versioning** — Plan for API evolution
10. **Documentation** — Write comprehensive API documentation

## Community Resources

- **Rust Book** — https://doc.rust-lang.org/book/
- **Axum Documentation** — https://docs.rs/axum
- **SQLx Documentation** — https://docs.rs/sqlx
- **Tokio Documentation** — https://tokio.rs/tokio/tutorial

## Final Thoughts

Building FicHub has been a journey through the Rust ecosystem. You've seen how Rust's type system, ownership model, and async runtime come together to create a fast, safe, and maintainable backend. The patterns you've learned — trait objects for polymorphism, `?` for error propagation, `Arc` for shared state — are the building blocks of any Rust application.

The most important lesson is that Rust rewards upfront investment. The time you spend satisfying the borrow checker and thinking about types is paid back many times over in fewer bugs, better performance, and more maintainable code.

Keep building, keep learning, and most importantly, keep enjoying the process.

## 📝 Practice Exercises

1. **Feature Proposal:** Design a new feature for FicHub. Write a technical specification including API design, database schema, and implementation plan.

2. **Code Review:** Review a pull request in an open-source Rust project. What patterns do you recognize?

3. **Teach Someone:** Explain the ownership system to a friend. If you can teach it, you understand it.

4. **Build Something New:** Take what you've learned and build a new project. It could be a simple API, a CLI tool, or a web scraper for a different site.

