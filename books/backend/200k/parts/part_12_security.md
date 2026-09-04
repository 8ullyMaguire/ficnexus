# Part 12: Security Hardening

---

# Chapter 58: Input Validation and Sanitization

Input validation is the first line of defense against attacks. This chapter covers how to validate and sanitize user input in FicHub.

## Why Validate Input?

Every piece of data that enters your application from an external source is potentially dangerous:
- HTTP request parameters
- Database query results
- File contents from disk
- Data from external APIs

**Real-world analogy:** Input validation is like airport security. Every passenger (data) must go through security screening (validation) before boarding the plane (entering your application). The screening checks for weapons (malicious input) and ensures everyone has a valid ticket (correct format).

## URL Validation

```rust
use url::Url;

fn validate_export_url(url: &str) -> Result<Url, AppError> {
    // Check URL length
    if url.len() > 2048 {
        return Err(AppError::BadRequest(-1, "URL too long".into()));
    }
    
    // Parse URL
    let parsed = Url::parse(url)
        .map_err(|_| AppError::BadRequest(-1, "invalid URL format".into()))?;
    
    // Check scheme
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(AppError::BadRequest(-1, "URL must use http or https".into()));
    }
    
    // Check domain
    let host = parsed.host_str()
        .ok_or_else(|| AppError::BadRequest(-1, "URL must have a host".into()))?;
    
    let allowed_hosts = [
        "archiveofourown.org",
        "fanfiction.net",
        "fictionpress.com",
        "forums.spacebattles.com",
        "forums.sufficientvelocity.com",
        "forum.questionablequesting.com",
    ];
    
    if !allowed_hosts.iter().any(|h| host.ends_with(h)) {
        return Err(AppError::BadRequest(-1, "unsupported URL domain".into()));
    }
    
    Ok(parsed)
}
```

## String Sanitization

```rust
fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect::<String>()
        .trim()
        .to_string()
}

fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
```

## Numeric Validation

```rust
fn validate_port(port: u16) -> Result<u16, AppError> {
    if port == 0 {
        return Err(AppError::BadRequest(-1, "port cannot be 0".into()));
    }
    Ok(port)
}

fn validate_word_count(words: i64) -> Result<i64, AppError> {
    if words < 0 {
        return Err(AppError::BadRequest(-1, "word count cannot be negative".into()));
    }
    if words > 10_000_000 {
        return Err(AppError::BadRequest(-1, "word count too large".into()));
    }
    Ok(words)
}
```

## Request Body Validation

```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct CreateTagRequest {
    #[validate(length(min = 1, max = 200))]
    name: String,
    
    #[validate(range(min = 1, max = 6))]
    tag_type_id: i16,
}

async fn submit_tag(
    Json(payload): Json<CreateTagRequest>,
) -> Result<Json<Value>, AppError> {
    // Validate using validator crate
    payload.validate()
        .map_err(|e| AppError::BadRequest(-1, e.to_string()))?;
    
    // Process valid input
    // ...
}
```

## 📝 Practice Exercises

1. **URL Validation:** Write a function that validates URLs for all supported fanfiction sites.

2. **Input Sanitization:** Create a sanitization function that handles all types of malicious input.

3. **Request Validation:** Add validation to the tag submission endpoint.

---

# Chapter 59: SQL Injection Prevention

SQL injection is one of the most common and dangerous web vulnerabilities. This chapter covers how to prevent it.

## What Is SQL Injection?

SQL injection occurs when user input is concatenated into SQL queries without proper escaping:

```rust
// DANGEROUS: SQL injection vulnerability
let query = format!("SELECT * FROM fic_info WHERE title LIKE '%{}%'", user_input);
// If user_input is: '; DROP TABLE fic_info; --
// The query becomes: SELECT * FROM fic_info WHERE title LIKE '%'; DROP TABLE fic_info; --%'
```

## Prevention with Parameterized Queries

Always use parameterized queries:

```rust
// SAFE: Parameterized query
let query = "SELECT * FROM fic_info WHERE title LIKE $1";
let pattern = format!("%{}%", user_input);
sqlx::query(query)
    .bind(&pattern)
    .fetch_all(&pool)
    .await?;
```

SQLx handles all escaping and type conversion automatically.

## Dynamic Query Building

For dynamic queries, use `QueryBuilder`:

```rust
use sqlx::QueryBuilder;

fn build_search_query(params: &SearchParams) -> QueryBuilder<Postgres> {
    let mut qb = QueryBuilder::new("SELECT * FROM fic_info WHERE 1=1");
    
    if let Some(ref q) = params.q {
        qb.push(" AND to_tsvector('english', title) @@ plainto_tsquery('english', ");
        qb.push_bind(q.clone());
        qb.push(")");
    }
    
    if let Some(min_words) = params.min_words {
        qb.push(" AND words >= ");
        qb.push_bind(min_words);
    }
    
    qb
}
```

`QueryBuilder` ensures that all values are properly parameterized.

## Stored Procedures

For complex operations, use stored procedures:

```sql
-- Create a stored procedure
CREATE OR REPLACE FUNCTION search_fics(
    search_query TEXT,
    min_words BIGINT DEFAULT 0
)
RETURNS TABLE(id VARCHAR, title TEXT, author TEXT) AS $$
BEGIN
    RETURN QUERY
    SELECT fi.id, fi.title, fi.author
    FROM fic_info fi
    WHERE to_tsvector('english', fi.title) @@ plainto_tsquery('english', search_query)
    AND fi.words >= min_words;
END;
$$ LANGUAGE plpgsql;
```

## Common SQL Injection Patterns

### String Concatenation

```rust
// DANGEROUS
let query = format!("SELECT * FROM users WHERE name = '{}'", name);

// SAFE
let query = "SELECT * FROM users WHERE name = $1";
```

### LIKE Clauses

```rust
// DANGEROUS
let query = format!("SELECT * FROM fic_info WHERE title LIKE '%{}%'", search);

// SAFE
let query = "SELECT * FROM fic_info WHERE title LIKE $1";
let pattern = format!("%{}%", search.replace('%', "\\%").replace('_', "\\_"));
```

### ORDER BY

```rust
// DANGEROUS: Can't parameterize ORDER BY
let query = format!("SELECT * FROM fic_info ORDER BY {}", sort_column);

// SAFE: Whitelist allowed columns
let allowed_columns = ["title", "words", "chapters", "fic_updated"];
let column = if allowed_columns.contains(&sort_column) {
    sort_column
} else {
    "fic_updated"
};
let query = format!("SELECT * FROM fic_info ORDER BY {} DESC", column);
```

## 📝 Practice Exercises

1. **Injection Test:** Try to perform SQL injection on a test endpoint. What happens?

2. **Parameterized Queries:** Convert 5 string-formatted queries to parameterized queries.

3. **Dynamic Queries:** Use `QueryBuilder` to build a search query with multiple optional filters.

---

# Chapter 60: Rate Limiting Security

Rate limiting prevents abuse, but it must be implemented securely.

## IP-Based Rate Limiting

```rust
async fn check_ip_rate_limit(
    redis: &mut MultiplexedConnection,
    ip: IpAddr,
    max_requests: u32,
    window_seconds: u64,
) -> Result<bool, AppError> {
    let key = format!("rate_limit:ip:{}", ip);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Use Redis pipeline for atomicity
    let mut pipe = redis::pipe();
    pipe.cmd("ZREMRANGEBYSCORE")
        .arg(&key)
        .arg(0)
        .arg(now - window_seconds)
        .ignore()
        .cmd("ZADD")
        .arg(&key)
        .arg(now)
        .arg(now)
        .ignore()
        .cmd("ZCARD")
        .arg(&key)
        .ignore()
        .cmd("EXPIRE")
        .arg(&key)
        .arg(window_seconds)
        .ignore();
    
    let results: ((), (), i64, ()) = pipe.query_async(redis).await?;
    let count = results.2;
    
    Ok(count <= max_requests as i64)
}
```

## Preventing Rate Limit Bypass

### IP Spoofing

When behind a proxy, trust only `X-Forwarded-For` from known proxies:

```rust
fn get_real_ip(headers: &HeaderMap, trusted_proxies: &[String]) -> IpAddr {
    if let Some(forwarded) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            let ips: Vec<&str> = forwarded_str.split(',').collect();
            // The leftmost IP is the client IP
            if let Some(client_ip) = ips.first() {
                if let Ok(ip) = client_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }
    
    // Fallback to connection info
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}
```

### API Key Abuse

For authenticated endpoints, rate limit by API key instead of IP:

```rust
async fn check_api_key_rate_limit(
    redis: &mut MultiplexedConnection,
    api_key: &str,
    max_requests: u32,
) -> Result<bool, AppError> {
    let key = format!("rate_limit:api:{}", api_key);
    // ... similar implementation ...
}
```

## Rate Limit Headers

Include rate limit information in responses:

```rust
fn rate_limit_headers(remaining: i64, limit: i64, reset_at: i64) -> Vec<(String, String)> {
    vec![
        ("X-RateLimit-Limit".into(), limit.to_string()),
        ("X-RateLimit-Remaining".into(), remaining.to_string()),
        ("X-RateLimit-Reset".into(), reset_at.to_string()),
    ]
}
```

## 📝 Practice Exercises

1. **Rate Limit Testing:** Test rate limiting with and without proxy headers.

2. **IP Spoofing Prevention:** Implement IP validation that checks trusted proxy lists.

3. **Rate Limit Headers:** Add rate limit headers to all API responses.

---

# Chapter 61: CORS and Headers Security

CORS (Cross-Origin Resource Sharing) controls which websites can access your API.

## CORS Configuration

```rust
use tower_http::cors::{CorsLayer, Any, Origin};

// Permissive (development)
let cors = CorsLayer::permissive();

// Restrictive (production)
let cors = CorsLayer::new()
    .allow_origin(Origin::list(vec![
        "https://fichub.net".parse().unwrap(),
        "https://www.fichub.net".parse().unwrap(),
    ]))
    .allow_methods(Any)
    .allow_headers(Any);
```

## Security Headers

```rust
use tower_http::set_header::SetResponseHeaderLayer;

let security_headers = ServiceBuilder::new()
    .layer(SetResponseHeaderLayer::overriding(
        http::header::X_CONTENT_TYPE_OPTIONS,
        http::HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
        http::header::X_FRAME_OPTIONS,
        http::HeaderValue::from_static("DENY"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
        http::header::X_XSS_PROTECTION,
        http::HeaderValue::from_static("1; mode=block"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
        http::header::CONTENT_SECURITY_POLICY,
        http::HeaderValue::from_static("default-src 'self'"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
        http::header::STRICT_TRANSPORT_SECURITY,
        http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    ));
```

## Content Security Policy

```rust
let csp = "default-src 'self'; \
           script-src 'self' 'unsafe-inline'; \
           style-src 'self' 'unsafe-inline'; \
           img-src 'self' data: https:; \
           font-src 'self'; \
           connect-src 'self' https://fichub.net; \
           frame-ancestors 'none';";
```

## 📝 Practice Exercises

1. **CORS Testing:** Test CORS configuration with different origins.

2. **Security Headers:** Add all recommended security headers to FicHub.

3. **CSP Testing:** Verify that Content Security Policy prevents XSS attacks.

---

# Chapter 62: Secrets Management

Never hardcode secrets in source code. This chapter covers secure secret management.

## Environment Variables

```rust
// Load from .env file
dotenvy::dotenv().ok();

// Read secrets
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

## Secret Storage

### Development

Use `.env` files (never commit to git):

```bash
# .env (add to .gitignore)
DATABASE_URL=postgres://user:password@localhost/fichub
REDIS_URL=redis://localhost:6379
CURATOR_TOKEN=my-secret-token
```

### Production

Use a secrets manager:
- **AWS Secrets Manager**
- **HashiCorp Vault**
- **Kubernetes Secrets**
- **Docker Secrets**

## Secret Rotation

```rust
// Periodically reload secrets
async fn reload_secrets(config: &Config) -> Result<(), AppError> {
    // Re-read environment variables
    let new_database_url = std::env::var("DATABASE_URL")
        .map_err(|_| AppError::Internal("DATABASE_URL not set".into()))?;
    
    // Create new connection pool
    let new_pool = PgPool::connect(&new_database_url).await?;
    
    // Swap pools (requires Arc swap)
    // ...
    
    Ok(())
}
```

## Logging Safety

Never log secrets:

```rust
// BAD: Logs the database URL with password
tracing::info!("Connecting to {}", database_url);

// SAFE: Mask the password
let safe_url = database_url.replace(
    &database_url[database_url.find('@').unwrap_or(0)..database_url.find('/').unwrap_or(database_url.len())],
    ":***@"
);
tracing::info!("Connecting to {}", safe_url);
```

## 📝 Practice Exercises

1. **Secret Rotation:** Implement automatic secret rotation for database credentials.

2. **Log Sanitization:** Write a function that sanitizes log messages to remove secrets.

3. **Environment Audit:** Audit FicHub's environment variables for secrets that should be rotated.

---

# Chapter 63: Dependency Auditing

Dependencies can introduce vulnerabilities. This chapter covers how to audit and manage dependencies.

## Cargo Audit

```bash
# Install cargo-audit
cargo install cargo-audit

# Audit dependencies
cargo audit

# Audit and fix
cargo audit fix
```

## Dependency Review

### Check for Known Vulnerabilities

```bash
# Using cargo-audit
cargo audit

# Using rustsec database
cargo search cargo-audit
```

### Review Dependency Quality

Before adding a dependency:
- Check download count (popular = more review)
- Check last update date (active maintenance)
- Check open issues (responsive maintainers)
- Check license (compatible with your project)

## Updating Dependencies

```bash
# Update all dependencies
cargo update

# Update specific dependency
cargo update -p serde

# Check for outdated dependencies
cargo install cargo-outdated
cargo outdated
```

## Dependency Minimization

```toml
# Instead of:
serde = { version = "1", features = ["full"] }

# Use only what you need:
serde = { version = "1", features = ["derive"] }
```

## Supply Chain Security

### Verify Checksums

Cargo.lock pins exact versions and includes checksums:

```bash
# Commit Cargo.lock to version control
git add Cargo.lock
```

### Use Verified Publishers

```toml
# Prefer well-known crates
axum = "0.8"           # Well-maintained by Tokio team
tokio = "1"            # Well-maintained by Tokio team
sqlx = "0.9"           # Well-maintained by launchbadge
```

## 📝 Practice Exercises

1. **Audit Run:** Run `cargo audit` on FicHub and fix any vulnerabilities.

2. **Dependency Review:** Review FicHub's dependencies for quality and maintenance.

3. **Minimal Features:** Audit feature flags and remove unnecessary ones.

---

# Chapter 64: Container Security

Docker containers need to be secured. This chapter covers container security best practices.

## Non-Root User

```dockerfile
# Create non-root user
RUN addgroup --system fichub && adduser --system --ingroup fichub fichub

# Switch to non-root user
USER fichub

# Run the application
CMD ["./fichub"]
```

## Read-Only Filesystem

```yaml
# docker-compose.yml
services:
  fichub:
    read_only: true
    tmpfs:
      - /tmp
    volumes:
      - fichub-cache:/cache
```

## Resource Limits

```yaml
services:
  fichub:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

## Network Security

```yaml
services:
  fichub:
    networks:
      - internal
  
  postgres:
    networks:
      - internal
  
  redis:
    networks:
      - internal

networks:
  internal:
    internal: true  # No external access
```

## Image Scanning

```bash
# Scan Docker image for vulnerabilities
docker scan fichub:latest

# Using Trivy
trivy image fichub:latest
```

## Security Best Practices

1. **Use minimal base images** — Alpine or distroless
2. **Don't run as root** — Use a non-root user
3. **Read-only filesystem** — Prevent writes to container
4. **Resource limits** — Prevent resource exhaustion
5. **Network isolation** — Limit network access
6. **Scan images** — Check for vulnerabilities
7. **Update regularly** — Keep base images updated
8. **Use secrets management** — Don't hardcode secrets

## 📝 Practice Exercises

1. **Security Audit:** Audit FicHub's Docker configuration for security issues.

2. **Non-Root User:** Modify the Dockerfile to run as a non-root user.

3. **Resource Limits:** Set memory and CPU limits for all Docker services.

