# Reputation XP + Trust System — Remaining Work

> Status 2026-08-26. Tracks what's left after the `010_reputation_bounties.sql`
> deploy (binary md5 `0e391891`, 735 lib tests green, migration 010 applied to prod).
> Canonical plan stays `docs/brainstorm-02-roadmap-status.md`; this file is the
> step-by-step backlog for the XP/trust subsystem specifically.

---

## Shipped (done + live on production thinkcentre)

| Item | Where | Verified |
|---|---|---|
| `award_xp` v3 engine — daily cap + cooldown + rate-limit + streak | `src/services/progression.rs:21` | 735 tests, 0 failed |
| Level-up resets `users.xp` to residual (curve `50*L^1.6`, L99→L100 ≈ 31,623) | `progression.rs` + `xp_events.level_up` | `level_curve_expensive_late` + `level_up_reset_zeroes_xp_after_threshold` tests |
| `work_publish` qualifier (≥5k words) at approval path | `src/routes/admin.rs:116-145` | binary has string; health=200 |
| `work_complete_qualified` scaled bonus (`100 + 2×words/1000`) on `status=complete` | `admin.rs` + `award_scaled_xp` | binary has both strings |
| Bounty staking/claim/resolve/cancel + idle-tick | `src/services/bounties.rs` + `/api/bounties` | routes mounted, health 200 |
| Migration 010: def-extension columns + 17 seed defs + streaks + bounty tables | `migrations/010_reputation_bounties.sql` | versions 1-8+10 applied; `xp_source_defs` has cooldown/rate_limit/streak cols |
| Trust-levels plan doc | `docs/plans/trust-levels-rework.md` | written, committed `681b9e9`; **frozen** (D dismissed) |
| A1 B3 audit + deploy | `docs/plans/code-along-xp-trust-complete.md` + `docs/plans/codebase-audit-low-value-features.md` | A1+B3 ✅ marked done; full low-value feature scan committed (`docs/plans/code-along-xp-trust-complete.md` Step D)

## Remaining work

### Tier A — Engine correctness (backend)

#### A1. Idle-tick deduplication ✅ DONE (deployed `12:07`, merged `9e96026`)
- **Solution**: per-user hourly time-bucket as `source_ref`.
  `bounties::idle_tick` computes `Utc::now().format("%Y-%m-%dT%H")` and passes it
  to `award_xp("idle_tick", Some(&hour_bucket))`. The engine's
  `cooldown_seconds=3600` (set by `011_idle_tick_cooldown.sql`) dedups calls with
  the same `(user_id, "idle_tick", hour_bucket)` within 3600s — so a client looping
  the endpoint 1440×/day gets **one** award per hour (2 XP × 24 = 48/day max).
- **Where**: `migrations/011_idle_tick_cooldown.sql` + `src/services/bounties.rs:216-230`.
- **Test**: `hour_bucket_truncates_to_hour` + `idle_tick_hour_bucket_is_stable_within_same_hour` — both green (`cargo test --lib 5/5`).

#### A2. Chapter-path + engagement wiring ⏳ TODO
- **Problem**: `work_chapter` and `engagement_dividend` only fire at admin
  *approval* (`admin.rs:116`), not when scrapers actually append chapters to an
  existing work. The "next chapter of a fic" bounty auto-claim only triggers on
  manual-upload approval, not on scraper chapter appends.
- **Where**: find the chapter-append path. Chapters live as a `jsonb` array in
  `fic_info.chapters`. The builder/scraper that mutates that array (`src/scrape/`,
  `lookup_authed`/`fetch_chapters`) is the hook point.
- **Also**: `engagement_dividend` (bookmark/5-star/list/follow) fire from
  `src/routes/social.rs` (bookmark, rating, review, follow) → call
  `award_xp("engagement_dividend", Some(fic_url_id))`.
- **Test**: grep `award_xp.*work_chapter` across `src/` — should show ≥2 call sites.

#### A3. Leaderboard switch + backfill ⏳ TODO
- **Problem**: `compute_weekly_leaderboard`/`compute_monthly_leaderboard` still
  read `reputation_events`. Reputation is now spendable currency (bounty stake),
  so the XP ledger (`xp_events`) is the correct source.
- **Where**: `src/db/queries.rs` (4 queries at :1605/:1638), replace
  `FROM reputation_events` → `FROM xp_events`, `SUM(points)` → `SUM(xp)`.
- **Backfill**: one-time migration to copy `reputation_events → xp_events`
  (the plan's §9c). New migration `011_leaderboard_xp_events.sql` (do NOT reuse
  `010`; that shipped — continue numbering). Idempotent guard.
- **Test**: `psql -c "SELECT count(*) FROM xp_events"` ≥ previous
  `reputation_events` count after backfill.

### Tier B — Tiered reward completeness (~3 sessions)

#### B1. Long-work passive dividend cron
- **Spec**: completed ≥20k-word work + favorited/bookmarked → 10 XP/min
  reading-time-equivalent, capped 2,880/wk/work, decays 30d no-engagement.
- **Where**: new `src/bin/long_work_dividend.rs` (mirror `compute_leaderboards`
  bin structure), scheduled via `scripts/install_systemd_timers.sh` nightly.
- **Formula**: `SUM(favorites + bookmarks) × minutes_read(words) × 10`, clamped
  to weekly cap per work per author. Decay: skip works with no engagement in 30d.
- **Test**: bin compiles + dry-run prints candidate works without awarding.

#### B2. Marathon-writer admin endpoint + title
- **Spec**: 1,000 XP + permanent `marathon` title for ≥3 completed works each
  ≥50k words. **Admin-gated only.**
- **Where**: add to the admin award endpoint (B3) allowlist; `marathon_writer`
  def already seeded as `Admin:` row.
- **Title display**: `users.marathon boolean` column? No — reuse `rank`? Add a
  `users.badges jsonb` column or a `user_badges` join table. (TODO: confirm
  schema — the plan doc leaves this open.)
- **Test**: `curl /api/admin/reputation/award {marathon_writer}` → 401 for non-admin.

#### B3. Admin reputation-award endpoint ✅ DONE (merged `9e96026`)
- **Spec**: `POST /api/admin/reputation/award` → `award_xp(user_id, event_type, source_ref)`.
- **Where**: `src/routes/admin.rs:380-424` + mounted at `server.rs:833`.
  ```rust
  // Gated twice: role >= 10 + description LIKE 'Admin:%' allowlist.
  // Non-admin event_types (e.g. work_chapter) → 400 "Not an admin-awarded reward".
  // Every grant writes a modlog row (reputation_award).
  ```
- **Test**: `cargo check --bin fichub` EXIT=0; 5 bounty tests green.
- **Commit**: `00043e8` on `feat/admin-reputation-award`, merged to main.

### Tier C — Frontend (2-3 sessions)

#### C1. Bounty page + claim flow
- `/bounties` (list open + user's staked), create modal (stake rep from dropdown),
  claim (proof input: free text / work url_id / PR ref), curator resolve modal.
- 12 new i18n keys × 6 locales (`bounty.create`, `bounty.claim`, `bounty.resolve`,
  `bounty.stake`, etc.). **No emoji** in UI text (project rule).
- Test file: `frontend/src/routes/bounties/page.test.ts`.

#### C2. Progression page
- Recent XP events feed from `GET /api/me/progression`.
- Level-up toast: `award_xp` returns the residual → frontend shows "Level N! +M XP overflow".
- Test file: `frontend/src/routes/progression/page.test.ts`.

### Tier D — Trust levels (✅ SHIPPED 2026-08-27)

> **Implemented** as a 7-level axis (TL0–TL6), not the TL0–TL4 sketched here.
> Reopen `docs/plans/trust-levels-rework.md` (status banner at top) for the
> shipped architecture, or go straight to `src/services/trust.rs`.
>
> Answers to the three open questions below, as built:
> 1. **Axis model** → a dedicated trust axis (`users.trust_level`), separate
>    from XP/level/rank; it sandboxes (TL0 cannot flag) and unlocks
>    (TL2 publishes, TL5 moderates) rather than gating reads.
> 2. **Curator power** → trust grants *report resolution* at TL5+ only
>    (staff-designated, surfaced via the weekly digest); `role >= 10` remains
>    the sole authority for hide/ban/admin actions.
> 3. **Reader data** → promotions read engagement signals (works/words read,
>    days active, forum posts, reviews) via cached `tl_metrics` jsonb computed
>    by `src/bin/trust_promote.rs`.

Previous D content (retained for reference):
- `011_trust_levels.sql`: `trust_level smallint`, `tl0_until`, `tl_metrics jsonb`.
- `src/cron/trust_promote.rs`: hourly TL0→TL4 promoter + TL3 revocation.
- Gate checks: swap `role < 10` → `trust_level >= 1` on PM/flag/hyperlink/image/reply/topic.
- `/forum/lounge` TL3-only category + welcome notices.

Three open questions for the replan (from `trust-levels-rework.md`):
1. **Sandbox vs feature-unlock vs fold-in-existing?** One of: (1) a TL axis that
   limits new users (writes gated until read), (2) gated UI features unlocked by
   activity, or (3) reuse `level` + `reputation` and drop the fourth axis.
2. **Curator power**: should TL3 readers gain curator actions, or does `role >= 10`
   stay the only gate (reading ≠ editorial judgment)?
3. **Reader data source**: `xp_events` is author/reputation-biased; reader signals
   (`total_words_read`, pageview logs) live in a different async pipeline.

---

## Priority order

```
A1 ✅  →  A2 (chapter + engagement)  →  A3 (leaderboard switch)
   →  B1 (dividend cron)  →  B2 (marathon, unlocked by B3)  →  C1/C2 (frontend)
   →  D (trust levels ✅ SHIPPED 2026-08-27 — 7-level axis, TL0–TL6)
```

Rationale: A1 stops the easiest XP farm (idle tick loop); A2 wires the actual
"complete long works" reward at the choke point where chapters land (the
finishing incentive the hierarchy depends on); A3 removes the legacy
dual-write so reputation-as-currency is unambiguous. Frontend (C) lands last
because it's user-visible polish on a correct backend. Trust levels (D) are a
parallel safety system — they don't depend on XP correctness but should share
its data source.

## Open decisions (block B2 + D)

1. **Marathon writer badge storage**: JSON `users.badges` vs `user_badges` join table?
2. **Trust-level like-quotas**: enforce Discourse's cross-user spread, or raw counts?
3. **TL0 consumption**: sandbox writes only, allow reads (recommended: yes).
4. **Marathon writer XP**: should the 1,000 XP use `award_xp` (feeds level) or a
   separate admin event (feeds reputation only)? Per the dual-track design, XP
   is level-progression — so yes, `award_xp("marathon_writer", ...)` to level
   the author, with the *title* being the reputation/status carrot.
