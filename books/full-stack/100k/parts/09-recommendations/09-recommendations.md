# Part 9 — The Recommendation Platform

> **Part 9 of 13** — In Part 8, the community told us what it wants: it
> voted on fic requests, ranked features, followed authors. Every one of
> those interactions left a trace — a bookmark, a rating, a download, a
> follow. By the end of Part 8 we had a platform full of *signals* and
> no machine to turn them into *suggestions*.
>
> This part builds that machine. We start with the strategy registry in
> `src/recommender/strategy.rs` and `registry.rs` — the `RecStrategy`
> trait that turns every recommendation algorithm into a swappable
> plug-in (Chapter 38). Then we wrap the engine that has powered
> FicHub's "readers also bookmarked" since the beginning as the first
> strategy, and pin it down with a golden test that proves the refactor
> changed nothing (Chapter 39). Next we meet the rest of the stable:
> time decay, embedding similarity, matrix factorization, a hybrid
> blender, a tag knowledge graph, taste clusters, and a Thompson-
> sampling bandit (Chapter 40). We learn how FicHub A/B-tests
> strategies safely by logging impressions in shadow mode before any
> user ever sees them (Chapter 41). And finally we wire up the personal
> recommendation feed that turns every reader's bookmarks and downloads
> into their next favorite story (Chapter 42).
>
> By the end of this part, the question "what should I read next?" is
> answered by a pipeline, not a guess — and every answer carries a
> machine-readable *reason* you can show to the user.

---

## Chapter 38 — The Strategy Registry

Every recommendation system you've ever seen has the same architecture
underneath, no matter how fancy the marketing: there is a *scoring
function* (or a dozen of them), and there is a *ranking step* that
decides what the user actually sees. The scoring functions are the
algorithms — "people who bookmarked this also bookmarked that", "this
fic's summary is semantically close to your history", "readers who
liked these tags tend to like these other tags". The ranking step is
the product — it decides which of those signals win, how many slots
each gets, and what the user sees first.

FicHub's pluggable recommendation platform draws a hard line between
the two. The line lives in `src/recommender/strategy.rs`, and it is
the most important design decision in this whole part:

```rust
//! A [`RecStrategy`] turns a [`StrategyContext`] into a ranked list of
//! [`ScoredRec`]s for either a seed work (item-to-item) or a signed-in user
//! (personalized). Strategies are pure scorers: they never decide what gets
//! shown. The [`crate::recommender::registry::StrategyRegistry`] decides which
//! strategies are enabled (config-gated) and the
//! [`crate::recommender::ranker`] blends their outputs with Reciprocal Rank
//! Fusion, applies the curator prior, and slots in bandit exploration.
```

Read that module doc carefully, because it is the entire roadmap for
this part compressed into four lines. **Strategies are pure scorers:
they never decide what gets shown.** That sentence is what makes
everything else possible. A strategy doesn't know about the other
strategies, doesn't know how many slots it gets, doesn't know about
the curator's taste, doesn't know about exploration. It just answers
one question: *given this context, how much would this reader (or this
seed work) like each candidate work?* Everything downstream — blending,
curation, exploration, truncation — is someone else's job.

💡 **Key Concept — a trait is a contract with the pipeline.** When you
design a plug-in system, the trait definition IS the interface between
the algorithm authors and the platform authors. `RecStrategy` promises
the platform: "give me a context and a target, I'll give you a ranked
list or an honest error." The platform promises the strategies: "you'll
never be called with data you can't use, and if you fail I'll degrade
gracefully instead of breaking the page." Get that contract right and
you can add a tenth strategy without touching a single line of the
ranker.

### 38.1 The three shared types

Before the trait itself, the file defines the three types every
strategy speaks in. First, the output — one scored recommendation:

```rust
/// A single scored recommendation produced by a strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoredRec {
    /// url_id of the recommended work (fic_info.id).
    pub work_id: String,
    /// Raw strategy score (before RRF blending).
    pub score: f64,
    /// Strategy name that produced this rec (e.g. `legacy_cooccur`).
    pub strategy: String,
    /// Human-readable reason, surfaced in the API for QA ("because you
    /// bookmarked 3 fics in this fandom", "co-bookmarked by 12 readers", …).
    pub reason: String,
}
```

Four fields. The `reason` field is the one most recommendation systems
skip, and it's the one that makes FicHub feel *honest*. Every rec the
user sees can answer "why did you show me this?" — "tag overlap with
your bookmarks", "co-bookmarked by other readers", "similar content
(embeddings)", "exploration — trying something new". The API surfaces
it, the QA team checks it, and the frontend can render it. When you
ship a recommendation system, the "why" is a feature, not a footnote.

The `strategy` field is how the platform knows who produced what. You
saw in Part 1 how `tracing` spans keep attribution through async code;
here attribution is a plain string on every record, because the ranker
needs it later to build the diagnostics you'll see in Chapter 41.

Second, the input — everything a strategy might need to do its job:

```rust
/// Context handed to every strategy invocation.
///
/// Cheap to clone (pools/clients are `Arc`-like); strategies build it once
/// per request via [`StrategyContext::new`].
#[derive(Debug, Clone)]
pub struct StrategyContext {
    pub db: PgPool,
    pub config: Arc<Config>,
    /// Shared HTTP client (scrapers, external sidecars, Ollama).
    pub http_client: reqwest::Client,
    /// Ollama embeddings client (nomic-embed-text). The embeddings strategy
    /// is the only consumer, but the context carries it so strategies don't
    /// construct their own clients.
    pub ollama: OllamaClient,
    /// Wall-clock "now" for decay math. Frozen per request so all strategies
    /// agree on time.
    pub now: DateTime<Utc>,
}
```

One `PgPool`, the config, an HTTP client, the Ollama client, and a
frozen timestamp. Two details are worth slowing down for.

The **frozen `now`** is a subtle but genuinely important idea. The
decay strategy (Chapter 40) computes `exp(-Δt / half_life)` where `Δt`
is the time since a co-occurrence was recorded. If each strategy
called `Utc::now()` itself, two strategies running a millisecond apart
would disagree about "now" — not by much, but decay math is the kind of
thing where a consistent frame of reference makes tests reproducible
and results explainable. Freeze time once per request, hand it out,
and every strategy agrees on what "now" means. This is the same
philosophy you saw with the frozen `now` in the roadmap Elo code in
Part 8, and the `started` timestamps in the training pipeline: **time
is an input, not an ambient global.**

The **Ollama client in the context** is a dependency-injection
decision. Only the embeddings strategy uses it — but rather than let
that strategy construct its own client (and its own connection pool,
and its own failure modes), the context carries one shared client. You
saw this pattern in Part 6 with `AppState`: inject the shared
resource, don't let each consumer build its own. The comment says it
plainly: "the context carries it so strategies don't construct their
own clients."

Third, the error type. Strategies fail — the DB is slow, Ollama is
down, a seed work has no co-occurrence data. The question is what
*failure means*:

```rust
/// Errors that can abort a strategy run. Strategies are expected to degrade
/// gracefully: the ranker treats an `Err` as "skip this strategy and fall
/// through to the next" (the fallback chain), never as a hard failure of the
/// whole recommendation request.
#[derive(Debug, thiserror::Error)]
pub enum RecError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("strategy error: {0}")]
    Strategy(String),
    #[error("external sidecar error: {0}")]
    External(String),
    #[error("not enough data: {0}")]
    NotEnoughData(String),
    #[error("work {0} not found")]
    NotFound(String),
}
```

This is a *typed failure taxonomy* — each variant says *why* the
strategy failed, and `NotEnoughData` is the most important one. A
strategy that says "not enough data" isn't broken; it's just honest
about having nothing to say. A brand-new user with two bookmarks has
no MF model trained on them — `mf` will return `NotEnoughData` and the
pipeline moves on. Compare this with an `Err`-as-crash philosophy:
one flaky strategy would take down the whole recommendations page.
Instead, `NotEnoughData` is treated as a *soft* skip, logged
distinctly (you saw the fallback-chain semantics in the doc comment),
and the rest of the pipeline continues. The `#[from] sqlx::Error` on
the `Database` variant is the thiserror pattern you met in Part 2's
`AppError` — a `?` on any sqlx call auto-converts into `RecError`.

⚠️ **Watch Out — an error type's *variants* are a design decision, not
a formality.** Notice what `RecError` does NOT have: a `Timeout`
variant for slow strategies. If a strategy's HTTP call can hang
forever, that's handled at the call site with `tokio::time::timeout`
(you'll see it in the embeddings and external strategies in Chapter
40), not by adding an error variant that might get ignored. Every
variant here has a distinct *handling* downstream: `Database` propagates
as a real problem, `NotEnoughData` is a soft skip, `NotFound` is a
404. When you design your own error enums, ask "who catches this, and
what do they do differently for each variant?" If the answer is
"nothing", the variant is decoration.

### 38.2 The trait itself

Now the contract — what it means to *be* a recommendation strategy:

```rust
/// A recommendation strategy.
///
/// Implementations MUST be cheap to construct and hold no per-request state;
/// the registry keeps one instance per strategy. `score` is called with a
/// `user_id` for personalized requests or `None` for item-to-item requests
/// (seed work in `ctx`-independent params).
#[async_trait]
pub trait RecStrategy: Send + Sync {
    /// Canonical strategy name (registry key, `rec_*` log/training key).
    fn name(&self) -> &str;

    /// Score candidate works.
    ///
    /// * `user_id: Some(id)` — personalized: score from the user's signals.
    /// * `user_id: None` — item-to-item: `seed` must be set and scoring is
    ///   relative to the seed work.
    ///
    /// Return an ordered (descending score) list, or `Err` to be skipped by
    /// the ranker's fallback chain.
    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError>;

    /// Run this strategy's training/precompute batch step (nightly/hourly
    /// pipeline). The default is a no-op; strategies that need precomputed
    /// artifacts (embeddings, MF factors, clusters, transitions, bandit
    /// decay) override it. Returns a JSON-ish metrics map for
    /// `rec_training_runs`.
    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        Ok(serde_json::json!({ "note": "no-op" }))
    }

    /// Whether this strategy serves personalized requests. Default true;
    /// item-to-item-only strategies (e.g. `sequential`) override.
    fn supports_personal(&self) -> bool {
        true
    }
}
```

Let's read the contract like a platform engineer, because the
constraints are the point.

**"Implementations MUST be cheap to construct and hold no per-request
state; the registry keeps one instance per strategy."** This is the
`Send + Sync` requirement made concrete. The registry is shared across
every request (it lives in `AppState`), so a strategy instance may be
called concurrently by hundreds of requests at once. That means no
`RefCell`, no interior mutation, no per-request scratch state — every
strategy is a stateless scorer, and any state it needs lives in the
database or in the context. This is exactly the discipline you learned
in Part 2 with handlers: shared state must be `Sync`-safe. The
strategies obey it by being structs with no fields at all (or one
immutable field, like the external strategy's `base_url`).

**The `score` signature encodes the two modes.** `user_id: Some(id)`
means "personalize for this user"; `user_id: None` with `seed:
Some(...)` means "score relative to this seed work". A strategy can
support both (like `cooccur` and `embeddings`), one (like `sequential`,
which is seed-only and says so via `supports_personal() -> false`), or
neither for a given request — in which case it returns
`NotEnoughData` and the pipeline skips it. The `Option` types are the
type system doing the work of expressing "this strategy can't answer
this kind of question" *before* the ranker has to guess.

**`train` defaults to a no-op with a JSON metrics payload.** Some
strategies are pure SQL — they compute their scores live from existing
tables and have nothing to precompute (`decay`, `tag_graph`). Others
need batch artifacts: embeddings must call Ollama for every new work,
MF must fit factors, clusters must run k-means, the bandit must
reconcile impressions. Those override `train`. The default
implementation returns a JSON note so the training pipeline (Chapter
41) can record "this strategy has nothing to train" in
`rec_training_runs` instead of special-casing.

The `#[async_trait]` attribute is the async-trait crate pattern you
saw in Part 4's `SiteFetcher` — it lets the trait have `async fn`
methods while remaining object-safe, so the registry can store
`Arc<dyn RecStrategy>` and call `score(...).await` through a trait
object.

### 38.3 The registry: config as the single source of truth

Now the registry in `src/recommender/registry.rs`. Its job: read which
strategies are enabled from the environment, resolve names to
instances, and expose the enabled list to the ranker. The module doc
again sets the tone:

```rust
//! Config-driven strategy registry.
//!
//! Enabled strategies come from `REC_STRATEGIES` — a comma-separated list of
//! `name:weight` pairs, e.g. `cooccur:0.4,embeddings:0.3,mf:0.2,bandit:0.1`.
//! The default is `["cooccur"]` (the legacy engine wrapped as a strategy),
//! which combined with `REC_ENGINE_MODE=legacy` (default) preserves today's
//! behavior exactly.
```

Two configuration knobs, one story. `REC_ENGINE_MODE` decides *which
pipeline* runs — `legacy` (the exact code path that shipped before
this platform existed) or `pluggable` (registry + ranker). `REC_STRATEGIES`
decides *which strategies* the pluggable pipeline blends. You saw both
parsed in `src/config.rs` in Part 3; let's look at the mode enum since
it's small and complete:

```rust
pub enum RecEngineMode {
    Legacy,
    Pluggable,
}

impl RecEngineMode {
    pub fn from_env() -> Self {
        match std::env::var("REC_ENGINE_MODE").as_deref() {
            Ok("pluggable") => RecEngineMode::Pluggable,
            _ => RecEngineMode::Legacy,
        }
    }

    pub fn is_pluggable(&self) -> bool {
        matches!(self, RecEngineMode::Pluggable)
    }
}
```

The `_ => RecEngineMode::Legacy` catch-all is a deliberate safety
choice: an unset or typo'd `REC_ENGINE_MODE` falls back to the mode
that is *known to work*, not the mode that was just written. Defaults
should be the boring, safe option — the platform's golden test
(Chapter 39) exists precisely because "legacy" must mean
"byte-identical to what shipped."

The heart of the registry is the parsing of `REC_STRATEGIES`:

```rust
/// Parse `REC_STRATEGIES` ("a:0.5,b:0.3" or plain "a,b"; missing weight → 1.0).
/// Unknown names survive parsing; they are dropped at resolution time.
pub fn parse_strategy_specs(raw: &str) -> Vec<StrategySpec> {
    raw.split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .filter_map(|part| {
            let (name, weight) = match part.split_once(':') {
                Some((n, w)) => (n.trim().to_string(), w.trim().parse::<f64>().unwrap_or(1.0)),
                None => (part.to_string(), 1.0),
            };
            if name.is_empty() {
                None
            } else {
                Some(StrategySpec {
                    name,
                    weight: if weight.is_finite() && weight > 0.0 { weight } else { 1.0 },
                })
            }
        })
        .collect()
}
```

This function is a masterclass in *defensive parsing* — it's a pure
function, unit-tested in the same file, and every garbage input has a
defined output:

- `"a:0.5,b:0.3"` → two specs with those weights;
- `"a,b"` → two specs, weight 1.0 each (missing weight defaults);
- `"a:notanumber"` → one spec with weight 1.0 (`unwrap_or(1.0)` on the
  parse);
- `"mf:-2"` → weight clamps to 1.0 (`weight > 0.0` check — you can't
  *disable* a strategy by setting a negative weight, which would be a
  confusing foot-gun);
- `""` or `",,,"` → empty vec.

A config parser that can't crash on bad input is a config parser you
can trust with `unwrap_or` and defaults everywhere. Notice the
philosophy: **never let a config typo take the site down.** Weights
are advisory anyway (the ranker normalizes them), so clamping to a
sane value is always safer than erroring.

The registry struct itself is deliberately thin:

```rust
/// The config-driven registry. Built once at server start from `Config`.
pub struct StrategyRegistry {
    /// Enabled strategy specs, in config order.
    specs: Vec<StrategySpec>,
    /// Resolved strategy instances keyed by name.
    strategies: HashMap<String, Arc<dyn RecStrategy>>,
}
```

And its constructor does the resolution — with the crucial *unknown
name* handling:

```rust
    pub fn new(
        available: Vec<Arc<dyn RecStrategy>>,
        raw_specs: &str,
    ) -> Self {
        let parsed = parse_strategy_specs(raw_specs);
        let mut by_name: HashMap<String, Arc<dyn RecStrategy>> = HashMap::new();
        for s in available {
            by_name.insert(s.name().to_string(), s);
        }

        let mut specs = Vec::new();
        for spec in parsed {
            if by_name.contains_key(&spec.name) {
                specs.push(spec);
            } else {
                warn!("rec strategy '{0}' in REC_STRATEGIES is unknown — skipping", spec.name);
            }
        }
        // Fall back to the legacy co-occurrence strategy when nothing valid
        // was configured (or the config was empty).
        if specs.is_empty() {
            if let Some(_legacy) = by_name.get("cooccur") {
                warn!("REC_STRATEGIES resolved to nothing — falling back to ['cooccur']");
                specs.push(StrategySpec {
                    name: "cooccur".into(),
                    weight: 1.0,
                });
            }
        }

        Self { specs, strategies: by_name }
    }
```

Trace the failure paths: `REC_STRATEGIES=embeddings:0.5,nope:0.1` with
only the legacy strategy registered → `nope` is dropped with a
warning, `embeddings` is also unknown so it's dropped, and the spec
list is empty → **fall back to `["cooccur"]`**, the one strategy
guaranteed to exist. That is the degrade-don't-crash principle applied
to configuration. A typo'd strategy name means you get the safe
default, with a warning in the logs explaining exactly why.

The rest of the registry is accessors: `specs()` for the ranker,
`get(name)` for looking up an instance, `all()` for the admin
strategies listing, `enabled_names()` for logs, and `total_weight()`
for normalizing RRF weights. There's also a custom `Debug` impl that
prints the enabled names instead of dumping the trait objects:

```rust
impl std::fmt::Debug for StrategyRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrategyRegistry")
            .field("specs", &self.specs)
            .field("strategies", &self.enabled_names())
            .finish()
    }
}
```

Because `Arc<dyn RecStrategy>` has no useful `Debug` output, the impl
replaces it with the readable list. When you hold shared trait objects
in a struct you'll log, write a `Debug` impl that tells you what you'd
actually want to see in a log line.

### 38.4 Wiring the registry into the server

The registry is built once at startup, in `src/server.rs`, right next
to the legacy engine it will eventually replace:

```rust
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    // Pluggable recommendation platform: config-driven strategy registry.
    // In legacy mode the registry exists but the handlers route around it
    // (REC_ENGINE_MODE=legacy → byte-identical behavior).
    let strategy_registry = crate::recommender::registry::StrategyRegistry::new(
        crate::recommender::available_strategies(&config),
        &config.rec_strategies,
    );
    tracing::info!(
        "rec engine mode: {:?}, strategies: {}",
        config.rec_engine_mode,
        strategy_registry.enabled_names().join(",")
    );
```

The comment is another explicit promise: **in legacy mode the registry
exists but the handlers route around it.** The registry is *always*
built — even in legacy mode — so that `GET /api/recommendations/strategies`
(Chapter 41) can list strategies and the training pipeline has
something to run. The platform is staged: schema first (inert,
migration 027), registry always present, behavior unchanged until the
operator flips `REC_ENGINE_MODE=pluggable`.

`available_strategies` lives in `src/recommender/mod.rs` and assembles
the full stable:

```rust
/// Assemble the full set of available strategies for the registry.
/// `external` is included only when `REC_EXTERNAL_URL` is set.
pub fn available_strategies(
    config: &crate::config::Config,
) -> Vec<std::sync::Arc<dyn RecStrategy>> {
    let mut strategies: Vec<std::sync::Arc<dyn RecStrategy>> = vec![
        std::sync::Arc::new(legacy_cooccur::LegacyCooccurStrategy::new()),
        std::sync::Arc::new(decay::DecayCooccurStrategy::new()),
        std::sync::Arc::new(embeddings::EmbeddingsStrategy::new()),
        std::sync::Arc::new(hybrid::HybridStrategy::new()),
        std::sync::Arc::new(author_graph::AuthorGraphStrategy::new()),
        std::sync::Arc::new(tag_graph::TagGraphStrategy::new()),
        std::sync::Arc::new(sequential::SequentialStrategy::new()),
        std::sync::Arc::new(clusters::ClustersStrategy::new()),
        std::sync::Arc::new(bandit::BanditStrategy::new()),
    ];
    #[cfg(feature = "rec-mf")]
    strategies.push(std::sync::Arc::new(mf::MfStrategy::new()));
    if config.rec_external_url.is_some() {
        strategies.push(std::sync::Arc::new(external::ExternalStrategy::new()));
    }
    strategies
}
```

Two gated entries worth noting before Chapter 40: `mf` is only
compiled when the `rec-mf` Cargo feature is on (the heavy linear
algebra stays out of default builds — and when it's off, the *stub*
strategy still registers so configs mentioning `mf` don't crash), and
`external` only joins when `REC_EXTERNAL_URL` is set, because there's
no point registering a strategy that can never serve. Everything else
is always available — availability and enablement are separate axes.
**Available means "compiled and registered". Enabled means "listed in
REC_STRATEGIES".** The registry's whole job is to keep those two lists
straight.

### 38.5 The tests that pin the parsing

The registry's unit tests live in the same file — the pattern you've
seen all book long: pure functions get pure tests. A couple of the
best:

```rust
    #[test]
    fn parses_weighted_list() {
        let specs = parse_strategy_specs("cooccur:0.4,embeddings:0.3,mf:0.2,bandit:0.1");
        assert_eq!(specs.len(), 4);
        assert_eq!(specs[0], StrategySpec { name: "cooccur".into(), weight: 0.4 });
        assert_eq!(specs[3], StrategySpec { name: "bandit".into(), weight: 0.1 });
    }
```

```rust
    #[test]
    fn drops_unknown_names_and_falls_back_to_cooccur() {
        let legacy = super::super::legacy_cooccur::LegacyCooccurStrategy::new();
        let available: Vec<Arc<dyn RecStrategy>> = vec![Arc::new(legacy)];
        let reg = StrategyRegistry::new(available, "embeddings:0.5,cooccur:0.5,nope:0.1");
        assert_eq!(reg.enabled_names(), vec!["cooccur"]);
        assert_eq!(reg.specs()[0].weight, 0.5);

        // Empty config → fallback to ['cooccur'].
        let legacy = super::super::legacy_cooccur::LegacyCooccurStrategy::new();
        let available: Vec<Arc<dyn RecStrategy>> = vec![Arc::new(legacy)];
        let reg = StrategyRegistry::new(available, "");
        assert_eq!(reg.enabled_names(), vec!["cooccur"]);
    }
```

The second test is the contract we care about most: a config that
resolves to nothing must *never* leave the platform with zero
strategies. `cooccur` is the floor. Every future strategy can be
disabled; the legacy engine can't.

🧪 **Try It Yourself — run the registry tests.** The registry tests
are pure unit tests — no database needed — so they run with a plain
`cargo test`:

```bash
cd /personal/documents/code/rust/fichub
cargo test --lib recommender::registry 2>&1 | tail -20
```

You should see the four parsing tests pass. Now try breaking the
contract on purpose: change the fallback block in `registry.rs` so
that an empty spec list leaves `specs` empty instead of falling back
to `cooccur`, and re-run. `drops_unknown_names_and_falls_back_to_cooccur`
fails — that's the test doing its job: it pins the *degradation*
behavior, not just the happy path. Tests that only pass when things
work are easy; the tests that matter are the ones that fail when
things *break in the specific way you promised they wouldn't*.

⚠️ **Watch Out — the `Arc<dyn RecStrategy>` box means every strategy
is behind one allocation and one vtable call, and that's fine.** When
you first see `Arc<dyn RecStrategy>` you might worry about performance
— dynamic dispatch on every `score()` call. But `score()` is an
asynchronous database-backed function that takes milliseconds; a
vtable lookup costs nanoseconds. The flexibility of swapping
strategies from *config* — without recompiling, without touching the
ranker — is worth infinitely more than the dispatch cost. Optimize the
queries, not the trait objects. (This is the same lesson as Part 4's
`ScraperRegistry`: trait objects are for *extension points*, and the
extension point is hot enough to matter far less than what it calls.)

💡 **Key Concept — separation of concerns is what makes A/B testing
possible.** Because strategies are pure scorers and the registry is
config-driven, you can serve one user `cooccur` and another user
`cooccur,embeddings` by changing *one environment variable*. You can
run a new strategy at 5% weight without a code deploy. You can turn a
broken strategy off in seconds. None of this would be true if
strategies were if-chains inside the handler. The registry's whole
design — trait, config, defaults, degradation — is in service of one
operational capability: *change the ranking without changing the code*.

### 38.6 Where we are

We have a trait (`RecStrategy`), a context (`StrategyContext`), an
error taxonomy (`RecError`), a config parser, a registry with a safe
fallback, and a startup wiring that builds it unconditionally. What we
don't have yet is a single strategy that actually *runs* — the ranker
that blends them, the strategies themselves, and the endpoints that
serve them.

That changes in the next chapter, with the most important strategy of
all: the legacy co-occurrence engine that has been powering FicHub's
recommendations since before this platform existed — wrapped as
`cooccur`, and pinned down by a golden test that proves the wrap is
invisible.
---

## Chapter 39 — The Legacy Co-Occurrence Engine

Here's the situation every big refactor faces: the existing code works,
users depend on it, and you'd like to build something better *without*
breaking what's already there. The recommendation platform's answer to
this problem is a strategy called `cooccur` — the entire legacy
engine, wrapped so it can sit in the registry next to strategies that
didn't exist six months ago. And the refactor's safety net is a
**golden test**: an integration test that seeds a user, runs the old
path and the new path against the same data, and asserts they produce
identical output.

Before we look at the wrap, let's understand the thing being wrapped.
The legacy engine lives in `src/recommender/engine.rs` — you've met
it before in Part 6 when we served `GET /api/recommendations`, but now
we read it as a *scoring algorithm*, not a route.

💡 **Key Concept — a golden test is a refactoring safety net.** When
you refactor working code, the scariest failure mode isn't "it
crashes" (you'd notice that), it's "it quietly produces different
results" (you might not). A golden test captures the current output
for a fixed input and asserts the new code produces exactly that
output. It doesn't tell you the output is *good* — it tells you the
output is *unchanged*, which is exactly the property you need while
refactoring. Golden tests are how you make "don't break anything" a
machine-checked promise instead of a hope.

### 39.1 The engine, read as an algorithm

The engine's public face is `RecommendationEngine` with one method
everyone calls:

```rust
    /// Return recommendations for the given query.
    ///
    /// 1. Tries the precomputed cache first.
    /// 2. Falls back to live computation (co-occurrence + optional tag fallback).
    pub async fn get_recommendations(
        &self,
        query: &RecQuery,
        config: &Config,
    ) -> Result<Vec<RecResult>, AppError> {
        let limit = query.n.min(config.rec_max_recommendations);
        if limit == 0 {
            return Ok(Vec::new());
        }

        // 1. Check precomputed cache
        let cached = check_cache(&self.db, query, config, limit).await?;
        if !cached.is_empty() {
            return Ok(cached);
        }

        // 2. Compute live
        compute_live(&self.db, query, config, limit).await
    }
```

Cache first, compute live second — the same two-tier pattern as the
export pipeline in Part 5. The `precomputed_recommendations` table is
filled by `compute_and_cache` (called by the collection worker when it
refreshes a work's favourites), and `check_cache` reads it with a TTL
from config. For our purposes, the interesting part is `compute_live`
— the actual recommendation algorithm. Let's walk it, because the
`cooccur` strategy wraps it *function-for-function*.

**Step 1: seed metadata.** The engine needs the seed work's
favouriter count, title, and author:

```rust
    let seed = sqlx::query_as::<_, (i32, String, String)>(
        r#"SELECT COALESCE(fw.favouriter_count, 0),
                  COALESCE(fi.title, ''),
                  COALESCE(fi.author, '')
           FROM fic_works fw
           JOIN fic_info fi ON fi.id = fw.url_id
           WHERE fw.url_id = $1"#,
    )
    .bind(&query.url_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Work {} not found", query.url_id)))?;

    let (favouriter_count, seed_title, seed_author) = seed;
```

The `favouriter_count` matters because it decides *which algorithm
runs* — collaborative filtering if the work is popular enough, tag
fallback if not. That decision is the heart of the engine's design.

**Step 2: co-occurrence candidates.** This is the famous "readers also
bookmarked" query — and the purest expression of the collaborative
filtering idea in the whole codebase:

```rust
    let cooccur: Vec<CooccurRow> = sqlx::query_as::<_, CooccurRow>(
        r#"WITH seed AS (SELECT favouriter_count FROM fic_works WHERE url_id = $1),
            candidates AS (
              SELECT CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
                     cooccur_count
              FROM fic_bookmark_cooccur
              WHERE work_a = $1 OR work_b = $1
            )
            SELECT c.candidate_id, c.cooccur_count, fw.favouriter_count,
              (c.cooccur_count::float
                 / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)
              ) AS jaccard
            FROM candidates c
            JOIN fic_works fw ON fw.url_id = c.candidate_id
            CROSS JOIN seed s
            WHERE c.candidate_id != $1
            ORDER BY jaccard DESC
            LIMIT $2"#,
    )
    .bind(&query.url_id)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;
```

Let's decode this carefully, because it is the conceptual heart of the
whole platform.

`fic_bookmark_cooccur` is the table the collection worker builds: one
row per *pair* of works that were bookmarked by the same user,
`(work_a, work_b, cooccur_count)`. The `CHECK (work_a < work_b)`
constraint you saw in migration 001 keeps each pair stored once, in a
canonical order — no duplicates, no "A-B and B-A" confusion. When the
worker scrapes a user's favourites, it takes every pair in that list
and increments the pair's count (you saw this pipeline in Part 4's
`worker.rs`, from `collect_favouriters` to `compute_and_cache`).

The query does three things:

1. `candidates` picks the *other* work from each pair involving the
   seed — `CASE WHEN work_a = $1 THEN work_b ELSE work_a END` — so a
   candidate list of every work co-bookmarked with the seed, with its
   co-occurrence count.
2. The Jaccard score is computed *in the query*:
   `cooccur_count / (favouriter_count_seed + favouriter_count_candidate - cooccur_count)`.
   That's the actual set formula for overlap of two bookmarker sets:
   the number of readers who bookmarked both, divided by the union of
   the readers of either. Jaccard similarity is the same `|A ∩ B| /
   |A ∪ B|` formula you saw in Part 8's roadmap clustering — it's the
   workhorse of "how similar are two things" everywhere in this
   codebase.
3. `ORDER BY jaccard DESC LIMIT $2` — take the most similar works.

⚠️ **Watch Out — the Jaccard denominator double-counts nothing
*because* the pairs table is deduplicated, but the formula only
approximates the true union.** `favouriter_count_a + favouriter_count_b
- cooccur_count` equals `|A ∪ B|` only if `cooccur_count` is exactly
`|A ∩ B|`. The collection worker computes `cooccur_count` per pair, so
that holds for a single pair. But the query is careful to compute the
union per candidate, in SQL, instead of pre-storing a similarity —
because precomputed similarities rot (a work's favouriter count
changes as new readers arrive), while the counts it's derived from
stay fresh. **Derive, don't store** — you saw this exact principle in
Part 8 with next-in-series.

**Step 3: the tag fallback.** The collaborative path needs enough
co-bookmarking signal to be meaningful. A brand-new work with 2
favouriters has almost no pairs — its co-occurrence list would be
empty or noise. So the engine switches algorithms by data
availability:

```rust
    // --- Tag fallback when seed has few favouriters ---
    let min_collab = config.rec_min_favouriters_for_collab as i32;
    let use_tag_fallback = favouriter_count < min_collab;

    let tag_candidates = if use_tag_fallback {
        fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
    } else {
        Vec::new()
    };
```

`rec_min_favouriters_for_collab` is the config knob for "how much
collaborative signal is enough" — set it too low and cold-start works
get noise; set it too high and you never use the good data. The tag
fallback itself is simpler and cruder than the tag systems you saw in
Parts 6-8: same-author works score 0.8, title-keyword matches score
0.5. It's a cold-start rescue, not a first-class strategy — which is
exactly why the platform later built `embeddings` and `tag_graph` as
proper strategies (Chapter 40). The legacy engine's fallback is the
minimum viable version.

**Step 4: blend collaborative and tag scores.** The two candidate
lists are merged with a weight that depends on how much collaborative
signal exists:

```rust
    // --- Blend collaborative + tag scores ---
    // weight = min(1.0, favouriter_count / 5.0)
    // final  = collab * weight + tag * (1.0 - weight)
    let weight = (favouriter_count as f64 / 5.0).min(1.0);

    let mut candidates: Vec<CandidateScore> = scored.into_values().collect();
    for c in &mut candidates {
        let collab = c.score;
        let tag = c.tag_score;
        c.score = collab * weight + tag * (1.0 - weight);
    }
```

A work with 5+ favouriters gets 100% collaborative signal; a work with
1 favouriter gets 80% tag signal. The weight is a *linear interpolation*
between two sources of evidence, tuned by a magic-ish threshold (5
favouriters) that is really saying "after about five readers, the
crowd knows more than the tags do." This is the seed of the hybrid
strategy you'll meet in Chapter 40 — the legacy engine already blends;
the platform just made blending a first-class citizen.

**Step 5: community vote boost.** Part 8's suggestion voting doesn't
just sit in the community board — it feeds the engine:

```rust
    let gamma = config.rec_voting_boost_gamma;
    for c in &mut candidates {
        let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
        c.community_score = nv;
        // Boost: score * (1.0 + gamma * ln(1 + max(0, net_votes)))
        let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
        c.score *= boost;
    }
```

The boost formula is worth a moment: `score * (1 + γ·ln(1 + net_votes))`.
The natural log means the first few upvotes matter a lot and the
100th matters little — a *diminishing returns* curve that rewards
community validation without letting vote-counting dominate
similarity. And crucially, it's `max(0, nv)`: negative net votes
(downvoted suggestions) never *penalize* a work in recommendations —
they just don't boost. The public surface stays positive-only, a
philosophy you saw in Part 7 with ratings.

Then: sort, truncate, hydrate metadata from `fic_info`, and filter by
site domain if requested. The engine returns `Vec<RecResult>` — the
same DTO the API has always served.

🧪 **Try It Yourself — trace one recommendation by hand.** Pick a work
in your dev database with a decent favouriter count, then run the
co-occurrence query manually:

```bash
cd /personal/documents/code/rust/fichub
set -a; . ./.env; set +a
psql "$DATABASE_URL" <<'SQL'
WITH seed AS (SELECT favouriter_count FROM fic_works WHERE url_id = '<YOUR_URL_ID>'),
     candidates AS (
       SELECT CASE WHEN work_a = '<YOUR_URL_ID>' THEN work_b ELSE work_a END AS candidate_id,
              cooccur_count
       FROM fic_bookmark_cooccur
       WHERE work_a = '<YOUR_URL_ID>' OR work_b = '<YOUR_URL_ID>'
     )
SELECT c.candidate_id, c.cooccur_count,
       (c.cooccur_count::float /
          (s.favouriter_count + fw.favouriter_count - c.cooccur_count)) AS jaccard
FROM candidates c
JOIN fic_works fw ON fw.url_id = c.candidate_id
CROSS JOIN seed s
WHERE c.candidate_id != '<YOUR_URL_ID>'
ORDER BY jaccard DESC
LIMIT 10;
SQL
```

You'll see the engine's exact candidate list, ranked by Jaccard —
this is *the* number that the whole collaborative recommendation
system is built on. Then look at the top result and ask: "is this
actually similar to the seed?" If yes, the co-occurrence table is
doing its job. If it's nonsense, the pair counts are polluted — a good
investigation to have in your back pocket when your recommendations
look wrong.

### 39.2 The personal path: the same engine, a different question

The engine above answers "similar to *this work*". FicHub also has a
personalized question: "similar to *this user*". The legacy
personalized computation lives in `src/recommender/routes.rs` as
`compute_personal_recommendations` — and it will get its own chapter
(42). For now, the important thing is that *both* legacy computations
are wrapped by the `cooccur` strategy, so the golden test can assert
byte-identical behavior on both paths.

### 39.3 Wrapping the engine as a strategy

Now the wrap itself — `src/recommender/legacy_cooccur.rs`. It's a
struct with no fields (stateless, as the trait contract demands), a
`name()` of `"cooccur"`, and a `score()` that dispatches to the exact
legacy functions:

```rust
#[async_trait]
impl RecStrategy for LegacyCooccurStrategy {
    fn name(&self) -> &str {
        "cooccur"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        match (user_id, seed) {
            // Personalized: the exact same computation the legacy handler
            // runs (tag Jaccard + bookmarks + downloads).
            (Some(uid), _) => {
                let (recs, _based_on, enough) =
                    crate::recommender::routes::compute_personal_recommendations(
                        &ctx.db,
                        uid,
                        &ctx.config,
                    )
                    .await?;
                if !enough {
                    return Err(RecError::NotEnoughData(
                        "fewer than 3 signals — legacy gate".into(),
                    ));
                }
                Ok(recs
                    .into_iter()
                    .map(|r| ScoredRec {
                        work_id: r.url_id,
                        score: r.score,
                        strategy: self.name().into(),
                        reason: "tag overlap with your bookmarks".into(),
                    })
                    .collect())
            }
            // Item-to-item: the legacy co-occurrence engine.
            (None, Some(seed_id)) => {
                let engine = crate::recommender::engine::RecommendationEngine::new(ctx.db.clone());
                let query = crate::recommender::engine::RecQuery {
                    url_id: seed_id.to_string(),
                    n: ctx.config.rec_max_recommendations,
                    site_domain: None,
                };
                let results = engine.get_recommendations(&query, &ctx.config).await?;
                Ok(results
                    .into_iter()
                    .map(|r| ScoredRec {
                        work_id: r.url_id,
                        score: r.score,
                        strategy: self.name().into(),
                        reason: "co-bookmarked by other readers".into(),
                    })
                    .collect())
            }
            (None, None) => Err(RecError::Strategy(
                "legacy_cooccur needs a user or a seed work".into(),
            )),
        }
    }
```

The `match (user_id, seed)` is the strategy contract's two modes made
explicit, and each arm calls *the same function the legacy handlers
call* — not a copy, not a reimplementation. That's the golden-test
guarantee in code form: the strategy is a **thin adapter that reuses
the legacy implementation**, so there is literally nothing new to get
wrong.

Notice the details:

- The personalized arm checks `enough` — the legacy handler's own
  `enough_data` gate — and converts "not enough" into
  `RecError::NotEnoughData`. The ranker will then skip `cooccur` for
  this user and try the next strategy (or return empty). The strategy
  is honest about having nothing to say, and the pipeline treats that
  as a soft skip.
- The item-to-item arm constructs a `RecommendationEngine` per call.
  That's cheap — the struct holds only a `PgPool` clone — and keeps
  the strategy stateless, as required.
- Each arm sets a *human-readable reason*. This is where the
  `reason` field from Chapter 38 starts earning its keep: "co-bookmarked
  by other readers" is exactly what the user should hear for a
  collaborative-filtering rec.

The `train` method is a no-op with an honest note:

```rust
    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        // The legacy engine has no batch step (precomputation happens in the
        // collection worker via compute_and_cache). Nothing to do here.
        Ok(json!({ "note": "legacy engine has no batch training step" }))
    }
```

Right — the legacy engine's "training" *is* the collection worker,
which has been running since Part 4. There's nothing for the pipeline
to add.

💡 **Key Concept — wrapping is better than rewriting.** The instinct
when building a new platform around an old engine is to rewrite the
engine "properly" as part of the move. FicHub does the opposite: the
old engine is wrapped *untouched*, registered as `cooccur`, and made
the default. The golden test then proves the wrap is invisible. Only
*after* that safety net exists do new strategies get built, each
competing with `cooccur` on its own merits. You refactor by wrapping
what works, then improving *beside* it — never by rewriting what works
in the same change that introduces the new architecture.

### 39.4 The golden test

Now the test that makes the refactor safe — `golden_legacy_equals_cooccur_strategy`
in `tests/rec_strategies.rs`. It's `#[ignore]`d and DB-gated, like
every integration test in this repo (you'll get the full harness
tour in Part 13). First, the fixture: a user, four fics, a shared
fandom tag, and three bookmarks:

```rust
/// Seed a user with 3 bookmarks sharing a fandom tag + 1 candidate fic.
async fn seed_golden_fixture(pool: &sqlx::PgPool) -> (i32, [&'static str; 4]) {
    let user = "rectestit_golden_user";
    let user_id = seed_user(pool, user).await;
    let fics = ["rectestit_a", "rectestit_b", "rectestit_c", "rectestit_d"];
    seed_fic(pool, "rectestit_a", "Rec Test Alpha", "Dragon story.", 1000, 1, "complete", 1).await;
    seed_fic(pool, "rectestit_b", "Rec Test Beta", "Another dragon story.", 2000, 2, "complete", 2).await;
    seed_fic(pool, "rectestit_c", "Rec Test Gamma", "More dragons.", 3000, 3, "ongoing", 3).await;
    seed_fic(pool, "rectestit_d", "Rec Test Candidate", "The candidate.", 4000, 4, "ongoing", 1).await;
    let tag_fandom = seed_tag(pool, "RecTestIt Fandom", 1).await;
    let tag_free = seed_tag(pool, "RecTestIt Freeform", 4).await;
    for f in &fics[..3] {
        seed_fic_tag(pool, f, tag_fandom, 5).await;
    }
    seed_fic_tag(pool, "rectestit_d", tag_fandom, 5).await;
    seed_fic_tag(pool, "rectestit_a", tag_free, 5).await;
    for f in &fics[..3] {
        seed_bookmark(pool, user_id, f).await;
    }
    (user_id, fics)
}
```

Three bookmarked fics all tagged with the fandom tag, plus a fourth
candidate sharing that tag. The user has exactly `PERSONAL_RECS_MIN_SIGNAL`
(3) signals — enough to pass the legacy gate, minimal enough that the
test runs fast and the expectations are predictable. Note the `rectestit_`
prefix on every identifier: the repo's test convention is that
database-backed tests own their data completely, seed with `ON
CONFLICT DO NOTHING`, and clean up everything they created. You saw
this in Part 8; it's what lets hundreds of DB tests coexist.

Then the test body — legacy path first, pluggable path second, assert
they match:

```rust
/// The golden test: registry with ONLY cooccur → same recs as the legacy
/// handler's direct computation for the same seeded user.
#[ignore]
#[tokio::test]
async fn golden_legacy_equals_cooccur_strategy() {
    let _guard = db_guard();
    let db = pool().await;
    let user = "rectestit_golden_user";
    let (user_id, fics) = seed_golden_fixture(&db).await;

    // Legacy path: the exact computation the legacy handler runs.
    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let (legacy_recs, _based_on, enough) =
        fichub::recommender::routes::compute_personal_recommendations(&db, user_id, &config).await
            .expect("legacy computation");
    assert!(enough, "seeded user must have enough signals");

    // Pluggable path: registry with ONLY cooccur.
    let registry = fichub::recommender::registry::StrategyRegistry::new(
        vec![std::sync::Arc::new(fichub::recommender::legacy_cooccur::LegacyCooccurStrategy::new())],
        "cooccur",
    );
    let ctx = test_ctx(db.clone(), config.clone());
    let (blended, diagnostics) = fichub::recommender::ranker::blend(
        &ctx,
        &registry,
        None,
        Some(user_id),
        20,
    )
    .await
    .expect("pluggable blend");

    assert_eq!(
        blended.len(),
        legacy_recs.len(),
        "strategy count must match legacy count: legacy={} strategy={}",
        legacy_recs.len(),
        blended.len()
    );
    // Same order, same work ids.
    for (i, (l, b)) in legacy_recs.iter().zip(blended.iter()).enumerate() {
        assert_eq!(
            l.url_id, b.work_id,
            "rec {i} must match: legacy {} vs strategy {}",
            l.url_id, b.work_id
        );
    }
    // Diagnostics: cooccur contributed.
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].contributed, "{diagnostics:?}");

    cleanup(&db, user, &fics, &["RecTestIt Fandom", "RecTestIt Freeform"], &[]).await;
}
```

The assertions are deliberately *not* "compare scores with a float
epsilon". The test compares **count, order, and work ids** — because
those are the properties users actually experience. Two rec lists with
the same works in the same order are, from the user's perspective, the
same list. The scores behind them may differ in representation (the
pluggable path runs recs through RRF, which rescales everything), and
that's *fine* — the guarantee being pinned is behavioral identity, not
numerical identity.

There's also a subtle but important detail: the pluggable path is
exercised through `ranker::blend` with a registry containing *only*
`cooccur`. That's the minimal platform: one strategy, no blending, no
curator, no bandit. The golden test proves that even the *pipeline*
around the strategy — the RRF accumulate, the attribution, the
truncation — is invisible when there's a single strategy. `diagnostics`
has exactly one entry and it says `contributed: true`: the strategy
ran, the pipeline saw its output, and the result is identical to
legacy.

⚠️ **Watch Out — a golden test is only as good as the fixture it
covers.** This fixture exercises the personalized path with a
just-barely-enough user. It does *not* cover the item-to-item
co-occurrence path, the tag-fallback branch, the community-vote boost,
or the `NotEnoughData` gate. A golden test pins what it seeds — if the
refactor broke the tag fallback, this test wouldn't notice (that code
is exercised by other tests in the repo). When you write golden tests,
be explicit about which paths they cover, and add fixtures for the
paths that matter most. Coverage honesty beats coverage theater.

### 39.5 Running the golden test

Because it's DB-gated, the golden test needs the same ritual as every
database test in this repo — load `.env`, use the serializing mutex,
run with one thread:

```bash
cd /personal/documents/code/rust/fichub
set -a; . ./.env; set +a
cargo test --test rec_strategies golden_legacy_equals_cooccur_strategy \
  -- --include-ignored --test-threads=1 2>&1 | tail -15
```

When it passes, you have just verified — mechanically, reproducibly —
that the pluggable platform's `cooccur` strategy is behaviorally
identical to the legacy engine for a seeded user. That's the green
light the whole rest of the platform builds on. Now try the
experiment: change the `reason` string in the personalized arm of
`legacy_cooccur.rs` (say, "tag overlap" → "TAG OVERLAP!!"). The test
still passes — reasons aren't compared. Change the *order* of the
mapped recs (reverse the iterator) and the test fails. That tells you
exactly what the golden test is protecting: the shape of what users
see, not the strings attached to it.

🧪 **Try It Yourself — extend the golden test to the item-to-item
path.** The golden fixture only seeds the personalized path. Add a
second test (or extend the fixture) that seeds a `fic_bookmark_cooccur`
pair for `rectestit_a` and `rectestit_d`, then runs `engine.get_recommendations`
directly versus the `cooccur` strategy with `seed: Some("rectestit_a")`,
asserting the same count + order + work ids. This is exactly the kind
of extension the previous Watch Out called for — and it's a great way
to internalize how the strategy dispatches between its two modes.

### 39.6 What the golden test buys the whole platform

The golden test is the load-bearing wall of this refactor, and it
works by *narrowing the blast radius* of every later change. Think
about what it means for the chapters ahead:

- **Chapter 40** adds eight new strategies. Each one is a new
  `Arc<dyn RecStrategy>` in `available_strategies` — and the golden
  test proves that adding them *doesn't change the default output*,
  because the default config (`REC_STRATEGIES=cooccur`) enables
  nothing else.
- **Chapter 41** adds impressions, shadow mode, and the training
  pipeline. All of those touch `rec_*` tables the legacy path never
  reads — and the golden test proves the legacy path stays untouched.
- **Chapter 42** rewires the personal handler to route through the
  registry in `pluggable` mode. The golden test proves the
  `legacy` mode still produces the old answer.

That's the property that makes a staged rollout safe: **the new
architecture can be half-built, half-shipped, and half-deployed
without anyone noticing** — because `cooccur`-only output is pinned
forever. When you undertake a big refactor of working code, build your
golden test *first*, wrap the old code *second*, and only then start
adding the new stuff. The test is the permission slip for everything
that follows.

### 39.7 Where we are

The `cooccur` strategy is registered, default-enabled, and pinned by a
golden test. The platform now has a working pipeline — one strategy
that provably produces the legacy output.

Now the fun part: the rest of the stable. In the next chapter we meet
eight more strategies — time-decayed co-occurrence, embedding
similarity, matrix factorization, a hybrid blender, a tag knowledge
graph, author graphs, taste clusters, and a Thompson-sampling bandit —
each one a self-contained answer to the same question: *what should
this reader read next?*
---

## Chapter 40 — The Rest of the Stable: Decay, Embeddings, MF, Hybrid, Tag Graph, Bandit, and Clusters

Chapter 39 ended with one strategy and a promise: the `cooccur` strategy
is pinned by a golden test, and now the fun begins. This chapter is the
fun. We meet the rest of the stable — the seven strategies that share
the `RecStrategy` contract with `cooccur` and compete for the same
slots in the ranker.

Here is the roster, straight from `available_strategies` in
`src/recommender/mod.rs`:

| Strategy | File | Question it answers | Modes | Precompute |
|---|---|---|---|---|
| `cooccur` | `legacy_cooccur.rs` | "what do other readers co-bookmark?" | seed + personal | none (collection worker) |
| `decay` | `decay.rs` | "what did readers co-bookmark *recently*?" | seed only | none |
| `embeddings` | `embeddings.rs` | "what reads like this / like me?" | seed + personal | `train` → `rec_embeddings` |
| `mf` | `mf.rs` | "what do similar *tastes* like?" | personal only | `train` → factors file + `rec_models` |
| `hybrid` | `hybrid.rs` | "what does a weighted mix say?" | seed + personal | none (reuses components) |
| `tag_graph` | `tag_graph.rs` | "what do the tags connect to?" | seed only | none |
| `clusters` | `clusters.rs` | "what does your taste *group* like?" | personal only | `train` → `rec_user_clusters` |
| `bandit` | `bandit.rs` | "what's worth exploring?" | both (via ranker slots) | `train` → reconcile + decay arms |
| `author_graph` | `author_graph.rs` | "who else writes what this author's readers like?" | seed only | `train` → `rec_author_graph` |
| `sequential` | `sequential.rs` | "what do readers read *next*?" | seed only | `train` → `rec_transitions` |
| `external` | `external.rs` | "what does my Python sidecar say?" | both | `train` → sidecar call |

Eleven strategies, one trait, zero if-chains in the ranker. Each row is
a different *kind of evidence* that a candidate work is worth reading,
and they disagree with each other all the time — which is the point.
The ranker (Chapter 41) fuses disagreement into a single list.

Before we dive in, three things to notice in the table, because they
encode the platform's design philosophy:

1. **The `train` column is mostly "none".** Six of the eleven
   strategies are pure SQL over tables that already exist. Only the
   strategies that need expensive batch artifacts — embeddings, MF,
   clusters, bandit, author graph, sequential — override `train`. The
   platform doesn't build a training pipeline for its own sake; it
   builds one for the strategies that genuinely need precomputation.
2. **Modes are split by data availability, not by fashion.** Decay and
   tag graph are seed-only because their signal is inherently
   per-work. MF and clusters are personal-only because their signal is
   inherently per-user. `cooccur` and `embeddings` do both. A strategy
   that can't answer a request says `NotEnoughData` and the pipeline
   moves on — you saw that contract in Chapter 38.
3. **The bench is as important as the starters.** `author_graph`,
   `sequential`, and `external` exist and register, but none of them is
   enabled by default. They're the *next* experiments, already
   compiled, already testable, waiting for an operator to add them to
   `REC_STRATEGIES`. Building a strategy doesn't mean deploying it.

### 40.1 `decay` — the legacy engine, but honest about time

The first strategy in the file order is `decay`, and it is the most
direct upgrade to `cooccur` you can imagine. The legacy engine counts
every co-occurrence pair equally, whether it was recorded last week or
two years ago. `decay` says: *time matters*. A pair of works bookmarked
together last week is strong evidence of similarity; a pair last seen
together a year ago is a memory, not a signal.

The module doc gives the scoring formula:

```rust
//! Scoring (per candidate work):
//!   score = Σ_pair  cooccur_count · exp(-Δt / halflife) · log-likelihood
//!
//! where the log-likelihood term is the standard co-occurrence
//! informativeness: how much more often A&B co-occur than expected by chance
//! (pointwise mutual information with a log base, smoothed).
```

Three factors multiplied together. The first, `cooccur_count`, is the
familiar raw pair count from Chapter 39. The second is the new idea —
an exponential time decay. The third is a *different* kind of
informativeness than the Jaccard index: instead of "what fraction of
readers who like one like both?", it asks "how much more often do A and
B co-occur than *chance* would predict?"

The decay weight is a tiny pure function with its own unit test:

```rust
/// Decay half-life in days for the exponential `exp(-ln2 · Δt / half_life_days)`.
/// `REC_DECAY_HALFLIFE_DAYS` (default 30): a 30-day-old co-occurrence counts
/// exactly half as much as a fresh one.
pub fn decay_weight(days_ago: f64, half_life_days: f64) -> f64 {
    if days_ago <= 0.0 {
        return 1.0;
    }
    (-std::f64::consts::LN_2 * days_ago / half_life_days.max(1e-9)).exp()
}
```

This is the classic exponential decay you've seen everywhere from
memory models to TCP retransmission: after one half-life (30 days by
default, configurable via `REC_DECAY_HALFLIFE_DAYS`) the signal is
halved, after two it's a quarter, and it asymptotically approaches zero
without ever going negative. The `max(1e-9)` on the half-life is
defensive — a misconfigured `0` half-life would otherwise divide by
zero and produce NaN scores that silently poison the whole ranking.

💡 **Key Concept — half-life is a tunable, not a constant.** The
30-day default encodes a product decision: "co-bookmarking evidence
stays relevant for about a month." If FicHub's readership turns over
faster (fandom trends move in waves — a pairing explodes for six weeks
then cools), an operator can set `REC_DECAY_HALFLIFE_DAYS=7` and the
engine tracks the wave. Nothing in the strategy's code changes. This is
the config-everything philosophy from Chapter 3 paying off: the
algorithm's *shape* is fixed, its *behavior* is a knob.

The second factor, the log-likelihood ratio, is also a pure function:

```rust
pub fn log_likelihood(cooccur: f64, favouriters_a: f64, favouriters_b: f64, total: f64) -> f64 {
    if cooccur <= 0.0 || favouriters_a <= 0.0 || favouriters_b <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    // Expected co-occurrence under independence.
    let expected = (favouriters_a * favouriters_b) / total;
    if expected <= 0.0 {
        return 0.0;
    }
    let ratio = cooccur / expected;
    // Log2 of the ratio, clamped to a sane range (avoids infinities on
    // tiny expected values).
    ratio.ln() / std::f64::consts::LN_2
}
```

If A has 100 favouriters, B has 100, and there are 1,000 users total,
then *by chance alone* we'd expect `100·100/1000 = 10` users to have
bookmarked both. If 100 actually did, the ratio is 10, and
`log2(10) ≈ 3.32` — the pair is ten times more associated than chance.
If exactly 10 did, the ratio is 1 and the score is 0 — no evidence of
association. The log compresses the scale: a ratio of 100 scores 6.64,
not 100. This is the pointwise mutual information idea from
information theory, and it is the standard answer to the question
"which co-occurrences are *informative* and which are just *popular*?"
A wildly popular work co-occurs with everything; PMI-style scoring
penalizes exactly that.

The strategy's `score` then puts all three factors together in one SQL
query (we saw the shape of it in Chapter 39, now with the math in the
SELECT):

```rust
SELECT c.candidate_id,
       (c.cooccur_count::float8
            * exp(-(ln(2.0) * EXTRACT(EPOCH FROM (NOW() - c.last_updated)) / 86400.0) / $3)
            * (ln((c.cooccur_count::float8 * t.total)
                  / GREATEST(s.favouriter_count * fw.favouriter_count, 1.0))
               / ln(2.0))) AS score
FROM candidates c
CROSS JOIN seed s
CROSS JOIN tot t
JOIN fic_works fw ON fw.url_id = c.candidate_id
WHERE c.candidate_id != $1
ORDER BY score DESC
LIMIT $2
```

Read the middle factor closely: `EXTRACT(EPOCH FROM (NOW() - last_updated)) / 86400.0`
is "days since this pair was last updated", and the `exp(-ln2 · days / half_life)`
is exactly the `decay_weight` function from Rust, expressed in SQL.
Note that `NOW()` here is the database's clock, not the frozen `now`
from the context — a small inconsistency in the codebase, and a good
reminder that the frozen-context pattern from Chapter 38 is the *right*
way to do it, precisely because it makes this kind of thing impossible.

Also notice the `GREATEST(..., 1.0)` guarding the denominator of the
log-likelihood term — the same "never divide by zero, never produce
NaN" discipline as the Rust side. The strategy then returns
`NotEnoughData` when there are no co-occurrence rows for the seed, and
its `train` is the honest no-op we expect from a pure-SQL strategy.

🧪 **Try It Yourself — watch the half-life change the ranking.** In
your dev database, pick a seed work with several co-occurrence pairs
recorded at different times, and run the decay query twice — once with
`$3 = 30` and once with `$3 = 7`:

```bash
cd /personal/documents/code/rust/fichub
set -a; . ./.env; set +a
psql "$DATABASE_URL" <<'SQL'
WITH seed AS (SELECT url_id, favouriter_count FROM fic_works WHERE url_id = '<YOUR_URL_ID>'),
     tot AS (SELECT COALESCE(SUM(favouriter_count), 1)::float8 AS total FROM fic_works),
     candidates AS (
       SELECT CASE WHEN work_a = '<YOUR_URL_ID>' THEN work_b ELSE work_a END AS candidate_id,
              cooccur_count, last_updated
       FROM fic_bookmark_cooccur
       WHERE work_a = '<YOUR_URL_ID>' OR work_b = '<YOUR_URL_ID>'
     )
SELECT c.candidate_id,
       c.cooccur_count,
       (c.cooccur_count::float8
          * exp(-(ln(2.0) * EXTRACT(EPOCH FROM (NOW() - c.last_updated)) / 86400.0) / 7.0)
          * (ln((c.cooccur_count::float8 * t.total)
                / GREATEST(s.favouriter_count * fw.favouriter_count, 1.0)) / ln(2.0))) AS score_7d,
       (c.cooccur_count::float8
          * exp(-(ln(2.0) * EXTRACT(EPOCH FROM (NOW() - c.last_updated)) / 86400.0) / 30.0)
          * (ln((c.cooccur_count::float8 * t.total)
                / GREATEST(s.favouriter_count * fw.favouriter_count, 1.0)) / ln(2.0))) AS score_30d
FROM candidates c
CROSS JOIN seed s
CROSS JOIN tot t
JOIN fic_works fw ON fw.url_id = c.candidate_id
WHERE c.candidate_id != '<YOUR_URL_ID>'
ORDER BY score_7d DESC
LIMIT 10;
SQL
```

Compare the two orderings. A pair updated yesterday that outranks
everything at half-life 7 probably shouldn't — it's the freshest
evidence you have. That's the whole argument for `decay`.

### 40.2 `embeddings` — content similarity via pgvector

The next strategy is the one that finally uses the pgvector extension
you've had installed since Part 3. `embeddings` embeds each work's text
(title + summary + top tags) with Ollama's `nomic-embed-text` model,
stores the vectors in `rec_embeddings`, and scores candidates by cosine
similarity.

The text that gets embedded is built by a small pure function:

```rust
/// Text used to embed a work: title + summary + top tags by score.
/// The content_hash gates re-embedding — same text → same hash → skipped.
pub fn build_embed_text(
    title: &str,
    description: &str,
    top_tags: &[String],
) -> String {
    let mut parts = Vec::with_capacity(2 + top_tags.len());
    parts.push(title.trim().to_string());
    if !description.trim().is_empty() {
        parts.push(description.trim().to_string());
    }
    for t in top_tags.iter().take(10) {
        parts.push(t.clone());
    }
    parts.join(" | ")
}
```

Title, summary, and up to ten top tags, joined with a separator. Note
the deliberate decisions: the description is optional (some works have
none), tags are capped at ten, and each part is trimmed. Embedding
models are sensitive to input distribution — a summary that's 40KB of
HTML artifacts would produce a garbage vector — so the strategy
controls exactly what enters the model.

The interesting engineering problem is *when to embed*. Calling Ollama
for every work on every training run would be slow and wasteful. The
answer is a content hash gate — the same pattern you used for
deduplicating scrapes in Part 4:

```rust
/// Cheap content hash (FNV-1a over the embed text). Not cryptographic —
/// only used to gate re-embedding.
pub fn content_hash(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}
```

FNV-1a is a 30-line classic — not cryptographic, but perfectly adequate
for "did this text change?" The gate works like this: `works_needing_embedding`
joins `fic_info` against `rec_embeddings`, finds works with no
embedding row, builds their embed text, hashes it, and skips any work
whose stored `content_hash` matches. Same text → same hash → no
re-embedding. The work's summary gets edited → new hash → the work is
queued for a fresh embedding on the next training run.

The item-to-item scoring is one query — the `<=>` operator is
pgvector's cosine-distance operator, and `1 - distance` is cosine
similarity:

```rust
let rows: Vec<(String, f64)> = sqlx::query_as(
    r#"
    SELECT e2.work_id,
           1.0 - (e1.embedding <=> e2.embedding) AS cosine
    FROM rec_embeddings e1
    JOIN rec_embeddings e2 ON e2.model = e1.model AND e2.work_id != e1.work_id
    WHERE e1.work_id = $1 AND e1.model = $2
    ORDER BY cosine DESC
    LIMIT $3
    "#,
)
.bind(seed_id)
.bind(model)
.bind(limit)
.fetch_all(&ctx.db)
.await?;
```

The `e2.model = e1.model` self-join is subtle and important: `rec_embeddings`
keys on `(work_id, model)`, so the table can hold vectors from different
models side by side. Comparing across models would be meaningless — a
384-dim nomic vector and a 768-dim all-MiniLM vector live in different
spaces. The query pins both sides to the same model.

The personalized path is where embeddings get genuinely clever. Instead
of "find works near this seed", it builds a **recency-weighted
centroid** of everything the user has signalled — the average of their
bookmarked and downloaded works' vectors, weighted so recent signals
count more — and then runs an approximate-nearest-neighbour search for
works near that centroid:

```rust
let profile = user_profile_centroid(ctx, uid).await?;
let rows: Vec<(String, f64)> = sqlx::query_as(
    r#"
    SELECT e.work_id, 1.0 - (e.embedding <=> $1::vector) AS cosine
    FROM rec_embeddings e
    WHERE e.model = $2
    ORDER BY e.embedding <=> $1::vector
    LIMIT $3
    "#,
)
.bind(profile.0.clone())
.bind(model)
.bind(limit * 3)
.fetch_all(&ctx.db)
.await?;
```

"Tell me what your history looks like as one point in vector space,
then show me what lives nearby." That's the whole of content-based
personalization in one sentence. Two details are worth slowing down
for. First, the `LIMIT $3` is `limit * 3` — the strategy fetches three
times as many neighbours as it needs, because it filters out works the
user already knows (`profile.1` is the set of known work ids) before
truncating. Second, the centroid itself is normalized to unit length
before the ANN query — cosine distance is insensitive to vector
magnitude anyway, but a normalized query vector keeps the `<=>` scores
interpretable as actual cosines in [0, 1].

The `train` method is the strategy's heart — a batch loop over works
needing embedding, with a per-item timeout so one stuck Ollama call
never blocks the batch:

```rust
for (work_id, text) in batch {
    // Per-item timeout so a stuck Ollama call never blocks the batch.
    let call = ctx.ollama.embed(&text);
    let result = tokio::time::timeout(ctx.call_timeout(), call).await;
    match result {
        Ok(Ok(vec)) => {
            if vec.len() != ctx.config.rec_embed_dim {
                skipped += 1;
                continue;
            }
            // Vector literal for pgvector: '[1,2,3]'
            let lit = format!(
                "[{}]",
                vec.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            sqlx::query(
                r#"INSERT INTO rec_embeddings (work_id, model, embedding, content_hash, updated_at)
                   VALUES ($1, $2, $3::vector, $4, NOW())
                   ON CONFLICT (work_id, model) DO UPDATE SET
                     embedding = EXCLUDED.embedding,
                     content_hash = EXCLUDED.content_hash,
                     updated_at = NOW()"#,
            )
            .bind(&work_id)
            .bind(&model)
            .bind(lit)
            .bind(content_hash(&text))
            .execute(&ctx.db)
            .await?;
            embedded += 1;
        }
        _ => {
            skipped += 1;
        }
    }
}
```

Three defensive layers in one loop: a per-item timeout, a dimension
check (`vec.len() != rec_embed_dim` — the model returned something
unexpected, treat it as a skip), and an upsert so re-embedding an
edited work overwrites its old vector atomically. Every failure path
increments `skipped` and moves on; the batch never fails. The metrics
returned — `{embedded, skipped, model}` — flow into `rec_training_runs`
where you can watch the skip rate over time (a rising skip rate is
often the first sign that Ollama is degrading).

⚠️ **Watch Out — a vector database is a database, with the same
staleness problems.** `rec_embeddings` has an HNSW index for fast ANN
search (you saw the index in migration 027), and HNSW is brilliant at
"give me the 60 nearest neighbours fast". But the vectors inside it are
snapshots: a work's embedding is only as fresh as the last training run
that touched it. New works have no embedding until `train` runs; edited
works keep their old vector until the content hash changes and the next
batch re-embeds them. If you serve embeddings and never run the
training pipeline, the strategy quietly serves a frozen view of the
corpus. The content-hash gate makes staleness *visible* (the `skipped`
metric drops to zero because nothing needs embedding), which is the
first step to fixing it — but only the nightly pipeline actually fixes
it. Every strategy with a precompute step has this shape: *the
artifacts are the product, and the training pipeline is the janitor.*

### 40.3 `mf` — implicit matrix factorization by hand

The `mf` strategy is the most "machine learning" thing in this
codebase, and it's also the most stubbornly dependency-free: hand-rolled
implicit ALS — the Hu–Koren formulation that powers "people who liked
this also liked" at every scale from a notebook to a shopping site — in
about 150 lines, no linear-algebra crate. The module doc is upfront
about the trade:

```rust
//! Implementation: hand-rolled ALS over a sparse (user, work, confidence)
//! matrix built from `rec_user_signals`. No ndarray — dependency-light, ~150
//! lines. Factors are persisted to disk (params_path) and registered in
//! `rec_models`; serving is a dot product.
```

The idea: instead of describing users by *which works* they interacted
with (sparse, noisy, huge), learn a small set of *latent factors* — say
8 numbers — for each user and each work, such that a user's
preference for a work is approximated by the dot product of their
factor vectors. Two users with similar factor vectors have similar
tastes; two works with similar factor vectors are similar. The factors
are learned by alternating least squares: hold the work factors fixed
and solve for the user factors, then hold the user factors fixed and
solve for the work factors, repeat.

The implicit-feedback twist: with bookmarks and downloads you never
observe "the user rated this 4/5". You observe *actions*, and the
standard trick is to convert each action into a confidence value:

```rust
/// One implicit-feedback observation: confidence = 1 + α·r_ui.
#[derive(Debug, Clone)]
pub struct MfObservation {
    pub user: usize,
    pub work: usize,
    pub confidence: f64,
}
```

`confidence = 1 + α·r_ui`, where `r_ui` is the strength of the signal
(bookmarks weigh 4.0, downloads 2.0, ratings their value — you'll see
the full table in Chapter 42) and `α = 40` in the training code. The
`+1` is the implicit-feedback trick: every user has a baseline
confidence of 1 in *every* work (they might like it), and each action
adds `α·r_ui` on top. The matrix is stored sparsely — only the
observed cells:

```rust
/// Sparse implicit-confidence matrix.
#[derive(Debug, Clone)]
pub struct MfMatrix {
    pub n_users: usize,
    pub n_works: usize,
    /// user → (work, confidence)
    pub user_items: Vec<Vec<(usize, f64)>>,
    /// work → (user, confidence)
    pub item_users: Vec<Vec<(usize, f64)>>,
}
```

The ALS loop itself is the densest math in the book, so let's read it
as a *shape*, not as linear algebra. The core of one user step:

```rust
// G_u = G + Σ_i (c_ui - 1) y_i y_iᵀ + λI
let mut a = gram.clone();
for (i, c) in items {
    let y = &item_factors[*i];
    for r in 0..factors {
        for c2 in 0..factors {
            a[r][c2] += (c - 1.0) * y[r] * y[c2];
        }
    }
}
for d in 0..factors {
    a[d][d] += lambda;
}
// b = Σ_i c_ui y_i
let mut b = vec![0.0f64; factors];
for (i, c) in items {
    let y = &item_factors[*i];
    for r in 0..factors {
        b[r] += c * y[r];
    }
}
// Solve a·x = b (Gaussian elimination with partial pivoting).
if let Some(x) = solve_linear(&a, &b) {
    user_factors[u] = x;
}
```

The `gram` matrix is `YᵀY` — the item-factor Gram matrix, precomputed
once per iteration. The trick labeled "the 'all items minus the user's'
trick" in the module doc is what makes implicit ALS tractable: rather
than looping over *all* items for every user (quadratic in the corpus),
the per-user adjustment adds only the *observed* items' contribution
`(c_ui - 1) y_i y_iᵀ` on top of the shared Gram matrix, and the `λI`
ridge term keeps the solve from blowing up on users with few signals.
The solve itself is Gaussian elimination with partial pivoting, and the
`None` return on a singular matrix is handled gracefully — the previous
factors stay put rather than the whole training run crashing.

The serving side is almost anticlimactically simple:

```rust
let mut scored: Vec<(String, f64)> = item_factors
    .iter()
    .zip(work_ids.iter())
    .map(|(item, wid)| (wid.clone(), dot(uf, item)))
    .collect();
scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
scored.truncate(ctx.config.rec_max_recommendations);
```

Load the factors, take the user's factor vector, dot it with every
work's factor vector, sort. That's it — the entire expensive learning
happened at `train` time; serving is one dot product per work. This is
the canonical ML serving pattern: **train offline, serve with a
multiplication.**

Two engineering decisions in `mf` deserve a close look, because they're
the kind of thing that separates a demo from a system.

First, the **feature gate**. `mf` is the only strategy that doesn't
compile by default — it's behind the `rec-mf` Cargo feature:

```rust
#[cfg(not(feature = "rec-mf"))]
#[async_trait]
impl RecStrategy for MfStrategy {
    fn name(&self) -> &str {
        "mf"
    }

    async fn score(
        &self,
        _ctx: &StrategyContext,
        _seed: Option<&str>,
        _user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        Err(RecError::NotEnoughData(
            "MF strategy disabled — build with --features rec-mf".into(),
        ))
    }
    ...
}
```

When the feature is off, the strategy *still registers* — it just
returns `NotEnoughData` with a helpful message. Remember the registry's
fallback logic from Chapter 38: a config that mentions `mf` without the
feature enabled would otherwise produce "unknown strategy" warnings and
fall back to `cooccur`. The stub keeps `mf` resolvable, so
`REC_STRATEGIES=cooccur:0.4,mf:0.2` works on every build — the MF
strategy simply contributes nothing until you compile it in. This is
availability-vs-enablement (Chapter 38) taken to its logical end:
*available* is a compile-time property, *enabled* is a config-time
property, and they're allowed to disagree without breaking anything.

Second, the **model registry**. Training writes factors to a JSON file
on disk (timestamped in the cache dir) and registers the run in
`rec_models`:

```rust
let path = format!(
    "{}/rec-mf-{}.json",
    ctx.config.cache_dir.display(),
    ctx.now.format("%Y%m%d-%H%M%S")
);
...
save_factors(&path, &uf, &wf, &work_ids, &user_ids, metrics.clone())
    .map_err(|e| RecError::Strategy(format!("save factors: {e}")))?;
sqlx::query(
    r#"INSERT INTO rec_models (name, version, params_path, trained_at, metrics)
       VALUES ('mf-als', '1', $1, NOW(), $2)
       ON CONFLICT (name, version) DO UPDATE SET
         params_path = EXCLUDED.params_path,
         trained_at = NOW(),
         metrics = EXCLUDED.metrics"#,
)
.bind(&path)
.bind(metrics.clone())
.execute(&ctx.db)
.await?;
```

Every training run leaves an audit trail: which factors file, trained
when, with what metrics. Serving loads the *latest* model, so a bad
training run can be rolled back by pointing `params_path` at the
previous file — or by deleting the row so the next `train` regenerates
it. The file-on-disk-plus-registry-in-DB split is deliberate: the
database holds the *metadata* (queryable, comparable across runs), the
filesystem holds the *payload* (potentially megabytes of floats that
don't belong in rows).

🧪 **Try It Yourself — build with the feature and watch the test
recover structure.** The MF module's unit tests run with a plain
`cargo test` regardless of the feature — the math is pure. But to see
the full strategy, build with the feature on:

```bash
cd /personal/documents/code/rust/fichub
cargo test --lib recommender::mf 2>&1 | tail -20
cargo build --features rec-mf 2>&1 | tail -3
```

The `als_recovers_block_structure` test seeds a 3×3 matrix where users
0/1 like works 0/1 and user 2 likes work 2, runs ALS, and asserts the
dot products respect the structure — liked pairs score higher than
non-liked. It's the *entire promise* of matrix factorization compressed
into 15 lines: from sparse observations, learn factors that let you
predict the unobserved cells. If you want to feel how the algorithm
works, change `iterations` from 20 to 2 in the test and watch the
assertions fail — two passes isn't enough for the structure to emerge.
Iterations are a quality knob, and more isn't always better: too many
iterations on too little data overfits the few signals you have.

### 40.4 `hybrid` — a blender that is itself a strategy

Here's the design tension the `hybrid` strategy exists to resolve:
each strategy above answers a *different* question, and all of them are
right some of the time. MF captures collaborative taste but can't score
a brand-new work (cold start). Embeddings capture content similarity
but ignore what the crowd thinks. The hybrid combines three component
signals — MF scores, embedding cosine, and tag Jaccard — into a single
weighted blend:

```rust
/// Default hybrid weights: (mf, embedding, tag).
pub const DEFAULT_HYBRID_WEIGHTS: (f64, f64, f64) = (0.4, 0.4, 0.2);
```

The module doc has an important warning about double-counting: if you
enable `hybrid` *and* its components individually in `REC_STRATEGIES`,
the ranker's RRF will include the hybrid's output alongside the
components' — the same works voted on twice by overlapping evidence.
The recommended config is `hybrid` **or** the components, not both.

The score function is a study in graceful degradation — each component
is fetched through a helper that swallows its own failure:

```rust
// Component 1: MF scores (requires trained factors).
let mf_scores = mf_component_scores(ctx, user_id).await;
// Component 2: embedding cosine vs user profile / seed.
let emb_scores = embedding_component_scores(ctx, seed, user_id).await;
// Component 3: tag Jaccard vs user's top tags / seed tags.
let tag_scores = tag_component_scores(ctx, seed, user_id).await;

if mf_scores.is_empty() && emb_scores.is_empty() && tag_scores.is_empty() {
    return Err(RecError::NotEnoughData(
        "hybrid has no component signals".into(),
    ));
}
```

Each component helper literally instantiates the component strategy and
calls its `score` — `mf_component_scores` builds an `MfStrategy`, calls
it, and maps `Err(_)` to an empty vec. That's the trait contract paying
off in the most direct way possible: *strategies reuse strategies*. If
MF isn't trained, its component contributes nothing and the hybrid
blends embeddings + tags. If the user has no embeddings, it blends MF +
tags. The hybrid is never *broken* by a missing component — it just has
less to say.

The blending math is deliberately simple: min-max normalize each
component into [0, 1], multiply by its weight, and sum. The normalize
function is the kind of pure, testable utility that makes the whole
strategy trustworthy:

```rust
/// Min-max normalize a scored list into [0,1]; empty stays empty.
pub fn normalize(scores: &[(String, f64)]) -> Vec<(String, f64)> {
    if scores.is_empty() {
        return Vec::new();
    }
    let max = scores.iter().map(|(_, s)| *s).fold(f64::NEG_INFINITY, f64::max);
    let min = scores.iter().map(|(_, s)| *s).fold(f64::INFINITY, f64::min);
    let range = max - min;
    if range <= 1e-12 {
        return scores.iter().map(|(w, _)| (w.clone(), 1.0)).collect();
    }
    scores
        .iter()
        .map(|(w, s)| (w.clone(), (s - min) / range))
        .collect()
}
```

Why normalize at all? Because the three components score on wildly
different scales: MF dot products can be anything, embedding cosines
live in [0, 1], tag Jaccards are counts. Blending raw scores would mean
"the component with the biggest numbers wins", which is exactly the
arbitrary weighting you don't want. Min-max normalization makes each
component's *shape* — its ordering and relative gaps — comparable, and
the weights decide the importance. The flat-list special case (`range
<= 1e-12` → all scores 1.0) is a nice touch: a component where every
candidate scored identically carries no ranking information, and
treating all as 1.0 means the other components decide.

💡 **Key Concept — blending is where you encode product judgment.**
The default weights `(0.4, 0.4, 0.2)` are a product decision wearing a
math costume: "collaborative taste and content similarity matter
equally; tags matter half as much." An operator who believes FicHub's
users care more about fandom-fit than prose style can set
`REC_HYBRID_WEIGHTS=0.5,0.1,0.4` in the environment — no recompile, no
deploy. And the `train` method's metrics note ("static weights; learning
pass is a placeholder") is an honest roadmap: the infrastructure to
*learn* weights from `rec_training_runs` exists in the same module doc
that describes it. Start static, measure, then automate the tuning.
That's the same staged-rollout philosophy as the whole platform.

### 40.5 `tag_graph` — walking the tag knowledge graph

The `tag_graph` strategy treats tags as a *graph* and recommends by
walking it. Works connect to tags (via `fic_tags`), tags connect to
works; two works are related if there's a short path between them. The
module doc describes the walk:

```rust
//! Scoring a seed work:
//!   1. Collect the seed's tags with their scores.
//!   2. Walk meta-paths of length ≤ 2 through the tag graph:
//!      seed-tag → shared-work-tag → candidate-work-tag.
//!   3. Score each candidate by the product of tag scores along the path,
//!      summed over all paths (a soft random-walk with restart flavor).
```

"Soft random-walk with restart" is a mouthful; the code is friendlier.
`meta_path_score` is a pure function over three maps — seed tags,
tag→works incidence, and work→tags incidence:

```rust
for (seed_tag, seed_score) in seed_tags {
    // Direct neighbours: works sharing this tag.
    if let Some(neighbours) = tag_works.get(seed_tag) {
        for (work, work_tag_score) in neighbours {
            if work == seed_work {
                continue;
            }
            let s = seed_score * work_tag_score;
            let entry = scores.entry(work.clone()).or_insert((0.0, format!("tag {seed_tag}")));
            entry.0 += s;
        }
    }
    // Two-hop: works sharing a tag with a work that shares seed_tag.
    ...
}
```

One hop: a work sharing the seed's fandom tag scores
`seed_score · work_tag_score` — the tag's weight on the seed times its
weight on the candidate. Two hops: walk seed-tag → intermediate work →
*its* tag → candidate work, with a path-length discount of 0.5 applied
to the two-hop product. The scores accumulate over all paths, so a
candidate reachable by many paths (the seed's fandom tag *and* a
character tag) outranks a candidate reachable by one.

The tag graph is loaded fresh per request — a full scan of `fic_tags`
into two HashMaps:

```rust
let rows: Vec<(String, i32, f64)> = sqlx::query_as(
    r#"SELECT ft.url_id, ft.tag_id, ft.score::float8
       FROM fic_tags ft
       WHERE ft.score > 0"#,
)
.fetch_all(&ctx.db)
.await?;
let mut tag_works: HashMap<i32, Vec<(String, f64)>> = HashMap::new();
let mut work_tags: HashMap<String, Vec<(i32, f64)>> = HashMap::new();
for (work, tag, score) in rows {
    tag_works.entry(tag).or_default().push((work.clone(), score));
    work_tags.entry(work).or_default().push((tag, score));
}
```

This is the "cheap at this scale" trade you'll see repeatedly in this
chapter: the table is a few hundred thousand rows, the HashMap build
takes tens of milliseconds, and the per-request cost is far less than
the complexity of maintaining a precomputed graph. The `WHERE score > 0`
filter is deliberate — a tag with score 0 (you saw the score semantics
in Part 6: main char = 10, primary ship = 5) carries no signal, and
skipping it shrinks the graph.

The reason strings are the star here: `format!("tag path via {}", w.via)`.
The `via` field is the *path* the candidate was reached by, which means
every rec from this strategy can tell the user exactly why: "tag path
via tag 42" — which the frontend can turn into "because it shares the
'Angst' tag". That's the `reason` field from Chapter 38 earning its
keep again.

### 40.6 `clusters` — taste groups via k-means

The `clusters` strategy is the social answer to cold start: instead of
"what does *you* look like?", it asks "what does your *group* look
like?" During `train`, it runs k-means over users' tag-affinity
vectors — the same `user_tag_vector` helper the ranker uses, which you
saw in Chapter 38's context — and writes memberships to
`rec_user_clusters`. At serve time, it finds the user's cluster and
recommends works heavily tagged with that cluster's characteristic
tags.

The k-means implementation is pure Rust over sparse vectors:

```rust
pub fn kmeans_step(
    vectors: &[Vec<(i32, f64)>],
    centroids: &[Vec<(i32, f64)>],
) -> Vec<usize> {
    vectors
        .iter()
        .map(|v| {
            let mut best = 0usize;
            let mut best_d = f64::INFINITY;
            for (i, c) in centroids.iter().enumerate() {
                let d = sparse_distance(v, c);
                if d < best_d {
                    best_d = d;
                    best = i;
                }
            }
            best
        })
        .collect()
}
```

k-means in its purest form: assign every vector to its nearest centroid,
recompute the centroids as the mean of their members, repeat until
stable. The `run_kmeans` loop even has the classic convergence check —
stop early when the centroids stop moving:

```rust
for _ in 0..iterations {
    assignments = kmeans_step(vectors, &centroids);
    let new_centroids = recompute_centroids(vectors, &assignments, k);
    if new_centroids == centroids {
        centroids = new_centroids;
        break;
    }
    centroids = new_centroids;
}
```

The cluster count is `k = 5.min(vectors.len() / 2)` and `train` refuses
to run with fewer than 4 users — k-means with more clusters than users
is nonsense, and the strategy says so honestly via `NotEnoughData`.
Memberships are written idempotently: a `DELETE FROM rec_user_clusters`
then an insert, all inside one transaction, so readers never see a
half-rebuilt cluster table. (You'll see this DELETE-and-rebuild-in-a-
transaction pattern again in Chapter 41's training pipeline — it's the
house style for precomputed tables.)

Serving picks the user's highest-affinity cluster, finds that cluster's
most characteristic tags, and queries for works carrying them that the
user hasn't seen:

```rust
let rows: Vec<(String, i64)> = sqlx::query_as(
    r#"SELECT f.id, COUNT(DISTINCT ft.tag_id) AS shared
       FROM fic_info f
       JOIN fic_tags ft ON ft.url_id = f.id
       WHERE ft.tag_id = ANY($1) AND f.id <> ALL($2)
       GROUP BY f.id
       ORDER BY shared DESC
       LIMIT $3"#,
)
.bind(&top_tags)
.bind(&known)
.bind(ctx.config.rec_max_recommendations as i64)
.fetch_all(&ctx.db)
.await?;
```

The `f.id <> ALL($2)` exclusion of the user's known works is the
platform's "don't recommend what they've already got" invariant — you
saw it in the legacy personal engine, and every personalized strategy
reimplements it. And the reason string tells the user *why the group
matters*: "popular in your taste cluster (3 shared tags)". For a new
user with two bookmarks, this strategy can surface the third, fourth,
and fifth works of their fandom long before MF has anything to say
about them.

### 40.7 `bandit` — Thompson sampling, or how to explore on purpose

The `bandit` strategy is the one that looks least like a
recommendation algorithm and most like a slot machine — because it is,
mathematically, a multi-armed bandit. The insight: recommending is
not just about *exploiting* what you know ("this user will probably
like this") but also about *exploring* ("let's try something uncertain
and learn from the response"). Every recommendation system faces the
explore/exploit trade-off; the bandit makes it explicit.

Each (work, strategy) pair is an *arm* with a Beta posterior — a
probability distribution over "how likely is engagement with this
arm?". The default `Beta(1, 1)` is the uniform distribution: total
uncertainty. Every engagement (bookmark, download, completion after an
impression) increments `alpha` — the arm's success count — and every
shown-but-ignored impression increments `beta` — its failure count.
The Thompson sampling trick: sample each arm's posterior, rank by the
samples, show the top ones. Arms with high expected engagement get
shown often (exploit), but arms with *uncertain* posteriors get sampled
high sometimes by luck (explore) — and every showing refines the
posterior.

The sampler is worth reading because it's a masterclass in "good
enough for production" math:

```rust
/// Thompson sample from Beta(alpha, beta) via two Gamma samples
/// (Marsaglia-Tsang). Deterministic-ish given the rng closure (testable).
pub fn beta_sample(alpha: f64, beta: f64, mut rng: impl FnMut() -> f64) -> f64 {
    let g1 = gamma_sample(alpha, &mut rng);
    let g2 = gamma_sample(beta, &mut rng);
    if g1 + g2 <= 1e-12 {
        // Degenerate: both shapes ≤ 0 → uniform (0.5) when both are 0,
        // else the mean of the surviving parameter.
        if alpha + beta <= 1e-12 {
            0.5
        } else {
            alpha / (alpha + beta)
        }
    } else {
        g1 / (g1 + g2)
    }
}
```

A Beta(a, b) sample equals `G1 / (G1 + G2)` where G1 ~ Gamma(a) and G2 ~
Gamma(b) — a standard identity. The gamma sampler uses the
Wilson–Hilferty transform, which the comment defends honestly: it
"never loops, never hangs" — unlike a textbook rejection sampler, which
can in principle spin forever on a bad random stream. For a function
called on every request, "never hangs" beats "slightly more exact".
Note the testability design: the rng is a *parameter*, so the unit
tests pass a fixed `|| 0.42` and get reproducible samples.

Where do the impressions come from? The strategy's `score` method
samples the arms and *logs every shown rec as an impression*:

```rust
if let Some(uid) = user_id {
    for r in &out {
        log_impression(ctx, Some(uid), &r.work_id, self.name()).await;
    }
}
```

And `log_impression` is the simplest possible write:

```rust
pub async fn log_impression(
    ctx: &StrategyContext,
    user_id: Option<i32>,
    work_id: &str,
    strategy: &str,
) {
    let _ = sqlx::query(
        "INSERT INTO rec_impressions (user_id, work_id, strategy) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(work_id)
    .bind(strategy)
    .execute(&ctx.db)
    .await;
}
```

One row per showing: who was shown what, by which strategy, when. That
table is the *measurement instrument* for everything in Chapter 41, so
we'll leave the bandit's `train` — the reconciliation of impressions
into alpha/beta updates — for there too. For now, the key point is the
feedback loop: show → log → engage-or-not → update posterior → sample
again. The bandit gets *better at exploring* the longer it runs,
because it learns which arms deserve more shows.

### 40.8 The bench: `author_graph`, `sequential`, `external`, and the curator

Four strategies didn't make the featured lineup, but each is one config
line away from being live. Read them as future work with the harness
already built.

**`author_graph`** (`author_graph.rs`) lifts co-bookmarking one level
of abstraction: from works to *authors*. `train` rebuilds
`rec_author_graph` from `fic_bookmark_cooccur` plus a tag Jaccard per
author pair:

```rust
// Co-occurrence edges: from fic_bookmark_cooccur (work_a, work_b) →
// (author_a, author_b).
let cooccur_edges: Vec<(String, String, i64)> = sqlx::query_as(
    "SELECT work_a, work_b, cooccur_count FROM fic_bookmark_cooccur",
)
.fetch_all(&ctx.db)
.await?;
```

Serving ranks the seed author's neighbours by `cooccur_count ·
(1 + tag_jaccard)` and surfaces their best works. The idea is
beautifully simple: if readers who bookmark author A also bookmark
author B, and their tag sets overlap, then "readers of A will probably
like B". It's the co-occurrence engine with the granularity turned up
from individual fics to careers.

**`sequential`** (`sequential.rs`) is the Markov chain: it learns
*what comes next*. `train` rebuilds `rec_transitions` from two sources
— sequel links in `series_works` (weight 3.0) and co-download sequences
in `request_log` (same reader, within 6 hours, weight 1.0):

```rust
/// Increment (or insert) a transition edge. Pure function over a map so it
/// is unit-testable; `recency` multiplies the added weight.
pub fn add_transition(
    map: &mut std::collections::HashMap<(String, String), f64>,
    from: &str,
    to: &str,
    weight: f64,
) {
    if from == to || from.is_empty() || to.is_empty() {
        return;
    }
    *map.entry((from.to_string(), to.to_string())).or_insert(0.0) += weight;
}
```

This is the strategy behind the "Next Up" panel — "readers typically
continue with this" — and it declares `supports_personal() -> false`
because a *sequence* is meaningless without a seed. The fallback chain
you saw in Part 6 (sequel → community pick → also-bookmarked) stays
exactly as it was; `sequential` is a *better* first link in that chain,
not a replacement for it.

**`external`** (`external.rs`) is the escape hatch: when
`REC_EXTERNAL_URL` is set, it proxies scoring to a Python sidecar
(recipes live in `rec-engines/` — LightFM, implicit, VW-bandit,
RecBole). The protocol is a tiny JSON contract:

```rust
/// Request payload sent to the sidecar.
#[derive(Debug, serde::Serialize)]
pub struct ExternalRequest {
    pub seed: Option<String>,
    pub user_id: Option<i32>,
    pub n: usize,
}
```

The call is wrapped in `tokio::time::timeout(ctx.call_timeout(), call)` —
the same per-call timeout discipline you saw in `embeddings` — and any
failure degrades to `RecError::External`, which the ranker treats as a
soft skip. FicHub is a Rust shop, but the platform refuses to pretend
the Python ML ecosystem doesn't exist: if the data team wants to
experiment with a LightFM model, they stand up a sidecar, set one env
var, and the strategy joins the blend.

**The curator** (`curator.rs`) isn't a strategy at all — it's a *prior*,
and it's the subject of Chapter 42. Its module doc states the design in
one line:

```rust
//!    final_profile = α·user + (1−α)·curator
//!    α = floor + (1−floor)·exp(−n_signals/τ)
```

A new user with zero signals gets α = 1.0 — their feed is *entirely*
curator picks. An active user with hundreds of signals converges to α =
0.2 — their own taste dominates, but the curator keeps a floor vote.
The math is in the ranker (`curator_alpha`), and we'll spend real time
on it in Chapter 42, because it's the answer to the question "what do
you show a reader who just signed up?"

### 40.9 Reading the whole stable

Step back and look at the roster as a designer would. The eleven
strategies are eleven different *answers* to "what should I read
next?", and they fail in complementary ways:

- `cooccur` fails on cold-start works (no co-bookmarking history).
- `decay` fails the same way, plus it needs fresh pair timestamps.
- `embeddings` fails when Ollama is down or the corpus is unembedded.
- `mf` fails for users outside the training set and works outside the
  matrix.
- `tag_graph` fails for tagless works.
- `clusters` fails for users who don't fit any cluster.
- `bandit` fails until it has arms (impressions to learn from).
- `hybrid` fails only when *everything* fails — and even then it says
  `NotEnoughData` politely.

That complementarity is not an accident; it's the architecture. The
ranker doesn't need any single strategy to be right — it needs the
*combination* to be robust, and a system where each component fails in
a different mode is a system whose failures are also complementary.
When you design your own recommendation stack, list the failure modes
of each component before you list its strengths. A strategy that can't
fail gracefully isn't a strategy — it's a liability.

⚠️ **Watch Out — every strategy carries its own cold-start story, and
the stories are different.** Cold start isn't one problem; it's one
problem per strategy. `embeddings` cold-starts on *works* (no vector
until first training run), `mf` cold-starts on *users* (no factors
until they accumulate signals), `clusters` cold-starts on *corpus size*
(no clusters until ≥4 users), `bandit` cold-starts on *feedback* (no
arms until impressions exist). The platform's answer is the fallback
chain: a user with 2 bookmarks gets `cooccur` (tag overlap), which
always works, and the fancier strategies join as their preconditions
are met. When you read a strategy's `NotEnoughData` messages, you're
reading its cold-start conditions in plain English.

### 40.10 Where we are

The stable is full: eleven strategies, each registered, each unit-tested,
each with an honest `train` and honest failure modes. The registry can
resolve any of them, the golden test still pins `cooccur`-only output,
and an operator can enable any combination with one environment
variable.

What we don't have yet is the machinery that makes *choosing* between
them scientific: the impressions table that records what users were
shown, the training pipeline that keeps the precomputed artifacts
fresh, and the diagnostics that tell you which strategies are pulling
their weight. That's Chapter 41 — where FicHub learns to A/B-test its
own strategies in shadow mode, before a single user ever sees the
result.
---

## Chapter 41 — Impressions, Shadow Mode, and the A/B-Test Pipeline

In Chapter 40 we built eleven strategies, each with a different answer
to "what should this reader read next?" — and immediately hit the
hardest question in recommendation engineering: *how do we know which
answer is best?*

Think about the naive approach for a moment, because it's instructive.
You enable `embeddings`, ship it to everyone, and compare average
engagement before and after. What's wrong with that picture? Everything
— the comparison is contaminated by seasonality (fandom trends move in
waves, Chapter 40 reminded us), by the fact that you can't *unshow*
what you already showed, and by the slow drip of other changes landing
at the same time. Worse: if `embeddings` is actually *worse* than
`cooccur`, you've degraded the experience for every user in the fleet
before you learn the answer. A/B testing is only safe when the
experiment can't hurt anyone — and the way to guarantee that is to run
the experiment *off to the side* first.

FicHub's answer is **shadow mode**: run the new strategies exactly as
if they were live — score, blend, rank — but show users the old
pipeline's output. Log everything the shadow pipeline *would have*
shown as impressions. Then, days or weeks later, look at the
impressions table and ask: "if we had shown these, would users have
engaged?" That's the whole chapter in one paragraph. Let's build it.

### 41.1 The measurement instrument: `rec_impressions`

Every experiment needs a measurement instrument, and the platform's is
the `rec_impressions` table — the table the bandit strategy has been
writing to since Chapter 40:

```sql
-- ── 5. Impressions (what was shown) ────────────────────────────────────────
CREATE TABLE IF NOT EXISTS rec_impressions (
    id         BIGSERIAL PRIMARY KEY,
    user_id    INTEGER,                    -- NULL = anonymous
    work_id    VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    strategy   TEXT NOT NULL,              -- which strategy produced the rec
    shown_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    engaged_at TIMESTAMPTZ                 -- set when the user bookmarks/
                                           -- downloads/completes the work later
);
```

Five columns, and every one is a decision. `user_id` is nullable
because anonymous users get recommendations too (the legacy item-to-item
endpoint serves anyone), and an anonymous impression is still
measurable in aggregate. `strategy` records *which strategy produced
this rec* — the attribution string from Chapter 38, now doing real work.
`shown_at` defaults to now because "when was this shown" is the one
field you never want to forget to set. And `engaged_at` is the
experiment's dependent variable: it starts NULL, and it gets stamped
when the user later bookmarks, downloads, or completes the work. An
impression that never gets an `engaged_at` is, by definition, a miss.

💡 **Key Concept — an impression is the unit of measurement, and it is
a *promise*.** When the ranker logs "user 42 was shown work 123 by
strategy `embeddings` at 14:03", it is making a falsifiable claim: *if
we show this again, the user will probably engage*. The impressions
table is where every strategy's claims accumulate, and the
reconciliation logic in Chapter 40 is where they get judged. Notice
what makes this honest: the log happens *at show time*, before anyone
knows whether the user will engage. You can't retroactively log an
impression you didn't show — that would be manufacturing the experiment
data. If you take one operational habit from this chapter, let it be:
**log what you show, when you show it, with attribution, or you have no
experiment at all.**

The indexes are chosen for the queries you'll actually run:

```sql
CREATE INDEX IF NOT EXISTS idx_rec_impressions_user
    ON rec_impressions (user_id, shown_at DESC);
CREATE INDEX IF NOT EXISTS idx_rec_impressions_work
    ON rec_impressions (work_id);
```

The first serves "what has this user been shown, most recent first" —
the per-user audit trail. The second serves "what was shown that
eventually got bookmarked" — the engagement join. Both are the
workhorses of the reconciliation queries you're about to see.

### 41.2 Shadow mode: `REC_SHADOW_MODE` and the two pipelines

Shadow mode is a single config flag — `REC_SHADOW_MODE`, parsed back
in Part 3's `config.rs` with a default of `false`:

```rust
let rec_shadow_mode = std::env::var("REC_SHADOW_MODE")
    .unwrap_or_else(|_| "false".to_string())
    .parse::<bool>().unwrap_or(false);
```

One boolean, and the whole A/B-testing strategy hangs off it. Here's
the architecture in one sentence: **shadow mode runs the pluggable
pipeline *alongside* the legacy one, serves the legacy output, and logs
the pluggable output as impressions.** The users see the old
recommendations; the impressions table fills with what the new
strategies *would have* recommended; the experiment harvests the data
without anyone experiencing the treatment.

To see how the flag threads through the code, look at where it's
surfaced — the strategies endpoint. `GET /api/recommendations/strategies`
is the admin dashboard for the whole platform, and its response
includes the mode and the flag so an operator can see at a glance what
state the pipeline is in:

```rust
Ok(Json(json!({
    "err": 0,
    "mode": if state.config.rec_engine_mode.is_pluggable() { "pluggable" } else { "legacy" },
    "shadow_mode": state.config.rec_shadow_mode,
    "strategies": strategies,
})))
```

`mode` and `shadow_mode` are *different axes* — that's the key insight.
`REC_ENGINE_MODE` decides which pipeline **serves** (legacy or
pluggable). `REC_SHADOW_MODE` decides whether the non-serving pipeline
**runs and logs**. The four combinations are all meaningful:

| `REC_ENGINE_MODE` | `REC_SHADOW_MODE` | What happens |
|---|---|---|
| `legacy` (default) | `false` (default) | Legacy serves; nothing else runs. Byte-identical to pre-platform behavior — the golden test's world. |
| `legacy` | `true` | Legacy serves; pluggable pipeline runs in shadow, logging impressions. **The safe A/B test.** |
| `pluggable` | `false` | Pluggable serves; legacy path stays available but idle. The staged rollout, step two. |
| `pluggable` | `true` | Pluggable serves *and* logs. Full telemetry in production. |

The dangerous cell is `pluggable` + `false`-shadow — you're serving new
strategies with no impressions logged, which means no measurement. And
the *safe* cell is `legacy` + `true` — the whole point of this chapter.
An operator moves through the cells in order: shadow-test in cell two,
roll out in cell three, add telemetry in cell four. Notice that the
safety comes from the *defaults*: both flags default to the legacy,
non-measuring mode, so an unset environment is always the known-good
state. You saw this default-to-safe philosophy in Chapter 38's
`RecEngineMode::from_env`; `REC_SHADOW_MODE` is the same principle
applied to experimentation.

⚠️ **Watch Out — shadow mode has a cost: it doubles the 
recommendation workload.** Every shadow request runs the full pluggable
pipeline — every enabled strategy scores, the RRF ranker blends, the
bandit samples — and then throws the result away except for the
impression log. That's real CPU and real database load on every request,
for output nobody sees. FicHub pays it because the data is worth more
than the compute (and because the strategies are mostly pure SQL over
indexed tables, so each one is milliseconds). But when you design your
own shadow system, budget for the doubling: shadow mode at 100% of
traffic is a deliberate decision, not a free lunch. Many systems shadow
only a *sample* of requests for exactly this reason — the impressions
from 10% of traffic are usually plenty for a statistically meaningful
comparison.

### 41.3 The bandit's `train`: reconciliation as the measurement payoff

Now we can finally read the bandit's `train` method with full context —
it's the piece that turns raw impressions into *knowledge*. The nightly
training run calls `reconcile_impressions`, and the first half finds
the impressions that *earned* their engagement:

```rust
let engaged: Vec<(i32, String)> = sqlx::query_as(
    r#"SELECT DISTINCT i.user_id, i.work_id
       FROM rec_impressions i
       WHERE i.user_id IS NOT NULL
         AND i.engaged_at IS NULL
         AND EXISTS (
            SELECT 1 FROM bookmarks b
            WHERE b.user_id = i.user_id AND b.url_id = i.work_id
              AND b.created_at > i.shown_at
         )"#,
)
.fetch_all(&ctx.db)
.await?;
```

Read the WHERE clause as a sentence: *impressions where the user was
known, engagement hasn't been recorded yet, and there exists a bookmark
created after the impression was shown.* That last condition — `b.created_at
> i.shown_at` — is the entire causal discipline of the system. The
bookmark has to come *after* the showing, or it doesn't count as
evidence that the recommendation worked. If the user bookmarked the
work three months before the impression, the rec didn't cause the
bookmark; it just repeated the user's existing taste. This is the
temporal ordering requirement that separates real measurement from
correlation theater, and it's a two-line SQL clause. When you build
your own measurement, always ask: *did the outcome happen after the
treatment?* If you can't answer yes, you can't attribute.

For each engaged impression, the arm's `alpha` gets incremented and
the impression is stamped so it's never double-counted:

```rust
for (uid, wid) in &engaged {
    let _ = sqlx::query(
        r#"INSERT INTO rec_bandit_arms (work_id, strategy, alpha, beta, updated_at)
           VALUES ($1, 'bandit', 1.0, 1.0, NOW())
           ON CONFLICT (work_id, strategy) DO UPDATE SET
             alpha = rec_bandit_arms.alpha + 1,
             updated_at = NOW()"#,
    )
    .bind(wid)
    .execute(&ctx.db)
    .await;
    let _ = sqlx::query(
        "UPDATE rec_impressions SET engaged_at = NOW() WHERE user_id = $1 AND work_id = $2 AND engaged_at IS NULL",
    )
    .bind(uid)
    .bind(wid)
    .execute(&ctx.db)
    .await;
    engaged_count += 1;
}
```

The `ON CONFLICT DO UPDATE` handles the cold-start arm: the first time
a work gets an engagement, there's no arm row yet, so the upsert
creates `Beta(1, 1)` and immediately bumps it to `Beta(2, 1)`. The
`engaged_at` stamp makes reconciliation idempotent — run the pipeline
twice and the second run finds nothing to do. Idempotency is a
recurring theme in this chapter's code, and it's not a nicety: the
training pipeline is a *cron job*, and cron jobs get re-run, retried,
and double-fired.

The second half finds the misses — impressions shown at least three
days ago with no engagement:

```rust
let cold: Vec<(i32, String)> = sqlx::query_as(
    r#"SELECT user_id, work_id
       FROM rec_impressions
       WHERE user_id IS NOT NULL AND engaged_at IS NULL
         AND shown_at < NOW() - interval '3 days'"#,
)
.fetch_all(&ctx.db)
.await?;
let mut missed = 0i64;
for (_, wid) in &cold {
    let _ = sqlx::query(
        r#"INSERT INTO rec_bandit_arms (work_id, strategy, alpha, beta, updated_at)
           VALUES ($1, 'bandit', 1.0, 1.0, NOW())
           ON CONFLICT (work_id, strategy) DO UPDATE SET
             beta = rec_bandit_arms.beta + 1,
             updated_at = NOW()"#,
    )
    .bind(wid)
    .execute(&ctx.db)
    .await;
    missed += 1;
}
```

The three-day window is the "give the user a fair chance" constant: a
rec shown yesterday might still be acted on today, but after three days
of no engagement it's a reasonable bet that the user saw it and passed.
Each cold impression increments `beta` — the failure count. Over time,
`alpha` and `beta` accumulate, and the Beta posterior sharpens: an arm
with `Beta(80, 20)` has a tight distribution around 0.8, while an arm
with `Beta(2, 1)` is still wide open. Thompson sampling (Chapter 40)
then does its thing: tight high arms get exploited, wide arms get
explored, and the whole loop *learns which works earn their slots*.

Then the nightly decay keeps the posteriors honest — old evidence
shouldn't weigh as much as new:

```rust
pub async fn decay_arms(ctx: &StrategyContext) -> Result<usize, RecError> {
    let half_life = ctx.config.rec_decay_halflife_days;
    let rows: Vec<(String, f64, f64)> = sqlx::query_as(
        "SELECT work_id, alpha, beta FROM rec_bandit_arms WHERE strategy = 'bandit'",
    )
    .fetch_all(&ctx.db)
    .await?;
    let n = rows.len();
    for (work, alpha, beta) in rows {
        let age_days = 1.0; // decay relative to last update below
        let w = (-age_days / half_life.max(1e-9)).exp();
        // Pull both params toward 1.0: evidence = (param - 1) * w + 1.
        let new_a = 1.0 + (alpha - 1.0) * w;
        let new_b = 1.0 + (beta - 1.0) * w;
        let _ = sqlx::query(
            "UPDATE rec_bandit_arms SET alpha = $1, beta = $2, updated_at = NOW() WHERE work_id = $3 AND strategy = 'bandit'",
        )
        .bind(new_a.max(1.0))
        .bind(new_b.max(1.0))
        .bind(&work)
        .execute(&ctx.db)
        .await;
    }
    Ok(n)
}
```

This is the decay idea from Chapter 40 applied to *evidence counts*
instead of co-occurrence scores: each nightly run pulls `alpha` and
`beta` partway back toward 1.0 (the uniform prior), so an arm's
posterior reflects roughly the last month of feedback rather than all
of history. The `new_a.max(1.0)` clamp keeps the posterior valid — a
Beta parameter below 1 would make the distribution U-shaped, which is
the opposite of "slightly optimistic prior". Notice the comment
`age_days = 1.0` with the shrug "decay relative to last update below" —
the code decays by a fixed one-day factor per nightly run, which is
equivalent to a per-night half-life and keeps the function simple. This
is the kind of "good enough, documented" approximation that real
systems are made of.

💡 **Key Concept — the bandit's `train` is a lesson in causal
hygiene.** Read the three pieces as a unit: *only count outcomes that
happen after the treatment* (the temporal WHERE clause), *only count
each impression once* (the idempotent `engaged_at` stamp), *let old
evidence decay* (the nightly pull toward the prior). Any one of those
missing, and the posteriors silently lie to you. The first piece is
the one people get wrong most often — it's astonishingly easy to build
an "A/B test" that counts pre-existing bookmarks as wins. When you
measure anything, write down the temporal ordering rule *before* you
write the query.

### 41.4 The training pipeline: `run_training_pipeline`

The bandit's `train` is one strategy's step. The pipeline that runs
*every* strategy's `train` on a schedule lives in `src/recommender/worker.rs`
— the same file that holds the collection worker's `SiteFetcher` trait
from Part 4. The two workers share a file because they share a job
description: *keep the recommendation data fresh*.

`run_training_pipeline` is the orchestrator, and its shape should feel
familiar — it's the same "run everything, tolerate individual
failures, record results" pattern as the collection worker:

```rust
pub async fn run_training_pipeline(
    db: &PgPool,
    config: &Config,
    http_client: &Client,
    ollama: &crate::services::ollama::OllamaClient,
    registry: &crate::recommender::registry::StrategyRegistry,
) -> Vec<serde_json::Value> {
    use crate::recommender::registry::build_context;

    let ctx = build_context(
        db.clone(),
        std::sync::Arc::new(config.clone()),
        http_client.clone(),
        ollama.clone(),
    );

    // 1. Refresh the unified signal view (with the curator multiplier).
    if let Err(e) = crate::recommender::signals::refresh_signals(db, config.rec_curator_prior).await
    {
        tracing::warn!("rec signal refresh failed: {e}");
    }
    ...
```

Step one is `refresh_signals` — the materialized signal view you'll
meet properly in Chapter 42, refreshed *first* because every strategy
that trains reads from it. If the signal view is stale, embeddings
embed the wrong profile texts and MF trains on yesterday's bookmarks.
The pipeline's ordering is a dependency chain: signals first, then the
strategies that consume them.

Then the per-strategy loop. Each run is recorded in `rec_training_runs`
*before* it starts — with `ok = FALSE` as the pessimistic default —
and updated when it finishes:

```rust
let run_id: i64 = sqlx::query_scalar(
    r#"INSERT INTO rec_training_runs (strategy, started_at, duration_ms, metrics, ok)
       VALUES ($1, NOW(), 0, '{}'::jsonb, FALSE)
       RETURNING id"#,
)
.bind(&spec.name)
.fetch_one(db)
.await
.unwrap_or(0);

let outcome = match strategy.train(&ctx).await {
    Ok(metrics) => {
        let duration_ms = started.elapsed().as_millis() as i64;
        if run_id > 0 {
            let _ = sqlx::query(
                r#"UPDATE rec_training_runs SET duration_ms = $1, metrics = $2, ok = TRUE
                   WHERE id = $3"#,
            )
            .bind(duration_ms)
            .bind(&metrics)
            .bind(run_id)
            .execute(db)
            .await;
        }
        serde_json::json!({
            "strategy": spec.name,
            "ok": true,
            "metrics": metrics,
            "duration_ms": duration_ms,
        })
    }
    Err(e) => {
        let duration_ms = started.elapsed().as_millis() as i64;
        if run_id > 0 {
            let _ = sqlx::query(
                r#"UPDATE rec_training_runs SET duration_ms = $1,
                       metrics = jsonb_build_object('error', $2), ok = FALSE
                   WHERE id = $3"#,
            )
            .bind(duration_ms)
            .bind(e.to_string())
            .bind(run_id)
            .execute(db)
            .await;
        }
        tracing::warn!("rec strategy {} training failed: {e}", spec.name);
        ...
    }
};
```

The `ok = FALSE`-first pattern is the classic "crash-safe bookkeeping"
move: if the process dies mid-train, the run is *visible* as a failed
run rather than silently absent. A stuck embeddings batch that OOMs the
server leaves a `rec_training_runs` row saying "started, never
finished" — which is exactly the signal an operator needs to notice
"embeddings training has been failing for three days". And the failure
of one strategy never aborts the others: `match` on each `train`
result, log the error, continue. This is the degrade-don't-crash
philosophy from Chapter 38 applied to the batch side.

Where does the pipeline run from? Two callers. The first is the worker
tick, on the `REC_TRAIN_EVERY_H` schedule (default 6 hours). The second
is the admin endpoint we saw in the routes file — `POST
/api/recommendations/train`, which is literally the pipeline behind an
auth check:

```rust
pub async fn train_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let results = crate::recommender::worker::run_training_pipeline(
        &state.db,
        &state.config,
        &state.http_client,
        &state.ollama,
        &state.strategy_registry,
    )
    .await;
    Ok(Json(json!({ "err": 0, "runs": results })))
}
```

One endpoint, every strategy, all of it idempotent and failure-tolerant.
The admin can watch the pipeline run live from the dashboard instead of
ssh-ing into the server and poking the worker. When you build batch
machinery, always expose the same entry point to the scheduler *and* to
a human — the human path is how you debug what the scheduler path did
at 3 a.m.

🧪 **Try It Yourself — run a training cycle and watch the metrics.**
With your dev server up (or just the DB reachable), hit the admin
endpoint:

```bash
cd /personal/documents/code/rust/fichub
curl -s -X POST http://localhost:8080/api/recommendations/train \
  -H "Authorization: Bearer <your_admin_token>" | python3 -m json.tool
```

You'll see one object per enabled strategy: `ok`, `duration_ms`, and
the strategy-specific metrics from Chapter 40 — `{embedded, skipped}`,
`{users, clusters}`, `{engaged, missed, arms_decayed}`, the honest
`{"note": "no-op"}` notes. Now run it again. Notice that embeddings
skips everything (nothing new to embed — the content hash gate did its
job) and the bandit reconciles zero new impressions (nothing new was
shown). The pipeline is *converging to steady state*, and the metrics
tell you so. That's the dashboard working.

### 41.5 The strategies endpoint: the experiment's instrument panel

The `strategies_handler` we keep referring to is worth reading in
full, because it's the *observability surface* of the whole platform —
the one endpoint that tells an operator what's enabled, what's
weighted, and how each strategy's last training run went:

```rust
// Last training run per strategy.
let last_runs: Vec<(String, Option<serde_json::Value>, Option<bool>, Option<i64>)> =
    sqlx::query_as(
        r#"SELECT DISTINCT ON (strategy) strategy, metrics, ok, duration_ms
           FROM rec_training_runs
           ORDER BY strategy, started_at DESC"#,
    )
    .fetch_all(&state.db)
    .await?;
```

`DISTINCT ON (strategy)` with `ORDER BY strategy, started_at DESC` is
the PostgreSQL idiom for "the most recent row per strategy" — a pattern
you'll want in your own toolbelt (it also powers "latest per user" and
"newest per work" queries all over this codebase). The endpoint then
merges the registry's *available* strategies with the *enabled* specs
and the *last-run* data into one row per strategy:

```rust
let strategies: Vec<Value> = state
    .strategy_registry
    .all()
    .into_iter()
    .map(|s| {
        let name = s.name().to_string();
        let (metrics, ok, duration_ms) = run_map.get(&name).cloned().unwrap_or((None, None, None));
        json!({
            "name": name,
            "enabled": enabled.contains_key(&name),
            "weight": enabled.get(&name).copied().unwrap_or(0.0),
            "supports_personal": s.supports_personal(),
            "last_run": {
                "ok": ok,
                "duration_ms": duration_ms,
                "metrics": metrics.unwrap_or(serde_json::json!({})),
            },
        })
    })
    .collect();
```

This is the three-lists problem from Chapter 38 — available, enabled,
last-run — resolved into a single table. The row for `mf` with
`"enabled": false` and `"last_run": {"ok": false, "metrics": {"error":
"MF strategy disabled — build with --features rec-mf"}}` tells the
operator *exactly* why MF isn't contributing, without digging through
logs. The row for `cooccur` with `"enabled": true, "weight": 1.0` and a
no-op note confirms the default is doing its job.

⚠️ **Watch Out — an instrument panel is only as good as the
*attribution* feeding it.** The `strategy` column in `rec_impressions`
comes from the `ScoredRec.strategy` field, which is set by each
strategy's `score` — a plain string, set by hand. If someone adds a
twelfth strategy and forgets to set `strategy: self.name().into()`,
every impression it logs gets attributed to... whatever string it did
set, or an empty one, and the bandit arms for that strategy silently
never update. This is a whole class of bug — *misattribution* — and
the defense is the same as everywhere in this codebase: pin it with a
test. The `name_is_decay`-style unit tests you saw in Chapter 40 are
not decoration; they're the guard that keeps attribution honest. When
you build a measurement system, test the *labels* as carefully as you
test the numbers.

### 41.6 Shadow mode in action: a worked example

Let's walk the full lifecycle of one shadow experiment, because it's
the chapter's thesis in narrative form.

**Day 1 — the setup.** The operator wants to try `embeddings` on the
personal feed. They set:

```bash
export REC_ENGINE_MODE=legacy      # users keep seeing the old recs
export REC_SHADOW_MODE=true        # but the pluggable pipeline runs alongside
export REC_STRATEGIES="cooccur:0.4,embeddings:0.3,mf:0.2,bandit:0.1"
```

The server starts, logs `rec engine mode: Legacy, strategies: cooccur,embeddings,mf,bandit`,
and from that moment every personal request runs the full pluggable
pipeline in the background. Users see the legacy tag-overlap recs —
nothing changes for them. The shadows, though, are logged: every work
the pluggable pipeline *would have* shown gets a `rec_impressions` row
with `strategy = 'embeddings'` or `'mf'` or `'bandit'`.

**Day 3 — the first harvest.** The nightly training run calls the
bandit's `reconcile_impressions`. Some shadow-shown works got
bookmarked by the users they were shown to — `alpha` ticks up. Most
didn't, and after the three-day window they're counted as cold —
`beta` ticks up. The arm table now contains real evidence about which
works earn engagement when shown by which strategy.

**Day 14 — the read.** The operator queries the impressions table
directly:

```sql
SELECT strategy,
       COUNT(*) AS shown,
       COUNT(engaged_at) AS engaged,
       ROUND(COUNT(engaged_at)::numeric / NULLIF(COUNT(*), 0), 4) AS engagement_rate
FROM rec_impressions
WHERE shown_at > now() - interval '14 days'
GROUP BY strategy
ORDER BY engagement_rate DESC;
```

If `embeddings`'s engagement rate beats `cooccur`'s by a healthy margin
with a decent sample size, the experiment is a green light. If it's
worse, the operator learns it *without any user ever seeing the worse
recs* — which is the entire promise of shadow mode. The failed
experiment cost nothing but compute.

**Day 15 — the rollout.** Confident in the numbers, the operator flips
`REC_ENGINE_MODE=pluggable`. The same strategies that were shadowed
now serve. And because `REC_SHADOW_MODE` is still true, the serving
pipeline keeps logging impressions, so the measurement never stops —
the experiment becomes continuous monitoring.

🧪 **Try It Yourself — run the shadow comparison query.** If you've
been running your dev server with `REC_SHADOW_MODE=true` for a few
days, the impressions table has real data. If not, seed a few rows
manually and run the query to see the shape of the output:

```bash
cd /personal/documents/code/rust/fichub
set -a; . ./.env; set +a
psql "$DATABASE_URL" <<'SQL'
INSERT INTO rec_impressions (user_id, work_id, strategy, shown_at, engaged_at) VALUES
  (1, 'fic_a', 'cooccur',   now() - interval '2 days', now() - interval '1 day'),
  (1, 'fic_b', 'cooccur',   now() - interval '2 days', NULL),
  (2, 'fic_c', 'embeddings', now() - interval '2 days', now() - interval '1 day'),
  (2, 'fic_d', 'embeddings', now() - interval '2 days', NULL),
  (3, 'fic_e', 'embeddings', now() - interval '2 days', NULL);

SELECT strategy,
       COUNT(*) AS shown,
       COUNT(engaged_at) AS engaged,
       ROUND(COUNT(engaged_at)::numeric / NULLIF(COUNT(*), 0), 4) AS engagement_rate
FROM rec_impressions
GROUP BY strategy
ORDER BY engagement_rate DESC;
SQL
```

(Use real user ids and work ids from your dev DB — the INSERT above is
a template.) The `NULLIF(COUNT(*), 0)` guards the division by zero for
strategies with no impressions, and `ROUND(..., 4)` keeps the output
readable. This exact query, pointed at production data, is the core of
FicHub's "should this strategy go live?" decision.

### 41.7 What shadow mode is *not*

Before we move on, three honest limits of this design — because every
real system needs its engineers to know where the edges are.

**Shadow mode measures *shown-would-engage*, not *would-engage-if-
ranked-higher*.** The shadow pipeline logs what it would show, but the
user never sees it, so the engagement it harvests is engagement with
works the user found *some other way* — a bookmark from a friend, a
search result, a random browse. That's a reasonable proxy for "is this
work appealing to this user at all", but it's *not* a measure of
position effects — whether rank #1 beats rank #4. Position is exactly
what the impressions table can't tell you in shadow mode. This is the
classic gap between offline evaluation and online serving, and the
standard answer is the one FicHub uses: shadow mode is the *gate*, the
pluggable rollout with continuous impressions is the *confirmation*.
Don't treat the gate as the last word.

**Cold-start strategies have no shadow evidence.** A brand-new strategy
starts with an empty arms table and zero impressions — it can't
*accumulate* shadow evidence until it's shadowed. So the first shadow
run of a new strategy is necessarily data-poor, and the honest reading
is "we have no evidence", not "the strategy is bad". FicHub handles
this by shadowing new strategies *before* they need evidence — the
whole point of the bench strategies in Chapter 40 is that they can run
in shadow for weeks accumulating impressions while `cooccur` keeps
serving. Patience is a feature.

**Impressions measure *your* showing, not *all* showings.** Every
impression row is logged by FicHub's own pipeline. If a user reads a
rec elsewhere — a link shared on social media, a cross-post — that
engagement never enters the table, and the reconciliation logic might
count a "cold" impression that the user actually enjoyed elsewhere.
There's no fix for this; it's the price of only measuring what you
control. Just know it's there, and don't over-interpret small
differences in engagement rates as strategy superiority.

### 41.8 Where we are

The platform now has its measurement instrument (`rec_impressions`),
its safe experimentation mode (`REC_SHADOW_MODE`), its nightly harvest
(the bandit's reconciliation + decay), its orchestrator
(`run_training_pipeline`), and its instrument panel (the strategies
endpoint). Every strategy's claims are logged at show time and judged
later, and the pipeline that does the judging is idempotent,
failure-tolerant, and crash-safe.

But there's a piece of the story we've been circling all chapter: the
*signals* — the materialized `rec_user_signals` view that every
personalized strategy reads, refreshed at the top of every training
run. Where do those signals come from, and how does a user's bookmarks
and downloads become a personalized feed? That's the last piece of the
platform, and it's Chapter 42 — where the personal recommendation
endpoint finally gets its full pluggable treatment, and the curator
gets a vote in everyone's feed.
---

## Chapter 42 — The Personal Recommendation Feed

Every chapter in this part has been building toward one moment: a
signed-in reader opens FicHub's home page, and a feed appears that
feels like it was written just for them. This chapter wires that
moment. We follow the life of a personal recommendation from the raw
signal — a bookmark, a download — through the unified signal view, the
strategy blend, the curator's vote, and out through the API as the
feed the frontend renders. By the end, `GET /api/recommendations/personal`
is a pipeline with a paper trail, not a mystery.

### 42.1 The signals: bookmarks and downloads, unified

Chapter 41 kept referring to "the signal view" like it was common
knowledge. It's time to read it properly — `src/recommender/signals.rs`
and the `rec_user_signals` materialized view from migration 027. The
module doc is a table, and tables are the right format for this:

```rust
//! | source    | table        | keyed by      | weight      |
//! |-----------|--------------|---------------|-------------|
//! | bookmark  | bookmarks    | users.id      | 4.0         |
//! | download  | request_log  | ANONYMOUS     | 2.0         |
//! | rating    | work_ratings | users.id      | = rating (1..=5) |
//! | review    | reviews      | users.id      | 3.0         |
```

Four sources, four weights, one question: *how strongly did this user
signal that they like this work?* A bookmark is worth 4.0 — it's
deliberate, persistent, and visible to the world. A download is worth
2.0 — it's a strong "I want this", but it's *anonymous* in `request_log`
and therefore can't be attributed to a user (more on that wrinkle in a
moment). A rating counts its own value — 1..=5, so a 5-star rating is
the strongest single signal in the system. A review is worth 3.0 — it
takes effort and thought, more than a click but less public than a
bookmark. The weights are a product decision encoded as constants:

```rust
/// Weight of a bookmark signal.
pub const WEIGHT_BOOKMARK: f64 = 4.0;
/// Weight of a download signal.
pub const WEIGHT_DOWNLOAD: f64 = 2.0;
/// Weight of a review signal.
pub const WEIGHT_REVIEW: f64 = 3.0;
/// Curator multiplier (their interactions count 5×).
pub const CURATOR_MULTIPLIER: f64 = 5.0;
```

💡 **Key Concept — signal weights are the product's taste, written
down.** Every recommendation system makes this choice, explicitly or
implicitly: a bookmark counts more than a view; a 5-star rating counts
more than a bookmark; a follow counts more than a one-off read. The
difference between a thoughtful system and a sloppy one is whether the
choice is a named constant with a comment or a magic number buried in
a query. When you build your own signal model, write the weights down
in one place, with comments, and revisit them as deliberately as you
revisit a pricing table — because that's exactly what they are.

Now the wrinkle, and it's an important one — the module doc is upfront
about it:

```rust
//! Downloads are anonymous in `request_log` (no user linkage — this is a
//! long-standing platform constraint; see the personal-recs reference). For
//! personal strategies they are treated as *global* popularity context
//! rather than per-user signal; the materialized `rec_user_signals` view
//! stores only per-user signals, and download popularity is computed
//! directly by strategies that need it.
```

`request_log` has no user column — the platform never linked downloads
to accounts. (You saw the consequences in Part 6 when we built the
download pipeline: privacy by architecture, anonymous by design.) So
the unified view stores *only* the per-user signals — bookmarks,
ratings, reviews — and download activity enters personalization
through the back door: as *global popularity context* via
`download_popularity`, which counts recent downloads per work and
scales by the download weight. The legacy personal engine we read in
Chapter 39 had a similar split — bookmarks keyed by user, downloads as
a recent-activity list. This is that design made explicit and
systematic.

The view is materialized — an actual table, rebuilt in a transaction —
by `refresh_signals`, which we saw called at the top of every training
run:

```rust
pub async fn refresh_signals(db: &PgPool, curator_user_id: Option<i32>) -> Result<(), AppError> {
    let curator = curator_user_id.unwrap_or(-1);
    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM rec_user_signals").execute(&mut *tx).await?;

    // Bookmarks (per-user).
    sqlx::query(
        r#"INSERT INTO rec_user_signals (user_id, work_id, signal_type, signal_weight, occurred_at)
           SELECT b.user_id, b.url_id, 'bookmark',
                  $2 * (CASE WHEN b.user_id = $1 THEN $3 ELSE 1.0 END),
                  COALESCE(b.created_at, NOW())
           FROM bookmarks b
           WHERE b.url_id IS NOT NULL"#,
    )
    .bind(curator)
    .bind(WEIGHT_BOOKMARK)
    .bind(CURATOR_MULTIPLIER)
    .execute(&mut *tx)
    .await?;
    ...
```

Read the bookmark INSERT's SELECT closely, because it contains the
curator multiplier in action: `$2 * (CASE WHEN b.user_id = $1 THEN $3
ELSE 1.0 END)` — every bookmark is weighted 4.0, except the curator's
own, which is weighted `4.0 · 5.0 = 20.0`. The curator's interactions
count five times in the shared view, which is what makes their profile
dominate the blend for low-signal users — the mechanism behind the
curator prior we'll apply in §42.4. And the negative-rating decision is
deliberate and documented in the ratings INSERT:

```rust
// Ratings 1..=5 (per-user). Legacy -1 (dislike) is intentionally
// excluded from the unified signal view — the public surface is
// positive-only, and negative signals are handled by exclusions, not
// scoring.
sqlx::query(
    r#"INSERT INTO rec_user_signals (user_id, work_id, signal_type, signal_weight, occurred_at)
       SELECT r.user_id, r.url_id, 'rating', r.rating::float8,
              COALESCE(r.created_at, NOW())
       FROM work_ratings r
       WHERE r.url_id IS NOT NULL AND r.rating BETWEEN 1 AND 5"#,
)
.execute(&mut *tx)
.await?;
```

The legacy `-1` "dislike" rating — a relic from before Part 7 — is
*intentionally excluded*. You saw this philosophy in Chapter 39's
community-vote boost (negative net votes never penalize) and in Part
7's ratings rework: **FicHub's public scoring surface is positive-only,
and dislikes are expressed by *absence* — you simply don't recommend
works the user has interacted with negatively, and the "known works"
exclusion in every personalized strategy handles that.** The `WHERE ...
BETWEEN 1 AND 5` is the exclusion made into a constraint.

Why materialize at all, instead of joining the source tables per
query? Three reasons, all of them about the strategies in Chapter 40.
First, *consistency*: every strategy — MF training, embedding
centroids, cluster vectors, curator alignment — reads the exact same
signal snapshot, so they can't disagree about what the user's history
is. Second, *performance*: the strategies query this view constantly
(`count_user_signals`, `user_tag_vector`, the MF matrix build all hit
it), and a materialized table with its primary key on
`(user_id, work_id, signal_type)` is far cheaper than re-aggregating
four source tables on every access. Third, *the curator multiplier*:
the 5× weighting exists only in this view — the source `bookmarks`
table is untouched, so the multiplication can't leak into other
features. A materialized view is a *cached computation with a defined
refresh point* — and the refresh point (top of every training run) is
exactly when the cache should update.

⚠️ **Watch Out — a materialized view is a snapshot, and snapshots go
stale.** `rec_user_signals` reflects the database as of the last
`refresh_signals`. A user who bookmarks a work at 2 p.m. and requests
their personal feed at 2:01 p.m. might get a feed that doesn't know
about the bookmark — until the next training run (up to `REC_TRAIN_EVERY_H`
= 6 hours later) refreshes it. For a recommendation feed that's
acceptable — recommendations are allowed to lag a few hours; they're
not billing statements. But the *choice* must be explicit: if a
feature needs sub-minute freshness, it must not read the materialized
view. This is the same stale-cache discipline as Part 5's export cache:
know your snapshot's age, and know which features can tolerate it.

### 42.2 The handler: legacy path, pluggable path, one endpoint

The endpoint is `GET /api/recommendations/personal` — the handler we
met in passing in Chapter 39. Let's read it as the *router* it really
is:

```rust
pub async fn personal_recommendations_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let Some(user_id) = auth.user_id else {
        return Ok(empty_personal_response());
    };

    // ── Pluggable mode: registry + RRF ranker (REC_ENGINE_MODE=pluggable) ──
    if state.config.rec_engine_mode.is_pluggable() {
        return pluggable_personal_handler(state, user_id).await;
    }
    let _ = state; // legacy path keeps the borrow pattern below

    let db = &state.db;
    let (recs, based_on_titles, enough) =
        compute_personal_recommendations(db, user_id, &state.config).await?;

    if !enough {
        return Ok(empty_personal_response());
    }
    ...
}
```

Three branches, three exit points, one contract: **a 200 with
`enough_data: false` and empty lists is a *valid* response, not an
error.** Anonymous user → empty. Not enough signals → empty. Strategies
all failed → empty. The frontend renders an empty feed as a hint card
("bookmark a few fics to get recommendations") instead of an error
state — no auth round-trip, no 500, no JSON shape surprises. The
`empty_personal_response` helper pins that shape:

```rust
fn empty_personal_response() -> Json<Value> {
    Json(json!({
        "err": 0,
        "enough_data": false,
        "recs": [],
        "based_on": [],
    }))
}
```

This is the response contract as a *function* — one definition, used by
every branch, impossible to get subtly different between them. When you
have a "no data" outcome that should look identical across many code
paths, factor it into a helper exactly like this.

The legacy branch calls `compute_personal_recommendations` — the full
computation we already read in Chapter 39: bookmarks + download ids →
gate on `PERSONAL_RECS_MIN_SIGNAL` (3) → top tags → SQL candidate
generation → Rust scoring (Jaccard overlap + recency popularity) →
truncate to `PERSONAL_RECS_N` (20) → `based_on` from bookmark titles.
It's the engine that has served this endpoint for years, and thanks to
Chapter 39's golden test, it's also the `cooccur` strategy's
personalized arm. The legacy path and the pluggable path *share* it —
the golden test proved they agree.

### 42.3 The pluggable handler: blend, then shape

The interesting branch is `pluggable_personal_handler`, which is where
all of Part 9's machinery finally meets the user. It's short, and its
shortness is the point:

```rust
pub async fn pluggable_personal_handler(
    state: Arc<AppState>,
    user_id: i32,
) -> Result<Json<Value>, AppError> {
    use crate::recommender::ranker;

    let ctx = crate::recommender::registry::build_context(
        state.db.clone(),
        std::sync::Arc::new(state.config.clone()),
        state.http_client.clone(),
        state.ollama.clone(),
    );
    let (blended, diagnostics) = ranker::blend(
        &ctx,
        &state.strategy_registry,
        None,
        Some(user_id),
        crate::recommender::engine::PERSONAL_RECS_N,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if blended.is_empty() {
        return Ok(empty_personal_response());
    }
    ...
}
```

Build a context, call `ranker::blend` with `seed: None, user_id:
Some(user_id)`, handle the empty case, shape the response. Every
strategy in `REC_STRATEGIES` runs inside that one `blend` call — the
fallback chain skips the ones that can't answer, RRF fuses the rest,
the curator prior (next section) re-ranks, the bandit slots in
exploration. The endpoint's only *new* work is hydration: the blend
returns `BlendedRec`s with `work_id`, `score`, `strategy`, and `reason`
— but the frontend needs titles, authors, word counts, summaries. So
each blended rec gets its metadata fetched and packed into the same
`RecResult` DTO the legacy path always served:

```rust
let mut recs: Vec<Value> = Vec::with_capacity(blended.len());
for br in blended {
    let info = sqlx::query_as::<_, FicInfoRow>(
        r#"SELECT id, title, author, words, chapters, status, source, description
           FROM fic_info WHERE id = $1"#,
    )
    .bind(&br.work_id)
    .fetch_optional(&state.db)
    .await?;
    if let Some(info) = info {
        recs.push(serde_json::to_value(RecResult {
            url_id: info.id,
            title: info.title,
            author: info.author,
            words: info.words,
            chapters: info.chapters,
            status: info.status,
            site_domain: info.source,
            summary: info.description,
            score: br.score,
            community_score: 0,
            download_urls: HashMap::new(),
        })
        .unwrap_or(Value::Null));
    }
}
```

A work that vanished from `fic_info` between the blend and the
hydration (deleted, re-scraped) is silently dropped rather than
breaking the response — `fetch_optional` + `if let Some`. The response
keeps the legacy shape *exactly* — `err`, `enough_data`, `recs`,
`based_on` — and adds two new fields the legacy response never had:

```rust
Ok(Json(json!({
    "err": 0,
    "enough_data": true,
    "recs": recs,
    "based_on": based_on,
    "strategies": diagnostics,
})))
```

`strategies` is the `StrategyRunInfo` diagnostics from Chapter 38 —
per-strategy `{name, contributed, count, error, duration_ms}` — and
`based_on` is the familiar bookmark-title list. The response contract
is *backward-compatible by construction*: every field the old frontend
read is still there, and the new fields are additive. The frontend
doesn't need to know which engine produced the feed; it reads the same
JSON either way. That's the payoff of the staged rollout: `legacy` and
`pluggable` are one endpoint, two pipelines, one wire format.

💡 **Key Concept — the wire format is the compatibility contract.**
Notice what the pluggable handler did *not* do: it did not invent a new
response shape for the new engine. The DTO (`RecResult`) and the JSON
envelope (`err`/`enough_data`/`recs`/`based_on`) predate the platform,
and the new pipeline was built to speak them. This is the lesson of
Chapter 39's golden test applied at the API level: when you replace an
engine, the *interface* the consumers depend on is sacred, and the
implementation behind it is disposable. Every feature that consumes
personal recommendations — the home feed, the dashboard widget, the
mobile app — kept working across the entire refactor because nobody
changed the shape they read.

### 42.4 The curator's vote: `curator_alpha` and `apply_curator_prior`

Now the piece Chapter 40 promised: the curator prior, applied inside
`ranker::blend` after all strategies fuse. Its purpose is the coldest
of cold-start problems: **what do you show a brand-new user who has one
bookmark and no history?** Any strategy's answer is going to be
terrible — there's simply no evidence. FicHub's answer is to hand the
wheel to a human: the curator, a real person (or small team) whose
interactions are weighted 5× in the signal view (42.1), whose profile
is fetched as `curator_top`, and whose taste *shapes every
low-signal user's feed*.

The alpha — the curator's weight in the blend — is a pure function with
a memorable shape:

```rust
/// Compute the curator-prior alpha for a user:
/// `α = floor + (1 - floor) · exp(-n_signals / τ)`
///
/// α is the CURATOR's weight in the blend:
/// `final = (1-α)·user + α·curator`
///
/// * `floor` = REC_PRIOR_FLOOR (default 0.2) — the minimum curator
///   influence. Even a very active user keeps ≥ 20% curator shaping.
/// * `τ` (tau) = REC_CURATOR_TAU (default 25 signals)
///
/// A brand-new user (0 signals) gets α = 1.0 → pure curator picks; an
/// active user converges to α ≈ floor (their own taste dominates, curator
/// floor respected).
pub fn curator_alpha(n_signals: i64, floor: f64, tau: f64) -> f64 {
    let floor = floor.clamp(0.0, 0.999);
    let n = n_signals.max(0) as f64;
    floor + (1.0 - floor) * (-n / tau.max(1.0)).exp()
}
```

Read the formula as a curve. At `n = 0` signals: `α = floor + (1-floor)
· 1 = 1.0` — pure curator. As `n` grows, the exponential term shrinks
and `α` decays toward `floor`. At 25 signals (τ), `α` is halfway from
1.0 to the floor. At hundreds of signals, `α ≈ floor = 0.2` — the
user's own taste dominates, but the curator keeps a 20% vote forever.
The `clamp(0.0, 0.999)` on the floor and `tau.max(1.0)` are the
defensive parsing you've seen all book long — a misconfigured floor of
1.5 or a tau of 0 must not produce α outside [0, 1] or a division by
zero.

The blend itself is applied to the fused score list:

```rust
pub fn apply_curator_prior(
    fused: &[(String, f64)], // (work_id, user_blend_score)
    curator_top: &[(String, f64)], // (work_id, curator_score)
    alpha: f64, // curator weight — see curator_alpha
) -> Vec<(String, f64)> {
    let curator: HashMap<&str, f64> = curator_top.iter().map(|(w, s)| (w.as_str(), *s)).collect();
    let mut out: Vec<(String, f64)> = fused
        .iter()
        .map(|(w, s)| {
            let lift = curator.get(w.as_str()).copied().unwrap_or(0.0);
            // user contribution keeps (1-α) of its weight; curator lift adds α·curator_score
            (w.clone(), s * (1.0 - alpha) + alpha * lift)
        })
        .collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}
```

For every fused candidate: the user's blended score keeps `(1-α)` of
its weight, and the curator's own score for that work (0 if the curator
hasn't signalled it) adds `α·curator_score`. A work the curator loves
gets lifted; a work the curator has never touched keeps its user score
scaled down slightly; the list is re-sorted by the blended value. The
unit test in `ranker.rs` pins the exact arithmetic:

```rust
let fused = vec![
    ("w1".to_string(), 0.8),
    ("w2".to_string(), 0.7),
    ("w3".to_string(), 0.6),
];
let curator = vec![("w1".to_string(), 1.0), ("w3".to_string(), 0.9)];
// α = 0.4 curator weight: w1: 0.8·0.6 + 0.4·1.0 = 0.88;
// w3: 0.6·0.6 + 0.4·0.9 = 0.72; w2: 0.7·0.6 = 0.42
let alpha = 0.4;
let out = apply_curator_prior(&fused, &curator, alpha);
assert!(out[0].0 == "w1", "{out:?}");
assert!(out[1].0 == "w3", "{out:?}");
assert!(out[2].0 == "w2", "{out:?}");
```

w1 gets the lift (0.88), w3 gets a smaller lift (0.72), and w2 — the
work the curator never touched — falls from second to last (0.42).
That's the curator's vote in its purest form: *it re-ranks, it doesn't
reinvent*. The user's own signals still decide what's *in* the feed;
the curator decides the *emphasis*.

Where does `curator_top` come from? The ranker fetches the curator's
own profile from the signal view — the works they signalled, weighted
by signal strength:

```rust
pub async fn fetch_curator_top(ctx: &StrategyContext, curator_id: i32) -> Vec<(String, f64)> {
    // Curator profile = works the curator bookmarked/rated, weighted by
    // signal strength (curator interactions are 5× in rec_user_signals).
    sqlx::query_as::<_, (String, f64)>(
        r#"SELECT work_id, SUM(signal_weight) AS s
           FROM rec_user_signals
           WHERE user_id = $1
           GROUP BY work_id
           ORDER BY s DESC
           LIMIT 50"#,
    )
    .bind(curator_id)
    .fetch_all(&ctx.db)
    .await
    .unwrap_or_default()
}
```

"Curator profile = works the curator bookmarked/rated" — because their
interactions were multiplied 5× at refresh time, this list is
dominated by their strongest signals, exactly as intended. And the
alpha used is recorded per-user for QA, so you can audit *how much*
curator shaping each user received:

```rust
// Best-effort alignment tracking (cos(user_centroid, curator_centroid) is
// computed by the pipeline worker; here we record the alpha used).
async fn record_alignment(ctx: &StrategyContext, user_id: i32, curator_id: i32, alpha: f64) {
    let alignment = compute_alignment(ctx, user_id, curator_id).await;
    let _ = sqlx::query(
        r#"INSERT INTO rec_user_curator_align (user_id, alignment, n_signals, alpha, updated_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (user_id) DO UPDATE SET ..."#,
    )
    ...
}
```

`rec_user_curator_align` is the audit table: for every user, the
cosine alignment between their tag profile and the curator's (via
`compute_alignment`, which compares their `user_tag_vector`s), their
signal count, and the alpha that was applied. An operator can query
"which users are getting 80%+ curator shaping?" and "is the low-signal
cohort actually converging toward the curator's taste?" — the QA loop
that tells you whether the prior is working, or just running.

💡 **Key Concept — the curator prior is a *prior*, not a filter.** The
name matters. A Bayesian prior is what you believe *before* you see the
data, and it gets *updated* by the data — it never overrides it. The
curator prior behaves exactly that way: it's strongest when the user's
data is thinnest (α → 1.0) and it asymptotically steps aside as real
signals accumulate (α → floor). It never *blocks* a work the user's
own signals produced — it only re-weights and re-ranks. If you find
yourself building "cold start" logic that *replaces* a user's signals
with a default, ask whether a prior that *blends* wouldn't serve
better: blending degrades gracefully as evidence arrives, replacing
doesn't.

### 42.5 The response: reasons, strategies, and the frontend contract

The final JSON carries one field we haven't celebrated properly, and
it's the field Chapter 38 promised would "earn its keep": the `reason`
string on every rec. Let's look at what the feed actually says to the
user:

```json
{
  "err": 0,
  "enough_data": true,
  "based_on": [
    { "title": "The Dragon's Apprentice", "url_id": "fic_alpha" },
    { "title": "Flame and Scale", "url_id": "fic_beta" }
  ],
  "strategies": [
    { "name": "cooccur", "contributed": true, "count": 12, "duration_ms": 4, "error": null },
    { "name": "embeddings", "contributed": true, "count": 8, "duration_ms": 9, "error": null },
    { "name": "mf", "contributed": false, "count": 0, "duration_ms": 1, "error": "user 42 not in MF training set" }
  ],
  "recs": [
    {
      "url_id": "fic_gamma",
      "title": "Embers of the North",
      "author": "SomeAuthor",
      "score": 0.0213,
      "strategy": "cooccur",
      "reason": "co-bookmarked by other readers"
    },
    {
      "url_id": "fic_delta",
      "title": "The Last Wyrm",
      "author": "AnotherAuthor",
      "score": 0.0187,
      "strategy": "embeddings",
      "reason": "matches your reading profile (embeddings)"
    }
  ]
}
```

Every rec carries its `strategy` and `reason`. The QA team can spot a
misbehaving strategy by its reason strings. The frontend can render
"why am I seeing this?" as a tooltip — and the `strategies` block
explains the *engine*: `mf` tried and honestly reported "user 42 not
in MF training set" rather than silently vanishing. A feed that can
explain itself is a feed that can be debugged, audited, and trusted.
This is the machine-readable *why* the part's introduction promised,
and it's the field most recommendation systems skip because it's "just
UI". It's not — it's accountability.

⚠️ **Watch Out — the reason strings are a public API surface, not
log noise.** They're serialized into every response, they may be
rendered to users, and once a frontend starts displaying them, they
become a contract. Changing "co-bookmarked by other readers" to
"similar readers liked this" is a product change; changing it to a
debug dump of internal state would be a bug. The strategies set them
as short, human-readable phrases — keep them that way. If you need
verbose diagnostics, that's what `diagnostics` (the `strategies`
block) is for. The wire format has a place for each kind of
information, and mixing them up is how you end up explaining
`RecError::NotEnoughData("fewer than 3 signals — legacy gate")` to a
confused user.

### 42.6 Running the whole feed

Time to see the pipeline breathe. First, make sure your dev database
has a user with at least three signals (three bookmarks is plenty),
then hit the endpoint in legacy mode and in pluggable mode, and compare
the shapes:

```bash
cd /personal/documents/code/rust/fichub
# Legacy mode (default)
curl -s http://localhost:8080/api/recommendations/personal \
  -H "Authorization: Bearer <your_token>" | python3 -m json.tool

# Pluggable mode, shadow off — the pipeline serves
REC_ENGINE_MODE=pluggable cargo run 2>&1 | tail -5 &
# ...wait for startup, then:
curl -s http://localhost:8080/api/recommendations/personal \
  -H "Authorization: Bearer <your_token>" | python3 -m json.tool
```

In legacy mode you get the classic four-field envelope. In pluggable
mode you get the same envelope plus `strategies` and per-rec
`strategy`/`reason` fields. The recs themselves may differ — that's the
platform working: in pluggable mode the feed is the RRF blend of every
enabled strategy, not just `cooccur`'s tag overlap.

Now the experiment that ties the part together — run the shadow
comparison from Chapter 41 against your own feed. Serve the legacy
pipeline (`REC_SHADOW_MODE=true`), read your feed, then check what the
shadow pipeline logged:

```bash
set -a; . ./.env; set +a
psql "$DATABASE_URL" -c "
SELECT strategy, COUNT(*) AS shown, COUNT(engaged_at) AS engaged
FROM rec_impressions
GROUP BY strategy ORDER BY shown DESC;"
```

Your own reading session just became an experiment row. That's the
whole arc of Part 9 in one command: strategies score, the ranker
blends, the shadow logs, the bandit reconciles, and the next feed is
slightly better informed than the last one.

🧪 **Try It Yourself — read the reasons, then trust them.** Pick the
top three recs from your pluggable-mode feed and check each `reason`
against reality: for a `cooccur` rec ("co-bookmarked by other
readers"), does the work actually share bookmarkers with your
bookmarks? For an `embeddings` rec ("matches your reading profile"),
does the summary actually read like your history? For a bandit rec
("exploration — trying something new"), is it genuinely off your usual
tracks? This five-minute audit is the single best QA habit for a
recommendation system — the reasons make it *possible*, which is why
the platform insists on them. When a rec's reason and reality diverge,
you've found a strategy bug, a signal bug, or a data staleness problem
— and you found it before a user did.

### 42.7 The part, in review

Step back and see what Part 9 built, one chapter at a time.

**Chapter 38** drew the line that made everything else possible:
strategies are pure scorers, the registry is config-driven, and the
`RecStrategy` trait is the contract between algorithm authors and the
pipeline. **Chapter 39** wrapped the legacy engine as `cooccur` and
pinned it with a golden test — proving the new architecture could be
half-built without changing what users saw. **Chapter 40** filled the
stable: decay, embeddings, MF, hybrid, tag graph, clusters, bandit,
author graph, sequential, external — eleven answers to "what should I
read next?", each with honest failure modes. **Chapter 41** built the
measurement: impressions logged at show time, shadow mode as the safe
A/B test, the nightly reconciliation that turns impressions into
posteriors, and the training pipeline that keeps it all fresh.
**Chapter 42** wired the endpoint: signals unified into one view, the
blend hydrated into the legacy wire format, the curator's vote applied
to cold-start users, and every rec carrying its reason.

The architecture that held it all together is worth restating, because
it's the part that transfers to any project you'll build next:

- **Separate scoring from serving.** Strategies never decide what's
  shown; the ranker does. That one rule makes blending, curation,
  exploration, and A/B testing all possible.
- **Pin the legacy behavior.** The golden test made the refactor safe;
  the wire-format compatibility made the rollout invisible.
- **Measure before you trust.** Impressions at show time, engagement
  after, temporal ordering enforced in SQL, idempotent reconciliation.
- **Default to safe.** `legacy` mode, shadow off, `cooccur` as the
  floor — every default is the known-good state, and every knob is an
  explicit operator decision.
- **Explain every rec.** The `reason` field turned a black box into an
  auditable, debuggable, trustable feed.

### 42.8 Where we are — and where we go next

The recommendation platform is complete: strategies, registry, ranker,
measurement, and the personal feed that turns every reader's bookmarks
and downloads into their next favorite story — with a reason attached
to every suggestion, and the curator shaping what new readers see while
their own taste forms.

And now the platform can tell us *why* it recommends what it does, in
plain English. That's a capability with a name, and it's about to get a
lot more powerful: in Part 10, we take FicHub into the world of AI
features — where the same signals that drive recommendations power
summaries, tags, and conversations with your own reading history. The
recommendation platform gave every reader a personalized answer to
"what should I read next?". Part 10 gives FicHub a voice to go with
it. See you there.
