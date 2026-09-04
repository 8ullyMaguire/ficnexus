# Part 13 — Follows and Feed

> In this chapter you will learn how FicHub's follow system works — following users, works, and authors, with per-follow exclusion rules that filter out specific works/series/fandoms from your updates feed. Plus the personalized updates feed that powers the timeline.

---

## Overview

FicHub's follow system supports three target types:

- **Users** — follow another user, get their activity (reviews, follows, kudos)
- **Works** — follow a specific work, get updates (new chapters, edits)
- **Authors** — follow by author name (string-based), get all their works' updates

Each follow supports **exclusions** — per-follow rules that filter out specific works, series, or fandoms from your updates feed. This lets you follow an author but exclude their "crossover" fandoms.

The backend lives in `src/routes/follows.rs` (298 lines) and the feed in `src/routes/feed.rs`.

---

## Chapter 13.1 — Following Users, Works, and Authors

### Goal

Build the universal follow endpoint that handles three target types with type-specific validation.

### Actions

#### 1. The follow request

```rust
// src/routes/follows.rs (lines 12-17)
pub struct FollowBody {
    pub target_type: String,    // "user" | "work" | "author"
    pub target_id: Option<i32>,  // required for "user" and "work"
    pub author_name: Option<String>,  // required for "author"
}
```

#### 2. The follow handler

```rust
// src/routes/follows.rs (lines 19-64)
pub async fn follow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<FollowBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    match body.target_type.as_str() {
        "user" => {
            let target = body.target_id.ok_or_else(|| AppError::BadRequest("target_id required for user follow".to_string()))?;
            if target == user_id {
                return Err(AppError::BadRequest("Cannot follow yourself".to_string()));
            }
            queries::follow_user(&state.db, user_id, target).await?;
        }
        "work" => {
            let target = body.target_id.ok_or_else(|| AppError::BadRequest("target_id required for work follow".to_string()))?;
            // Only works that exist can be followed
            if queries::get_work(&state.db, target).await?.is_none() {
                return Err(AppError::BadRequest("work not found".to_string()));
            }
            queries::follow_work(&state.db, user_id, target).await?;
        }
        "author" => {
            let name = body.author_name.as_deref().ok_or_else(|| AppError::BadRequest("author_name required for author follow".to_string()))?;
            if name.trim().is_empty() {
                return Err(AppError::BadRequest("author_name must not be empty".to_string()));
            }
            queries::follow_author(&state.db, user_id, name).await?;
        }
        _ => return Err(AppError::BadRequest("target_type must be 'user', 'work', or 'author'".to_string())),
    }

    // Return the follow id so the client can unfollow without re-listing
    let follow_id = match body.target_type.as_str() {
        "work" => queries::find_follow_for_work(&state.db, user_id, body.target_id.unwrap_or(0)).await?,
        "author" => queries::find_follow_for_author(&state.db, user_id, body.author_name.as_deref().unwrap_or("")).await?,
        _ => None,
    };

    Ok(Json(json!({
        "err": 0,
        "msg": "Following",
        "follow_id": follow_id,
    })))
}
```

> **⚠️ Watch Out**: Author follows use **string-based name matching** — `author_name` is stored as a string, not a foreign key. This handles pseuds, name variations, and legacy data. The query `queries::follow_author` inserts the raw name string. If an author changes their pen name, existing follows continue to work (they follow the stored string).

#### 3. Unfollowing

```rust
// DELETE /api/v1/follows/{id} — unfollow
pub async fn unfollow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let removed = queries::unfollow(&state.db, user_id, follow_id).await?;

    Ok(Json(json!({ "err": 0, "removed": removed })))
}
```

#### 4. Listing follows

```rust
// GET /api/v1/follows — list who I follow
pub async fn list_follows_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let follows = queries::list_follows(&state.db, user_id).await?;

    let items: Vec<Value> = follows.into_iter().map(|f| {
        json!({
            "id": f.id,
            "followee_id": f.followee_id,
            "work_id": f.work_id,
            "author_name": f.author_name,
            "created_at": f.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "follows": items })))
}

// GET /api/v1/follows/check/{target_type}/{target_id} — check if following
pub async fn check_follow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((target_type, target_id)): Path<(String, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let is_following = match target_type.as_str() {
        "user" => queries::is_following_user(&state.db, user_id, target_id).await?,
        "work" => queries::is_following_work(&state.db, user_id, target_id).await?,
        _ => false,
    };

    let follow_id = if target_type == "work" {
        queries::find_follow_for_work(&state.db, user_id, target_id).await?
    } else {
        None
    };

    Ok(Json(json!({
        "err": 0,
        "is_following": is_following,
        "follow_id": follow_id,
    })))
}

// GET /api/v1/follows/followers/{user_id} — get user's followers
pub async fn get_followers_handler(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let followers = queries::get_followers(&state.db, user_id).await?;

    let items: Vec<Value> = followers.into_iter().map(|f| {
        json!({
            "follower_id": f.follower_id,
            "created_at": f.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "followers": items, "count": items.len() })))
}
```

### Try It Yourself

```bash
# Follow a user
curl -X POST /api/v1/follows -H "Authorization: Bearer <token>" \
  -d '{"target_type": "user", "target_id": 42}'

# Follow a work
curl -X POST /api/v1/follows -d '{"target_type": "work", "target_id": 123}'

# Follow an author
curl -X POST /api/v1/follows -d '{"target_type": "author", "author_name": "J.K. Rowling"}'

# Check if following a work (returns follow_id for the client)
curl /api/v1/follows/check/work/123 -H "Authorization: Bearer <token>"

# Get user's followers
curl /api/v1/follows/followers/42
```

### Check

- ✅ Three target types with type-specific validation (target_id for user/work, author_name for author).
- ✅ Self-follow prevented (target != user_id for user follows).
- ✅ Work follow verifies work exists (404 if not found).
- ✅ Author follow validates non-empty author_name.
- ✅ Follow id returned on create (for client-side unfollow without re-listing).
- ✅ `check` endpoint returns `is_following` + `follow_id` for works.
- ✅ `followers` endpoint returns list of users following a given user.

### What you built

The universal follow endpoint — three target types (user/work/author), self-follow prevention, work existence check, author string-based name matching, follow id return, and the check/followers endpoints.

---

## Chapter 13.2 — Follow Exclusions

### Goal

Build per-follow exclusion rules that filter specific works/series/fandoms from an author follow's updates feed.

### Actions

#### 1. The exclusion request

```rust
// src/routes/follows.rs (lines 148-155)
pub struct FollowExclusionBody {
    /// "work" | "series" | "fandom"
    pub exclude_type: String,
    pub work_id: Option<i32>,
    pub series_id: Option<i32>,
    pub fandom: Option<String>,
}
```

#### 2. Add/exclusion validation

```rust
// src/routes/follows.rs (lines 159-180)
fn validate_exclusion_target(
    exclude_type: &str,
    work_id: Option<i32>,
    series_id: Option<i32>,
    fandom: Option<&str>,
) -> Result<(Option<i32>, Option<i32>, Option<String>), &'static str> {
    match exclude_type {
        "work" => {
            let wid = work_id.ok_or("work_id required for a work exclusion")?;
            Ok((Some(wid), None, None))
        }
        "series" => {
            let sid = series_id.ok_or("series_id required for a series exclusion")?;
            Ok((None, Some(sid), None))
        }
        "fandom" => {
            let name = fandom.map(str::trim).filter(|s| !s.is_empty());
            let name = name.ok_or("fandom required for a fandom exclusion")?;
            Ok((None, None, Some(name.to_string())))
        }
        _ => Err("exclude_type must be 'work', 'series', or 'fandom'"),
    }
}
```

#### 3. Add exclusion

```rust
// POST /api/v1/follows/{follow_id}/exclusions — add an exclusion to a follow
pub async fn add_follow_exclusion_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
    Json(body): Json<FollowExclusionBody>,
) -> Result<Json<Value>, AppError> {
    let (work_id, series_id, fandom) = validate_exclusion_target(
        &body.exclude_type, body.work_id, body.series_id, body.fandom.as_deref(),
    ).map_err(|e| AppError::BadRequest(e.to_string()))?;

    let exclusion_id = queries::create_follow_exclusion(
        &state.db, auth.user_id.unwrap(), follow_id, &body.exclude_type,
        work_id, series_id, fandom.as_deref(),
    ).await?.ok_or_else(|| AppError::NotFound("follow not found".to_string()))?;

    Ok(Json(json!({
        "err": 0, "msg": "Exclusion added",
        "exclusion_id": exclusion_id,
    })))
}
```

> **⚠️ Watch Out**: Author follows are the primary use case for exclusions — an author follow includes ALL their works' updates, but the user can exclude specific works, series, or fandoms. The `EXCLUDE` clause in the feed query filters them out. This is a **content-based** filter, not an author-based one — you follow the author, but exclude specific content.

#### 4. List exclusions and remove

```rust
// GET /api/v1/follows/{follow_id}/exclusions — list a follow's exclusions
pub async fn list_follow_exclusions_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let exclusions = queries::list_follow_exclusions(&state.db, user_id, follow_id).await?;

    let items: Vec<Value> = exclusions.into_iter().map(|e| {
        json!({
            "id": e.id,
            "follow_id": e.follow_id,
            "exclude_type": e.exclude_type,
            "work_id": e.exclude_work_id,
            "series_id": e.exclude_series_id,
            "fandom": e.exclude_fandom,
            "created_at": e.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "exclusions": items })))
}

// DELETE /api/v1/follows/{follow_id}/exclusions/{exclusion_id} — remove one
pub async fn delete_follow_exclusion_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((follow_id, exclusion_id)): Path<(i64, i64)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let removed = queries::delete_follow_exclusion(&state.db, user_id, follow_id, exclusion_id).await?;

    Ok(Json(json!({ "err": 0, "removed": removed })))
}
```

### Try It Yourself

```bash
# Add work exclusion (exclude a specific work from an author's feed)
curl -X POST /api/v1/follows/42/exclusions -H "Authorization: Bearer <token>" \
  -d '{"exclude_type": "work", "work_id": 123}'

# Add fandom exclusion (exclude all Twilight fandom works from an author's feed)
curl -X POST /api/v1/follows/42/exclusions -d '{"exclude_type": "fandom", "fandom": "Twilight"}'

# List exclusions
curl /api/v1/follows/42/exclusions -H "Authorization: Bearer <token>"

# Remove
curl -X DELETE /api/v1/follows/42/exclusions/7 -H "Authorization: Bearer <token>"
```

### Check

- ✅ Three exclusion types: work (by id), series (by id), fandom (by name).
- ✅ `validate_exclusion_target` enforces exactly one target non-null.
- ✅ Fandom exclusion: trimmed string, non-empty.
- ✅ Owner auth: only the follow owner can add/remove exclusions.
- ✅ `list_follow_exclusions` returns all exclusions for a follow with full detail.
- ✅ `delete_follow_exclusion` by exclusion_id.

### What you built

The follow exclusion system — type-specific validation, three exclusion types, owner-gated add/remove, and the feed exclusion query.

---

## Chapter 13.3 — The Updates Feed

### Goal

Build the personalized updates feed that powers the user's timeline — follows-based content with exclusion filtering, deduplication, and pagination.

### Actions

#### 1. The feed query

```rust
// src/routes/feed.rs (lines 1-50, excerpt)
pub async fn updates_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<UpdatesParams>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let limit = params.limit.unwrap_or(20).min(100).max(1);
    let offset = params.offset.unwrap_or(0).max(0);

    let updates = queries::get_updates_feed(&state.db, user_id, limit, offset).await?;

    let items: Vec<Value> = updates.into_iter().map(|u| {
        json!({
            "id": u.id,
            "update_type": u.update_type,
            "work_id": u.work_id,
            "url_id": u.url_id,
            "author_name": u.author_name,
            "title": u.title,
            "excerpt": u.excerpt,
            "canonical_title": u.canonical_title,
            "canonical_author": u.canonical_author,
            "source": u.source,
            "seen": u.seen,
            "created_at": u.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({
        "err": 0,
        "updates": items,
        "count": items.len(),
        "next": if items.len() == limit as usize { Some(offset + limit) } else { None },
    })))
}
```

```sql
-- The feed query (simplified — real implementation is more complex)
SELECT DISTINCT ON (u.url_id)
    u.id, u.update_type, u.work_id, u.url_id, u.author_name,
    u.title, u.excerpt, u.canonical_title, u.canonical_author,
    u.source, u.seen, u.created_at
FROM updates u
JOIN follows f ON (
    (f.follow_type = 'work' AND f.work_id = u.work_id) OR
    (f.follow_type = 'author' AND f.author_name = u.author_name)
)
WHERE f.user_id = $1
  AND NOT EXISTS (
      SELECT 1 FROM follow_exclusions fe
      WHERE fe.follow_id = f.id
        AND (
            (fe.exclude_type = 'work' AND fe.exclude_work_id = u.work_id) OR
            (fe.exclude_type = 'series' AND fe.exclude_series_id = u.series_id) OR
            (fe.exclude_type = 'fandom' AND u.fandom = fe.exclude_fandom)
        )
  )
ORDER BY u.url_id, u.created_at DESC
LIMIT $2 OFFSET $3;
```

> **⚠️ Watch Out**: The feed uses `DISTINCT ON (url_id)` to deduplicate — if multiple follows match the same update (e.g., you follow both the work AND the author), you only see it once. The exclusion `NOT EXISTS` clause filters out updates from excluded works/series/fandoms. The query joins follows (work_id or author_name match) and excludes by follow_exclusions.

#### 2. Mark as seen

```rust
// src/routes/feed.rs (lines 51-100)
pub async fn mark_seen_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
    Json(body): Json<MarkSeenBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let count = queries::mark_follow_seen(&state.db, user_id, follow_id, body.last_read_post_id)
        .await?;

    Ok(Json(json!({ "err": 0, "marked": count })))
}
```

### Try It Yourself

```bash
# Get your updates feed
curl /api/v1/updates -H "Authorization: Bearer <token>"

# Mark a follow as seen (updates after this point show as unseen)
curl -X POST /api/v1/follows/42/seen -H "Authorization: Bearer <token>" \
  -d '{"last_read_post_id": 12345}'
```

### Check

- ✅ `DISTINCT ON (url_id)` prevents duplicate updates from multiple follows.
- ✅ Exclusion `NOT EXISTS` filters out excluded works/series/fandoms.
- ✅ Feed pagination: `limit` (1-100), `offset`, `next` cursor.
- ✅ Each update shows `update_type`, `work_id`, `url_id`, `author_name`, `title`, `excerpt`, `canonical_title`, `canonical_author`, `source`.
- ✅ `seen` flag per update.
- ✅ Mark-seen per follow (resets unseen count for that follow).

### What you built

The updates feed — follows-based content gathering with exclusion filtering, deduplication (DISTINCT ON), pagination, seen tracking per follow, and exclusion query joins.

---

## Chapter 13.4 — Feed Architecture Deep Dive

### Goal

Understand how the feed query is structured under the hood — the JOIN logic, exclusion handling, and deduplication strategy.

### Actions

#### 1. The follow table model

```sql
-- src/db/migrations/001_initial.sql (follow tables)
CREATE TABLE follows (
    id          SERIAL PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    follow_type TEXT NOT NULL CHECK (follow_type IN ('user', 'work', 'author')),
    followee_id INTEGER,          -- for user follows
    work_id     INTEGER,          -- for work follows
    author_name TEXT,             -- for author follows (string-based)
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (follow_type = 'user' AND followee_id IS NOT NULL AND work_id IS NULL AND author_name IS NULL) OR
        (follow_type = 'work' AND work_id IS NOT NULL AND followee_id IS NULL AND author_name IS NULL) OR
        (follow_type = 'author' AND author_name IS NOT NULL AND followee_id IS NULL AND work_id IS NULL)
    )
);

CREATE TABLE follow_exclusions (
    id              SERIAL PRIMARY KEY,
    follow_id       INTEGER NOT NULL REFERENCES follows(id) ON DELETE CASCADE,
    exclude_type    TEXT NOT NULL CHECK (exclude_type IN ('work', 'series', 'fandom')),
    exclude_work_id INTEGER,
    exclude_series_id INTEGER,
    exclude_fandom  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (exclude_type = 'work' AND exclude_work_id IS NOT NULL AND exclude_series_id IS NULL AND exclude_fandom IS NULL) OR
        (exclude_type = 'series' AND exclude_series_id IS NOT NULL AND exclude_work_id IS NULL AND exclude_fandom IS NULL) OR
        (exclude_type = 'fandom' AND exclude_fandom IS NOT NULL AND exclude_work_id IS NULL AND exclude_series_id IS NULL)
    )
);
```

> **💡 Key Concept**: The `CHECK` constraint on `follows` enforces that exactly one target column is non-null based on `follow_type`. Same for `follow_exclusions`. This prevents invalid data at the database level — no "work follow with author_name" or "work exclusion with series_id".

#### 2. Feed query optimization

The feed query has several optimization challenges:

```sql
-- Problem: DISTINCT ON with ORDER BY — must order by the DISTINCT column FIRST
SELECT DISTINCT ON (u.url_id) ...
FROM updates u
JOIN follows f ON (...)
WHERE f.user_id = $1 AND NOT EXISTS (...)
ORDER BY u.url_id, u.created_at DESC  -- url_id first, then timestamp descending
LIMIT $2 OFFSET $3;
```

> **⚠️ Watch Out**: PostgreSQL's `DISTINCT ON` requires the `ORDER BY` to start with the same expression as `DISTINCT ON`. This means `ORDER BY u.url_id, u.created_at DESC` — first sort by url_id groups duplicates, then within each group sort by created_at descending picks the most recent. The feed returns the most recent update for each url_id.

#### 3. Exclusion join strategy

```sql
-- Exclusion filter: NOT EXISTS with OR conditions
WHERE NOT EXISTS (
    SELECT 1 FROM follow_exclusions fe
    WHERE fe.follow_id = f.id
      AND (
          (fe.exclude_type = 'work' AND fe.exclude_work_id = u.work_id) OR
          (fe.exclude_type = 'series' AND fe.exclude_series_id = u.series_id) OR
          (fe.exclude_type = 'fandom' AND u.fandom = fe.exclude_fandom)
      )
)
```

> **💡 Key Concept**: The exclusion filter is an `OR` of three `AND` conditions — each matching a different exclusion type. This is efficient because PostgreSQL can use indexes on `follow_exclusions(follow_id)` and the conditional logic is simple equality checks. The `NOT EXISTS` short-circuits on the first match.

#### 4. Author follow string matching

```sql
-- Author follow: string match on author_name
JOIN follows f ON f.follow_type = 'author' AND f.author_name = u.author_name
```

Authors are followed by name string, not by author ID. This means:
- If an author uses multiple pen names (e.g., "J.K. Rowling" and "Robert Galbraith"), you need separate follows.
- Follows are "sticky" — even if the author changes their name, existing follows keep working.

### Try It Yourself

```bash
# Add an index to speed up the feed query
CREATE INDEX idx_updates_url_id_created ON updates (url_id, created_at DESC);
CREATE INDEX idx_follows_user_id_type ON follows (user_id, follow_type);
CREATE INDEX idx_follow_exclusions_follow_id ON follow_exclusions (follow_id);
```

### Check

- ✅ `follows` CHECK constraint enforces exactly one non-null target column.
- ✅ `follow_exclusions` CHECK constraint enforces exactly one non-null exclude column.
- ✅ `DISTINCT ON` requires `ORDER BY` to start with the DISTINCT column.
- ✅ Exclusion query uses `NOT EXISTS` with OR of three AND conditions.
- ✅ Author follows use string-based name matching (not foreign key).
- ✅ Feed pagination with `next` cursor.

### What you built

The feed architecture — CHECK constraints for data integrity, `DISTINCT ON` with required ORDER BY ordering, exclusion filter with `NOT EXISTS` + OR of ANDs, author string-based follow matching, pagination with next cursor, and mark-seen per-follow.

---

## Conclusion

You now understand FicHub's follow and feed system:

1. **Follows** — three target types (user/work/author), self-follow prevention, follow id return, check/followers endpoints.
2. **Exclusions** — type-specific validation (work/series/fandom), owner-gated add/remove, three exclusion types.
3. **Feed** — follows-based content with exclusion filtering, `DISTINCT ON` deduplication, pagination, seen tracking, mark-seen per follow.
4. **Architecture** — CHECK constraints, `DISTINCT ON` ORDER BY requirement, exclusion `NOT EXISTS` with OR of ANDs, author string-based follows.

Follows and exclusions together give users fine-grained control over what appears in their timeline — follow an author but exclude their Twilight work, follow a series but exclude specific fandoms. The feed query handles it all in a single PostgreSQL query with JOINs, NOT EXISTS, and DISTINCT ON.
