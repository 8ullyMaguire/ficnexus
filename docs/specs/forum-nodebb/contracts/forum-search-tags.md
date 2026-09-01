# Contract: Forum Search & Tags

**Route Prefix**: `/api/forum/search`, `/api/forum/tags`  
**Spec References**: FR-123..135 (search), FR-136..142 (tags)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level` (read: anonymous OK)

---

## 1. Full-Text Search

### GET `/api/forum/search`

**Query Params**:
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `q` | string | **required** | Search query (plain text, tsquery syntax supported) |
| `category` | string | - | Category slug filter |
| `tag` | string | - | Tag name filter |
| `author` | string | - | Username filter |
| `type` | string | `all` | `all` \| `topics` \| `posts` \| `users` \| `groups` |
| `date_from` | ISO8601 | - | Created after |
| `date_to` | ISO8601 | - | Created before |
| `sort` | string | `relevance` | `relevance` \| `date` \| `views` \| `replies` |
| `limit` | int | 25 | Max 100 |
| `cursor` | int64 | - | Pagination (ts_rank or created_at) |

**Response 200**:
```json
{
  "err": 0,
  "results": {
    "topics": [
      {
        "id": 42,
        "title": "Hello <mark>World</mark>",
        "slug": "hello-world-42",
        "category": {"id": 1, "slug": "general", "title": "General"},
        "author": {"id": 7, "username": "alice"},
        "snippet": "This is a <mark>test</mark> post about...",
        "reply_count": 15,
        "view_count": 128,
        "created_at": "2026-08-30T10:00:00Z",
        "rank": 0.85
      }
    ],
    "posts": [
      {
        "id": 101,
        "topic_id": 42,
        "topic_title": "Hello World",
        "topic_slug": "hello-world-42",
        "author": {"id": 5, "username": "bob"},
        "snippet": "Replying to <mark>hello</mark>...",
        "created_at": "2026-08-30T10:10:00Z",
        "rank": 0.72
      }
    ],
    "users": [
      {"id": 7, "username": "alice", "trust_level": 3, "avatar_url": "..."}
    ],
    "groups": [
      {"id": 2, "name": "HP Fans", "slug": "hp-fans", "member_count": 42}
    ]
  },
  "total_hits": 124,
  "next_cursor": 1725105600000,
  "has_more": true
}
```

**Search Implementation**:
- Postgres `tsvector` + `tsquery` (English config)
- Title weight 'A', body 'D', tags 'B' (see data-model.md triggers)
- `ts_rank_cd` with cover density normalization
- Anonymous: searches public categories only
- Authenticated: includes private categories user has `read` privilege

**Special Syntax**:
| Syntax | Meaning |
|--------|---------|
| `term1 term2` | AND (both terms) |
| `term1 | term2` | OR |
| `!term` | NOT |
| `"exact phrase"` | Phrase search |
| `term:*` | Prefix match |

---

## 2. Search Suggestions (Autocomplete)

### GET `/api/forum/search/suggest`

**Query Params**:
| Param | Type | Default |
|-------|------|---------|
| `q` | string | required (min 2 chars) |
| `type` | string | `tags` \| `users` \| `categories` \| `all` |

**Response 200**:
```json
{
  "err": 0,
  "suggestions": {
    "tags": [{"name": "harry-potter", "count": 156}],
    "users": [{"id": 7, "username": "alice", "trust_level": 3}],
    "categories": [{"id": 1, "slug": "general", "title": "General"}]
  }
}
```

---

## 3. Tags

### 3.1 List Tags

#### GET `/api/forum/tags`

**Query Params**:
| Param | Type | Default |
|-------|------|---------|
| `q` | string | - (prefix filter) |
| `sort` | string | `usage` \| `name` \| `created` |
| `limit` | int | 50 |

**Response 200**:
```json
{
  "err": 0,
  "tags": [
    {"id": 1, "name": "harry-potter", "description": "HP discussions", "color": "#7c3aed", "icon": "🧙", "usage_count": 156, "merged_into": null}
  ]
}
```

---

### 3.2 Get Tag Detail (Topic List)

#### GET `/api/forum/tags/{tagName}`

**Query Params**: Same as topic list (cursor, limit, sort).

**Response 200**:
```json
{
  "err": 0,
  "tag": {"id": 1, "name": "harry-potter", "description": "...", "color": "#7c3aed", "usage_count": 156},
  "topics": [...],  // same shape as topic list
  "next_cursor": ...,
  "has_more": true
}
```

---

### 3.3 Create Tag (Auto on Topic Create)

**No direct endpoint** — tags created implicitly when added to topic (TL3+).

**Admin Tag Management**:

#### POST `/api/admin/forum/tags`
```json
{"name": "canon", "description": "Canon-compliant", "color": "#ef4444", "icon": "📜"}
```
**Auth**: Admin only.

#### PATCH `/api/admin/forum/tags/{tagId}`
```json
{"description": "Updated", "color": "#dc2626"}
```
**Auth**: Admin only.

#### POST `/api/admin/forum/tags/{tagId}/merge`
```json
{"into_tag_id": 5}  // merge this tag into target
```
**Auth**: Admin only.

**Effect**:
- All `forum_topic_tags` rows updated to `into_tag_id`
- Source tag `merged_into` set to target
- `usage_count` recomputed
- Tag page redirects

#### DELETE `/api/admin/forum/tags/{tagId}`
**Auth**: Admin only. Only if `usage_count = 0` and not merged.

---

## 4. Topic Tags (Client-Side on Compose)

### Add Tags to Topic (during create/update)

**Handled in** `POST /api/forum/topics` and `PATCH /api/forum/topics/{id}` via `tags: string[]`.

**Rules**:
- Max 5 tags per topic
- New tags auto-created if user **TL3+** (else 403: `"Creating new tags requires Regular (L3) or higher"`)
- Tag names normalized: lowercase, hyphens, max 30 chars
- Duplicate tags ignored

---

## 5. Search Index Maintenance

### Manual Reindex (Admin)

#### POST `/api/admin/forum/search/reindex`

**Auth**: Admin only.

**Response 200**: `{"err": 0, "reindexed": 12450, "duration_ms": 3200}`

Runs backfill for all `search_vector` NULL rows + triggers refresh.

---

## 6. Realtime Search Updates

**Not realtime** — search index updated synchronously on write (triggers). No WS events for search results.