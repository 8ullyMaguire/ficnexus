# Code-Along: Building the Reputation (XP) Reward System v3

In this tutorial, you'll bring FicHub's v3 XP engine online, wire config-driven rewards, and add reputation-backed bounties (stakeable currency) on top — ending with the hierarchy **Consistent Authors > Infrequent Authors > Consistent Contributors > Average Users**, guarded against gaming.

You'll follow the implementation plan (`docs/plans/reputation-xp-rewards-v3.md`) one verifiable step at a time. Each step has a Check so you know you're on the rails.

## What you'll build
- A config-driven XP engine reading `award_xp`'s def columns (`xp_source_defs`): daily/per-period caps, cooldown, streak multiplier. **(Shipped `src/services/progression.rs:21-201`.)**
- **XP resets to the residual on level-up** — `users.xp` zeroes past the threshold and the leftover is logged in `xp_events.xp_residual`. Level cost curve `50 * level^1.6` (early levels cheap, L99→L100 ≈ 31,623 XP alone).
- **Reputation as spendable currency**: users stake `users.reputation` on bounties; creator loses it on stake, claimant gains it on resolve. Reputation accumulates for life (never resets).
- Bounty endpoints `POST /api/bounties` (create+stake), `POST /api/bounties/{id}/claim`, `POST /api/bounties/{id}/resolve` (admin), `GET /api/bounties`, plus passive `POST /api/xp/idle-tick`. **(Shipped `src/services/bounties.rs` + `src/routes/bounties.rs`, mounted `server.rs:393-397`.)**
- Approval-gated auto-claim: when an admin approves a ≥5k-word manual upload, open `work` bounties on that url_id auto-claim for the uploader. **(Shipped `src/routes/admin.rs:116-133`.)**

## Prerequisites
- Rust toolchain (edition 2021), `cargo`.
- PostgreSQL 14+ with a FicHub DB.
- Repo checked out at `/personal/documents/code/rust/fichub`.
- `cargo test --lib` passes (baseline 730 — now 735 with the new level-curve tests).

## Codebase geography (read before you start)
- `src/services/progression.rs` — the **live** v3 engine `award_xp()` (was dormant; now wired).
- `src/services/bounties.rs` — bounty service (create/claim/resolve/idle/auto-claim).
- `src/routes/bounties.rs` — bounty HTTP handlers (admin resolve gated `role >= 10`).
- `src/routes/admin.rs:116` — approval path that fires `work_publish` + auto-claims work bounties.
- `src/db/queries.rs:2289` — the **legacy** `update_reputation_and_promote` still dual-wires during the transition sprint (reputation_events + the shim's rep delta).
- `src/progression.rs:105` — `xp_for_level(level) = Σ 50*l^1.6`; `xp_to_level` inverts it; `xp_to_rank` maps level→rank.
- `migrations/010_reputation_bounties.sql` — THIS deploy's migration (see Step 1).

**Golden constraint:** runtime `sqlx::query(...)` with `$N` binds only — never `query!` compile-time macros. Keep it that way.

---

## Step 0: Environment Setup

**Goal:** land a clean, testable workspace.

### Actions
- **File**: repo root
- ```
  cd /personal/documents/code/rust/fichub
  export CARGO_TARGET_DIR=/media/alvaro/code-worktrees/target
  git checkout -b feat/xp-rewards-v3
  cargo test --lib
  ```

### Check
- `git branch --show-current` → `feat/xp-rewards-v3`.
- `cargo test --lib` → 735 passed, 0 failed.

### Troubleshooting
- Incremental cache stale (MISSING new symbols) → nuke `$CARGO_TARGET_DIR/release/{.fingerprint,deps,build}`, re-touch entrypoints, rebuild.
- `cargo` builds into `/home/...` anyway → re-export `CARGO_TARGET_DIR` in the same shell.

---

## Step 1: Write the migration `010_reputation_bounties.sql`

**Goal:** extend the schema for bounties + level-up reset columns. Migration is **`010`** (NOT `009` — that slot is reserved on prod; never reuse).

### Actions
- **File**: `migrations/010_reputation_bounties.sql`
- Create `bounty_pots` (id, creator_id, target_type, target_ref, amount, goal_desc, expiry_at, status, claim_ref, claimant_id, resolver_id, resolved_at), `bounty_resolutions` (audit), extend `xp_events` with `level_up` + `xp_residual`, seed `xp_source_defs` rows with anti-gaming columns, seed `xp_sources_streaks`, add `idx_users_xp`/`idx_users_reputation`.

The key anti-gaming rows (see full plan §010c):
| event_type | xp | daily_cap | rate_limit_per_period | min_value | streak |
|---|---|---|---|---|---|
| `work_chapter` | 15 | 150 | — | 1000 | 30d x1.5 @5 |
| `engagement_dividend` | 1 | 50 | 50 | — | — |
| `long_work_dividend` | 10/min | capped 2880/wk | 2880 | 20000 (words) | 7d window |
| `work_complete_qualified` | `100 + 2×(words/1000)` | — | — | 5000 | 60d streak |
| `completion_streak_bonus` | `50 × N` (cap 500) | — | — | — | 60d |
| `marathon_writer` | 1000 | — | — | — | admin-only |
| `curation_tier1` | 15 | — | 70 | — | — |
| `idle_tick` | 2 | 1440 | 720 | — | — |

### Check
- `fichub migrate --dry-run` → lists `010_reputation_bounties` pending.
- `psql $DATABASE_URL -c '\d xp_source_defs'` → shows `cooldown_seconds`, `rate_limit_per_period`, `streak_window_days`, `streak_multiplier`.

### Troubleshooting
- Checksum mismatch on `_sqlx_migrations` → `DELETE FROM _sqlx_migrations WHERE version=10;` + restart (do NOT hand-INSERT).

---

## Step 2: Extend `award_xp` with cooldown + per-period cap + streak

**Goal:** make the engine honor the new rule columns in ONE select + resolve `actual_xp` before inserting.

### Actions
- **File**: `src/services/progression.rs:28`
- Read all six def columns in one query:
```sql
SELECT xp_amount, daily_cap, cooldown_seconds, rate_limit_per_period,
       streak_window_days, streak_multiplier FROM xp_source_defs WHERE event_type = $1
```
- Check **cooldown** (count same `(user_id, event_type)` in the cooldown window → if >0, log a 0-xp row + return 0).
- Check **daily cap** (`created_at > now() - interval '1 day'`), then `min(base, remaining)`.
- Check **rate_limit** (`> 7 days`), zero XP if exceeded.
- Check **streak**: if `streak_window_days` + `streak_multiplier` set, count events in window; at `boost_at`-th apply `((base * multiplier) * (1 + (count - boost_at)/boost_at))`. **Streak is applied inline here, not in a separate `rewards.rs` module** (the plan's Task 2 was deferred; inlining keeps one level-up code path).

### Check
- `grep -n "cooldown_seconds\|rate_limit\|streak_multiplier" src/services/progression.rs` → all referenced.
- `grep -rn "rewards.rs\|award_if_qualifies\|award_with_streak" src/` → empty (no `rewards.rs` created in this deploy).

### Troubleshooting
- `make_interval(secs => $N)` bind mismatch → bind `cd as i64` or `$3::bigint`.
- `&mut *tx` vs `&mut tx` → use `&mut *tx` for sqlx 0.9 executors inside transactions.

---

## Step 3: Wire the approval path to fire `work_publish` + auto-claim bounties

**Goal:** when an admin approves a ≥5k-word upload, (a) auto-claim any open `work` bounty on that url_id, (b) fire `award_xp("work_publish")`.

### Actions
- **File**: `src/routes/admin.rs:116-133`
```rust
// After `UPDATE works SET is_visible = TRUE`:
let (words,): (Option<i64>,) = sqlx::query_as(
    "SELECT fi.word_count FROM fic_info fi WHERE fi.work_id = $1"
).bind(work_id).fetch_optional(&state.db).await?.unwrap_or((None,));
let w = words.unwrap_or(0);
let _ = crate::services::bounties::auto_claim_work_bounties(&state.db, work_id, uploader_id).await;
if w >= 5000 {
    if let Some((url_id,)) = sqlx::query_as::<_, (String,)>(
        "SELECT fi.source_url FROM fic_info fi WHERE fi.work_id = $1"
    ).bind(work_id).fetch_optional(&state.db).await? {
        let _ = crate::services::progression::award_xp(&state.db, uploader_id, "work_publish", Some(&url_id)).await;
    }
}
// Legacy reputation path retained for one transition sprint (see Risks).
let _ = crate::db::queries::update_reputation_and_promote(&state.db, uploader_id, 25, "upload_approved").await;
```
- **Explanation**: qualifies at the call site (word count ≥5k), auto-claims matching `work` bounties (the "more chapters of this fic" use case), then awards XP. The legacy +25 reputation still fires so the existing `/leaderboard` keeps moving until Step 6 (leaderboard switch) ships.

### Check
- Approve a ≥5k-word manual upload → `SELECT event_type FROM xp_events WHERE user_id=<uploader> ORDER BY id DESC LIMIT 1` → `work_publish`.
- If open `work` bounties existed on that url_id → their `status` advanced `open→claimed`, `claimant_id` set.

---

## Step 4: Build the bounty service

**Goal:** create/claim/resolve the spendable-reputation bounties + idle-tick XP.

### Actions
- **File**: `src/services/bounties.rs` (create module + register in `src/services/mod.rs`)
- `create_bounty(pool, creator_id, target_type, target_ref, amount, goal_desc?, expiry_days)`:
  - gate: `users.reputation >= MIN_REP_FOR_BOUNTY` (50).
  - `UPDATE users SET reputation = reputation - amount` (reserve, not burn).
  - `INSERT INTO reputation_events (..., event_type='bounty_staked', points=-amount)`.
  - `INSERT INTO bounty_pots`.
- `claim_bounty_with(pool, pot_id, claimant_id, claim_ref)`: set `status='claimed'`, `claimant_id`, `claim_ref`.
- `resolve_bounty(pool, resolver_id, pot_id, action, note) -> i32 payout`:
  - admin-only at route layer.
  - `resolved` → claimant += amount - fee (fee burns via `bounty_fee_burned` rep event); `cancelled` → creator += amount (refund); `slashed` → 0 (staked rep stays burned). Audit `bounty_resolutions` row.
  - `CURATOR_FEE_BPS = 1000` (10%).
- `auto_claim_work_bounties(pool, work_id, uploader_id)`: resolve the work's url_id, find open `work` bounties matching, auto-claim as uploader.
- `idle_tick(pool, user_id)`: `award_xp("idle_tick", Some(auth_session_token))` — tiny presence reward, capped 1440/day by the engine.

### Check
- Unit tests `cargo test --lib bounties`:
- `payout_math_resolved` → `1000 - 100 = 900`.
- `payout_math_slashed_yields_zero` → slashed pays 0.
- `min_rep_gate_blocks_zero_rep_creator` → gate exists (`MIN_REP_FOR_BOUNTY > 0`).
- `grep -n "CURATOR_FEE_BPS\|MIN_REP_FOR_BOUNTY" src/services/bounties.rs` → both defined, fee = 1000.

### Troubleshooting
- `Transaction<'_, Postgres>` not in scope → use `sqlx::Transaction<'_, sqlx::Postgres>` fully-qualified.
- `claim_bounty` vs `claim_bounty_with` signature mismatch → the route passes `user_id`; match exactly.

---

## Step 5: Mount the bounty routes

**Goal:** expose the bounty + idle-tick HTTP surface.

### Actions
- **File**: `src/routes/bounties.rs` (create) + `src/server.rs:393-397` (mount):
```rust
.route("/api/bounties", get(crate::routes::bounties::list_bounties))
.route("/api/bounties", post(crate::routes::bounties::create_bounty_handler))
.route("/api/bounties/{id}/claim", post(crate::routes::bounties::claim_bounty_handler))
.route("/api/bounties/{id}/resolve", post(crate::routes::bounties::resolve_bounty_handler))
.route("/api/xp/idle-tick", post(crate::routes::bounties::idle_tick_handler))
```
- Auth: `create/claim/idle-tick` require login (`auth.user_id`); `resolve` requires `auth.role >= 10` (admin). `/bounties` list is public.

### Check
- Unauthenticated: `curl /api/bounties/{id}/resolve` → `{"err":401,"msg":"Login required"}`.
- `strings $(cargo build --release --bin fichub | ... fz)` path → `grep /api/bounties` present in binary.

---

## Step 6 (DEFERRED — next sprint): Switch leaderboard to `xp_events`

The leaderboard (`compute_weekly_leaderboard`/`compute_monthly_leaderboard`) still reads `reputation_events`. This deploy keeps the **dual-write transition** (legacy rep + new XP) so `/leaderboard` doesn't regress. Switching the source + running the `reputation_events → xp_events` backfill lands next sprint — see plan §Deferred Tasks 2 & 4.

---

## Final Verification (run this deploy)
1. `cargo test --lib` → **735 passed, 0 failed** (5 new: `level_curve_expensive_late`, `level_up_reset_zeroes_xp_after_threshold`, `payout_math_resolved`, `payout_math_slashed_yields_zero`, `min_rep_gate_blocks_zero_rep_creator`).
2. `cargo build --release --bin fichub` → clean, `md5 e9efbd1214b4f957d588fae024d35823`.
3. Binary deployed to thinkcentre: `systemctl is-active fichub → active`, `health=200`.
4. Routes live: unauthenticated `/api/bounties/{id}/resolve` → 401 (auth-gated correctly).
5. Migration 010 schema present: `xp_source_defs` has streak+cooldown columns; `xp_events.level_up`/`xp_residual` exist; `bounty_pots`/`bounty_resolutions` tables exist.

## Next Steps
- Ship the leaderboard source switch (Task 6) → removes the legacy dual-write.
- Wire `work_chapter` + `engagement_dividend` at the scraper/chapter boundaries (currently only `work_publish` at the approval path).
- Frontend bounty page (`/bounties`) consuming the new API + Tier-8 i18n labels.
- Add `admin_award` handler (the plan's `POST /api/admin/reputation/award`) as a dedicated grant path for meta-rewards.

---

*Per the code-along-tutorial-writer skill. Pair with `docs/plans/reputation-xp-rewards-v3.md` for the full task breakdown.*
