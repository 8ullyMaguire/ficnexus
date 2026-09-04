# Extended Content: Complete API Documentation

---

# FicHub API Reference

## Overview

The FicHub API provides programmatic access to the fanfiction download service. All endpoints return JSON responses unless otherwise specified. The API is versioned under `/api/v0/`.

## Authentication

Currently, the FicHub API does not require authentication for most endpoints. The curator endpoints require a bearer token:

```
Authorization: Bearer <curator_token>
```

## Rate Limiting

All requests are subject to rate limiting. Rate limits are applied at three levels:

1. **Global** — Total requests across all endpoints (60 per minute)
2. **Per-IP** — Requests per individual IP address (30 per minute)
3. **Per-site** — Requests to specific fanfiction sites (20 per minute)

When rate limited, the response includes a `retry_after` field indicating how many seconds to wait.

## Error Format

All API responses include an `err` field:

```json
{
    "err": 0,
    "data": {}
}
```

Error codes:
- `0` — Success
- `-1` — Internal error
- `-5` — Not found
- `-6` — Upstream error
- `-403` — Forbidden
- `-429` — Rate limited

---

## Export Endpoints

### GET /api/v0/epub

Export a fanfiction story as EPUB and HTML bundle.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | URL of the story to export |

**Response:**

```json
{
    "err": 0,
    "url_id": "abc123def456",
    "meta": {
        "url_id": "abc123def456",
        "title": "My Favorite Story",
        "author": "GreatAuthor",
        "chapters": 10,
        "words": 50000,
        "desc": "Story description...",
        "status": "complete",
        "source": "https://archiveofourown.org/works/123456",
        "author_url": "https://archiveofourown.org/users/GreatAuthor"
    },
    "urls": {
        "epub": "/cache/epub/abc123def456?h=a1b2c3d4e5f6",
        "html": "/cache/html/abc123def456?h=f6e5d4c3b2a1"
    }
}
```

**Error Responses:**

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | -1 | Invalid URL or unsupported site |
| 404 | -5 | Story not found |
| 429 | -429 | Rate limited |
| 502 | -6 | Upstream site error |

**Example Request:**

```bash
curl "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/123456"
```

**Example Response:**

```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "meta": {
        "title": "Harry Potter and the Methods of Rationality",
        "author": "Less Wrong",
        "chapters": 122,
        "words": 660000,
        "status": "complete"
    },
    "urls": {
        "epub": "/cache/epub/a1b2c3d4e5f6?h=x9y8z7w6v5u4",
        "html": "/cache/html/a1b2c3d4e5f6?h=t3s2r1q0p9o8"
    }
}
```

### GET /api/v0/meta

Get metadata for a story without generating exports.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Story identifier |
| `url` | string | No | Original URL (alternative to url_id) |

**Response:**

```json
{
    "err": 0,
    "url_id": "abc123def456",
    "meta": {
        "title": "My Favorite Story",
        "author": "GreatAuthor",
        "chapters": 10,
        "words": 50000,
        "status": "complete"
    }
}
```

### GET /api/v0/remote

Get information about the client's connection.

**Response:**

```json
{
    "ip": "192.168.1.1",
    "port": 12345,
    "is_automated": false
}
```

---

## Cache Download Endpoints

### GET /cache/{type}/{url_id}

Download a cached export file.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `type` | string | Export type: `epub` or `html` |
| `url_id` | string | Story identifier |

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `h` | string | No | Content hash for validation |

**Response:**

Binary file (EPUB or ZIP) with appropriate Content-Type header.

### GET /cache/{type}/{url_id}/{filename}

Download a cached export file with explicit filename.

**Response:**

Binary file with Content-Disposition header for download.

---

## Search Endpoints

### GET /api/v0/search

Search for stories with full-text search and filters.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | No | Search query |
| `tags` | string | No | Include tags (format: `type_id:name,type_id:name`) |
| `exclude_tags` | string | No | Exclude tags |
| `any_tags` | string | No | Include any of these tags |
| `min_words` | integer | No | Minimum word count |
| `max_words` | integer | No | Maximum word count |
| `complete` | boolean | No | Filter by completion status |
| `source` | string | No | Filter by source site |
| `sort` | string | No | Sort order: `-relevance`, `-words`, `-date`, `-title` |
| `page` | integer | No | Page number (default: 1) |

**Response:**

```json
{
    "err": 0,
    "total": 1234,
    "page": 1,
    "per_page": 20,
    "results": [
        {
            "url_id": "abc123def456",
            "title": "My Favorite Story",
            "author": "GreatAuthor",
            "chapters": 10,
            "words": 50000,
            "status": "complete",
            "source": "ao3",
            "rank": 0.85
        }
    ]
}
```

**Example Request:**

```bash
curl "http://localhost:3000/api/v0/search?q=harry+potter&tags=1:Harry+Potter&min_words=10000&complete=true&sort=-words"
```

**Tag Filter Format:**

Tags are specified as comma-separated `type_id:name` pairs:

- `1:Harry Potter` — Fandom tag
- `2:Hermione Granger` — Character tag
- `3:Harry/Hermione` — Relationship tag
- `4:Angst` — Freeform tag
- `5:Violence` — Warning tag

---

## Tag Endpoints

### POST /api/v0/tags/submit

Submit a new tag for a story.

**Request Body:**

```json
{
    "url_id": "abc123def456",
    "name": "Harry Potter",
    "tag_type_id": 1
}
```

**Response:**

```json
{
    "err": 0,
    "tag_id": 42,
    "tag_name": "Harry Potter",
    "tag_type_id": 1,
    "is_new": false
}
```

### POST /api/v0/tags/vote

Vote on a tag for a story.

**Request Body:**

```json
{
    "url_id": "abc123def456",
    "tag_id": 42,
    "value": 1
}
```

**Response:**

```json
{
    "err": 0,
    "new_score": 5,
    "hidden": false
}
```

### POST /api/v0/tags/flag

Flag a tag for moderation.

**Request Body:**

```json
{
    "url_id": "abc123def456",
    "tag_id": 42,
    "reason": "Incorrect tag"
}
```

**Response:**

```json
{
    "err": 0,
    "msg": "tag flagged"
}
```

### GET /api/v0/tags

Get tags for a story.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Story identifier |

**Response:**

```json
{
    "err": 0,
    "tags": [
        {
            "id": 42,
            "name": "Harry Potter",
            "tag_type_id": 1,
            "score": 5,
            "hidden": false
        }
    ]
}
```

---

## Recommendation Endpoints

### GET /api/v0/recommendations

Get recommendations for a story.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Story identifier |
| `limit` | integer | No | Maximum recommendations (default: 10) |

**Response:**

```json
{
    "err": 0,
    "recommendations": [
        {
            "url_id": "def789abc012",
            "title": "Similar Story",
            "author": "AnotherAuthor",
            "score": 0.75,
            "reason": "15 readers also favourited this"
        }
    ]
}
```

### POST /api/v0/recommendations/suggest

Suggest a story for the recommendation engine.

**Request Body:**

```json
{
    "url": "https://archiveofourown.org/works/789012"
}
```

**Response:**

```json
{
    "err": 0,
    "url_id": "def789abc012",
    "title": "Suggested Story",
    "msg": "suggestion submitted"
}
```

### POST /api/v0/recommendations/vote

Vote on a recommendation.

**Request Body:**

```json
{
    "url_id": "abc123def456",
    "recommended_url_id": "def789abc012",
    "value": 1
}
```

**Response:**

```json
{
    "err": 0,
    "upvotes": 10,
    "downvotes": 2,
    "net_score": 8
}
```

### GET /api/v0/recommendations/votes

Get votes for a recommendation.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Story identifier |
| `recommended_url_id` | string | Yes | Recommended story identifier |

**Response:**

```json
{
    "err": 0,
    "upvotes": 10,
    "downvotes": 2,
    "net_score": 8
}
```

---

## Curator Endpoints

These endpoints require authentication via the `Authorization: Bearer <token>` header.

### POST /api/v0/curator/alias

Create a tag alias.

**Request Body:**

```json
{
    "alias_name": "HP",
    "canonical_name": "Harry Potter",
    "tag_type_id": 1
}
```

**Response:**

```json
{
    "err": 0,
    "alias": "HP",
    "canonical": "Harry Potter",
    "msg": "alias created"
}
```

### POST /api/v0/curator/merge

Merge two tags.

**Request Body:**

```json
{
    "source_tag_id": 42,
    "target_tag_id": 43
}
```

**Response:**

```json
{
    "err": 0,
    "msg": "tags merged"
}
```

### DELETE /api/v0/curator/tags/{id}

Delete a tag.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `force` | boolean | No | Force deletion even if tag is in use |

**Response:**

```json
{
    "err": 0,
    "msg": "tag deleted"
}
```

### GET /api/v0/curator/flags

List unresolved flags.

**Response:**

```json
{
    "err": 0,
    "flags": [
        {
            "id": 1,
            "url_id": "abc123def456",
            "tag_id": 42,
            "tag_name": "Harry Potter",
            "reason": "Incorrect tag"
        }
    ]
}
```

### POST /api/v0/curator/flags/{id}/resolve

Resolve a flag.

**Response:**

```json
{
    "err": 0,
    "msg": "flag resolved"
}
```

---

## OPDS Endpoints

### GET /opds

Root OPDS catalog.

**Response:** Atom XML

### GET /opds/new

Recent stories.

**Response:** Atom XML

### GET /opds/popular

Popular stories.

**Response:** Atom XML

### GET /opds/tags

List tag types.

**Response:** Atom XML

### GET /opds/tags/{type_id}

List tags of a specific type.

**Response:** Atom XML

### GET /opds/tags/{type_id}/{tag_name}

List stories with a specific tag.

**Response:** Atom XML

### GET /opds/authors

List authors.

**Response:** Atom XML

### GET /opds/search

Search OPDS catalog.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query |

**Response:** Atom XML

### GET /opds/shelves

List user shelves.

**Response:** Atom XML

### GET /opds/shelf/{shelf_id}

List stories in a shelf.

**Response:** Atom XML

---

## Health Check Endpoints

### GET /health

Check application health.

**Response:**

```json
{
    "status": "healthy",
    "checks": {
        "database": true,
        "redis": true
    }
}
```

### GET /ready

Check application readiness.

**Response:** HTTP 200 OK or 503 Service Unavailable

---

## API Documentation Endpoint

### GET /api/

Get API documentation.

**Response:**

```json
{
    "name": "fichub-rs API",
    "version": "0.1.0",
    "endpoints": {
        "/api/v0/epub": {
            "method": "GET",
            "description": "Export a story as EPUB",
            "params": {
                "q": "URL of the story to export"
            }
        }
    }
}
```

---

## Common Patterns

### Pagination

Most list endpoints support pagination:

```bash
# First page
GET /api/v0/search?q=harry+potter&page=1

# Second page
GET /api/v0/search?q=harry+potter&page=2
```

### Filtering

Multiple filters can be combined:

```bash
GET /api/v0/search?q=harry+potter&tags=1:Harry+Potter&min_words=10000&complete=true&sort=-words
```

### Error Handling

Always check the `err` field:

```javascript
const response = await fetch('/api/v0/epub?q=https://archiveofourown.org/works/123456');
const data = await response.json();

if (data.err !== 0) {
    console.error('Error:', data.msg);
    if (data.retry_after) {
        console.log('Retry after:', data.retry_after, 'seconds');
    }
} else {
    console.log('Success:', data.urls.epub);
}
```

### Caching

Cache download URLs include a content hash for validation:

```
/cache/epub/abc123def456?h=a1b2c3d4e5f6
```

If the story is updated, the hash changes, causing a cache miss and re-export.

---

## Rate Limit Headers

When rate limited, the response includes:

```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1640000000
Retry-After: 30

{"err": -429, "msg": "rate limited", "retry_after": 30}
```

---

## Versioning

The API is versioned under `/api/v0/`. When breaking changes are introduced, a new version will be created (e.g., `/api/v1/`). The current version will continue to work but may receive only bug fixes.

---

## Changelog

### v0.1.0 (2024-01-01)

- Initial API release
- Export endpoints (epub, html)
- Search endpoint
- Tag system
- Recommendation engine
- OPDS catalog
- Curator tools

