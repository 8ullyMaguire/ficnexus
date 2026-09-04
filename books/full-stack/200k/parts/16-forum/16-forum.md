# Part 16 — Forum

> In this chapter you will learn how FicHub's community forum works — categories, cursor-paginated topics, threaded posts, follow/unfollow topics, the moderation point system, and admin tools (bans, locks, pins). The forum is built on a separate `forum_core` crate with FTS search via PostgreSQL `tsvector`.

---

## Overview

```
GET  /api/forum/categories            — visible categories + topic counts
GET  /api/forum/topics?category=&cursor=&limit= — cursor-paginated topics
POST /api/forum/categories            — create category (role ≥ 10)
GET  /api/forum/topics/{id}           — topic detail (OP + posts, cursor, view_count)
POST /api/forum/topics                — create topic + OP post (one tx)
PATCH /api/forum/topics/{id}          — edit (author ≤ 15 min or mod)
DELETE /api/forum/topics/{id}         — soft-delete (author or mod; mod → modlog)
POST  /api/forum/topics/{id}/posts    — reply (+ notifications)
PATCH  /api/forum/posts/{id}          — edit post body (author ≤ 15 min or mod)
POST  /api/forum/topics/{id}/follow   — toggle follow (returns following + count)
GET   /api/forum/topics/{id}/read     — mark topic read
GET   /api/forum/search?q=&category=  — FTS over topics + posts (snippets)
GET   /api/forum/moderation/status    — my mod points / window / eligibility
POST  /api/forum/posts/{id}/moderate  — spend 1 point, adjust score
POST  /api/admin/forum/hide/{id}      — fast-hide (role ≥ 5)
POST  /api/admin/forum/topics/{id}/lock|pin  — status toggles (role ≥ 5)
POST  /api/admin/forum/bans            — ban management (role ≥ 5/10)
```

Roles: 0 reader, 5 curator, 10 admin. Forum levels are site-wide: curator = level ≥ 50, admin = level ≥ 100 (env-overridable via `FORUM_CURATOR_LEVEL` / `FORUM_ADMIN_LEVEL`). EXP per level defaults to 100 (env: `FORUM_EXP_PER_LEVEL`). Level capped at 100.

---

## Chapter 16.1 — Categories and Topic Listing

### Goal

Build the forum's category system and cursor-paginated topic listing.

### Actions

```rust
// src/routes/forum.rs (lines 228-278)
#[derive(Debug, Deserialize)]
pub struct CreateCategoryBody { pub slug: String, pub title: String, pub description: String, pub position: Option<i32> }

#[derive(Debug, Deserialize)]
pub struct ListTopicsParams { pub category: String, pub cursor: Option<i64>, pub limit: Option<i64> }

const DEFAULT_LIMIT: i64 = 25;
const MAX_LIMIT: i64 = 100;

/// GET /api/forum/categories — visible categories + topic counts
pub async fn list_categories(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    // Returns categories ordered by position, with topic_count per category
    let rows = sqlx::query_as::<_, (i32, String, String, String, i64, i32)>(
        r#"SELECT id, slug, title, description, position,
                  (SELECT COUNT(*) FROM forum_topics WHERE category_id = fc.id AND status = 'open' AND deleted_at IS NULL)
           FROM forum_categories fc
           ORDER BY position ASC, id ASC"#
    ).fetch_all(&state.db).await?;
    Ok(Json(json!({ "err": 0, "categories": rows })))
}

/// GET /api/forum/topics?category=general&cursor=0&limit=25
pub async fn list_topics(State(state): State<Arc<AppState>>,
    Query(params): Query<ListTopicsParams>) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT).max(1);
    let cursor = params.cursor.unwrap_or(0);

    // Cursor pagination: id > cursor, ordered by id DESC (newest first)
    let rows = sqlx::query_as::<_, (i64, String, Option<String>, i64, i64, i64, bool, String, String, i64)>(
        r#"SELECT ft.id, ft.title, ft.payload, ft.author_id, ft.category_id,
                  ft.view_count, ft.locked, ft.sticky, ft.created_at::text, ft.updated_at::text,
                  (SELECT COUNT(*) FROM forum_posts fp WHERE fp.topic_id = ft.id AND fp.deleted_at IS NULL AND fp.is_hidden = FALSE) AS post_count
           FROM forum_topics ft
           JOIN forum_categories fc ON fc.id = ft.category_id
           WHERE fc.slug = $1 AND ft.deleted_at IS NULL AND ft.status = 'open'
             AND ft.id > $2
           ORDER BY ft.id DESC, ft.created_at DESC
           LIMIT $3::int4"#
    ).bind(&params.category).bind(cursor).bind(limit).fetch_all(&state.db).await?;
    Ok(Json(json!({ "err": 0, "topics": rows, "cursor": limit })))
}
```

> **💡 Key Concept**: Cursor pagination (`id > cursor`) instead of OFFSET — this avoids the "missing rows on insert" problem that OFFSET has when new topics are created between page loads. The cursor is the last topic ID seen, and the next page fetches `id > cursor`.

### Try It Yourself

```bash
# Get categories
curl /api/forum/categories

# List topics in "general" category
curl "/api/forum/topics?category=general&cursor=0&limit=25"
```

### Check

- ✅ Categories ordered by `position ASC, id ASC`.
- ✅ `topic_count` = open non-deleted topics per category.
- ✅ Cursor pagination: `id > cursor`, ordered by `id DESC`.
- ✅ `limit` capped at 100, min 1.
- ✅ `post_count` computed via correlated subquery (not a JOIN).

### What you built

The forum category system and cursor-based topic listing — avoiding OFFSET pagination pitfalls, computing counts via subqueries.

---

## Chapter 16.2 — Topic Creation and Posts

### Goal

Build topic + first post creation (in one transaction), threaded replies, and the 15-minute edit window.

### Actions

```rust
// src/routes/forum.rs (lines 271-295)
#[derive(Debug, Deserialize)]
pub struct CreateTopicBody { pub title: String, pub category_slug: String, pub body: String, pub payload: Option<Value> }

/// POST /api/forum/topics — create topic + OP post (one tx)
pub async fn create_topic(auth: AuthUser, State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTopicBody>) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    if body.title.trim().is_empty() { return Err(AppError::BadRequest("Title required".into())); }
    if body.body.trim().is_empty() { return Err(AppError::BadRequest("Body required".into())); }

    let mut tx = state.db.begin().await?;
    let category_id: i32 = sqlx::query_scalar("SELECT id FROM forum_categories WHERE slug = $1")
        .bind(&body.category_slug).fetch_one(&mut tx).await?;

    let topic_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_topics (category_id, title, author_id, payload) VALUES ($1, $2, $3, $4) RETURNING id"
    ).bind(category_id).bind(&body.title).bind(user_id).bind(&body.payload).execute(&mut tx).await?.last_insert_id() as i64;

    // Insert first post (OP) in the same transaction
    sqlx::query("INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, $3)")
        .bind(topic_id).bind(user_id).bind(&body.body).execute(&mut tx).await?;
    tx.commit().await?;

    // Award EXP for topic creation (F7)
    award_post_exp(&state.db, user_id, topic_id).await;

    Ok(Json(json!({ "err": 0, "topic_id": topic_id })))
}
```

> **⚠️ Watch Out**: Topic creation wraps **both** the topic insert and the first post insert in a single database transaction (`tx`). If either fails, the whole thing rolls back — you never get a topic without an OP post. EXP is awarded **after** commit (so it doesn't fail the transaction if the EXP insert errors).

```rust
// POST /api/forum/topics/{id}/posts — reply
#[derive(Debug, Deserialize)]
pub struct CreatePostBody { pub body: String, pub quote_of: Option<i64> }

// Edit: author ≤ 15 min OR mod
// "if post.created_at < now() - 15min AND user is not mod → 403"
```

### Try It Yourself

```bash
# Create a topic with the first post
curl -X POST /api/forum/topics -H "Authorization: Bearer <token>" \
  -d '{"title": "Chapter 5 discussion", "category_slug": "general", "body": "What did everyone think?"}'

# Reply to a topic
curl -X POST /api/forum/topics/42/posts -H "Authorization: Bearer <token>" \
  -d '{"body": "I loved the magic system reveal!", "quote_of": 10}'
```

### Check

- ✅ Topic + first post created in one transaction (tx.begin/commit).
- ✅ Category resolved by slug (not ID) for API friendliness.
- ✅ EXP awarded after commit (non-blocking, best-effort).
- ✅ Edit window: 15 minutes for authors, unlimited for moderators.
- ✅ `quote_of` field for reply threading (references parent post ID).

### What you built

The topic/post creation flow — transaction-wrapped topic + OP post, EXP rewards, 15-minute edit window, quote-reply threading.

---

## Chapter 16.3 — Moderation Point System

### Goal

Understand FicHub's moderation system where curators spend points to rate posts (F5), with daily caps and reputation gates.

### Actions

```rust
// src/routes/forum.rs (lines 318-350+)
#[derive(Debug, Deserialize)]
pub struct ModeratePostBody { pub reason: String, pub score: i8 } // score: -1 to +3

/// GET /api/forum/moderation/status — my mod points / window / eligibility
pub async fn moderation_status(auth: AuthUser, State(state): State<Arc<AppState>>)
    -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let row: Option<(i32, bool)> = sqlx::query_as(
        r#"SELECT points, EXISTS(SELECT 1 FROM forum_bans WHERE user_id = $1 AND expires_at > NOW()) AS banned
           FROM forum_user_moderation WHERE user_id = $1"#
    ).bind(user_id).fetch_optional(&state.db).await?;

    Ok(Json(json!({
        "err": 0,
        "points": row.as_ref().map(|r| r.0).unwrap_or(0),
        "banned": row.as_ref().map(|r| r.1).unwrap_or(false),
        "level_required": curator_level(),
        "exp_per_level": exp_per_level(),
    })))
}

/// POST /api/forum/posts/{id}/moderate — spend 1 point to rate a post
pub async fn moderate_post(auth: AuthUser, State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>, Json(body): Json<ModeratePostBody>)
    -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let user_level = ...; // computed from exp
    if user_level < curator_level() { return Err(AppError::Forbidden("Curator level required".into())); }

    // Spend 1 moderation point
    // Insert into forum_moderations (post_id, moderator_id, score, reason)
    // Update post's aggregate score
    // Log to modlog
    Ok(Json(json!({ "err": 0, "msg": "Moderated" })))
}
```

> **💡 Key Concept**: The moderation point system (F5) means curators can only moderate a **limited number of posts per day** — they "spend" 1 point per moderation action. Points are earned through participation (forum posts, helpful actions) and reset daily. This distributes moderation load across many curators rather than concentrating it.

### Admin Tools

```rust
// src/routes/forum.rs — role ≥ 5 (curator) or ≥ 10 (admin)
// POST /api/admin/forum/hide/{postId}      — fast-hide 72h (role ≥ 5)
// POST /api/admin/forum/topics/{id}/lock    — lock topic (role ≥ 5)
// POST /api/admin/forum/topics/{id}/pin     — pin topic (role ≥ 5)
// POST /api/admin/forum/bans                — ban user (role ≥ 10)
// DELETE /api/admin/forum/bans/{id}         — remove ban (role ≥ 10)
// GET  /api/admin/forum/bans                — list bans (role ≥ 5)
```

### Try It Yourself

```bash
# Check moderation status
curl /api/forum/moderation/status -H "Authorization: Bearer <token>"
# → { "points": 5, "banned": false, "level_required": 50 }

# Moderate a post (+1 score, requires 1 point)
curl -X POST /api/forum/posts/42/moderate -H "Authorization: Bearer <token>" \
  -d '{"reason": "helpful", "score": 1}'

# Admin: lock a topic
curl -X POST /api/admin/forum/topics/42/lock -H "Authorization: Bearer <token>"
```

### Check

- ✅ Moderation requires level ≥ 50 (env-overridable `FORUM_CURATOR_LEVEL`).
- ✅ Each moderation spends 1 point from `forum_user_moderation.points`.
- ✅ `score` ranges from -1 (harmful) to +3 (excellent).
- ✅ Bans checked: `forum_bans` with `expires_at > NOW()`.
- ✅ Admin actions: hide (72h, role ≥ 5), lock/pin (role ≥ 5), bans (role ≥ 10).

### What you built

The moderation point system — daily-point-limited curation, level gates (curator ≥ 50, admin ≥ 100), score-based moderation (-1 to +3), ban management, and admin tools (hide/lock/pin/ban).

---

## Conclusion

You now understand FicHub's complete forum system:

1. **Categories + topics** — cursor-based pagination (not OFFSET), topic counts via subqueries.
2. **Topic/post creation** — transaction-wrapped topic + first post, EXP rewards, 15-min edit window, quote-reply threading.
3. **Moderation** — point-based system (spend 1 per action), daily caps, level gates (curator ≥ 50), score-based voting (-1 to +3), ban management.
4. **Admin tools** — hide (72h), lock, pin, ban management with role thresholds.
5. **FTS search** — `tsvector` full-text search over topics + posts with snippet generation.