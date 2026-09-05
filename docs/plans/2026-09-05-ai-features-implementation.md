# Plan: Implementing the AI Features from `docs/ideas/ai-features-orange-pi.md`

**Audience:** a junior dev implementing task-by-task, one commit per task. Every task is independently shippable and verified by `cargo test --lib` + the task's own smoke test.

**Repo state this plan was written against:** main @ `5e18ecd` (2026-09-05). Stack: Rust/Axum + SvelteKit (adapter-static) + Postgres + Redis. Existing AI plumbing you will reuse (do NOT create parallel systems):
- `crate::services::ollama::OllamaClient` — already in `AppState` (`state.ollama`), used by embeddings/translation/triage/translation-queue
- `src/recommender/` — strategy registry + 9 strategies (cooccur, decay, embeddings, hybrid, author_graph, tag_graph, sequential, clusters, bandit) with a nightly training pipeline in `src/server.rs` (~line 205, `run_training_pipeline` every `REC_TRAIN_EVERY_H` hours)
- `pgvector` already in the DB: `rec_embeddings(embedding vector(768))` (migration `001_initial.sql:2423`), populated by the embeddings strategy via Ollama `nomic-embed-text` (768-d)
- Background-job pattern: `tokio::spawn` + `loop` + `sleep` in `src/server.rs:205-225` — copy this pattern for new batches; models are load-per-run + dispose, never daemons
- `/ask` NL→query translation already exists (`src/search/ask.rs`, Ollama llama3.1:8b → validated v2 query string, Redis 24h cache, graceful keyword fallback)
- `usage_events` table (`001_initial.sql:3206`) — the observability target

**Hardware rule from the ideas doc:** request-time = only ONNX ≤200MB / traditional ML / pgvector binary ops. LLM (3B via Ollama) = background batches only, load → run → dispose.

---

## Already implemented (do NOT redo)

| Idea # | Status | Evidence |
|---|---|---|
| #2 Semantic "More Like This" | **DONE** | `src/routes/search.rs::similar_by_vector` at `GET /api/search/similar/{work_id}`, pgvector `<=>` KNN with tag-based fallback; embeddings strategy in `src/recommender/embeddings.rs` |
| #6 Bandits for Blind Date | **DONE (in recs, not Blind Date)** | `src/recommender/bandit.rs` is already **Thompson sampling** over `rec_bandit_arms` with `REC_BANDIT_SLOTS`; but `src/routes/blind.rs` still picks `ORDER BY random()` — see Task 1 (the only remaining glue) |
| #10 NL query translation for /ask | **DONE** | `src/search/ask.rs` translates NL → v2 query string incl. `@char:`, `relationship:`, `include_tags` facets (`src/search/builder.rs:37-52`), cached in Redis |
| #15 Markov "Next Up" | **MOSTLY DONE** | `src/recommender/sequential.rs` (first-order Markov over `rec_transitions`, trained nightly) is registered in `available_strategies` — but the reader's Next Up panel uses `src/routes/reader.rs::sequel` (series-order only), NOT the strategy. See Task 2 |
| #20 QA triage | **DONE** | `qa/triage.js` exists (ollama llama3.2:3b, single-shot) |
| Auto-tagging (#8's core) | **DONE** | `src/services/auto_tagger.rs` — zero-shot via `tag_embeddings` + cosine, `is_machine_suggested` queue |
| Content warnings scan (#7's core) | **DONE** | `src/services/content_scan.rs` — LLM pass over bodies → `warnings` flags feeding the `no_warnings` filter |
| Comment triage | **DONE** | `src/services/comment_triage.rs` → `comment_triage` table + curator queue |
| Translation drafting (#17) | **DONE** | `src/services/translation.rs` — 3-tier fallback, Redis `translate_queue`, `translation_strings` cache (migration 074) |
| Fic-request candidate matching (#4's core) | **DONE** | `src/routes/requests.rs::candidates` — engine-backed suggestions for requests **with a seed work** |
| Zero-result search mining | **DONE** | `src/services/search_mining.rs` |
| Embedding dedupe | **DONE** | `src/services/embedding_dedupe.rs` |
| Spam shadowban (#19's data) | **PARTIAL** | `bot_scores` table exists (`001:412`) — no ML scorer. Rejected for now (see "Rejected ideas") |

The ideas doc predates several of these — it was written against an earlier tree.

---

## Tasks (worth implementing)

## Task 1: Bandit-driven Blind Date pick (idea #6 — the missing glue)

**Files:** `src/routes/blind.rs`

The bandit strategy already does Thompson sampling, but Blind Date ignores it: `blind_date_handler` (~line 67-80) does `ORDER BY random() LIMIT 1` over eligible fics.

**Steps:**

1. In `blind_date_handler`, replace the random pick with a bandit-sampled pick. Query the eligible set as today (same `WHERE` filters — do not change eligibility), then instead of `ORDER BY random() LIMIT 1`:
   - Load `alpha, beta` from `rec_bandit_arms` for those work ids (strategy name `'blind_date'`), defaulting missing arms to `(1,1)`.
   - Sample one `Beta` draw per candidate **in Rust**. Add `rand_distr = { version = "0.4", features = ["std_math"] }` to `Cargo.toml` and use `Beta::new(alpha, beta).sample(&mut rng)`. Pick the max sample.
   - **Cap candidate load:** sample at most 200 eligible rows first (`ORDER BY random() LIMIT 200`), then bandit-rank those. This keeps the arm load bounded and preserves serendipity.
2. Record the impression: `INSERT INTO usage_events (client_id, path, event_type, user_agent) VALUES ($1, '/api/blind-date', 'blind_date_impression:' || $2, '')` where `$2` = picked work id. (`event_type` is text — use the `feature:work_id` convention, don't migrate the table.)
3. Record the reward in the existing reveal handler (`blind_date_reveal_handler`, ~line 154): on successful reveal, insert `'blind_date_click:' || work_id`. The nightly bandit `train` already reconciles impressions → rewards; check `bandit.rs::train` handles event types other than rec impressions — if hard-wired to rec paths, add a branch matching the `blind_date_` prefix (impression on `blind_date_impression:X` pairs with click on `blind_date_click:X`).
4. **TDD:** unit-test the pure selection fn: given arms `[(w1, 9.0, 1.0), (w2, 1.0, 9.0), (w3, 1.0, 1.0)]`, over 100 samples `w1` wins most but `w2`/`w3` win sometimes (exploration). Also test missing-arm default `(1,1)`.
5. **Verification:** `cargo test --lib routes::blind recommender::bandit` → green. Manual: hit `/api/blind-date` 5×, confirm `usage_events` rows appear and the picked work varies.
6. **Commit:** `feat(blind-date): Thompson-sampling pick over eligible fics with impression/click rewards`

---

## Task 2: Wire the Markov "Next Up" into the reader (idea #15 — wire-up only)

**Files:** `src/routes/reader.rs`, `frontend/src/routes/read/[urlId]/+page.svelte`

The sequential strategy (`src/recommender/sequential.rs`, transitions in `rec_transitions`, trained nightly) is live in the recommender but the reader's sequel endpoint doesn't use it.

**Steps:**

1. `src/routes/reader.rs` has `GET /api/reader/{url_id}/sequel` (~line 137) returning `{ "sequel": {...} }`. Extend the response: after the series-order lookup, also fetch the sequential strategy's top picks from this work (3 items), returned as `"next_up": [{url_id, title, author, score}]` alongside `sequel`. Resolve url_ids via the same `fic_info` lookup pattern already in that handler (~line 180).
2. Implementation: call `state.strategy_registry` the way `src/recommender/routes.rs::recommendations_handler` does (copy its `StrategyContext` construction) with `RecQuery { url_id, n: 3, site_domain: None }`, then filter results to `name == "sequential"`. If the registry approach is awkward, query `rec_transitions` directly (`SELECT to_work, weight FROM rec_transitions WHERE from_work = $1 ORDER BY weight DESC LIMIT 3`) — acceptable for a first cut, note it in the commit message.
3. Degrade silently: empty `rec_transitions` → `"next_up": []`. Never 5xx the reader page over this.
4. **Frontend:** in `frontend/src/routes/read/[urlId]/+page.svelte`, the "Next Up" panel (grep `sequel`) — render `next_up` items below the sequel link as a small list.
5. **TDD:** DB-gated test asserting transitions rows → ordered `next_up` output (mark `#[ignore]`-style like other DB tests). Pure-fn test: empty transitions → empty vec.
6. **Verification:** `cargo test --lib routes::reader` → green; manual: open a work with transition data, see the panel.
7. **Commit:** `feat(reader): Next Up panel from Markov transitions (sequential strategy)`

## Task 3: Taste-cluster endpoint (idea #5 — expose existing data)

**Files:** `src/recommender/routes.rs`, `src/server.rs`

`rec_user_clusters` is already populated nightly by the clusters strategy — nothing writes a user-facing endpoint.

**Steps:**

1. New handler `GET /api/users/{id}/taste-cluster` in `src/recommender/routes.rs`:
   - SQL: `SELECT cluster_id, affinity FROM rec_user_clusters WHERE user_id = $1 ORDER BY affinity DESC LIMIT 1`; adjacent = other distinct cluster ids for the user.
   - Response: `{ "cluster": n, "affinity": 0.83, "adjacent": [1,4,7] }`; logged-out or no row → `{ "cluster": null }` with `err: 0`. Logged-in only (`AuthUser`).
2. Register in `src/server.rs` near the other `/api/users` routes.
3. **TDD:** DB-gated test: seeded `rec_user_clusters` rows → correct top cluster + adjacency. Handler test for the logged-out path.
4. **Verification:** `cargo test --lib recommender::routes` → green; curl with a session cookie of a bookmarking user after a training run.
5. **Commit:** `feat(recs): taste-cluster endpoint exposing rec_user_clusters`

---

## Task 4: Fic-request auto-matching for seedless requests (idea #4 — the other half)

**Files:** `src/routes/requests.rs`, new `migrations/078_request_body_embedding.sql`

`candidates` (~line 777) returns engine recs only when the request has `seed_work_id`; seedless (pure-text) requests get `[]`.

**Steps:**

1. New migration `078_request_body_embedding.sql`: `ALTER TABLE fic_requests ADD COLUMN body_embedding public.vector(768);`
2. Hook the request-create handler: after insert, embed `title + body` via `state.ollama.embed()` (copy the timeout/error handling from `src/recommender/embeddings.rs:201`) and store the vector. Ollama down → leave NULL, never fail the request creation.
3. In `candidates`: when `seed_work_id IS NULL` but `body_embedding IS NOT NULL`, KNN against `rec_embeddings` with `<=>` (copy the SQL shape from `similar_by_vector` in `src/routes/search.rs:20`), return top 3 with `"source": "semantic"` (seeded path → `"source": "engine"`).
4. **TDD:** unit-test the pure "build embed text from request title+body" fn. DB-gated: seeded `rec_embeddings` + request row with embedding → top-3 returned in cosine order.
5. **Verification:** `cargo test --lib routes::requests` → green. Manual: create a text-only request, confirm candidates non-empty. If embedding in the create path proves too slow, move it to the nightly worker (`recommender/worker.rs` pattern) and note it in the commit.
6. **Commit:** `feat(requests): semantic auto-match for seedless fic requests`

---

## Task 5: Personalized search chips (idea #3)

**Files:** `src/routes/search.rs` (new handler), `src/server.rs`, `frontend/src/routes/search/+page.svelte`

The search page shows static sort/filter chips. The idea: swap "Popular"-style chips for "For You" chips derived from the user's bookmark embeddings.

**Steps:**

1. New handler `GET /api/search/chips` (auth optional; logged-out → static defaults):
   - Load the user's recent bookmarks' embeddings: `SELECT re.embedding FROM rec_embeddings re JOIN bookmarks b ON b.work_id = re.work_id WHERE b.user_id = $1 ORDER BY b.created_at DESC LIMIT 20`.
   - Average them **in Rust** (parse pgvector text/float8[] format the same way existing pgvector reads do — check how `similar_by_vector` binds vectors; reuse that parsing). No Redis for v1 — compute per request, it's a 20-row query + arithmetic. Add Redis caching later only if measurements say so (the idea doc's Redis note is premature; note this deviation in the commit message).
   - Build 3 personalized chips from the user's **top tag affinities** instead of raw vector math against trending queries (trending query embeddings don't exist yet and building them is out of scope): `SELECT t.name FROM bookmarks b JOIN work_tags wt ON wt.work_id = b.work_id JOIN tags t ON t.id = wt.tag_id WHERE b.user_id = $1 GROUP BY t.name ORDER BY count(*) DESC LIMIT 3` — cheaper, explainable, and needs no vector plumbing. The embedding centroid is still computed and logged as `chips_centroid_computed` in `usage_events` so we keep the option warm.
   - Response: `{ "chips": ["for-you:harry potter", "for-you:slow burn", ...], "personalized": true }`.
2. Frontend: in `search/+page.svelte`, fetch chips on mount; if `personalized`, render a "For you" chip row above the existing static filters. Clicking a chip runs a normal search with that tag (`include_tags`-style query, same as other tag links on the page).
3. **TDD:** unit-test the centroid-average pure fn (float parsing + mean). Handler test: logged-out → static chips.
4. **Verification:** `cargo test --lib routes::search` → green; `npx svelte-check --threshold error` → clean; manual: bookmark a few works, observe chips.
5. **Commit:** `feat(search): personalized For You chips from bookmark tag affinities`

---

## Task 6: Reading-time estimation v1 (idea #14, heuristic only)

**Files:** `src/routes/` (work meta serializer — grep where `words`/`reading time` is computed), frontend work card

The ideas doc wants a regression model trained on `reading_sessions` — **that table does not exist** (only `reading_history`, which tracks progress, not session durations). A model would have no training data. Ship the honest heuristic now; leave the model as a follow-up gated on data collection.

**Steps:**

1. Find the current words/min calculation (grep `wpm\|words_per_min\|reading` in `src/`). Replace with a dialogue-aware estimate: pure dialogue reads faster than dense prose. Cheap proxy without NLP: dialogue ratio ≈ count of `"` marks per 1k chars (already computable from the cached first chapter, or default 0 when body not cached).
   - `est_minutes = words / (240 + 60 * dialogue_ratio)` clamped to `[120, 400]` wpm. Constants in one `pub const` block with comments.
2. Store nothing: compute on the fly per work page (it's O(len of first chapter) — cheap) or cache on the work row only if the page query already touches the body. Do not add a migration.
3. **TDD:** unit tests for the pure estimator: zero dialogue → 240 wpm; all-dialogue → 300 wpm; clamping edge cases.
4. **Verification:** `cargo test --lib` green; check the work page renders the new estimate.
5. **Commit:** `feat(meta): dialogue-aware reading time estimate`

---

## Rejected ideas (do not implement)

| Idea # | Why not |
|---|---|
| #1 /ask facet cache table | `/ask` already caches translations in Redis (24h) — a second `ask_cache` Postgres table duplicates that. Skip. |
| #7 Content Shield spice spans | `content_scan.rs` already classifies violence/sexual/profanity per work and feeds the `no_warnings` filter. Passage-level spans need an ONNX BERT the hardware budget can't serve at import scale; revisit only if curators ask for tap-to-reveal. |
| #8 tag merge suggestions | `auto_tagger.rs` + `embedding_dedupe.rs` + curator proposals already cover tag suggestions; a separate `tag_merge_suggestions` table would be a third overlapping queue. If curators want merges specifically, extend the existing proposals pipeline, not a new one. |
| #9 NER character attributes | v2 search already has `@char:`/`relationship:` facets populated from scraped tags; NER adds a 150MB model for marginal recall. Revisit if users report missing characters. |
| #11 scraper self-healing NER | The heal agent (`src/heal/`) already uses the on-the-fly scraper creator; a DOM-locating classifier is speculative until heal failures are frequent enough to matter. |
| #12 quote cards | Fun but zero data model behind it; would need chapter-text pipeline work + a card generator UI. Defer until requested. |
| #13 mood classification | Ollama single-shot per work would take days for 100k works on the Pi; the ONNX emotion classifier adds a new model class for a nicety. Tags already carry mood signal (the chips in Task 5 surface it). |
| #16 reaction suggestion | Adds a ~100MB ONNX classifier to the comment POST hot path to pre-highlight an emoji. Weak value; violates the request-time budget spirit. |
| #17 translation drafts | `translation.rs` (M1, migration 074) already does machine translation + review flow. Done. |
| #18 update-cadence forecasting | `statrs` ARIMA-style forecasting on sparse, irregular author update histories produces confident nonsense. Defer until we have ≥6 months of dense update data. |
| #19 spam shadowban | `bot_scores` exists but training an Isolation Forest without labeled outcomes risks shadowbanning real users silently. `comment_triage` + pow/limiter already cover the worst abuse. Revisit when spam is an actual problem. |
| #20 QA triage categorization | `qa/triage.js` exists; grouping is a nice-to-have on a dev-only tool. Skip. |

---

## Suggested execution order

1 → 2 → 3 (pure wire-ups, zero new deps except `rand_distr` in Task 1) → 4 (small migration) → 5 → 6. Tasks 1–4 are backend-only-plus-tiny-frontend; 5–6 touch user-visible ranking/estimates so they benefit from running after the wire-ups prove the pipelines have data.

## Verification (whole plan)

```bash
cd ~/code/rust/ficnexus
cargo test --lib            # all tasks' unit tests green
cargo clippy --lib -- -D warnings
cd frontend && npx svelte-check --threshold error   # 0 errors
npm run build               # adapter-static build succeeds
```

Manual smoke: Blind Date pick varies and logs events; reader shows Next Up; taste-cluster endpoint returns data for a bookmarking user; seedless fic request produces semantic candidates; search page shows For You chips; work page reading time changes with dialogue density.

## Notes for the implementer

- **Never add a new AI service without checking `src/services/` first** — there are already 18 services; most "new" ideas are extensions of existing ones.
- All Ollama calls must keep the existing pattern: per-call timeout, best-effort failure (log + degrade), never fail the parent request.
- Every feature logs to `usage_events` with the `feature:event` `event_type` convention (Task 1 shows the pattern) so adoption is measurable.
- The ideas doc's model-lifecycle rules apply: no new resident models; Ollama only in background batches or per-request single-shots with timeouts; anything requiring a new ONNX dependency (`fastembed`, `ort`) is out of scope for this plan and needs a new hardware-budget discussion.
- Line numbers in this plan refer to main @ `5e18ecd`; re-grep before editing — `server.rs` especially tends to shift.
