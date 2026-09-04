# Supplementary Content: Complete API Reference and Operations Guide

---

# Complete API Reference

## Authentication

FicHub uses two authentication mechanisms:

### Bearer Token (Curator Endpoints)

Curator endpoints require a Bearer token in the Authorization header:

```http
POST /api/v0/curator/alias
Authorization: Bearer my-secret-token
Content-Type: application/json

{
    "alias_name": "HP",
    "canonical_tag_id": 42
}
```

The token is configured via the `CURATOR_TOKEN` environment variable. If not configured, curator endpoints return an error.

### IP-Based Identification (Tags and Recommendations)

Tag submissions, votes, and recommendations use the client's IP address for identification. This is extracted via the `ConnectInfo` extractor:

```rust
async fn handler(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> Json<Value> {
    let ip = remote.ip();
    // Use ip for rate limiting and vote tracking
}
```

## Endpoint Reference

### GET /api/

Returns API documentation. No parameters required.

**Response:**
```json
{
    "name": "fichub-rs API",
    "version": "0.1.0",
    "endpoints": {
        "/api/v0/epub": {
            "method": "GET",
            "params": { "q": "URL of the fanfiction" },
            "description": "Fetch metadata and download links"
        }
    }
}
```

### GET /api/v0/epub

The main export endpoint. Scrapes a fanfiction URL, generates exports, and returns download links.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | URL of the fanfiction |
| `automated` | string | No | Set to "true" to block automated requests |
| `format` | string | No | Specific format to generate (epub, html, mobi, pdf) |

**Example Request:**
```
GET /api/v0/epub?q=https://archiveofourown.org/works/12345678
```

**Success Response (200):**
```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/12345678",
    "url_id": "a1b2c3d4e5f6",
    "slug": "My_Amazing_Fic-a1b2c3d4e5f6",
    "meta": {
        "id": "a1b2c3d4e5f6",
        "title": "My Amazing Fanfiction",
        "author": "SomeAuthor",
        "chapters": 15,
        "words": 125000,
        "description": "A story about things...",
        "status": "complete",
        "source": "https://archiveofourown.org/works/12345678",
        "created": "2023-01-15T00:00:00Z",
        "updated": "2023-06-20T00:00:00Z"
    },
    "hashes": {
        "epub": "abc123def456",
        "html": "789xyz012abc"
    },
    "urls": {
        "epub": "/cache/epub/a1b2c3d4e5f6?h=abc123def456",
        "html": "/cache/html/a1b2c3d4e5f6?h=789xyz012abc"
    },
    "epub_url": "/cache/epub/a1b2c3d4e5f6?h=abc123def456",
    "html_url": "/cache/html/a1b2c3d4e5f6?h=789xyz012abc",
    "info": "My Amazing Fanfiction by SomeAuthor\n125000 words in 15 chapters\nStatus: complete\n",
    "notes": []
}
```

**Error Responses:**

| err | HTTP Status | Meaning |
|-----|-------------|---------|
| -1 | 400 | Missing query parameter |
| -5 | 400 | Unsupported URL or story not found |
| -7 | 400 | Fic or author is blacklisted |
| -10 | 400 | Automated request blocked |
| -429 | 429 | Rate limited |
| -6 | 502 | Scraper error (site down, blocked, etc.) |
| -1 | 500 | Internal server error |

### GET /api/v0/meta

Returns metadata without generating exports. Same parameters as `/api/v0/epub` but faster.

**Example Request:**
```
GET /api/v0/meta?q=https://archiveofourown.org/works/12345678
```

**Response:** Same as `/api/v0/epub` but with empty hashes and URLs.

### GET /api/v0/remote

Returns the client's IP address.

**Response:**
```json
{
    "ip": "192.168.1.100",
    "port": 54321,
    "is_automated": false
}
```

### GET /cache/{etype}/{url_id}/{fname}

Downloads a cached export file with hash validation.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `etype` | string | Export type: epub, html, mobi, pdf |
| `url_id` | string | Fic's unique ID |
| `fname` | string | Filename (contains the hash) |
| `h` | string | MD5 hash for validation (query param) |

**Example:**
```
GET /cache/epub/a1b2c3d4e5f6/my_fic.epub?h=abc123def456
```

**Response:** Binary file download with appropriate Content-Type header.

### GET /cache/{etype}/{url_id}

Downloads a cached export or redirects to trigger generation.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `etype` | string | Export type |
| `url_id` | string | Fic's unique ID |
| `h` | string | MD5 hash (query param, optional) |

If `h` is provided and the file exists, it's served directly. Otherwise, redirects to the frontend to trigger generation.

### GET /api/v0/recommendations

Returns recommendations for a fic.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | One of q/url_id | Fanfiction URL |
| `url_id` | string | One of q/url_id | Fic's unique ID |
| `n` | integer | No | Number of recommendations (default: 20, max: 100) |
| `site_domain` | string | No | Filter by site domain |

**Example:**
```
GET /api/v0/recommendations?url_id=a1b2c3d4e5f6&n=10
```

**Response:**
```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "recommendations": [
        {
            "url_id": "x9y8z7w6v5u4",
            "title": "Similar Story",
            "author": "Another Author",
            "words": 85000,
            "chapters": 20,
            "status": "complete",
            "score": 0.75,
            "community_score": 3,
            "download_urls": {}
        }
    ],
    "generated_at": "2024-01-15T10:30:00Z"
}
```

### POST /api/v0/recommendations/suggest

Submit a community suggestion linking two fics.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "suggested_url": "https://archiveofourown.org/works/87654321",
    "comment": "Both feature time travel to the Marauders era"
}
```

**Response:**
```json
{
    "err": 0,
    "suggestion_id": 42
}
```

### POST /api/v0/recommendations/vote

Vote on a community suggestion.

**Request Body:**
```json
{
    "suggestion_id": 42,
    "vote": 1
}
```

**Response:**
```json
{
    "err": 0,
    "new_score": 5
}
```

### GET /api/v0/recommendations/votes

List suggestions and votes for a fic.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Fic's unique ID |

**Response:**
```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "suggestions": [
        {
            "id": 42,
            "suggested_url_id": "x9y8z7w6v5u4",
            "comment": "Similar time travel premise",
            "net_votes": 5,
            "created": "2024-01-15T10:30:00Z"
        }
    ]
}
```

### POST /api/v0/tags/submit

Submit a tag on a fic.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "tag_name": "Angst",
    "tag_type_id": 4
}
```

**Response:**
```json
{
    "err": 0,
    "tag_id": 123,
    "tag_name": "Angst",
    "tag_type_id": 4,
    "is_new": false
}
```

### POST /api/v0/tags/vote

Vote on a tag.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "tag_id": 123,
    "value": 1
}
```

**Response:**
```json
{
    "err": 0,
    "new_score": 3,
    "hidden": false
}
```

### POST /api/v0/tags/flag

Flag a tag for curator review.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "tag_id": 123,
    "reason": "Tag is incorrect for this story"
}
```

**Response:**
```json
{
    "err": 0,
    "msg": "flag submitted"
}
```

### GET /api/v0/tags

Get tags for a fic.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Fic's unique ID |

**Response:**
```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "tags": [
        {
            "id": 1,
            "name": "Harry Potter",
            "tag_type_id": 1,
            "score": 5,
            "hidden": false
        },
        {
            "id": 42,
            "name": "Angst",
            "tag_type_id": 4,
            "score": -4,
            "hidden": true
        }
    ]
}
```

### POST /api/v0/curator/alias

Create a tag alias (requires Bearer token).

**Request Body:**
```json
{
    "alias_name": "HP",
    "canonical_tag_id": 1
}
```

### POST /api/v0/curator/merge

Merge two tags (requires Bearer token).

**Request Body:**
```json
{
    "source_tag_id": 10,
    "target_tag_id": 1
}
```

### DELETE /api/v0/curator/tags/{id}

Delete a tag (requires Bearer token).

### GET /api/v0/curator/flags

List unresolved flags (requires Bearer token).

### POST /api/v0/curator/flags/{id}/resolve

Resolve a flag (requires Bearer token).

### GET /api/v0/search

Full-text search with filters.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Full-text search query |
| `include_tags` | string | Comma-separated "type_id:name" (ALL must match) |
| `exclude_tags` | string | Comma-separated "type_id:name" (NONE must match) |
| `include_any_tags` | string | Comma-separated "type_id:name" (at least ONE) |
| `min_words` | integer | Minimum word count |
| `max_words` | integer | Maximum word count |
| `min_chapters` | integer | Minimum chapter count |
| `max_chapters` | integer | Maximum chapter count |
| `complete` | boolean | Filter by completion status |
| `source` | string | Filter by source URL |
| `date_from` | string | ISO 8601 date |
| `date_to` | string | ISO 8601 date |
| `sort` | string | Sort: -relevance, -date, -words, -chapters, -title |
| `page` | integer | Page number (default: 1) |
| `per_page` | integer | Results per page (default: 20, max: 50) |

**Example:**
```
GET /api/v0/search?q=harry+potter&include_tags=1:Harry+Potter&complete=true&sort=-words
```

**Response:**
```json
{
    "total": 1234,
    "page": 1,
    "per_page": 20,
    "results": [
        {
            "url_id": "a1b2c3d4e5f6",
            "title": "My Amazing Harry Potter Fic",
            "author": "SomeAuthor",
            "source": "https://archiveofourown.org/works/12345678",
            "words": 125000,
            "chapters": 15,
            "status": "complete",
            "description": "A Harry Potter story...",
            "rank": 0.85,
            "tags": [
                {"name": "Harry Potter", "type": "fandom", "type_id": 1, "score": 10}
            ],
            "total_freeform": 5
        }
    ]
}
```

---

# Operations Guide

## Monitoring

### Health Check

Add a health check endpoint:

```rust
async fn health_check() -> &'static str {
    "OK"
}

// In the router:
.route("/health", get(health_check))
```

### Key Metrics to Monitor

1. **Request rate** — Requests per second
2. **Response time** — p50, p95, p99 latencies
3. **Error rate** — Percentage of 4xx and 5xx responses
4. **Database connections** — Active vs pool size
5. **Cache hit rate** — Percentage served from cache
6. **Disk usage** — Cache directory size
7. **Redis memory** — Used memory and peak memory

### Log Analysis

FicHub logs every request with timing:

```
2024-01-15T10:30:00.123Z INFO request{method=GET uri=/api/v0/epub}: 200 response_time=1.234s
```

Use `grep` or `awk` to analyze:

```bash
# Requests per minute
grep "INFO request" fichub.log | awk '{print $1}' | cut -d: -f1-2 | sort | uniq -c

# Slow requests (> 5 seconds)
grep "response_time=[5-9]" fichub.log

# Error rates
grep -c "response_time=.*5[0-9][0-9]" fichub.log
```

## Backups

### Database Backup

```bash
# Daily backup
pg_dump -U fichub fichub | gzip > backup_$(date +%Y%m%d).sql.gz

# Restore
gunzip -c backup_20240115.sql.gz | psql -U fichub fichub

# Automated (add to crontab)
0 2 * * * pg_dump -U fichub fichub | gzip > /backups/fichub_$(date +\%Y\%m\%d).sql.gz
```

### Cache Backup

```bash
# Backup cache directory
rsync -avz /data/cache/ backup:/backups/fichub-cache/

# Automated
0 3 * * * rsync -avz /data/cache/ backup:/backups/fichub-cache/
```

### Full Backup Script

```bash
#!/bin/bash
set -e

BACKUP_DIR="/backups/fichub/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Database
pg_dump -U fichub fichub | gzip > "$BACKUP_DIR/database.sql.gz"

# Cache
rsync -avz /data/cache/ "$BACKUP_DIR/cache/"

# Config
cp /home/fichub/.env "$BACKUP_DIR/"

# Cleanup old backups (keep 30 days)
find /backups/fichub -maxdepth 1 -type d -mtime +30 -exec rm -rf {} +

echo "Backup complete: $BACKUP_DIR"
```

## Troubleshooting

### Common Issues

**"DATABASE_URL must be set"**
- Check that the `.env` file exists and has the correct value
- Verify the environment variable is exported

**"Failed to connect to database"**
- Verify PostgreSQL is running: `systemctl status postgresql`
- Check the connection string: `psql "postgres://fichub:fichub@localhost/fichub"`
- Check firewall rules if connecting remotely

**"Failed to connect to Redis"**
- Verify Redis is running: `systemctl status redis`
- Test connection: `redis-cli ping`

**"No scraper found for URL"**
- The URL format might be wrong
- Check supported sites: AO3, FF.net, XenForo, FictionPress, AFF, HP FanFic

**"Rate limited"**
- Too many requests from the same IP
- Wait for the retry-after period
- Check rate limiter configuration

**"Fic is blacklisted"**
- The fic or author is on the blacklist
- Check the blacklist tables in the database

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug,fichub=debug ./fichub
```

### Database Debugging

```sql
-- Check table sizes
SELECT pg_size_pretty(pg_total_relation_size('fic_info')) AS size;

-- Check index usage
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Check slow queries
SELECT query, calls, mean_exec_time, total_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

## Performance Tuning

### Connection Pool

If you see "connection pool exhausted" errors:

```rust
// Increase pool size
PgPoolOptions::new()
    .max_connections(50)  // Default is 20
    .connect(url).await?;
```

### Cache Size

Monitor cache directory size:

```bash
du -sh /data/cache/
```

If it's growing too large, set up automatic cleanup:

```bash
# Delete cache files older than 30 days
find /data/cache -type f -mtime +30 -delete
```

### Rate Limiting

Adjust rate limits based on your traffic:

```bash
# More lenient for internal use
REC_DEFAULT_DELAY_SECS=2

# Stricter for public-facing
REC_DEFAULT_DELAY_SECS=10
```

## Summary

This operations guide covers monitoring, backups, troubleshooting, and performance tuning. A well-maintained FicHub instance requires regular monitoring, automated backups, and proactive performance optimization.
