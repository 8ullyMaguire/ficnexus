## Part 2A — Deep Dives: The Systems You'll Touch Most

### Chapter 6A: The Search Engine — Boolean Parser & main_char_attr

Search is one of FicHub's flagship differentiators. Open
`src/search/parser.rs` — this is the boolean query parser, and it's the
best single file to study if you want to understand how the product thinks.

**What the parser supports:**

- **Simple words** — `coffee` matches fics mentioning coffee.
- **Boolean operators** — `AND`, `OR`, `NOT` (and implicit AND between
  words: `coffee angst` means `coffee AND angst`).
- **Quoted phrases** — `"slow burn"` matches the phrase as a whole.
- **Exclusion** — `-angst` or `NOT angst` hides stories with that term.
- **Fielded search** — `title:harry`, `author:rowling`, `fandom:...`,
  `character:...`, `relationship:...`, `main_char_attr:...`.
- **Parentheses** — group terms: `(fluff OR humor) AND angst`.

The parser turns a query string into a filter AST. The search handler
(`src/routes/search.rs`) walks that AST and builds SQL against `fic_info`
plus the tag tables, with faceted navigation (click values in the sidebar
to add filters) and filter chips (active filters shown as removable chips).

**The `main_char_attr` semantics.** This is AO3-parity niche search — the
kind of thing that made FicHub's search "better than AO3" for power users.
The idea: a filter that says "the MAIN character is X with attribute Y."
`main_char_attr: dark harry potter` means stories whose main character is
Harry Potter, dark!Harry in particular. It's parsed as (main character, +
attribute) — the attribute applies to the MAIN character, not just any
character in the fic. The data comes from character tags with main-character
scores (the score columns feed the "primary tag" and "main char" filters).

**Why this matters for you:** search is where "AO3-parity niche searches"
come from — the product's community values being able to find *exactly* the
fic they want, not just "Harry Potter" broadly. The parser is pure Rust,
heavily unit-tested, and a great place to practice reading a non-trivial
recursive descent parser.

> 🧪 **Try it:** in the search UI, run `(fluff OR humor) AND angst`, then
> `main_char_attr: dark harry potter`, then `-angst "slow burn"`. Watch the
> chips update. Then read `parser.rs` and find where `main_char_attr` is
> handled.
>
> ⚠️ **Watch out:** the parser has known pitfalls (see the
> `boolean-query-parser-pitfalls` skill) — don't rewrite it casually. Tune,
> don't replace.
>
> 💡 **Key concept:** search is a filter AST → SQL. Fielded search lets
> users target metadata; `main_char_attr` targets the main-character
> semantic.

### Chapter 6B: Ask the Archive — Natural-Language Search

`/api/search/ask` is the "Ask the Archive" endpoint. A user types a
sentence like "completed slow-burn Dramione over 50k, no major character
death," and the server uses Ollama (an LLM) to convert that into search
filters, then runs the filters through the same search pipeline.

The flow:

1. User posts the natural-language query.
2. The server calls Ollama with a prompt that says "convert this to search
   filters" and gets back a structured filter object (JSON).
3. The filters are applied to the search handler.
4. If Ollama is down or the parse fails, it degrades gracefully to a plain
   full-text search — **the feature never fails the user**, it just gets
   dumber.

This "graceful fallback" pattern is worth studying: the LLM is an
enhancement, not a dependency. When the model is unavailable (or slow —
cold starts can be ~17 seconds), users still get results.

> 🧪 **Try it:** open `/ask` on the live site and ask for something
> specific. Then read `src/routes/search.rs`'s ask handler to see the
> fallback path.
>
> ⚠️ **Watch out:** LLM responses are unstructured — always validate/parse
> the model's output before using it. Never trust it blindly.
>
> 💡 **Key concept:** LLM features degrade gracefully. The model is a
> best-effort enhancement on top of a deterministic pipeline.

### Chapter 6C: The Recommendation Platform

`src/recommender/` is the pluggable recommendation platform — one of the
most ambitious parts of the codebase. The core idea: **recommendations are
produced by a registry of strategies**, and the product can A/B them
safely.

**The strategy trait:**

```rust
pub trait RecStrategy {
    fn name(&self) -> &str;
    async fn recommend(&self, ctx: &StrategyContext, query: &RecQuery)
        -> Result<Vec<ScoredRec>, RecError>;
}
```

**The strategies** (all built, most inert):

- `cooccur` — the legacy co-occurrence engine (bookmarked-together
  statistics), currently the default.
- `decay` — time-decayed SAR (users' recent actions weighted more).
- `embeddings` — pgvector similarity over fic embeddings.
- `mf` — implicit matrix factorization (ALS).
- `hybrid` — RRF-blended combination of strategies.
- `tag_graph` / `author_graph` — graph-based similarity.
- `sequential` — Markov chain for "what to read next."
- `clusters` — user taste clusters.
- `bandit` — exploration/exploitation.
- `external` — call an off-box rec service.

**How the engine picks:** config-driven registry (`REC_STRATEGIES` weights),
golden test protecting legacy parity, curator prior, and bandit exploration.
The **golden test** (`golden_legacy_equals_cooccur_strategy`) asserts the
pluggable `cooccur` strategy produces identical output to the legacy
engine — that's the safety net that lets new strategies be developed
without fear.

**Shadow mode.** `REC_SHADOW_MODE=true` computes pluggable strategies but
returns the legacy output while logging impressions to `rec_impressions`.
This is how you A/B a new strategy: run it in shadow, compare impressions
after a week, promote the winner by flipping `REC_ENGINE_MODE` /
`REC_STRATEGIES`.

The **recommender worker** (`src/recommender/worker.rs`) is a background
task that maintains co-occurrence data and imports AO3 profiles (feeding
the legacy `fic_bookmarks` system).

> 🧪 **Try it:** read `src/recommender/registry.rs` to see the strategy
> list + weights. Then read the golden test to see how legacy parity is
> protected.
>
> ⚠️ **Watch out:** most strategies are inert today
> (`REC_ENGINE_MODE=legacy`). Don't assume a strategy's endpoint works
> until you've enabled it in config.
>
> 💡 **Key concept:** recommendations are pluggable + shadow-testable. The
> golden test is the contract that keeps the legacy engine's quality from
> regressing.

### Chapter 6D: Anti-Bot Defense

FicHub is a public download service — bots love it. `src/limiter/` and
`src/routes/pow.rs` implement a layered defense:

1. **Honeypot traps** — invisible form fields that bots fill and humans
   don't; filling one marks the client a bot.
2. **Tiered rate limits** — Redis token buckets per IP and per client ID,
   with dynamic tiers. `src/limiter/mod.rs` defines `RateLimitResult`,
   `Tier`, and the `TieredRateLimiter` trait; `redis_bucket.rs` implements
   it with a Lua script for atomicity.
3. **Shadowban** — Redis set of shadowbanned clients; they get PoW
   challenges and degraded responses without knowing it.
4. **Proof of work** — hashcash-style: `GET /api/pow/challenge` + `POST
   /api/pow/solve`. Shadowbanned clients must solve a SHA-256 prefix
   challenge (difficulty 16 → `0000` hex prefix) before exporting. Solves
   are cached in Redis with a TTL.
5. **Hourly bot-scorer** — a background job scores clients and promotes
   bot-like behavior into the shadowban set.

The PoW gate is wired into the export handler BEFORE the tiered limiter:
a shadowbanned client with no stored solve gets `429 {err:-429, "proof of
work required", ...}`; everyone else gets `{err:0, not_needed:true}` —
zero friction.

> 🧪 **Try it:** read `src/services/pow.rs` — the challenge generation and
> verification are small, testable, and pure (no I/O).
>
> ⚠️ **Watch out:** the PoW gate only applies to shadowbanned clients.
> Don't add friction for normal users.
>
> 💡 **Key concept:** defense in depth — honeypots → rate limits →
> shadowban → PoW — with zero friction for real users.

### Chapter 6E: The Roadmap Consensus — MaxDiff/Elo on Embeddings

`src/routes/roadmap.rs` + migration 011 implement a genuinely unusual
feature: users propose feature ideas, the system embeds them with Ollama
(`nomic-embed-text`, 768-dim vectors), clusters similar ideas, and runs a
**MaxDiff arena** where users pick best/worst from sets of four; the votes
feed a virtual **Elo rating** per cluster.

The pipeline:

1. `POST /api/roadmap/suggest` — embed the idea, find the nearest cluster
   (cosine distance < 0.22), join it or spawn a new one. Rate-limited to 3
   per day. Ollama down? Store unclustered — never fail the submission.
2. `GET /api/roadmap/arena` — return 4 open clusters, least-played first.
3. `POST /api/roadmap/vote` — K=32 Elo update in an atomic transaction;
   double-vote → 400.
4. Admin sees the leaderboard + controversy scatter
   (`/admin/consensus`).

The user's product philosophy is visible here: **rigorous consensus
mechanics (Semantic Clustering + MaxDiff/Elo) over upvotes** — this is how
the community decides what to build next, and it feeds the roadmap.

> 🧪 **Try it:** open `/roadmap` and vote in the arena. Then read the Elo
> update in `roadmap.rs` — it's a small, beautiful function.
>
> ⚠️ **Watch out:** the embedding dimension must match the vector column
> (768). A dimension mismatch is a runtime error, not a compile error.
>
> 💡 **Key concept:** consensus is engineered. Embeddings + MaxDiff + Elo
> turn noisy feature requests into a ranked, community-driven roadmap.

### Chapter 6F: Fic Requests — the Prompt Board

`src/routes/requests.rs` + migrations 016-018 implement the fic-request
board: users post a request ("I want a completed slow-burn Dramione over
50k"), other users answer with works, and the community votes.

**The data model:**

- `fic_requests` — the request: title, description, status (open /
  answered), the requester.
- `fic_request_answers` — one row per suggested work, UNIQUE per request,
  capped at +3 per user.
- `fic_request_answer_votes` — fit votes (up/down), net score only, no
  self-vote.

**The endpoints:**

```rust
POST /api/requests                    // create
GET  /api/requests                    // list
POST /api/requests/{id}/answers       // add a work answer
POST /api/requests/{id}/answers/{aid}/vote  // vote
POST /api/requests/{id}/accept/{aid}  // requester accepts → 'answered'
GET  /api/requests/{id}/candidates    // engine suggestions (M2)
```

The **candidates endpoint** is the interesting recent addition: it uses the
recommender engine to suggest works for a request based on the request's
seed work. The board can answer itself with engine suggestions, not just
human votes.

> 🧪 **Try it:** open `/requests` on the live site or create a request on
> dev. Then read `requests.rs` — the vote logic (net score, no self-vote)
> is the pattern to match in any new voting feature.
>
> ⚠️ **Watch out:** answer cap (+3/user) and the UNIQUE constraint keep the
> board from being spammed by one user. Don't remove them casually.
>
> 💡 **Key concept:** requests are a voting board with a clear lifecycle
> (open → answered) and an engine-assisted candidate path.

### Chapter 6G: The Consensus & Feedback Philosophy

Two product decisions explain a LOT of the codebase:

1. **No public downvotes.** Ratings and reviews are positive-only on the
   public surface. Users can rate 1-5 stars, but the *displayed* lists and
   aggregate numbers emphasize constructive feedback. Dislikes feed
   algorithms internally (via `work_feedback_signals`) but are never shown
   as a public "downvote count."

2. **Consensus over popularity.** The roadmap uses MaxDiff/Elo, not
   upvotes. The idea: upvotes are noisy and gameable; structured pairwise
   comparison produces a more honest ranking.

When you add a social feature, match these principles: keep the public
surface constructive, keep the internal signal rich. The
`work_feedback_signals()` query is the bridge — it turns ratings + reviews
+ comments into rec-engine signal.

> 🧪 **Try it:** find `work_feedback_signals` in `queries.rs` and see what
> signals feed the recommender.
>
> ⚠️ **Watch out:** "positive-only public surface" is a rule — a public
> downvote UI would be a product regression, not an improvement.
>
> 💡 **Key concept:** feedback is a signal, not a scoreboard. Public =
> constructive; internal = rich.

---
