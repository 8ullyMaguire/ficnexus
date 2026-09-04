---
title: "FicHub Developer Onboarding"
author: "FicHub Contributors"
date: "2026-08-11"
---

# FicHub Developer Onboarding

> A ~20,000-word guide for junior developers joining the
> FicHub codebase: architecture, repo map, how to run it,
> where the key systems live, and where to start contributing.

---



---

<!-- part: 01-intro.md -->


# FicHub Developer Onboarding

A ~20,000-word guide for junior developers joining the FicHub codebase.

**What this book is:** a fast, hands-on orientation to the FicHub
architecture — Rust/Axum backend, SvelteKit 5 SPA, PostgreSQL 16 + pgvector,
Redis, Ollama — so you can open the repo, find your way around, and start
contributing with confidence.

**How to use it:** read it in order once (about a day), then keep it handy
as a reference. Every chapter points at real files. Open them as you read.
Run the code when a chapter tells you to. Type the commands yourself — the
muscle memory is the point.

**Repo:** `/personal/documents/code/rust/fichub` (canonical:
`https://opencommit.eu/MagicZhang/fichub`)

---

## Part 1 — What FicHub Is & How It's Built

### Chapter 1: The Product in One Paragraph (and Then Some)

FicHub is a self-hosted fanfiction archive and community platform. Paste a
story URL from Archive of Our Own (AO3), FanFiction.net (FFN), RoyalRoad, a
XenForo forum like boards.theforce.net, or dozens of other sites, and FicHub
scrapes the metadata and full text, converts it to EPUB, HTML, MOBI, PDF,
TXT, or Markdown, and lets you read it in a built-in web reader. Around that
core sits a full community layer: user accounts, bookmarks, five-star
ratings plus in-depth reviews, threaded comments, follows with an updates
feed, reading lists and shelves, a fic-request prompt board, per-fic
similar-fic suggestions, and a "roadmap consensus" feature where users vote
on what to build next.

It started as a self-hosted replacement for the old fichub.net service. The
difference from a plain download tool is the *community*: people rate,
review, recommend, request, and curate. The difference from a fanfiction
aggregator is the *cache*: FicHub keeps everything it scrapes, so it works
even when the upstream sites are slow, blocked, or gone.

The big architectural idea (as of August 2026) is worth stating twice:
**FicHub is a cache of all gathered fanfiction.** Every scraped fic body is
persisted as a JSON blob on the attached drive
(`/public/literature/fichub/bodies`), not just in the database. The database
holds metadata; the filesystem holds content. That makes repeat exports
instant, makes the archive grow even when upstream sites are down, and gives
curators a peer-voted "fix the body" workflow when a scrape is wrong.

Why does that matter for a junior developer joining the team? Because it
changes where the interesting code lives. In a naive downloader, the
interesting code is the scraper. In FicHub, the interesting code is
everywhere: scrapers, yes, but also the cache layer, the export pipeline,
the search parser, the recommendation engine, the voting/consensus
mechanics, the anti-bot defenses, and the transparency systems (modlog,
analytics). You'll never be short of something to learn.

### Chapter 2: The Stack, Chosen for Reasons

Here is the architecture you'll be working inside, drawn as a picture:

```
┌────────────────────────────────────────────────────────┐
│  Browser (SvelteKit 5 SPA, one origin)                │
└──────────────┬─────────────────────────────────────────┘
               │ HTTP /api/*
┌──────────────▼─────────────────────────────────────────┐
│  Axum 0.8 (Rust) — the whole backend in one binary     │
│  ├─► PostgreSQL 16 + pgvector   (metadata, social,     │
│  │                             search, recs)           │
│  ├─► Redis                     (rate limits,           │
│  │                             shadowban, PoW)         │
│  ├─► Ollama                    (embeddings, consensus, │
│  │                             ask-the-archive, tags)  │
│  ├─► Scraper subsystem         (native scrapers +      │
│  │                             FanFicFare catch-all)   │
│  └─► EPUB/TXT/MD builder + Calibre sidecar (MOBI/PDF)  │
└────────────────────────────────────────────────────────┘
```

Let's justify each piece, because you'll defend these choices in code review
and in architecture discussions.

**Rust + Axum.** One static binary. No runtime, no interpreter, no garbage
collector pauses. Memory-safe without a garbage collector. Axum 0.8 is a
modern, well-loved web framework built on tokio and hyper. Its three super
powers for us:

1. **Extractors** — `State<T>`, `Json<T>`, `Query<T>`, `Path<T>` let a
   handler declare exactly what it needs and Axum parses it for you.
2. **`IntoResponse`** — every handler returns something that *becomes* an
   HTTP response; errors included.
3. **Middleware** — we layer tracing, rate limiting, CORS, and the usage
   analytics recorder around the router in a composable way.

**PostgreSQL 16 + pgvector.** One database for everything relational
(users, works, bookmarks, comments, votes) *and* for vector similarity (the
roadmap consensus embeds feature ideas into 768-dimensional vectors and
clusters them by cosine distance). Using pgvector instead of a separate
vector store keeps operations simple: one Postgres instance, one backup,
one mental model.

**Redis.** In-memory speed for the hot paths: token-bucket rate limiting
(the `limiter/` module), shadowban sets, and proof-of-work challenge solves.
Redis is also where the bookmark-import worker parks its blocking BRPOP
wait — a detail that matters later when we talk about health checks.

**Ollama.** Local LLM inference, no API keys, no data leaving the box. The
deploy machine runs Ollama with `lfm2.5:8b` as the canonical default model
(it won a benchmark at ~21 tok/s for prose). Ollama powers: embeddings for
roadmap clustering and rec strategies, the auto-tagger, translations, and
the "Ask the Archive" natural-language search.

**SvelteKit 5.** The SPA is a static build (adapter-static) served by the
same Axum binary. That means one origin, no CORS config, and simple deploys:
build the frontend, build the backend, run one process. Svelte 5's runes
system (the `.svelte.ts` files you'll see) is a pleasant modern reactive
model.

> 💡 **Key concept — one binary, one origin.** The backend serves the API
> *and* the built SPA. This single choice removes an entire class of
> deployment and CORS problems. Keep it that way.

### Chapter 3: The Repository Map

Before we run anything, let's learn to navigate. You should be able to close
this book and walk a newcomer through the repo from memory after this
chapter.

```
/personal/documents/code/rust/fichub
├── src/                      # Rust backend
│   ├── main.rs               # binary entry (tokio, tracing)
│   ├── lib.rs                # library root (module tree)
│   ├── server.rs             # AppState + build_router + run()
│   ├── config.rs             # every env knob
│   ├── error.rs              # AppError → IntoResponse
│   ├── routes/               # one file per API area (40+ files)
│   ├── scrape/               # scraper subsystem (registry + sites/)
│   ├── export/               # EPUB/HTML/TXT/MD builders
│   ├── search/               # boolean query parser + suggest
│   ├── recommender/          # pluggable rec strategies + worker
│   ├── body_cache.rs         # JSON blob store on disk
│   ├── modlog.rs             # transparent moderation log
│   ├── cache/                # disk cache + export semaphores
│   ├── limiter/              # tiered rate limiting (Redis)
│   ├── services/             # pow, ollama, etc.
│   └── db/                   # pool, models, the big queries module
├── migrations/               # 34 numbered SQL migrations (1-34)
├── frontend/                 # SvelteKit 5 SPA
│   └── src/
│       ├── routes/           # SvelteKit routes (pages)
│       ├── lib/              # api client, stores, i18n, components
│       └── static/           # manifest, sw.js, docs (mdbook output)
├── tests/                    # Rust integration tests (DB-gated)
├── docs/                     # ROADMAP, STATUS, src/ (mdbook chapters)
├── books/                    # tutorial books (this one lives here)
└── qa/                       # nightly QA harness (playwright)
```

A few neighborhoods deserve immediate attention:

- **`src/routes/`** — this is where 80% of your day-to-day work happens.
  One file per API area: `auth.rs`, `search.rs`, `export.rs`, `requests.rs`,
  `admin.rs`, `modlog.rs`, `analytics.rs`, `curator_content.rs`, and more.
  Each file is a set of `pub async fn` handlers plus their tests.
- **`src/scrape/`** — the scraper registry and one file per site adapter.
- **`src/recommender/`** — the pluggable recommendation strategy registry,
  the legacy co-occurrence engine, and the background worker.
- **`migrations/`** — the schema story, one file per change, applied in
  order at boot.
- **`frontend/src/routes/`** — mirrors the API: `/search`, `/read/[urlId]`,
  `/admin/*`, `/modlog`, `/requests`, etc. Each route folder has
  `+page.svelte` (the page) and often a `page.test.ts` (vitest).
- **`tests/`** — DB-gated Rust integration tests that exercise the real
  server with a real Postgres.

The two-crate trick deserves its own callout. `src/main.rs` is the *binary
crate* — a thin entry point. `src/lib.rs` is the *library crate* — everything
real. Integration tests in `tests/` import `fichub::...` from the library,
so they exercise real code paths, not a fake. This is why `main.rs` is
almost boring: by design.

> 🧪 **Try it:** open `src/lib.rs` and count the module declarations. Then
> open `src/server.rs` and find `build_router`. That function is your feature
> map — every route in the product is listed there.
>
> ⚠️ **Watch out:** `frontend/src/routes/` has a `+layout.svelte` and
> `+layout.ts` at the root, plus `admin/` sub-routes that require role
> gates. Don't confuse a frontend route folder with a backend route file —
> they're parallel but separate worlds.

### Chapter 4: Running It For the First Time

Let's get the thing running. This is the moment where the abstract map
becomes a live system.

**Prerequisites:** Rust toolchain (stable), Node 20+, PostgreSQL 16, Redis,
and Ollama (optional for dev, required for AI features).

```bash
# from the repo root
cp .env.example .env          # then edit with your DB/Redis URLs
cargo run                     # backend on :8000
```

In a second terminal:

```bash
cd frontend
npm install                   # first time only
npm run dev                   # SPA on :5173, proxies /api to :8000
```

The backend serves the built SPA too (ServeDir fallback), so for production
you only run the Rust binary. In dev, Vite handles the SPA and proxies
`/api/*` to the backend. Either way, your API is at `http://localhost:8000`.

Let's verify the backend is alive:

```bash
curl localhost:8000/api/health
# {"status":"ok","db":true,"redis":true,"version":"0.2.0"}
```

That health endpoint (`src/routes/health.rs`) is your first real code: it
checks the database pool and the dedicated `health_redis` connection and
reports booleans. We'll dissect why it uses a *dedicated* Redis connection
in Chapter 6 — for now, just know it's deliberate.

`src/config.rs` is the single source of config truth: every environment
variable is read there into one `Config` struct, then shared everywhere via
`AppState`. If a feature needs a knob, it goes through `Config`. You'll see
`DATABASE_URL`, `REDIS_URL`, `PORT`, `BODY_CACHE_DIR`, `REC_ENGINE_MODE`,
`AGENT_ENABLED`, and dozens more — all parsed in one place.

> 🧪 **Try it:** change `PORT=8001` in `.env`, restart, and confirm the
> server listens on 8001. Then change it back.
>
> ⚠️ **Watch out:** cargo builds are slow the first time (2–5 minutes for
> the full dependency tree). On this dev box, builds go to
> `/media/alvaro/cargo-target-sh` — **never let cargo build on
> `/home/alvaro`** (NFS/space constraints). If you see disk-full or NFS
> errors, the target dir is the first thing to check.
>
> 💡 **Key concept:** one binary serves both API and SPA. `Config` is the
> single source of truth for behavior. Health checks hit a dedicated Redis
> connection. Three ideas that structure everything else in this book.

### Chapter 4A: The Unified Works Model — One Story, Many URLs

Before you touch any data, you must understand FicHub's central
abstraction: **the work vs the source.**

A *work* is the abstract story: "Draco Malfoy and the Mortifying Ordeal of
Being in Love" by isthisselfcare. A *source* (fic_info row) is one URL
where that story appears: the AO3 copy, the FFN copy, maybe a RoyalRoad
copy. The same story posted on multiple sites appears as **one canonical
entry** with multiple sources.

Why does this matter?

- **Deduplication.** If a user bookmarks the AO3 copy and another user
  bookmarks the FFN copy, the system understands they're the same story —
  via the shared `work_id`.
- **Auto-merge.** When a new source is scraped, the system tries to match
  it to an existing work: exact title + author with word-count tolerance.
  Match → attach as another source. No match → create a new work.
- **Curator proposals.** When auto-merge is wrong (or missed), curators can
  propose a merge/split with voting (the community-curation model).

The schema:

```sql
works      (id, canonical_title, canonical_author, description, ...)
fic_info   (id=url_id, work_id FK, site_domain, title, author, words, ...)
```

`fic_info.id` is the url_id you'll see everywhere (e.g. `ao3_21845264`).
`fic_info.work_id` links a source to its abstract work.

The practical rule for junior devs: **when you store anything about "a
fic," store the url_id (source) or the work_id (abstract story) depending
on what you mean.** Bookmarks, ratings, and reviews attach to works or
sources consistently — check the existing tables before choosing.

> 🧪 **Try it:** in the DB, run a query joining `fic_info` to `works` and
> find a story that has more than one source. See how they share a work_id.
>
> ⚠️ **Watch out:** don't conflate `fic_info.id` (url_id, per-source) with
> `works.id` (per-story). Mixing them up is a classic junior bug here.
>
> 💡 **Key concept:** work = story, fic_info = URL. The unified model is
> what makes cross-site dedup possible.

### Chapter 4B: Export Formats — What the Pipeline Produces

The export subsystem (`src/export/`) builds the files users download:

- **EPUB** — the flagship. Built in pure Rust with the `epub-builder`
  crate: a zip of XHTML chapters + metadata + TOC.
- **HTML** — a single self-contained HTML file (styles inline, images
  embedded or linked).
- **TXT / Markdown** — plain-text derivations for e-ink readers and
  note-takers.
- **MOBI / PDF / AZW3** — via the **Calibre sidecar**: a Docker container
  running `ebook-convert` that converts the EPUB to the target format.

The export handler picks the format, streams the file with the right
`Content-Type` and a clean `Content-Disposition` filename, and records the
event (the analytics "action" event, plus `request_log` for anti-bot).

Why pure Rust for EPUB but Calibre for MOBI? EPUB is a simple zip
container — easy to generate directly with no external deps. MOBI/PDF/AZW3
are proprietary/complex formats — Calibre's `ebook-convert` is battle-
tested at converting them. The sidecar keeps the Rust binary small and the
formats reliable.

> 🧪 **Try it:** export the same fic as EPUB and as TXT from the dev
> server. Compare the file sizes and structures.
>
> ⚠️ **Watch out:** the Calibre sidecar must be running (Docker) for
> MOBI/PDF/AZW3. EPUB/HTML/TXT/MD work without it. Health checks don't
> cover Calibre — watch logs.
>
> 💡 **Key concept:** pure Rust where cheap, battle-tested sidecar where
> complex. The export pipeline is cache-first and format-pluggable.

---



---

<!-- part: 02-server-core.md -->


## Part 2 — The Server Core

### Chapter 5: Entry Point & Module Tree

Open `src/main.rs`. It is deliberately small:

```rust
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let config = fichub::config::Config::from_env();
    fichub::server::run(config).await;
}
```

That's essentially the whole binary. Three lines of substance:

1. **`#[tokio::main]`** — the attribute that turns `main` into a tokio
   async runtime. Tokio is the async executor that makes thousands of
   concurrent connections possible on a handful of OS threads.
2. **`tracing_subscriber::fmt().init()`** — sets up structured logging.
   You'll see `tracing` and `log` macros throughout the codebase; this line
   is why `RUST_LOG=debug` actually shows you things.
3. **`Config::from_env()` then `server::run(config)`** — the entire
   startup flow lives in the library crate, not here.

Why keep `main.rs` boring? Because a boring entry point is a *readable*
entry point. Anyone — including a junior dev on day one — can see exactly
what the binary does: read config, run the server. All the interesting work
is importable and testable.

Now open `src/lib.rs`. It declares the module tree:

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod limiter;
pub mod modlog;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod services;
pub mod works;
// ... plus body_cache, heal, etc.
```

Two things to notice:

- **Everything is `pub`** — that's what makes `fichub::routes::admin::...`
  reachable from integration tests.
- **The module list is your table of contents.** If you want to know where
  something lives, this list plus `src/routes/mod.rs` is your index.

The crate is a *library + a thin binary* — the "two-crate trick." Integration
tests in `tests/` use the library crate's public API, so they test the real
server, the real config, the real queries. This is a hugely underrated
design decision: it means you can write end-to-end-ish tests in plain Rust
with `cargo test`, no separate test harness.

> 💡 **Key concept:** the two-crate split. `main.rs` = boot. `lib.rs` =
> everything. Tests import the library. If you're tempted to put logic in
> `main.rs`, don't — put it in the library so it's testable.

### Chapter 6: AppState — the Dependency-Injection Heart

Every handler in this codebase receives the same shared state. It's defined
in `src/server.rs`:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: Arc<RedisPool>,
    pub health_redis: RedisPool,      // dedicated conn for health checks
    pub http_client: reqwest::Client,
    pub scraper_registry: ScraperRegistry,
    pub cache_semaphores: CacheSemaphores,
    // ... heal state, suggest cache, worker handles, etc.
}
```

Handlers grab it with Axum's `State` extractor:

```rust
pub async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let db_ok = state.db.acquire().await.is_ok();
    // ...
}
```

Why `Arc<AppState>`? Because every request shares the same state; `Arc`
(the atomic reference-counted pointer) lets thousands of concurrent
requests hold a cheap shared reference. Nothing is per-request except the
extractor itself.

A few fields deserve explanation:

- **`config`** — the frozen config snapshot. Handlers read settings from
  here rather than re-reading env vars.
- **`db`** — the SQLx connection pool. `PgPool` internally manages a set of
  connections; `state.db` is cheap to clone (it's an `Arc` under the hood).
- **`redis` vs `health_redis`** — here's the story I promised in Chapter 4.
  The **bookmark-import worker parks an unbounded BRPOP on the shared
  `state.redis` connection**. BRPOP blocks the connection waiting for list
  items. If a health check PINGs *that* connection, the PING queues behind
  the BRPOP and times out — falsely reporting "Redis down." That's why
  `health_redis` exists: a dedicated connection for health checks, never
  touched by the worker. **Lesson: never PING the shared connection from a
  health check.**
- **`http_client`** — a shared `reqwest::Client` with connection pooling,
  used by scrapers and the LLM feature.
- **`scraper_registry`** — the list of site adapters (Part 4 covers this in
  depth).
- **`cache_semaphores`** — the bounded map that prevents duplicate
  concurrent exports (Part 5).

`src/server.rs` also contains:

- **`run()`** — builds the pool, connects Redis, spawns background workers
  (recommender worker, bookmark importer), then serves.
- **`build_router()`** — the big route-registration function. Read it as
  your feature map: every endpoint in the product is listed there, grouped
  by area.

> 🧪 **Try it:** `grep -n "route(" src/server.rs | wc -l` — count the
> routes. Then open `build_router()` and find the route for the feature you
> care about most.
>
> ⚠️ **Watch out:** when you add a new subsystem, it goes into `AppState`
> — and then **every** literal `AppState` construction in tests must be
> updated. The skill notes literally say "every literal construction in
> tests needs the new field or they won't compile." This is annoying but
> safe: the compiler tells you exactly which test files need the field.
>
> 💡 **Key concept:** dependency injection via a shared state struct +
> `State` extractor. New subsystems = new `AppState` field + new handler
> access. Keep the pattern; don't invent global singletons.

### Chapter 7: The Router — Every Route Registered

`build_router()` in `src/server.rs` chains `.route()` calls. The path syntax
uses `{param}` (Axum 0.8's syntax). Examples you'll see constantly:

```rust
.route("/api/epub", get(export::epub_handler))
.route("/api/search", get(search::search_handler))
.route("/api/requests/{id}/candidates", get(requests::candidates))
.route("/api/works/{url_id}/also-bookmarked", get(works::also_bookmarked))
```

Two things to internalize:

1. **Order matters.** Some routes are registered before the `ServeDir`
   fallback that serves the SPA. API routes must win over the fallback. If
   you add a route and the SPA swallows it, check the order.
2. **The SPA fallback.** Any path that doesn't match an API route gets
   `index.html` — that's how `/read/xyz` works in the browser while
   `/api/*` stays JSON. This is the "one origin" idea made concrete.

Route naming conventions:

- `/api/<area>` for the main endpoints.
- `/api/v1/...` for a few versioned legacy endpoints (e.g.
  `/api/v1/works/{url_id}/also-bookmarked`).
- `/api/admin/<area>` for admin-only endpoints (role ≥ 10).
- Static files served from `frontend/build` via ServeDir.

When you add a feature, you typically touch four places: `AppState` (if new
state), a handler in `src/routes/<area>.rs`, a `.route()` in
`build_router()`, and a frontend page. We'll do this end-to-end in
Chapter 22.

> 🧪 **Try it:** add a trivial route in your head: `GET /api/ping` returning
> `{"pong":true}`. Find where you'd register it. (Hint: `build_router` +
> a handler in `src/routes/health.rs` or a new file.)
>
> ⚠️ **Watch out:** path param syntax changed across Axum versions. This
> repo uses `{param}` (Axum 0.8), not `:param`. If you copy old Axum code
> with `:id`, it won't compile.

### Chapter 8: Errors That Don't Crash

Open `src/error.rs`. This file defines the backbone of every handler's
return type.

```rust
pub enum AppError {
    NotFound,
    BadRequest(i32, String),      // custom code + message
    Unauthorized,
    Database(sqlx::Error),
    Network(String),
    RateLimitedJson(serde_json::Value),  // 429 with a custom JSON body
    // ... more variants
}

pub type AppResult<T> = Result<T, AppError>;
```

The magic is the `IntoResponse` impl:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // translate AppError → HTTP status + JSON body {err, msg}
    }
}
```

Handlers return `AppResult<T>`. The `?` operator converts anything that
implements `From<AppError>`-compatible conversions — sqlx errors, reqwest
errors, etc. — via `From` impls in this same file. **A handler can never
panic from a failed DB query**: it becomes a clean JSON error instead.

The JSON error shape is the same everywhere:

```json
{"err": -6, "msg": "network error: ..."}
```

Negative error codes are custom (e.g. `-6` network, `-403` admin access
required, `-429` rate limited). The frontend's API client checks `err != 0`
and shows `msg`.

Three rules to live by when you touch errors:

1. **Return `AppResult<T>`, let `?` do the work.** Don't hand-map every
   failure; add a `From` impl when you introduce a new error source.
2. **Details in, generic out.** Log the real error (tracing), return a safe
   message. Never leak internal paths, credentials, or SQL.
3. **Custom codes are a contract.** The frontend and the API tests depend
   on specific `err` values. Change codes deliberately, with tests.

> 🧪 **Try it:** `curl localhost:8000/api/meta?q=https://archiveofourown.org/works/21845264`
> — a real scrape. Then try a bogus URL and watch the `{err, msg}` JSON.
> Notice the shape stays consistent.
>
> ⚠️ **Watch out:** the `?` operator needs the `From` impls. If you return
> `AppResult` from a handler that calls a function returning `sqlx::Error`,
> make sure `From<sqlx::Error> for AppError` exists (it does).
>
> 💡 **Key concept:** errors are values, not panics. `AppError` + `From` +
> `?` + `IntoResponse` = handlers that read like happy-path code but fail
> gracefully everywhere.

### Chapter 8A: Middleware — What Happens Before/After Handlers

Axum middleware wraps the router and runs before (and after) your handler.
FicHub layers several:

1. **Tracing** — logs every request with method, path, status, duration.
2. **Usage analytics recorder** (`from_fn_with_state`) — classifies each
   path as `view` or `action` and writes to `usage_events` (with the
   anonymous X-Client-ID). Health = view; meta = action.
3. **Rate limiting** — the tiered limiter (per-IP + per-client token
   buckets).
4. **CORS** — though with the one-origin architecture, CORS is mostly
   moot in production.
5. **The SPA fallback** — ServeDir at the end.

Middleware order matters: tracing first (see everything), then analytics
(measure everything), then rate limiting (protect), then routes, then the
fallback.

When you add middleware, think about *what it needs to know*: request
metadata (path, method, headers) → run before; response status → run
after. The `from_fn_with_state` middleware pattern in this repo is a good
template:

```rust
pub async fn track_usage(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let client_id = req.headers().get("x-client-id")...;
    let resp = next.run(req).await;
    // classify + record
    state.analytics.record(&path, &client_id, &resp).await;
    resp
}
```

> 🧪 **Try it:** read the analytics middleware in `src/routes/analytics.rs`
> and identify the view vs action classification.
>
> ⚠️ **Watch out:** middleware runs on EVERY request — keep it cheap. A
> slow middleware becomes a global latency tax.
>
> 💡 **Key concept:** middleware is where cross-cutting concerns live:
> logging, analytics, rate limiting. Order = visibility → measurement →
> protection.

### Chapter 8B: The Health Check Dissected

`src/routes/health.rs` is small but teaches several lessons. The handler:

1. Acquires a DB connection (`state.db.acquire()`) → `db: bool`.
2. PINGs the dedicated `health_redis` connection → `redis: bool`.
3. Returns `{"status":"ok","db":true,"redis":true,"version":"0.2.0"}`.

Why the dedicated Redis connection (again, because it's important): the
bookmark-import worker parks an unbounded BRPOP on the shared `state.redis`
connection. A PING on that connection would queue behind the BRPOP and
time out — falsely reporting Redis down. `health_redis` is a separate
connection reserved for health checks. **Rule: health checks must never
share connections with workers.**

The version string comes from the crate version (Cargo.toml). The status
is "ok" only when both DB and Redis report healthy; degraded components
flip their boolean so monitoring can see exactly what's wrong.

> 🧪 **Try it:** read `health.rs` and trace where `version` comes from.
> Then run `curl localhost:8000/api/health` and correlate the booleans
> with the actual DB/Redis state.
>
> ⚠️ **Watch out:** never add slow checks to the health endpoint — it's
> polled by uptime monitors and the deploy script. Keep it fast.
>
> 💡 **Key concept:** the health check is the contract with monitoring:
> fast, honest, and connection-safe (dedicated `health_redis`).

---



---

<!-- part: 02a-deep-dives.md -->


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



---

<!-- part: 03-data-layer.md -->


## Part 3 — Configuration & Data Layer

### Chapter 9: config.rs — Every Knob in One Place

`src/config.rs` is the file to read when you ask "how do I turn on X?". It
parses environment variables into one `Config` struct in `from_env()`.

```rust
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub body_cache_dir: PathBuf,        // default /public/literature/fichub/bodies
    pub rec_engine_mode: RecEngineMode, // legacy | pluggable
    pub rec_shadow_mode: bool,
    pub agent_enabled: bool,            // self-healing agent autonomy
    // ... dozens more
}
```

The patterns you'll see throughout:

- **Default then override.** `Config::from_env()` reads `std::env::var`,
  and when a var is missing, falls back to a sane default. Example:
  `BODY_CACHE_DIR` defaults to `/public/literature/fichub/bodies`.
- **Typed knobs.** Enums like `RecEngineMode` are parsed from strings with a
  `FromStr` impl. Booleans are parsed with a helper that accepts
  `true/false/1/0`.
- **One struct, shared everywhere.** Once built, `Config` is frozen inside
  `AppState` and handlers read `state.config.x`. No handler re-reads env
  vars.

Why centralize? Because configuration is *behavior*. If you want to know
what the server will do in production, you read `Config`. If you want to
test a different behavior, you construct a `Config` with the values you
want — no env mutation needed (well, almost; the tests use an `ENV_LOCK`
mutex for the rare cases that do touch env).

The test pattern in `config.rs` is worth stealing for your own features:

```rust
// ENV_LOCK guards env mutation so parallel tests don't stomp each other
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
```

> 🧪 **Try it:** read `from_env()` and list five knobs you didn't know
> existed. Then grep the codebase for one of them to see where it's read.
>
> ⚠️ **Watch out:** don't add a new `std::env::var(...)` call in a handler.
> That's the "rogue env read" anti-pattern — it bypasses `Config` and makes
> testing harder. Add the field to `Config` and thread it through
> `AppState`.
>
> 💡 **Key concept:** configuration is behavior; centralize it. `Config` is
> the contract between the deployment environment and the running server.

### Chapter 10: PostgreSQL + SQLx

Open `src/db/mod.rs`. It builds the pool and runs migrations on boot:

```rust
let pool = PgPoolOptions::new()
    .max_connections(...)
    .connect(&config.database_url).await?;

sqlx::migrate!().run(&pool).await?;   // applies migrations/1..N
```

Two ideas matter here.

**The pool.** `PgPool` manages a set of live connections. Handlers call
`state.db.clone()` (cheap) or `&state.db` and the pool hands out a
connection from its set. This is why the server can handle many concurrent
requests without opening a connection per request. Pool sizing is a real
tuning knob: too few connections and you serialize on the DB; too many and
you exhaust Postgres's `max_connections`.

**Migrations at boot.** `sqlx::migrate!()` embeds `migrations/*.sql` into
the binary at compile time. On startup, it compares the embedded list
against the `_sqlx_migrations` table in the database and applies anything
new, in order. This is why deploys "just work": the new binary carries its
own schema changes. Never run migrations by hand in production — the binary
does it, transactionally per file.

`src/db/models.rs` defines the `FromRow` structs that match table shapes:

```rust
#[derive(FromRow)]
pub struct FicInfo {
    pub id: String,          // url_id
    pub site_domain: String,
    pub title: String,
    pub author: String,
    // ...
}
```

One gotcha to internalize: this repo uses the **runtime query forms**
(`sqlx::query()`, `query_as()`, `query_scalar()`) heavily rather than the
compile-time-checked macros (`query!`, `query_as!`). The trade-off:
flexibility (dynamic SQL, fewer macro headaches) for less compile-time
safety. Column names in the SQL must match the `FromRow` struct fields at
runtime, or you get a runtime decode error. **When you change a table, keep
the model and the migration in lockstep.**

> 🧪 **Try it:** connect to the dev database and inspect the
> `_sqlx_migrations` table. You should see 34 rows, in order.
> `psql -h localhost -U fichub -d fichub -c "SELECT version, description FROM _sqlx_migrations ORDER BY version;"`
>
> ⚠️ **Watch out:** on this setup, Postgres tables are owned by the `fichub`
> role — if you create tables in a scratch DB with a different owner, you'll
> hit permission errors. Use `ALTER OWNER TO fichub` after manual DDL.
>
> 💡 **Key concept:** pool + embedded migrations = deployable schema. The
> binary is the source of truth for schema changes.

### Chapter 11: The Migrations — 34 Files, One Schema Story

`migrations/` is numbered `001_*.sql` … `034_*.sql`. Together they tell the
entire history of the product's data model. Reading them in order is like
reading the product's diary. The story they tell:

- **001–004:** the core — `fic_info` (sources), `works`, tags, users,
  bookmarks, and the generated tsvector column + trigger for search.
- **005–010:** search analytics, ratings/reviews, comments, and the
  `search_queries` table.
- **011:** roadmap consensus — `feature_clusters` with the pgvector
  embedding column (768 dims, HNSW index).
- **013:** user roles/reputation.
- **016–018:** roadmap seeding + fic requests (the prompt board).
- **020:** series & authors.
- **023–025:** translations, rating/score fixes, extra metadata.
- **029–030:** self-healing — `scrape_failures` + `agent_runs`.
- **033:** usage analytics — `usage_events`.
- **034:** modlog.

Let's look at two representative patterns.

**Upsert-style DDL.** Many migrations create tables designed for idempotent
writes: unique constraints plus `ON CONFLICT` handling in the queries
(Chapter 12). For example, fic metadata is upserted so re-scraping a URL
updates rather than duplicates.

**Generated columns + triggers.** Migration 001 creates a tsvector column
for full-text search:

```sql
ALTER TABLE fic_info ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (... ) STORED;
CREATE INDEX ... USING GIN (search_vector);
```

The DB maintains the search index as rows change — no application-side
sync.

**Rules for working with migrations:**

1. **Never edit an applied migration.** Add `035_...` instead. The
   `_sqlx_migrations` table records a checksum of each applied file; editing
   an applied one causes a checksum mismatch error on the next boot.
2. **One logical change per migration.** Numbering is sequential; if two
   people add migrations concurrently, the numbers conflict — coordinate.
3. **Down migrations don't exist here.** The project doesn't roll back; it
   moves forward. If a migration is wrong, add a corrective one.

> 🧪 **Try it:** `ls migrations/ | head -20` then `ls migrations/ | tail -6`.
> You'll see the progression from core tables to analytics and modlog.
>
> ⚠️ **Watch out:** the `search_vector` column and its trigger are
> *generated* — you can't insert into them directly. If a query fails with
> "column ... is generated," that's why.
>
> 💡 **Key concept:** migrations are the schema's version history, applied
> transactionally at boot, never edited after the fact.

### Chapter 12: queries.rs — The Big Query Module

`src/db/queries.rs` is the workhorse: roughly 2,700 lines of SQL functions.
The patterns matter more than any individual query, so let's study the
three you'll see everywhere.

**Pattern 1 — Upsert:**

```rust
sqlx::query(
    "INSERT INTO fic_info (id, site_domain, title, author, ...)
     VALUES ($1, $2, $3, ...)
     ON CONFLICT (id) DO UPDATE SET
       title = EXCLUDED.title, author = EXCLUDED.author, ..."
)
```

Scrapes are idempotent: running the same URL twice updates, never
duplicates. This is the foundation of the "cache of all gathered
fanfiction" idea.

**Pattern 2 — RETURNING:**

```rust
let id: i32 = sqlx::query_scalar(
    "INSERT INTO works (...) VALUES (...) RETURNING id"
).fetch_one(&pool).await?;
```

Insert and get the id in one round-trip. Saves a `SELECT` and a race.

**Pattern 3 — inet binding:**

```rust
.bind(ip_string)   // SQL says: ... WHERE client_ip = $N::inet
```

Postgres `inet` columns are bound as strings with an explicit `::inet` cast
in the SQL. Don't try to bind an IP as an integer or a CIDR type.

Three lessons for working here:

1. **Grep before you write.** The module is huge. The function you need may
   already exist — `get_fic_info`, `upsert_fic_info`, `insert_request_log`,
   `lookup_alias`, and dozens more. Search first.
2. **Add near siblings.** When you add a query, put it next to related ones
   and follow the naming convention (`<verb>_<noun>`).
3. **Parameterize everything.** No string interpolation into SQL. `$1, $2,
   ...` binds prevent SQL injection and keep types explicit.

> 🧪 **Try it:** find `upsert_fic_info` and trace what happens when you
> export a fic twice. The second export hits the body cache, but the
> metadata upsert still runs — safely updating.
>
> ⚠️ **Watch out:** runtime `query()` means typos surface at runtime, not
> compile time. Always exercise new queries with a real test (DB-gated or
> via an endpoint) before declaring done.
>
> 💡 **Key concept:** one query module keeps SQL discoverable and
> consistent. Upserts + RETURNING + parameterized binds are the house
> style.

### Chapter 12A: A Walk Through the Key Queries

Let's read a few real functions from `queries.rs` so the patterns click.

**`get_fic_info`** — the metadata lookup. It fetches one source row by
url_id:

```rust
pub async fn get_fic_info(pool: &PgPool, url_id: &str)
    -> Result<Option<FicInfo>, sqlx::Error>
{
    sqlx::query_as::<_, FicInfo>(
        "SELECT id, site_domain, title, author, description, words,
                status, updated_at, ...
         FROM fic_info WHERE id = $1"
    )
    .bind(url_id)
    .fetch_optional(pool)
    .await
}
```

Note `fetch_optional` — it returns `Option`, not a row, so a missing fic is
`None`, not an error. That's the house pattern for "might not exist."

**`insert_request_log`** — writes a request/telemetry row. Notice the
`::inet` cast we mentioned:

```rust
sqlx::query(
    "INSERT INTO request_log (url_id, ip, client_id, etype, ...)
     VALUES ($1, $2::inet, $3, $4, ...)"
)
.bind(url_id)
.bind(ip_string)   // string → ::inet in SQL
.bind(client_id)
// ...
```

**`upsert_fic_info`** — the idempotent write. `ON CONFLICT (id) DO UPDATE
SET ...` with `EXCLUDED` referencing the would-be-inserted row. This is
why re-scraping never creates duplicates.

**`lookup_alias`** — tag alias lookup. Small, but shows the join style
used across the tag system.

The takeaways: `fetch_optional` for maybe-rows, `::inet` for IPs, `ON
CONFLICT` for idempotence, and every query parameterized with `$N`.

> 🧪 **Try it:** open `queries.rs` and find `get_fic_info`, then
> `upsert_fic_info`. Read them side by side — the select and the upsert
> of the same table. That pair is the backbone of the archive.
>
> ⚠️ **Watch out:** `fetch_optional` vs `fetch_one` — use `fetch_one` only
> when the row MUST exist, and handle `RowNotFound` via `?` (it converts
> to `AppError::NotFound` through the `From` impl).
>
> 💡 **Key concept:** the query module's style is: parameterized,
> idempotent, optional-aware. Match it and your queries will fit right in.

### Chapter 12B: The Model Types — FicInfo, WorkRow & Friends

`src/db/models.rs` defines the `FromRow` structs that map rows to Rust
types. The central ones:

```rust
#[derive(FromRow, Serialize)]
pub struct FicInfo {
    pub id: String,             // url_id
    pub site_domain: String,
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub words: i64,
    pub status: String,
    pub updated_at: chrono::NaiveDateTime,
    // ...
}
```

Notice `description: Option<String>` — nullable columns become `Option`.
That's a SQLx `FromRow` rule: if the column can be NULL, the field must be
`Option`. Get this wrong and you get a runtime decode error.

The **works model** is separate from fic_info because of the unified works
architecture: a `work` is the abstract story, and each `fic_info` row is a
*source* (URL) for it. The same story on AO3 and FFN is one work with two
sources. Fields like `work_id` on fic_info link them.

> 🧪 **Try it:** find `WorkRow` and compare it with `FicInfo`. Identify
> which fields are per-source vs per-work.
>
> ⚠️ **Watch out:** nullable columns MUST be `Option<T>` in the struct.
> When you add a nullable column, update the model in the same change.
>
> 💡 **Key concept:** models mirror the schema; `Option` marks NULLable;
> work vs fic_info is the abstraction boundary.

### Chapter 12C: A Tour of the API Areas

Now that you know the data layer, here's the tour of `src/routes/` you'll
reference constantly:

- **`auth.rs`** — register/login, JWT issue/verify, `AuthUser`.
- **`search.rs`** — the search handler (parser → SQL) + Ask the Archive.
- **`meta.rs`** — `/api/meta`: scrape metadata for a URL.
- **`export.rs`** — the export pipeline (URL → EPUB/HTML/...).
- **`download.rs` / `kindle.rs` / `updates.rs`** — download flows, Kindle
  email, updates feed.
- **`reader.rs`** — chapter content for the reader.
- **`social.rs` / `comments.rs` / `reviews.rs` / `follows.rs`** — the
  social layer.
- **`requests.rs`** — the fic-request board (create/answer/vote/accept/
  candidates).
- **`lists.rs` / `shelves.rs` / `series.rs` / `authors.rs`** — reading
  lists, shelves, series & author pages.
- **`roadmap.rs`** — the consensus arena.
- **`admin.rs`** — admin endpoints (users, bans, stats, bots, analytics).
- **`modlog.rs`** — the public moderation log.
- **`analytics.rs`** — usage analytics.
- **`curator_content.rs`** — peer-voted body fixes.
- **`auto_tag.rs` / `locales.rs`** — AI features (auto-tagger,
  translations).
- **`pow.rs` / `honeypot.rs`** — anti-bot.
- **`health.rs`** — liveness.
- **`notifications.rs` / `badges.rs` / `quests.rs` / `trending.rs`** —
  engagement features.

Each file follows the same shape: `pub async fn` handlers, `AppResult<T>`
returns, `State<Arc<AppState>>` extractor, and (for admin/curator actions)
a modlog record.

> 🧪 **Try it:** pick the feature you care most about and open its route
> file. Read one handler end to end: extractors → logic → response.
>
> ⚠️ **Watch out:** some handlers are long. Read the return type first,
> then the extractors, then skim the logic. You don't need every line to
> understand the flow.
>
> 💡 **Key concept:** one file per API area, one handler per endpoint,
> one pattern per handler. The tour above is your index.

---



---

<!-- part: 04-scraper.md -->


## Part 4 — The Scraper Subsystem

### Chapter 13: How FicHub Talks to Other Sites

Open `src/scrape/mod.rs`. This is where the "fetch fanfiction from the
internet" magic lives. It defines the shapes every site adapter must honor:

```rust
pub struct FicMetadata {
    pub title: String,
    pub author: String,
    pub description: String,
    pub words: i64,
    pub status: String,
    // ... chapters, updated, etc.
}

pub struct Chapter {
    pub title: String,
    pub body_html: String,
    // ...
}

pub enum ScrapeError {
    Network(String),
    NotFound,
    UnsupportedSite,
    // ...
}

#[async_trait]
pub trait SiteScraper {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str)
        -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, url: &str)
        -> Result<Vec<Chapter>, ScrapeError>;
}
```

The contract is beautifully simple: **can you handle this URL? If yes, give
me metadata and chapters.** Every site adapter implements this trait, and
the rest of the system doesn't care *how* the site is scraped — HTML
parsing, JSON API, FanFicFare subprocess — it only cares about the
`SiteScraper` interface.

The `ScrapeError` enum matters because callers map errors to user-facing
messages and telemetry. A `NotFound` (URL is valid-looking but the fic
doesn't exist) is very different from a `Network` error (upstream site is
down or blocking us). The self-healing system (Part 6) classifies scrape
failures into transient/blocked/structural/systemic — the error variants
feed that classification.

> 💡 **Key concept:** interface over implementation. `SiteScraper` is the
> seam between the site-specific world (HTML, cookies, bot walls) and the
> site-agnostic world (metadata, chapters, exports). New sites = new
> adapter = no changes to the rest of the system.

### Chapter 14: The Registry + `find_specific_or_fff`

The registry (`src/scrape/registry.rs`) owns the list of site adapters.
Two functions matter:

```rust
pub fn find_scraper(&self, url: &str) -> &dyn SiteScraper {
    // the naive version: return the FIRST scraper whose can_handle(url) is true
}

pub fn find_specific_or_fff(&self, url: &str) -> &dyn SiteScraper {
    // 1. try each NON-catch-all native scraper whose can_handle(url) is true
    // 2. fall back to FanFicFare (the catch-all)
}
```

Why does `find_specific_or_fff` exist? This is a real debugging story from
the project's history, and it's worth telling because it explains a subtle
bug class you'll encounter.

**The shadowing problem.** FanFicFare is the catch-all scraper: it accepts
*every* URL and hands it to a Python CLI that knows 100+ sites. In the
registry, FanFicFare is pushed **first** — it's the safety net. But if
`find_scraper` just returned the first match, FanFicFare would *always*
win, and the native scrapers (AO3, XenForo, etc.) would never run. That's
exactly what happened when TheForce.net was added: the native XenForo
scraper was registered, but `find_scraper` kept returning FanFicFare, which
didn't know the site — `UnknownSite` errors for every request.

The fix was `find_specific_or_fff`: **prefer a real native scraper, fall
back to FanFicFare.** Every route that scrapes — meta, export, kindle,
updates, download, recommender, fic_suggestions — now calls this function
instead of `find_scraper`.

The rule for adding a new site:

1. Implement `SiteScraper` for your site (Chapter 13's trait).
2. Add it to the registry **before** the FanFicFare catch-all.
3. If it's a *specific* site (AO3, XenForo, ...), `find_specific_or_fff`
   will prefer it automatically. Don't make your scraper accept every URL
   unless it truly is a catch-all — that shadows everyone else.

> 🧪 **Try it:** read `find_specific_or_fff` and list the sites it would
> prefer over FanFicFare. Then read the registry's construction to see the
> order.
>
> ⚠️ **Watch out:** a new scraper with `can_handle` that returns `true`
> for everything is a landmine. Keep `can_handle` precise (domain + path
> patterns).
>
> 💡 **Key concept:** catch-alls are dangerous when they shadow specific
> adapters. Prefer specificity, fall back to the catch-all.

### Chapter 15: The Body Cache — the Archive on Disk

`src/body_cache.rs` implements the "site as a cache" idea — the most
important architectural decision in the current codebase. Let's unpack it.

**Sharding.** Bodies are stored as JSON files under `BODY_CACHE_DIR`,
sharded by the first characters of the `url_id`:

```
BODY_CACHE_DIR/<2ch>/<2ch>/<url_id>.json
# e.g. url_id "xenforo_50062326" → xe/nf/.../xenforo_50062326.json
```

Sharding keeps any single directory small (a few hundred files max) so
filesystem lookups stay fast even with hundreds of thousands of fics.

**Versioning.** Each blob has a `current_version`. When a curator fixes a
body (Part 6), the version bumps — readers and exports use the corrected
content, and the old version is preserved for audit.

**The API:**

```rust
pub fn save_body(config, url_id, body) -> Result<(), CacheError>;
pub fn load_body(config, url_id) -> Result<Option<CachedBody>, CacheError>;
pub fn delete_body(config, url_id) -> Result<(), CacheError>;
pub fn save_html(config, url_id, html) -> Result<(), CacheError>;
```

**The flow.** In the export pipeline (Part 5), the body cache is checked
FIRST. A cache hit skips the network scrape entirely. A miss scrapes,
persists, and serves. Repeat exports are instant, and the archive is
resilient: even if AO3 blocks the server tomorrow, everything already
gathered keeps serving.

This is the concrete meaning of "the site should serve as a cache of all
gathered fanfiction." The database holds metadata; the filesystem holds
content. They're kept in sync by convention: scrape → persist body →
upsert metadata.

> 🧪 **Try it:** export a fic twice and time both. The second call should
> be dramatically faster (cache hit). Then look in `BODY_CACHE_DIR` for the
> sharded blob — you'll see the `xe/nf/...` structure.
>
> ⚠️ **Watch out:** body blobs are gitignored and live on the attached
> drive, NOT in the repo. Never commit them. If `BODY_CACHE_DIR` is
> unset in a test, it falls back to a temp dir — the tests use
> `BODY_CACHE_DIR=/tmp/fichub-body-verify`.
>
> 💡 **Key concept:** DB = metadata, filesystem = content. The cache is
> what makes FicHub a durable archive instead of a scraper-only proxy.

### Chapter 15A: The Site Adapters in Detail

Let's look at the actual adapters to see the range of strategies.

**AO3 (`sites/ao3.rs`).** The cleanest adapter. AO3 pages are
well-structured HTML. The scraper:

1. Extracts the work id from the URL with a regex:
   `r"/works/(\d+)"` → `21845264`.
2. Fetches the page with a normal User-Agent.
3. Parses metadata with the `scraper` crate (CSS selectors): title, author,
   summary, stats.
4. Parses chapter links and fetches each chapter's body.

AO3 also has **series pages** — the adapter can list all works in a
series (`list_series_works`), which is how series exports work.

**FanFicFare (`sites/fanficfare.rs`).** The catch-all. It shells out to
the FanFicFare Python CLI, which knows 100+ sites. The wrapper:

1. Builds a FanFicFare command line for the URL.
2. Runs it as a subprocess.
3. Parses the resulting metadata/HTML.

FanFicFare is powerful but: (a) it's a Python dependency, (b) it can fail
with `UnknownSite` for newer/lesser-known sites, and (c) it's slow. That's
why native adapters are preferred — see Chapter 14's `find_specific_or_fff`.

**XenForo (`sites/xenforo.rs`).** Forums (SpaceBattles, SufficientVelocity,
TheForce.Net) run XenForo. The adapter:

1. Recognizes the domain (`boards.theforce.net`, etc.).
2. Parses thread pages: `h1.p-title-value` for the title, `a.username`
   for authors, `article.message-body` for post bodies.
3. Each forum post becomes a chapter.

**The force.net story** is instructive: when TheForce.Net was added, the
native XenForo adapter was registered, but the app kept returning
FanFicFare `UnknownSite` errors because `find_scraper` returned the
catch-all first. The fix — `find_specific_or_fff` — is why the registry
prefers specificity. Also: forum threads have an OP announcement and
comments mixed in; the scraper returns all message bodies as chapters (a
known caveat), and curators fix the body via the peer-voted workflow.

**FFN / RoyalRoad / others.** Each site adapter implements the same
`SiteScraper` trait with site-specific selectors and URL patterns.

> 🧪 **Try it:** open `sites/ao3.rs` and `sites/xenforo.rs` side by side.
> Notice both implement the same trait, but the HTML parsing differs
> completely. That's the interface seam doing its job.
>
> ⚠️ **Watch out:** site HTML changes frequently. When an adapter breaks,
> it's usually a selector change — the classifier will label it
> "structural" (Chapter 26).
>
> 💡 **Key concept:** every site is a `SiteScraper`; the registry prefers
> specific adapters over the catch-all; HTML parsing is adapter-local.

### Chapter 15B: The FanFicFare Wrapper in Depth

`src/scrape/sites/fanficfare.rs` deserves its own look because it's the
safety net and the source of a recurring failure mode. The wrapper runs
the FanFicFare CLI and parses its output:

```rust
// (simplified)
let output = tokio::process::Command::new("fanficfare")
    .arg("--json-meta").arg(url)
    .output().await?;
if !output.status.success() {
    // map stderr → ScrapeError (e.g. "UnknownSite" → UnsupportedSite)
}
```

Common failure modes you'll see in logs:

- `UnknownSite` — FanFicFare doesn't know the domain (the force.net case).
- Timeouts/network errors from the FanFicFare subprocess.
- Parse failures when the site's HTML doesn't match FanFicFare's
  expectations.

The lesson: **the catch-all is a last resort, not a first choice.** Native
adapters are faster and more controllable; FanFicFare covers the long tail
of sites nobody has written a native adapter for.

> 🧪 **Try it:** run the FanFicFare CLI manually on a URL:
> `fanficfare --json-meta <url>` and see its output shape.
>
> ⚠️ **Watch out:** the FanFicFare subprocess is a dependency — if it's
> missing or broken on a host, every catch-all scrape fails. Health checks
> don't cover it; watch logs.
>
> 💡 **Key concept:** FanFicFare is the safety net for the long tail.
> Native adapters win when they exist.

---



---

<!-- part: 05-social.md -->


## Part 5 — Exports & the Reader

### Chapter 16: From URL to EPUB

`src/routes/export.rs` is the export pipeline — the heart of the original
product. Follow a URL through the whole journey:

1. **Pick the scraper.** `find_specific_or_fff(url)` (Chapter 14) decides
   who handles this URL.
2. **Check the body cache.** `load_body(config, url_id)`. On a hit, use the
   cached chapters — no network.
3. **On a miss, scrape.** `scraper.lookup()` for metadata, then
   `scraper.fetch_chapters()` for the text.
4. **Persist to the body cache.** `save_body(...)` — the archive grows.
5. **Build the format.** The EPUB builder (pure Rust, `export/`), or HTML /
   TXT / MD. MOBI/PDF/AZW3 go through the Calibre sidecar (a Docker
   container).
6. **Stream the file back.** Set `Content-Type`, `Content-Disposition`
   filename, and return the bytes.

The export handler is also where the **self-healing telemetry** hooks in:
on a scrape failure, `state.heal.record_failure(...)` records the URL,
error, and a snapshot — feeding `scrape_failures` (migration 029) and the
classifier that decides whether the failure is transient, blocked, or
structural.

**The semaphore guard.** `cache::get_export_semaphore(&state.cache_semaphores,
url_id, etype)` prevents two concurrent exports of the same URL from
double-scraping. The map is *bounded* (a 10k cap) so it can't grow forever
— a deliberate fix for a real leak. In-flight requests hold an `Arc` to the
semaphore, so removing the key from the map after completion is safe.

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;   // only one export per url_id at a time
// ... scrape + build + stream ...
```

> 🧪 **Try it:** read the export handler and count the steps between "URL
> comes in" and "file goes out." Then look at where `save_body` is called —
> that's the archive growing.
>
> ⚠️ **Watch out:** the semaphore is per `(url_id, etype)` — EPUB and HTML
> exports of the same fic can run in parallel, but two EPUBs of the same
> fic cannot. That's the intended behavior.
>
> 💡 **Key concept:** the export pipeline is cache-first. The body cache
> turns a network scrape into an occasional cost instead of a per-request
> one.

### Chapter 17: The Reader + The SPA

The SPA talks to `/api/*` via `frontend/src/lib/api/client.ts`. Every
frontend feature is: a page under `frontend/src/routes/`, an API call
through the client, and (usually) a `page.test.ts` vitest file.

The reader page (`/read/[urlId]`) is the flagship frontend feature:

- Fetches chapter HTML from the API (cache-first, thanks to the body
  cache).
- Tracks scroll progress in `localStorage` (keyed `fichub:reader:state:{url_id}`)
  so you resume where you left off.
- Offers a "Next Up" panel: sequel → community pick → readers-also-
  bookmarked. That panel is powered by the recommendation engine (Part 7).
- Has typography preferences and chapter navigation.

The frontend is a standard SvelteKit 5 app: `+layout.svelte` for the shell
and nav, `+page.svelte` per route, `+layout.ts` for client-side
initialization (e.g., PWA registration). The i18n system (`src/lib/i18n/`)
provides the `t('nav.modlog')` style translation calls across six
dictionaries.

One quirk to know: **route tests must be named `page.test.ts`.** The vitest
setup picks up exactly that pattern. If you name a test anything else next
to a route, it won't run.

> 🧪 **Try it:** open `frontend/src/routes/read/` and find the reader page.
> Then open `client.ts` and find how it attaches the JWT (`Authorization:
> Bearer <token from localStorage['fichub_token']>`).
>
> ⚠️ **Watch out:** the API client reads the token from
> `localStorage['fichub_token']`. The auth store may hold it in a different
> place — a known mismatch to check if personalization seems "silently
> disabled."
>
> 💡 **Key concept:** the frontend is a thin client over a rich API. The
> interesting logic is backend; the frontend renders and calls.

---

## Part 6 — Social, Community & Governance

### Chapter 18: Auth, Users & Roles

`src/routes/auth.rs` issues JWTs (`create_token`/`verify_token`). The
`AuthUser` extractor reads `Authorization: Bearer <jwt>` and yields the
user's id + role.

```rust
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
    pub role: i16,
    // ...
}
```

Roles are integers:

- **0** = regular user
- **1** = curator-ish (can propose merges/fixes)
- **5** = moderator (modlog actions, comment moderation)
- **10** = admin (everything, analytics, users)

Handlers gate with `auth.user_id` and role checks. Two gate styles you'll
see:

```rust
// "any logged-in user"
if auth.user_id == 0 {
    return Err(AppError::Unauthorized);  // or BadRequest(-403, ...)
}

// "admin only"
if auth.role < 10 {
    return Err(AppError::BadRequest(-403, "Admin access required".into()));
}
```

Anonymous requests get `AuthUser::default()` (id 0, role 0) — so "logged
in?" is `user_id != 0`. The JWT Claims shape is
`{ sub: i32, username, role, exp, iat }`.

> ⚠️ **Watch out:** role checks use `>=` thresholds. Check the existing
> pattern before writing your own gate — some endpoints are
> `require_logged_in`, others `role >= 10`, and a few use custom codes like
> `-403`. Match the neighborhood.
>
> 💡 **Key concept:** auth is a JWT in a header + a role integer. "Logged
> in" = `user_id != 0`; "admin" = `role >= 10`.

### Chapter 19: Bookmark Identity — Two Systems, Don't Conflate

This is a genuine footgun, documented in the project's skill notes, and
worth internalizing before you build personalization features.

There are **two bookmark systems**:

1. **`bookmarks`** — the shipped one. Keyed by `(user_id, url_id)`, driven
   by `AuthUser` (the JWT user). This is what the UI uses, and what
   personalization reads.
2. **`fic_bookmarks`** — legacy/anonymous. Keyed by `user_hash` (the
   sha256 of a lowercased AO3 profile URL), fed ONLY by the recommender
   worker's AO3-profile import. No user linkage.

Why the split? The old anonymous system predates accounts. The recommender
worker still imports AO3 profiles by hashing the profile URL. But anything
new that personalizes must key on `bookmarks.user_id` — never on
`fic_bookmarks`/`user_hash`, or you'll silently mix anonymous imports with
real user data.

The personal recommendations endpoint
(`/api/recommendations/personal`) uses bookmarks + anonymous download
signals from `request_log` (last 90 days, `url_id IS NOT NULL AND etype IN
('download','export')`). The gate is `PERSONAL_RECS_MIN_SIGNAL = 3` — fewer
than 3 signals returns `{"enough_data": false, "recs": []}`.

> ⚠️ **Watch out:** `request_log` has no user linkage and no `path` column
> — downloads are identified by `url_id IS NOT NULL` + the etype/export
> filename. If you need "who downloaded what," that's a deliberate gap.
>
> 💡 **Key concept:** when in doubt, key on `bookmarks.user_id`. The
> legacy anonymous system is worker-only.

### Chapter 20: The Modlog — Radical Transparency

`src/modlog.rs` + migration 034. Every moderator, curator, and admin action
is recorded — bans, role changes, upload approvals, comment removals, tag
merges, body-fix votes, flag resolves, and more.

The `record` helper is deliberately **best-effort**: it inserts into
`modlog` and never fails the main action. If the log write fails, the
action still succeeds — transparency is a goal, not a bottleneck.

```rust
crate::modlog::record(
    &state.db,
    auth.user_id,
    auth.username.clone(),
    "ban_user",
    "user",
    &user_id.to_string(),
    serde_json::json!({"banned": true}),
).await;
```

Any logged-in user can read `GET /api/modlog` (the `/modlog` page). This is
a product decision: **moderation is completely transparent** in FicHub.
When you add an admin action, call `modlog::record(...)` after the
successful DB write — it's a requirement, not an afterthought.

> 🧪 **Try it:** open `src/modlog.rs` and count the actions the codebase
> records. Then open `/modlog` on the live site.
>
> ⚠️ **Watch out:** the modlog is readable by any logged-in user — never
> write PII or sensitive details into the `details` JSON.
>
> 💡 **Key concept:** transparency is a feature. Log every admin action;
> make the log public to logged-in users.

### Chapter 21: Usage Analytics — Zero PII

`usage_events` (migration 033) + middleware. Every request records a `view`
(browsing) or `action` (export, vote, comment, ...) tagged with an
anonymous `X-Client-ID` — a random UUID stored in the browser's
localStorage, **never an IP**.

The middleware (`from_fn_with_state`) classifies each path: health = view,
meta = action, etc. The `/admin/analytics` dashboard shows:

- **Unique visitors** — daily (30d), weekly (12w), monthly (12m).
- **Engagement** — active users (performed actions) vs view-only users,
  with action events + total events + action rate.
- **Recent activity timeline** — the last 50 events (anonymous client,
  path, type, time).
- **Search analytics** — zero-result queries, trope popularity, search
  volume, and search→export conversion (searchers vs exporters, joined by
  anonymous client ID).

The zero-PII rule is absolute: client IDs only, aggregates never expose IPs.
`request_log` retains IPs internally for anti-bot, but analytics never read
them.

> 🧪 **Try it:** open `/admin/analytics` (as admin) and look at the cards.
> Then read the middleware that classifies view vs action.
>
> ⚠️ **Watch out:** if you add a new endpoint, decide its classification
> (view vs action) in the middleware — don't leave it unclassified.
>
> 💡 **Key concept:** measure without identifying. Anonymous client IDs
> give you funnels and engagement without PII.

### Chapter 21A: Ratings, Reviews & Comments

The social layer is where the community lives. Three subsystems worth
knowing in detail.

**Ratings (`work_ratings`).** Users rate a work 1-5 stars. The table is
UNIQUE per `(user_id, work_id)` — one rating per user per work. The
aggregate shown publicly is derived from ratings + the constructive filter:
public lists emphasize constructive feedback, never a raw dislike count.

**Reviews (`reviews`).** In-depth reviews, UNIQUE per `(user_id,
work_id)`. Reviews feed the recommendation engine through
`work_feedback_signals()` — they're signal, not just content.

**Comments (`comments`).** Threaded comments on works, with a
`constructive` flag. Moderators can hide or delete comments (each action is
recorded in the modlog). The comment triage feature uses Ollama to help
moderators classify incoming comments as constructive vs spam/toxic.

The house pattern for "user-generated content with moderation":

1. Insert with the author's user_id.
2. Validate content (length, spam heuristics).
3. Make it visible, with moderation hooks (hide/delete).
4. Record moderation actions in the modlog.

> 🧪 **Try it:** open `comments.rs` and find the hide/delete handlers.
> Notice they call `modlog::record` — transparency by default.
>
> ⚠️ **Watch out:** comment triage uses an LLM to *help* classify — the
> human moderator makes the final call. Never auto-hide based on model
> output alone.
>
> 💡 **Key concept:** social content is moderated + transparent. Every
> moderation action is logged and public.

### Chapter 21B: Follows, Notifications & the Updates Feed

The engagement loop:

**Follows (`follows.rs`).** Users follow works, authors, or other users.
The `follows` table tracks the target type + id.

**Updates feed (`updates.rs` + `/api/v1/feed`).** When a followed fic is
re-scraped and changes (new chapter, edited metadata), the feed shows it.
The refresh-fic re-scrape triggers follower notifications.

**Notifications (`notifications.rs`).** User-scoped notification rows for
events: "your request got an answer," "a fic you follow updated," etc.
Fic Requests M3 (a starter task) is about wiring the request board into
this notification plumbing.

The pattern: follow → track changes → notify → show in feed. Each piece is
a simple table + endpoint.

> 🧪 **Try it:** follow a fic on dev, trigger a re-scrape, and watch the
> feed + notification appear.
>
> ⚠️ **Watch out:** notification events should be idempotent — don't
> create duplicate notifications when a job runs twice.
>
> 💡 **Key concept:** follows + updates + notifications form the
> engagement loop; the pieces are simple but the wiring must be careful.

### Chapter 21C: Series, Authors, Lists & Shelves

Four content-organization features round out the social layer:

- **Series (`series.rs`)** — ordered works. The reader's "next in series"
  comes from here.
- **Authors (`authors.rs`)** — author profiles, bibliography, linked
  accounts. Keyed by canonical name.
- **Lists (`lists.rs`)** — curated reading lists: title, description,
  positioned items, blurbs. Users build themed collections.
- **Shelves (`shelves.rs`)** — personal shelves (like bookshelves for
  fics).

All four follow the same CRUD pattern: list → detail → create/update →
delete, with auth for ownership and modlog for moderation actions.

> 🧪 **Try it:** create a reading list on dev, add a few fics, and look at
> how `lists.rs` stores positions.
>
> ⚠️ **Watch out:** shelf/list items have ORDER (position) — don't treat
> them as unordered sets.
>
> 💡 **Key concept:** content organization is standard CRUD with order +
> ownership. The novelty is in how the reader surfaces it ("Next Up").

---



---

<!-- part: 05a-frontend.md -->


## Part 5A — The Frontend Deep Dive

The backend gets most of the attention, but the frontend is where users
actually live. This part takes you through the SvelteKit 5 app in enough
depth that you can add pages, wire them to the API, and follow the house
patterns.

### Chapter 17A: SvelteKit 5 Basics in This Repo

The frontend lives in `frontend/`. It's a SvelteKit 5 app using the
`@sveltejs/adapter-static` — it builds to a static site served by the Axum
backend.

The core files you'll touch:

- **`frontend/src/routes/+layout.svelte`** — the app shell: nav, footer,
  theme. This is where global navigation lives (the "Moderation Log"
  link, admin links, etc.).
- **`frontend/src/routes/+layout.ts`** — client-side init (PWA
  registration, auth store bootstrap).
- **`frontend/src/routes/+page.svelte`** — the home page.
- **`frontend/src/lib/api/client.ts`** — the API client. Every backend
  call goes through this file. It attaches the JWT from
  `localStorage['fichub_token']` as `Authorization: Bearer <token>`.
- **`frontend/src/lib/stores/auth.svelte.ts`** — the auth store (Svelte 5
  runes).
- **`frontend/src/lib/i18n/`** — the translation system: six dictionaries
  (en, de, fr, es, ...) + a `t()` function used in every page.

**Svelte 5 runes.** You'll see `.svelte.ts` files using `$state`,
`$derived`, and `$effect`. The auth store is a good example:

```ts
// auth.svelte.ts (simplified)
export const auth = (() => {
    let user = $state<User | null>(null);
    let token = $state<string | null>(null);
    return {
        get user() { return user; },
        get token() { return token; },
        async login(...) { ... },
        async logout() { ... },
    };
})();
```

Components import `auth` and read `auth.user` reactively — when it
changes, the UI updates. No manual subscriptions.

**The API client pattern.** Every endpoint has a typed function:

```ts
export async function fetchModlog(params: { limit?: number; action?: string } = {}) {
    const qs = new URLSearchParams(...).toString();
    return apiFetch(`/api/modlog?${qs}`).then(r => r.json());
}
```

`apiFetch` handles the JWT header, JSON parsing, and error shape
(`{err, msg}`). New endpoints get a function here first.

> 🧪 **Try it:** open `client.ts` and find the function for your favorite
> backend endpoint. Then open the matching page and trace how it calls it.
>
> ⚠️ **Watch out:** the API client reads the token from
> `localStorage['fichub_token']`. If the auth store holds the token in a
> different variable, they can drift — a known quirk worth checking when
> debugging auth issues.
>
> 💡 **Key concept:** one API client file, typed functions per endpoint,
> JWT attached centrally. Pages are thin; the client owns the fetch
> logic.

### Chapter 17B: i18n — Six Languages, One t() Function

Every user-facing string goes through the i18n system. The dictionaries
live in `frontend/src/lib/i18n/dictionaries/*.ts` (one per language). The
`t()` function is imported and used everywhere:

```svelte
<a href="/modlog">{t('nav.modlog')}</a>
```

When you add a string, you add a key to ALL six dictionaries — the house
rule is no hardcoded user-facing text. The key naming convention is
`area.item` (e.g. `nav.modlog`, `search.results`, `admin.analyticsTitle`).

> 🧪 **Try it:** add a key to one dictionary and see the type/consistency
> check fail until you add it to the rest. That's the safety net.
>
> ⚠️ **Watch out:** forgetting a key in one language shows a raw key or
> empty string to that language's users. Add all six at once.
>
> 💡 **Key concept:** i18n is structural, not optional. The t() function
> is the only way user-facing text enters the UI.

### Chapter 17C: PWA Offline — The Service Worker

FicHub is installable and offline-capable. The pieces:

- **`frontend/static/manifest.webmanifest`** — the PWA manifest (name,
  icons, display mode).
- **`frontend/static/sw.js`** — the service worker. Registered from
  `+layout.ts` via `src/lib/pwa/register.ts` (prod only).
- **`src/lib/pwa/strategies.ts`** — pure functions (unit-tested!) that
  decide the caching strategy per URL:
  - `/fic/*` + `/read/*` — **cache-first** (offline reading works).
  - public fic APIs (`/api/reader/*`, `/api/epub`, `/api/meta`,
    `/api/search/similar/*`) — **network-first**.
  - shell precache — the app shell is pre-cached at install.
  - **everything else — 'none'** (auth/bookmarks/admin/social are NEVER
    intercepted; per-user data privacy).

The most important pitfall (from the project's history): **precache the
hashed `/_app/` build assets or the SPA renders blank offline.** The SW
discovers them at install by fetching `/` and regex-ing out the
`/_app/...` URLs.

> 🧪 **Try it:** open the site in Chrome, check DevTools → Application →
> Service Workers, then set the network to Offline and reload a fic you've
> read. It renders from cache.
>
> ⚠️ **Watch out:** never intercept auth/admin/social endpoints in the SW
> — that was a real privacy bug in v1. `strategies.ts` is the enforced
> contract.
>
> 💡 **Key concept:** offline is a strategy decision per URL, enforced in
> pure tested functions.

### Chapter 17D: The Admin UI

`frontend/src/routes/admin/` contains the admin pages: dashboard, usage
analytics, modlog, scrapers, users, bots, comment triage. The pattern:

- Each page is a `+page.svelte` that calls an admin API (via `adminFetch`,
  a variant of the client that requires auth).
- Role-gated in the backend (role ≥ 10); the frontend also hides admin
  links from non-admins.
- Each page has a `page.test.ts` asserting it renders with fixture data.

The admin pages are a great place to start contributing: the backend
endpoints exist, and several pages (auto-tag review, metadata correction,
translation post-edit) are still TODO — the exact starter tasks from
Chapter 24.

> 🧪 **Try it:** open `/admin/analytics` and find the card components.
> Then read its `page.test.ts` to see the fixture pattern.
>
> ⚠️ **Watch out:** admin routes are role-gated server-side. Don't rely
> on the frontend hiding links as the only protection.
>
> 💡 **Key concept:** admin UI is thin; the API enforces the role gate.

### Chapter 17E: Page Tests — The `page.test.ts` Pattern

Every user-facing route folder may carry a `page.test.ts` using vitest +
@testing-library/svelte. The pattern:

```ts
// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';

// mock the API client so the page renders with fixture data
vi.mock('$lib/api/client', () => ({ fetchModlog: vi.fn(() => Promise.resolve({ entries: [...] })) }));

describe('modlog page', () => {
    it('renders entries', async () => {
        render(Page);
        expect(screen.getByText(/Moderation Log/)).toBeTruthy();
    });
});
```

Two vitest gotchas to remember:

1. **The file MUST be named `page.test.ts`** — that's the exact pattern
   the runner picks up.
2. **`vi.mock` hoisting** — if the mock references a variable defined
   later in the file, use `vi.hoisted()` or define the mock value inline;
   vitest hoists `vi.mock` calls to the top of the file.

Run a single page test:

```bash
npx vitest run src/routes/modlog/page.test.ts
```

> 🧪 **Try it:** read any `page.test.ts` in the repo and identify the
> mock, the render, and the assertions. Then add one assertion of your
> own and watch it pass.
>
> ⚠️ **Watch out:** tests that query text appearing both in a dropdown
> option and in page content need `getAllByText(...)` — `getByText` throws
> on duplicates.
>
> 💡 **Key concept:** every page has a test. Fixtures via mocks, render
> via testing-library, assert via screen queries.

---



---

<!-- part: 06-workflow.md -->


## Part 7 — Testing, Working Here, and What's Next

### Chapter 22: The Testing Strategy

FicHub takes testing seriously — the repo convention is full test coverage
with frequent small commits. Three layers:

**1. Unit tests** — in-module `#[cfg(test)]` blocks. Fast, no DB. Run with:

```bash
cargo test --lib <name>        # e.g. cargo test --lib cache
```

**2. DB-gated integration tests** — `tests/*.rs` (e.g. `admin_api.rs`,
`modlog_api.rs`, `analytics_api.rs`, `heal_api.rs`). These boot a real
server against the real dev database. They're `#[ignore]`-marked by default
so a plain `cargo test` skips them, and they use a `Mutex` + unique seed
names + cleanup because the shared dev DB can't run them in parallel. Run
them with:

```bash
cargo test --test admin_api -- --include-ignored --test-threads=1
```

The pattern: each test seeds its own rows with unique names (e.g.
`test-client-%`), asserts, then cleans up. The `Mutex` guarantees one
DB-gated suite at a time.

**3. Frontend vitest** — `page.test.ts` next to each route. Run with:

```bash
npx vitest run src/routes/<x>/page.test.ts
```

**Repo convention: never run the FULL suite in normal dev.** Cargo takes
~10 minutes and DB-gated suites are slow. Run targeted suites only —
that's the house style.

The **canonical gate** is `hermes verify --save` (detect → build → test →
boot → readiness). It runs the real build and the full test pass and
records evidence. Known quirks: the shared NFS target's `incremental`
directory gets stale after mount wedges — `rm -rf target/debug/incremental`
before a canonical verify.

> 🧪 **Try it:** run one DB-gated suite end to end:
> `cargo test --test modlog_api -- --include-ignored --test-threads=1`
> (needs `.env` + dev DB). Watch it seed, assert, clean up.
>
> ⚠️ **Watch out:** if a DB-gated test fails with a duplicate-key error,
> it's usually a missing cleanup from a previous crashed run — the unique
> seed names are exactly what makes the cleanup tractable.
>
> 💡 **Key concept:** targeted suites, unique seeds, mutex-guarded DB
> access. The canonical verify is the final gate.

### Chapter 23: The Developer Workflow

Here's how work actually gets done on this repo. Follow this and you'll fit
right in.

**Branching.** Work on a feature branch, merge when verified. The repo
owner edits concurrently in another pane — don't squat on main. One git
worktree per feature when delegating (`/media/alvaro/code-worktrees/wt-<name>`).

**Builds.** Never build on `/home`. Use
`CARGO_TARGET_DIR=/media/alvaro/cargo-target-sh` (local ext4; the repo
lives on NFS). If you see NFS errors or disk-full, check the target dir.

**DB-gated tests need `.env`.** Worktrees ship only `.env.example`; copy
the real one:
`cp /personal/documents/code/rust/fichub/.env <worktree>/.env`.

**The NFS git-objects quirk.** After commit/merge, refs can dangle because
the repo lives on NFS. The recipe:

```bash
# per commit, when refs dangle:
export GIT_OBJECT_DIRECTORY=/home/alvaro/.cache/git-objects-<name>
git add ... && git commit ...
cp -rn /home/alvaro/.cache/git-objects-<name>/* .git/objects/ 2>/dev/null
unset GIT_OBJECT_DIRECTORY
git cat-file -t <sha>    # always verify
```

Also: `git config core.fsync none` first, and remove stale
`.git/refs/remotes/github/main.lock` before pushes.

**Conventional commits.** `feat:`, `fix:`, `test:`, `docs:`, `refactor:`.
Small, meaningful units.

**Deploy.** The live box is ThinkCentre (`fichub.polarisocial.xyz`).
Release build → copy binary → `sudo systemctl restart fichub.service` →
migrations apply on boot → verify health + the changed endpoint.

> 🧪 **Try it:** if you have a worktree, create a scratch branch, make a
> trivial commit, and walk the NFS object-copy recipe end to end. The
> `git cat-file -t` check is your safety net.
>
> ⚠️ **Watch out:** node_modules symlinks in worktrees point at a shared
> install — never commit them (mode 120000). The main frontend's
> node_modules is a real dir.
>
> 💡 **Key concept:** branch → build on local ext4 → targeted tests →
> conventional commit → verify → deploy. Boring on purpose.

### Chapter 24: Where to Start Contributing

The highest-leverage first tasks (from `docs/ROADMAP.md`) — all are real
gaps, sized for a junior dev, with the backend mostly or fully in place:

1. **Curator/admin UI backlog.** The APIs exist; the pages don't:
   - Auto-tag review UI (draft → approve ML tag suggestions)
   - Manual fic approval UI (`/api/admin/moderation/queue` exists)
   - Metadata correction (fix title/author/status/description on works)
   - Translation post-edit (draft → human)
2. **Search frontend control for `main_char_attr`.** The backend already
   supports "Dark Harry" semantics (main character + attribute). The UI
   pickers are missing — a clean frontend task.
3. **Fic Requests M3.** Notifications when a request gets answers/accepted,
   plus request upvotes. Backend plumbing mostly exists.
4. **Ops.** CI/CD, an external uptime probe, secrets hygiene (move DB
   password out of `.env` into a 600-perm systemd EnvironmentFile), and
   body-cache backups.

Pick one, branch, and follow the pattern: read the route file + its
`page.test.ts`, implement backend → frontend → test, verify with targeted
suites, commit conventionally, and get review.

**The pattern for a backend feature** (worth writing down):

1. `AppState` gets anything new it needs (or reuse existing).
2. Handler in `src/routes/<area>.rs` — `pub async fn`, `AppResult<T>`.
3. Register in `build_router()` (`{param}` syntax, Axum 0.8).
4. `modlog::record` if it's an admin/curator action.
5. DB-gated test in `tests/<area>_api.rs` (unique seeds + cleanup).
6. Frontend page + `page.test.ts`.
7. Targeted suites green → conventional commit.

> 🧪 **Try it:** pick ONE of the four starter tasks and spend an hour
> reading the files you'd touch. You don't have to finish — just map the
> change.
>
> ⚠️ **Watch out:** don't start with a huge architectural task. The
> starter list is sized to build confidence fast.
>
> 💡 **Key concept:** the API→test→frontend→test loop is the rhythm of
> this codebase. Every feature follows it.

### Chapter 24A: A Day in the Life — Troubleshooting Cheat-Sheet

When something breaks (and it will), here's the fast triage path. This is
the exact playbook the team uses.

**"The site is down / health check fails."**
1. `curl https://fichub.polarisocial.xyz/api/health` — which boolean is
   false?
2. `systemctl status fichub.service` + `journalctl -u fichub.service -n
   100` on the deploy box — what's the last error?
3. DB issue? `sudo -u postgres psql -d fichub -c "SELECT 1"`.
4. Redis issue? `redis-cli PING` — expect PONG. (Remember: a false
   redis:false used to happen from the shared-connection BRPOP; the fix is
   health_redis.)

**"A scrape fails for one URL."**
1. Check `scrape_failures` in the DB — what classification did the
   classifier assign?
2. Transient → retry. Blocked → likely a bot wall (cookies/proxy needed).
3. Structural → the site changed its HTML; the scraper needs a selector
   fix. This is a real bug, not an ops problem.

**"The search returns wrong results."**
1. Test the query in the UI, then check the parser's unit tests for the
   pattern.
2. Remember `main_char_attr` semantics — the attribute applies to the MAIN
   character.
3. Check `zero_result_queries` in admin search analytics — is this a known
   blind spot?

**"A DB-gated test fails mysteriously."**
1. Is it the incremental cache? (`rm -rf target/debug/incremental`)
2. Is it a leftover row from a crashed run? (unique seed names should
   isolate; clean up.)
3. Is it a parallel-suite collision? (run with `--test-threads=1`)

**"The frontend shows the wrong thing but the API is right."**
1. Check the API response first (`curl` the endpoint).
2. Check `client.ts` for the function the page calls — right URL? right
   token?
3. Check the `page.test.ts` — does the fixture match reality?

> 🧪 **Try it:** next time something breaks, follow the cheat-sheet
> top-to-bottom before diving into code. You'll find most problems in the
> first two steps.
>
> ⚠️ **Watch out:** the cheat-sheet assumes the health/telemetry systems
> are recording. If `scrape_failures` is empty when you expected a
> failure, check whether the route records it (a known telemetry gap for
> some paths).
>
> 💡 **Key concept:** triage by layers: health → logs → telemetry → code.
> Most issues resolve before you open an editor.

### Chapter 24B: The Docs You Should Read (and Keep Current)

FicHub takes documentation seriously — the house rule is "rich,
self-contained markdown fed to chatbots; features/decisions in markdown
not only code." The docs you'll use:

- **`docs/ROADMAP.md`** — the canonical plan: shipped, in-flight, and
  prioritized suggestions (P1-P7). Read this when deciding what to work
  on.
- **`docs/SPECIFICATION.md`** — the API spec + endpoint inventory. Check
  it before adding an endpoint.
- **`docs/STATUS.md` / `docs/TODO.md`** — session status + feature notes
  (superseded by ROADMAP for planning, still useful for history).
- **`docs/src/`** — the user-facing mdbook chapters (intro, searching,
  downloading, transparency, ...). Served at `/docs/`.
- **`docs/using-recommender-platform.md`** — how to use the rec platform.
- **The `fichub-development` skill** — the maintainer's playbook with
  references for every subsystem (in the Hermes profile, not the repo).

**The rule:** when you ship a feature, update the docs in the same commit.
ROADMAP gets the status change; SPECIFICATION gets the endpoint; the
mdbook gets a user-facing blurb if it's user-visible. Docs drift is a
review-blocking issue here.

> 🧪 **Try it:** pick a shipped feature (say, the modlog) and trace it
> through ROADMAP → SPECIFICATION → docs/src. See how the docs mirror the
> code.
>
> ⚠️ **Watch out:** don't add an endpoint without updating the
> SPECIFICATION's endpoint inventory. It's the contract.
>
> 💡 **Key concept:** docs are part of the deliverable. Ship code + docs
> together, or the review will bounce it.

---

### Chapter 25: Glossary of Terms You'll See in the Code

- **url_id** — the string key for a fic source (e.g. `ao3_21845264`,
  `xenforo_50062326`). Used everywhere as the primary identifier.
- **work** — the abstract story; a work has multiple *sources* (fic_info
  rows) if posted on several sites.
- **fic_info** — one source row: url_id, site_domain, title, author, etc.
- **main_char_attr** — a search filter with AO3 "Dark Harry" semantics:
  main character + attribute (e.g. `main_char_attr: dark harry potter`).
- **body cache** — the on-disk JSON blob store of scraped chapter bodies
  (`BODY_CACHE_DIR`).
- **modlog** — the transparent moderation log (migration 034).
- **usage_events** — the zero-PII analytics table (migration 033).
- **scrape_failures / agent_runs** — self-healing telemetry (migrations
  029-030).
- **health_redis** — the dedicated Redis connection for health checks
  (never share the worker's BRPOP connection).
- **CacheSemaphores** — bounded map of export semaphores (one concurrent
  export per url_id).
- **REC_ENGINE_MODE** — `legacy` (current cooccurrence engine) vs
  `pluggable` (strategy registry with decay/embeddings/mf/... strategies).
- **REC_SHADOW_MODE** — compute new strategies but return legacy output
  while logging impressions (safe A/B).
- **find_specific_or_fff** — registry helper preferring native scrapers
  over the FanFicFare catch-all.
- **X-Client-ID** — the anonymous browser UUID for usage analytics (never
  an IP).
- **AuthUser** — the JWT-backed auth extractor; `user_id != 0` means
  logged in, `role >= 10` means admin.
- **BODY_CACHE_DIR** — env knob for where scraped bodies live (default
  `/public/literature/fichub/bodies`).
- **hermes verify** — the canonical build+test+readiness gate that records
  verification evidence.

---

## Conclusion

You now know the shape of the whole system: one Axum binary serving both
API and SPA, a config struct that tames every env var, a query module that
owns the SQL, a scraper registry that prefers native adapters, a body
cache that makes the site a durable archive, and a social/governance layer
built on JWT auth, a transparent modlog, and zero-PII analytics.

The fastest way to learn the rest is to change something small. Pick a
chapter from "Where to Start Contributing," open the route file, add a
feature + a test, and run the targeted suite. The repo is well-commented,
the docs (`docs/ROADMAP.md`, `docs/SPECIFICATION.md`) map the territory,
and this book gives you the vocabulary.

Welcome aboard — the fics are waiting.



---

<!-- part: 06a-deploy.md -->


## Part 6A — Deployment & Operations Deep Dive

Knowing how the product runs in production makes you a better developer of
it. This part covers the deploy pipeline, the ops gotchas, and what "done"
means here.

### Chapter 23A: The Deploy Pipeline

The production box is **ThinkCentre** (hostname `thinkcentre`), serving
`fichub.polarisocial.xyz`. The deploy path is deliberately simple:

1. **Build the backend release** on the dev machine (or the deploy box):
   ```bash
   cargo build --release --bin fichub
   ```
2. **Copy the binary** to the deploy box (or build there).
3. **Restart the service**:
   ```bash
   sudo systemctl restart fichub.service
   ```
4. **Migrations apply on boot** — the binary runs `sqlx::migrate!()` and
   applies any new migrations transactionally.
5. **Verify**: `curl https://fichub.polarisocial.xyz/api/health` shows
   `{"status":"ok","db":true,"redis":true}`, and the changed endpoint
   behaves.

The frontend: `cd frontend && npm run build` writes to `frontend/build`,
which the backend serves via ServeDir. Because the build dir is on shared
NFS, updating static files can be as simple as rebuilding — the running
service reads from disk. (A full restart picks up backend changes.)

**Key production config** (from the service env):

- `BODY_CACHE_DIR=/public/literature/fichub/bodies` — the body cache lives
  on the attached drive, NOT the DB.
- `REC_ENGINE_MODE=legacy` — the legacy rec engine is the live default.
- `AGENT_ENABLED=false` — the self-healing agent is off (diagnose-only).
- The Ollama default model is `lfm2.5:8b`.

> 🧪 **Try it:** on the deploy box, `systemctl status fichub.service`
> shows the service; `journalctl -u fichub.service -n 50` shows startup
> logs including migration output.
>
> ⚠️ **Watch out:** never build the backend on the deploy box's `/home`
> (NFS/space). Use the local ext4 target dir, as on dev.
>
> 💡 **Key concept:** deploy = build → copy → restart → migrations →
> verify. The binary carries its own schema.

### Chapter 23B: The Ops Gotchas (Learned the Hard Way)

These are documented because they bit the team — read them once and you'll
save yourself hours.

**1. The NFS git-objects quirk.** The repo lives on NFS. Git refs
repeatedly dangle after commits/merges. The recipe (from the workflow
chapter): set `GIT_OBJECT_DIRECTORY`, commit, copy objects into
`.git/objects`, unset, verify with `git cat-file -t <sha>`. If `git
cat-file` fails, the objects didn't land.

**2. The mergerfs wedge.** The deploy box's storage pool (mergerfs) can
wedge under heavy I/O. Recovery: `sudo umount -l /personal` +
`sudo systemctl restart mergerfs-personal.service` +
`sudo systemctl restart nfs-kernel-server`, then remount. If dev's
`/personal` goes EIO, this is the fix.

**3. The shared NFS cargo target.** `target/debug/incremental` on the
shared NFS target gets stale rlib fingerprints after mount wedges,
producing "required to be available in rlib format, but was not found"
errors. Fix: `rm -rf target/debug/incremental` and rebuild.

**4. The false "Redis down" report.** The admin dashboard once showed
Redis 🔴 when Redis was fine. Root cause: the health check PINGed the
shared `state.redis` connection, which the bookmark-import worker had
parked in an unbounded BRPOP — the PING timed out. Fix: dedicated
`health_redis` connection. **Rule: never PING the shared connection from a
health check.**

**5. The `.env` file.** Dev `.env` contains `DATABASE_URL`, `JWT_SECRET`,
`REDIS_URL`, etc. Worktrees ship only `.env.example` — copy the real one
for DB-gated tests. Secrets are in `.env`, never in test specs.

> 🧪 **Try it:** pick one gotcha and find the code/script that addresses
> it (e.g. the `health_redis` field in `AppState`).
>
> ⚠️ **Watch out:** if the DB-gated tests fail with weird rlib errors,
> it's the incremental cache (gotcha 3), not your code. Clear it first.
>
> 💡 **Key concept:** ops problems are deterministic once understood. The
> gotchas above have known recipes — follow them, don't improvise.

### Chapter 23C: What "Done" Means

A feature is done when it passes the house gates:

1. **Targeted tests pass** — the suite you touched, not the full suite
   (`cargo test --test <suite>` or `vitest run <file>`).
2. **The canonical verify passes** — `hermes verify --save` runs
   build → test → boot → readiness and records evidence. (On this setup,
   clear `target/debug/incremental` first if the shared target is stale.)
3. **Deployed + live-verified** — the binary is on the deploy box, the
   service restarted, migrations applied, and the changed endpoint checked
   via the public URL.
4. **Committed conventionally + pushed** — small meaningful commits,
   `feat:`/`fix:`/`test:`/`docs:`, mirror pushed to the opencommit.eu
   remote.

**The test conventions again, because they matter:**

- Never run the FULL cargo suite in normal dev (10 min).
- DB-gated suites: `#[ignore]` + `--include-ignored --test-threads=1`.
- Frontend: `page.test.ts` per route; `npx vitest run <file>`.

> 🧪 **Try it:** look at a recent commit message in `git log --oneline` and
> classify each by type. Notice how small the units are.
>
> ⚠️ **Watch out:** "works on my machine" isn't done. The verify + deploy
> steps are what make a feature real here.
>
> 💡 **Key concept:** done = tests + verify + deploy + commit. The bar is
> mechanical, not aspirational.

### Chapter 23D: The Live Systems You Can Inspect

The live site is a great learning tool. Things to try on
`fichub.polarisocial.xyz`:

- `/api/health` — the liveness check (db + redis booleans).
- `/roadmap` — the MaxDiff/Elo consensus arena (vote!).
- `/modlog` — the public moderation log (read it as any user).
- `/admin/analytics` — the zero-PII usage dashboard (admin only).
- `/docs/` — the user-facing docs (mdbook).
- `/ask` — Ask the Archive (natural-language search).
- `/requests` — the fic-request prompt board.
- `/read/<url_id>` — the web reader (offline-capable via PWA).

> 🧪 **Try it:** pick three of these and trace each back to its backend
> route in `build_router()`. You'll see the whole map again, this time
> live.
>
> ⚠️ **Watch out:** the live site is production — don't spam endpoints or
> create junk data while exploring. Use the dev server for experiments.
>
> 💡 **Key concept:** the live site is the product's truth. When in
> doubt about how something behaves, check production behavior — then
> find the code that produced it.

---



---

<!-- part: 07-feature-build.md -->


## Part 7A — Build a Feature End to End (The Walkthrough You'll Reuse)

The single most valuable thing an onboarding guide can give you is a
complete, realistic feature build. Let's do one together. We'll add a tiny
feature that touches every layer: **`GET /api/ping` returning
`{"pong": true}`** — deliberately small, but it walks the entire pipeline
you'll repeat for every real feature.

### Step 1: The handler

In `src/routes/health.rs` (or a new file — let's reuse health since it's
the right neighborhood):

```rust
use axum::Json;
use serde_json::{json, Value};

/// GET /api/ping — trivial liveness probe
pub async fn ping_handler() -> Json<Value> {
    Json(json!({"pong": true}))
}
```

Notice: no `State` needed — this handler doesn't touch shared state. When
yours does, add `State(state): State<Arc<AppState>>`.

### Step 2: Register the route

In `src/server.rs`, inside `build_router()`, add:

```rust
.route("/api/ping", get(crate::routes::health::ping_handler))
```

Put it with the other health/liveness routes, BEFORE the ServeDir fallback.

### Step 3: A unit test (fast, no DB)

In `src/routes/health.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ping_returns_pong() {
        let body = ping_handler().await;
        assert_eq!(body.0["pong"], true);
    }
}
```

### Step 4: An integration test (DB-gated, optional for this size)

If the endpoint touches the DB or auth, add it to `tests/<area>_api.rs`
with the house pattern: `#[ignore]`, unique seed names, cleanup. For a pure
`/api/ping`, the unit test is enough.

### Step 5: Run the targeted suite

```bash
cargo test --lib health
```

### Step 6: The frontend (when the feature is user-visible)

Add a call in `frontend/src/lib/api/client.ts` if the page needs it, then
the page under `frontend/src/routes/`, then a `page.test.ts`.

### Step 7: Commit conventionally + verify

```bash
git add src/routes/health.rs src/server.rs
git commit -m "feat: add /api/ping liveness probe"
```

Then, for a real feature, run `hermes verify --save` for the canonical
gate.

That's it. Every feature — search filters, modlog entries, admin pages —
follows the same skeleton. The size of the feature changes what goes in
steps 1-6, not the order.

> 🧪 **Try it:** actually do the ping feature right now. It'll take ten
> minutes and permanently demystify the loop.
>
> ⚠️ **Watch out:** if your route path uses a param, use `{param}` (Axum
> 0.8), and remember `build_router` order matters vs the SPA fallback.
>
> 💡 **Key concept:** handler → route → test → frontend → commit. The
> loop is small; the discipline is doing it every time.

### Step 8: A bigger example — the modlog entry

Now let's make the loop concrete with a *real* feature shape: adding a
modlog entry to an admin action. Say we're adding a "delete user" admin
endpoint. The handler skeleton:

```rust
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,                      // JWT extractor
    Path(user_id): Path<i32>,
) -> AppResult<Json<Value>> {
    if auth.role < 10 {
        return Err(AppError::BadRequest(-403, "Admin access required".into()));
    }
    // ... actually delete the user (or soft-delete) ...
    sqlx::query("UPDATE users SET is_deleted = TRUE WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    // TRANSPARENCY: record it in the modlog
    crate::modlog::record(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "delete_user",
        "user",
        &user_id.to_string(),
        serde_json::json!({"deleted": true}),
    ).await;
    Ok(Json(json!({"err": 0, "msg": "user deleted"})))
}
```

Notice the shape: **gate → mutate → record → respond.** That's the house
style for every admin/curator action. The modlog call is best-effort; even
if it fails, the action already happened.

> 🧪 **Try it:** pick an existing admin handler (e.g. in `admin.rs`) and
> identify the gate, the mutate, the record, and the respond steps. They're
> all there.
>
> ⚠️ **Watch out:** never log PII into the modlog `details` — it's
> readable by any logged-in user.
>
> 💡 **Key concept:** every admin action = gate → mutate → modlog →
> respond. The order is not arbitrary; the record is part of the product,
> not an afterthought.

### Chapter 26: Self-Healing Scraping (the Agent Loop)

FicHub has an ambitious self-healing system (`src/heal/`, migrations
029-030) for the scraper subsystem. When a scrape fails:

1. `state.heal.record_failure(url, error, snapshot)` writes to
   `scrape_failures` — including an HTML snapshot of the page when
   possible.
2. A pure-Rust classifier (`src/heal/classifier.rs`) labels the failure:
   **transient** (network blip), **blocked** (bot wall / 403), **structural**
   (the site changed its HTML so the parser breaks), or **systemic**.
3. Fingerprint + debounce dedupe repeated failures of the same URL.
4. An admin endpoint (`POST /api/admin/heal`) runs the diagnose-only loop
   today — it shows what it WOULD do.
5. The future (M2): an agent (Ollama or CommandCode) that CREATES a new
   scraper on the fly when the failure is structural, then the site uses it
   immediately.

Currently autonomy is OFF by default (`AGENT_ENABLED=false`) — the
diagnose-only loop is safe to run, and the agent loop is documented but
gated. This is a great example of "build the telemetry first, gate the
autonomy."

**Why the classification matters:** a transient failure (one network blip)
should be retried; a blocked failure (Cloudflare challenge) needs a
different strategy (user-supplied cookies, a proxy); a structural failure
(the site changed its HTML) means the scraper itself is broken and needs a
rewrite; a systemic failure (Postgres down) means nothing scraper-related
will help. The classifier's output drives what the healer does — and what
it records for human review.

> 🧪 **Try it:** look at `src/heal/classifier.rs` — the classification is
> pure Rust and unit-tested. Feed it a few failure strings and see what it
> labels them.
>
> ⚠️ **Watch out:** the agent loop is OFF. Don't enable it without the
> safety review documented in NEXT.md.
>
> 💡 **Key concept:** telemetry before autonomy. Record failures,
> classify them, dedupe, and only then consider automated healing.

### Chapter 27: Curator Content Fixes (Peer-Voted Bodies)

The body cache stores scraped content — but scrapes can be wrong (e.g. a
forum thread where the fic is post #2 and the "chapters" include the OP
announcement and comments). Curators fix this with a peer-voted workflow
(`src/routes/curator_content.rs`, migration 032):

1. `POST /api/curator/content/{url_id}/propose` — a curator proposes a
   corrected body.
2. Other curators vote up/down (`POST /api/curator/content/proposals/{id}/vote`,
   no self-vote).
3. At quorum (≥2 votes) with net ≥ 1, the fix is applied to the body cache
   (`save_body` + version bump) and recorded in the modlog.
4. `POST /api/curator/content/{url_id}/delete` removes a cached body.

This is the "human-in-the-loop" correction path for the cache: the scraper
doesn't guess (per product decision, "don't auto-guess fic content"), and
humans fix bodies post-hoc with votes.

**Why peer-voted and not "curator edits directly"?** Because the cache is
a public asset. One curator's mistake would corrupt the archive for
everyone; requiring a second curator's vote catches errors and keeps
quality high without making fixes slow (quorum is only 2).

> 🧪 **Try it:** read `curator_content.rs` and trace a proposal through to
> application. Note where it writes to the body cache vs the DB.
>
> ⚠️ **Watch out:** the quorum threshold is small (≥2 votes). Don't loosen
> it casually — peer review is the quality gate.
>
> 💡 **Key concept:** the cache is correctable. Propose → vote → apply
> gives human oversight to automated scraping.

### Chapter 28: The AI Features (Auto-Tagger & Translations)

Two more Ollama-backed features complete the picture:

**Auto-tagger** (`src/routes/auto_tag.rs`): given a fic's metadata/body,
Ollama suggests tags. Suggestions land as drafts; a curator approves or
rejects them (the review UI is a starter task). This directly improves
search quality — more accurate tags → better `main_char_attr` and trope
searches.

**Translations** (`src/routes/locales.rs`, migrations 023-025): ML
translation of fic metadata/content, with a human post-edit workflow.
Approved translations are stored and served in the reader.

Both follow the same pattern as Ask the Archive: LLM proposes, human
confirms, the result is stored deterministically. The LLM is never the
source of truth by itself.

**The pattern in code:**

```rust
// 1. Model proposes
let suggestion = ollama::suggest_tags(&state, &fic).await?;
// 2. Store as a DRAFT (never approved directly)
sqlx::query("INSERT INTO tag_suggestions (url_id, tag_name, status) VALUES ($1, $2, 'draft')")
    .bind(&fic.url_id).bind(&suggestion).execute(&state.db).await?;
// 3. Human approves later via the review endpoint (status → 'approved')
```

> 🧪 **Try it:** find the auto-tag endpoint and see how a suggestion flows
> to a draft. Then find where a human approves it.
>
> ⚠️ **Watch out:** ML outputs are drafts by design. Never let an LLM
> write directly to user-visible "approved" state without a human gate.
>
> 💡 **Key concept:** human-in-the-loop AI. The model proposes; a human
> disposes; the DB stores the human-confirmed result.

### Chapter 29: The Hidden Complexity — Worker Tasks & Background Jobs

Not everything happens in request handlers. FicHub has background workers
spawned in `server::run()`:

- **The recommender worker** (`src/recommender/worker.rs`) — maintains
  co-occurrence data, imports AO3 profiles (the legacy `fic_bookmarks`
  path), and refreshes rec-relevant statistics. It uses Redis for queues —
  including that BRPOP we met in Chapter 6.
- **The bookmark-import worker** — the one that parks a blocking BRPOP on
  the shared Redis connection. Its job is to watch a queue of AO3 profile
  URLs and import their bookmarks into `fic_bookmarks`.

When you add a background job, the pattern is: spawn it in `run()`, give it
`AppState` (or the specific connections it needs), log with tracing, and
make it resilient to failures (don't crash the server if a job errors).

> 🧪 **Try it:** find where workers are spawned in `server::run()` and list
> what each one does.
>
> ⚠️ **Watch out:** a worker that blocks forever on a connection can break
> health checks (the Redis story). Keep workers' blocking operations off
> shared connections.
>
> 💡 **Key concept:** request handlers are the front door; workers are the
> kitchen staff. Both matter, and both must be failure-tolerant.

---

## Where to Go Next

You've now seen: the one-binary architecture, the config struct, the
query module, the scraper registry, the body cache, the export pipeline,
the search parser, the rec platform, the anti-bot layers, the consensus
engine, self-healing, curation, and the AI features. That's the whole
product, end to end.

The best next step is to close this book and build something small — the
ping feature from the walkthrough, or one of the starter tasks from the
workflow chapter. The repo is well-commented; the docs
(`docs/ROADMAP.md`) are the living plan; and now you know the vocabulary
to ask good questions.

Welcome to FicHub. Happy building.




---

<!-- part: 08-appendix.md -->


## Appendix — Quick Reference

### The Commands You'll Type Daily

**Backend:**

```bash
cargo check                        # fast type-check
cargo test --lib <module>          # unit tests for a module
cargo test --test <suite> -- --include-ignored --test-threads=1   # DB-gated
cargo build --release --bin fichub # release binary
cargo run                          # dev server on :8000
```

**Frontend:**

```bash
cd frontend
npm run dev                        # SPA on :5173 (proxies /api)
npm run build                      # static build → build/
npx vitest run src/routes/<x>/page.test.ts   # one page test
```

**Database:**

```bash
psql -h localhost -U fichub -d fichub        # dev DB
sudo -u postgres psql -d fichub              # on the deploy box
```

**Deploy (deploy box):**

```bash
sudo systemctl restart fichub.service
journalctl -u fichub.service -n 100
curl https://fichub.polarisocial.xyz/api/health
```

**Git (with the NFS quirk):**

```bash
# before commit, when refs dangle on NFS:
export GIT_OBJECT_DIRECTORY=/home/alvaro/.cache/git-objects-<name>
git add ... && git commit ...
cp -rn /home/alvaro/.cache/git-objects-<name>/* .git/objects/ 2>/dev/null
unset GIT_OBJECT_DIRECTORY
git cat-file -t <sha>              # ALWAYS verify
```

### Fast Facts

| Fact | Value |
|------|-------|
| Backend | Rust + Axum 0.8, one binary |
| Frontend | SvelteKit 5, static adapter |
| Database | PostgreSQL 16 + pgvector |
| Cache/queues | Redis |
| LLM | Ollama, `lfm2.5:8b` default on deploy |
| Migrations | 34 (1-34), applied at boot |
| Roles | 0 user, 1 curator, 5 mod, 10 admin |
| Body cache | `/public/literature/fichub/bodies` (sharded JSON) |
| Rec mode | `legacy` live; `pluggable` with shadow mode |
| Self-healing | M1 telemetry live; agent loop OFF |
| Modlog | migration 034, public to logged-in users |
| Analytics | migration 033, zero-PII (X-Client-ID) |

### The File You Should Print

If you remember ONE file, remember **`src/server.rs`** — specifically
`build_router()`. It's the map of the whole product. Every route, grouped
by area. When you're lost, open it and find your feature.

Second place: **`src/config.rs`** — every knob. Third:
**`src/db/queries.rs`** — every query. Together those three files
are 80% of "where does X live."

### A Final Note on Asking for Help

When you get stuck, the fastest path to an answer in this codebase:

1. `grep` for the term in `src/` (route names, function names).
2. Read the route file for the area.
3. Check the docs (ROADMAP, SPECIFICATION, skill references).
4. If still stuck, ask with specifics: "In `src/routes/requests.rs`, the
   candidates endpoint returns empty for a request without a seed work —
   is that expected?"

Specific questions get fast answers. The codebase is well-organized;
the answer to most questions is a grep away.

---

*This onboarding guide was generated for the FicHub codebase on
2026-08-11. The repo evolves; if a path or behavior here has changed,
trust the code and update the book.*
