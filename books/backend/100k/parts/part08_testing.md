# Part 8: Testing and Production

---

# Chapter 34: Backend Testing

## The Testing Pyramid

Testing is essential for maintaining a reliable backend. FicHub follows the testing pyramid: many unit tests, fewer integration tests, and a handful of end-to-end tests.

## Unit Tests with #[cfg(test)]

FicHub embeds unit tests directly in source files:

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
    fn test_generate_url_id_different_source_id() {
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

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test config::tests

# Specific test
cargo test test_from_env_defaults

# With output
cargo test -- --nocapture

# By name pattern
cargo test url_id
```

## Assert Macros

```rust
// Basic assertions
assert_eq!(result, expected);
assert_ne!(result, unexpected);
assert!(condition);

// With messages
assert_eq!(result, expected, "got {:?}", result);

// Expected panics
#[test]
#[should_panic(expected = "DATABASE_URL must be set")]
fn test_panics_without_database_url() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("REDIS_URL", "redis://localhost");
    let _ = Config::from_env();
}
```

## Test Helpers

### EnvGuard for Environment Variables

```rust
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    keys: Vec<String>,
}

impl EnvGuard {
    fn new() -> Self {
        EnvGuard { keys: Vec::new() }
    }

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
```

### clear_config_env

```rust
fn clear_config_env() {
    let keys = [
        "DATABASE_URL", "REDIS_URL", "CACHE_DIR", "SECONDARY_CACHE_DIR",
        "EXPORT_VERSION", "DYNAMIC_RATE_LIMIT", "NODE_NAME", "CALIBRE_CONTAINER",
        "TMP_DIR", "PORT", "FRONTEND_DIR", "TRUSTED_PROXIES", "IP_TAG_SOURCES",
        "REC_DEFAULT_DELAY_SECS", "REC_SITE_RATE_LIMITS", "REC_MAX_FAVOURITE_PAGES",
        "REC_MAX_USER_FAVOURITE_PAGES", "REC_MAX_RECOMMENDATIONS",
        "REC_MIN_FAVOURITERS_FOR_COLLAB", "REC_VOTING_BOOST_GAMMA",
        "REC_CACHE_TTL_HOURS", "REC_SUGGEST_LIMIT_PER_HOUR", "REC_VOTE_LIMIT_PER_HOUR",
        "REC_PRECOMPUTE_ENABLED", "REC_PRECOMPUTE_INTERVALHours", "REC_ENABLE_CROSS_SITE",
    ];
    for key in &keys {
        unsafe { std::env::remove_var(key); }
    }
}
```

## Testing Configuration

### Default Values

```rust
#[test]
fn test_from_env_defaults() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test_db");
    guard.set("REDIS_URL", "redis://localhost/0");

    let config = Config::from_env();

    assert_eq!(config.database_url, "postgres://localhost/test_db");
    assert_eq!(config.cache_dir, PathBuf::from("./cache"));
    assert!(config.secondary_cache_dir.is_none());
    assert_eq!(config.export_version, 1);
    assert!(config.dynamic_rate_limit);
    assert_eq!(config.node_name, "orion");
    assert_eq!(config.app_port, 3000);
    assert_eq!(config.frontend_dir, PathBuf::from("./frontend/build"));
    assert!(config.trusted_proxies.is_empty());
    assert_eq!(config.rec_default_delay_secs, 5);
    assert_eq!(config.rec_max_favourite_pages, 3);
    assert_eq!(config.rec_max_recommendations, 20);
    assert_eq!(config.rec_min_favouriters_for_collab, 5);
    assert!((config.rec_voting_boost_gamma - 0.2).abs() < f64::EPSILON);
    assert_eq!(config.rec_cache_ttl_hours, 12);
    assert!(config.rec_precompute_enabled);
    assert!(config.rec_enable_cross_site);
}
```

### Custom Values

```rust
#[test]
fn test_from_env_custom_values() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://custom/db");
    guard.set("REDIS_URL", "redis://custom");
    guard.set("CACHE_DIR", "/alt/cache");
    guard.set("PORT", "9090");
    guard.set("FRONTEND_DIR", "/alt/frontend");
    guard.set("TRUSTED_PROXIES", "10.0.0.1, 10.0.0.2");
    guard.set("REC_MAX_RECOMMENDATIONS", "100");
    guard.set("REC_VOTING_BOOST_GAMMA", "0.8");

    let config = Config::from_env();

    assert_eq!(config.database_url, "postgres://custom/db");
    assert_eq!(config.cache_dir, PathBuf::from("/alt/cache"));
    assert_eq!(config.app_port, 9090);
    assert_eq!(config.trusted_proxies, vec!["10.0.0.1".into(), "10.0.0.2".into()]);
    assert_eq!(config.rec_max_recommendations, 100);
    assert!((config.rec_voting_boost_gamma - 0.8).abs() < f64::EPSILON);
}
```

### Panics on Missing Required Config

```rust
#[test]
#[should_panic(expected = "DATABASE_URL must be set")]
fn test_panics_without_database_url() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("REDIS_URL", "redis://localhost");
    let _ = Config::from_env();
}

#[test]
#[should_panic(expected = "REDIS_URL must be set")]
fn test_panics_without_redis_url() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test");
    let _ = Config::from_env();
}
```

### JSON Parsing

```rust
#[test]
fn test_rec_site_rate_limits_valid_json() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test");
    guard.set("REDIS_URL", "redis://localhost");
    guard.set("REC_SITE_RATE_LIMITS", r#"{"ao3": 10, "ffn": 5}"#);

    let config = Config::from_env();
    let mut expected = HashMap::new();
    expected.insert("ao3".to_string(), 10);
    expected.insert("ffn".to_string(), 5);
    assert_eq!(config.rec_site_rate_limits, expected);
}

#[test]
fn test_rec_site_rate_limits_invalid_json() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test");
    guard.set("REDIS_URL", "redis://localhost");
    guard.set("REC_SITE_RATE_LIMITS", "not valid json");

    let config = Config::from_env();
    assert!(config.rec_site_rate_limits.is_empty());
}
```

## Testing Error Types

### Display Implementation

```rust
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

#[test]
fn test_display_not_found() {
    let err = AppError::NotFound("story".into());
    let s = format!("{}", err);
    assert!(s.contains("NotFound"));
    assert!(s.contains("story"));
}

#[test]
fn test_display_internal() {
    let err = AppError::Internal("oops".into());
    let s = format!("{}", err);
    assert!(s.contains("Internal"));
    assert!(s.contains("oops"));
}

#[test]
fn test_display_scrape_error() {
    let err = AppError::ScrapeError("timeout".into());
    let s = format!("{}", err);
    assert!(s.contains("ScrapeError"));
    assert!(s.contains("timeout"));
}

#[test]
fn test_display_export_error() {
    let err = AppError::ExportError("epub fail".into());
    let s = format!("{}", err);
    assert!(s.contains("ExportError"));
    assert!(s.contains("epub fail"));
}

#[test]
fn test_display_database() {
    let err = AppError::Database("conn lost".into());
    let s = format!("{}", err);
    assert!(s.contains("Database"));
    assert!(s.contains("conn lost"));
}

#[test]
fn test_display_cache_error() {
    let err = AppError::CacheError("cache miss".into());
    let s = format!("{}", err);
    assert!(s.contains("CacheError"));
    assert!(s.contains("cache miss"));
}
```

### From Implementations

```rust
#[test]
fn test_from_io_error() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app: AppError = io.into();
    match app {
        AppError::Internal(msg) => assert!(msg.contains("file not found")),
        _ => panic!("expected Internal, got {:?}", app),
    }
}

#[test]
fn test_from_sqlx_error() {
    let sqlx = sqlx::Error::Protocol("bad query".into());
    let app: AppError = sqlx.into();
    match app {
        AppError::Database(msg) => assert!(msg.contains("bad query")),
        _ => panic!("expected Database, got {:?}", app),
    }
}

#[test]
fn test_from_redis_error() {
    let err = redis::RedisError::from((redis::ErrorKind::Io, "connection refused"));
    let app: AppError = err.into();
    match app {
        AppError::CacheError(msg) => assert!(msg.contains("connection refused")),
        _ => panic!("expected CacheError, got {:?}", app),
    }
}

#[test]
fn test_from_serde_json_error() {
    let json: Result<serde_json::Value, _> = serde_json::from_str("!invalid");
    let json_err = json.unwrap_err();
    let app: AppError = json_err.into();
    match app {
        AppError::Internal(_) => {}
        _ => panic!("expected Internal, got {:?}", app),
    }
}
```

## Testing Token Buckets

FicHub has a pure-Rust token bucket implementation for testing:

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

    fn penalize(&mut self, now: f64) {
        self.request(1.5, now);
    }
}

#[test]
fn test_initial_bucket_fill() {
    let mut bucket = TokenBucket::new(100.0, 10.0, 0.0);
    assert!((bucket.value - 100.0).abs() < f64::EPSILON);
    let result = bucket.request(10.0, 0.0);
    assert!((result - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 90.0).abs() < f64::EPSILON);
}

#[test]
fn test_token_refill_over_time() {
    let mut bucket = TokenBucket::new(50.0, 10.0, 0.0);
    bucket.request(50.0, 0.0);  // Empty
    assert!((bucket.value - 0.0).abs() < f64::EPSILON);

    let r = bucket.request(20.0, 3.0);  // After 3s, 30 refilled
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_capacity_limits_refill() {
    let mut bucket = TokenBucket::new(50.0, 100.0, 0.0);
    bucket.request(50.0, 0.0);
    // After 10s: 1000 tokens, but capped at 50
    let wait = bucket.request(60.0, 10.0);
    assert!(wait > 0.0);
    let expected = (60.0 - 50.0) / 100.0;
    assert!((wait - expected).abs() < f64::EPSILON);
}

#[test]
fn test_wait_calculation() {
    let mut bucket = TokenBucket::new(10.0, 2.0, 0.0);
    bucket.request(10.0, 0.0);
    let wait = bucket.request(5.0, 0.0);
    let expected = 5.0 / 2.0;
    assert!((wait - expected).abs() < f64::EPSILON);
}

#[test]
fn test_multiple_requests_over_time() {
    let mut bucket = TokenBucket::new(20.0, 5.0, 0.0);

    let r = bucket.request(10.0, 0.0);
    assert!((r - (-1.0)).abs() < f64::EPSILON);

    let r = bucket.request(15.0, 2.0);
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 5.0).abs() < f64::EPSILON);

    let r = bucket.request(20.0, 5.0);
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 0.0).abs() < f64::EPSILON);

    let wait = bucket.request(10.0, 6.0);
    let expected = (10.0 - 5.0) / 5.0;
    assert!((wait - expected).abs() < f64::EPSILON);
}

#[test]
fn test_penalize_reduces_tokens() {
    let mut bucket = TokenBucket::new(10.0, 5.0, 0.0);
    bucket.request(8.0, 0.0);
    let before = bucket.value;
    bucket.penalize(0.0);
    assert!((bucket.value - (before - 1.5)).abs() < f64::EPSILON);
}
```

## Testing Cache Paths

```rust
#[test]
fn test_cache_path_short_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Epub, "abc", "hash123");
    assert_eq!(path, Path::new("/cache/epub/abc/abc/hash123.epub"));
}

#[test]
fn test_cache_path_medium_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Html, "abcdef", "h");
    assert_eq!(path, Path::new("/cache/html/abc/def/abcdef/h.zip"));
}

#[test]
fn test_cache_path_long_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Mobi, "abcdefghijklm", "h1");
    assert_eq!(path, Path::new("/cache/mobi/abc/def/ghi/abcdefghijklm/h1.mobi"));
}

#[test]
fn test_cache_path_different_etypes() {
    let root = Path::new("/cache");
    let url_id = "abc";
    let hash = "h";
    assert_eq!(cache_path(root, &EType::Epub, url_id, hash), Path::new("/cache/epub/abc/abc/h.epub"));
    assert_eq!(cache_path(root, &EType::Html, url_id, hash), Path::new("/cache/html/abc/abc/h.zip"));
    assert_eq!(cache_path(root, &EType::Mobi, url_id, hash), Path::new("/cache/mobi/abc/abc/h.mobi"));
    assert_eq!(cache_path(root, &EType::Pdf, url_id, hash), Path::new("/cache/pdf/abc/abc/h.pdf"));
}
```

## Testing File MD5

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

## Testing Export Helpers

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
    let slug = super::generate_slug("The Best Story", "abc123");
    assert!(slug.contains("The_Best_Story"));
    assert!(slug.contains("abc123"));
    assert!(!slug.starts_with('_'));
    assert!(!slug.ends_with('_'));
}

#[test]
fn test_generate_slug_special_chars() {
    let slug = super::generate_slug("Hello: World? (Part 1/2)", "xyz789");
    assert!(!slug.contains(':'));
    assert!(!slug.contains('?'));
    assert!(!slug.contains('('));
    assert!(slug.contains("Hello_World_Part_1_2"));
}

#[test]
fn test_build_info_string() {
    let meta = make_test_meta();
    let (info, notes) = super::build_info_string(&meta);
    assert!(info.contains("Test Fic"));
    assert!(info.contains("Test Author"));
    assert!(info.contains("50000"));
    assert!(info.contains("10"));
    assert!(info.contains("complete"));
    assert!(notes.is_empty());
}

#[test]
fn test_build_meta_json() {
    let meta = make_test_meta();
    let json = super::build_meta_json(&meta);
    assert_eq!(json["id"], "test123");
    assert_eq!(json["title"], "Test Fic");
    assert_eq!(json["author"], "Test Author");
    assert_eq!(json["chapters"], 10);
    assert_eq!(json["words"], 50000);
    assert_eq!(json["status"], "complete");
    assert_eq!(json["source_id"], 1);
    assert_eq!(json["author_id"], 42);
}

#[test]
fn test_build_metadata_response_greylisted() {
    let meta = make_test_meta();
    let resp = super::build_metadata_response(&meta, &[], &1, None, true);
    let json = resp.0;
    assert_eq!(json["err"], 0);
    assert!(json["notes"][0].as_str().unwrap_or("").contains("greylisted"));
    assert!(json["urls"].as_object().unwrap().is_empty());
}
```

## Testing EType

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
fn test_etype_as_str() {
    assert_eq!(EType::Epub.as_str(), "epub");
    assert_eq!(EType::Html.as_str(), "html");
    assert_eq!(EType::Mobi.as_str(), "mobi");
    assert_eq!(EType::Pdf.as_str(), "pdf");
}

#[test]
fn test_etype_suffix() {
    assert_eq!(EType::Epub.suffix(), ".epub");
    assert_eq!(EType::Html.suffix(), ".zip");
    assert_eq!(EType::Mobi.suffix(), ".mobi");
    assert_eq!(EType::Pdf.suffix(), ".pdf");
}

#[test]
fn test_etype_version() {
    assert_eq!(EType::Epub.version(), 1);
    assert_eq!(EType::Html.version(), 1);
    assert_eq!(EType::Mobi.version(), 0);
    assert_eq!(EType::Pdf.version(), 0);
}
```

## Testing Export Version Computation

```rust
#[test]
fn test_compute_version_all_zero() {
    assert_eq!(compute_version(0, 0, 0), 0);
}

#[test]
fn test_compute_version_all_one() {
    assert_eq!(compute_version(1, 1, 1), 3);
}

#[test]
fn test_compute_version_mixed() {
    assert_eq!(compute_version(5, 0, 3), 8);
    assert_eq!(compute_version(0, 5, 3), 8);
    assert_eq!(compute_version(3, 5, 0), 8);
}
```

## Testing Search Filters

```rust
#[test]
fn test_parse_tag_filters_empty() {
    let result = parse_tag_filters("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_tag_filters_single() {
    let result = parse_tag_filters("1:Harry Potter").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].tag_type_id, 1);
    assert_eq!(result[0].tag_name, "Harry Potter");
}

#[test]
fn test_parse_tag_filters_multiple() {
    let result = parse_tag_filters("1:Harry Potter,2:Hermione Granger,4:Angst").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].tag_name, "Harry Potter");
    assert_eq!(result[1].tag_name, "Hermione Granger");
    assert_eq!(result[2].tag_name, "Angst");
}

#[test]
fn test_parse_tag_filters_invalid_format() {
    let result = parse_tag_filters("invalid");
    assert!(result.is_err());
}

#[test]
fn test_parse_tag_filters_trailing_comma() {
    let result = parse_tag_filters("1:test,").unwrap();
    assert_eq!(result.len(), 1);
}
```

## Testing Scraper Error Display

```rust
#[test]
fn test_scrape_error_display_not_found() {
    assert_eq!(format!("{}", ScrapeError::NotFound), "fic not found");
}

#[test]
fn test_scrape_error_display_blocked() {
    assert_eq!(format!("{}", ScrapeError::Blocked), "blocked by site");
}

#[test]
fn test_scrape_error_display_network() {
    let err = ScrapeError::Network("connection refused".into());
    assert_eq!(format!("{}", err), "network error: connection refused");
}

#[test]
fn test_scrape_error_display_parse_error() {
    let err = ScrapeError::ParseError("unexpected token".into());
    assert_eq!(format!("{}", err), "parse error: unexpected token");
}
```

## Testing Tag Resolution

```rust
#[test]
fn test_tag_resolution_struct_sizes() {
    assert_eq!(std::mem::size_of::<TagResolution>(), std::mem::size_of::<(i32, String, i16, bool)>());
}

#[test]
fn test_tag_resolution_new_tag() {
    let tag = TagResolution {
        tag_id: 1,
        tag_name: "Angst".into(),
        tag_type_id: 4,
        is_new: true,
    };
    assert_eq!(tag.tag_id, 1);
    assert_eq!(tag.tag_name, "Angst");
    assert!(tag.is_new);
}

#[test]
fn test_tag_resolution_existing_tag() {
    let tag = TagResolution {
        tag_id: 42,
        tag_name: "Fluff".into(),
        tag_type_id: 4,
        is_new: false,
    };
    assert!(!tag.is_new);
}
```

## Testing Vote Visibility

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
    assert!(is_hidden(-10, -3));
    assert!(is_hidden(0, 0));
}

#[test]
fn test_compute_visibility() {
    let scores = vec![(1, 5), (2, 0), (3, -3), (4, -10)];
    let result = compute_visibility(&scores, -3);
    assert_eq!(result.len(), 4);
    assert!(!result[0].1); // score=5 > -3
    assert!(!result[1].1); // score=0 > -3
    assert!(result[2].1);  // score=-3 == -3
    assert!(result[3].1);  // score=-10 < -3
}
```

## Watch Out!

**Tests must be deterministic!** Avoid current time, random numbers, or external services.

**Parallel test execution!** Rust runs tests in parallel. Use mutexes for shared resources.

**Clean up test data!** Tests that modify the database should clean up.

## Summary

FicHub has comprehensive unit tests covering configuration, error handling, cache paths, token buckets, search filters, tag resolution, vote visibility, and export helpers. The `EnvGuard` helper ensures clean environment variable management.
