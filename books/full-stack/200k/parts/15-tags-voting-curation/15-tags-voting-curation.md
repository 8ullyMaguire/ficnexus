# Part 15 — Tags, Voting, and Curation

> In this chapter you will learn how FicHub's tag system works — how users submit tags on fics, vote on them, flag bad tags, and how curators manage aliases, merges, deletions, and tag search. We'll build each handler from `src/tags/` step by step.

---

## Overview

FicHub's tag system lives in `src/tags/` with 6 files:

| File | Purpose |
|------|---------|
| `routes.rs` (594 lines) | 8 public handlers: submit, vote, flag, get (list tags on fic), resolve, search, tag detail, autocomplete |
| `curator.rs` (426 lines) | 6 curator handlers (role ≥ 10): aliases list/create/delete, merge, delete tag, update tag, flags list, resolve flag |
| `resolve.rs` (168 lines) | Tag resolution: exact match → alias match → create new canonical tag |
| `voting.rs` (150 lines) | Vote recording with `ON CONFLICT DO UPDATE`, trigger-driven score update, hidden threshold check |
| `backfill.rs` (6.3K) | One-time backfill script for schema changes |

The tag model:
- `tags` table: canonical tags (id, name, tag_type_id 1-7, description, is_canonical bool)
- `tag_types` lookup: 1=Character, 2=Relationship, 3=Fandom, 4=Freeform, 5=Category, 6=Rating, 7=Language
- `tag_aliases` table: alias_name → canonical_tag_id (ON CONFLICT DO NOTHING)
- `fic_tags` junction: tag_id × url_id, with `score` (sum of votes) and `added_by_ip`
- `fic_tag_votes` table: per (url_id, tag_id, voter_ip), value -1/1, trigger updates `fic_tags.score`
- `tag_flags` table: curator review queue — flagged tags with reason, resolved bool

---

## Chapter 15.1 — Submitting a Tag

### Goal

Build the tag submission handler — user types a tag, it resolves to canonical (or creates new), gets attached to the fic.

### Actions

#### 1. The submission request body

```rust
// src/tags/routes.rs (lines 14-21)
#[derive(Debug, Deserialize)]
pub struct SubmitBody {
    pub url_id: String,      // fim_info.id (e.g. "abc123def456")
    pub tag_name: String,    // tag as typed by user
    pub tag_type_id: i16,    // 1-7: Character, Relationship, Fandom, Freeform, Category, Rating, Language
}
```

#### 2. The submit handler

```rust
// src/tags/routes.rs (lines 44-108) — POST /api/v0/tags/submit
pub async fn submit_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<SubmitBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();

    // VALIDATE INPUT
    if body.url_id.is_empty() || body.tag_name.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id and tag_name are required"})));
    }
    if !(1..=7).contains(&body.tag_type_id) {
        return Ok(Json(json!({"err": -1, "msg": "tag_type_id must be 1-7"})));
    }

    // RATE LIMIT — Redis-backed, per-IP, per-action
    let mut redis = state.redis.clone();
    check_tag_rate_limit(
        &mut redis, "submit", ip,
        state.config.tag_submit_limit_per_hour,
    ).await?;

    // VERIFY THE FIC EXISTS
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)"
    ).bind(&body.url_id).fetch_one(&state.db).await?;
    if !exists {
        return Ok(Json(json!({"err": -5, "msg": "fic not found"})));
    }

    // RESOLVE THE TAG STRING TO A CANONICAL TAG
    let resolution = resolve::resolve_tag(&state.db, &body.tag_name, body.tag_type_id).await?;

    // ATTACH THE TAG TO THE FIC (IDEMPOTENT)
    let ip_str = ip.to_string();
    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score) VALUES ($1, $2, $3::inet, 0) ON CONFLICT (url_id, tag_id) DO NOTHING"
    ).bind(&body.url_id).bind(resolution.tag_id).bind(&ip_str).execute(&state.db).await?;

    // RESPONSE
    Ok(Json(json!({
        "err": 0, "tag_id": resolution.tag_id,
        "tag_name": resolution.tag_name, "tag_type_id": resolution.tag_type_id,
        "is_new": resolution.is_new,
    })))
}
```

> **⚠️ Watch Out**: `ON CONFLICT (url_id, tag_id) DO NOTHING` — same user submitting same tag on same fic twice is silently ignored. You can't "re-submit" to bump score. Use voting for that.

#### 3. The tag resolution algorithm

```rust
// src/tags/resolve.rs (168 lines) — 3-step resolution

pub struct TagResolution {
    pub tag_id: i32,
    pub tag_name: String,
    pub tag_type_id: i16,
    pub is_new: bool,
}

pub async fn resolve_tag(pool: &PgPool, tag_name: &str, tag_type_id: i16) -> AppResult<TagResolution> {
    // STEP 1: EXACT MATCH — case-insensitive lookup in tags table
    let exact: Option<(i32, String, i16)> = sqlx::query_as(
        "SELECT id, name, tag_type_id FROM tags WHERE LOWER(name) = LOWER($1) AND tag_type_id = $2"
    ).bind(tag_name).bind(tag_type_id).fetch_optional(pool).await?;

    if let Some((tag_id, name, ttype)) = exact {
        return Ok(TagResolution { tag_id, tag_name: name, tag_type_id: ttype, is_new: false });
    }

    // STEP 2: ALIAS MATCH — is this an alias pointing to a canonical tag?
    let alias: Option<(i32, String, i16)> = sqlx::query_as(
        "SELECT ta.canonical_tag_id, t.name, t.tag_type_id FROM tag_aliases ta JOIN tags t ON t.id = ta.canonical_tag_id WHERE ta.alias_name = $1 AND t.tag_type_id = $2"
    ).bind(tag_name).bind(tag_type_id).fetch_optional(pool).await?;

    if let Some((canonical_id, canonical_name, ttype)) = alias {
        return Ok(TagResolution { tag_id: canonical_id, tag_name: canonical_name, tag_type_id: ttype, is_new: false });
    }

    // STEP 3: CREATE NEW — insert a new canonical tag
    let new_tag_id: i32 = sqlx::query_scalar(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id"
    ).bind(tag_name).bind(tag_type_id).fetch_one(pool).await?;

    Ok(TagResolution { tag_id: new_tag_id, tag_name: tag_name.to_string(), tag_type_id, is_new: true })
}
```

> **💡 Key Concept**: "harry potter" (lowercase) resolves to canonical "Harry Potter" if it exists — the user's input is normalized to the canonical form. The 3-step lookup means "jk" could resolve to "Harry Potter" via an alias.

#### 4. Redis rate limiting

```rust
// src/tags/routes.rs (lines 235-281) — check_tag_rate_limit
async fn check_tag_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    action: &str,    // "submit" | "vote" | "flag"
    ip: std::net::IpAddr,
    limit: u32,       // config: tag_submit_limit_per_hour, etc.
) -> AppResult<()> {
    let key = format!("ratelimit:tag:{action}:{ip}");

    let count: Option<u32> = redis::cmd("GET").arg(&key).query_async(redis).await.unwrap_or(None);
    if let Some(c) = count {
        if c >= limit {
            let ttl: u64 = redis::cmd("TTL").arg(&key).query_async(redis).await.unwrap_or(3600);
            return Err(AppError::RateLimited(ttl));
        }
    }

    let new_count: u32 = redis::cmd("INCR").arg(&key).query_async(redis).await?;
    if new_count == 1 {
        redis::cmd("EXPIRE").arg(&key).arg(3600_i64).query_async(redis).await?;
    }
    Ok(())
}
```

Rate-limited responses: `429 {err: 429, msg: "rate limited", retry_after: N}` — client waits `retry_after` seconds.

#### 5. Tag types (1-7)

| ID | Type | Example |
|----|------|---------|
| 1 | Fandom | "Harry Potter" |
| 2 | Character | "Harry Potter" |
| 3 | Relationship | "Harry/Draco" |
| 4 | Freeform | "angst", "fluff", "slow burn" |
| 5 | Category | "Gen", "Romance" |
| 6 | Rating | "G", "T", "M", "Explicit" |
| 7 | Language | "English" |

`src/search/tags.rs` has `get_type_name(type_id)`:

```rust
pub fn get_type_name(type_id: i16) -> &'static str {
    match type_id {
        1 => "fandom", 2 => "character", 3 => "relationship",
        4 => "freeform", 5 => "category", 6 => "rating", 7 => "language", _ => "unknown",
    }
}
```

### Try It Yourself

```bash
curl -X POST /api/v0/tags/submit -H "Content-Type: application/json" \
  -d '{"url_id": "abc123def456", "tag_name": "angst", "tag_type_id": 4}'
# → { "err": 0, "tag_id": 3141, "tag_name": "angst", "tag_type_id": 4, "is_new": true }
```

### Check

- ✅ `tag_type_id` validated 1-7.
- ✅ Rate limited via Redis (per action, per IP, 1-hour TTL).
- ✅ Fic existence checked before attachment.
- ✅ Resolution: EXACT → ALIAS → CREATE NEW (3-step).
- ✅ Attachment idempotent (`ON CONFLICT DO NOTHING`).
- ✅ `is_new` in response.
- ✅ 7 tag types mapped by `get_type_name()`.

### What you built

The tag submission pipeline — validate, check Redis rate limit, verify fic exists, resolve tag (3-step), create new if needed, attach idempotently, return `is_new`.

---

## Chapter 15.2 — Voting on Tags

### Goal

Build the tag voting handler — users upvote/downvote tags, scores aggregate automatically via a PostgreSQL trigger, and low-scoring tags get hidden.

### Actions

#### 1. The vote request body and handler

```rust
// src/tags/routes.rs (lines 23-28, 111-154) — POST /api/v0/tags/vote
#[derive(Debug, Deserialize)]
pub struct VoteBody { pub url_id: String, pub tag_id: i32, pub value: i16 } // 1 or -1

pub async fn vote_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();
    if body.url_id.is_empty() { return Ok(Json(json!({"err": -1, "msg": "url_id is required"}))); }
    if body.value != 1 && body.value != -1 { return Ok(Json(json!({"err": -1, "msg": "value must be 1 or -1"}))); }

    // RATE LIMIT — separate bucket from submit
    let mut redis = state.redis.clone();
    check_tag_rate_limit(&mut redis, "vote", ip, state.config.tag_vote_limit_per_hour).await?;

    // RECORD THE VOTE — trigger on fic_tag_votes adjusts fic_tags.score automatically
    let result = voting::record_vote(&state.db, &body.url_id, body.tag_id, ip, body.value, state.config.tag_hidden_threshold).await?;

    Ok(Json(json!({ "err": 0, "new_score": result.new_score, "hidden": result.hidden })))
}
```

```rust
// src/tags/voting.rs (150 lines) — the record_vote function
pub struct VoteResult { pub new_score: i16, pub hidden: bool }

pub async fn record_vote(
    pool: &PgPool, url_id: &str, tag_id: i32, voter_ip: IpAddr,
    value: i16, hidden_threshold: i16,
) -> AppResult<VoteResult> {
    // INSERT ... ON CONFLICT DO UPDATE — same IP can change their vote
    // Trigger `update_fic_tag_score` on fic_tag_votes adjusts fic_tags.score automatically.
    sqlx::query(
        "INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value) VALUES ($1, $2, $3::inet, $4) ON CONFLICT (url_id, tag_id, voter_ip) DO UPDATE SET value = EXCLUDED.value"
    ).bind(url_id).bind(tag_id).bind(voter_ip).bind(value).execute(pool).await?;

    // Read the new score (updated by trigger)
    let (score,): (i16,) = sqlx::query_as("SELECT score FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
        .bind(url_id).bind(tag_id).fetch_one(pool).await?;

    let hidden = score < hidden_threshold;
    Ok(VoteResult { new_score: score, hidden })
}
```

> **⚠️ Watch Out**: The trigger `update_fic_tag_score` on `fic_tag_votes` runs AFTER INSERT/UPDATE. It recomputes `fic_tags.score` as the SUM of all vote values for that (url_id, tag_id) pair. Application doesn't need to manually update score — it's automatic, transactional, and always consistent.

#### 2. Hidden threshold

Tags with score below `tag_hidden_threshold` (config value) are hidden from `get_tags` listing. Default threshold is typically -3 (needs at least -2 net votes to stay visible).

```rust
// In get_tags handler:
let threshold = state.config.tag_hidden_threshold;
let tags: Vec<Value> = rows.into_iter().map(|(id, name, tag_type_id, score)| {
    json!({ "id": id, "name": name, "tag_type_id": tag_type_id, "score": score,
            "hidden": voting::is_hidden(score, threshold) })
}).collect();
```

The `is_hidden(score, threshold)` function: `score < threshold`. Simple.

### Try It Yourself

```bash
curl -X POST /api/v0/tags/vote -H "Content-Type: application/json" \
  -d '{"url_id": "abc123def456", "tag_id": 3141, "value": 1}'
# → { "err": 0, "new_score": 2, "hidden": false }
```

### Check

- ✅ Vote value restricted to -1 or 1.
- ✅ Rate limited via Redis (separate bucket from submit).
- ✅ `ON CONFLICT (url_id, tag_id, voter_ip) DO UPDATE SET value = EXCLUDED.value` — same IP can change vote.
- ✅ Trigger `update_fic_tag_score` auto-adjusts `fic_tags.score`.
- ✅ `is_hidden` computed from score vs threshold.
- ✅ Response includes both `new_score` and `hidden`.

### What you built

The tag voting system — -1/1 values, rate limited, trigger-driven score aggregation, hidden threshold check.

---

## Chapter 15.3 — Flagging Bad Tags

### Goal

Build the tag flagging handler — users flag tags they think are wrong/spam/inappropriate. Flags go into a curator review queue.

### Actions

#### 1. The flag request body and handler

```rust
// src/tags/routes.rs (lines 30-35, 157-188) — POST /api/v0/tags/flag
#[derive(Debug, Deserialize)]
pub struct FlagBody { pub url_id: String, pub tag_id: i32, pub reason: Option<String> }

pub async fn flag_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<FlagBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();
    if body.url_id.is_empty() { return Ok(Json(json!({"err": -1, "msg": "url_id is required"}))); }
    let ip_str = ip.to_string();

    sqlx::query(
        "INSERT INTO tag_flags (url_id, tag_id, flagged_by_ip, reason) VALUES ($1, $2, $3::inet, $4) ON CONFLICT (url_id, tag_id, flagged_by_ip) DO UPDATE SET reason = EXCLUDED.reason, resolved = FALSE"
    ).bind(&body.url_id).bind(body.tag_id).bind(&ip_str).bind(&body.reason).execute(&state.db).await?;

    Ok(Json(json!({ "err": 0, "msg": "flag submitted" })))
}
```

> **⚠️ Watch Out**: Re-flagging an already-resolved flag resets it to `resolved = FALSE` — forces curator re-review. The `ON CONFLICT DO UPDATE SET reason = EXCLUDED.reason, resolved = FALSE` handles this intentionally.

### Try It Yourself

```bash
curl -X POST /api/v0/tags/flag -H "Content-Type: application/json" \
  -d '{"url_id": "abc123def456", "tag_id": 3141, "reason": "spam tag, should not be here"}'
# → { "err": 0, "msg": "flag submitted" }
```

### Check

- ✅ `url_id` required, `reason` optional.
- ✅ Flag stored in `tag_flags` table with ip, reason, resolved fields.
- ✅ Re-flagging same IP+tag+url updates reason AND resets resolved to false.

### What you built

The tag flagging endpoint — insert flag, optional reason, idempotent re-flag with resolved reset.

---

## Chapter 15.4 — Curator Toolbox: Aliases, Merges, Deletions

### Goal

Build the 6 curator endpoints for managing tags — aliases (alternative spellings → canonical), merges (source → target, migrating all references), deletions (CASCADE), updates (description, tag_type_id).

### Actions

#### 1. Curator access control — role >= 10 (admin only)

```rust
// src/tags/curator.rs (lines 15-28) — require_curator gate
fn require_curator(user: &AuthUser) -> AppResult<()> {
    if user.user_id.is_none() { return Err(AppError::Unauthorized("Login required".into())); }
    if user.role < 10 { return Err(AppError::Forbidden("Curator access required".into())); }
    Ok(())
}
```

All 6 curator endpoints use this gate. Only admin role (10+) can manage tags.

#### 2. Aliases — list, create, delete

```rust
// src/tags/curator.rs (lines 56-133) — 3 endpoints for alias management

// GET /api/curator/aliases — list all alias→canonical mappings
pub async fn list_aliases(State(state): State<Arc<AppState>>, user: AuthUser) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    #[derive(sqlx::FromRow)] struct AliasRow { alias_name: String, canonical_tag_id: i32, canonical_name: Option<String>, tag_type_id: Option<i16>, created_at: chrono::DateTime<chrono::Utc> }
    let rows: Vec<AliasRow> = sqlx::query_as(
        "SELECT ta.alias_name, ta.canonical_tag_id, t.name AS canonical_name, t.tag_type_id, ta.created_at FROM tag_aliases ta LEFT JOIN tags t ON t.id = ta.canonical_tag_id ORDER BY ta.created_at DESC LIMIT 500"
    ).fetch_all(&state.db).await?;
    let aliases: Vec<Value> = rows.into_iter().map(|r| json!({ "alias_name": r.alias_name, "canonical_tag_id": r.canonical_tag_id, "canonical_name": r.canonical_name, "tag_type_id": r.tag_type_id, "created_at": r.created_at.to_rfc3339() })).collect();
    Ok(Json(json!({ "err": 0, "aliases": aliases, "total": aliases.len() })))
}

// POST /api/v0/curator/alias — create alias (idempotent)
pub async fn create_alias(State(state): State<Arc<AppState>>, user: AuthUser, Json(body): Json<CreateAliasBody>) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    if body.alias_name.is_empty() { return Ok(Json(json!({"err": -1, "msg": "alias_name is required"}))); }
    sqlx::query("INSERT INTO tag_aliases (alias_name, canonical_tag_id) VALUES ($1, $2) ON CONFLICT (alias_name) DO NOTHING").bind(&body.alias_name).bind(body.canonical_tag_id).execute(&state.db).await?;
    crate::modlog::record_json(&state.db, user.user_id, user.username.clone(), "create_alias", "tag_alias", &body.alias_name, vec![("canonical_tag_id", json!(body.canonical_tag_id))]).await;
    Ok(Json(json!({ "err": 0, "msg": "alias created" })))
}

// DELETE /api/curator/aliases/{alias_name} — delete alias
pub async fn delete_alias(State(state): State<Arc<AppState>>, user: AuthUser, Path(alias_name): Path<String>) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    let result = sqlx::query("DELETE FROM tag_aliases WHERE alias_name = $1").bind(&alias_name).execute(&state.db).await?;
    if result.rows_affected() == 0 { return Ok(Json(json!({"err": -5, "msg": "alias not found"}))); }
    crate::modlog::record_json(&state.db, user.user_id, user.username.clone(), "delete_alias", "tag_alias", &alias_name, vec![]).await;
    Ok(Json(json!({ "err": 0, "msg": "alias deleted" })))
}
```

#### 3. Merges — source → target, migrate everything

```rust
// src/tags/curator.rs (lines 159-213) — POST /api/v0/curator/merge
pub async fn merge_tags(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<MergeTagsBody>,  // { source_tag_id: i32, target_tag_id: i32 }
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    if body.source_tag_id == body.target_tag_id {
        return Ok(Json(json!({"err": -1, "msg": "cannot merge a tag into itself"})));
    }

    // STEP 1: Migrate fic_tags references (skip duplicates)
    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score) SELECT ft.url_id, $2, ft.added_by_ip, ft.score FROM fic_tags ft WHERE ft.tag_id = $1 ON CONFLICT (url_id, tag_id) DO NOTHING"
    ).bind(body.source_tag_id).bind(body.target_tag_id).execute(&state.db).await?;

    // STEP 2: Migrate tag_flags
    sqlx::query(
        "INSERT INTO tag_flags (url_id, tag_id, flagged_by_ip, reason, resolved) SELECT tf.url_id, $2, tf.flagged_by_ip, tf.reason, tf.resolved FROM tag_flags tf WHERE tf.tag_id = $1 ON CONFLICT (url_id, tag_id, flagged_by_ip) DO NOTHING"
    ).bind(body.source_tag_id).bind(body.target_tag_id).execute(&state.db).await?;

    // STEP 3: Remove old fic_tags rows (CASCADE cleans up votes)
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1").bind(body.source_tag_id).execute(&state.db).await?;

    // STEP 4: Delete the source tag itself
    sqlx::query("DELETE FROM tags WHERE id = $1").bind(body.source_tag_id).execute(&state.db).await?;

    // Record in modlog
    crate::modlog::record_json(&state.db, user.user_id, user.username.clone(), "merge_tags", "tag", &body.source_tag_id.to_string(), vec![("target_tag_id", json!(body.target_tag_id))]).await;
    Ok(Json(json!({ "err": 0, "msg": "tags merged" })))
}
```

> **⚠️ Watch Out**: Merge does 4 operations in sequence: (1) migrate fic_tags, (2) migrate tag_flags, (3) delete fic_tags rows for source, (4) delete source tag. `ON CONFLICT DO NOTHING` on fic_tags migration prevents duplicates when target already has those fic-tag associations.

#### 4. Deletions — CASCADE

```rust
// src/tags/curator.rs (lines 216-237) — DELETE /api/v0/curator/tags/{id}
pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    let result = sqlx::query("DELETE FROM tags WHERE id = $1").bind(id).execute(&state.db).await?;
    if result.rows_affected() == 0 { return Ok(Json(json!({"err": -5, "msg": "tag not found"}))); }
    crate::modlog::record(&state.db, user.user_id, user.username.clone(), "delete_tag", "tag", &id.to_string(), json!({})).await;
    Ok(Json(json!({ "err": 0, "msg": "tag deleted" })))
}
```

> **⚠️ Watch Out**: Deleting a tag is CASCADE — all `fic_tags` and `fic_tag_votes` rows referencing the deleted tag are automatically removed by PostgreSQL's ON DELETE CASCADE. `DELETE FROM tags WHERE id = $1` single query handles everything.

#### 5. Updates — description, tag_type_id (optional: canonical)

```rust
// src/tags/curator.rs (lines 239-343) — PUT /api/curator/tags/{id}
pub async fn update_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateTagBody>,  // { description: Option<String>, tag_type_id: Option<i16>, canonical: Option<bool> }
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    if body.description.is_none() && body.tag_type_id.is_none() && body.canonical.is_none() {
        return Ok(Json(json!({"err": -1, "msg": "nothing to update"})));
    }

    // Verify tag exists (404-style)
    let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM tags WHERE id = $1").bind(id).fetch_optional(&state.db).await?;
    if exists.is_none() { return Ok(Json(json!({"err": -5, "msg": "tag not found"}))); }

    // Validate tag_type_id against tag_types table
    if let Some(type_id) = body.tag_type_id {
        if type_id < 1 { return Ok(Json(json!({"err": -1, "msg": "tag_type_id must be >= 1"}))); }
        let type_ok: Option<(i16,)> = sqlx::query_as("SELECT id FROM tag_types WHERE id = $1").bind(type_id).fetch_optional(&state.db).await?;
        if type_ok.is_none() { return Ok(Json(json!({"err": -1, "msg": format!("unknown tag_type_id {}", type_id)}))); }
    }

    // `canonical` column — probed via information_schema (optional feature)
    let has_canonical: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tags' AND column_name = 'canonical')"
    ).fetch_one(&state.db).await.unwrap_or(false);

    let (description, tag_type_id): (Option<String>, Option<i16>) = if has_canonical {
        let row: (Option<String>, Option<i16>) = sqlx::query_as(
            "UPDATE tags SET description = COALESCE($2, description), tag_type_id = COALESCE($3, tag_type_id), canonical = COALESCE($4, canonical) WHERE id = $1 RETURNING description, tag_type_id"
        ).bind(id).bind(&body.description).bind(body.tag_type_id).bind(body.canonical).fetch_one(&state.db).await?;
        (row.0, row.1)
    } else {
        let row: (Option<String>, Option<i16>) = sqlx::query_as(
            "UPDATE tags SET description = COALESCE($2, description), tag_type_id = COALESCE($3, tag_type_id) WHERE id = $1 RETURNING description, tag_type_id"
        ).bind(id).bind(&body.description).bind(body.tag_type_id).fetch_one(&state.db).await?;
        (row.0, row.1)
    };

    Ok(Json(json!({ "err": 0, "tag": { "id": id, "description": description, "tag_type_id": tag_type_id, "canonical": if has_canonical { body.canonical } else { None } }, "msg": "tag updated" })))
}
```

> **⚠️ Watch Out**: `canonical` is an optional feature — probed via `information_schema.columns` at runtime. If the column doesn't exist, it's silently skipped. This allows schema to evolve without code changes.

### Try It Yourself

```bash
# Merge "Harry Potter (film)" (id=42) into canonical "Harry Potter" (id=3141)
curl -X POST /api/v0/curator/merge -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" -d '{"source_tag_id": 42, "target_tag_id": 3141}'

# Delete a tag
curl -X DELETE /api/v0/curator/tags/42 -H "Authorization: Bearer <admin_token>"

# Update description
curl -X PUT /api/curator/tags/3141 -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" -d '{"description": "J.K. Rowling\'s wizarding world"}'
```

### Check

- ✅ Merge: self-merge rejected; 4-step migration; `ON CONFLICT DO NOTHING` on fic_tags migration.
- ✅ Delete: CASCADE removes all references automatically.
- ✅ Update: validates `tag_type_id` against `tag_types` table.
- ✅ `canonical` column probed via `information_schema` — optional, silently ignored if absent.
- ✅ All operations require role >= 10 (admin).

### What you built

The merge and delete tools — 4-step merge migration, CASCADE delete, tag update with type validation and optional canonical support.

---

## Chapter 15.5 — Curator Flags and Resolutions

### Goal

Understand the flag queue — how curators review flagged tags and mark them as resolved or keep them open.

### Actions

#### 1. List flags

```rust
// src/tags/curator.rs (lines 346-399) — GET /api/v0/curator/flags
pub async fn list_flags(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<FlagListQuery>,  // { resolved: Option<bool> }
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    let resolved_filter = params.resolved.unwrap_or(false);

    #[derive(sqlx::FromRow)] struct FlagRow { id: i64, url_id: String, tag_id: i32, flagged_by_ip: String, reason: Option<String>, resolved: bool, created_at: chrono::DateTime<chrono::Utc> }

    let rows: Vec<FlagRow> = sqlx::query_as(
        "SELECT id, url_id, tag_id, flagged_by_ip::text AS flagged_by_ip, reason, resolved, created_at FROM tag_flags WHERE resolved = $1 ORDER BY created_at DESC"
    ).bind(resolved_filter).fetch_all(&state.db).await?;

    let flags: Vec<Value> = rows.into_iter().map(|r| json!({ "id": r.id, "url_id": r.url_id, "tag_id": r.tag_id, "flagged_by_ip": r.flagged_by_ip, "reason": r.reason, "resolved": r.resolved, "created_at": r.created_at.to_rfc3339() })).collect();
    Ok(Json(json!({ "err": 0, "flags": flags })))
}
```

#### 2. Resolve a flag

```rust
// src/tags/curator.rs (lines 402-426) — POST /api/v0/curator/flags/{id}/resolve
pub async fn resolve_flag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<ResolveFlagBody>,  // { resolved: bool }
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;
    let result = sqlx::query("UPDATE tag_flags SET resolved = $1 WHERE id = $2").bind(body.resolved).bind(id).execute(&state.db).await?;
    if result.rows_affected() == 0 { return Ok(Json(json!({"err": -5, "msg": "flag not found"}))); }
    crate::modlog::record(&state.db, user.user_id, user.username.clone(), "resolve_flag", "tag_flag", &id.to_string(), json!({"resolved": body.resolved})).await;
    Ok(Json(json!({ "err": 0, "msg": "flag updated" })))
}
```

### Try It Yourself

```bash
curl "/api/v0/curator/flags?resolved=false" -H "Authorization: Bearer <admin_token>"
curl -X POST /api/v0/curator/flags/123/resolve -H "Authorization: Bearer <admin_token>" -H "Content-Type: application/json" -d '{"resolved": true}'
```

### Check

- ✅ `list_flags` filters by `resolved` (default false = show unresolved).
- ✅ `resolve_flag` updates resolved state, records in modlog.
- ✅ Returns -5 if flag not found.

### What you built

The flag review tools — list with resolved filter, resolve/un-resolve with modlog recording.

---

## Chapter 15.6 — Tag Resolution, Search, and Autocomplete

### Goal

Understand how FicHub resolves tag strings on submit, searches the tag database with dynamic filters, and provides autocomplete for the search UI.

### Actions

#### 1. Tag resolution recap (submit flow)

```
User submits: { url_id: "abc123", tag_name: "harry potter", tag_type_id: 1 (Fandom) }

1. EXACT: SELECT id, name, tag_type_id FROM tags WHERE LOWER(name) = LOWER('harry potter') AND tag_type_id = 1
   → found "Harry Potter" (id=3141, is_new=false) → done

2. ALIAS (if exact fails): SELECT canonical_tag_id, t.name, t.tag_type_id FROM tag_aliases ta JOIN tags t ON t.id = ta.canonical_tag_id WHERE ta.alias_name = 'harry potter' AND t.tag_type_id = 1
   → not found → continue

3. CREATE NEW: INSERT INTO tags (name, tag_type_id) VALUES ('harry potter', 1) RETURNING id
   → new tag id=4200, is_new=true → done
```

#### 2. Tag search with dynamic filters

```rust
// src/tags/routes.rs (lines 285-430) — GET /api/tags/search?q=&type=&canonical=&fandom=&page=&limit=&sort=
pub async fn search_tags(State(state): State<Arc<AppState>>, Query(params): Query<TagSearchParams>) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * limit;

    // COUNT query (dynamic QueryBuilder)
    let mut count_qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT COUNT(*) FROM tags t WHERE 1=1");
    if let Some(ref q) = params.q { if !q.is_empty() { let lv = format!("%{}%", q); count_qb.push(" AND t.name ILIKE "); count_qb.push_bind(lv); } }
    if let Some(type_id) = params.tag_type { count_qb.push(" AND t.tag_type_id = "); count_qb.push_bind(type_id); }
    if let Some(is_canonical) = params.canonical {
        if is_canonical { count_qb.push(" AND t.id NOT IN (SELECT DISTINCT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)"); }
        else { count_qb.push(" AND t.id IN (SELECT DISTINCT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)"); }
    }
    let total: (i64,) = count_qb.build_query_as().fetch_one(&state.db).await?;

    // SORT clause
    let sort_clause_str = match params.sort.as_deref() {
        Some("usage") => "ORDER BY usage_count DESC",
        Some("created") => "ORDER BY t.created_at DESC NULLS LAST",
        _ => "ORDER BY t.name ASC",
    };

    // DATA query (same filters)
    let mut data_qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT t.id, t.name, t.tag_type_id, t.description, (SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = t.id) as usage_count, EXISTS(SELECT 1 FROM tag_aliases ta WHERE ta.alias_name = t.name) as is_alias FROM tags t WHERE 1=1");
    // ... same filters as count ...
    data_qb.push(" "); data_qb.push(sort_clause_str); data_qb.push(" LIMIT "); data_qb.push_bind(limit as i64); data_qb.push(" OFFSET "); data_qb.push_bind(offset as i64);

    #[derive(sqlx::FromRow)] struct TagSearchRow { id: i32, name: String, tag_type_id: i16, description: Option<String>, usage_count: i64, is_alias: bool }
    let rows: Vec<TagSearchRow> = data_qb.build_query_as().fetch_all(&state.db).await?;

    let tags: Vec<Value> = rows.into_iter().map(|r| json!({ "id": r.id, "name": r.name, "type_id": r.tag_type_id, "type_name": crate::search::tags::get_type_name(r.tag_type_id), "description": r.description, "usage_count": r.usage_count, "is_canonical": !r.is_alias })).collect();
    Ok(Json(json!({ "err": 0, "total": total.0, "page": params.page.unwrap_or(1), "limit": limit, "results": tags })))
}
```

#### 3. Tag autocomplete

```rust
// src/tags/routes.rs (lines 523-578) — GET /api/tags/autocomplete?q=&type=&limit=
pub async fn tag_autocomplete(State(state): State<Arc<AppState>>, Query(params): Query<TagAutocompleteParams>) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(10).min(25);
    let query = params.q.unwrap_or_default();
    if query.len() < 2 { return Ok(Json(json!({ "err": 0, "results": [] }))); }
    let like_pattern = format!("{}%", query);

    let rows = if let Some(type_id) = params.tag_type {
        sqlx::query_as::<_, (i32, String, i16, i64)>(
            "SELECT t.id, t.name, t.tag_type_id, (SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = t.id) as usage_count FROM tags t WHERE t.name ILIKE $1 AND t.tag_type_id = $2 AND t.id NOT IN (SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name) ORDER BY usage_count DESC LIMIT $3"
        ).bind(&like_pattern).bind(type_id).bind(limit as i64).fetch_all(&state.db).await?
    } else {
        sqlx::query_as::<_, (i32, String, i16, i64)>(
            "SELECT t.id, t.name, t.tag_type_id, (SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = t.id) as usage_count FROM tags t WHERE t.name ILIKE $1 AND t.id NOT IN (SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name) ORDER BY t.tag_type_id, usage_count DESC LIMIT $2"
        ).bind(&like_pattern).bind(limit as i64).fetch_all(&state.db).await?
    };

    Ok(Json(json!({ "err": 0, "results": rows.into_iter().map(|(id, name, type_id, count)| json!({ "id": id, "name": name, "type": crate::search::tags::get_type_name(type_id), "type_id": type_id, "usage_count": count })).collect::<Vec<_>>() })))
}
```

> **⚠️ Watch Out**: Autocomplete requires at least 2 characters. Filter `t.id NOT IN (SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)` excludes canonical tags that are themselves aliases for other tags — prevents duplicate entries. `ILIKE` for case-insensitive prefix matching.

#### 4. Tag detail endpoint

```rust
// src/tags/routes.rs (lines 443-521) — GET /api/tags/{id}
pub async fn get_tag_detail(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    #[derive(sqlx::FromRow)] struct TagDetailRow { id: i32, name: String, tag_type_id: i16, description: Option<String>, created_at: Option<chrono::DateTime<chrono::Utc>> }

    let tag: Option<TagDetailRow> = sqlx::query_as("SELECT id, name, tag_type_id, description, created_at FROM tags WHERE id = $1").bind(id).fetch_optional(&state.db).await?.ok_or_else(|| AppError::NotFound("tag not found".into()))?;

    // Synonyms — aliases pointing TO this canonical tag
    let synonyms: Vec<String> = { let rows: Vec<(String,)> = sqlx::query_as("SELECT alias_name FROM tag_aliases WHERE canonical_tag_id = $1").bind(id).fetch_all(&state.db).await?; rows.into_iter().map(|r| r.0).collect() };

    // Usage count
    let usage_count: i64 = { let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = $1").bind(id).fetch_one(&state.db).await?; count.0 };

    // Is this tag itself an alias target?
    let is_alias: bool = { let res: Option<(i32,)> = sqlx::query_as("SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = (SELECT name FROM tags WHERE id = $1)").bind(id).fetch_optional(&state.db).await?; res.is_some() };

    Ok(Json(json!({ "err": 0, "tag": { "id": tag.id, "name": tag.name, "type_id": tag.tag_type_id, "type_name": crate::search::tags::get_type_name(tag.tag_type_id), "description": tag.description, "created_at": tag.created_at, "is_canonical": is_alias, "synonyms": synonyms, "usage_count": usage_count } })))
}
```

### Try It Yourself

```bash
curl "/api/tags/search?q=harry&type=1&canonical=true&page=1&limit=20"
curl "/api/tags/autocomplete?q=har&type=1"
curl /api/tags/3141
```

### Check

- ✅ Search: dynamic `QueryBuilder` with optional `q`, `type`, `canonical`, `fandom` filters.
- ✅ `usage_count` computed via correlated subquery on `fic_tags`.
- ✅ `is_canonical` computed via `NOT IN (SELECT ...)` on `tag_aliases`.
- ✅ Autocomplete: min 2 chars, `ILIKE` prefix match, excludes aliases.
- ✅ Tag detail: synonyms (aliases pointing TO this canonical), usage count, is_alias check.

### What you built

The full tag resolution, search, and autocomplete system — dynamic `QueryBuilder` for search with multiple filters, `ILIKE` prefix match for autocomplete, tag detail with synonyms and usage count, and the 3-step resolution algorithm.

---

## Conclusion

You now understand FicHub's complete tag system end-to-end:

1. **Submission** — 3-step resolution (exact → alias → create), Redis rate limit, fic existence check, idempotent attachment, `is_new` flag.
2. **Voting** — 1/-1 values, rate limited, trigger-driven score aggregation (`update_fic_tag_score`), hidden threshold.
3. **Flagging** — insert with reason, idempotent re-flag with resolved reset, curator review queue.
4. **Curator tools (6 endpoints)** — aliases (list/create/delete), merges (4-step migration), deletions (CASCADE), updates (validate type, optional canonical column), flags list/resolve with modlog recording.
5. **Search + autocomplete** — dynamic `QueryBuilder` with optional filters (q, type, canonical), `usage_count` via correlated subquery, `ILIKE` prefix match, min 2 chars, alias exclusion.
6. **Tag resolution on submit** — 3-step algorithm (exact → alias → create new), idempotent attachment.

The tag system covers every feature: submit, vote, flag, curator management (aliases, merges, deletions, updates, flags), search with filters, autocomplete for the UI, and tag detail with synonyms and usage count.
