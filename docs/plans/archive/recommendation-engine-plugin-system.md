# Recommendation Engine Plugin System — Brainstorm & Plan

## Part 1: Brainstorm

### The core insight: reading depth > CTR

CTR is broken for recommendations. A clickbait title gets clicked; a slow-burn masterpiece gets abandoned after chapter 1. Both register as "success" in a CTR system. FicHub's unique advantage is that it **owns the reader** — it knows how many words a user actually consumed, at what pace, and whether they came back. That's a much richer signal.

**The metric**: `reading_depth_per_impression`

```
score(E) = Σ minutes_spent_reading(fics rec'd by E to user U)
           ÷ impressions_shown_by_E_to_U
           ÷ user_U's_baseline_reading_speed
```

Normalizing by the user's baseline reading speed is the key move — it means a fast reader who blitzes a rec list isn't penalized vs. a slow reader who savors. The metric becomes "how much of a reading session did this engine earn, relative to the user's natural pace."

**Attribution chain** (the data model that makes this possible):

```
Engine E shows fic F to user U at time T
  → rec_impressions(engine_id=E, user_id=U, work_id=F, shown_at=T)

User U opens fic F at time T' > T (within 30-day attribution window)
  → reading_sessions(user_id=U, work_id=F, started_at=T',
                     ended_at=T'', words_read=W, chapters_read=C)

Match on (user_id, work_id) with T' ∈ [T, T+30d]
  → attribute W words / (T''-T') minutes to engine E
```

Multiple engines can claim the same fic if they all recommended it — split credit proportionally to recency, or use "first to recommend" attribution. First-to-recommend is simpler and rewards the engine that got there first.

### Sandbox execution models (ranked)

| Model | Sandboxing | Flexibility | Perf | Fit for FicHub |
|-------|-----------|-------------|------|----------------|
| **WASM (wasmtime)** | ✅ fuel + memory + epoch interruption | High (Rust/C/AssemblyScript/Go) | Good | **Best** — mature, deterministic, no I/O by default |
| Lua (mlua) | ✅ instruction hooks | Medium | Excellent | Good fallback for simple rules, smaller ecosystem |
| SQL expression | ✅ query planner bounds | Low (scoring only) | Excellent | Good for tag-boost tweaks, not full engines |
| HTTP sidecar | ⚠️ network-isolated container | Very high | Variable | Already exists as `external` — keep for heavy ML |
| Rust DSL / proc-macro | ❌ no real sandbox | Low | Best | Reject — untrusted code can't run in-process |

**WASM wins.** wasmtime gives fuel metering (count instructions, abort at limit), linear memory caps, epoch interruption (wall-clock limit), and a clean host-function boundary. A plugin author writes Rust/AssemblyScript, compiles to `.wasm`, uploads. FicHub runs it with a strict budget per request.

### A/B testing design

**Sticky user bucketing** — hash `(user_id, salt) % 100` → bucket 0–99. Buckets are assigned to engines. A user seeing engine E today sees E tomorrow. This prevents the "my recs changed every day" UX disaster.

**Champion/challenger model**:
- 1 champion (the current default — gets ~70% of traffic)
- N challengers (each gets ~10% until one promotes)
- Admin sets max concurrent challengers (say 3) to bound compute

**Minimum sample size**: require 1000 impressions per engine before any promotion decision. Below that, variance is too high.

### Auto-promotion mechanics

**Statistical test**: bootstrap confidence interval on the mean reading depth per impression. If the challenger's 95% CI lower bound exceeds the champion's mean, promote.

**Hysteresis to prevent churn**:
- Promote: challenger must beat champion by ≥5% (relative)
- Demote: champion must fall ≥8% below its pre-promotion baseline
- Cooldown: 14 days between promotions for the same engine

**Rollback**: every promotion snapshots the previous champion's metric. If the new champion degrades past the demote threshold, auto-revert. Log every flip in `rec_promotions` for the audit trail.

### Compute budgeting

Per plugin, per request:
- **Fuel**: 1,000,000 instructions (wasmtime fuel; ~10ms of compute)
- **Memory**: 8MB linear memory cap
- **Wall-clock**: 50ms epoch interruption
- **I/O**: none (plugins are pure functions of their input)

Per plugin, per day:
- **Impression quota**: 10,000 impressions (prevents a popular plugin from starving others)
- **Training budget**: 5 minutes of batch compute

Plugins that hit their quota get throttled, not killed — they continue serving at a reduced rate.

### Marketplace UX

- **Plugin manifest** (JSON): name, author, version, description, entry_kind (`wasm` | `http` | `sql`), input schema, output schema, resource limits requested
- **Publish flow**: author uploads `.wasm` + manifest → curator review → public gallery
- **Gallery**: sort by installs, reading-depth score, "rising" (improvement rate)
- **User control**: "My engines" page — install/uninstall, set weights for blend, see per-engine stats ("This engine earned you 12 hours of reading this month")
- **Fork**: one-click fork of any plugin into your own workspace

### The 13 existing strategies — what happens to them?

Don't throw them away. Wrap each as a **"blessed plugin"** — same manifest, same sandbox, but pre-installed and curated. This:
1. Makes the marketplace credible from day 1 (13 real engines to browse)
2. Lets users mix them (e.g., 60% `cooccur`, 40% `embeddings`)
3. Subjects them to the same A/B and auto-promotion mechanics
4. Gives plugin authors concrete examples to learn from

The `hybrid` wrapper dissolves — users build hybrids in the UI now. The `sequential` strategy moves to the reader (per the earlier brainstorm — it's not a rec engine, it's a reader-flow concern).

### What I'd cut / defer

- **Lua sandbox**: defer. WASM covers the use case; adding two runtimes is maintenance debt.
- **Real-time plugin hot-reload**: defer. Plugins are versioned; updates go through the publish flow.
- **Paid marketplace / revenue share**: defer. FicHub isn't a platform economy.
- **Cross-instance plugin sync**: defer. Out of scope for single-host.

---

## Part 2: Plan

### Phase 0 — Reading depth telemetry (foundation, ~3 days)

**Why first**: nothing else works without this data. The metric is the product.

**Schema** (`migrations/054_reading_sessions.sql`):

```sql
CREATE TABLE reading_sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id),
    work_id INT4 NOT NULL REFERENCES works(id),
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    words_read INT4 NOT NULL DEFAULT 0,
    chapters_read INT4 NOT NULL DEFAULT 0,
    client_id TEXT,              -- for logged-out sessions
    source TEXT DEFAULT 'reader' -- 'reader' | 'export' | 'opds'
);
CREATE INDEX ON reading_sessions (user_id, work_id, started_at);
CREATE INDEX ON reading_sessions (ended_at) WHERE ended_at IS NOT NULL;

-- Per-user baseline reading speed (words per minute), rolling 90-day
CREATE TABLE user_reading_baselines (
    user_id INT4 PRIMARY KEY REFERENCES users(id),
    words_per_minute REAL NOT NULL DEFAULT 200.0,
    sample_size INT4 NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Attribution: which engine showed which fic to which user
ALTER TABLE rec_impressions ADD COLUMN engine_slug TEXT;
ALTER TABLE rec_impressions ADD COLUMN attributed_session_id BIGINT
    REFERENCES reading_sessions(id);
CREATE INDEX ON rec_impressions (engine_slug, shown_at);
```

**Reader integration**: the web reader (`/read/[urlId]`) already saves scroll position. Extend it to:
- Emit `session_start` on first chapter load (client-side)
- Emit `session_heartbeat` every 30s with current word offset
- Emit `session_end` on close/blur/navigate-away (use `visibilitychange` + `beforeunload`)
- Server endpoint: `POST /api/reader/{url_id}/session` — upserts the session row

**Baseline computation**: nightly cron (`src/bin/compute_reading_baselines.rs`) — for each user, compute median WPM over their last 90 days of sessions with ≥5 minutes duration and ≥1000 words. Update `user_reading_baselines`.

**Attribution job** (also nightly, `src/bin/attribute_reading_sessions.rs`):
- For each reading session in the last 24h, find `rec_impressions` where `(user_id, work_id)` match and `shown_at ∈ [started_at - 30d, started_at]`
- Pick the **earliest** impression (first-to-recommend attribution)
- Set `attributed_session_id` on that impression
- If multiple engines recommended, split credit: first gets 1.0, others get 0.0 (simple) OR proportional to recency (future)

**Test**: DB-gated `tests/reading_sessions_api.rs` — seed a session, verify attribution links to the right impression.

### Phase 1 — WASM plugin runtime (~5 days)

**Dependency**: add `wasmtime = "20"` to `Cargo.toml`. (~3MB binary increase, acceptable.)

**New module**: `src/recommender/wasm/`
- `runtime.rs` — wasmtime `Engine` + `Store` pool, fuel/memory config
- `host.rs` — host functions exposed to plugins (read tags, read bookmarks, lookup fic metadata)
- `plugin.rs` — `WasmPlugin` struct implementing `RecStrategy`
- `manifest.rs` — parse and validate plugin manifest JSON

**Plugin contract** (`docs/rec-engine-contract.md`):

```rust
// Plugin exports:
#[no_mangle]
pub extern "C" fn score(
    user_id: i32,
    seed_work_id: i32,       // -1 if none
    candidates_ptr: *const u8, candidates_len: usize,  // JSON array of work_ids
    out_ptr: *mut u8, out_len: *mut usize              // JSON array of {work_id, score, reason}
) -> i32;  // 0 = ok, non-zero = error code

// Plugin can import these host functions:
// - fic_tags(work_id) -> JSON array of tag_ids
// - user_bookmarks(user_id, limit) -> JSON array of work_ids
// - fic_metadata(work_id) -> JSON {title, author, word_count, ...}
// - log(message) -> void  (for debugging, captured in plugin run log)
```

**Sandbox config** (in manifest, enforced by runtime):
```json
{
  "fuel_limit": 1000000,
  "memory_pages": 128,
  "wall_clock_ms": 50,
  "allowed_host_fns": ["fic_tags", "user_bookmarks", "fic_metadata"]
}
```

**Plugin loader**: `src/recommender/wasm/loader.rs` — on service start, read `rec_plugins` table, compile each `.wasm` blob into an `Instance`, cache in `AppState`. Hot-reload via admin endpoint (recompile, swap pointer).

**Test**: `tests/wasm_plugin.rs` — compile a trivial Rust plugin to `.wasm`, load it, call `score`, assert output shape. Fuel-exhaustion test: plugin with infinite loop → aborts cleanly, doesn't hang the server.

### Phase 2 — A/B framework (~3 days)

**User bucketing** (`src/recommender/buckets.rs`):
```rust
fn bucket_for(user_id: i32, salt: &str) -> u8 {
    let hash = sha256(format!("{}:{}", user_id, salt));
    hash[0] % 100
}
```

**Routing in ranker** (`src/recommender/ranker.rs`):
- Read `rec_engine_assignments` table: `(user_id, engine_slug, assigned_at)`
- If user has an assignment, use that engine
- Else: pick from available challengers (round-robin until quota filled), else champion
- Log assignment in `rec_impressions.engine_slug`

**Schema**:
```sql
CREATE TABLE rec_engine_assignments (
    user_id INT4 PRIMARY KEY REFERENCES users(id),
    engine_slug TEXT NOT NULL,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    reassigned_at TIMESTAMPTZ
);

CREATE TABLE rec_plugins (
    id SERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    author_id INT4 REFERENCES users(id),
    description TEXT,
    entry_kind TEXT NOT NULL CHECK (entry_kind IN ('wasm', 'http', 'sql', 'blessed')),
    wasm_blob BYTEA,              -- NULL for http/sql/blessed
    manifest JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    is_blessed BOOLEAN NOT NULL DEFAULT false,  -- system plugins
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE rec_promotions (
    id BIGSERIAL PRIMARY KEY,
    champion_slug TEXT NOT NULL,
    challenger_slug TEXT NOT NULL,
    champion_metric REAL NOT NULL,
    challenger_metric REAL NOT NULL,
    promoted_at TIMESTAMPTZ DEFAULT NOW(),
    reason TEXT NOT NULL
);
```

**Admin endpoints**:
- `GET /api/recommendations/ab/status` — current champion, challengers, per-engine impression counts
- `POST /api/recommendations/ab/set-champion` — manual override
- `POST /api/recommendations/ab/add-challenger` — register a new engine for A/B

### Phase 3 — Auto-promotion (~3 days)

**Cron job** (`src/bin/promote_best_engine.rs`, runs daily):

```rust
for each active challenger E:
    let champion = current_champion();
    let champ_metric = mean_reading_depth(champion, last_14d);
    let chall_metric = mean_reading_depth(E, last_14d);
    let champ_ci = bootstrap_ci(champ_metric, 0.95);
    let chall_ci = bootstrap_ci(chall_metric, 0.95);

    if chall_ci.lower > champ_ci.upper  // statistically significant
       && (chall_metric - champ_metric) / champ_metric >= 0.05:  // 5% hysteresis
        promote(E);
        log_promotion(champion, E, champ_metric, chall_metric);
```

**Bootstrap CI**: resample the per-impression reading depths 1000 times with replacement, take the 2.5th and 97.5th percentiles. Standard, robust, no distributional assumptions.

**Demote logic**: if the current champion's metric falls ≥8% below its value at promotion time (tracked in `rec_promotions.champion_metric`), revert to the previous champion.

**Safety valve**: admin can `POST /api/recommendations/ab/pause-autopromote` to freeze the system during incidents.

### Phase 4 — Marketplace UI (~5 days)

**Backend endpoints**:
- `GET /api/recommendations/plugins` — public gallery (paginated, sortable by installs/reading-depth/rising)
- `POST /api/recommendations/plugins` — publish (author uploads `.wasm` + manifest, role ≥ 1)
- `GET /api/recommendations/plugins/{slug}` — detail page with stats
- `POST /api/recommendations/plugins/{slug}/install` — user installs
- `POST /api/recommendations/plugins/{slug}/fork` — fork into user's workspace
- `GET /api/recommendations/my-engines` — user's installed + authored plugins
- `PUT /api/recommendations/my-engines/weights` — set blend weights

**Curator review**: plugins go to a review queue (`/curator/plugins`). Curator checks:
- Manifest is well-formed
- `.wasm` runs within declared limits
- No abusive behavior (test with sample inputs)
- Description is accurate

**Frontend pages**:
- `/plugins` — gallery grid with cards (name, author, installs, reading-depth score, "Install" button)
- `/plugins/{slug}` — detail page: description, stats chart (reading depth over time), versions, reviews
- `/plugins/new` — publish form: upload `.wasm`, fill manifest, preview
- `/my/engines` — installed plugins, weight sliders, per-engine stats ("earned you 12h of reading this month")
- `/curator/plugins` — review queue

### Phase 5 — Blessed plugins & migration (~2 days)

Wrap each of the 13 existing strategies as a blessed plugin:

```sql
INSERT INTO rec_plugins (slug, name, entry_kind, is_blessed, is_active, manifest) VALUES
('cooccur', 'Bookmark Co-occurrence', 'blessed', true, true,
 '{"description": "Classic FicHub engine. Finds fics bookmarked together.", ...}'),
('embeddings', 'Semantic Embeddings', 'blessed', true, true, ...),
...
```

The `RecStrategy` trait impls stay in Rust — they're just registered with the plugin system as "blessed" (no sandbox overhead, direct function call). This:
- Preserves performance for the core engines
- Gives the marketplace 13 real entries on day 1
- Lets users mix them via the UI (e.g., "60% cooccur + 40% embeddings")
- Makes the champion/challenger mechanics apply uniformly

**Deprecation**:
- `hybrid` wrapper → removed (users build hybrids in UI)
- `sequential` → moved to `src/reader/sequel.rs` (it's a reader concern)
- `decay` → folded into `cooccur` as a time-window parameter (per earlier brainstorm)

### Phase 6 — Docs & rollout (~2 days)

- `docs/rec-plugin-authoring.md` — how to write a plugin (Rust/AssemblyScript examples, host function reference, testing locally)
- `docs/rec-engine-contract.md` — formal protocol spec
- `docs/src/recommendations.md` — update with marketplace, user control, auto-promotion
- mdbook rebuild + deploy

**Rollout sequence**:
1. Phase 0 (telemetry) → deploy, let it run for 2 weeks to build baseline data
2. Phase 1+2 (WASM + A/B) → deploy with autopromote OFF, run manually
3. Phase 3 (autopromote) → enable with conservative thresholds (10% improvement required)
4. Phase 4 (marketplace) → open to curators first, then all users
5. Phase 5 (blessed plugins) → migrate existing strategies

### Test matrix

| Suite | What it covers |
|-------|----------------|
| `tests/reading_sessions_api.rs` | session CRUD, attribution linking |
| `tests/wasm_plugin.rs` | load, call, fuel exhaustion, memory limit, host functions |
| `tests/ab_buckets.rs` | sticky bucketing, quota enforcement |
| `tests/autopromote.rs` | statistical test, hysteresis, rollback |
| `tests/marketplace_api.rs` | publish, install, fork, curator review |
| `tests/reading_baselines.rs` | WPM computation, rolling window |
| Frontend vitest | gallery, detail, my-engines, weight sliders |
| Playwright e2e | publish flow, install flow, autopromote flip |

### Risks & mitigations

| Risk | Mitigation |
|------|-----------|
| WASM binary size bloat | Cap plugin blob at 2MB; reject larger uploads |
| Plugin DoS via fuel exhaustion | Hard fuel limit per call; quota per day; circuit-breaker if plugin errors > 10% |
| Attribution window too long/short | Start with 30 days; make it config; review after 3 months |
| Auto-promote flips too often | 14-day cooldown + 5% hysteresis; pause button for admin |
| Reading sessions noisy (tab left open) | Require ≥5min duration AND ≥1000 words to count toward baseline |
| Plugin authors game the metric | Curator review; public stats (transparency); reading-depth is hard to game because it requires actual reading |
| wasmtime dependency | It's mature (Bytecode Alliance, used by Fastly/Cloudflare); audit on upgrade |

### Files likely to change (summary)

**New**:
- `migrations/054_reading_sessions.sql`
- `migrations/055_rec_plugins.sql`
- `src/recommender/wasm/{runtime,host,plugin,manifest,loader}.rs`
- `src/recommender/buckets.rs`
- `src/recommender/autopromote.rs`
- `src/recommender/telemetry.rs`
- `src/bin/compute_reading_baselines.rs`
- `src/bin/attribute_reading_sessions.rs`
- `src/bin/promote_best_engine.rs`
- `src/routes/plugins.rs`
- `frontend/src/routes/plugins/{+page,+page.svelte,[slug]/+page.svelte,new/+page.svelte}`
- `frontend/src/routes/my/engines/+page.svelte`
- `frontend/src/lib/api/plugins.ts`
- `docs/rec-engine-contract.md`
- `docs/rec-plugin-authoring.md`

**Modified**:
- `Cargo.toml` (+wasmtime, +sha2 if not present)
- `src/recommender/ranker.rs` (A/B routing)
- `src/recommender/engine.rs` (engine_slug logging)
- `src/recommender/registry.rs` (plugin-aware)
- `src/routes/recommendations.rs` (new endpoints)
- `src/server.rs` (register new routes)
- `frontend/src/routes/read/[urlId]/+page.svelte` (session tracking)
- `frontend/src/routes/recommendations/+page.svelte` (strategy selector)

**Deleted**:
- `src/recommender/hybrid.rs` (replaced by user-facing blend UI)
- `src/recommender/sequential.rs` (moved to `src/reader/sequel.rs`)
- `src/recommender/decay.rs` (folded into cooccur)

### Effort estimate

| Phase | Days | Risk |
|-------|------|------|
| 0 — Telemetry | 3 | Low |
| 1 — WASM runtime | 5 | Medium (wasmtime integration) |
| 2 — A/B framework | 3 | Low |
| 3 — Auto-promotion | 3 | Medium (statistics) |
| 4 — Marketplace UI | 5 | Low |
| 5 — Blessed plugins | 2 | Low |
| 6 — Docs & rollout | 2 | Low |
| **Total** | **~23 days** | — |

Can parallelize: Phase 0 + Phase 1 are independent; Phase 4 can start once Phase 2's endpoints exist.

### The open questions from the brainstorm doc — resolved

1. **Does `rec_impressions` store enough for per-strategy CTR?** — It does today, but Phase 0 adds `engine_slug` + `attributed_session_id` to make the reading-depth metric possible.
2. **`/api/recommendations/suggest` vs `/api/search/suggest`** — different endpoints, different purposes. `suggest` is ask-style NL query (Ollama), `search/suggest` is typeahead. Keep both.
3. **Is `external.rs`'s `reason` field surfaced?** — Not today. Phase 4's plugin detail page will show it.

### The one-sentence pitch

> FicHub's recommendation engine becomes a living marketplace: authors publish sandboxed plugins, readers install the ones that match their taste, and the best engine auto-promotes based not on clicks but on how deeply users actually read.