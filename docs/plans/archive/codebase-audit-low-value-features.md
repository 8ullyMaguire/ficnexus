# Codebase Audit: Low-Value / Superfluous Features

> A scan of the codebase for features, endpoints, and UI components that add
> minimal value to the site. Produced in support of the reputation/XP A-D
> implementation backlog — the goal is to flag candidates for **simplification,
> consolidation, or removal** to reduce maintenance burden before layering trust
> levels (or whatever replaces D) on top.
>
> Methodology: static analysis only (grep, svelte-check, svelte-check CSS-unused,
> route registration cross-reference, clippy). No runtime traffic logs consulted.
> "Low-value" = dead code, user-never-sees-it, or functionally-duplicated.

## Tier 1 — Definitely Remove (dead, confirmed)

### 1. `frontend/src/lib/components/SuggestionsTab.svelte` — DELETED ✅
- **What**: A tabbed variant of "fic suggestions" UI.
- **Why dead**: imported **only** in its own test file
  `SuggestionsTab.test.ts`. Never referenced by any route or parent component.
  It calls the **non-existent** `/api/recommendations/votes` + `/recommendations/suggest`
  + `/recommendations/vote` routes — the live equivalent is `FicSuggestionsPanel.svelte`
  (work page `routes/works/[urlId]`), which uses `/api/fic-suggestions`.
- **Status**: Deleted in commit `7b4b2e5` along with `SuggestionsTab.test.ts`,
  `client-legacy.ts`, `client.test.ts`, and `RouteHarness.svelte`.
  `client-extras.test.ts` was kept but its orphaned `fetchVotes` test removed.
- **Risk**: zero. Delete the `.svelte` + `.test.ts`.
- **Also**: removing it drops one of the 141 svelte-check CSS-unused warnings.

### 2. `frontend/src/lib/test/RouteHarness.svelte`
- **What**: Test harness component for routing integration tests.
- **Why dead**: imported only in `navigation.integration.test.ts`; provides no
  runtime value (it's test infra masquerading as a lib component).
- **Risk**: low — keep if tests are maintained, else delete + test file.

## Tier 2 — Redundant / Over-engineered (consolidate)

### 3. Recommendation strategies 13→5 — see brainstorm ✅
- Full idea catalog written to `docs/ideas/recommendation-engine-v31-brainstorm.md`
  (user-strategy preference, rec-engine marketplace/plugin system, shadow-run
  report card, graph-strategy consolidation, cooccur+decay fusion, sequential→reader).
  A full **plugin-system plan** (WASM runtime via wasmtime, A/B + auto-promotion on
  reading-depth metric, marketplace UX) is in `docs/plans/recommendation-engine-plugin-system.md`,
  with a step-by-step **code-along tutorial** at `docs/plans/code-along-recommendation-marketplace.md`.
- **Status**: recommendation engines **kept** per user directive; the brainstorm
  doc proposes making them user-configurable (rec-strategy pref key) + a plugin
  marketplace model where `external.rs` sidecars register as pluggable engines.
- Live set remains `cooccur`/`embeddings`/`bandit`; the 9 dormant strategies
  (`clusters`, `author_graph`, `tag_graph`, `decay`, `hybrid`, `sequential`,
  `legacy_cooccur`, `mf`, `external`, `curator`) stay feature-flagged in the
  `available` pool, gated for A/B testing.

### 4. Quests = reading-history + daily-streak duplication — IN PROGRESS
- `src/routes/quests.rs` (10 handlers) implements: reading list (`/api/reading/list`),
  read recording (`/api/reading/record`), reading history (`/api/reading/history`),
  reading stats (`/api/users/{id}/reading-stats`), streak (`/api/users/{id}/streak`),
  + `GET /api/quests` (daily quests + badges).
- `frontend/src/routes/stats/+page.svelte` calls `/api/reading/analytics` +
  `/api/site/stats` — **separate endpoints** for stats that overlap with
  `/api/users/{id}/reading-stats`. The daily-quest gamification
  ("login streak", "badge_type") is distinct from the streak in `quests.rs`,
  but they draw from the same `reading_stats` / `reading_history` tables.
- **Status**: DONE (commit `591776c`). The daily-quest gamification
  (`get_quests_handler` + `daily_quests` / `user_daily_progress` tables +
  `assign_quests` cron binary + `QuestProgress` frontend type + `/quests`
  route) was scrapped; the genuine reading-tracking endpoints (list, status,
  history, record, streak, reading-stats) were **kept** in `quests.rs` (not
  moved to `analytics.rs` — they're a distinct API surface from
  `/api/reading/analytics` which serves the `/stats` page's aggregate view).
  Dashboard dropped the `getQuests()` call. Verification: `cargo test --lib`
  736 green, prod health 200 md5 `698979b8`.

### 5. `badge` / `badges` feature
- `src/routes/badges.rs` is mounted (`GET /api/badges`, `GET /api/users/{id}/badges`,
  weekly/monthly curator leaderboard) + `frontend/src/routes/badges/+page.svelte`
  renders them with archive/modern modes (6 locales via i18n recipe).
- **Badges flow**: `queries::check_and_award_badges` (queries.rs:1024) fires on
  `award_xp` events (streak completion, work-chapter milestones) + `approve_upload`.
  The B3 admin-award path (`POST /api/admin/reputation/award` in admin.rs) now
  **also** calls `check_and_award_badges` after granting XP — so admin-granted
  sources like `marathon_writer` (migration 011 seed) issue the matching
  `user_badges` record automatically. Wired in commit `7b4b2e5`.
- **Status**: Badges are live, not dead-weight (Tier 2 finding resolved by wiring
  the admin path rather than deleting).

## Tier 3 — Low-traffic / Niche (probably keep, but question)

### 6. Blind-date / surprise discovery
- `src/routes/blind.rs` — `GET /api/blind-date` (hides the title behind a signed
  reveal) + `GET /api/blind-date/reveal`. `frontend/src/routes/blind-date/`.
- Uses `BlindDateCard.svelte` (also a confirmed unused component — see below).
- **Risk**: low — but the frontend `blind-date/+page.svelte` and
  `BlindDateCard.svelte` are orphans. Either wire or drop.

### 7. Archive-stub pages
- `frontend/src/routes/+page.svelte` (434 B): root home page that renders nothing
  in modern (dashboard) mode — `HomeDashboard` handles it via layout. **Intentional**
  (the modern layout renders the dashboard, archive mode renders ArchiveHome).
- `frontend/src/routes/fic/[urlId]/+page.svelte` (290 B): a 308 redirect stub
  to `/works/[urlId]`. **Intentional** legacy path shim.
- These are stubs by design, not dead code — but worth a comment.

### 8. CSS-unused selectors pile (141 warnings)
- `svelte-check` emits 141 "Unused CSS selector" warnings across 35 files.
  Top offenders: `.badge`, `.deleted-badge`, `.hidden-badge` (from the unused
  `Badge` component), `.archive-hello`, `.archive-logout-btn`, `.archive-dd-*`.
- Most map to the Tier 1/Tier 2 dead components — cleaning those up resolves
  a chunk of this cruft.

### 9. Unreferenced widgets in the v3 WidgetDashboard
- `CustomViewsManager.svelte`, `WidgetDashboard.svelte`,
  `LevelUpToast.svelte`, `OnboardingBanner.svelte` — all confirmed imported
  nowhere (the dashboard page at `routes/dashboard/+page.svelte` renders widgets
  but doesn't import these *specific* components).
- **Risk**: low-medium — the dashboard page (405 lines, rank-5+ feature) is
  live, but these supporting components are orphaned. LevelUpToast especially
  should be wired (it's the level-up celebration hook).

## Tier 4 — Not Low-Value (keep, confirmed useful)

- `Progressing` (`src/services/progression.rs`, 447 lines): live XP engine, 28
  call-sites in the codebase.
- `Bounties` (`src/services/bounties.rs`, 369 lines): wired to admin approval
  path.
- `Recipes` + `Recommendations` v3.1 extension platform: `/api/recipes` +
  `/api/recommendations/suggest` are user-facing preference tuning.
  Rec-engine ideas catalogued in `docs/ideas/recommendation-engine-v31-brainstorm.md`.
- `FicSuggestionsPanel.svelte` (work page): live, calls `/api/search`.
- `Stats.svelte` (rank-1+ feature): calls distinct `/api/site/stats` + `/api/reading/analytics`.

## Recommendation

1. **Immediate cleanup** (Tier 1): delete `SuggestionsTab.svelte` + its test +
   `RouteHarness.svelte` if tests are pruned. Resolves 3 of 141 CSS warnings.
2. **Badges consolidation**: wire `badges.rs` routes into the profile UI, or fold
   `GET /api/quests` `badge_type` into a `GET /api/badges` alias and delete the
   duplicate router.
3. **Recommendation strategies**: gate the 9 never-default strategies behind
   `#[cfg(feature = "rec-full")]` and document `cooccur`/`embeddings`/`bandit`
   as the supported set. This keeps them buildable for A/B tests but stops them
   counting against the "live surface" in review.
4. **Quests vs Stats dedup**: unify the reading-stat computation into a single
   `queries::get_user_reading_aggregate` (the `quests.rs` version is already
   the more-complete one) and have `/stats` reuse it.
5. **Trust-levels rework (replan)**: Tier D is dismissed (see
   `code-along-xp-trust-complete.md` Step D) — the four-axis user model
   (role / level / reputation / trust) is the root cause of this audit finding
   too. Folding trust into the existing `level` + `reputation` axes would remove
   ~800 LOC of dead-weight plan vs building a 5th axis.

## Files scanned

- `src/routes/*.rs` (71 modules), `src/services/*.rs` (13 modules),
  `frontend/src/routes/**/*`, `frontend/src/lib/components/*`,
  - `src/recommender/*` (21 files; 13 live strategies + 8 infra: engine/ranker/registry/routes/worker/signals/mod + wasm/ subdir in the marketplace plan).

## Commands used

```bash
cargo check --workspace           # clean build
cargo clippy --workspace          # style warnings only; crate has no dead_code deny
cargo test --lib                  # 737 passing
npx svelte-check                  # 0 errors, 141 CSS-unused warnings
grep / import-scan cross-ref      # 370 backend routes, 210 frontend API calls
```
