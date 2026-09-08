# Phase 9 — Backlog Execution Plan (post-Phase 8 audit)

**Status:** proposed
**Owner:** backend + frontend + ops
**Scope:** All open TODOs from the Phase 5–8 second-opinion audit (plan.md §9), plus the uncommitted author-batch-download working tree, plus the pending commit of Lane 6 UI that was noted at review time.

This plan consolidates every open item from the Phase 8 audit, the Phase 4 verification section (§4), and the author-batch-download hardening plan into a single sequenced backlog. Each task is written so a junior dev can execute it without additional archaeology.

---

## 0. Prerequisites (read before starting)

1. **Repo root:** `/home/alvaro/code/rust/ficnexus` (symlinked via `/personal/...` on the build server).
2. **Build/test commands:**
   - Backend: `cd /home/alvaro/code/rust/ficnexus && cargo build --release`
   - Backend tests: `set -a; . ./.env; set +a; cargo test --test <name> -- --include-ignored --test-threads=1`
   - Frontend: `cd /home/alvaro/code/rust/ficnexus/frontend && npm run build`
   - Frontend tests: `cd frontend && npx svelte-check && npm test`
   - Frontend E2E: `cd frontend && npm run test:e2e`
   - Release build: `cargo build --release` (output to `~/.cargo-target/release/`)
   - SQLx prepare: `cargo sqlx prepare` (after query changes, needs live DB)
3. **Deploy:** `cargo build --release && sudo systemctl restart fichub` (never check local system for deployment status — SSH to production ThinkCentre).
4. **Conventions** (from `docs/AGENTS.md`):
   - Auth-required endpoints return HTTP 400 with body `{"err":401}` (not 401).
   - Role-gated endpoints return HTTP 400 with body `{"err":403}`.
   - axum 0.8 forbids same-path different-param-name routes.
   - Route page tests must be named `page.test.ts` (not `+page.test.ts`).
   - DB-gated test suites each hold their own mutex; run suites individually for deterministic results.
5. **The plan file itself:** `docs/specs/forum-nodebb/plan.md` — read §9 before implementing any §9 finding. Each task below references the exact §9 finding it closes.

---

## 1. Phase 5 — Realtime producers + SSE fix + JWT consolidation

**Priority:** highest — the entire realtime infrastructure (Phase 5) is shipped but **does nothing**. Notifications, forum replies, DMs never reach connected clients.

**Files to touch:**
- `src/realtime/ws.rs`
- `src/realtime/sse.rs`
- `src/routes/notifications.rs` (producer side — `create_notification`)
- `src/routes/forum.rs` (reply events → `topic:{id}`)
- `src/routes/messages.rs` (message events → `user:{id}`)
- `src/routes/forum_polls.rs` (poll close → voter channels)
- `src/server.rs` (thread AppState's JWT secret into realtime handlers)

### 1.1 Fix the SSE busy-wait (§9.1 #4)

**Problem:** `SseStream::poll_next` calls `self.rx.try_recv()`, and on `Empty` calls `cx.waker().wake_by_ref()` then returns `Poll::Pending` — a 100%-CPU spin per connected SSE client.

**Fix:** Replace the manual `Stream` impl with `tokio_stream::wrappers::BroadcastStream`. The `broadcast::Receiver` already implements `Stream` via `tokio_stream`; `BroadcastStream` adds proper waker registration and handles `Lagged` by skipping.

**Steps:**
1. In `src/realtime/sse.rs`, import `tokio_stream::wrappers::BroadcastStream` and `tokio_stream::StreamExt`.
2. Replace the `SseStream` struct usage: `Sse::new(BroadcastStream(merged_rx))`.
3. Remove the `SseStream` struct entirely (it's only used here).
4. `cargo check` + run any SSE-related test if present (none currently exist — see 1.3).
5. Verify no custom `Stream` impl remains for SSE.

**Test:** Add a comment or assertion; full test coverage comes in 1.3.

### 1.2 Wire server-side producers into the realtime bus (§9.1 #1)

**Problem:** Nothing calls `manager.publish()` or `publish_to_redis()`. The bus only carries client-to-client messages.

**Decision:** Two publish paths per event type:
- **Local fanout** via `manager.publish()` (immediate delivery to in-process subscribers).
- **Cross-instance** via `publish_to_redis()` (delivered to other instances via the `ficnexus:rt` pubsub listener already started in `server.rs`).

Both should be called; `publish_to_redis` is best-effort (wrap in `let _ =`).

**Step 1.2a: Create a shared publish helper**

Add to `src/realtime/mod.rs`:

```rust
use super::{ConnectionManager, RealtimeMessage};

/// Publish a realtime message to the local bus + Redis (cross-instance).
pub async fn publish_event(
    manager: &ConnectionManager,
    redis: Option<&redis::Client>,
    channel: &str,
    event: &str,
    data: serde_json::Value,
) {
    let msg = RealtimeMessage {
        channel: channel.to_string(),
        event: event.to_string(),
        data,
    };
    // Local in-process delivery
    manager.publish(&msg).await;
    // Cross-instance fanout (best-effort)
    if let Some(r) = redis {
        let _ = crate::realtime::pubsub::publish_to_redis(r, &msg).await;
    }
}
```

**Step 1.2b: Make the redis client accessible from handlers**

In `src/server.rs`, store `redis_client` in `AppState` (currently it's only used locally to start the listener). Add a field:

```rust
pub redis_client: Option<redis::Client>,
```

Set it at construction (it's already created as `redis_client` in `main()`):

```rust
redis_client: Some(redis_client),
```

**Step 1.2c: Notification producer → `user:{id}` + SSE event-stream**

In `src/db/queries/social.rs` (line ~101 — the existing producer pattern), after inserting a notification row, publish to the user's personal channel. The current code inserts into `notifications` but never notifies subscribers.

Find the `create_notification` function (or wherever notifications are inserted) and add after the INSERT:

```rust
let _ = crate::realtime::mod::publish_event(
    &state.rt_manager,
    state.redis_client.as_ref(),
    &format!("user:{}", recipient_id),
    "notification_new",
    json!({ "notification_type": notif_type, "title": title, "link": link }),
).await;
```

You'll need access to `state` in the function — trace the call chain and pass `&Arc<AppState>` or the relevant fields.

**Step 1.2d: Forum reply → `topic:{id}`**

In `src/routes/forum.rs`, find the reply handler (`POST /api/forum/topics/{id}/posts`). After creating a post and sending notifications, publish a `topic_reply` event:

```rust
let _ = crate::realtime::mod::publish_event(
    &state.rt_manager,
    state.redis_client.as_ref(),
    &format!("topic:{}", topic_id),
    "topic_reply",
    json!({ "post_id": post_id, "author_id": user_id, "created_at": created_at }),
).await;
```

**Step 1.2e: New DM → `user:{id}`**

In `src/routes/messages.rs`, find the `send_message` handler. After the message is committed and notifications are sent (line ~464), publish a `message_new` event to each recipient's personal channel:

```rust
for member_id in &members {
    let _ = crate::realtime::mod::publish_event(
        &state.rt_manager,
        state.redis_client.as_ref(),
        &format!("user:{}", member_id),
        "message_new",
        json!({ "room_id": room_id, "message_id": msg_id, "author_id": sender_id }),
    ).await;
}
```

**Step 1.2f: Poll close → voter channels**

In `src/routes/forum_polls.rs`, find `close_poll`. After the close and voter notifications, publish a `poll_closed` event to the topic channel:

```rust
let _ = crate::realtime::mod::publish_event(
    &state.rt_manager,
    state.redis_client.as_ref(),
    &format!("topic:{}", topic_id),
    "poll_closed",
    json!({ "poll_id": poll_id }),
).await;
```

### 1.3 Authorization on channel subscribe/publish (§9.1 #2)

**Problem:** Any WS client (including anon) can `subscribe` to any channel and `publish` to any channel — including other users' `user:{id}` channels.

**Fix:** Add authorization to `ws.rs` `handle_socket`:

- **`publish`**: Remove client `publish` entirely. Only server-side code should publish. Delete the `"publish"` arm from the match in `handle_socket` (lines ~105–117).
- **`subscribe`**: Validate the channel:
  - `user:{id}` — only the owning session may subscribe (`user_id` from JWT == `{id}`).
  - `topic:{id}` — check the topic is publicly readable OR the user is a member (reuse `can_view_topic` or equivalent).
  - `room:{id}` — check `forum_room_members` for active membership.
  - `global` — allowed for everyone.
  - Unknown/other patterns — reject.

**Implementation:**

In `handle_socket`, change the `"subscribe"` arm:

```rust
"subscribe" => {
    if let Some(channel) = cmd.get("channel").and_then(|c| c.as_str()) {
        if authorize_channel(channel, user_id, &state.db).await {
            let ch = channel.to_string();
            let rx = manager2.subscribe(&ch).await;
            subs_recv.write().await.push((ch, rx));
        } else {
            // Optionally send an error event back to the client
            let _ = sender.send(Message::Text(
                json!({"event": "subscribe_denied", "channel": channel}).to_string().into()
            )).await;
        }
    }
}
```

Add the `authorize_channel` function:

```rust
async fn authorize_channel(channel: &str, user_id: Option<i32>, db: &sqlx::PgPool) -> bool {
    if channel == "global" {
        return true;
    }
    if let Some((prefix, id_str)) = channel.split_once(':') {
        if let Ok(id) = id_str.parse::<i32>() {
            return match prefix {
                "user" => user_id == Some(id),
                "topic" => crate::routes::forum::can_view_topic(db, id, user_id).await,
                "room" => crate::routes::messages::is_room_member(db, id, user_id).await,
                _ => false,
            };
        }
    }
    false
}
```

You may need to add `can_view_topic` and `is_room_member` if they don't exist. Check `forum.rs` for an existing visibility check function — if none, write a simple query: topic visible if `is_hidden = false AND deleted_at IS NULL` (or the user is a mod/staff).

### 1.4 JWT secret consolidation (§9.1 #5)

**Problem:** Three sources of the JWT secret: `ws.rs` and `sse.rs` fall back to `"fichub-dev-secret"`, `auth.rs` has its own fallback, and `config.rs` defines the real one.

**Fix:** Thread `AppState`'s configured JWT secret through to realtime handlers, just as `forum.rs` already uses `AuthUser` from `auth.rs`.

1. In `src/config.rs`, find where `jwt_secret` is loaded (it's already there per the audit).
2. In `src/server.rs` `AppState`, add a `jwt_secret: String` field, populated from `config.jwt_secret`.
3. In `ws.rs` and `sse.rs`, replace `std::env::var("JWT_SECRET")...unwrap_or("fichub-dev-secret")` with `state.jwt_secret.clone()` (passed through `State`).
4. Ensure `auth.rs`'s `verify_token` also reads from the same centralized location if it doesn't already.

**Note:** `ws.rs` currently reads `State(state)` for `rt_manager` — if it also needs the secret, it already has `state`. If `sse.rs` doesn't get `State`, add it.

### 1.5 Write integration test for realtime (§9.1 #3)

**File:** `tests/realtime_api.rs`

**Tests to write (DB-gated, serial, same conventions as `tests/forum_api.rs`):**

1. `rt_anon_connect_receives_welcome` — connect via SSE without a token → first event is `connected` with `user_id: null`.
2. `rt_subscribe_to_other_user_denied` — connect as user A, try to subscribe to `user:{B}` → subscription denied (no message arrives on that channel; optionally an error event).
3. `rt_server_publish_delivered_on_ws_and_sse` — connect via WS as user A; have the test harness publish directly to `manager.publish()` on `user:{A}` channel; assert the message arrives on both a WS client and an SSE client.
4. `rt_token_auth_works` — connect with a valid `?token=<jwt>` → `connected` event shows the correct `user_id`.

Use the same `app()` router builder pattern from `tests/forum_api.rs`. For publishing, you may need access to `state.rt_manager` — construct `AppState` directly in the test and call `manager.publish` from the test process, or add a test-only helper.

```cargo
// Run: set -a; . ./.env; set +a; cargo test --test realtime_api -- --include-ignored --test-threads=1
```

---

## 2. Phase 8 — Gamification bug fixes (§9.4)

### 2.1 Unify the two XP ledgers (§9.4 #1)

**Problem:** Phase 8d writes `exp_events` + `users.exp` (via `forum.rs::award_exp`), while the progression service writes `xp_events` + `users.xp` (via `services/progression.rs::award_xp` / `apply_xp_and_level_up`). Same concept, different tables, different balance columns. AuthorCard reads `exp`; rest of site reads `xp`.

**Decision:** Unify on `xp_events` / `users.xp` (the progression service's ledger). The `exp_events` table is only written by Phase 8d and read by Phase 8a/8d profile/xp-history endpoints. Migrate by:
1. Making `forum.rs::award_exp` delegate to `services::progression::award_xp` instead of writing `exp_events` directly.
2. Updating `user_profile` and `user_xp_history` to read from `xp_events` / `users.xp` instead of `exp_events` / `users.exp`.
3. Backfilling `exp_events` rows into `xp_events` via a SQL migration (`INSERT INTO xp_events SELECT ... FROM exp_events`).

**Steps:**

**Step 2.1a: Delegate `award_exp` to the progression service**

Replace the body of `award_exp` in `forum.rs` (line 98) to call the progression service:

```rust
async fn award_exp(
    db: &sqlx::PgPool,
    user_id: i32,
    amount: i64,
    event_type: &str,
    reference_type: Option<&str>,
    reference_id: Option<i64>,
) {
    let source_ref = match (reference_type, reference_id) {
        (Some(rt), Some(rid)) => Some(format!("{}:{}", rt, rid)),
        (Some(rt), None) => Some(rt.to_string()),
        _ => None,
    };
    // Delegate to the unified progression service
    let _ = crate::services::progression::award_xp(
        db, user_id, event_type, source_ref.as_deref(),
    ).await;
}
```

**Caveat:** The progression service's `award_xp` reads `xp_source_defs` for `event_type` and daily caps. The `exp_events` event types (`forum_post_create`, `post_reacted`, `mod_received`, `poll_voted`) must be added as rows in `xp_source_defs`. Add them via a migration (see Step 2.1d).

**Step 2.1b: Add forum-specific XP source defs**

Create migration `migrations/090_forum_xp_sources.sql`:

```sql
INSERT INTO xp_source_defs (event_type, xp_amount, daily_cap, description)
VALUES
  ('forum_post_create', 2, 100,  'Forum post or topic created'),
  ('post_reacted',       5, 50,   'Post received a reaction (upvote equivalent)'),
  ('mod_received',       1, 3,    'Positive moderation action on own post'),
  ('poll_voted',         3, 30,   'Voted on a forum poll')
ON CONFLICT (event_type) DO UPDATE
  SET xp_amount = EXCLUDED.xp_amount,
      daily_cap = EXCLUDED.daily_cap,
      description = EXCLUDED.description;

-- Backfill exp_events into xp_events for historical continuity
INSERT INTO xp_events (user_id, event_type, xp, source_ref, created_at)
SELECT user_id, event_type, amount, 
       CONCAT_WS(':', reference_type, reference_id),
       created_at
FROM exp_events
ON CONFLICT DO NOTHING;

-- Sync users.exp → users.xp for all users (one-time)
UPDATE users SET xp = users.exp WHERE users.exp != users.xp;
```

**Step 2.1c: Update profile/xp-history endpoints**

In `user_profile` (line 3840):
- Change SELECT to use `users.xp`, `users.level`, `users.rank` (already has these — it already selects `level, exp, xp, rank`).
- Remove `email` from the SELECT and the JSON response (see 2.3).
- Map `"reputation"` to the actual `reputation` column (see 2.4).
- Use `users.xp` for progress calculation, not `users.exp`.

In `user_xp_history` (line 3882):
- Change `FROM exp_events` to `FROM xp_events`.
- Map `xp` column (not `amount`).
- Update the today_by_type query similarly.

**Step 2.1d: Test**

Add to `tests/forum_api.rs` (or a new `tests/forum_xp_api.rs`):
- `forum_xp_awarded_on_post` — create a post as user A → assert `xp_events` has a `forum_post_create` row with 2 XP.
- `forum_xp_daily_cap_enforced` — create 51 posts in one day → assert only 100 XP awarded (cap).
- `forum_xp_progress_from_xp_column` — assert `user_profile` returns `xp` field matching `users.xp`, not `users.exp`.

### 2.2 Fix XP farming on reaction toggles (§9.4 #2)

**Problem:** `award_reaction_exp` fires unconditionally after the react toggle in `add_post_reaction` (line 3695). Un-react + re-react = repeated XP awards (only the daily count cap limits it). Same for `award_poll_vote_exp`.

**Decision:** Award only on the 0→1 transition (actual insertion of a new reaction, not a toggle cycle). The `add_post_reaction` handler already tracks `deleted` (rows_affected from DELETE + INSERT). Only call `award_reaction_exp` when a new reaction was actually inserted.

**Steps:**

**Step 2.2a: `add_post_reaction` (forum.rs, line ~3683–3695)**

Currently:
```rust
if deleted == 0 {
    sqlx::query("INSERT ... ON CONFLICT DO NOTHING")...
}
award_reaction_exp(&state.db, user_id, post_id).await;
```

Change to only award when the INSERT actually happened (not when it was a toggle that resulted in no change):

```rust
let inserted_new = if deleted == 0 {
    sqlx::query("INSERT ... ON CONFLICT DO NOTHING")
        .bind(post_id).bind(user_id).bind(&emoji)
        .execute(&state.db)
        .await?
        .rows_affected() > 0
} else {
    false  // was a new reaction after delete, so it IS new
};
if inserted_new {
    award_reaction_exp(&state.db, user_id, post_id).await;
}
```

Wait — actually the current logic is: try DELETE first; if nothing was deleted (the reaction wasn't there), INSERT. So `deleted == 0` means the user is adding a reaction for the first time. The bug is that `award_reaction_exp` is called regardless of whether the INSERT succeeded or hit `ON CONFLICT DO NOTHING` (race) or the reaction was already there.

The cleanest fix: check the INSERT's `rows_affected()`:

```rust
let did_insert = if deleted == 0 {
    sqlx::query("INSERT ... ON CONFLICT DO NOTHING")
        .bind(post_id).bind(user_id).bind(&emoji)
        .execute(&state.db)
        .await?
        .rows_affected() > 0
} else {
    // Reaction was toggled off then on — this IS a new reaction
    true
};
if did_insert {
    award_reaction_exp(&state.db, user_id, post_id).await;
}
```

**Step 2.2b: `award_poll_vote_exp` (forum_polls.rs, line 342)**

Find the vote handler. It currently calls `award_poll_vote_exp` unconditionally after a vote. Find where the vote INSERT happens and only call the XP award if a new row was inserted.

Check `vote_on_poll` in `forum_polls.rs` — look for the INSERT into `forum_poll_votes` (or equivalent) and gate the XP call on `rows_affected() > 0`.

**Step 2.2c: Add a dedup safety net in `award_exp` (belt + suspenders)**

Even with the transition gating, add a unique constraint check in `award_exp`: if `reference_type` and `reference_id` are passed (they will be after the unification in 2.1), skip if a row already exists for that user+type+ref within the same day.

The `award_exp` function in forum.rs currently does NOT use reference_type/reference_id for dedup (it just inserts). Update the INSERT:

```sql
INSERT INTO xp_events (user_id, event_type, xp, source_ref, created_at)
VALUES ($1, $2, $3, $4, NOW())
ON CONFLICT (user_id, event_type, date_trunc('day', created_at)) DO NOTHING;
```

This requires a unique index:
```sql
CREATE UNIQUE INDEX IF NOT EXISTS xp_events_user_event_day 
ON xp_events (user_id, event_type, date_trunc('day', created_at));
```

Add this to migration `090_forum_xp_sources.sql`. This is the cleanest dedup — daily per-event-type caps prevent farming even if the calling code has race conditions.

### 2.3 Remove `email` from profile endpoint (§9.4 #3)

**File:** `src/routes/forum.rs`, `user_profile` function (line 3836).

**Steps:**
1. Remove `email` from the SELECT tuple at line 3841.
2. Remove `email` from the destructured variables at line 3849.
3. Remove `"email": email` from the JSON response at line 3863.

This is a one-line delete from each of those three places. No migration needed — `email` stays on the `users` table, just not exposed.

### 2.4 Fix `reputation` returning `trust` (§9.4 #4)

**File:** `src/routes/forum.rs`, `user_profile` (line 3869).

**Problem:** The SELECT omits the `reputation` column. The JSON maps `"reputation": trust`. So every user's reputation shows as their trust score.

**Fix:**
1. Add `reputation` to the SELECT at line 3841: `SELECT id, username, level, exp, xp, rank, trust, reputation, created_at, ...`
2. Add `reputation` to the tuple type and destructuring (line 3840, 3849).
3. Change `"reputation": trust` to `"reputation": reputation` (line 3869).

### 2.5 Align envelope format + track achievements (§9.4 #5)

**Problem:** §8.1 spec'd `{err:0,…}` envelopes; Phase 8 endpoints return bare JSON. §8.4's "achievement notification via WebSocket" never landed.

**Decision:** Align the Phase 8 endpoints to return `{err:0, data: ...}` envelopes, matching the rest of the API. Track the missing achievement-notification feature as an explicit deferred item in the plan.

**Steps:**

**Step 2.5a: Wrap profile/xp-history/widget endpoints in `{err:0, data:...}`**

In `user_profile`, `user_xp_history`, `widget_recent_topics`, `widget_popular_topics`, `widget_stats`:
- Wrap the `json!({...})` in `json!({"err": 0, "data": ...})`.

**Step 2.5b: Add achievement endpoint stub**

§8.4 specified `GET /api/forum/users/{user_id}/achievements`. The schema says achievements are stored in `user_features` (feature_type='achievement'). Register the route but leave the handler returning a documented empty list until the achievement-tracking logic is implemented:

In `src/server.rs`, add:
```rust
.route(
    "/api/forum/users/{userId}/achievements",
    get(crate::routes::forum::user_achievements),
)
```

In `src/routes/forum.rs`, add:
```rust
pub async fn user_achievements(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<crate::server::AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let achievements = sqlx::query_as::<_, (String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT f.slug, f.name, uf.unlocked_at 
         FROM user_features uf JOIN features f ON f.id = uf.feature_id 
         WHERE uf.user_id = $1 AND f.feature_type = 'achievement' 
         ORDER BY uf.unlocked_at DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    
    Ok(axum::Json(serde_json::json!({
        "err": 0,
        "data": {
            "achievements": achievements.into_iter().map(|(slug, name, unlocked_at)| {
                serde_json::json!({
                    "type": slug,
                    "name": name,
                    "unlocked_at": unlocked_at,
                })
            }).collect::<Vec<_>>(),
        }
    })))
}
```

This makes the endpoint exist and return the right shape. The actual achievement-unlocking logic (what triggers unlocks) is deferred — document this.

---

## 3. Phase 7 — NodeBB importer hardening (§9.3)

**File:** `crates/forum-import/src/import.rs`, `crates/forum-import/src/main.rs`, `crates/forum-import/README.md`

### 3.1 Add empty-DB precondition guard

**Problem:** Importer inserts NodeBB IDs explicitly with `ON CONFLICT DO NOTHING`. On a non-empty DB, collisions silently skip, orphaning children.

**Fix:** Before any import phase, check that the target tables are empty. If not empty and `--force` was not passed, abort with an informative message.

**Steps:**

In `import.rs`, add a precondition check at the start of `run_import`:

```rust
async fn check_preconditions(pool: &PgPool, dry_run: bool) -> Result<()> {
    let (user_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool).await?;
    let (topic_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM forum_topics")
        .fetch_one(pool).await?;
    
    if user_count > 0 || topic_count > 0 {
        anyhow::bail!(
            "Import precondition failed: target DB is not empty \
             (users: {}, topics: {}). \
             NodeBB import uses explicit IDs and requires an empty database. \
             Re-run with --force to override (WARNING: will not remap IDs, \
             collisions will be silently skipped).",
            user_count, topic_count
        );
    }
    Ok(())
}
```

Call it at the top of `run_import` (before Phase 1). Add `--force` to the CLI struct in `main.rs` and pass it through.

Update `README.md` to document: "This importer must run against an empty database. It uses NodeBB's explicit IDs for users, categories, topics, and posts. Running against a populated DB will silently skip conflicting IDs, orphaning children. Use `--force` only on a confirmed-empty or intentionally-idempotent database."

### 3.2 Wrap import in a single transaction (or batched transactions)

**Problem:** Inserts are autocommit per statement. A crash mid-import strands partial data, and skip counters make a re-run look idempotent when it isn't.

**Fix:** Wrap the entire import in a single transaction. On any error, the transaction rolls back and nothing is persisted.

**Steps:**

In `import.rs`, modify `run_import`:

```rust
pub async fn run_import(pool: &PgPool, data: &ImportFile, dry_run: bool, force: bool) -> Result<ImportStats> {
    if !dry_run {
        check_preconditions(pool, force).await?;
    }
    
    if dry_run {
        // dry-run path stays the same (no transaction needed)
        ...
    }
    
    let mut tx = pool.begin().await?;
    let stats = do_import(&mut tx, data).await?;
    tx.commit().await?;
    Ok(stats)
}
```

The `import_user`, `import_category`, `import_topic`, `import_post` functions all take `&PgPool` — change them to take `&mut sqlx::Transaction<'_, sqlx::Postgres>`.

**Note:** The `notifications` phase (Phase 5) also needs to be inside the transaction.

### 3.3 Add DB-backed import test

**Problem:** 10 unit tests cover parsing/stats only; the INSERT path (419 lines of `import.rs`) is untested.

**Fix:** Add one integration test using `#[sqlx::test]` (or `#[tokio::test]` with a test DB) that imports a tiny fixture and asserts rows land correctly.

**File:** `crates/forum-import/tests/import_integration.rs`

```rust
use crate::schema::ImportFile;

#[sqlx::test]
async fn test_import_small_fixture(pool: sqlx::PgPool) -> anyhow::Result<()> {
    // Run migrations on the test pool first
    // (sqlx::test macro handles this if MIGRATOR is set)
    
    let json = r#"{
        "users": [{"uid": 1, "username": "alice", "email": "a@b.com", "joindate": 1672531200000}],
        "categories": [{"cid": 1, "name": "General", "slug": "general"}],
        "topics": [{"tid": 100, "cid": 1, "uid": 1, "title": "Hello", "timestamp": 1672531200000}],
        "posts": [{"pid": 1000, "tid": 100, "uid": 1, "content": "Hi!", "timestamp": 1672531200000}]
    }"#;
    let data: ImportFile = serde_json::from_str(json)?;
    let stats = crate::import::run_import(&pool, &data, false, true).await?;
    
    assert_eq!(stats.users_inserted, 1);
    assert_eq!(stats.categories_inserted, 1);
    assert_eq!(stats.topics_inserted, 1);
    assert_eq!(stats.posts_inserted, 1);
    
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = 1")
        .fetch_one(&pool).await?;
    assert_eq!(count, 1);
    
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM forum_topics WHERE id = 100")
        .fetch_one(&pool).await?;
    assert_eq!(count, 1);
    
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM forum_posts WHERE id = 1000")
        .fetch_one(&pool).await?;
    assert_eq!(count, 1);
    
    Ok(())
}
```

You'll need to add a `MIGRATOR` constant to the crate. Check how `tests/forum_api.rs` handles migrations — it likely uses `sqlx::migrate!`. The `forum-import` crate needs the same migration setup.

---

## 4. Phase 4 — Lane 2 e2e report test (§3 Lane 2 remaining item 4)

**Status:** explicitly deferred — "Requires live DB."

**File:** `tests/forum_api.rs` (add to existing file)

Add a test that:
1. Seeds a topic + post as user A.
2. User B (TL1) reports the post via `POST /api/reports` with `target_type: forum_post`.
3. User C (TL5/moderator) calls `GET /api/reports` and sees the report with a deep link.
4. User C resolves the report.

This can be done with a live test DB and the existing `app()` test router. Since the plan explicitly marks this as deferred (needs live DB), note the precondition clearly in the test comment.

---

## 5. Phase 4 — Lane 6 contract assertion test (§3 Lane 6 item 5)

**File:** `tests/forum_api.rs` or `tests/polls_api.rs`

Add an assertion that `topic_detail` returns the `poll` object with the shape `PollBar` reads (`question`, `options` with `text` + `votes`, `my_vote`, `is_closed`, etc.). This locks the contract between backend and frontend.

```rust
#[tokio::test]
#[ignore]
async fn test_topic_detail_includes_poll_object() {
    // Create a topic, attach a poll via POST /api/forum/topics/{id}/polls,
    // fetch topic detail, assert response.question + response.poll fields exist.
}
```

---

## 6. Uncommitted working tree: author batch download hardening (docs/plans/author-batch-download-hardening.md)

**Status:** VERIFIED DONE in the plan's §7, but the changes are **still in the working tree, uncommitted**.

**Files to commit (from `git status`):**
- `docs/plans/author-batch-download-hardening.md` (new)
- `frontend/src/lib/components/BatchAuthorDownload.svelte` (new)
- `.gitignore` (modified)
- `Cargo.lock`, `Cargo.toml` (modified)
- `crates/scrapers/src/lib.rs`, `crates/scrapers/src/sites/{ao3,xenforo}.rs` (modified)
- `docs/brainstorm-*.md`, `docs/frontend-spec/01-api-reference.md`, `docs/specs/frontend/01-api-reference.md`, `docs/src/downloading.md` (modified)
- `frontend/src/lib/api/social.ts` (modified — SSE auth + inline base64 delivery)
- `frontend/src/routes/authors/[id=numeric]/+page.svelte`, `authors/[name]/+page.svelte` (modified — mounted component)
- `src/routes/auth.rs` (modified — `auth_user_from_token` extraction)
- `src/routes/download.rs` (modified — cancellation checks, token auth)
- `src/server.rs` (modified — removed `.cookie_store(true)` from shared client)

**Steps:**
1. Review the diff for each file (already done per the hardening plan §7 — all 9 findings resolved).
2. Split into logical commits per the suggested commit sequence in the hardening plan:
   - `test: specify author batch stream contract`
   - `fix(auth): share validated token resolution with author SSE`
   - `fix(scrape): isolate outbound cookies per batch job`
   - `refactor(download): replace polling SSE stream with ReceiverStream`
   - `fix(download): deliver completed batch without a second scrape`
   - `feat(frontend): mount author batch download control`
   - `test(download): cover cancellation limits and artifact consumption`
   - `docs(download): document author batch behavior and privacy`
3. The security-sensitive commits (auth, cookie isolation) need human review per AGENTS.md §2.4.
4. Verify: `cargo build --release`, `cargo test --lib` (945/945), scraper tests (210/210), frontend social tests (19/19), 5 download-specific tests pass.

**Post-commit follow-ups (NOT in scope per the hardening plan):**
- Large file size limit: inline base64 has no enforced cap. Add a threshold with a one-time artifact endpoint for ZIPs above ~8 MiB.
- Redis-persisted jobs: SSE sessions don't survive browser navigation.

---

## 7. Phase 6 — PWA leftover maintenance (§9.2)

### 7.1 Manifest ↔ file sync check

**Problem:** `manifest.webmanifest` lists icons/screenshots that may 404 at install time.

**Fix:** Write a script (or add to the build step) that reads `manifest.webmanifest` and asserts every `icons[].src` and `screenshots[].src` exists on disk under `frontend/static/` (or wherever the build copies them).

**File:** `docs/plans/pwa-manifest-sync.md` or add to `build.rs` / a CI step.

### 7.2 UpdatePrompt skipWaiting timing (§9.2)

**Problem:** SW calls `skipWaiting()` immediately, so the new SW takes control before the user accepts the prompt — the "reload to update" toast can race the reload.

**Decision:** This is marked "acceptable for v1" — just document it. Add a note in the plan or a code comment.

---

## 8. Migration hygiene: baseline squash (§4 item 6, §9.5 cross-phase)

**Problem:** Forum migration numbering (072–079) was repaired/restored and is non-chronological on fresh DBs. Future DDL (e.g., 090 for XP sources) will add to the confusion.

**Decision:** Defer the full baseline squash to Phase 5/10. For now, ensure migration 090 lands cleanly on both fresh and existing DBs by using `CREATE ... IF NOT EXISTS` and `ON CONFLICT` guards.

No action needed in this plan beyond documenting it.

---

## 0.3 Critical code findings (verified 2026-09-08 before writing)

**JWT secret is NOT in config.rs** — it is read directly from `std::env::var("JWT_SECRET")` with a `"fichub-dev-secret"` fallback in 8 separate files (`auth.rs:234`, `social.rs:56/125/216`, `blind.rs:225`, `device_library.rs:102/127/282`, `rss/handlers.rs:24`, `download.rs:146/277/474/545`, `user_credentials.rs:61`). There is no centralized JWT secret in `AppState` or `Config`. The §9.1 #5 finding is accurate.

**Action:** Add `jwt_secret: String` to `AppState` (populated from `std::env::var("JWT_SECRET").unwrap_or("fichub-dev-secret")` at construction in `server.rs:209`), then replace all 8 hardcoded `std::env::var("JWT_SECRET")` sites with `state.jwt_secret.clone()` (or pass `&state` where already available). The realtime handlers already receive `State(state)` via axum extraction — they just need to read `state.jwt_secret` instead of `std::env::var`.

**AppState struct** (`src/server.rs:29-70`): currently has fields `config`, `db`, `redis`, `health_redis`, `http_client`, `scraper_registry`, `cache_semaphores`, `rate_limiter`, `recommender_engine`, `strategy_registry`, `collection_worker`, `suggest_cache`, `heal`, `wayback`, `ollama`, `mailer`, `rt_manager`. Adding `jwt_secret: String` requires updating:
1. The struct definition (line 69, after `rt_manager`).
2. The construction site (line 209, in the `AppState { ... }` literal).
3. The test `app()` builder in `tests/forum_api.rs` (line 68, the `AppState { ... }` literal — add `jwt_secret: "fichub-test-secret".into()`).
4. Any other test file that constructs `AppState` directly (search for `AppState {`).

**Forum XP awards are already in `forum.rs`:**
- `award_post_exp` (line 159) — called for topic + reply creation (lines 2744, 3005). Writes to `exp_events` + `users.exp`.
- `award_mod_received_exp` (line 173) — called at line 919 for positive mod actions. Writes to `exp_events`.
- `award_reaction_exp` (line 4048) — called at line 3695, **unconditionally after the toggle** (the bug). Writes to `exp_events`.
- `award_poll_vote_exp` (line 4074) — called from `forum_polls.rs:342`, unconditionally.

**Progression service** (`src/services/progression.rs`): `award_xp` (line 21) writes to `xp_events` + `users.xp`. Reads `xp_source_defs` for daily_caps, cooldowns. `apply_xp_and_level_up` (line 232) is the shared transactional core.

**`exp_events` table** (`migrations/001_initial.sql:1016`): columns `(id, user_id, amount, event_type, reference_type, reference_id, created_at)`. Has NO unique constraint for dedup.

**`xp_events` table** (`migrations/001_initial.sql:3823`): columns `(id, user_id, event_type, xp, source_ref, created_at)`. Has a unique constraint `xp_events_user_event_day` mentioned in audit — verify it doesn't already exist before adding.

**`users` table** (`migrations/001_initial.sql:3533`): has both `exp bigint DEFAULT 0` AND `xp integer DEFAULT 0`, plus `level smallint DEFAULT 0`, `rank integer DEFAULT 1`, `trust integer DEFAULT 0`, `reputation integer DEFAULT 0`.

**Forum post reactions** (`forum.rs`): `add_post_reaction` handler (line ~3660) uses a DELETE-then-INSERT toggle pattern. The XP award at line 3695 fires unconditionally — the fix is to check `rows_affected()` on the INSERT.

**Forum polls vote** (`forum_polls.rs`): `vote_on_poll` calls `award_poll_vote_exp` at line 342. Find the vote INSERT and gate on `rows_affected()`.

**Forum importer** (`crates/forum-import/`): No `MIGRATOR` constant, no `sqlx::migrate!` usage in the crate itself. The test needs to handle migrations separately — either copy the site `migrations/` dir path or use `sqlx::migrate!("../../migrations")` relative path. Check `Cargo.toml` — it does NOT depend on `tokio` with `macros` feature or `sqlx` with `migrate` feature (only `runtime-tokio`, `postgres`, `chrono`). The `#[sqlx::test]` macro requires the `migrate` feature on sqlx. See §3.3 for the fix.

**Forum group privileges** (`forum.rs`): `check_category_priv()` is wired into `create_topic` and `create_post` (open-by-default enforcement). `can()` is in `forum_privileges.rs`.

**Forum user block checks** use canonical site `blocked_users(user_id, blocked_user_id)` table, NOT the legacy `forum_user_blocks`.

---

## 9. Execution sequence (recommended order)
|----------|--------|-------------|--------|------|
| P0 | 1.1 | Fix SSE busy-wait | 30 min | low |
| P0 | 1.2 | Wire realtime producers | 2 hrs | medium |
| P0 | 1.3 | WS channel authz | 1 hr | high |
| P0 | 1.4 | JWT secret consolidation | 30 min | medium |
| P0 | 1.5 | Realtime integration test | 2 hrs | medium |
| P1 | 2.1 | Unify XP ledgers | 4 hrs | high |
| P1 | 2.2 | Fix XP farming on toggles | 1 hr | high |
| P1 | 2.3 | Remove email from profile | 15 min | low |
| P1 | 2.4 | Fix reputation=trust bug | 15 min | low |
| P1 | 2.5 | Envelope alignment + achievements stub | 1 hr | low |
| P1 | 3.1–3.3 | Importer hardening (guard, tx, test) | 3 hrs | high |
| P2 | 4 | Lane 2 e2e report test | 2 hrs | medium |
| P2 | 5 | Lane 6 contract assertion | 30 min | low |
| P2 | 6 | Commit author batch work tree | 1 hr | medium |
| P2 | 7 | PWA manifest sync script | 1 hr | low |

**Recommended first sprint:** Tasks 1.1 → 1.2 → 1.3 → 1.4 → 2.1 → 2.2 → 2.3 → 2.4 → 2.5, then run `cargo test` and `cargo build --release`. Realtime producers + XP ledger unification are the two fixes that change the most code surface; tackle them when the user is available for review on the security/authz implications.

---

## 10. Definition of done

- `cargo build --release` succeeds.
- `cargo test --test realtime_api -- --include-ignored --test-threads=1` — all green.
- `cargo test --test forum_api -- --include-ignored --test-threads=1` — all green (including new XP + poll contract + report e2e tests).
- `cargo test --lib` — all green (945/945 baseline).
- `cd frontend && npx svelte-check` — 0 errors.
- `cd frontend && npm test` — only pre-existing failures (if any) remain.
- `src/server.rs` contains no `unwrap_or("fichub-dev-secret")` in realtime code.
- `forum.rs::award_exp` delegates to `services::progression::award_xp`; no code writes to `exp_events` except migration 090 backfill.
- `user_profile` endpoint does not return `email`; `reputation` maps to the actual column.
- Import crate aborts on non-empty DB without `--force`.
- Author batch download working tree is committed in logical chunks.

---

## 11. Open questions for the user

1. **XP ledger unification:** Should we fully retire `exp_events` / `users.exp` (drop the columns in a future migration), or keep them as a read-only archive? The backfill migration copies them to `xp_events` / `users.xp`, but the columns remain.
2. **Achievement tracking:** What specific user actions should unlock achievements (e.g., "First Post", "Popular Post" = 10+ reactions)? The stub endpoint returns whatever is in `user_features` with `feature_type='achievement'`, but nothing populates it yet.
3. **Realtime `publish` from clients:** The plan says remove client `publish` entirely. If there's any current frontend code relying on client-to-client WS messages (chat-like), that will break. Confirm this feature doesn't exist in the current UI.
4. **Importer `--force`:** Should `--force` remap NodeBB IDs to avoid collisions, or just skip the guard and accept the `ON CONFLICT DO NOTHING` behavior (current) on non-empty DBs?
