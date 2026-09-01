# Recommendation Engines (rec-engines)

FicHub's pluggable recommendation platform. The **default mode is `legacy`**
— `REC_ENGINE_MODE=legacy` preserves today's behavior byte-for-byte. Every
strategy below is **inert by default**: the tables exist, the registry is
built, but nothing runs until you opt in with `REC_ENGINE_MODE=pluggable`
and a `REC_STRATEGIES` list.

```
Strategy trait ──► Registry (REC_STRATEGIES) ──► RRF Ranker ──► API
     │                    │                         │
     └─ train()           └─ weights                ├─ curator prior
        (batch pipeline)                            └─ bandit slots
```

## The golden guarantee

Stage 0's invariant: a registry containing **only `cooccur`** (the legacy
engine wrapped as a strategy) must produce **identical output** to the
current engine for the same user. This is asserted by the DB-gated test
`golden_legacy_equals_cooccur_strategy` in `tests/rec_strategies.rs`.

## Strategies

| key | module | mode | what it does | artifacts |
|-----|--------|------|--------------|-----------|
| `cooccur` | `legacy_cooccur.rs` | seed + personal | the CURRENT engine, unchanged | — |
| `decay` | `decay.rs` | seed | SAR-style time-decayed co-occurrence + log-likelihood (pure SQL) | — |
| `embeddings` | `embeddings.rs` | seed + personal | Ollama nomic-embed-text (384-d) content embeddings; user profile = recency-weighted centroid → ANN | `rec_embeddings` |
| `mf` | `mf.rs` | personal | Hu–Koren implicit ALS (feature `rec-mf`; hand-rolled, no ndarray) | `rec_models`, factors on disk |
| `hybrid` | `hybrid.rs` | seed + personal | learned-weight blend of MF + embeddings + tag Jaccard | — |
| `author_graph` | `author_graph.rs` | seed | author co-bookmark + shared-tag Jaccard | `rec_author_graph` |
| `tag_graph` | `tag_graph.rs` | seed | meta-path fandom→character→relationship→freeform traversal | — |
| `sequential` | `sequential.rs` | seed (Next Up) | first-order Markov next-read from sequels + download sequences | `rec_transitions` |
| `clusters` | `clusters.rs` | personal | k-means over tag-affinity vectors | `rec_user_clusters` |
| `bandit` | `bandit.rs` | exploration | Thompson sampling (Beta posteriors); engagement = bookmark/download/completion | `rec_bandit_arms`, `rec_impressions` |
| `external` | `external.rs` | seed + personal | HTTP adapter for a Python sidecar (`REC_EXTERNAL_URL`) | sidecar |
| `weighted` | `weighted.rs` *(planned, P8#98)* | seed + personal | Zeks/flipper rarity-tier weighting: per-author overlap ratio + matches + sigma, tier weights (unique 0.2×matches, rare 0.05×, uncommon 0.005×, common 1); explainable ("you share 5 faves with this author whose list is 90% rare fic") + list-manipulation detection | `rec_author_overlap` *(planned)* |
| `mood` | `mood.rs` *(planned, P8#96)* | seed + personal | fic-level mood similarity (Neutral/Funny/Shocky/Flirty/Dramatic/Hurty/Bondy from content signals: genre ratios, description, review sentiment, tags) — "rec me something dramatic" | `rec_fic_moods` *(planned)* |
| `author_recs` | `author_recs.rs` *(planned, P8#97)* | personal | author feature vectors (genre mix, mood distribution, fandom spread, popularity band) → "authors like X" / "authors who wrote fic you kudos'd"; feeds the "Authors you might like" panel | `rec_author_vectors` *(planned)* |

## Config reference

| env | default | meaning |
|-----|---------|---------|
| `REC_ENGINE_MODE` | `legacy` | `legacy` = today's behavior exactly; `pluggable` = registry + ranker |
| `REC_STRATEGIES` | `cooccur` | comma list `name:weight`, e.g. `cooccur:0.4,embeddings:0.3,mf:0.2,bandit:0.1` |
| `REC_DECAY_HALFLIFE_DAYS` | `30` | half-life for the `decay` strategy and arm decay |
| `REC_EMBED_MODEL` | `nomic-embed-text` | Ollama embedding model |
| `REC_EMBED_DIM` | `384` | embedding dimension (must match the model) |
| `REC_MF_FACTORS` | `16` | latent factors for `mf` |
| `REC_MF_ITERS` | `10` | ALS iterations |
| `REC_MF_TRAIN_MIN_SIGNALS` | `500` | minimum unified signals before MF trains |
| `REC_BANDIT_SLOTS` | `1` | exploration slots per list |
| `REC_TRAIN_EVERY_H` | `6` | batch pipeline cadence (hours) |
| `REC_SHADOW_MODE` | `false` | compute pluggable recs but return legacy output while logging impressions |
| `REC_CURATOR_PRIOR` | unset | curator user id whose profile shapes low-signal users |
| `REC_PRIOR_FLOOR` | `0.2` | curator-prior floor α |
| `REC_CURATOR_TAU` | `25` | signals at which curator influence decays toward the floor |
| `REC_STRATEGY_TIMEOUT_SECS` | `10` | per-strategy HTTP timeout (Ollama/sidecar) |
| `REC_EXTERNAL_URL` | unset | sidecar base URL (enables `external`) |

## Curator prior (taste-shaping)

`REC_CURATOR_PRIOR=<user_id>` makes the curator's taste shape every user's
recs. The curator's interactions are weighted **5×** in `rec_user_signals`.
In the ranker:

```
α = floor + (1 − floor)·exp(−n_signals / τ)     # α is the CURATOR weight
final = (1−α)·user + α·curator
```

* brand-new user (0 signals): α = 1.0 → pure curator picks
* active user: α → floor (their taste dominates, curator floor kept)

Alignment `cos(user_centroid, curator_centroid)` is tracked per user in
`rec_user_curator_align` for QA. The home dashboard shows **"Curator's
pick"** when the response carries `curator_alpha`.

## Eval methodology

1. **Shadow first**: `REC_ENGINE_MODE=pluggable` + `REC_SHADOW_MODE=true`
   returns legacy recs to users while the pluggable pipeline computes and
   logs impressions into `rec_impressions`.
2. **Read the runbook data**: `GET /api/recommendations/strategies`
   (admin) shows per-strategy enabled state + last `rec_training_runs`
   metrics (duration, counts, ok).
3. **Engagement**: the bandit's `reconcile_impressions` nightly step turns
   impressions + later bookmarks into α/β updates — high α/β means the arm
   converts.
4. **Promote**: set `REC_STRATEGIES` to the winners (e.g.
   `cooccur:0.4,embeddings:0.3`), `REC_SHADOW_MODE=false`.

## Runbook: enabling a strategy

```bash
# 1. Shadow the embeddings strategy (users keep legacy recs, impressions logged)
REC_ENGINE_MODE=pluggable REC_SHADOW_MODE=true \
  REC_STRATEGIES=cooccur:0.6,embeddings:0.4

# 2. Let the batch pipeline run (REC_TRAIN_EVERY_H=6 embeds new works),
#    then read the eval:
curl -H "Authorization: Bearer <admin-jwt>" \
     http://localhost:8000/api/recommendations/strategies

# 3. Promote when engagement/coverage looks good:
REC_ENGINE_MODE=pluggable REC_SHADOW_MODE=false \
  REC_STRATEGIES=cooccur:0.5,embeddings:0.3,mf:0.2
```

## Python sidecar recipes

`rec-engines/` holds **recipes only** — they are NOT part of the default
runtime. Set `REC_EXTERNAL_URL` to point the `external` strategy at a
sidecar speaking the tiny JSON protocol:

```
POST {base}/score  {"seed": "...", "user_id": 7, "n": 20}
  → {"recs": [{"work_id": "...", "score": 0.5, "reason": "..."}]}
POST {base}/train  {} → {"ok": true}
```

Recipes included: **LightFM** (`lightfm_recipe.py`), **implicit**
(`implicit_recipe.py`), **Vowpal Wabbit bandit** (`vw_bandit_recipe.py`),
**RecBole** (`recbole_config.yaml`), plus a `Dockerfile` and `README.md`.

## User Recipes (v3.1)

Users can compose their own recipe blends via the Recipe Builder
(`/settings/recipes`). A recipe is a named JSON composition of strategy
weights, filters, and boosts — it runs through the existing RRF ranker
with zero new engine code.

```json
{
  "name": "dark-fic-finder",
  "blend": { "cooccur": 0.3, "tag_graph": 0.4, "embeddings": 0.3 },
  "filters": { "min_words": 30000, "exclude_warnings": ["Major Character Death"] },
  "boost": { "tag": "Dark Harry Potter", "weight": 1.5 }
}
```

**Activation:** `POST /api/recipes/{id}/activate` sets the recipe as
active (mutual exclusion — only one active recipe per user). The active
recipe's `to_strategy_weights()` method is called at rec-request time,
overriding the default `REC_STRATEGIES` env config.

**Gallery:** publish a recipe (`POST /api/recipes/{id}/publish`) to make
it available in the public gallery. Other users can browse
(`GET /api/recipes/gallery`) and install copies.

**Plugin manifest:** recipes, themes, layouts, and views are all stored
in the unified `extensions` table with kind/slug/version/tier/payload.
