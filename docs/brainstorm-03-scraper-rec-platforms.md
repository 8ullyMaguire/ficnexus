<!-- CONSOLIDATED BRAINSTORM FILE — Generated 2026-08-13, updated 2026-08-17 from repo markdown (docs/src user docs excluded). -->

> THE TWO PLATFORMS: (1) fanfic-scrapers crate — native Rust adapters with FULL FanFicFare parity (107/107 real sites), login support, is_adult gate, FF CLI removed (gated behind fff-fallback feature); (2) the recommendation platform — pluggable strategy registry (cooccur/decay/embeddings/mf/hybrid/author_graph/tag_graph/sequential/clusters/bandit/external/curator), RRF ranker, shadow-mode rollout, operator playbook, and Python sidecar recipes.


---


======================================================================
SOURCE: scrapers/README.md
======================================================================

# fanfic-scrapers

General-purpose fanfiction scraping library (AGPL-3.0), extracted from the
FicHub codebase and made site-agnostic so any project can use it.

## What it does

- Shared, site-agnostic metadata model: `FicMetadata`, `Chapter`,
  `ExtractedTag`, `ScrapeError`.
- Pluggable adapter trait `SiteScraper` — implement it for your own site and
  register it.
- A `Registry` that routes URLs to the right adapter (native scrapers
  preferred; a FanFicFare catch-all is optional behind the `fff-fallback`
  feature).
- Native adapters: AO3, FanFiction.net, FictionPress, RoyalRoad, XenForo
  boards (+XenForo2: alternatehistory, althistory, the-sietch),
  HPFanficArchive, adultfanfiction.org, FimFiction, Literotica, ScribbleHub,
  WordPress-novel sites, the eFiction family (19 niche archives), the
  OTW-family siblings (adastrafanfic, cfaa, squidgeworld, superlove),
  FicBook, Wattpad, SpiritFanfiction, Syosetu, AsianFanFics, DeviantArt,
  Twisting the Hellmouth, StoriesOnline family (storiesonline.net,
  scifistories, storyroom), Quotev, Fanfictions.fr, Kakuyomu, MediaMiner,
  ChiReads, PMDFanFiction, LCFanFic, FireflyFans, FicWad, FictionMania,
  FanFiktion.de, TouchFluffyTail, Fanfics.me, FanFicAuthors, FictionHunt,
  InkBunny, SoFurry, Dokuga, PhoenixSong, StoriesOfArda, Fictionalley,
  NovelAll, MassEffect2.in, ReadonlyMind, UtopiaStories, ASexStories,
  AnEroticStory, MCStories, HentaiFoundry, BDSMLibrary,
  and the eFiction-variant
  family (26 viewstory.php?sid= archives: psychfic, wolverineandrogue,
  sycophanthex sites, walkingtheplank, themasque, ksarchive, twilighted,
  whofic, etc.) — **full FanFicFare adapter parity (all 107 real
  sites)**. Login support for author/adult-gated sites (fanfics.me,
  fictionhunt, inkbunny, sofurry, dokuga) + is_adult flag for adult
  archives (readonlymind, utopiastories, asexstories, aneroticstory,
  mcstories, hentaifoundry, bdsmlibrary).
- Optional FanFicFare fallback via its `--meta-only` CLI output, gated
  behind the `fff-fallback` Cargo feature (off by default — the native
  adapter set already covers every real FFF site, so the CLI is only
  useful as a safety net for future FFF-only additions).
- Optional LLM-assisted author-profile-link extraction (`author_link`
  module) — own generic HTML patterns first, LLM fallback for unknown
  sites. The LLM interface is a tiny `LlmClient` trait so hosts plug in
  their own provider (Ollama, OpenAI, ...).

## Usage

```rust
use fanfic_scrapers::{Registry, NoopHealHook};
use fanfic_scrapers::sites;

let mut registry = Registry::new();
// Native scrapers preferred — one per site.
registry.register(Box::new(sites::ao3::Ao3Scraper));
registry.register(Box::new(sites::ffnet::FfNetScraper));
// ... (all 107 parity adapters)
// Optional catch-all (requires the `fff-fallback` feature + the CLI):
// registry.register(Box::new(sites::fanficfare::FanFicFareScraper));

let client = reqwest::Client::new();
let meta = registry
    .lookup(&client, "https://archiveofourown.org/works/21845264", None)
    .await?;

let chapters = registry.fetch_chapters(&client, &meta).await?;
```

## License

AGPL-3.0-or-later. The library itself carries no FicHub code or types; it
is a standalone reusable component.


======================================================================
SOURCE: scrapers/FFF_PARITY.md
======================================================================

# FanFicFare parity tracking

This file records which FanFicFare version the crate's adapters are
tracked against, so updates can diff precisely against the tracked
baseline.

## Tracked version

**v4.60.0** — https://github.com/JimmXinu/FanFicFare/releases/tag/v4.60.0
(commit `86832ac463d00ac6f1dfc10c94c47c0127c2a67c`, 111 adapters)

Local baseline checkout (shallow clone of the GitHub tag, on local
ext4 — do NOT use the NFS reference clone for this):

    /home/alvaro/.cache/fff-tags/FanFicFare-v4.60.0

## How to update the crate to match a new FFF release

1. Fetch the new tag into the baseline area:

       cd /home/alvaro/.cache/fff-tags
       git clone --depth 1 --branch v<NEW> \
         https://github.com/JimmXinu/FanFicFare.git FanFicFare-v<NEW>

2. Diff the adapters between the new tag and the tracked baseline:

       diff -rq FanFicFare-v<NEW>/fanficfare/adapters \
                FanFicFare-v4.60.0/fanficfare/adapters

   Also check `base_*.py` (the shared base classes: base_adapter,
   base_efiction, base_xenforo, base_otw, etc.) and `defaults.ini`.

3. For each changed/added `adapter_<site>.py`:
   - If the crate already has a native `sites/<site>.rs` adapter,
     port the changed selectors/URL logic into it.
   - If it's a new site not yet natively covered, port it as a new
     `sites/<site>.rs` module (follow the existing adapter pattern:
     scoped-doc Send discipline, shared `sites::http::fetch`).
   - Register in BOTH the crate registry (`scrapers/src/registry.rs`)
     and the FicHub wrapper (`src/scrape/registry.rs`).
4. Add unit tests per adapter; run the crate suite (`cargo test`).
5. Update this file's "Tracked version" + bump the crate version,
   push to opencommit.eu, `cargo publish`.

## Reference clone caveats

- `/home/alvaro/code/python/ao3/fanficfare` is a PERSONAL mirror
  (remotes: hirrolot Forgejo + git.disroot.org), has NO tags, and lives
  on NFS. Do not fetch GitHub tags there (NFS object-close errors).
- The GitHub repo (JimmXinu/FanFicFare) is the canonical source for
  tags. Fetch tags only into `/home/alvaro/.cache/fff-tags/` (local
  ext4).

## Native site coverage (as of crate v0.10.0 — FULL PARITY)

**All 107 real FanFicFare adapters now have native Rust ports.** The only
FFF adapters without a native port are `test1`-`test4`, which are FFF's
own internal test fixtures (fake adapters with no real site — not parity
targets).

- AO3 + OTW siblings (adastrafanfic, cfaa, squidgeworld, superlove)
- FanFiction.net, FictionPress, RoyalRoad, HPFanficArchive,
  adultfanfiction.org, ScribbleHub, FimFiction, Literotica
- XenForo boards (spacebattles, sufficientvelocity, questionablequesting,
  theforce, fiction.live, alternatehistory, althistory, the-sietch)
- WordPress-novel sites (novelfull + family)
- eFiction family (19 niche archives)
- eFiction-variant family (26 viewstory.php?sid= archives: psychfic,
  wolverineandrogue, sycophanthex, walkingtheplank, themasque, ksarchive,
  twilighted, whofic, etc.)
- StoriesOnline family (storiesonline, scifistories, storyroom)
- FicBook, Wattpad, SpiritFanfiction, Syosetu, AsianFanFics, DeviantArt,
  Twisting the Hellmouth, Quotev, Fanfictions.fr, Kakuyomu, MediaMiner,
  ChiReads, PMDFanFiction, LCFanFic, FireflyFans, FicWad, FictionMania,
  FanFiktion.de, TouchFluffyTail, Fanfics.me, FanFicAuthors, FictionHunt,
  InkBunny, SoFurry, Dokuga, PhoenixSong, StoriesOfArda, Fictionalley,
  NovelAll, MassEffect2.in, ReadonlyMind, UtopiaStories, ASexStories,
  AnEroticStory, MCStories, HentaiFoundry, BDSMLibrary
- **Login support**: fanfics.me, fictionhunt, inkbunny, sofurry, dokuga
  (form/CSRF/token login via `SiteScraper::login` + `SiteCredentials`)
- **`is_adult` gate**: readonlymind, utopiastories, asexstories,
  aneroticstory, mcstories, hentaifoundry, bdsmlibrary (unlocked via
  `SiteCredentials::with_adult()`)
- **FanFicFare CLI removed** (user directive 2026-08-12): the `fanficfare`
  fallback adapter is gated behind the crate's `fff-fallback` feature (off
  by default) and FicHub registers native adapters only. Parity covers all
  107 real FFF sites natively; there is no documented CLI fallback.


======================================================================
SOURCE: rec-engines/README.md
======================================================================

# Rec-engines sidecar recipes (OPTIONAL — not in the default runtime)

These are **recipes only**. The FicHub server never runs Python; to use any
of these you run the sidecar yourself and point the `external` strategy at
it via `REC_EXTERNAL_URL`.

## Protocol

```
POST {base}/score
{"seed": "<url_id or null>", "user_id": <int or null>, "n": 20}
→ 200
{"recs": [{"work_id": "...", "score": 0.5, "reason": "..."}]}

POST {base}/train
{} → 200 {"ok": true}
```

## Recipes

* `lightfm_recipe.py` — LightFM hybrid (content + collaborative) on
  `rec_user_signals`-shaped CSVs.
* `implicit_recipe.py` — implicit ALS (the `implicit` package) — a drop-in
  for the Rust `mf` strategy with more factors.
* `vw_bandit_recipe.py` — Vowpal Wabbit contextual bandit over
  impressions (better than the built-in Thompson sampler at scale).
* `recbole_config.yaml` — RecBole model config (e.g. BPR, NCF) for offline
  leaderboard experiments.

## Docker

```bash
docker build -t fichub-rec-sidecar .
docker run -d -p 8300:8300 \
  -e DATABASE_URL=postgres://fichub:fichub@host.docker.internal:5432/fichub \
  fichub-rec-sidecar
# then: REC_EXTERNAL_URL=http://127.0.0.1:8300
```


======================================================================
SOURCE: REC-PLATFORM-SUMMARY.md
======================================================================

# Recommendation platform — worktree summary (wt-rec-platform)

## What shipped

**Stage 0 — Foundation (golden)**
- `src/recommender/strategy.rs` — `RecStrategy` trait + `ScoredRec` /
  `StrategyContext` / `RecError`.
- `src/recommender/registry.rs` — config-driven registry (`REC_STRATEGIES`,
  `name:weight` list; default `cooccur`).
- `src/recommender/ranker.rs` — RRF blend (`k=60`), fallback chain (a
  failing/empty strategy is skipped), curator prior, bandit exploration
  slots, attribution.
- `src/recommender/signals.rs` — unified signal view (bookmarks 4.0,
  downloads 2.0, ratings 1..=5, reviews 3.0; curator ×5).
- `src/recommender/legacy_cooccur.rs` — the CURRENT engine wrapped as the
  default `cooccur` strategy. Golden test
  `golden_legacy_equals_cooccur_strategy` asserts identical output.
- Config plumbing in `src/config.rs` (`REC_ENGINE_MODE` default `legacy`,
  `REC_STRATEGIES`, `REC_DECAY_HALFLIFE_DAYS`, `REC_EMBED_MODEL`,
  `REC_EMBED_DIM`, `REC_MF_FACTORS`, `REC_MF_ITERS`, `REC_MF_TRAIN_MIN_SIGNALS`,
  `REC_BANDIT_SLOTS`, `REC_TRAIN_EVERY_H`, `REC_SHADOW_MODE`,
  `REC_CURATOR_PRIOR`, `REC_PRIOR_FLOOR`, `REC_CURATOR_TAU`,
  `REC_STRATEGY_TIMEOUT_SECS`, `REC_EXTERNAL_URL`).
- Personal handler: `REC_ENGINE_MODE=pluggable` → registry+ranker;
  `legacy` (default) → unchanged code path. Legacy rec computation moved to
  `routes::compute_personal_recommendations` (byte-identical SQL).

**Strategies (all disabled except `cooccur`)**
- `decay` — SAR time-decay + log-likelihood, pure SQL.
- `embeddings` — Ollama nomic-embed-text 384-d pgvector, content-hash gated,
  user centroid → ANN, item-to-item.
- `mf` — Hu–Koren implicit ALS (hand-rolled, Cargo feature `rec-mf`; stub
  without the feature).
- `hybrid` — MF + embeddings + tag Jaccard blend.
- `author_graph` — author co-bookmark + tag Jaccard.
- `tag_graph` — meta-path traversal.
- `sequential` — Markov next-read (Next Up source).
- `clusters` — k-means over tag-affinity vectors.
- `bandit` — Thompson sampling + impressions + decay.
- `external` — HTTP sidecar adapter (`REC_EXTERNAL_URL`).

**Pipeline** — `worker::run_training_pipeline` runs every
`REC_TRAIN_EVERY_H` hours: refresh signals → each enabled strategy's
`train()` → `rec_training_runs` bookkeeping. Admin
`POST /api/recommendations/train` triggers it manually; admin
`GET /api/recommendations/strategies` lists state + last-run metrics.

**Frontend** — home dashboard shows "🧑‍🏫 Curator's pick" when the response
carries `curator_alpha`; `PersonalRecsResponse` extended with
`strategies`/`curator_alpha`.

**Docs** — `docs/src/rec-engines.md` (+ SUMMARY), STATUS.md stage table,
TODO.md section, README architecture note. `rec-engines/` Python sidecar
recipes (LightFM, implicit, VW bandit, RecBole, Dockerfile) — recipes only.

## Migration

`migrations/027_rec_platform.sql` — 10 `rec_*` tables + HNSW index +
`CREATE EXTENSION IF NOT EXISTS vector`. Applied to the live local DB
(and recorded in `_sqlx_migrations`).

## Tests

| suite | result |
|-------|--------|
| `tests/rec_strategies.rs` (golden + registry + RRF + curator, 4 DB-gated) | ✅ |
| `tests/rec_embeddings.rs` (item-to-item cosine, content-hash gating) | ✅ |
| `tests/rec_bandit.rs` (engagement→α, cold→β, decay) | ✅ |
| `tests/rec_curator.rs` (prior pull, floor, alignment) | ✅ |
| `tests/recommender_personal.rs` (pre-existing, kept green) | ✅ |
| lib `recommender::*` unit tests (57) | ✅ |
| frontend vitest (HomeDashboard 7, recommendations 3+3) | ✅ |
| `cargo check --all-targets` (default + `--features rec-mf`) | ✅ |

## Notes

- Everything inert by default: `REC_ENGINE_MODE=legacy` preserves today.
- Suggest/vote endpoints untouched (fic-suggestions branch owns them).
- `.env` copied into the worktree for DB-gated tests (gitignored).


======================================================================
SOURCE: docs/using-recommender-platform.md
======================================================================

# Using the Recommendation Platform — Operator's Playbook

> The pluggable recommendation platform (migration 027, `src/recommender/`)
> ships with 11 strategies, but `REC_ENGINE_MODE=legacy` preserves the old
> single-engine behavior by default. This doc explains how to actually USE
> the platform — the knobs, the evaluation loop, and the rollout order.

## What you have

| Stage | Strategy | What it does | When to enable |
|-------|----------|--------------|----------------|
| 0 | `cooccur` (legacy) | item-to-item co-occurrence (bookmarks/downloads), the old engine | always (default) |
| 1 | `decay` | SAR-style time-decayed co-occurrence, log-likelihood | first improvement |
| 2 | `embeddings` | pgvector: title+summary+tags → NN recs (Ollama nomic-embed-text) | cold-start + semantic similarity |
| 3 | `mf` | Hu–Koren implicit ALS matrix factorization (feature `rec-mf`) | after enough signals (≥500) |
| 4 | `hybrid` | learned-weight blend of MF + embeddings + tag Jaccard | the "final blend" target |
| 5 | `author_graph` | co-bookmark + tag Jaccard at author level | author discovery |
| 6 | `tag_graph` | meta-path traversal fandom→char→ship→freeform | niche/trope discovery |
| 7 | `sequential` | Markov next-read (reader position, sequels) | "Next Up" panel |
| 8 | `clusters` | k-means over tag affinity | taste segments, "people like you" |
| 9 | `bandit` | Thompson sampling exploration slots | production tuning |
| 10 | curator prior | α-blend user profile toward curator profile | taste-shaping (user's product vision) |
| 11 | `external` | HTTP adapter to Python sidecars (LightFM/implicit/VW/RecBole recipes) | exotic models, off-box training |

## The core idea

Everything is config-gated. `REC_ENGINE_MODE=legacy|pluggable` switches the
personal recs handler between the old engine and the strategy registry +
RRF ranker. `REC_STRATEGIES=cooccur:0.4,embeddings:0.3,mf:0.2,bandit:0.1`
sets which strategies are active and their blend weights (RRF, not raw
weighted sum). `REC_SHADOW_MODE=true` runs strategies WITHOUT serving them —
the eval loop.

## Recommended rollout (safe, measurable)

### Step 1 — Shadow mode (zero risk)
Set `REC_ENGINE_MODE=pluggable`, `REC_SHADOW_MODE=true`,
`REC_STRATEGIES=cooccur:1`. Everything serves exactly the legacy output
(golden test proves it), but every strategy you add to the list ALSO scores
and records `rec_training_runs` + `rec_impressions` (engagement tracking).
No user-visible change.

### Step 2 — Turn on one strategy at a time
1. `decay` first: `REC_STRATEGIES=cooccur:0.5,decay:0.5`. Compare vs pure
   legacy. The eval loop (below) tells you if it's better.
2. `embeddings`: `REC_STRATEGIES=cooccur:0.4,embeddings:0.3,decay:0.3`.
   Great for cold-start (new works with no bookmark history still get
   semantic recs from embeddings).
3. `mf`: needs ≥`REC_MF_TRAIN_MIN_SIGNALS` (default 500) user-signal rows.
   Enable `rec-mf` cargo feature (adds ndarray dep) when the DB has enough.
4. `hybrid` once MF + embeddings are both producing: it learns the blend.
5. `sequential` for the reader's "Next Up" — it's a different surface
   (reader page), doesn't compete with the home dashboard.
6. `bandit` LAST, with `REC_BANDIT_SLOTS=1` (1 exploration slot per list):
   it adaptively trades off which strategy fills the last slot.
7. `curator` prior: set `REC_CURATOR_PRIOR=<user id>` to the curator
   account, `REC_PRIOR_FLOOR=0.2`. Cold users (few signals) lean toward the
   curator's taste; established users get their own profile (α decays with
   signal count, τ=25).

### Step 3 — Eval loop (the "only use a bit of it" fix)
- **Offline**: `rec_training_runs.metrics` per strategy (duration, ok).
  `GET /api/recommendations/strategies` (admin) shows enabled + last run.
- **Online A/B**: shadow mode records `rec_impressions` (what was shown) +
  `engaged_at` (bookmark/download/completion after seeing). Compare
  engagement rate per strategy: shown-to-engaged ratio. That's your
  "which strategy actually drives user action" number.
- **Per-user alignment**: `rec_user_curator_align` tracks
  cos(user_centroid, curator_centroid) — see which users the curator prior
  is actually shaping.
- **Promotion**: when a strategy's engagement beats `cooccur`, promote it in
  the weights; when it loses, drop it. This is the whole point of the
  registry — you can ship strategies dark, measure, and flip weights
  without redeploying (env change + restart).

## Concrete feature ideas (what to DO with it)

1. **Home "Recommended for you"** — switch to `pluggable` with
   `cooccur+decay+embeddings` once shadow data supports it. This is the
   flagship surface; the golden test guarantees no worse-than-legacy.
2. **"People who bookmarked X also…"** — `cooccur` already powers
   also-bookmarked; `author_graph` + `tag_graph` add "also by these
   authors / with these tags" panels on fic pages (diversification).
3. **Reader "Next Up"** — `sequential` (Markov) replaces the naive
   "next chapter" fallback; feed the reader page.
4. **Cold-start UX** — for logged-out/new users, `embeddings` + curator
   prior give good recs with zero personal history (currently they get
   popularity — a big upgrade).
5. **Taste profiles / "Because you bookmarked"** — `clusters` assigns users
   to taste clusters; use it for "More from this cluster" rows and to
   explain recs ("People in your cluster loved this").
6. **Controversial/curated picks** — curator prior + `bandit` exploration
   means the home dashboard can show "Curator's pick" (already wired) with
   real taste-shaping behind it.
7. **Recommendation explainability** — every `ScoredRec` carries a
   `strategy` + `reason`; surface "Recommended because of: your
   bookmarks / tags in common / this author" — trust + feedback loop.
8. **Niche/trope discovery** — `tag_graph` meta-paths power "Fics with this
   pairing you haven't read" — the AO3-parity niche searches the user
   values.
9. **A/B the weights** — with `REC_STRATEGIES` weights as env, run
   week-long A/B (weight set A vs B), read engagement from
   `rec_impressions`, keep the winner. No code change.
10. **Python sidecars** — `rec-engines/` has LightFM/implicit/RecBole
    recipes + Dockerfile; when the Rust ALS (`mf`) hits scale limits, run
    LightFM off-box via `REC_EXTERNAL_URL` and let `external` strategy
    call it.
11. **Fic Requests candidates** — `GET /api/requests/{id}/candidates`
    already calls the engine on the request's seed work (dd64fe5); the
    board answers itself with engine suggestions. Promote strategies and
    the suggestions get better automatically.
12. **Fic-level moods** (roadmap P8#96) — classify each work's tone
    (Neutral/Funny/Shocky/Flirty/Dramatic/Hurty/Bondy) from content
    signals (genre ratios, description tone, review sentiment, tag mix).
    Feed mood-similarity into the recommender as a strategy + a search
    facet ("rec me something dramatic"). Adapted from Zeks/flipper's
    mood-adjusted calculator; unlike author-moods, fic-moods describe the
    story itself.
13. **Author recommendations** (roadmap P8#97) — author feature vectors
    (genre mix, mood distribution, fandom spread, popularity band) →
    "authors like X" / "authors who wrote fic you kudos'd". Add an
    `author_graph` strategy (works by similar authors) + "Authors you
    might like" panel. Today we recommend fics only; this opens the
    author surface.
14. **Rarity-tier weighted strategy** (roadmap P8#98) — port Zeks/flipper's
    weighted calculator: per-author overlap ratio + matches + sigma, then
    rarity-tier weights (unique 0.2×matches, rare 0.05×, uncommon 0.005×,
    common 1). Explainable recs ("you share 5 faves with this author whose
    list is 90% rare fic") + powers list-manipulation detection.
15. **Audience-genre inference** (roadmap P8#99) — compute each fic's
    genre profile *of its kudos/bookmark graph* ("funny *to
    humor-lovers*"), one SQL, no ML. Powers "recommend me funny fic" +
    per-fic "audience tastes" + genre facets.
16. **Explorer / size / popularity bands** (roadmap P8#100) — popularity
    band (barely-known/relatively-unknown/popular) + size class
    (small ≤20k/medium ≤100k/big ≤400k/huge) as search + rec filters;
    "obscure gems" exploration, short-completed fic, etc.

## Knobs cheat-sheet

```
REC_ENGINE_MODE=legacy|pluggable     # master switch
REC_STRATEGIES=cooccur:0.4,...       # active strategies + RRF weights
REC_SHADOW_MODE=true|false           # evaluate without serving
REC_EMBED_MODEL=nomic-embed-text     # embeddings model
REC_MF_FACTORS / REC_MF_ITERS        # ALS dims/iters
REC_MF_TRAIN_MIN_SIGNALS=500         # min signals to train MF
REC_BANDIT_SLOTS=1                   # exploration slots per list
REC_TRAIN_EVERY_H=6                  # batch training cadence
REC_CURATOR_PRIOR=<user id>          # curator account for prior
REC_PRIOR_FLOOR=0.2 / REC_CURATOR_TAU=25  # prior blend curve
REC_STRATEGY_TIMEOUT_SECS=10         # per-strategy timeout (fail-fast)
REC_EXTERNAL_URL=                    # optional sidecar HTTP adapter
```

## Next step (recommended first action)

Enable shadow mode today:
`REC_ENGINE_MODE=pluggable REC_SHADOW_MODE=true REC_STRATEGIES=cooccur:1`
+ restart. Golden test still passes (served output unchanged), and from
then on every strategy you add to the list starts accumulating
`rec_impressions` evidence. After 1-2 weeks, read the engagement table and
promote the winner.
