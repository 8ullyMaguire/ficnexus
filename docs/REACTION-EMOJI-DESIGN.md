# Reaction Emoji System Design for Forum Posts

## Executive Summary

Replace the existing `forum_post_votes` table (±1 binary voting) with a rich emoji reaction system. The moderation point system (`forum_posts.score`) and metamoderation stay unchanged — reactions are a parallel engagement layer, not a moderation layer.

---

## Current State

### Existing Infrastructure (DO NOT DELETE)

| Table | Purpose | Status |
|---|---|---|
| `forum_posts` | Post content, moderation score | Keep — `score` column stays for mod points |
| `forum_post_votes` | ±1 binary vote per user per post | **REPLACE** with reactions |
| `forum_mod_actions` | Mod point audit trail | Keep as-is |
| `forum_metamod_votes` | Metamoderation audit trail | Keep as-is |
| `kudos` | One-click kudos on works (not forum) | Keep as-is (separate feature) |

### How `forum_post_votes` Is Used Today

1. **Topic listing** (`GET /api/forum/topics?category=`) — computes `vote_score` as `SUM(v.value)` across all posts in a topic (line 1695 of forum.rs)
2. **No dedicated vote endpoints exist** — the table is created but there are no POST/DELETE handlers for `forum_post_votes` in forum.rs (the contract doc says "Vote endpoints are gone" in v2)

### Key Observation

The `forum_post_votes` table exists in the schema but **has no write endpoints** in the API. The only usage is the aggregate `vote_score` in topic listings. This makes it a clean candidate for replacement.

---

## Proposed Schema: `forum_post_reactions`

```sql
-- 051_forum_reactions.sql
-- Replace forum_post_votes (±1 binary) with multi-emoji reactions.
-- The moderation point system (forum_posts.score) stays untouched.

-- 1. Drop the old vote table (unused write endpoints).
DROP TABLE IF EXISTS forum_post_votes;

-- 2. New reactions table: one reaction per user per post per emoji.
CREATE TABLE IF NOT EXISTS forum_post_reactions (
    post_id    BIGINT  NOT NULL REFERENCES forum_posts(id) ON DELETE CASCADE,
    user_id    INT4    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji      TEXT    NOT NULL,  -- validated app-side to allowed set
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, user_id, emoji)
);
CREATE INDEX IF NOT EXISTS idx_forum_reactions_post ON forum_post_reactions (post_id);
CREATE INDEX IF NOT EXISTS idx_forum_reactions_user ON forum_post_reactions (user_id, created_at DESC);

-- 3. Constraint: max 3 different emojis per user per post (app-enforced, 
--    but a safety net index helps). The app limits to 3; DB doesn't enforce count
--    but the UNIQUE PK + app logic handle it.
```

### Why This Schema

- **One row per (post, user, emoji)** — same pattern as `forum_post_votes` PK but allows multiple emojis
- **No denormalized count columns** — reaction counts are computed at read time (small cost, no consistency issues)
- **`emoji` is TEXT** — flexible, future-proof. App validates against allowed set. Avoids enum migrations when adding new emojis.
- **No `reaction_type` column** — the emoji IS the type. No ambiguity.

---

## Allowed Emoji Set

Defined as a Rust constant + app-side validation:

```rust
const ALLOWED_EMOJIS: &[&str] = &[
    "👍",   // like / agree
    "❤️",   // love / support
    "😂",   // funny
    "🔥",   // hot / exciting
    "👏",   // applause / well said
    "🤔",   // thinking / interesting
    "😢",   // sad / sympathetic
    "😮",   // surprised / wow
];
```

- Start with 8 emojis — enough variety, not overwhelming
- Can be expanded later by adding to the constant + optional DB seed
- `textarea` picker in frontend; no free-text emoji input

---

## API Design

### POST /api/forum/posts/{postId}/reactions

Add or toggle a reaction. Idempotent: if the reaction already exists, remove it (toggle behavior).

```
Request:  { "emoji": "👍" }
Response: { "err": 0, "added": true, "reaction_count": 42, "my_reactions": ["👍", "❤️"] }
```

**Logic:**
1. Validate emoji is in `ALLOWED_EMOJIS`
2. Check post exists, not deleted/hidden
3. Check user is not banned in the post's category
4. Try DELETE → if a row was deleted, `added: false` (toggle off)
5. Else INSERT → `added: true` (toggle on)
6. Return updated reaction summary for the post

### GET /api/forum/posts/{postId}/reactions

Read reactions for a post.

```
Response: {
  "err": 0,
  "reactions": {
    "👍": { "count": 12, "usernames": ["alice", "bob", ...] },
    "❤️": { "count": 5, "usernames": ["charlie", ...] },
  },
  "my_reactions": ["👍"],
  "total_count": 17
}
```

- `usernames` is a list of reactors (capped at 5, with `+N more` count)
- `my_reactions` is the caller's own reactions (empty for anon)
- Public-read: works for anonymous visitors when `FORUM_PUBLIC_READ=true`

### Batch endpoint for topic detail

The topic detail endpoint (`GET /api/forum/topics/{id}`) returns posts. Add a reactions subquery per post to avoid N+1:

```sql
-- In the topic detail query, add a lateral join:
LEFT JOIN LATERAL (
    SELECT jsonb_object_agg(
        r.emoji, jsonb_build_object('count', r.cnt, 'usernames', r.unames)
    ) AS reactions,
    COALESCE(SUM(CASE WHEN r.user_id = $viewer_id THEN 1 ELSE 0 END), 0) AS my_reaction_count
    FROM (
        SELECT emoji, user_id,
               COUNT(*) OVER (PARTITION BY emoji) AS cnt,
               ARRAY_AGG(u.username) OVER (PARTITION BY emoji ORDER BY u.username) AS unames
        FROM forum_post_reactions pr
        JOIN users u ON u.id = pr.user_id
        WHERE pr.post_id = p.id
    ) r
) reactions ON TRUE
```

**Simpler approach for v1:** Two queries — get posts, then batch-load reactions for all post IDs in one query.

```rust
// After fetching posts in topic detail:
let post_ids: Vec<i64> = posts.iter().map(|p| p.id).collect();
let reactions = load_reactions_batch(&state.db, &post_ids, user_id).await?;
```

---

## Topic Listing Changes

Replace `vote_score` (sum of ±1 votes) with a richer metric:

```sql
-- Old: SUM(v.value) FROM forum_post_votes v
-- New: total reaction count across all posts in the topic
(SELECT COALESCE(SUM(sub.cnt), 0) FROM (
    SELECT COUNT(*) AS cnt FROM forum_post_reactions pr
    JOIN forum_posts p ON p.id = pr.post_id
    WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE
    GROUP BY pr.post_id, pr.emoji
) sub)::bigint
```

Alternatively, for simplicity, keep `vote_score` as a total count of all reactions:

```sql
(SELECT COUNT(*)::bigint FROM forum_post_reactions pr
 JOIN forum_posts p ON p.id = pr.post_id
 WHERE p.topic_id = t.id
   AND p.deleted_at IS NULL AND p.is_hidden = FALSE)
```

**Recommendation:** Use total reaction count as the topic-level "engagement score". It's simpler and more intuitive than a weighted sum.

---

## Rust Backend Changes

### New file: `src/routes/forum_reactions.rs`

```rust
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::forum::require_user;
use crate::server::AppState;

const ALLOWED_EMOJIS: &[&str] = &[
    "👍", "❤️", "😂", "🔥", "👏", "🤔", "😢", "😮",
];

#[derive(Debug, Deserialize)]
pub struct ReactionBody {
    pub emoji: String,
}

/// POST /api/forum/posts/{postId}/reactions — toggle a reaction
pub async fn toggle_reaction(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
    Json(body): Json<ReactionBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let emoji = body.emoji.trim();
    
    if !ALLOWED_EMOJIS.contains(&emoji) {
        return Err(AppError::BadRequest("invalid emoji".to_string()));
    }
    
    // Check post exists
    let post_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM forum_posts WHERE id = $1 AND deleted_at IS NULL AND is_hidden = FALSE)"
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);
    
    if !post_exists {
        return Err(AppError::BadRequest("post not found".to_string()));
    }
    
    // Toggle: try delete first
    let deleted = sqlx::query_scalar::<_, i64>(
        "DELETE FROM forum_post_reactions WHERE post_id = $1 AND user_id = $2 AND emoji = $3 RETURNING post_id"
    )
    .bind(post_id)
    .bind(user_id)
    .bind(emoji)
    .fetch_optional(&state.db)
    .await?;
    
    let added = deleted.is_none();
    if added {
        sqlx::query(
            "INSERT INTO forum_post_reactions (post_id, user_id, emoji) VALUES ($1, $2, $3)"
        )
        .bind(post_id)
        .bind(user_id)
        .bind(emoji)
        .execute(&state.db)
        .await?;
    }
    
    // Return updated state
    let summary = load_post_reactions(&state.db, post_id, Some(user_id)).await?;
    
    Ok(Json(json!({
        "err": 0,
        "added": added,
        "reactions": summary.reactions,
        "my_reactions": summary.my_reactions,
    })))
}

/// GET /api/forum/posts/{postId}/reactions — read reactions
pub async fn get_reactions(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id;
    let summary = load_post_reactions(&state.db, post_id, user_id).await?;
    
    Ok(Json(json!({
        "err": 0,
        "reactions": summary.reactions,
        "my_reactions": summary.my_reactions,
    })))
}
```

### Route registration in `src/server.rs`

```rust
.route("/api/forum/posts/:postId/reactions",
    get(forum_reactions::get_reactions)
     .post(forum_reactions::toggle_reaction))
```

---

## Frontend Changes

### Svelte Component: `PostReactions.svelte`

```
┌─────────────────────────────────────┐
│ 👍 12  ❤️ 5  🔥 3  + Add          │  ← reaction bar below post body
└─────────────────────────────────────┘
```

- Clicking an existing reaction: toggle (remove if already reacted, add if not)
- Clicking "+ Add": opens emoji picker with `ALLOWED_EMOJIS`
- My reactions highlighted (border/background)
- Count + usernames on hover (tooltip or popover)
- Compact layout in topic list; expanded in post detail

### Topic List Card Update

Replace `vote_score` display with total reaction count + top 2 emojis:

```
🔥 42  👍 12                    →  54 reactions
```

---

## Migration Steps

1. **Create migration** `051_forum_reactions.sql`:
   - `DROP TABLE IF EXISTS forum_post_votes`
   - `CREATE TABLE forum_post_reactions`
   
2. **Add Rust module** `src/routes/forum_reactions.rs`

3. **Register routes** in `src/server.rs`

4. **Update forum.rs** topic listing query (replace `vote_score` subquery)

5. **Add batch load helper** in `forum_reactions.rs`

6. **Frontend** new `PostReactions.svelte` component

7. **Update topic list cards** to show reaction count instead of vote_score

---

## Performance Considerations

- **Index on `post_id`** covers the main query pattern (reactions for a post)
- **Batch loading** in topic detail avoids N+1 (one query for all posts in page)
- **No denormalized counts** — consistency is more important than micro-optimization at FicHub's scale
- **3-emoji-per-user limit** (app-enforced) caps index size per user

---

## What We're NOT Changing

- `forum_posts.score` — still used for moderation points (Slashdot system)
- `forum_mod_actions` — mod audit trail stays
- `forum_metamod_votes` — metamoderation stays
- `kudos` — work-level kudos, unrelated to forum
- `forum_post_votes` — **dropped** (unused write endpoints, replaced by reactions)
