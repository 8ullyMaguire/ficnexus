# Recommendation Engine Design Brainstorm — v3.1 Extension Platform

> Pair with: `docs/plans/reputation-xp-remaining-work.md` (A–D backlog),
> `src/recommender/` (19 strategy modules), `src/server.rs` (6 recommendation
> routes), and the v3.1 spec (`docs/v3-customization-spec.md`).

## Current state (static analysis, 2026-08-26)

The recommendation engine (`src/recommender/`) has **13 strategy implementations**
of the `RecStrategy` trait, but only **3 are active by default** via the
`REC_STRATEGIES` env (parsed in `registry.rs:30`, default `cooccur`):

| Strategy | File | Registered? | Live by default? | Purpose |
|----------|------|-------------|------------------|---------|
| `cooccur` | `engine.rs`/`legacy_cooccur.rs` | ✅ | ✅ | Bookmark co-occurrence (legacy, default) |
| `embeddings` | `embeddings.rs` | ✅ | opt-in | pgvector dot-product |
| `bandit` | `bandit.rs` | ✅ | opt-in | ε-greedy feedback loop |
| `mf` | `mf.rs` | ✅ | ❌ | Matrix factorization |
| `clusters` | `clusters.rs` | ✅ | ❌ | Tag-affinity clusters |
| `sequential` | `sequential.rs` | ✅ | ❌ | Reader path Markov |
| `author_graph` | `author_graph.rs` | ✅ | ❌ | Co-bookmark by author |
| `tag_graph` | `tag_graph.rs` | ✅ | ❌ | Shared-tag propagation |
| `hybrid` | `hybrid.rs` | ✅ | ❌ | Weighted composition wrapper |
| `external` | `external.rs` | ✅ | ❌ | Python sidecar proxy |
| `curator` | `curator.rs` | ✅ | ❌ | Editorial picks |
| `decay` | `decay.rs` | ✅ | ❌ | Time-decayed cooccur |

Routes wired in `server.rs`:
- `GET /api/recommendations` — main recs (uses `REC_STRATEGIES`, defaults to `cooccur`)
- `GET /api/recommendations/personal` — personalized (logged-in)
- `GET /api/recommendations/embeddings` — raw embeddings endpoint
- `GET /api/recommendations/strategies` — admin: list/select active strategies
- `POST /api/recommendations/suggest` — ask-style "what should I read?"
- `POST /GET /api/recommendations/vote` + `/votes` — feedback loop
- `POST /api/recommendations/train` — rebuild model tables (admin/cron)

### The gap
`REC_STRATEGIES` is operator-configured (env var). There is **no user-facing
control** — a reader can't prefer "tag-graph" over "embeddings" per-request,
and the shadow-run dashboard (roadmap P3#8) that would surface per-strategy
engagement was described as "half-built." Users who want "more like my
recent shelf" vs "discover something weird" have no lever.

### The duplication
- **`cooccur` (live) vs `decay` (dormant)**: `decay` is "time-weighted cooccur"
  — same data, different score. With a decay window, `cooccur` could absorb it.
- **`author_graph` vs `tag_graph` vs `clusters`**: three strategies that all
  compute tag-Jaccard / co-bookmark at different grains. `tag_graph` is the most
  general; `author_graph` is tag-graph restricted to author-to-author.
- **`hybrid`**: a composition wrapper, not a real scorer — pure plumbing.

## Ideas (ranked by leverage)

### 1. User strategy preference — the high-leverage move
> **Goal**: let users pick their recommendation lens, not the operator.
- **Schema**: `user_prefs.rec_strategy = 'cooccur' | 'embeddings' | 'bandit'`
  (default: `embeddings` for logged-in, `cooccur` for guest). A `rec_strategy`
  pref in `user_prefs` (already the prefs table).
- **API**: `GET /api/recommendations?strategy=embeddings` — explicit per-request
  override. Falls back to pref, falls back to env default.
- **Frontend**: a small "Why these?" dropdown on the `/recommendations` page
  (or the reader Next-Up panel) toggling the strategy. Show a 1-sentence reason
  per strategy ("Because you bookmarked…") — the `external` sidecar already
  returns a `reason` field; surface that.
- **Cost**: 1 pref key × 6 locales, ~30 LOC in the endpoint + ranker.

### 2. Recommendation marketplace / plugin system
> **Goal**: treat recommendation engines as interchangeable, user-mixed plugins
> — the v3.1 "extension platform" applied to recs.
- **Schema**: `rec_engines` table (id, slug, name, description, entry_kind:
  'internal' | 'http' | 'sqlite_fn', weight, is_active). Mirrors the existing
  `recipes` + `plugin_manifest` pattern from v3.1.
- **Mix**: a user's rec blend = weighted combo of up to 3 active engines,
  configurable in UI. `hybrid` becomes a user-facing "blend" widget, not a
  code-level wrapper.
- **Sidecar contract**: formalize `external.rs`'s JSON protocol into
  `docs/rec-engine-contract.md`. Any HTTP service that speaks it becomes a
  plug-in engine (LightFM, RecBole, even another FicHub instance).
- **Plugin manifest**: `rec_engines.active` + `user_rec_mix` (user_id, engine_id,
  weight). `/api/recommendations/strategies` already lists + selects — extend
  it to persist the user's mix.
- **Hook**: `rec_engines.on_score` = optional `sqlite_fn` (a user-authored
  SQL expression over `fic_tags`) for power users who want custom tag boosts.

### 3. Shadow-run dashboard → recommendation report card
> **Goal**: give operators + power users visibility into which strategy
> actually performs, not just which one is configured.
- **Signal**: reuse the existing `rec_impressions` table (logged by the ranker in
  `engine.rs:173` fallback) — it already stores `strategy_used` +
  `impression_type` + `user_id` + `outcome` (click/read/streak).
- **Dashboard**: `/admin/analytics/recommendations` showing per-strategy CTR,
  conversion-to-read, 7-day survival (did they abandon after 1 chapter?).
  500×500 scatter: strategy X vs strategy Y engagement.
- **Auto-promote**: if `embeddings` beats `cooccur` on CTR for 7 days, flip the
  default — no config redeploy. (This was the "shadow-run" P3 item.)

### 4. Consolidate the 3 near-identical graph strategies
> **Goal**: cut `author_graph` + `clusters` into a parameterized `tag_graph`.
- `tag_graph` already computes tag-Jaccard. `author_graph` is the same query
  restricted to (authorA → authorB). `clusters` is tag-Jaccard + K-means.
- **Action**: one `tag_graph` strategy with params `scope = 'work' | 'author'
  | 'cluster'` + `decay_days`. `coexist` as `name=tag_graph,scope=author,...`.
- Saves ~340 lines + a maintenance surface, one less `RecStrategy` impl to test.

### 5. Sequential reader = Next-Up wiring
> **Goal**: the `sequential` Markov chain isn't a *recommendation* strategy —
> it's the reader's "what do I read next in this series" flow.
- **Action**: move `sequential.rs` from `recommender/` to `src/reader/` (or
  `src/reader/sequel.rs` — `/api/reader/{url_id}/sequel` already exists).
  It consumes `rec_impressions`/`reading_history` to rank the next logical
  chapter, not the rec engine's candidate set.
- Frees the rec engine from reader-flow concerns + clarifies the boundary.

### 6. Decay vs cooccur fusion
> **Goal**: kill `decay.rs` by folding time-weighting into `cooccur`.
- `rec_author_graph` (used by `author_graph`) and `rec_impressions` both have
  timestamps. `cooccur`'s query just needs a `WHERE created > NOW() - '30 days'`
  filter + a linear decay factor.
- Removes one more strategy impl; the "decay" behavior becomes a cooccur option.

## Recommendation (top 3 to build)

1. **User strategy preference** (Idea 1) — 1 hour, highest user signal. Lets a
   reader say "I want embeddings, not cooccur" and see why. Unblocks
   shadow-run data collection (Idea 3).
2. **Shadow-run dashboard** (Idea 3) — turns the 9 dormant strategies from
   "configured but unused" into "measured, pick the winner."
3. **Graph consolidation** (Idea 4 + 6) — cut 3 strategies to 1 parameterized
   one, delete `decay`. Cleans the surface so Ideas 1+3 have a sane set to
   report on.

Ideas 2 (marketplace) and 5 (sequential→reader) are v3.2/extension-platform
scope — big lifts with their own schema + plugin manifest work.

## Open questions

- Does `rec_impressions` store enough to compute per-strategy CTR today, or does
  the ranker need to start logging `strategy_used`? (Check
  `src/recommender/engine.rs` ranker + `queries.rs::record_rec_impression`.)
- Is the `/api/recommendations/suggest` (ask-style) endpoint the same as
  `/api/search/suggest` (search-as-you-type)? They have different names —
  verify they don't duplicate.
- `external.rs` sidecar returns `reason` per rec — is that surfaced today?
  (Currently not in the frontend rec render path.)
