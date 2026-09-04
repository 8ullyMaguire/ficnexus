# Part 8: Testing and Polish

---

## Chapter 34: Backend Testing

Testing is how you sleep at night. FicHub has tests for every major component — from individual functions to complete API endpoints. Let's walk through the testing strategy.

### The Testing Pyramid

FicHub follows the testing pyramid:
1. **Unit tests** (many) — Test individual functions and types
2. **Integration tests** (some) — Test component interactions
3. **End-to-end tests** (few) — Test complete request flows

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test config::tests

# Run a specific test
cargo test test_from_env_defaults

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test -- --test-threads=4
```

### Unit Testing Patterns

**Testing Display implementations:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_bad_request() {
        let err = AppError::BadRequest(400, "invalid input".into());
        let s = format!("{}", err);
        assert!(s.contains("BadRequest"));
        assert!(s.contains("400"));
        assert!(s.contains("invalid input"));
    }
    
    #[test]
    fn test_display_rate_limited() {
        let err = AppError::RateLimited(30);
        let s = format!("{}", err);
        assert!(s.contains("RateLimited"));
        assert!(s.contains("30"));
    }
}
```

**Testing From implementations:**

```rust
#[test]
fn test_from_io_error() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app: AppError = io.into();
    match app {
        AppError::Internal(msg) => assert!(msg.contains("file not found")),
        _ => panic!("expected Internal"),
    }
}

#[test]
fn test_from_sqlx_error() {
    let sqlx = sqlx::Error::Protocol("bad query".into());
    let app: AppError = sqlx.into();
    match app {
        AppError::Database(msg) => assert!(msg.contains("bad query")),
        _ => panic!("expected Database"),
    }
}
```

**Testing configuration loading:**

```rust
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    keys: Vec<String>,
}

impl EnvGuard {
    fn new() -> Self { EnvGuard { keys: Vec::new() } }
    fn set(&mut self, key: &str, val: &str) {
        self.keys.push(key.to_string());
        unsafe { std::env::set_var(key, val); }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for key in &self.keys {
            unsafe { std::env::remove_var(key); }
        }
    }
}

#[test]
fn test_from_env_defaults() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test_db");
    guard.set("REDIS_URL", "redis://localhost/0");
    
    let config = Config::from_env();
    
    assert_eq!(config.database_url, "postgres://localhost/test_db");
    assert_eq!(config.cache_dir, PathBuf::from("./cache"));
    assert_eq!(config.app_port, 3000);
}

#[test]
#[should_panic(expected = "DATABASE_URL must be set")]
fn test_panics_without_database_url() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvGuard::new();
    // Don't set DATABASE_URL
    let _ = Config::from_env();
}
```

The `EnvGuard` pattern ensures test isolation — env vars are cleaned up after each test, even if the test panics.

**Testing EType parsing:**

```rust
#[test]
fn test_etype_from_str() {
    assert_eq!("epub".parse::<EType>().unwrap(), EType::Epub);
    assert_eq!("html".parse::<EType>().unwrap(), EType::Html);
    assert_eq!("mobi".parse::<EType>().unwrap(), EType::Mobi);
    assert_eq!("pdf".parse::<EType>().unwrap(), EType::Pdf);
    assert!("invalid".parse::<EType>().is_err());
    assert_eq!("EPUB".parse::<EType>().unwrap(), EType::Epub);
}

#[test]
fn test_etype_suffix() {
    assert_eq!(EType::Epub.suffix(), ".epub");
    assert_eq!(EType::Html.suffix(), ".zip");
    assert_eq!(EType::Mobi.suffix(), ".mobi");
    assert_eq!(EType::Pdf.suffix(), ".pdf");
}
```

**Testing URL ID generation:**

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);
}

#[test]
fn test_generate_url_id_different_source_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert!(id1 != id2);
}

#[test]
fn test_generate_url_id_length() {
    let id = generate_url_id(42, "abc123");
    assert_eq!(id.len(), 12);
}
```

**Testing cache path computation:**

```rust
#[test]
fn test_cache_path_short_url_id() {
    let path = cache_path(Path::new("/cache"), &EType::Epub, "abc", "hash123");
    assert_eq!(path, Path::new("/cache/epub/abc/abc/hash123.epub"));
}

#[test]
fn test_cache_path_medium_url_id() {
    let path = cache_path(Path::new("/cache"), &EType::Html, "abcdef", "h");
    assert_eq!(path, Path::new("/cache/html/abc/def/abcdef/h.zip"));
}

#[test]
fn test_cache_path_long_url_id() {
    let path = cache_path(Path::new("/cache"), &EType::Mobi, "abcdefghijklm", "h1");
    assert_eq!(path, Path::new("/cache/mobi/abc/def/ghi/abcdefghijklm/h1.mobi"));
}
```

**Testing token bucket logic:**

```rust
struct TokenBucket {
    value: f64,
    last_drain: f64,
    capacity: f64,
    flow: f64,
}

impl TokenBucket {
    fn new(capacity: f64, flow: f64, now: f64) -> Self {
        TokenBucket { value: capacity, last_drain: now, capacity, flow }
    }
    
    fn request(&mut self, requested: f64, now: f64) -> f64 {
        let elapsed = now - self.last_drain;
        let new_tokens = (self.value + elapsed * self.flow).min(self.capacity);
        let allowed = new_tokens - requested;
        if allowed >= 0.0 {
            self.value = allowed;
            self.last_drain = now;
            -1.0
        } else {
            (requested - new_tokens) / self.flow
        }
    }
}

#[test]
fn test_initial_bucket_fill() {
    let mut bucket = TokenBucket::new(100.0, 10.0, 0.0);
    let result = bucket.request(10.0, 0.0);
    assert!((result - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 90.0).abs() < f64::EPSILON);
}

#[test]
fn test_token_refill_over_time() {
    let mut bucket = TokenBucket::new(50.0, 10.0, 0.0);
    bucket.request(50.0, 0.0); // Empty the bucket
    let r = bucket.request(20.0, 3.0); // After 3s: 30 tokens refilled
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 10.0).abs() < f64::EPSILON);
}
```

**Testing tag parsing:**

```rust
#[test]
fn test_parse_tag_filters_empty() {
    let result = parse_tag_filters("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_tag_filters_multiple() {
    let result = parse_tag_filters("1:Harry Potter,2:Hermione Granger,4:Angst").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].tag_type_id, 1);
    assert_eq!(result[1].tag_name, "Hermione Granger");
}

#[test]
fn test_parse_tag_filters_invalid_format() {
    assert!(parse_tag_filters("invalid").is_err());
}
```

### Testing Export Functions

```rust
#[test]
fn test_compute_version_all_zero() {
    assert_eq!(compute_version(0, 0, 0), 0);
}

#[test]
fn test_compute_version_mixed() {
    assert_eq!(compute_version(5, 0, 3), 8);
    assert_eq!(compute_version(0, 5, 3), 8);
}

#[test]
fn test_etype_versions_map() {
    let versions = etype_versions();
    assert_eq!(versions.len(), 4);
    assert_eq!(*versions.get(ETYPE_EPUB).unwrap(), 1);
    assert_eq!(*versions.get(ETYPE_MOBI).unwrap(), 0);
}
```

### Testing Export Response Building

```rust
fn make_test_meta() -> FicMetadata {
    FicMetadata {
        url_id: "test123".into(),
        title: "Test Fic".into(),
        author: "Test Author".into(),
        chapters: 10,
        words: 50000,
        desc: "<p>A great story</p>".into(),
        published: 1700000000000,
        updated: 1700000000000,
        status: "complete".into(),
        source: "https://archiveofourown.org/works/123456".into(),
        source_id: 1,
        author_id: 42,
        author_url: "https://archiveofourown.org/users/TestAuthor".into(),
        author_local_id: "123456".into(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    }
}

#[test]
fn test_generate_slug_basic() {
    let slug = generate_slug("The Best Story", "abc123");
    assert!(slug.contains("The_Best_Story"));
    assert!(slug.contains("abc123"));
}

#[test]
fn test_build_info_string() {
    let meta = make_test_meta();
    let (info, notes) = build_info_string(&meta);
    assert!(info.contains("Test Fic"));
    assert!(info.contains("Test Author"));
    assert!(info.contains("50000"));
}
```

### Testing Visibility Logic

```rust
#[test]
fn test_is_hidden_above_threshold() {
    assert!(!is_hidden(0, -3));
    assert!(!is_hidden(-2, -3));
    assert!(!is_hidden(10, -3));
}

#[test]
fn test_is_hidden_at_or_below_threshold() {
    assert!(is_hidden(-3, -3));
    assert!(is_hidden(-5, -3));
    assert!(is_hidden(0, 0));
}

#[test]
fn test_compute_visibility() {
    let scores = vec![(1, 5), (2, 0), (3, -3), (4, -10)];
    let result = compute_visibility(&scores, -3);
    assert!(!result[0].1); // visible
    assert!(!result[1].1); // visible
    assert!(result[2].1);  // hidden
    assert!(result[3].1);  // hidden
}
```

### Testing MD5 Computation

```rust
#[test]
fn test_file_md5_known_content() {
    let dir = std::env::temp_dir().join("fichub_test_disk_md5");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test.txt");
    std::fs::write(&file_path, b"hello world").unwrap();
    let result = file_md5(&file_path).unwrap();
    assert_eq!(result, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_file_md5_empty_file() {
    let dir = std::env::temp_dir().join("fichub_test_disk_empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("empty.txt");
    std::fs::write(&file_path, b"").unwrap();
    let result = file_md5(&file_path).unwrap();
    assert_eq!(result, "d41d8cd98f00b204e9800998ecf8427e");
    let _ = std::fs::remove_dir_all(&dir);
}
```

### Testing Serialization

```rust
#[test]
fn test_rec_query_serialization() {
    let q = RecQuery {
        url_id: "abc123".into(),
        n: 10,
        site_domain: Some("archiveofourown.org".into()),
    };
    let json = serde_json::to_string(&q).unwrap();
    assert!(json.contains("abc123"));
    assert!(json.contains("archiveofourown.org"));
}

#[test]
fn test_rec_result_serialization() {
    let r = RecResult {
        url_id: "def456".into(),
        title: "Test Fic".into(),
        author: "Test Author".into(),
        words: 50000,
        chapters: 10,
        status: "complete".into(),
        site_domain: "archiveofourown.org".into(),
        summary: "A test fic".into(),
        score: 0.85,
        community_score: 3,
        download_urls: HashMap::new(),
    };
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("0.85"));
}
```

### Best Practices

1. **Use `#[cfg(test)]` modules** — Tests live alongside the code they test
2. **Test edge cases** — Empty strings, zero values, boundary conditions
3. **Test error paths** — Not just the happy path
4. **Isolate tests** — Use env guards, temp directories, and cleanup
5. **Make tests deterministic** — No random values, no time-dependent logic
6. **Test Display and serialization** — These are public interfaces

---

## Chapter 35: Production Hardening

Running in production requires more than just working code. Let's cover TLS, logging, monitoring, backups, and security.

### TLS

FicHub doesn't handle TLS directly — it uses a reverse proxy (Nginx, Caddy, or Traefik) for TLS termination:

```nginx
server {
    listen 443 ssl http2;
    server_name fichub.example.com;
    
    ssl_certificate /etc/letsencrypt/live/fichub.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fichub.example.com/privkey.pem;
    
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Structured Logging

FicHub uses `tracing` for structured logging:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
    )
    .init();
```

Log levels:
- `ERROR` — Something broke (database errors, export failures)
- `WARN` — Something unexpected (rate limits, blacklist hits)
- `INFO` — Normal operation (startup, migrations, request counts)
- `DEBUG` — Detailed info (scrape results, cache hits/misses)

In production, set `RUST_LOG=info,fichub=debug` for a good balance.

### Environment Variables

All configuration is via environment variables — no config files to manage:

```bash
# Required
DATABASE_URL=postgres://fichub:password@host:5432/fichub
REDIS_URL=redis://host:6379

# Optional (with defaults)
CACHE_DIR=/var/cache/fichub
TMP_DIR=/tmp/fichub
PORT=3000
RUST_LOG=info,fichub=debug
DYNAMIC_RATE_LIMIT=true
```

### Database Backups

PostgreSQL backups with `pg_dump`:

```bash
# Daily backup
pg_dump -h localhost -U fichub fichub | gzip > /backup/fichub_$(date +%Y%m%d).sql.gz

# Restore
gunzip -c /backup/fichub_20240101.sql.gz | psql -h localhost -U fichub fichub
```

For automated backups, add a cron job:

```bash
0 2 * * * pg_dump -h localhost -U fichub fichub | gzip > /backup/fichub_$(date +\%Y\%m\%d).sql.gz
```

### Log Rotation

```bash
# /etc/logrotate.d/fichub
/var/log/fichub/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 fichub fichub
    sharedscripts
    postrotate
        systemctl reload fichub
    endscript
}
```

### Security Considerations

1. **Parameterized queries** — Never interpolate user input into SQL
2. **Rate limiting** — Prevent abuse and DDoS
3. **Input validation** — Validate all user input at the handler level
4. **Error messages** — Don't leak internal details to clients
5. **Curator tokens** — Protect admin endpoints with bearer tokens
6. **IP-based limits** — Prevent automated abuse
7. **CORS** — Restrict in production to your frontend domain
8. **Updates** — Keep dependencies updated (`cargo update`)

### Health Monitoring

```bash
# Check if the server is responding
curl -sf http://localhost:3000/api/ > /dev/null || systemctl restart fichub

# Check database connectivity
psql -h localhost -U fichub -c "SELECT 1" fichub > /dev/null || echo "DB down"

# Check Redis connectivity
redis-cli ping | grep -q PONG || echo "Redis down"
```

### Resource Limits

For systemd, set resource limits:

```ini
[Service]
MemoryMax=512M
CPUQuota=80%
LimitNOFILE=65536
```

---

## Chapter 36: What's Next?

Congratulations! You've built a complete, production-ready backend server in Rust. Let's recap what we've accomplished and look at where to go from here.

### What We Built

Over 36 chapters, we built:

1. **A Rust web server** with Axum — handling HTTP requests, routing, and middleware
2. **A PostgreSQL database layer** with SQLx — compile-time checked queries, migrations, and connection pooling
3. **Web scrapers** for 6 fanfiction sites — AO3, FF.net, XenForo, FictionPress, AdultFanFiction, HPFanFic
4. **EPUB generation** — Proper e-books with metadata, chapters, and table of contents
5. **HTML bundles** — Single-page reading experiences in ZIP archives
6. **A disk cache** — Hash-based directory structure with semaphore deduplication
7. **Rate limiting** — Token bucket algorithm with Redis-backed state
8. **A recommendation engine** — Collaborative filtering with Jaccard coefficient and tag fallback
9. **Community features** — Tagging, voting, suggestions, and curation
10. **Full-text search** — PostgreSQL FTS with dynamic query building
11. **OPDS catalog** — E-reader compatible feeds with shelves and recommendations
12. **Docker deployment** — Multi-stage Dockerfile and docker-compose stack
13. **Systemd service** — Production-grade process management
14. **Comprehensive tests** — Unit tests for every major component

### What We Learned

Beyond the technical implementation, we learned:

- **Rust's ownership model** — How it prevents entire categories of bugs
- **Async Rust** — Tokio, futures, and the async/await pattern
- **Web architecture** — HTTP, REST APIs, JSON, and state management
- **Database design** — Schema design, migrations, and query optimization
- **Web scraping** — HTML parsing, CSS selectors, and ethical scraping
- **Caching strategies** — Two-layer caching, deduplication, and cache invalidation
- **Rate limiting** — Token bucket algorithms and distributed state with Redis
- **Recommendation systems** — Collaborative filtering, co-occurrence, and scoring
- **Deployment** — Docker, systemd, reverse proxies, and monitoring

### Ideas for Extension

There's always more to build. Here are some ideas:

**New Scrapers:**
- Wattpad
- Quotev
- LiveJournal
- Dreamwidth

**New Export Formats:**
- CBZ (comic book archive)
- Plain text (for terminal reading)
- Markdown

**Enhanced Recommendations:**
- Neural collaborative filtering
- Content-based filtering (using story descriptions)
- Author similarity graphs

**New Features:**
- User accounts and reading lists
- Reading progress tracking
- Annotation and highlighting
- Story statistics and analytics

**Infrastructure:**
- Kubernetes deployment
- Prometheus + Grafana monitoring
- Elasticsearch for advanced search
- Message queue (RabbitMQ/Kafka) for background jobs

### Learning Resources

**Rust:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/) — The official book
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — Learn by doing
- [Async Book](https://rust-lang.github.io/async-book/) — Deep dive into async Rust

**Web Development:**
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples) — Official examples
- [SQLx Documentation](https://docs.rs/sqlx) — Database queries
- [Tower](https://docs.rs/tower) — Middleware framework

**Databases:**
- [PostgreSQL Tutorial](https://www.postgresqltutorial.com/) — SQL fundamentals
- [Use The Index, Luke](https://use-the-index-luke.com/) — Query optimization

**Deployment:**
- [Docker Documentation](https://docs.docker.com/) — Container deployment
- [Systemd Documentation](https://www.freedesktop.org/software/systemd/man/) — Service management

### Contributing

FicHub is an open-source project. If you want to contribute:

1. **Fork the repository**
2. **Create a feature branch** — `git checkout -b feature/my-feature`
3. **Write tests** — Every change should include tests
4. **Follow conventions** — Use the existing code style
5. **Submit a PR** — Describe what you changed and why

### Final Thoughts

Building FicHub taught me that Rust isn't just a systems language — it's a fantastic choice for web backends. The compiler catches bugs that would be runtime errors in other languages. The async ecosystem is mature and performant. And the package ecosystem (crates.io) has high-quality libraries for everything we needed.

The best part? When FicHub compiles and the tests pass, you can be confident it works correctly. That confidence is worth the learning curve.

Thank you for reading this book. Now go build something amazing.

Happy coding! 🦀


---

