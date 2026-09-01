# Part 18 — Bounties and Reputation

Bounties let users stake reputation on desirable works ("someone please archive this"). The progression system tracks XP, levels (1-100), and ranks (Finishers, Consistent, Infrequent, Contributors, Average). This part builds both.

---

## 18.1 Bounties: create, list, claim, resolve

Open `src/routes/bounties.rs`:

```rust
// src/routes/bounties.rs (lines 1-45, excerpt)
//! Bounty endpoints. Reputation (spendable) on stake; XP (level ladder) untouched.

use std::sync::Arc;
use axum::extract::{Json, Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::bounties;

#[derive(Debug, Deserialize)]
pub struct BountyListParams {
    pub target_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BountyRow {
    pub id: i32,
    pub creator_id: i32,
    pub target_type: String,
    pub goal_desc: String,
    pub amount: i32,
    pub created_at: String,
}

/// GET /api/bounties — open bounties, optionally filtered by target_type.
pub async fn list_bounties(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BountyListParams>,
) -> Result<Json<Vec<BountyRow>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200).max(1);
    let rows = bounties::list_open_bounties(
        &state.db,
        params.target_type.as_deref(),
        limit,
    )
    .await?;
    Ok(Json(rows.into_iter().map(|(id, creator_id, tt, desc, amt, ts)| BountyRow {
        id, creator_id, target_type: tt, goal_desc: desc, amount: amt, created_at: ts,
    }).collect()))
}
```

### Breakdown

**Bounty row**: id, creator_id, target_type (work/series/tag/meta), goal_desc, amount (reputation staked), created_at.

**List**: Returns open bounties, optionally filtered by target_type, limited to 200.

---

## 18.2 Create a bounty

```rust
// src/routes/bounties.rs (lines 47-76)
#[derive(Debug, Deserialize)]
pub struct CreateBountyReq {
    pub target_type: String,     // work | series | tag | meta
    pub target_ref: String,      // url_id / series id / tag slug / label
    pub amount: i32,             // rep to stake
    pub goal_desc: Option<String>,
    pub expiry_days: Option<i32>,
}

/// POST /api/bounties — create + stake a bounty (authenticated, rep-gated).
pub async fn create_bounty_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateBountyReq>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let valid_targets = ["work", "series", "tag", "meta"];
    if !valid_targets.contains(&req.target_type.as_str()) {
        return Err(AppError::BadRequest("bad target_type".into()));
    }
    let days = req.expiry_days.unwrap_or(30).clamp(1, 90);
    let pot_id = bounties::create_bounty(
        &state.db, user_id, &req.target_type, &req.target_ref,
        req.amount, req.goal_desc.as_deref(), days,
    ).await?;
    Ok(Json(json!({ "ok": true, "bounty_id": pot_id })))
}
```

### Breakdown

**Target types**: work (archive a specific URL), series (archive a series), tag (improve tag coverage), meta (general archive goal).

**Amount**: Reputation staked. This is spendable reputation, not XP. The user must have enough reputation.

**Expiry**: Bounties expire after 1-90 days (default 30).

---

## 18.3 Claim and resolve

```rust
// src/routes/bounties.rs (lines 78-126)
#[derive(Debug, Deserialize)]
pub struct ClaimBountyReq {
    pub claim_ref: String,
}

/// POST /api/bounties/{id}/claim — submit proof of completion.
pub async fn claim_bounty_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(pot_id): Path<i32>,
    Json(req): Json<ClaimBountyReq>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    bounties::claim_bounty_with(&state.db, pot_id, user_id, &req.claim_ref).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Debug, Deserialize)]
pub struct ResolveBountyReq {
    pub action: String,
    pub note: Option<String>,
}

/// POST /api/bounties/{id}/resolve — admin resolves/slashes/cancels.
pub async fn resolve_bounty_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(pot_id): Path<i32>,
    Json(req): Json<ResolveBountyReq>,
) -> Result<Json<Value>, AppError> {
    if auth.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let resolver_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let valid_actions = ["resolved", "slashed", "cancelled"];
    if !valid_actions.contains(&req.action.as_str()) {
        return Err(AppError::BadRequest("bad action".into()));
    }
    let payout = bounties::resolve_bounty(
        &state.db, resolver_id, pot_id, &req.action, req.note.as_deref(),
    ).await?;
    Ok(Json(json!({ "ok": true, "payout": payout })))
}
```

### Breakdown

**Claim**: A user submits proof of completion (e.g., the URL of the archived fic). The claim_ref is the URL or identifier.

**Resolve**: Only admins (role >= 10) can resolve bounties. Actions:
- `resolved` — payout the full amount to the claimant.
- `slashed` — partial payout.
- `cancelled` — no payout (bounty abandoned).

---

## 18.4 XP idle tick

```rust
// src/routes/bounties.rs (lines 128-138)
/// POST /api/xp/idle-tick — tiny XP for authenticated presence (capped by engine).
pub async fn idle_tick_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let total = bounties::idle_tick(&state.db, user_id).await?;
    Ok(Json(json!({ "ok": true, "xp_now": total })))
}
```

---

## 18.5 The progression system

Open `src/services/progression.rs`. This is the XP engine.

### XP sources and caps

```rust
// src/services/progression.rs (lines 21-80, excerpt)
/// Award XP to a user, respecting daily caps.
pub async fn award_xp(
    pool: &PgPool,
    user_id: i32,
    event_type: &str,
    source_ref: Option<&str>,
) -> Result<i64, AppError> {
    // Look up the XP source definition
    let src = sqlx::query_as::<_, (i32, Option<i32>, Option<i64>, Option<i64>, Option<i32>, Option<f64>)>(
        "SELECT xp_amount, daily_cap, cooldown_seconds, rate_limit_per_period, streak_window_days, streak_multiplier FROM xp_source_defs WHERE event_type = $1",
    )
    .bind(event_type)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Unknown event_type: {event_type}")))?;

    let (base_xp, daily_cap, cooldown_seconds, rate_limit, streak_window, streak_mult) = src;

    // Cooldown: block duplicate identical award inside the window
    if let Some(cd) = cooldown_seconds {
        let recent: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - make_interval(secs => $3::bigint)",
        )
        .bind(user_id)
        .bind(event_type)
        .bind(cd)
        .fetch_one(pool)
        .await?;
        if recent > 0 {
            // Still log for audit, but no XP
            sqlx::query(
                "INSERT INTO xp_events (user_id, event_type, xp, source_ref) VALUES ($1, $2, $3, $4)",
            )
            .bind(user_id)
            .bind(event_type)
            .bind(0i32)
            .bind(source_ref)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("xp_events insert (cooldown) failed: {e}")))?;
            return Ok(0);
        }
    }

    // Daily cap
    let mut actual_xp = base_xp as i64;
    if let Some(cap) = daily_cap {
        let today_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - interval '1 day'",
        )
        .bind(user_id)
        .bind(event_type)
        .fetch_one(pool)
        .await?;
        let remaining = cap as i64 - today_count;
        if remaining <= 0 {
            actual_xp = 0;
        } else {
            actual_xp = (base_xp as i64).min(remaining);
        }
    }

    // Weekly per-period cap
    if let Some(rl) = rate_limit {
        let period: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - interval '7 days'",
        )
        .bind(user_id)
        .bind(event_type)
        .fetch_one(pool)
        .await?;
        if period >= rl as i64 {
            actual_xp = 0;
        }
    }

    // Streak multiplier
    if let (Some(window_days), Some(mult)) = (streak_window, streak_mult) {
        // ... streak logic ...
    }

    // Log the event and update level
    // ... (transaction with level-up check)
}
```

### Breakdown

**XP sources**: Each event type (e.g., "work_uploaded", "fic_downloaded", "bookmark_added") has a definition with:
- `xp_amount` — base XP.
- `daily_cap` — max times per day.
- `cooldown_seconds` — min time between identical awards.
- `rate_limit_per_period` — weekly cap.
- `streak_window_days` + `streak_multiplier` — bonus for consecutive days.

**Cooldown**: Prevents spam. If the user earned the same XP type within the cooldown window, they get 0 XP (but the event is still logged for audit).

**Daily cap**: Limits how many times per day the user can earn this XP type.

**Weekly cap**: Limits how many times per week.

**Streak multiplier**: Bonus XP for consecutive days of the same activity.

### Level calculation

The level curve is `50 * level^1.6` cumulative XP. Level 1 → 2 is cheap (fast early progression). Level 99 → 100 is very expensive (50 * 100^1.6 ≈ 200,000 XP).

```rust
// src/progression.rs (conceptual — the progression module)
pub fn xp_for_level(level: i32) -> i64 {
    (50.0 * (level as f64).powf(1.6)) as i64
}

pub fn xp_to_level(xp: i64) -> i32 {
    // Inverse of xp_for_level — find the highest level where xp_for_level(level) <= xp
    // Binary search or direct calculation
    // ...
}

pub fn xp_to_rank(level: i32) -> i16 {
    // 0-19: Average, 20-39: Contributor, 40-59: Infrequent, 60-79: Consistent, 80-100: Finisher
    if level >= 80 { 5 }       // Finisher
    else if level >= 60 { 4 }  // Consistent
    else if level >= 40 { 3 }  // Infrequent
    else if level >= 20 { 2 }  // Contributor
    else { 1 }                 // Average
}
```

### Ranks

| Level | Rank | Title |
|-------|------|-------|
| 0-19 | Average | New reader |
| 20-39 | Contributor | Active reader |
| 40-59 | Infrequent | Regular reader |
| 60-79 | Consistent | Dedicated reader |
| 80-100 | Finisher | Archive finisher |

Ranks are gate types for features — e.g., only Finishers can access certain curator tools.

---

## 18.6 Feature gates

Open `src/routes/progression.rs`:

```rust
// src/routes/progression.rs (lines 49-100, excerpt)
/// GET /api/me/progression — return the current user's level, rank, XP, and recent events.
pub async fn get_progression(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProgressionInfo>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let user_row = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT COALESCE(level, 0), COALESCE(rank, 0), COALESCE(xp, 0) FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let (level, rank, xp) = user_row;

    let events = sqlx::query_as::<_, (String, i32, Option<String>, String)>(
        "SELECT event_type, xp, source_ref, created_at::text
         FROM xp_events
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ProgressionInfo {
        level,
        rank,
        rank_title: rank_title(rank).to_string(),
        xp: xp as i64,
        xp_to_next_level: xp_for_next_level(level),
        recent_events,
    }))
}
```

### Breakdown

**ProgressionInfo**: level, rank, rank_title, xp, xp_to_next_level, recent_events (last 20 XP events).

**rank_title**: Maps rank number to a human-readable title (e.g., "Finisher", "Consistent").

---

## 18.7 User preferences and layout

```rust
// src/routes/progression.rs (lines 102-204, excerpt)
/// GET /api/me/prefs — return all user preferences.
pub async fn get_user_prefs(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserPrefResponse>>, AppError> {
    // ... fetch from user_prefs table ...
}

/// PUT /api/me/prefs — update user preferences.
pub async fn set_user_prefs(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(updates): Json<Vec<PrefUpdate>>,
) -> Result<Json<Value>, AppError> {
    // upsert each pref
}
```

Preferences include reading settings, theme overrides, and other user-specific config. Layouts let users customize dashboard widget positions.

---

## 18.8 Try It Yourself: create a bounty and check progression

### Step 1: Create a bounty

```bash
curl -s -X POST http://localhost:8000/api/bounties \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{"target_type": "work", "target_ref": "https://example.com/fic", "amount": 10, "goal_desc": "Archive this fic", "expiry_days": 30}' | python3 -m json.tool
```

### Step 2: Get your progression

```bash
curl -s http://localhost:8000/api/me/progression \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" | python3 -m json.tool
```

**Expected**:

```json
{
    "level": 1,
    "rank": 1,
    "rank_title": "Average",
    "xp": 0,
    "xp_to_next_level": 50,
    "recent_events": []
}
```

---

## 18.9 What you have now

- You understand bounties: create (stake rep), list, claim (submit proof), resolve (admin: resolved/slashed/cancelled).
- You understand XP: sources with caps/cooldowns/streaks, daily and weekly limits.
- You understand levels: 50*level^1.6 cumulative curve, residual on level-up.
- You understand ranks: 5 ranks based on level, feature gates.
- You understand feature gates: rank/trust/XP gates, prerequisite features.
- You understand user preferences and layouts.
- You tested bounties and progression with curl.

Next: Part 19 — Notifications. You will build the notification bell, notification list with pagination, read-all, unread-count, and granular notification preferences.

---

*End of Part 18. On to [Part 19 — Notifications](./19-notifications/19-notifications.md).*
