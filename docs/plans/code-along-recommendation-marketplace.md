# Code-Along: Building the Recommendation Engine Into a Living Marketplace

In this tutorial, you'll turn FicHub's recommendation engine into a living marketplace: sandboxed WASM plugins that real users run, A/B tested against each other by a metric that measures **reading depth** (not clicks), with auto-promotion that swaps in the winner. You'll end with a champion/challenger system, a plugin gallery, and the reading-depth telemetry that decides what gets shipped.

You'll follow `docs/plans/recommendation-engine-plugin-system.md` one verifiable step at a time. Each step has a Check so you know you're on the rails.

## What you'll build

- **Reading-depth telemetry**: `POST /api/reader/{url_id}/session` emits the session that powers the metric; nightly jobs compute per-user WPM baselines + attribute sessions to the engine that recommended them. *(Phase 0)*
- **A WASM plugin runtime** on wasmtime: fuel metering, memory cap, wall-clock interruption, host functions (tags/bookmarks/fic metadata). Plugins are pure functions — no I/O. *(Phase 1)*
- **A/B framework**: sticky per-user bucketing (hash `user_id:salt % 100`), champion/challenger traffic split, `rec_impressions.engine_slug` logging. *(Phase 2)*
- **Auto-promotion cron**: nightly statistical test (bootstrap CI on reading depth), 5% hysteresis, 14-day cooldown, auto-rollback to the previous champion if the new one degrades. *(Phase 3)*
- **Marketplace UI**: plugin gallery (`/plugins`), detail + publish pages, user "My engines" control (install, weight sliders, per-engine stats), curator review queue. *(Phase 4)*
- **Blessed-plugin migration**: wrap the 13 existing strategies as blessed plugins (no sandbox), fold `hybrid` into a user-facing blend, move `sequential` into the reader. *(Phase 5)*

End-to-end: deploy all six and verify that a challenger engine can climb through A/B, win on reading-depth, auto-promote over `cooccur`, and be visible in the gallery — all without a single hardcoded strategy.

## Prerequisites

- Rust toolchain (edition 2021+), `cargo`.
- PostgreSQL 14+.
- Repo at `/personal/documents/code/rust/fichub`.
- `cargo test --lib` green (baseline — currently 736 passing post-quests-scrub).
- Plan file: `docs/plans/recommendation-engine-plugin-system.md`.

## Codebase geography

- `src/recommender/registry.rs:1-12` — `REC_STRATEGIES`/`REC_ENGINE_MODE` parsing; default `["cooccur"]`.
- `src/recommender/engine.rs` — `RecStrategy` trait + `score(work_id) -> Score`.
- `src/recommender/ranker.rs` — ranks candidate works per strategy.
- `migrations/001_initial.sql` — `rec_impressions` table (line ~2436), `works`, `users`.
- `frontend/src/routes/read/[urlId]/+page.svelte` — the reader (extend with session tracking).

**Golden constraints:**
- Runtime `sqlx::query(...)` with `$N` binds only — never `query!` macros (the crate standard).
- All git writes use the NFS object-directory shim.
- Binary deploy: scp to `/tmp` → `cp -f` → `mv` → restart.

---

## Step 0: Reading-depth telemetry foundation (~3 days)

**Goal:** ship the metric. Without `reading_sessions` + `engine_slug` on impressions, nothing downstream works.

### Actions

- **File**: `migrations/054_reading_sessions.sql`

```sql
CREATE TABLE reading_sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id),
    work_id INT4 NOT NULL REFERENCES works(id),
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    words_read INT4 NOT NULL DEFAULT 0,
    chapters_read INT4 NOT NULL DEFAULT 0,
    client_id TEXT,
    source TEXT DEFAULT 'reader'
);
CREATE INDEX ON reading_sessions (user_id, work_id, started_at);

CREATE TABLE user_reading_baselines (
    user_id INT4 PRIMARY KEY REFERENCES users(id),
    words_per_minute REAL NOT NULL DEFAULT 200.0,
    sample_size INT4 NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Annotate existing impressions with which engine showed them
ALTER TABLE rec_impressions ADD COLUMN engine_slug TEXT;
ALTER TABLE rec_impressions ADD COLUMN attributed_session_id BIGINT
    REFERENCES reading_sessions(id);
CREATE INDEX ON rec_impressions (engine_slug, shown_at);
```

- **File**: `src/routes/reader.rs` (extend `reader_handler` or add a sub-route)

```rust
// POST /api/reader/{url_id}/session
// body: { "event": "start"|"heartbeat"|"end", "words_read": N, "chapters_read": N }
pub async fn reader_session_handler(
    auth: AuthUser,
    Path(url_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReaderSessionPayload>,
) -> Result<Json<JsonValue>, AppError> {
    let user_id = auth.user_id.ok_or(AppError::Unauthorized)?;
    let work_id = queries::resolve_work_id(&state.db, &url_id).await?;
    match payload.event.as_str() {
        "start" => sqlx::query(
            "INSERT INTO reading_sessions (user_id, work_id, started_at) VALUES ($1,$2,now()) RETURNING id"
        ).bind(user_id).bind(work_id).fetch_one(&state.db).await,
        "heartbeat" => sqlx::query("UPDATE reading_sessions SET words_read=$1, chapters_read=$2 WHERE id=$3 AND ended_at IS NULL")
            .bind(payload.words_read).bind(payload.chapters_read).bind(payload.session_id).execute(&state.db).await,
        "end" => sqlx::query("UPDATE reading_sessions SET ended_at=now(), words_read=$1, chapters_read=$2 WHERE id=$3 AND ended_at IS NULL")
            .bind(payload.words_read).bind(payload.chapters_read).bind(payload.session_id).execute(&state.db).await,
        _ => return Err(AppError::BadRequest("bad event".into())),
    }?;
    Ok(Json(json!({"err": 0})))
}
```

- **File**: `frontend/src/routes/read/[urlId]/+page.svelte` — emit start/heartbeat/end on load/30s interval/visibilitychange.

### Check

- `psql $DATABASE_URL -c '\d reading_sessions'` → columns `words_read`, `chapters_read`, `started_at`, `ended_at`.
- `psql $DATABASE_URL -c '\d rec_impressions'` → has `engine_slug`.
- `npx tsc --noEmit` → no errors referencing `reader_session_handler`.

### Troubleshooting

- `work_id` not on `rec_impressions` → join through `works` table; the attribution job matches on `(user_id, work_id)`.
- Tab left open inflates sessions → Phase 0's baseline job filters sessions with `duration < 5min OR words_read < 1000`.

---

## Step 1: WASM plugin runtime (~5 days)

**Goal:** load + sandbox untrusted `.wasm` recommendation plugins with a hard resource budget.

### Actions

- **File**: `Cargo.toml`

```toml
[dependencies]
wasmtime = "20"
serde_json = "1.0"
sha2 = "0.10"
```

- **File**: `src/recommender/wasm/mod.rs` + `runtime.rs` + `host.rs` + `plugin.rs`

`runtime.rs` — the engine pool:

```rust
pub struct WasmRuntime {
    engine: wasmtime::Engine,
}
impl WasmRuntime {
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        config.async_support(wasmtime::AsyncSupport::none()); // sync only, no I/O
        let engine = wasmtime::Engine::new(&config).unwrap();
        Self { engine }
    }
    pub fn run_plugin(
        &self,
        wasm_bytes: &[u8],
        fuel_limit: u64,
        memory_pages: u64,
        wall_ms: u64,        // epoch interruption budget
        input: &str,         // JSON candidates
    ) -> Result<String, PluginError> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        let mut linker = wasmtime::Linker::new(&module);
        // host functions registered via host.rs
        wasmtime::register_global_state(&module, &linker)?;
        let mut store = wasmtime::Store::new(&self.engine, ());
        store.set_fuel(fuel_limit)?;
        store.set_epoch_deadline(wall_ms)?;
        let instance = linker.instantiate(&mut store, &module)?;
        let score_fn = instance.get_typed_func::<(i32,i32,i32,i32,i32), i32>(
            &mut store, "score"
        )?;
        // ... write input JSON to plugin memory, call score, read output ...
        let result = score_fn.call(&mut store, (...))?;
        let _remaining = store.get_fuel()?; // aborts if over budget
        Ok(output_json)
    }
}
```

`plugin.rs` — implements `RecStrategy`:

```rust
pub struct WasmPlugin {
    pub slug: String,
    pub runtime: Arc<WasmRuntime>,
    pub wasm_bytes: Vec<u8>,
    pub manifest: PluginManifest,
}
#[async_trait]
impl RecStrategy for WasmPlugin {
    async fn score(&self, ctx: &RecContext) -> Vec<(i32, f64, String)> {
        let input = serde_json::json!({
            "user_id": ctx.user_id,
            "seed_work_id": ctx.seed_work_id,
            "candidates": ctx.candidates,
        }).to_string();
        let out = self.runtime.run_plugin(
            &self.wasm_bytes,
            self.manifest.fuel_limit,
            self.manifest.memory_pages,
            self.manifest.wall_clock_ms,
            &input,
        ).unwrap_or_default();
        // deserialize {work_id, score, reason}[]
        serde_json::from_str(&out).unwrap_or_default()
    }
}
```

- **File**: `src/recommender/registry.rs` — load `rec_plugins` table on startup, wrap `wasm`/`http` entry_kinds in `WasmPlugin`/sidecar, register `blessed` strategies as direct trait impls.

### Check

- `grep -n "wasmtime" Cargo.toml` → present.
- `cargo check` → no errors in `src/recommender/wasm/`.
- Fuel-exhaustion test: a plugin with an infinite loop aborts cleanly (returns `PluginError::FuelExhausted`, server returns 503, no hang).

### Troubleshooting

- `wasmtime` needs `cranelift` (default features) → if building on aarch64, add `config.cratcodes(true)` or cross-compile notes.
- Host functions can't allocate → pass JSON in/out via linear memory with a shared allocator pattern (`alloc`/`dealloc` exports in the plugin contract).

---

## Step 2: A/B framework + sticky bucketing (~3 days)

**Goal:** route a user to one engine per session + log which engine got the impression.

### Actions

- **File**: `migrations/055_rec_plugins.sql`

```sql
CREATE TABLE rec_plugins (
    id SERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    author_id INT4 REFERENCES users(id),
    description TEXT,
    entry_kind TEXT NOT NULL CHECK (entry_kind IN ('wasm','http','sql','blessed')),
    wasm_blob BYTEA,
    manifest JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    is_blessed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE rec_engine_assignments (
    user_id INT4 PRIMARY KEY REFERENCES users(id),
    engine_slug TEXT NOT NULL,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    reassigned_at TIMESTAMPTZ
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

- **File**: `src/recommender/buckets.rs`

```rust
pub fn bucket_for(user_id: i32, salt: &str) -> u8 {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", user_id, salt).as_bytes());
    hasher.finalize()[0] % 100
}
```

- **File**: `src/recommender/ranker.rs` — A/B routing:

```rust
pub async fn resolve_engine(db: &Pool<Postgres>, user_id: i32) -> String {
    // sticky: reuse existing assignment
    if let Some(row) = sqlx::query_as::<_, (String,)>("SELECT engine_slug FROM rec_engine_assignments WHERE user_id=$1")
        .bind(user_id).fetch_optional(db).await { return row.0; }
    let b = bucket_for(user_id, &env::var("REC_BUCKET_SALT").unwrap_or_default());
    let champion = env::var("REC_CHAMPION").unwrap_or_else(|_| "cooccur".into());
    let challengers: Vec<String> = sqlx::query_scalar("SELECT slug FROM rec_plugins WHERE is_active AND NOT is_blessed ORDER BY created_at LIMIT 3")
        .fetch_all(db).await.unwrap_or_default();
    if b < 70 { champion.clone() }
    else if b < 70 + (10 * challengers.len() as u8) {
        challengers[(b - 70) as usize % challengers.len()].clone()
    } else { champion }
}
```

- **File**: `src/recommender/engine.rs` — attach `engine_slug` to each `rec_impressions` insert.

### Check

- `cargo test --lib buckets` → `bucket_for(1, "salt") != bucket_for(2, "salt")` (stability) + same input → same output.
- `curl /api/recommendations/personal` (logged in) → `rec_impressions` row has non-null `engine_slug`.
- `SELECT engine_slug, count(*) FROM rec_impressions WHERE shown_at > now() - interval '1h' GROUP BY 1` → ~70% rows `cooccur`, challengers get the rest, no user flips within the hour.

### Troubleshooting

- `resolve_engine` returns a challenger that doesn't exist in `rec_plugins` → the ranker only selects from `is_active AND NOT is_blessed` rows, so deactivating a challenger in the DB removes it from rotation without touching the binary.
- A user flips between champion + challenger → the `rec_engine_assignments` upsert (insert-on-first-impression, select-on-subsequent) isn't running. Check `engine.rs` writes it after the first `rec_impressions` insert.

---

## Step 3: Nightly auto-promotion (~3 days)

**Goal:** let the best-performing challenger dethrone the champion, with statistical rigor + hysteresis to prevent thrashing.

### Actions

- **File**: `src/bin/promote_best_engine.rs` (new cargo binary — add `[[bin]]` to `Cargo.toml`)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = connect_db()?;
    let champion: String = sqlx::query_scalar("SELECT slug FROM rec_plugins WHERE is_blessed AND is_active ORDER BY name LIMIT 1")
        .fetch_one(&db).await?;
    let champ_sessions: Vec<(f64,)> = sqlx::query_as(
        "SELECT reading_depth_per_impression(i.id)
         FROM rec_impressions i
         WHERE i.engine_slug=$1 AND i.shown_at > now() - interval '14 days'"
    ).bind(&champion).fetch_all(&db).await?;
    let champ_ci = bootstrap_ci(&champ_sessions, 0.95);
    for (slug, sessions) in active_challengers_with_sessions(&db).await? {
        let chall_ci = bootstrap_ci(&sessions, 0.95);
        // promote only if challenger 95% CI-lower > champion mean, AND ≥5% relative lift
        if chall_ci.lower > champ_ci.upper
            && (chall_ci.median - champ_ci.median) / champ_ci.median >= 0.05
            && days_since_last_promotion(&slug, &db)? >= 14
        {
            sqlx::query(
                "UPDATE rec_plugins SET is_blessed=true, name='Champion' WHERE slug=$1"
            ).bind(&slug).execute(&db).await?;
            sqlx::query(
                "UPDATE rec_plugins SET is_blessed=false WHERE slug=$2"
            ).bind(&champion).bind(&slug).execute(&db).await?;
            sqlx::query(
                "INSERT INTO rec_promotions (champion_slug, challenger_slug, champion_metric, challenger_metric, reason) VALUES ($1,$2,$3,$4,'auto: reading_depth beat champion by ≥5%')"
            ).bind(&champion).bind(&slug).bind(champ_ci.median).bind(chall_ci.median).execute(&db).await?;
            println!("Promoted {} over {} (champion={:.3}, challenger={:.3})",
                slug, champion, champ_ci.median, chall_ci.median);
        }
    }
    db.close().await;
    Ok(())
}
```

- **`reading_depth_per_impression(impression_id)`** (SQL helper): joins `rec_impressions` → `reading_sessions` on `(user_id, work_id)` within the 30-day attribution window, computes `SUM(minutes_reading) / impressions / user_baseline_wpm`.

- **Rollback guard**: a daily watchdog compares the new champion's 14-day metric to its value at promotion time (`rec_promotions.champion_metric`). If it drops ≥8%, revert `is_blessed` back to the previous slug (also stored as `challenger_slug` in the same row).

### Check

- `fichub-promote-best-engine --dry-run` (reads but doesn't update) prints the champion + each challenger's reading-depth mean + CI bounds.
- `cargo test --lib autopromote` → `hysteresis_blocks_3pct_lift` + `cooldown_enforces_14_days` + `rollback_reverts_degraded_champion`.
- Cron registered via `scripts/install_systemd_timers.sh` — `promote_best_engine.timer` fires daily at 03:00.

### Troubleshooting

- Bootstrap CI always promotes → lower bound isn't being checked. Assert `chall_ci.lower > champ_median` explicitly.
- Demote flaps daily → the 8% threshold + 14-day cooldown aren't gating. Verify `days_since_last_promotion` reads `rec_promotions.promoted_at` for the current champion.

---

## Step 4: Marketplace UI (~5 days)

**Goal:** let curators approve plugins + users install/blend them.

### Actions

- **File**: `src/routes/plugins.rs` — endpoints:
  - `GET /api/recommendations/plugins` — gallery (`ORDER BY installs DESC`, `?sort=reading_depth|rising|newest`)
  - `POST /api/recommendations/plugins` — publish (role ≥ 1; accepts `wasm_blob` + `manifest` JSON)
  - `GET /api/recommendations/plugins/{slug}` — detail + per-engine reading-depth chart
  - `POST /api/recommendations/plugins/{slug}/install` — user installs
  - `POST /api/recommendations/plugins/{slug}/fork` — fork into user's workspace
  - `PUT /api/recommendations/my-engines/weights` — set blend weights
  - `GET /api/recommendations/my-engines` — user's installed + authored + per-engine stats

- **File**: `src/routes/curator.rs` — `GET /api/curator/plugins` (review queue: pending, with manifest + blob-size preview), `POST /api/curator/plugins/{slug}/approve|reject`.

- **File**: `frontend/src/routes/plugins/+page.svelte` — gallery grid (name, author, installs, reading-depth score, Install button).
- **File**: `frontend/src/routes/plugins/[slug]/+page.svelte` — detail page with stats chart (`reading-depth over time`), versions, reviews.
- **File**: `frontend/src/routes/plugins/new/+page.svelte` — publish form (upload `.wasm`, fill manifest fields, preview).
- **File**: `frontend/src/routes/my/engines/+page.svelte` — installed plugins, weight sliders (Σ=100%), per-engine stats ("earned you 12h of reading this month").
- **File**: `frontend/src/routes/curator/plugins/+page.svelte` — review queue.

### Check

- `npx vitest run src/routes/plugins` → gallery renders, install toggle works, weight sliders sum to 100%.
- `curl /api/recommendations/plugins` (public) → JSON array with the 13 blessed plugins + any user-published ones.
- Upload a `.wasm` > 2MB → `400` with `{"err":-8,"msg":"plugin blob exceeds 2MB"}`.

### Troubleshooting

- Plugin returns wrong results → curator can re-run it against sample inputs from the detail page (`/plugins/{slug}/test`).
- Weight sliders don't persist → `PUT /api/recommendations/my-engines/weights` validates the sum client-side + server-side (reject if `Σ != 100` with `400`).

---

## Step 5: Blessed-plugin migration (~2 days)

**Goal:** wrap the 13 existing strategies as blessed plugins so the marketplace ships with real engines, and dissolve the `hybrid` wrapper + `sequential` strategy.

### Actions

- **File**: `src/recommender/registry.rs` — on startup, seed `rec_plugins` with `is_blessed=true` for each compiled strategy (`cooccur`, `embeddings`, `bandit`, `clusters`, `author_graph`, `tag_graph`, `mf`, `signals`, `curator`, `legacy_cooccur`, `external`). Each registered as a `BlessedPlugin` (direct `RecStrategy` impl, no WASM overhead).

```sql
-- one-time seed (idempotent)
INSERT INTO rec_plugins (slug, name, entry_kind, is_blessed, is_active, manifest) VALUES
('cooccur','Bookmark Co-occurrence','blessed',true,true,
 '{"description":"Classic co-occurrence engine. Finds fics bookmarked together.",
   "host_functions":[]}'::jsonb),
('embeddings','Semantic Embeddings','blessed',true,true,
 '{"description":"Vector-similarity over fic embeddings.",
   "host_functions":[]}'::jsonb),
-- ... 11 more
ON CONFLICT (slug) DO UPDATE SET is_active=true;
```

- **Delete**: `src/recommender/hybrid.rs` — the user-facing blend (weight sliders in `my/engines`) replaces it.
- **Move**: `src/recommender/sequential.rs` → `src/reader/sequel.rs` (it's a reader-flow concern, not a rec engine).
- **Fold**: `src/recommender/decay.rs` — merge the time-decay multiplier into `cooccur`'s scoring as a `recency_weight` parameter.

- **File**: `src/routes/recommendations.rs` — `/api/recommendations/personal` now calls `resolve_engine()` (from Step 2) to pick the user's assigned engine, then dispatches to either the `BlessedPlugin` or `WasmPlugin` runner.

### Check

- `cargo check` → no references to `hybrid.rs` or `sequential.rs` in `registry.rs`.
- `psql $DATABASE_URL -c "SELECT slug FROM rec_plugins WHERE is_blessed"` → 13 rows.
- `curl /api/recommendations/personal` (logged in) → returns a blend from `my-engines/weights` if installed, else the assigned champion.

### Troubleshooting

- `external` sidecar stops working → it's registered as a blessed plugin with `entry_kind='http'` (the sidecar runner, not wasm). Verify `external.rs` still compiles as a strategy impl.
- `sequential` move breaks imports → update `use` paths in `reader.rs`.

---

## Phase 6: Docs, tests & rollout (~2 days)

**Goal:** lock it down with tests/docs + deploy incrementally.

### Actions

- **File**: `docs/rec-plugin-authoring.md` — full plugin authoring guide (Rust `cdylib` + `wasm32-unknown-unknown` target, AssemblyScript alternative, host function reference, `wasmtime` local testing, manifest schema).
- **File**: `docs/rec-engine-contract.md` — the formal `score()` ABI, memory layout, error codes.
- **File**: `docs/src/recommendations.md` — update mdbook with marketplace, user control, auto-promotion behavior.
- **File**: `tests/marketplace_api.rs` — publish/install/fork/curator-approve e2e (DB-gated).
- **File**: `tests/wasm_plugin.rs` — load test, fuel exhaustion, memory limit, host function calls.
- **File**: `tests/autopromote.rs` — statistical test, hysteresis, cooldown, rollback.
- **File**: `tests/reading_sessions_api.rs` — session CRUD + attribution.
- **File**: `frontend/src/routes/plugins/[slug]/page.test.ts` + `my/engines/+page.test.ts` — vitest.

**Rollout sequence**:
1. Deploy Phase 0 (telemetry) alone — let it run 2 weeks to build baseline data.
2. Deploy Phase 1+2 (WASM + A/B) with `autopromote` OFF — run challenger manually.
3. Enable Phase 3 (cron) with conservative thresholds (10% improvement required).
4. Deploy Phase 4 (marketplace) to curators first, then all users.
5. Deploy Phase 5 (blessed migration) — the 13 strategies become visible + mixable.

### Check

- `cargo test --lib` → all new modules green (`buckets`, `wasm_plugin`, `autopromote`, `marketplace_api`, `reading_sessions_api`).
- `npx vitest run` → 0 failures.
- `cargo build --release --bin fichub` + `cargo build --release --bin promote_best_engine` + `cargo build --release --bin compute_reading_baselines` + `cargo build --release --bin attribute_reading_sessions` → all clean.

---

## Final Verification

1. `cargo test --lib` → **736 baseline** + new test modules all green.
2. `cargo build --release --bin fichub` → clean, md5 fingerprint.
3. Deploy: scp → `/tmp` → `cp -f` → `mv` → `systemctl restart fichub`.
4. Health: `curl /health` → 200.
5. A/B routing live: two different logged-in users get different `engine_slug` in their `rec_impressions` within the 70/30 split.
6. Reader session tracking: open a fic, fire the heartbeat, verify `reading_sessions` row populated with `words_read`.
7. Auto-promotion: run `promote_best_engine` manually — challenger that beats champion by ≥5% (CI) gets `is_blessed=true`, previous champion demoted, `rec_promotions` row inserted.
8. Marketplace: `GET /api/recommendations/plugins` returns the 13 blessed plugins + any user-published ones; `GET /api/recommendations/my-engines` returns the authenticated user's installed/weight set.

## Next Steps

- Add `reading_depth_per_impression` as a materialized view (the nightly cron computes it 3x/day — a view keeps the real-time dashboard fast).
- Surface `reason` strings from `external.rs` on the plugin detail page (the contract's third tuple element).
- Build the "rising" sort (improvement rate over last 7 days) — one extra `LAG()` window function.

---

*Per the code-along-tutorial-writer skill. Pair with `docs/plans/recommendation-engine-plugin-system.md` for the full task breakdown + risk register.*