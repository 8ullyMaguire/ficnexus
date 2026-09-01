# Code-Along: Building FicHub's Reputation, XP, and Trust Systems (A–D)

> Pair with `docs/plans/reputation-xp-rewards-v3.md` (core design) and
> `docs/plans/reputation-xp-remaining-work.md` (Tier A–D backlog).
> This tutorial walks the **remaining** implementation end-to-end: idle-tick dedup,
> chapter-path wiring, leaderboard switch, long-work dividend cron, admin-award
> endpoint, bounty UI, progression UI, and Discourse-style trust levels.
> Build in priority order: **A1 → A2 → A3 → B1 → B3 → B2 → C1/C2 → D**.

## Prerequisites

- `cargo test --lib` green (735 tests after A1–A3; 740+ after tests added).
- Binary md5 `0e391891` on prod (the baseline this tutorial extends).
- Migration 010 applied — `xp_source_defs` has cooldown/rate_limit/streak cols.
- FichHub repo at `/personal/documents/code/rust/fichub`, worktree at
  `/media/alvaro/code-worktrees/target`.
- Feature branch: `git checkout -b feat/xp-trust-complete`.

---

## Step 0: Verify baseline

```bash
cd /personal/documents/code/rust/fichub
export CARGO_TARGET_DIR=/media/alvaro/code-worktrees/target
cargo test --lib 2>&1 | tail -1
```
→ `test result: ok. 735 passed; 0 failed`.

---

## Step A1: Idle-tick deduplication (block farming) ✅ DONE

**Goal**: `POST /api/xp/idle-tick` awards only ONE `idle_tick` XP (2 XP) per user
per hour, even under rapid POST spam. Deployed `12:07`; commit `bcaabb5`
(branch `feat/idle-tick-hourly-dedup`, merged to main).

### Actions

- **`migrations/011_idle_tick_cooldown.sql`** (NEW — does NOT reuse `010`):
```sql
UPDATE xp_source_defs SET cooldown_seconds = 3600 WHERE event_type = 'idle_tick';
```
- **`src/services/bounties.rs`** `idle_tick` function:
  - Compute a per-user-per-hour time-bucket via `chrono::Utc::now()`.
  - Pass it as `source_ref` to `award_xp("idle_tick", Some(&hour_bucket))`.
  The engine dedups on `(user_id, event_type, source_ref)` within `cooldown_seconds`
  (3600s) — same hour = same bucket = cooldown fires and suppresses the dupe.
```rust
pub async fn idle_tick(pool: &sqlx::PgPool, user_id: i32) -> AppResult<i64> {
    let hour_bucket = Utc::now().format("%Y-%m-%dT%H").to_string();
    award_xp(pool, user_id, "idle_tick", Some(&hour_bucket)).await
}
```
- **`src/routes/bounties.rs`** `idle_tick_handler`: no change — already calls
  `bounties::idle_tick(&state.db, user_id)`; the bucket is computed inside.

### Check
```bash
cargo test --lib hour_bucket idle_tick
```
→ both `hour_bucket_truncates_to_hour` +
`idle_tick_hour_bucket_is_stable_within_same_hour` pass (5/5 bounty tests green).

### Troubleshooting
- `AuthUser` has no `session_id` (JWT has no `jti`) — session-based dedup would
  require a JWT change. The **hour-bucket-as-source_ref** approach achieves the
  same anti-farm outcome without touching the auth layer (per-user, not
  per-session, but that matches "1 tick per hour" intent).
- Cooldown too tight for legit returns: 3600s is correct per the 2XP/tick plan;
  a user ticking once/hour maxes at 48/day (well under the 1440/day engine cap).

---

## Step A2: Wire `work_chapter` + `engagement_dividend` at the chapter/scrape path

**Goal**: when a scraper appends a NEW chapter to an existing work, fire
`work_chapter` (≥1k words) AND auto-claim any open `work` bounties on that url_id.
Also fire `engagement_dividend` from social actions.

### Actions

- **Find the chapter-append path**: chapters live as a `jsonb` array in
  `fic_info.chapters`. Find where the builder/scraper mutates it:
  ```bash
  grep -rn "chapters" src/scrape/ src/builder.rs | grep -i "UPDATE\|patch\|jsonb_set" | head
  ```
  Likely in `src/scrape/lookup_authed.rs` or `src/ingest/manual.rs`.

- **Hook**: after a new chapter is confirmed appended (chapters array length
  grew), and the chapter has `word_count >= 1000`:
  ```rust
  // In the chapter-append path:
  let url_id = &fic.source_url;
  if chapter_words >= 1000 {
      let _ = crate::services::progression::award_xp(&state.db, author_id, "work_chapter", Some(url_id)).await;
      // Auto-claim work bounties ("next chapter of this fic"):
      let _ = crate::services::bounties::auto_claim_work_bounties(&state.db, work_id, author_id).await;
  }
  ```

- **Engagement dividend**: in `src/routes/social.rs` bookmark/rating/review/follow handlers,
  fire `work_chapter`... no — `engagement_dividend`:
  ```rust
  let _ = crate::services::progression::award_xp(&state.db, actor_id, "engagement_dividend", Some(&fic_url_id)).await;
  ```
  Fire on: bookmark added, 5-star rating given, review posted, list/follow created.

### Check
- Append a 1200-word chapter to an existing work → `SELECT event_type FROM xp_events WHERE user_id=<author> ORDER BY id DESC LIMIT 1` → `work_chapter`.
- `grep -rn "work_chapter" src/ | grep -v test` → ≥2 sites (admin.rs + scrape path).
- `grep -rn "engagement_dividend" src/ | grep -v test` → ≥3 sites.

### Troubleshooting
- Can't find the chapter-append mutation → search for `chapters =` or `jsonb_set` or the chapter struct mutation in `src/scrape/`. If chapters are rebuilt wholesale (full array replace), fire `work_chapter` once per *new* chapter detected via length diff.

---

## Step A3: Switch leaderboard to `xp_events` + backfill

**Goal**: `/leaderboard` reads the canonical XP ledger, not the legacy reputation_events.

### Actions

- **File**: `migrations/011_leaderboard_xp_backfill.sql`
```sql
-- Idempotent: backfill reputation_events → xp_events ONCE.
INSERT INTO xp_events (user_id, event_type, xp, source_ref, created_at, level_up, xp_residual)
SELECT user_id, event_type, points, COALESCE(reference_id, reference_type), created_at, false, 0
FROM reputation_events
ON CONFLICT (user_id, event_type, source_ref, created_at) DO NOTHING;
```
Needs a unique constraint — check `xp_events` has one on `(user_id, event_type, source_ref, created_at)` or add it:
```sql
-- If missing:
ALTER TABLE xp_events ADD CONSTRAINT xp_events_uniq UNIQUE (user_id, event_type, source_ref, created_at);
```

- **File**: `src/db/queries.rs` — 4 queries (`compute_weekly_leaderboard`,
  `compute_monthly_leaderboard` at :1605/:1638):
  - `FROM reputation_events` → `FROM xp_events`
  - `SUM(points)` → `SUM(xp)`
  - `WHERE event_type IN (...)` stays (same event types already backfilled).

- **File**: `src/bin/compute_leaderboards.rs` — no change needed if it just
  calls the queries; verify it still compiles.

### Check
- `psql -tAc "SELECT count(*) FROM xp_events"` ≥ previous `count(*) FROM reputation_events`.
- `cargo test --lib` → green.
- `/api/leaderboard/top` returns ranked users (query prod after recompute).

### Troubleshooting
- `ON CONFLICT DO NOTHING` fails (no unique constraint) → add the constraint
  first, or run backfill in a transaction that truncates `xp_events` for
  backfilled types then re-inserts (risky — do the constraint approach).
- Leaderboard scores drop → the weekly window started fresh from xp_events;
  noted acceptable in the plan (release notes).

---

## Step B1: Long-work passive dividend cron

**Goal**: completed ≥20k-word works that are favorited/bookmarked earn the author
10 XP/min reading-time-equivalent, capped 2,880/wk/work, decaying after 30d.

### Actions

- **File**: `src/bin/long_work_dividend.rs` (new, mirror `compute_leaderboards.rs`)
```rust
// Pseudo-logic:
// 1. SELECT w.id, fic.source_url, fic.word_count,
//         COUNT(bookmarks)+COUNT(favorites) AS engagement,
//         MAX(engagement.created_at) AS last_engagement
//    FROM works w JOIN fic_info fic ON fic.work_id=w.id
//    LEFT JOIN bookmarks b ON b.work_id=w.id
//    WHERE w.is_visible AND fic.status='complete' AND fic.word_count >= 20000
//    GROUP BY w.id
//    HAVING last_engagement > now() - interval '30 days'   -- decay gate
// 2. minutes = word_count / 200  (reading speed 200wpm); minutes capped...
//    Actually: 10 XP/min × minutes_read × engagement_count, clamped to 2880/week.
// 3. For each: award_xp('long_work_dividend', Some(source_url)) with the
//    *weekly remaining* under the rate_limit cap (engine enforces 2880/wk).
```
- The engine's `rate_limit_per_period` on `long_work_dividend` def enforces 7-day cap = 2,880. Set it:
  ```sql
  UPDATE xp_source_defs SET rate_limit_per_period = 2880 WHERE event_type = 'long_work_dividend';
  ```
  (Add to 011 migration.)

- **Schedule**: add to `scripts/install_systemd_timers.sh`:
  ```
  [Timer]   OnCalendar=*-*-* 03:00:00   (runs daily at 3am, after compute_leaderboards)
  ```

### Check
- `cargo build --bin long_work_dividend` → clean.
- `--dry-run` flag prints candidate works + computed XP without awarding.
- 011 migration sets `rate_limit_per_period=2880` on `long_work_dividend`.

### Troubleshooting
- `minutes_read` formula: use `word_count / 200` (standard 200wpm) — or reuse
  any existing `reading_time` fn in the codebase (check `content_scan.rs`).
- Decay re-arms: the `HAVING last_engagement > 30d` filter means a dormant work
  simply gets skipped that run; a fresh favorite/bookmark re-includes it next run.

---

## Step B3: Admin reputation-award endpoint ✅ DONE

**Goal**: the only way to grant admin-only rewards (scraper fixes, code PRs,
tag wikis, marathon_writer). Deployed; commit `00043e8` on
`feat/admin-reputation-award`, merged to main.

### Actions

- **`src/routes/admin.rs:380-424`** — `rep_award_handler`:
```rust
#[derive(Deserialize)]
pub struct RepAwardReq {
    pub user_id: i32,
    pub event_type: String,
    pub source_ref: Option<String>,
    pub note: Option<String>,
}

pub async fn rep_award_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<RepAwardReq>,
) -> Result<Json<Value>, AppError> {
    if auth.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let admin_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    // Allowlist: only Admin:-prefixed source defs are grantable here.
    let is_admin_only: (bool,) = sqlx::query_as::<_, (bool,)>(
        "SELECT description LIKE 'Admin:%' FROM xp_source_defs WHERE event_type = $1",
    )
    .bind(&req.event_type)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest(format!("Unknown event_type: {}", req.event_type)))?;
    if !is_admin_only.0 {
        return Err(AppError::BadRequest("Not an admin-awarded reward".into()));
    }
    let awarded = crate::services::progression::award_xp(
        &state.db, req.user_id, &req.event_type, req.source_ref.as_deref(),
    )
    .await?;
    crate::modlog::record_json(
        &state.db,
        Some(admin_id),
        auth.username.clone(),
        "reputation_award",
        "user",
        &req.user_id.to_string(),
        vec![
            ("event_type", json!(req.event_type)),
            ("source_ref", json!(req.source_ref)),
            ("note", json!(req.note)),
            ("xp_awarded", json!(awarded)),
        ],
    )
    .await;
    Ok(Json(json!({ "err": 0, "ok": true, "xp_awarded": awarded })))
}
```
- **Mount** in `src/server.rs`:
```rust
.route("/api/admin/reputation/award", axum::routing::post(crate::routes::admin::rep_award_handler))
```

### Check
- `cargo check --bin fichub` → EXIT=0.
- Admin token: `curl -X POST localhost:8000/api/admin/reputation/award -d '{"user_id":42,"event_type":"scraper_fix_merged"}'` → `{"err":0,"ok":true,"xp_awarded":...}`.
- Non-admin: → 403.
- Non-admin event_type (e.g. `work_chapter`): → 400 "Not an admin-awarded reward".
- Modlog has a `reputation_award` row.

### Troubleshooting
- `is_admin_only` type was `Option<(bool,)>` then `?` unwrapped it to `(bool,)` —
  the `ok_or_else` converts `None` to a 400, so the final `.0` is safe.
- Modlog uses `record_json` (the `Vec<(&str, Value)>` form from `modlog.rs:40`),
  not the older `record()` signature.

---

## Step B2: Marathon-writer title + badge

**Goal**: `marathon_writer` (1000 XP) awarded via B3 endpoint + a visible title badge.

### Actions

- **Depends on B3** (admin award endpoint allowlist).
- **Badge storage**: add `users.badges` or reuse. Check if a `user_badges` table
  exists; if not, the simplest: `ALTER TABLE users ADD COLUMN marathon boolean DEFAULT false`.
- **Where**: in the `rep_award_handler` (B3), after awarding `marathon_writer`,
  set `users.marathon = true` for that user + modlog it.
- **Badge display**: show "🏆 Marathon Writer" on the author profile. Check
  `src/routes/profile.rs` or `WorkAuthor.svelte` for where titles render.

### Check
- Admin grants `marathon_writer` → `SELECT marathon FROM users WHERE id=42` → `true`.
- Author profile page renders the badge.

### Troubleshooting
- If `marathon` column not added in 011 → add it; coordinate with the badge plan.

---

## Step C1: Bounty UI (`/bounties` + claim flow)

**Goal**: user-facing pages to view/create/claim/resolve bounties.

### Actions

- **File**: `frontend/src/routes/bounties/+page.svelte` (list + create modal)
- **File**: `frontend/src/routes/bounties/[id]/+page.svelte` (claim + resolve modal)
- **API calls** (reuse existing `/api/bounties` endpoints):
  - GET `/api/bounties?target_type=work` → open bounties
  - POST `/api/bounties` → create {target_type, target_ref, amount, goal_desc, expiry_days}
  - POST `/api/bounties/{id}/claim` → {claim_ref}
  - POST `/api/bounties/{id}/resolve` → admin {action: resolved|slashed|cancelled}
- **i18n**: 12 new keys × 6 dicts (`en,de,es,fr,pt-BR,zh`):
  `bounty.create`, `bounty.stake`, `bounty.claim`, `bounty.resolve`, `bounty.resolved`,
  `bounty.slashed`, `bounty.cancelled`, `bounty.amount`, `bounty.goal`, `bounty.expiry`,
  `bounty.your_stake`, `bounty.claim_proof`. **No emoji** in any label.
- **Route tests**: `frontend/src/routes/bounties/page.test.ts`

### Check
```bash
cd frontend && npx svelte-check 2>&1 | grep -E "error|warn" | wc -l
npm run build 2>&1 | tail -1
```
→ 0 svelte errors; build "Wrote site to build".
Visit `/bounties` logged in → list renders. Create a 50-rep bounty → rep decremented.

### Troubleshooting
- `svelte-check` flags missing i18n keys → grep all 6 dicts for `bounty.` prefix, add missing.
- Auth token on API call → use the shared `auth` store + `authHeaders()`.

---

## Step C2: Progression page (XP history + level-up toast)

**Goal**: `/progression` shows recent XP events + celebrates level-ups.

### Actions

- **File**: `frontend/src/routes/progression/+page.svelte` (exists per roadmap but needs XP feed)
- **API**: `GET /api/me/progression` returns `recent_events` (xp_events) + `level` + `rank` + `xp_residual`.
- **Render**: list events (event_type → localized label, xp, created_at), a
  level bar (current XP / xp_for_next_level), and a toast "Level N! +M residual"
  when an event has `level_up=true`.
- **i18n**: `prog.event.work_publish`, `prog.event.work_complete_qualified`,
  `prog.event.work_chapter`, `prog.event.engagement_dividend`,
  `prog.event.long_work_dividend`, `prog.event.level_up`, `prog.event.idle_tick`,
  `prog.event.bookmark`, `prog.event.rate_fic`. **All 6 dicts.**
- **Route test**: `frontend/src/routes/progression/page.test.ts`

### Check
- `npx svelte-check` → 0 errors.
- `grep -rn "prog.event" frontend/src/lib/i18n/dictionaries/*.ts | wc -l` → ≥48 (8 keys × 6).
- Visit `/progression` → recent events render; level-up event shows toast.

---

## Step D: Trust levels (DISMISSED — pending user replan, see analysis)

> **Status**: The user has requested this section be replanned from scratch.
> The Discourse TL0–TL4 design documented below is superseded — DO NOT implement
> until a replacement decision lands. This placeholder is retained only so the
> A–D structure of the tutorial stays intact while D is in flight.
>
> Current blocker analysis (kept for the replan):
> - **Trust ≠ Reputation, ≠ Level, ≠ Curator role**: there are FOUR overlapping
>   axes already (`users.role` = {0,1,5,10}, `users.level` = XP level 0–~58,
>   `users.reputation` = spendable bounty currency, proposed TL0–4 = read/write
>   sandboxes). Users confuse them; support tickets conflate "level 45" with "can
>   edit others' posts."
> - **No natural data source yet**: A3 (leaderboard) is still deferred, so the
>   30-day read/visit signals the promoter would run on don't exist in a clean
>   form — `xp_events` only has author/reputation events, not reader behavior.
> - **Curator gate is `role >= 10`, not TL**: the existing mod system (`work_delete`,
>   `tag_merge`, `content_proposal`) is role-gated. Re-mapping to TL would either
>   (a) elevate active TL3 readers to curator powers (risky — reading ≠ judgment)
>   or (b) duplicate the role table as TL (redundant).
> - **`users.total_words_read`** exists and could seed TL0→TL1, but it's updated
>   only by the body-cache scan (async, lags real reads by hours).
>
> **Decision needed before replan**: do we want (1) a *sandbox* axis (TL) that
> limits new users, (2) a *feature-unlock* axis that gates UI behind activity,
> or (3) fold both into the existing `level` + `reputation` and kill the fourth
> axis? Each has different schema + migration cost.

### D1. Migration `012_trust_levels.sql`

```sql
ALTER TABLE users
  ADD COLUMN IF NOT EXISTS trust_level smallint DEFAULT 0 NOT NULL CHECK (trust_level BETWEEN 0 AND 4),
  ADD COLUMN IF NOT EXISTS tl0_until timestamptz,
  ADD COLUMN IF NOT EXISTS tl_metrics jsonb;
CREATE INDEX IF NOT EXISTS idx_users_trust_level ON users (trust_level);
-- Backfill: new accounts (<14d) stay TL0; anyone with reads gets TL1 immediately.
UPDATE users SET trust_level = 1 WHERE id NOT IN (
  SELECT user_id FROM xp_events WHERE event_type IN ('read_topic','read_post') LIMIT 1
) AND created_at > now() - interval '14 days';
```

### D2. Promoter `src/bin/trust_promote.rs`

Hourly job. Pseudocode (full thresholds from `docs/plans/trust-levels-rework.md`):
```
for user in users where trust_level < 4 and (tl0_until is null or tl0_until < now):
  m = load_or_compute_tl_metrics(user)  # reads xp_events + usage_events 30-day
  if meets_tl3(m): set trust_level=3; log event
  elif meets_tl2(m): set trust_level=2; log + welcome notice
  elif meets_tl1(m): set trust_level=1; log + welcome notice
  # TL3 demotion (after 14-day grace) if m dips below threshold
```

### D3. Gate checks (trust vs role)

Replace `user.role < 10` sandbox gates. Map:
- PM / flag / >2 hyperlinks / image+attachment / create-topic / reply-as-new-topic → `trust_level >= 1`
- TL0 can still READ (fundamental action) — no gate change for reads.
- Likes limits scale: TL1=6, TL2=9, TL3=12, TL4=15/day.
- `/forum/lounge` → `trust_level >= 3` readable.
- **Role stays** for moderation: `curator_content.rs:25-31` (`role >= 10`) unchanged.

Grep targets: `grep -rn "role < 10\|role >= 10\|role <= 10" src/routes/ | grep -v admin` → those become `trust_level >= N` checks (social gates), leaving admin.rs untouched.

### Check
- `fichub migrate` applies 012 (TL0/1/2 backfilled).
- `src/bin/trust_promote` runs dry → prints candidates per tier.
- `curl /api/forum/...` as a TL0 user → 403 on PM/flag.
- Non-admin route (`src/routes/social.rs`) still gates on role for *moderation* actions, trust for *sandbox* actions.

### Troubleshooting
- `trust_level` column already exists (partial prior attempt) → `ADD COLUMN IF NOT EXISTS` is safe.
- Backfill false-positives TL1 → tune the `xp_events` check; the goal is "engaged reader" not "spammy account."

---

## Final verification (whole A–D stack)

1. `cargo test --lib` → 0 failed (target ≥ 740 tests).
2. `cargo build --release --bin fichub --bin long_work_dividend --bin trust_promote`.
3. `fichub migrate` → versions 1-8, 10, 11, 12 applied (no gaps).
4. Smoke test against prod via `ssh -L` tunnel:
   - Approve a complete 50k-word work → `xp_events` has `work_publish` (100) +
     `work_complete_qualified` (scaled to 200) + level-up residual logged.
   - `curl /api/bounties/{id}/resolve` unauthenticated → 401; as admin → works.
   - `GET /api/leaderboard/top` returns XP from `xp_events` (not reputation_events).
   - A user with 5+ `read_post` events + visits → TL1 after promoter run.
5. Verify prod health stays 200, no regressions in `/health`.

---

*Per the code-along-tutorial-writer skill. Pair with:*
- `docs/plans/reputation-xp-rewards-v3.md` (core design + shipped baseline)
- `docs/plans/reputation-xp-remaining-work.md` (Tier A–D backlog + priority queue)
- `docs/plans/trust-levels-rework.md` (TL0–TL4 thresholds + gate map)
- `docs/brainstorm-02-roadmap-status.md` (canonical roadmap, hierarchy statement)
