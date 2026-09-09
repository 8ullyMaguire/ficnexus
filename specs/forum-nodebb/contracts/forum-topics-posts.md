# Contract: Forum Topics & Posts

**Route Prefix**: `/api/forum/topics`, `/api/forum/posts`  
**Spec References**: FR-009..029, FR-032..037, FR-042..047 (quotes, mentions, uploads, scheduled)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level`

---

## 1. List Topics (Cursor Paginated)

### GET `/api/forum/topics`

**Query Params**:
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `category` | string | - | Category slug (required for category view) |
| `cursor` | int64 | - | `last_activity_at` timestamp (ms) for pagination |
| `limit` | int | 25 | Max 100 |
| `sort` | string | `activity` | `activity` | `created` | `views` | `votes` |
| `filter` | string | - | `pinned` | `locked` | `solved` | `unread` (auth only) |
| `tag` | string | - | Tag name filter |

**Response 200**:
```json
{
  "err": 0,
  "topics": [
    {
      "id": 42,
      "category_id": 1,
      "category_slug": "general",
      "title": "Hello World",
      "slug": "hello-world-42",
      "author": {"id": 7, "username": "alice", "trust_level": 3, "avatar_url": "/avatars/7.png"},
      "body": "First post **markdown** body...",
      "payload": {"type": "work", "id": 123},
      "status": "open",
      "view_count": 128,
      "reply_count": 15,
      "last_post_id": 105,
      "last_activity_at": "2026-08-31T12:00:00Z",
      "created_at": "2026-08-30T10:00:00Z",
      "tags": [{"id": 1, "name": "welcome", "color": "#3b82f6"}],
      "poll": {"id": 1, "question": "Favorite?", "vote_count": 12, "closed": false},
      "unread": true,
      "is_following": false,
      "follower_count": 3
    }
  ],
  "next_cursor": 1725105600000,
  "has_more": true
}
```

**Auth Matrix**:
| Role / Trust | Anonymous | TL0 | TL1+ | Curator | Admin |
|--------------|-----------|-----|------|---------|-------|
| **Read** | ✓ | ✓ | ✓ | ✓ | ✓ |
| **Unread/Following** | 401 | ✓ | ✓ | ✓ | ✓ |
| **Mod-only category** | ✗ | ✗ | ✗ | ✓ | ✓ |

**Notes**: `unread` = `last_read_post_id < last_post_id` (from `forum_read_state`). Anonymous gets no unread/following.

---

## 2. Get Topic Detail (with Posts)

### GET `/api/forum/topics/{topicId}`

**Query Params**:
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `after` | int64 | - | Post ID cursor for pagination (load newer) |
| `before` | int64 | - | Post ID cursor for pagination (load older) |
| `limit` | int | 20 | Posts per page (max 50) |
| `include_op` | bool | true | Include OP in posts array |

**Response 200**:
```json
{
  "err": 0,
  "topic": {
    "id": 42,
    "category_id": 1,
    "category_slug": "general",
    "title": "Hello World",
    "slug": "hello-world-42",
    "author": {"id": 7, "username": "alice", "trust_level": 3},
    "body": "Full OP markdown...",
    "payload": {"type": "work", "id": 123},
    "status": "pinned",
    "view_count": 128,
    "reply_count": 15,
    "last_post_id": 105,
    "last_activity_at": "2026-08-31T12:00:00Z",
    "created_at": "2026-08-30T10:00:00Z",
    "updated_at": "2026-08-30T10:05:00Z",
    "tags": [{"id": 1, "name": "welcome", "color": "#3b82f6"}],
    "poll": {"id": 1, "question": "Favorite?", "options": [...], "user_vote": null, "closed": false},
    "is_following": true,
    "follower_count": 3,
    "can_edit": true,
    "can_delete": true,
    "can_moderate": false
  },
  "posts": [
    {
      "id": 101,
      "topic_id": 42,
      "author": {"id": 7, "username": "alice", "trust_level": 3},
      "body": "Reply markdown...",
      "payload": {},
      "quote_of": null,
      "quote_preview": null,
      "edited_at": null,
      "created_at": "2026-08-30T10:10:00Z",
      "uploads": [{"id": 1, "url": "/uploads/forum/2026/08/abc.webp", "width": 800, "height": 600}],
      "mentions": [{"id": 5, "username": "bob"}],
      "vote_counts": {"up": 3, "down": 0},
      "user_vote": null,
      "can_edit": true,
      "can_delete": true
    }
  ],
  "next_cursor": 105,
  "prev_cursor": 95,
  "has_more": false
}
```

**Auth Matrix**: Same as List. `can_edit`/`can_delete`/`can_moderate` computed per post (author ≤15min or mod).

---

## 3. Get Topic by Slug

### GET `/api/forum/topics/by-slug/{topicSlug}`

**Response**: Same as Get Topic Detail.  
**Use Case**: Canonical URL `/forum/board/{slug}.{id}` resolves via slug first.

---

## 4. Create Topic (+ OP Post)

### POST `/api/forum/topics`

**Request**:
```json
{
  "title": "New Topic Title",
  "category_slug": "general",
  "body": "Opening post **markdown** with @mention and /fic 123",
  "payload": {"type": "work", "id": 123},
  "tags": ["welcome", "intro"],
  "poll": {
    "question": "Favorite color?",
    "options": ["Red", "Blue", "Green"],
    "max_selections": 1,
    "allow_change": true,
    "close_at": "2026-09-07T12:00:00Z"
  },
  "scheduled_at": "2026-09-01T12:00:00Z"
}
```

**Validation**:
- `title`: 1-120 chars
- `category_slug`: must exist, user must have `write` privilege (via group/category privileges)
- `body`: 1-20000 chars
- `tags`: max 5, auto-create if new (TL3+), admin can merge
- `poll`: optional; if present, creates `forum_polls` + options
- `scheduled_at`: optional; if future, topic hidden until cron promotes (TL2+)

**Trust Gates**:
- Topic create: **TL1+** (`assert_min_trust(db, user, 1, "Create topic")`)
- Poll create: **TL1+**
- Scheduled publish: **TL2+**
- Tags (new): **TL3+**

**Response 201**:
```json
{
  "err": 0,
  "topic": {
    "id": 43,
    "slug": "new-topic-title-43",
    "status": "scheduled",  // or "open"
    "created_at": "2026-08-31T12:00:00Z"
  },
  "post": {
    "id": 106,
    "topic_id": 43,
    "created_at": "2026-08-31T12:00:00Z"
  }
}
```

**Errors**:
- `400`: validation failed
- `401`: not authenticated
- `403`: trust level insufficient / no write privilege in category
- `404`: category not found
- `409`: duplicate slug (handled by appending `-{id}`)

**Side Effects**:
- Author auto-follows topic
- `forum_read_state` upsert for author (last_read_post_id = OP post id)
- Mentions parsed → `forum_notifications` type `mention`
- Search vector updated
- Reputation hook: `topic_create` (+5 default)
- WS `new_topic` broadcast to category watchers

---

## 5. Update Topic (Title/Body/Status/Tags)

### PATCH `/api/forum/topics/{topicId}`

**Request** (all optional):
```json
{
  "title": "Updated Title",
  "body": "Updated OP markdown",
  "status": "locked",           // "open" | "locked" | "pinned" | "archived"
  "tags": ["updated", "tags"],  // replaces all tags
  "scheduled_at": "2026-09-01T12:00:00Z"  // TL2+ only, resets if in past
}
```

**Auth Rules**:
- Author: can edit title/body/tags within **15 min** of creation (or last edit)
- Curator (role ≥ 5): can edit any field anytime, status changes
- TL5+: can lock/pin own topics
- Admin: full control

**Response 200**: Updated topic object (same as GET detail).

**Errors**: `403` (not author/mod, or edit window expired), `404`, `400` (validation).

---

## 6. Delete Topic (Soft)

### DELETE `/api/forum/topics/{topicId}`

**Auth Rules**:
- Author: can delete own topic (soft delete, `deleted_at` set)
- Curator+: can delete any topic (modlog entry)
- **Never hard delete** — content preserved for modlog

**Response 200**:
```json
{"err": 0, "deleted": true, "topic_id": 42}
```

---

## 7. Create Post (Reply)

### POST `/api/forum/topics/{topicId}/posts`

**Request**:
```json
{
  "body": "Reply with @mention and quote",
  "payload": {"type": "work", "id": 456},
  "quote_of": 101
}
```

**Validation**:
- `body`: 1-20000 chars
- `quote_of`: must exist in same topic, not deleted/hidden
- Topic must be postable (`status.is_postable()`)

**Trust Gates**:
- Reply: **TL1+** (same as topic create)
- Uploads in reply: **TL1+**

**Response 201**:
```json
{
  "err": 0,
  "post": {
    "id": 107,
    "topic_id": 42,
    "author": {"id": 7, "username": "alice", "trust_level": 3},
    "body": "Reply with @mention...",
    "quote_of": 101,
    "quote_preview": {"post_id": 101, "author": "bob", "excerpt": "First post **markdown**..."},
    "created_at": "2026-08-31T12:05:00Z",
    "uploads": [],
    "mentions": [{"id": 5, "username": "bob"}]
  }
}
```

**Side Effects**:
- Topic `last_post_id`, `last_activity_at`, `reply_count` updated
- Followers notified (`forum_notifications` type `reply`)
- Mentions parsed → `forum_notifications` type `mention`
- Quote creates soft link (`quote_of` FK)
- Search vector updated
- Reputation hook: `post_create` (+2 default)
- WS `new_post` broadcast to topic subscribers

---

## 8. Update Post

### PATCH `/api/forum/posts/{postId}`

**Request**:
```json
{"body": "Edited markdown"}
```

**Auth Rules**: Same as topic edit (author ≤15min, Curator+ anytime).

**Response 200**: Updated post object.

**Side Effects**: `edited_at` set, search vector updated, WS `edit_post` broadcast.

---

## 9. Delete Post (Soft)

### DELETE `/api/forum/posts/{postId}`

**Auth Rules**: Same as topic delete.

**Response 200**: `{"err": 0, "deleted": true, "post_id": 107}`

---

## 10. Follow / Unfollow Topic

### POST `/api/forum/topics/{topicId}/follow`

**Response 200**:
```json
{"err": 0, "following": true, "follower_count": 4}
```

### GET `/api/forum/topics/{topicId}/follow`

**Response 200**:
```json
{"err": 0, "following": true, "follower_count": 4}
```

---

## 11. Mark Topic Read

### POST `/api/forum/topics/{topicId}/read`

**Request**:
```json
{"last_read_post_id": 107}
```

**Response 200**:
```json
{"err": 0, "read": true}
```

**Upserts** `forum_read_state` for user+topic.

---

## 12. Vote on Post (Removed in v2 — Mod Points Only)

**Note**: Post ±1 votes removed per spec v2. Moderation points replace voting. See `forum-moderation.md` (future).

---

## 13. Upload in Post/Topic (see contracts/forum-uploads.md)

### POST `/api/forum/uploads` (multipart/form-data)

Returns upload ID for embedding in `payload` or composer.