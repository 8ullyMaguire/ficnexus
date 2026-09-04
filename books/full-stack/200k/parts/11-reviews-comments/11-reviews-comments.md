# Part 11 — Reviews and Comments

> In this chapter you will learn how FicHub's written feedback system works — written reviews (1–5 stars + title + body) and threaded comments with a constructive-content heuristic. We'll build the review UPSERT, comment tree with recursive CTE, and the 24-hour edit window.

---

## Overview

FicHub has two tiers of written feedback:

- **Reviews** (`src/routes/reviews.rs`, 232 lines): 1–5 star rating + title + body. One per user per work (UPSERT). Positive-only via constructive heuristic.
- **Comments** (`src/routes/comments.rs`, 876 lines): threaded replies with recursive CTE, 24h edit window, curator hide, soft-delete.

Both share the **constructive-content heuristic** (`src/routes/comments.rs:29-41`) — a cheap substring check against 30+ negative markers. Non-constructive posts are stored but hidden from public views.

---

## Chapter 11.1 — Reviews

### Goal

Build the review system: 1–5 star rating + title + body, one per user per work (UPSERT), constructive heuristic, positive-only public list.

### Actions

The `reviews` table has `UNIQUE(user_id, work_id)` — a second POST per user per work is an UPSERT. Rating must be 1–5. Body is required (max 8000 chars), title optional (max 200 chars). The `constructive` flag is evaluated at INSERT time using `crate::routes::comments::constructive_score(&text)`.

```rust
// POST /api/reviews — upsert review
pub async fn upsert_review_handler(auth: AuthUser, State(state): State<Arc<AppState>>,
    Json(body): Json<ReviewBody>) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    if !(1..=5).contains(&body.rating) { return Err(AppError::BadRequest("Rating must be 1..=5".to_string())); }
    let text = body.body.as_deref().unwrap_or("").trim().to_string();
    if text.is_empty() { return Err(AppError::BadRequest("Review body cannot be empty".to_string())); }
    let constructive = crate::routes::comments::constructive_score(&text);
    sqlx::query(
        "INSERT INTO reviews (user_id, work_id, url_id, rating, title, body, constructive)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (user_id, work_id) DO UPDATE SET
             rating = EXCLUDED.rating, title = EXCLUDED.title, body = EXCLUDED.body,
             constructive = EXCLUDED.constructive, updated_at = NOW()
         RETURNING id, created_at::text, COALESCE(updated_at::text, created_at::text), constructive",
    ).bind(user_id).bind(body.work_id).bind(url_id).bind(body.rating)
    .bind(&title).bind(&text).bind(constructive).fetch_one(&state.db).await?;
    Ok(Json(json!({ "err": 0, "review": { ... } })))
}
```

Public list filters `deleted_at IS NULL AND constructive = TRUE` — negative reviews stay in DB for the rec engine but are invisible.

```bash
# Write a constructive review
curl -X POST /api/reviews -H "Authorization: Bearer <token>" \
  -d '{"work_id": 123, "rating": 4, "title": "Great read", "body": "Loved the pacing!"}'

# Non-constructive review → stored with constructive=false, hidden from GET
curl -X POST /api/reviews -H "Authorization: Bearer <token>" \
  -d '{"work_id": 123, "rating": 1, "body": "This is garbage"}'
# → not visible in GET /api/works/123/reviews
```

### Check

- ✅ `ON CONFLICT (user_id, work_id) DO UPDATE` — reviews are replaceable (unlike kudos no-op).
- ✅ `constructive` evaluated at INSERT, not at read time.
- ✅ Public list: `deleted_at IS NULL AND constructive = TRUE`.

### What you built

The review system — 1–5 star UPSERT, constructive heuristic, positive-only public list, reviews feed the "likes" count in the rating aggregate.

---

## Chapter 11.2 — Threaded Comments

### Goal

Build threaded comments with recursive CTE for reply trees, 24-hour edit window, curator hide, and soft-delete.

### Actions

```rust
// src/routes/comments.rs (876 lines)
const NEGATIVE_MARKERS: &[&str] = &[
    "terrible", "awful", "horrible", "garbage", "trash", "rubbish",
    "disappointing", "hate", "stupid", "idiotic", "dumb", "boring",
    "cringe", "worst", "sucks", "useless", "pointless", "pathetic",
    "annoying", "disgusting", "crap", "shit", "abandon",
    "quit reading", "couldn't finish", "drops this", "dropped",
];

pub fn is_negative_comment(body: &str) -> bool {
    let lower = body.to_lowercase();
    NEGATIVE_MARKERS.iter().any(|m| lower.contains(m))
}
```

```rust
// GET /api/v1/works/{url_id}/comments — threaded tree via recursive CTE
let all_replies = sqlx::query_as(
    "WITH RECURSIVE thread AS (
        SELECT c.id, c.body, c.user_id, u.username, c.created_at::text, c.parent_id AS root_id
        FROM comments c LEFT JOIN users u ON u.id = c.user_id
        WHERE c.parent_id = ANY($1) AND c.deleted_at IS NULL AND c.is_hidden = FALSE AND c.constructive = TRUE
        UNION ALL
        SELECT c.id, c.body, c.user_id, u.username, c.created_at::text, t.root_id
        FROM comments c LEFT JOIN users u ON u.id = c.user_id
        JOIN thread t ON c.parent_id = t.id
        WHERE c.deleted_at IS NULL AND c.is_hidden = FALSE AND c.constructive = TRUE
    )
    SELECT id, body, user_id, username, created_at, root_id FROM thread ORDER BY root_id, created_at ASC"
).bind(&top_ids).fetch_all(&state.db).await?;
```

```rust
// POST — comment with honeypot + parent-scoping
// PATCH (24h window) — edit via EXTRACT(EPOCH FROM (NOW() - created_at)) < 86400
// DELETE — soft delete (deleted_at = NOW()), owner or curator
// PATCH /hide — curator only (is_hidden = TRUE)
```

> **⚠️ Watch Out**: The 24h edit window uses `EXTRACT(EPOCH FROM (NOW() - created_at)) < 86400` — a SQL comparison, not Rust-side `chrono`. Parent comments must belong to the same `url_id` (reply scoping).

### Check

- ✅ Recursive CTE fetches full reply tree in one query (no N+1).
- ✅ 24h edit window via SQL epoch comparison.
- ✅ Honeypot trap (`website` + `form_opened_at`) on comment POST.
- ✅ Soft-delete vs curator-hide distinction.
- ✅ Constructive heuristic shared with reviews.

### What you built

The threaded comments system — recursive CTE, constructive heuristic, honeypot trap, 24h SQL edit window, soft-delete, curator hide, parent-scoping by url_id.

---

## Conclusion

You now understand FicHub's written feedback system:

1. **Reviews** — 1–5 star rating + title + body, UPSERT per user/work, constructive heuristic at insert, positive-only public list. Feeds the "likes" count in the aggregate.
2. **Comments** — threaded replies via recursive CTE, honeypot anti-bot trap, 24h SQL edit window, soft-delete (owner) + curator-hide, constructive heuristic shared with reviews.
3. **Design** — both stored with `constructive` flag; non-constructive hidden from public but kept for rec engine. Comments are threaded (parent_id), reviews are flat.