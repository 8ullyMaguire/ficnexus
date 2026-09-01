# Part 32 — Ollama Embeddings and Recommender

> In this chapter you will learn how FicHub's pluggable recommendation engine works — from Ollama nomic-embed-text embeddings to a 15-strategy platform with RRF blending, curator priors, bandit exploration, and community voting. We'll build the Strategy trait, the RRF ranker, the EmbeddingsStrategy, and the recommendation API that switches between legacy and pluggable modes.

---

## Overview

FicHub's recommendation system has two modes, switched by `REC_ENGINE_MODE`:

- **Legacy** (default) — the `RecommendationEngine` in `src/recommender/engine.rs` computes item-to-item recs from bookmark co-occurrence, with a tag-overlap fallback for works with few bookmarks. Used for the public `/api/recommendations` endpoint.
- **Pluggable** — the strategy platform in `src/recommender/` runs multiple `RecStrategy` implementations through an RRF ranker, with a config-driven registry (`REC_STRATEGIES`), curator prior, and bandit exploration. Used for the personalized `/api/recommendations/personal` endpoint when enabled.

Both modes share the same 304-dimensional embedding space from Ollama's `nomic-embed-text`, stored in PostgreSQL's `pgvector` extension.

---

## Chapter 32.1 — The Strategy Architecture

### Goal

Understand the `RecStrategy` trait, the `StrategyContext`, and the config-driven `StrategyRegistry`.

### Actions

#### 1. The RecStrategy trait

```rust
// src/recommender/strategy.rs (lines 112–145)
#[async_trait]
pub trait RecStrategy: Send + Sync {
    /// Canonical strategy name (registry key, rec_* log/training key).
    fn name(&self) -> &str;

    /// Score candidate works. Return ordered (descending) list, or Err to skip.
    /// * user_id: Some(id) → personalized; user_id: None → item-to-item (seed required).
    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError>;

    /// Training/precompute batch step. Default = no-op.
    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        Ok(serde_json::json!({ "note": "no-op" }))
    }

    /// Whether this strategy serves personalized requests. Default true.
    fn supports_personal(&self) -> bool { true }
}
```

> **💡 Key Concept**: Strategies are **pure scorers** — they produce a ranked list of `ScoredRec`s, but never decide what gets shown. The ranker (Chapter 32.3) fuses multiple strategies' outputs and decides the final ranking. This separation means you can add a new algorithm by implementing `RecStrategy` and adding it to `REC_STRATEGIES` — no changes to the API layer.

#### 2. The StrategyContext

```rust
// src/recommender/strategy.rs (lines 44–78)
#[derive(Debug, Clone)]
pub struct StrategyContext {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub http_client: reqwest::Client,     // shared: scrapers, external sidecars, Ollama
    pub ollama: OllamaClient,             // nomic-embed-text embeddings
    pub now: DateTime<Utc>,              // frozen per request for decay math
}

impl StrategyContext {
    pub fn new(db, config, http_client, ollama) -> Self { ... }
    /// Per-strategy HTTP timeout for external calls (config.rec_strategy_timeout_secs)
    pub fn call_timeout(&self) -> Duration { ... }
}
```

#### 3. The StrategyRegistry

```rust
// src/recommender/registry.rs (lines 53–132)
/// Config-driven registry: parses REC_STRATEGIES (e.g. "cooccur:0.4,embeddings:0.3,bandit:0.1")
pub struct StrategyRegistry {
    specs: Vec<StrategySpec>,  // enabled, in config order
    strategies: HashMap<String, Arc<dyn RecStrategy>>,
}

/// Parse "cooccur:0.4,embeddings:0.3" — missing weight → 1.0, negative → clamped.
pub fn parse_strategy_specs(raw: &str) -> Vec<StrategySpec> { ... }

impl StrategyRegistry {
    pub fn new(available: Vec<Arc<dyn RecStrategy>>, raw_specs: &str) -> Self {
        // Drops unknown names with a warning. Falls back to ['cooccur'] if empty.
    }
    pub fn specs(&self) -> &[StrategySpec] { &self.specs }  // ordered, weighted
    pub fn get(&self, name: &str) -> Option<Arc<dyn RecStrategy>> { ... }
    pub fn all(&self) -> Vec<Arc<dyn RecStrategy>> { ... }     // all registered (for admin listing)
    pub fn enabled_names(&self) -> Vec<String> { ... }
    pub fn total_weight(&self) -> f64 { ... }
}
```

#### 4. The 15 strategies

The available strategies come from `available_strategies()` in `recommender/mod.rs`:

| Strategy | Algorithm |
|---|---|
| `legacy_cooccur` | Jaccard on bookmark co-occurrence (wraps the legacy engine) |
| `decay` | Time-decayed co-occurrence (SAR-style) |
| `embeddings` | pgvector cosine on Ollama `nomic-embed-text` (768-d) |
| `hybrid` | Learned-weight blend of MF + embeddings + tags |
| `author_graph` | Author-level co-bookmark + shared-tag Jaccard |
| `tag_graph` | Meta-path tag knowledge-graph traversal |
| `sequential` | First-order Markov next-read transitions |
| `clusters` | Taste-cluster k-means memberships |
| `bandit` | Contextual Thompson sampling |
| `mf` | Implicit ALS matrix factorization (requires `rec-mf` feature) |
| `external` | HTTP adapter for Python sidecars (`REC_EXTERNAL_URL`) |

#### 5. Strategy error handling

```rust
// src/recommender/strategy.rs (lines 85–103)
#[derive(Debug, thiserror::Error)]
pub enum RecError {
    #[error("database error: {0}")] Database(#[from] sqlx::Error),
    #[error("strategy error: {0}")] Strategy(String),
    #[error("external sidecar error: {0}")] External(String),
    #[error("not enough data: {0}")] NotEnoughData(String),
    #[error("work {0} not found")] NotFound(String),
}

/// Strategies are expected to degrade gracefully. The ranker treats an Err
/// as "skip this strategy and fall through to the next" (the fallback chain),
/// never as a hard failure of the whole recommendation request.
```

### Try It Yourself

```bash
# Unit tests for the registry (no DB)
cargo test --lib recommender::registry::tests -- --nocapture
```

### Check

- ✅ `RecStrategy` trait has `name()`, `score()`, optional `train()` and `supports_personal()`.
- ✅ Registry drops unknown strategy names with a warning (graceful degradation).
- ✅ Empty `REC_STRATEGIES` config falls back to `["cooccur"]` (preserves legacy behavior).
- ✅ Strategies return `Err(RecError)` on failure; the ranker skips them (never hard-fails).
- ✅ `StrategyContext` carries frozen `now: DateTime<Utc>` so all strategies agree on time.

### What you built

The strategy architecture — a trait every recommendation algorithm implements, a shared context carrying DB/Redis/Ollama handles, and a config-driven registry that selects, weights, orders, and gracefully degrades strategies.

---

## Chapter 32.2 — The Embeddings Strategy (pgvector + Ollama)

### Goal

Build the `EmbeddingsStrategy` — item-to-item cosine similarity via pgvector, personalized profile centroids, and the FNV-1a content-hash gate that prevents re-embedding unchanged fics.

### Actions

#### 1. Embedding text construction

```rust
// src/recommender/embeddings.rs (lines 25–39)
/// Build the text to embed: "title | description | tag1 | tag2 | ..."
/// Separated by " | " so the model can distinguish fields.
pub fn build_embed_text(title: &str, description: &str, top_tags: &[String]) -> String {
    let mut parts = Vec::with_capacity(2 + top_tags.len());
    parts.push(title.trim().to_string());
    if !description.trim().is_empty() {
        parts.push(description.trim().to_string());
    }
    for t in top_tags.iter().take(10) {  // top 10 tags by score
        parts.push(t.clone());
    }
    parts.join(" | ")
}

/// FNV-1a content hash — gates re-embedding (same text → same hash → skip).
/// Not cryptographic; only used to detect unchanged fic metadata.
pub fn content_hash(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;  // FNV offset basis
    for b in text.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);  // FNV prime
    }
    format!("{:016x}", hash)
}
```

> **💡 Key Concept**: The `content_hash` is computed in Rust and stored alongside the embedding in `rec_embeddings.content_hash`. During training, `works_needing_embedding` SQL-joins `fic_info` LEFT JOIN `rec_embeddings` `WHERE re.work_id IS NULL` — works already embedded (matching hash) are never re-fetched. This is a **cache invalidation gate**: only changed fics (title/summary/tags changed since last embed) get re-embedded.

#### 2. Item-to-item scoring (seed cosine)

```rust
// src/recommender/embeddings.rs (lines 127–158)
async fn score(&self, ctx: &StrategyContext, seed: Option<&str>, user_id: Option<i32>)
    -> Result<Vec<ScoredRec>, RecError>
{
    let model = &ctx.config.rec_embed_model;
    let limit = ctx.config.rec_max_recommendations as i64;

    // Item-to-item: cosine similarity between seed and every other work
    if let Some(seed_id) = seed {
        let rows: Vec<(String, f64)> = sqlx::query_as(
            r#"SELECT e2.work_id, 1.0 - (e1.embedding <=> e2.embedding) AS cosine
               FROM rec_embeddings e1
               JOIN rec_embeddings e2 ON e2.model = e1.model AND e2.work_id != e1.work_id
               WHERE e1.work_id = $1 AND e1.model = $2
               ORDER BY cosine DESC LIMIT $3"#
        ).bind(seed_id).bind(model).bind(limit).fetch_all(&ctx.db).await?;

        if rows.is_empty() {
            return Err(RecError::NotEnoughData(format!("no embeddings for seed {seed_id}")));
        }
        return Ok(rows.into_iter()
            .map(|(work_id, cosine)| ScoredRec {
                work_id,
                score: cosine,
                strategy: self.name().into(),
                reason: "similar content (embeddings)".into(),
            }).collect());
    }

    // Personalized: centroid of the user's bookmarked work embeddings
    // ... (see user_profile_centroid below)
}
```

> **⚠️ Watch Out**: PostgreSQL's `pgvector` `<=>` operator computes **Euclidean (L2) distance**, not cosine. `cosine = 1.0 - euclidean_distance` works correctly **only when vectors are unit-normalized**. The training step normalizes centroids (Section: user_profile_centroid), but raw per-work embeddings from Ollama are NOT pre-normalized. For correctness, either normalize at insert time or use `1.0 - (embedding <=> seed)` knowing the distance is L2-based.

#### 3. Personalized: user profile centroid

```rust
// src/recommender/embeddings.rs (lines 247–315)
pub async fn user_profile_centroid(
    ctx: &StrategyContext, user_id: i32,
) -> Result<(String /* vector literal */, HashSet<String> /* known works */), RecError> {
    // Fetch user's signaled works + their embeddings
    let rows: Vec<(String, String, f64)> = sqlx::query_as(
        r#"SELECT s.work_id, e.embedding::text, s.signal_weight
           FROM rec_user_signals s
           JOIN rec_embeddings e ON e.work_id = s.work_id AND e.model = $1
           WHERE s.user_id = $2"#
    ).bind(&ctx.config.rec_embed_model).bind(user_id).fetch_all(&ctx.db).await?;

    // Weighted centroid with recency decay: exp(-days_since_signal / 30)
    let mut centroid: Vec<f64> = vec![0.0; ctx.config.rec_embed_dim];
    let mut total_weight = 0.0f64;
    let mut known = HashSet::new();

    for (work_id, vec_text, weight) in rows {
        known.insert(work_id);  // exclude from recommendations
        let parsed = parse_vector_literal(&vec_text);
        let recency = (-age_days / 30.0).exp();
        let w = weight * recency;
        for (i, v) in parsed.iter().enumerate() {
            if let Some(c) = centroid.get_mut(i) { *c += v * w; }
        }
        total_weight += w;
    }

    // Normalize: divide by total weight, then L2-normalize (cosine-friendly)
    if total_weight > 0.0 { for c in centroid.iter_mut() { *c /= total_weight; } }
    let norm: f64 = centroid.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm > 1e-9 { for c in centroid.iter_mut() { *c /= norm; } }

    let lit = format!("[{}]", centroid.iter().map(|v| v.to_string())
        .collect::<Vec<_>>().join(","));
    Ok((lit, known))
}
```

#### 4. Vector literal parser

```rust
/// Parse a pgvector literal "[1.5,2.0,...]" into f64s. Lenient: skips garbage.
pub fn parse_vector_literal(s: &str) -> Vec<f64> {
    let trimmed = s.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed.is_empty() { return Vec::new(); }
    trimmed.split(',').filter_map(|p| p.trim().parse::<f64>().ok()).collect()
}
```

#### 5. Training (batch embedding)

```rust
// src/recommender/embeddings.rs (lines 196–241)
async fn train(&self, ctx: &StrategyContext) -> Result<Value, RecError> {
    let batch = works_needing_embedding(ctx, &ctx.config.rec_embed_model, 200).await?;
    let mut embedded = 0usize;
    let mut skipped = 0usize;

    for (work_id, text) in batch {
        // Per-item timeout prevents a stuck Ollama call from blocking the batch
        let result = tokio::time::timeout(
            ctx.call_timeout(),
            ctx.ollama.embed(&text),
        ).await;

        match result {
            Ok(Ok(vec)) => {
                if vec.len() != ctx.config.rec_embed_dim { skipped += 1; continue; }
                // INSERT with ON CONFLICT to refresh existing embeddings
                sqlx::query(
                    r#"INSERT INTO rec_embeddings (work_id, model, embedding, content_hash, updated_at)
                       VALUES ($1, $2, $3::vector, $4, NOW())
                       ON CONFLICT (work_id, model) DO UPDATE SET
                         embedding = EXCLUDED.embedding,
                         content_hash = EXCLUDED.content_hash,
                         updated_at = NOW()"#
                ).bind(&work_id).bind(&ctx.config.rec_embed_model)
                  .bind(format!("[{}]", vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")))
                  .bind(content_hash(&text))
                  .execute(&ctx.db).await?;
                embedded += 1;
            }
            _ => { skipped += 1; }  // timeout, Ollama error, or dimension mismatch
        }
    }
    Ok(json!({ "embedded": embedded, "skipped": skipped, "model": ctx.config.rec_embed_model }))
}
```

### Try It Yourself

```bash
# Unit tests (no live Ollama needed — embed() is mocked in tests)
cargo test --lib recommender::embeddings::tests -- --nocapture

# Pull the embedding model locally
ollama pull nomic-embed-text

# Check which works need embedding
psql dbname -c "SELECT f.id, f.title FROM fic_info f
  LEFT JOIN rec_embeddings re ON re.work_id = f.id AND re.model = 'nomic-embed-text'
  WHERE re.work_id IS NULL ORDER BY f.updated DESC LIMIT 20;"
```

### Check

- ✅ `build_embed_text` produces `"title | description | tag1 | tag2"`.
- ✅ `content_hash` is deterministic FNV-1a, 16-char hex.
- ✅ `parse_vector_literal("[1.0,oops,2.0]")` → `[1.0, 2.0]` (garbage skipped).
- ✅ Training uses `ON CONFLICT ... DO UPDATE` (idempotent refreshes).
- ✅ Per-item timeout wraps `ctx.ollama.embed()` — a stuck call is skipped, not fatal.
- ✅ `user_profile_centroid` applies `exp(-days/30)` recency decay then L2-normalizes.

### What you built

The embeddings strategy — a `RecStrategy` that uses Ollama's `nomic-embed-text` (768-d) to generate embeddings, stores them in pgvector, computes item-to-item cosine similarity via the `<=>` operator, and computes personalized recommendations via a recency-weighted, L2-normalized user profile centroid. Training is a robust batch loop with per-item timeouts and idempotency.

---

## Chapter 32.3 — The RRF Ranker

### Goal

Understand how the ranker fuses multiple strategies' outputs via Reciprocal Rank Fusion (RRF), applies the curator prior, and slots in bandit exploration.

### Actions

#### 1. RRF scoring

```rust
// src/recommender/ranker.rs (lines 25–45)
const RRF_K: f64 = 60.0;  // standard RRF constant

/// Blend one strategy's ranked list into the fused map.
/// score(w) = Σ_strategy weight_s · 1/(k + rank_s(w))
/// rank is 1-based position in the strategy's output.
pub fn rrf_accumulate(
    fused: &mut HashMap<String, (f64, String, String)>,  // work_id → (score, strategy, reason)
    list: &[ScoredRec],
    weight: f64,
) {
    for (rank, rec) in list.iter().enumerate() {
        let contribution = weight * (1.0 / (RRF_K + (rank as f64) + 1.0));
        fuse.entry(rec.work_id.clone())
            .or_insert_with(|| (0.0, rec.strategy.clone(), rec.reason.clone()));
        entry.0 += contribution;  // accumulate weighted reciprocal rank
    }
}
```

> **💡 Key Concept**: RRF is **scale-invariant** — it only cares about rank position, not raw score magnitude. A strategy with cosine scores 0.0–1.0 and one with 0.0–1000 produce identical blends if rankings are identical. This is why `weight` matters (it scales each strategy's reciprocal contribution) but raw `score` doesn't.

#### 2. The blend pipeline

```rust
// src/recommender/ranker.rs (lines 108–247)
pub async fn blend(
    ctx: &StrategyContext,
    registry: &StrategyRegistry,
    seed: Option<&str>,
    user_id: Option<i32>,
    n: usize,
) -> Result<(Vec<BlendedRec>, Vec<StrategyRunInfo>), RecError> {
    let mut fused: HashMap<String, (f64, String, String)> = HashMap::new();
    let mut diagnostics = Vec::new();

    // 1. Run each enabled strategy (in config order), accumulate via RRF
    for spec in registry.specs() {
        let Some(strategy) = registry.get(&spec.name) else { continue; };

        // Skip strategies incompatible with the request type
        if user_id.is_some() && !strategy.supports_personal() { continue; }
        if user_id.is_none() && seed.is_none() { continue; }

        let started = std::time::Instant::now();
        match strategy.score(ctx, seed, user_id).await {
            Ok(list) if !list.is_empty() => {
                rrf_accumulate(&mut fused, &list, spec.weight);
                diagnostics.push(StrategyRunInfo { name: spec.name.clone(), contributed: true,
                    count: list.len(), error: None, duration_ms: started.elapsed() });
            }
            Ok(_) => { diagnostics.push(/* contributed: false */); }
            Err(e) => { warn!("strategy {} failed — falling through: {e}", spec.name);
                diagnostics.push(/* contributed: false, error: Some */); }
        }
    }

    if fused.is_empty() {
        return Err(RecError::NotEnoughData("no enabled strategy produced recs".into()));
    }

    // 2. Curator prior (personalized only)
    let mut fused_vec = fused.into_iter().map(|(w, (s, _, _))| (w, s)).collect::<Vec<_>>();
    fused_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let mut alpha_used = 1.0f64;
    if let (Some(uid), Some(curator_id)) = (user_id, ctx.config.rec_curator_prior) {
        if uid != curator_id {
            let n_signals = count_user_signals(ctx, uid).await.unwrap_or(0);
            alpha_used = curator_alpha(n_signals, ctx.config.rec_prior_floor, ctx.config.rec_curator_tau);
            let curator_top = fetch_curator_top(ctx, curator_id).await;
            if !curator_top.is_empty() {
                fused_vec = apply_curator_prior(&fused_vec, &curator_top, alpha_used);
            }
        }
    }

    // 3. Bandit exploration slots
    if let Some(bandit) = registry.get("bandit") {
        if let Some(recs) = slot_exploration(ctx, bandit.as_ref(), user_id, seed, &fused_vec).await {
            // Add up to rec_bandit_slots exploration arms (deduped)
        }
    }

    // 4. Dedupe + truncate to N
    Ok((out, diagnostics))
}
```

#### 3. Curator prior alpha

```rust
// src/recommender/ranker.rs (lines 56–73)
/// α = floor + (1 - floor) · exp(-n_signals / τ)
/// α is the CURATOR's weight: final = (1-α)·user + α·curator
/// * floor = REC_PRIOR_FLOOR (default 0.2) — minimum curator influence
/// * τ = REC_CURATOR_TAU (default 25 signals)
///
/// New user (0 signals) → α = 1.0 (pure curator); active user → α ≈ floor.
pub fn curator_alpha(n_signals: i64, floor: f64, tau: f64) -> f64 {
    let floor = floor.clamp(0.0, 0.999);
    let n = n_signals.max(0) as f64;
    floor + (1.0 - floor) * (-n / tau.max(1.0)).exp()
}

/// Re-rank toward curator's top list: final(w) = user_score·(1-α) + α·curator_score
pub fn apply_curator_prior(
    fused: &[(String, f64)],       // user-blended scores
    curator_top: &[(String, f64)],  // curator's own scored recs
    alpha: f64,
) -> Vec<(String, f64)> {
    let curator: HashMap<&str, f64> = curator_top.iter().map(|(w, s)| (w.as_str(), *s)).collect();
    let mut out: Vec<(String, f64)> = fused.iter()
        .map(|(w, s)| {
            let lift = curator.get(w.as_str()).copied().unwrap_or(0.0);
            (w.clone(), s * (1.0 - alpha) + alpha * lift)
        }).collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    out
}
```

#### 4. Bandit exploration

```rust
// src/recommender/ranker.rs (lines 259–344)
/// Thompson-sample REC_BANDIT_SLOTS exploration arms.
pub fn sample_arms(
    arms: &[(String, f64, f64)],  // (work_id, alpha, beta)
    slots: usize,
    mut rng: impl FnMut() -> f64,
) -> Vec<String> {
    let mut scored: Vec<(String, f64)> = arms.iter()
        .map(|(w, a, b)| {
            let mean = a / (a + b);
            let variance = if a + b > 2.0 { (a * b) / ((a + b).powi(2) * (a + b + 1.0)) } else { 0.25 };
            let noise = (rng() - 0.5) * 2.0 * variance.sqrt() * 3.0;  // mean + noise
            (w.clone(), (mean + noise).max(0.0))
        }).collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    scored.into_iter().take(slots).map(|(w, _)| w).collect()
}

/// Pull arms from rec_bandit_arms table, sample, log impressions.
async fn slot_exploration(ctx, bandit, user_id, _seed, fused) -> Option<Vec<ScoredRec>> {
    if ctx.config.rec_bandit_slots == 0 { return None; }
    let arms = fetch_bandit_arms(ctx).await;  // SELECT work_id, alpha, beta FROM rec_bandit_arms
    let slots = ctx.config.rec_bandit_slots.min(arms.len());
    let picked = sample_arms(&arms, slots, || {
        let t = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64;
        ((t ^ (t << 13) ^ (t >> 7)) as f64) / u64::MAX as f64  // xorshift PRNG
    });

    // Log impressions to rec_impressions table
    for w in picked { sqlx::query("INSERT INTO rec_impressions (user_id, work_id, strategy) VALUES (...)")
        .bind(uid).bind(&w).execute(&ctx.db).await; }
}
```

### Try It Yourself

```bash
# RRF + curator prior unit tests (pure functions, no DB)
cargo test --lib recommender::ranker::tests -- --nocapture

# Test the curator alpha curve
cargo test --lib recommender::ranker::test_curator_alpha -- --nocapture
```

### Check

- ✅ `rrf_accumulate` with two strategies: a work ranked #1 by both beats a work ranked #1 by only one.
- ✅ RRF is scale-invariant: scores 0.9/500.0 and 0.9/0.1 produce the same blend if rankings match.
- ✅ `curator_alpha(0, 0.2, 25)` → 1.0 (new user gets pure curator).
- ✅ `curator_alpha(100, 0.2, 25)` → ~0.2 (active user converges to floor).
- ✅ Bandit uses xorshift PRNG for deterministic sampling (reproducible tests).
- ✅ Errors from individual strategies are logged + skipped (fallback chain, never hard-fail).

### What you built

The RRF ranker — reciprocal rank fusion across strategies with config weights, the curator prior (alpha decay from 1.0→floor as signals accumulate), bandit exploration with Thompson sampling, per-strategy diagnostics for ops, and a non-fatal fallback chain where strategy errors are logged but never abort the request.

---

## Chapter 32.4 — The Recommendation API

### Goal

Understand the five recommendation endpoints and how they wire up the strategy platform, with mode switching between legacy and pluggable.

### Actions

#### 1. Route registrations

```rust
// src/server.rs (lines 369–379)
.route("/api/recommendations", get(crate::recommender::routes::recommendations_handler))
.route("/api/recommendations/embeddings", get(crate::recommendation::embedding_recs::embedding_recommendations_handler))
.route("/api/recommendations/personal", get(crate::recommender::routes::personal_recommendations_handler))
.route("/api/recommendations/strategies", get(crate::recommender::routes::strategies_handler))
.route("/api/recommendations/train", post(crate::recommender::routes::train_handler))
.route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
.route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
.route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
```

#### 2. GET /api/recommendations — item-to-item

```rust
// src/recommender/routes.rs (lines 115–167)
pub async fn recommendations_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecQueryParams>,
) -> Result<Json<Value>, AppError> {
    // Resolve url_id from either ?q=<url> (scrapes metadata) or ?url_id=<id> (DB lookup)
    let (url_id, _) = if let Some(q) = &params.q {
        let scraper = state.scraper_registry.find_specific_or_fff(q)
            .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {}", q)))?;
        let meta = scraper.lookup(&state.http_client, q).await?;
        (meta.url_id, build_seed_meta_from_meta(&meta))
    } else if let Some(id) = &params.url_id {
        (id.clone(), json!({ "id": id }))
    } else {
        return Ok(Json(json!({ "err": -1, "msg": "no query or url_id provided" })));
    };

    // Enqueue for collection if not in DB yet
    let exists = queries::get_fic_info(&state.db, &url_id).await?.is_some();
    if !exists { state.collection_worker.enqueue(&url_id, "unknown", "unknown").await?; }

    let n = params.n.unwrap_or(20).max(1).min(100) as usize;
    let rec_query = RecQuery { url_id: url_id.clone(), n, site_domain: params.site_domain.clone() };

    // Mode switch
    let recommendations = if state.config.rec_engine_mode.is_pluggable() {
        // Pluggable: run strategies through RRF ranker
        let ctx = registry::build_context(state.db.clone(), Arc::new(state.config.clone()),
            state.http_client.clone(), state.ollama.clone());
        let (blended, _) = ranker::blend(&ctx, &state.strategy_registry,
            Some(&url_id), None, n).await?;
        // Resolve metadata for each blended rec from fic_info
    } else {
        // Legacy: state.recommender_engine.get_recommendations()
        state.recommender_engine.get_recommendations(&rec_query, &state.config).await?
    };

    Ok(Json(json!({
        "err": 0, "url_id": url_id, "site_domain": params.site_domain,
        "recommendations": recommendations,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    })))
}
```

#### 3. GET /api/recommendations/personal

```rust
// src/recommender/routes.rs (lines 354–406)
pub async fn personal_recommendations_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let Some(user_id) = auth.user_id else {
        return Ok(empty_personal_response());  // anonymous → enough_data: false
    };

    // Check opt-out preference
    let opted_out = sqlx::query_scalar::<_, Option<String>>(
        "SELECT value FROM user_prefs WHERE user_id = $1 AND key = 'recs.personalized'"
    ).bind(user_id).fetch_optional(&state.db).await?
     .flatten().map(|v| v == "false").unwrap_or(false);
    if opted_out { return Ok(empty_personal_response()); }

    // Mode switch: pluggable (registry + RRF) or legacy (compute_personal_recommendations)
    if state.config.rec_engine_mode.is_pluggable() {
        return pluggable_personal_handler(state, user_id).await;
    }

    let (recs, based_on, enough) = compute_personal_recommendations(db, user_id, &state.config).await?;
    if !enough { return Ok(empty_personal_response()); }
    Ok(Json(json!({ "err": 0, "enough_data": true, "recs": recs, "based_on": based_on })))
}
```

#### 4. The legacy personal algorithm

```rust
// src/recommender/routes.rs (lines 225–333)
pub async fn compute_personal_recommendations(
    db: &PgPool, user_id: i32, config: &Config,
) -> Result<(Vec<RecResult>, Vec<(String, String)>, bool), AppError> {
    // 1. Bookmarks (up to 100) + their titles
    let bookmarks: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT b.url_id, COALESCE(fi.title, '') AS title
           FROM bookmarks b LEFT JOIN fic_info fi ON fi.id = b.url_id
           WHERE b.user_id = $1 ORDER BY b.created_at DESC LIMIT 100"#
    ).bind(user_id).fetch_all(db).await?;
    let bookmarked_ids: Vec<String> = bookmarks.iter().map(|(id, _)| id.clone()).collect();

    // 2. Download/export signal (last 90 days) — additional "known" fics
    let download_ids: Vec<String> = sqlx::query_scalar(
        r#"SELECT DISTINCT url_id FROM request_log
           WHERE url_id IS NOT NULL AND created > now() - interval '90 days'
             AND (export_file_name LIKE '%.epub' OR etype IN ('download', 'export'))"#
    ).fetch_all(db).await?;

    // 3. Gate: need at least PERSONAL_RECS_MIN_SIGNAL (3) total signals
    if bookmarked_ids.len() + download_ids.len() < PERSONAL_RECS_MIN_SIGNAL {
        return Ok((Vec::new(), Vec::new(), false));
    }

    // 4. User's top 20 tags by bookmark frequency (SQL aggregation, not Rust)
    let top_tag_ids: Vec<i32> = fetch_user_top_tags(db, user_id).await?;

    // 5. SQL candidate generation: fics sharing ≥1 top tag, excluding known fics
    let candidates: Vec<PersonalCandidateRow> = sqlx::query_as(
        r#"SELECT f.id, f.title, f.author, f.updated
           FROM fic_info f JOIN fic_tags ft ON ft.url_id = f.id
           WHERE ft.tag_id = ANY($1) AND f.id <> ALL($2)
           GROUP BY f.id ORDER BY COUNT(DISTINCT ft.tag_id) DESC LIMIT 200"#
    ).bind(&top_tag_ids).bind(&known_ids).fetch_all(db).await?;

    // 6. Rust scoring: Jaccard tag overlap + recency popularity
    for c in candidates {
        let fic_tags: Vec<i32> = sqlx::query_scalar("SELECT tag_id FROM fic_tags WHERE url_id = $1").bind(&c.id).fetch_all(db).await?;
        let overlap = tag_overlap(&top_tag_ids, &fic_tags);   // Jaccard
        let popularity = popularity_norm(c.updated, now);       // exp(-days/30)
        let score = personal_score(overlap, popularity);        // overlap + 0.2 * popularity
    }
    Ok((recs, based_on, true))
}
```

#### 5. The v0 community suggestion API

```rust
// src/recommender/routes.rs (lines 606–687)
/// POST /api/v0/recommendations/suggest — user-submitted rec (upvote/downvote)
pub async fn suggest_handler(...) -> ... {
    // Validate body, resolve suggested_url to url_id via scraper,
    // check both fics exist, INSERT into rec_suggestions
    let suggestion_id = submit_suggestion(&state.db, &body.url_id, &suggested_url_id,
        "0.0.0.0", body.comment.as_deref()).await?;
    Ok(Json(json!({ "err": 0, "suggestion_id": suggestion_id })))
}

/// POST /api/v0/recommendations/vote — upvote (1) or downvote (-1) a suggestion
pub async fn vote_handler(...) -> ... {
    let new_score = cast_vote(&state.db, body.suggestion_id, "0.0.0.0", body.vote as i16).await?;
    Ok(Json(json!({ "err": 0, "new_score": new_score })))
}

/// GET /api/v0/recommendations/votes?url_id=... — list all suggestions + votes for a fic
pub async fn votes_handler(...) -> ... {
    let suggestions = get_community_suggestions(&state.db, &params.url_id).await?;
    // Returns: url_id, suggested_url_id, comment, score, upvotes, downvotes, created_at
}
```

### Try It Yourself

```bash
# Item-to-item (legacy mode)
curl "http://localhost:8000/api/recommendations?url_id=some-fic-id&n=10" | jq '.recommendations[].title'

# Personalized (requires JWT)
curl "http://localhost:8000/api/recommendations/personal" \
  -H "Authorization: Bearer <jwt>" | jq '.enough_data'

# Community suggestions (public)
curl "http://localhost:8000/api/v0/recommendations/votes?url_id=some-fic-id" | jq '.suggestions'

# Submit a suggestion
curl -X POST "http://localhost:8000/api/v0/recommendations/suggest" \
  -H "Content-Type: application/json" \
  -d '{"url_id": "some-fic-id", "suggested_url": "https://..."}' | jq

# Vote on a suggestion
curl -X POST "http://localhost:8000/api/v0/recommendations/vote" \
  -H "Content-Type: application/json" \
  -d '{"suggestion_id": 42, "vote": 1}' | jq

# Admin: list strategies
curl "http://localhost:8000/api/recommendations/strategies" \
  -H "Authorization: Bearer <jwt>" | jq '.strategies[] | {name, enabled, weight}'

# Admin: trigger training
curl -X POST "http://localhost:8000/api/recommendations/train" \
  -H "Authorization: Bearer <jwt>" | jq '.runs[]'
```

### Check

- ✅ `GET /api/recommendations` works without auth (seed-based, anonymous-friendly).
- ✅ `GET /api/recommendations/personal` returns `enough_data: false` for anonymous.
- ✅ `GET /api/recommendations/strategies` requires `role >= 10`.
- ✅ `POST /api/recommendations/train` requires `role >= 10`.
- ✅ `n` parameter clamped to 1–100 (default 20).
- ✅ Personal recs check the `recs.personalized` user_prefs opt-out.
- ✅ Both fics must exist in the DB before a community suggestion is accepted.

### What you built

The complete recommendation API — item-to-item (seed-based, anonymous-friendly), personalized (JWT-gated, opt-out, legacy or pluggable mode), community voting (public suggest + vote), admin strategy listing, and a manual train trigger — all with mode switching via `REC_ENGINE_MODE`.

---

## Chapter 32.5 — Running the Training Pipeline

### Goal

Understand the nightly batch training pipeline that refreshes signal tables and runs each enabled strategy's `train()` method.

### Actions

The `run_training_pipeline` function in `src/recommender/worker.rs` (or the train handler at `src/recommender/routes.rs` line 588) does:

1. **Refresh `rec_user_signals`** — rebuild from bookmarks, ratings, downloads, reviews, follows (with signal weights: bookmarks=1.0, ratings=1.5, downloads=0.5, reviews=2.0, follows=1.0).
2. **Refresh `rec_user_signals`** — curator interactions are weighted 5×.
3. **Refresh `rec_user_curator_align`** — cosine alignment between user and curator tag-affinity vectors.
4. **Run each enabled strategy's `train()`** — embeddings strategy calls Ollama, MF strategy runs ALS, etc.
5. **Record metrics** in `rec_training_runs` (strategy name, ok, duration_ms, metrics JSON).

The pipeline is invoked by:
- The worker daemon (hourly/daily cron).
- `POST /api/recommendations/train` (admin manual trigger).

```rust
pub async fn train_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 { return Err(AppError::Forbidden("Admin access required".into())); }
    let results = crate::recommender::worker::run_training_pipeline(
        &state.db, &state.config, &state.http_client, &state.ollama, &state.strategy_registry,
    ).await;
    Ok(Json(json!({ "err": 0, "runs": results })))
}
```

### Try It Yourself

```bash
# Trigger training manually (admin)
curl -X POST "http://localhost:8000/api/recommendations/train" \
  -H "Authorization: Bearer <jwt>" | jq

# Check the last training run per strategy
psql dbname -c "SELECT DISTINCT ON (strategy) strategy, ok, duration_ms, metrics
  FROM rec_training_runs ORDER BY strategy, started_at DESC;"
```

### Check

- ✅ Training pipeline refreshes `rec_user_signals` from raw tables (bookmarks, ratings, downloads).
- ✅ Curator interactions get 5× signal weight.
- ✅ `POST /api/recommendations/train` requires admin role.
- ✅ Each strategy's `train()` returns a metrics JSON stored in `rec_training_runs`.
- ✅ A failed strategy's `train()` is logged + skipped, doesn't abort the pipeline.

### What you built

The training pipeline orchestration — the admin-triggered entry point that refreshes the signal tables, runs each enabled strategy's batch training, and records metrics for QA.

---

## Conclusion

You now understand FicHub's complete recommendation system:

1. **Strategy architecture** — trait-based, config-driven registry with graceful degradation, 15 pluggable algorithms.
2. **Embeddings strategy** — Ollama `nomic-embed-text` (768-d) → pgvector, with content-hash gating, per-item timeouts, and recency-decayed user profile centroids.
3. **RRF ranker** — rank-based fusion (scale-invariant), curator prior (alpha decay 1.0→0.2), bandit exploration (Thompson sampling), per-strategy diagnostics.
4. **Recommendation API** — item-to-item (anonymous), personalized (JWT-gated, opt-out), community voting (public), admin strategy listing, manual train trigger.
5. **Two-mode operation** — `legacy` (default, co-occurrence) and `pluggable` (registry + RRF), switchable at runtime.

The system degrades gracefully at every layer: strategy errors are logged + skipped (not fatal), unknown config strategies are dropped with a warning, and the curator prior ensures new users get high-quality curator-curated recs even with zero personal signals.

In Part 21 we'll build [Recipe Builder — the customization layer that lets power users override strategy blend weights via saved "recipes"].
