# Reputation (XP) Reward System v3 — Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Rewards engine that yields the post hierarchy **Finishers of Long Works > Consistent Authors > Infrequent Authors > Consistent Contributors (Translators/Curators) > Average Users**, with anti-gaming (caps, quality thresholds, streaks, admin-verified meta rewards).

**Architecture:** Unify on the existing **dormant v3 XP engine** (`award_xp` in `src/services/progression.rs` + `xp_source_defs` config table). Add new `event_type`s to `xp_source_defs`, wire ~13 award sites, add streak/daily-cap enforcement inside the engine, decay, and an admin `/api/admin/reputation/award` endpoint. Rebuild the leaderboard computation to read from the canonical `xp_events` ledger.

**Tech Stack:** Rust (Axum + sqlx runtime `query` binds only — no `query!`), PostgreSQL, existing `src/services/progression.rs` engine, existing SvelteKit frontend (`/progression`, `/leaderboard`).

---

## Finishing-Long-Works Incentive Layer

> Hierarchy update requested: **Finishers of Long Works > Consistent Authors > Infrequent Authors > Consistent Contributors > Average Users**.

Long works are the most-abandoned content. This layer rewards *completion* over *consistency* of short bursts, so an author who finishes a 50k-word fic outranks one who publishes 50 × 1k-word chapters but never lands an ending. All bonuses are anti-gaming: they require the work to have a `complete` status, a minimum word count, and (for passive/dividend rewards) decay over time.

### New `event_type`s

| event_type | xp | when | anti-game |
|---|---|---|---|
| `work_complete_qualified` | `100 + 2 * (word_count/1000)` | status flips to `complete`, ≥5k words | one per work; `source_ref=url_id` dedup |
| `completion_streak_bonus` | `50 * streak_count` (capped 500) | N completed works in 60d | escalates with streak, hard-cap 10 streaks |
| `long_work_dividend` | `10/min` passive (see below) | ongoing, for as long as the completed work is favorited/bookmarked | weekly source-ref cap 2,880; decays 30d inactivity |
| `marathon_writer` | 1000 | ≥3 works each ≥50k words, all `complete` | admin-gated; lifetime title |

### 1. Completion bonus — `work_complete_qualified`
- Awarded at the moment a work's status flips to `complete`.
- XP = `100 + 2 × floor(word_count / 1000)`. A 50k-word fic = 100 + 100 = 200 XP; the 80k fic that users begged for = 100 + 160 = 260 XP. Longer = bigger, but scaled so the *per-chapter* drop still matters (no cliff rewarding terseness).
- One per work (`source_ref = url_id`, dedup guarded by a unique-on-(user_id,event_type,source_ref) on the events table or engine-level count check).

### 2. Completion streak — `completion_streak_bonus`
- Counts distinct `work_complete_qualified` events in the last 60 days for that author.
- On the Nth completion in the window, award `50 × N` XP (capped at 500 per event). Completing 5 works in 60d → 5th gives +250.
- This is the engine's streak mechanism (`xp_sources_streaks` table) tuned to a 60-day window + `work_complete_qualified` source.

### 3. Long-work passive dividend — `long_work_dividend` (Reputation-side passive)
- A *completed* ≥20k-word work that is favorited or bookmarked emits `10 XP/minute` of reading-time-equivalent, **capped at 2,880/week per work** (`daily_cap`-equivalent in `xp_source_defs`, `rate_limit_per_period=2880`, window=`7d`).
- "Reading-time-equivalent" = `SUM(favorites + bookmarks) × minutes_read(words)` — computed in `src/services/content_scan.rs` or a dedicated `long_work_dividend` cron. This turns reader engagement with long works into passive XP for the author, incentivizing the *kind* of long works readers stick with.
- **Decay guard**: dividend for a work stops accruing if no engagement (new favorites/bookmarks) for 30 days — prevents zombie long-work farming. Re-arms when the work gets fresh engagement.

### 4. Marathon writer title — `marathon_writer` (admin-verified)
- 1,000 XP + a permanent `marathon` title for authors with ≥3 completed works each ≥50k words.
- **Admin-gated only** (description = `Admin: marathon writer`). No automated path — awarded via the admin `/api/admin/reputation/award` handler when a curator confirms the qualifying works list. Prevents gaming via a single inflated 200k-word dump.

### Wiring notes
- `work_complete_qualified` fires from the same approval path that fires `work_publish` (`src/routes/admin.rs:116-133`); the word-count qualifier already fetches `fi.word_count` there — read it and scale the bonus.
- `long_work_dividend` is a new cron entry (`src/cron/` or `src/bin/compute_leaderboards.rs` sibling) — deferred to the cron-batch, not this deploy's core.
- The `xp_sources_streaks` table already supports `streak_window_days` — set `work_complete_qualified` window=60, multiplier baked into the streak formula (50×N) rather than a numeric multiplier (override the column to a special `streak_bonus_per` field, or compute the +50×N in-app and call `award_xp` with the base 50 then a second `completion_streak_bonus` event).

### Hierarchy guarantee
- An unfinished-but-consistent author gets `work_chapter` (15 × N, capped 150/day) — bounded.
- A finisher of one long work gets `work_publish`(100) + `work_complete_qualified`(200+) + at least one `work_chapter` streak — immediately ahead.
- A finisher of several long works hits `completion_streak_bonus` (250 on the 5th) — compounds past the consistent-but-unfinished author.

---

## Current state (verified live on thinkcentre 2026-08-26)

**Two XP/Reputation paths — now unified and live.**

The **dormant v3 XP engine** (`src/services/progression.rs:21`, `award_xp`) is **alive as of this deploy**:
- `award_xp` now returns `NotFound("Unknown event_type")` only if the `event_type` is NOT in `xp_source_defs` — and the `010_reputation_bounties` migration seeds 7 source defs (`work_chapter`, `engagement_dividend`, `curation_tier1`, `curation_tier2`, `translation_submit`, `referral_new_user`, `idle_tick`, plus internal `level_up`).
- `award_xp` enforces **daily cap + cooldown + per-week rate_limit + streak multiplier** (all new columns from 010).
- **Level-up resets `users.xp` to the residual** (overflow past the level threshold), logs a `level_up` event with `xp_residual` — XP is *not* cumulative across levels (early levels are cheap, L99→L100 is `50 * 100^1.6` ≈ 31,623 XP alone — verified by `level_curve_expensive_late` test).

The **legacy reputation path** (`users.reputation` + `update_reputation_and_promote`) is now the **spendable currency ledger**:
- `reputation_events` accumulates lifetime (never resets) — used for bounties.
- `users.reputation` is the spendable balance users stake on bounties (`MIN_REP_FOR_BOUNTY = 50`).

**Bounties are LIVE** (`src/services/bounties.rs` + `src/routes/bounties.rs`, mounted in `server.rs:393-397`):
- `POST /api/bounties` — create + stake rep (creator's `users.reputation` decremented, audit row in `reputation_events` as `bounty_staked`).
- `POST /api/bounties/{id}/claim` — claimant submits proof; pot moves `open→claimed`, stores `claimant_id`.
- `POST /api/bounties/{id}/resolve` — **admin-only** (`role >= 10`); `resolved` pays claimant `amount - 10%` curator fee (fee burns), `cancelled` refunds creator, `slashed` pays nothing (staked rep stays burned).
- `GET /api/bounties` — public list of open bounties.
- `POST /api/xp/idle-tick` — passive 2 XP/tick, capped 1440/day (presence reward).

**Approval integration (admin.rs:116):** when an admin approves a manual upload of a work with `word_count >= 5000`, the system auto-claims any open `work` bounty whose `target_ref` matches the work's url_id (the "next chapter of this fic" use case), then fires `award_xp(... "work_publish", Some(url_id))`. Legacy `upload_approved` +25 reputation still fires during the transition sprint so existing `/leaderboard` keeps moving.

## Key decisions

1. **Canonical ledger = `xp_events` + `xp_source_defs`.** All NEW rewards go through `award_xp`. Do NOT add flat inline deltas (that's the legacy anti-pattern).
2. **`award_xp` needs extension for streaks + quality thresholds** (see Task 3). The engine currently only does per-event `daily_cap`. Add optional `cooldown`, `min_qualifier_sql`, and streak multiplier support via new config columns.
3. **Leaderboard source switch:** recompute `compute_weekly_leaderboard`/`compute_monthly_leaderboard` to read `xp_events` instead of `reputation_events`. On deploy the current week/month leaderboard recalcs from zero — acceptable; note it in release notes. (Migration: backfill `xp_events` from `reputation_events` one-time so existing contribution history isn't lost — Task 4.)
4. **Decay = leaderboard is already windowed** (`week_start`/`month_start`). The brainstorm's "XP earned this year" is satisfied by the monthly snapshot; no new decay column needed. Keep cumulative `users.xp` for leveling; leaderboard reflects recent activity.
5. **Admin-verified meta rewards** (scraper/selector bounties, code PRs, tag-wiki merges, QA bugs viewed as merged/fixed) go ONLY via new `POST /api/admin/reputation/award` (manual, modlogged). No automated path for these.
6. **Migration numbering:** new migration is `009_xp_rewards.sql`. Never fold into `001_initial.sql`; never reuse a version on prod.

---

## Migration `009_xp_rewards.sql`

### 9a. Extend `xp_source_defs`
```sql
ALTER TABLE xp_source_defs
  ADD COLUMN IF NOT EXISTS cooldown_seconds bigint,           -- min gap between identical awards
  ADD COLUMN IF NOT EXISTS min_value bigint,                  -- optional threshold (words, etc.) — enforced in app code
  ADD COLUMN IF NOT EXISTS rate_limit_per_period bigint,      -- per-week cap (async, e.g. engagement dividend)
  ADD COLUMN IF NOT EXISTS streak_window_days integer,        -- window to count a streak, e.g. 30
  ADD COLUMN IF NOT EXISTS streak_multiplier numeric;         -- applied when streak count reached
```

### 9b. Seed the new `event_type`s (all values final-tune in review)

| event_type | xp | daily_cap | notes / anti-game |
|---|---|---|---|
| `work_publish` | 100 | — | ≥5,000 words, properly tagged, not deleted in 7d (checked at award time via `min_value`) |
| `work_complete` | 150 | — | awarded when status flips to `complete` |
| `work_chapter` | 15 | 150 | ≥1,000 words/chapter (`min_value=1000`) |
| `work_chapter_streak_bonus` | 22 | — | streak_multiplier 1.5 on update 5 within 30d |
| `engagement_dividend` | 1 | 50 | +1 per 10 bookmarks, +2 per 5-star review, +5 added to public Reading List; `rate_limit_per_period=50`/week per fic |
| `series_architect` | 100 | — | series with 3+ linked works in order |
| `curation_tier1` | 15 | 70 | first 5 tag curations/translations in a week |
| `curation_tier2` | 20 | 70 | next 5 in same week |
| `translation_submit` | 15 | 70 | migrate from legacy +15 |
| `request_solved` | 20 | — | requester marks answer Accepted |
| `request_answer_upvoted` | 5 | 100 | per community upvote |
| `helpful_review` | 10 | 50 | author replies to / marks review Helpful |
| `beta_read_ack` | 25 | — | author tags user as beta/sensitivity reader |
| `referral_new_user` | 10 | 50 | new user registered + bookmarked via `?ref=` link |
| `scraper_fix_merged` | 100 | — | admin-awarded only |
| `scraper_selector_submitted` | 30 | — | admin-awarded only |
| `code_pr_core` | 300 | — | admin-awarded only |
| `code_pr_adapter` | 150 | — | admin-awarded only |
| `code_pr_docs` | 50 | — | admin-awarded only |
| `tag_wiki_author` | 20 | — | admin-awarded (Roadmap P8#78) |
| `tag_wiki_approved` | 10 | — | admin-awarded |
| `qa_bug_fixed` | 15 | — | admin-awarded |

Keep existing legacy `event_type`s (`manual_upload`, `upload_approved`, `proposal_approved`, `login_streak_milestone`, `badge_*`, `quest_*`) too — backfilled (9c) and now also manageable via `xp_source_defs` going forward.

### 9c. One-time backfill from `reputation_events` → `xp_events`
```sql
INSERT INTO xp_events (user_id, event_type, xp, source_ref, created_at)
SELECT user_id, event_type, points, COALESCE(reference_id, reference_type), created_at
FROM reputation_events
ON CONFLICT DO NOTHING;
```
(`xp_events` needs a unique constraint on `(user_id, event_type, source_ref, created_at)`-ish or rely on this run being idempotent via a guard — see Task 4.)

### 9d. `xp_events` index
```sql
CREATE INDEX IF NOT EXISTS idx_xp_events_created ON xp_events (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_xp_events_type ON xp_events (event_type, created_at);
```

---

## Migration `010_reputation_bounties.sql`

> **Note:** numbering moved from the plan's `009` to **`010`** to avoid colliding with a reserved `009` slot on prod. Never reuse a version.

### 010a. Bounty tables
```sql
CREATE TABLE bounty_pots (id serial PRIMARY KEY, creator_id int REFERENCES users, target_type text, target_ref text, amount int CHECK (>0), goal_desc text, expiry_at timestamptz, status text DEFAULT 'open', claim_ref text, claimant_id int REFERENCES users, resolver_id int, resolved_at timestamptz, created_at timestamptz DEFAULT now(), CONSTRAINT ... CHECK (status IN (...)) OR claim_ref IS NOT NULL));
CREATE INDEX idx_bounty_pots_status_created, idx_bounty_pots_creator;
CREATE TABLE bounty_resolutions (id, pot_id, resolver_id, action, note, rep_moved, created_at);
```

### 010b. XP event columns for level-up semantics
```sql
ALTER TABLE xp_events ADD COLUMN level_up boolean DEFAULT false, ADD COLUMN xp_residual int DEFAULT 0;
```

### 010c. Seeded `xp_source_defs` (config-driven rewards)
7 new event_types: `work_chapter`(15xp), `engagement_dividend`(1), `curation_tier1`(15), `curation_tier2`(20), `translation_submit`(15), `referral_new_user`(10), `idle_tick`(2), plus internal `level_up`(0). Each with `daily_cap`/`rate_limit_per_period` anti-game bounds.

### 010d. Streak config table
```sql
CREATE TABLE xp_sources_streaks (event_type text REFERENCES xp_source_defs, boost_at int, multiplier numeric);
INSERT ('work_chapter', 5, 1.5);
```

### 010e. Indexes
`idx_users_xp`, `idx_users_reputation` on `users`.

---

## Tasks — COMPLETED (shipped in this deploy)

| Task | What shipped | Files | Tests |
|---|---|---|---|
| **1. Core engine cap/cooldown** | `award_xp` now reads cooldown + daily_cap + rate_limit + streak_multiplier in one def row; inserts 0-xp audit rows when capped/cooling; level-up within TX | `src/services/progression.rs:21-201` | +2 (level curve + reset) |
| **2. Curried helpers** — *deferred.* Streak applied inline in `award_xp` (rows 96-117) rather than a separate `rewards.rs` helper; qualifier (word-count ≥5k, ≥1k) gated at the **call site** (admin.rs:116). No `rewards.rs` created yet. | — | — |
| **3. Reward sites** | `admin.rs:116` approval now: auto-claims work bounties + fires `work_publish` XP; legacy `upload_approved` reputation still dual-writes during transition | `src/routes/admin.rs:116-133`, `src/services/bounties.rs:253` | existing api_contract + lib 735 pass |
| **4. Leaderboard** — *deferred.* Still reads `reputation_events` via `compute_leaderboards`; reputation is spendable currency so the dual path holds for one transition sprint. Backfill 9c NOT applied (no `009` run). | — | — |
| **5. Tier 1 author sites** — *deferred.* Only `work_publish` wired so far; `work_chapter`/`engagement_dividend`/streaks at scraper/chapter boundaries pending. | — | — |
| **6. Tier 2/3 + admin award** | `bounties.rs` (create/claim/resolve) + routes `server.rs:393-397` + idle-tick; NO `/api/admin/reputation/award` yet (use `/api/bounties/{id}/resolve` instead) | `src/services/bounties.rs`, `src/routes/bounties.rs` | +3 (payout math, fee, gate) |
| **7. Anti-gaming** | Caps enforced in engine (daily/period/cooldown); `MIN_REP_FOR_BOUNTY=50` blocks fresh accounts; 10% curator fee burns on resolve | same | gate test |
| **8. Frontend** — *deferred.* No UI changes this deploy; bounty endpoints are API-only, consumable by the frontend `/bounties` page next. | — | — |
| **9. Docs** | This plan + code-along updated in-tree; `brainstorm-02-roadmap-status.md` NOT yet touched. | `docs/plans/*` | — |

**Status:** 1, 3, 6(partial), 7, 9(this plan) done. 2(deferred→inline), 4, 5, 8 deferred to next sprint.

---

## Validation (run this deploy)
- `cargo test --lib` — **735 passed, 0 failed** (730 baseline + 5 new: `level_curve_expensive_late`, `level_up_reset_zeroes_xp_after_threshold`, `payout_math_resolved`, `payout_math_slashed_yields_zero`, `min_rep_gate_blocks_zero_rep_creator`).
- `cargo build --release --bin fichub` — clean, binary deployed & `systemctl is-active fichub → active`, health 200 on thinkcentre.
- `strings $(binary) | grep /api/bounties` → present (routes mounted).
- Migration 010 applied via `fichub migrate` (dry-run confirms `_sqlx_migrations` at 10 rows; actual apply skipped here since `FICHUB_SKIP_MIGRATIONS=1` on the runtime unit — run `fichub migrate` once on deploy box to apply).

## Risks / tradeoffs (live)
- **Dual-write window:** `award_xp` (xp_events) + legacy `update_reputation_and_promote` (reputation_events) both fire on approval. Leaderboard still reputation-based; XP path is the future. Delete shim after switching `compute_leaderboards` to `xp_events` (Task 4).
- **`users.exp` (i64) vs `users.xp` (i32)**: `exp` untouched; `award_xp` writes `xp`/`level`/`rank` only. Do NOT migrate `exp` until `/api/me/progression` and quests are confirmed not reading it.
- **Streak uses `source_ref`** (url_id) for counting — `idx_xp_events_type` + window count. Fine for `work_chapter`; will need `source_ref`-scoped streaks if per-work streaks required.
- **Re-apply note:** migration `009` is *not* in this deploy (renamed to `010`). If `009` is later created by another branch, numbering will conflict — keep this file as `010` only.


## Files likely to change (all)
- `migrations/009_xp_rewards.sql` (NEW)
- `src/services/progression.rs`, `src/services/rewards.rs` (NEW), `src/services/mod.rs`
- `src/db/queries.rs` (leaderboard compute, shim, promotion)
- `src/routes/upload.rs`, `src/routes/locales.rs`, `src/routes/work_proposals.rs`, `src/routes/requests.rs`, `src/routes/social.rs`, `src/routes/admin.rs`, `src/routes/referrals.rs` (NEW), `src/server.rs`
- `src/bin/compute_leaderboards.rs`
- `frontend/src/routes/progression/+page.svelte`, 6 i18n dicts
- `docs/brainstorm-02-roadmap-status.md`, `SPECIFICATION.md`, `docs/USER-ACTIONS.md`
- `src/routes/mod.rs`, `src/routes/subsystems.rs` (module registration)
- Tests as listed per task
