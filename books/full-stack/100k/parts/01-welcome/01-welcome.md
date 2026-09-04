# Part 1 — Welcome to Your Fanfiction Platform

> **Part 1 of 13** — This is where it all starts. By the end of this part you
> will understand *what* FicHub is, *which tools* build it, *where everything
> lives* in the repository, and you will have **run the entire platform on
> your own machine** for the first time. Every later part assumes you made it
> through Chapter 4, so take your time.

---

## Chapter 1 — What We're Building: The Big Picture

Every great project starts with a story. Ours is about a magic bookshelf.

Fanfiction is scattered across dozens of websites. The biggest is **Archive
of Our Own (AO3)**. There's **FanFiction.net**, one of the oldest archives
around. There's **FictionPress** for original fiction, and weird, wonderful
forum-based archives like **SpaceBattles** and **SufficientVelocity** where
stories grow one thread-post at a time. Each site has its own layout, its own
search, its own rules. Finding a good story can feel like searching for
treasure without a map.

**FicHub** is the map. It's a self-hosted fanfiction archive and download
server — a "fichub.net replacement", as the repository's own description puts
it. Paste any story link into FicHub, and it:

1. **Grabs the story** from the original site (that's the *scraper*).
2. **Cleans it up** into structured metadata and clean text.
3. **Wraps it into a file** — EPUB, HTML, MOBI, PDF, AZW3, TXT, or Markdown.
4. **Saves it** so anyone can re-read it instantly — even offline.

But that's just the download half. FicHub is also a **full community
platform**. Let me show you the repository's own summary, straight from
`README.md`, because it's the best one-sentence description of the project
that exists:

> Scrapes fanfiction sites, generates EPUB, HTML, MOBI, PDF, AZW3, TXT,
> Markdown. Serves a full community platform via HTTP API: user accounts,
> bookmarks, 5-star ratings, reviews, threaded comments, Fic Requests (prompt
> board), reading lists, follows + updates feed, series/author pages,
> RSS/Atom feeds, in-browser reader, roadmap consensus, and an anti-bot
> defense system.

Read that sentence again, slowly. Count the features. Accounts. Bookmarks.
Ratings. Reviews. Comments. Requests. Lists. Follows. Feeds. A reader. A
voting system for the project's own roadmap. An anti-bot defense system.
**This is not a toy.** This is a production-grade platform with more moving
parts than most commercial apps — and over the next 100,000 words, we're
going to build it together, line by line, starting from this very repository.

### The problem FicHub solves

The official documentation (`docs/src/what-is-fichub.md`) puts it more
playfully — FicHub is "like a magic bookshelf for fanfiction. It takes
stories from different websites and puts them all in one place." It lists the
scattering problem we just talked about, then describes the solution in four
points:

1. **Brings stories together** — paste any link and FicHub downloads it
2. **Makes them easy to read** — converts to EPUB, HTML, or PDF so you can read on any device
3. **Helps you find more** — smart search, recommendations, and bookmarks
4. **Stays honest** — a public moderation log shows every staff action, and
   usage analytics are non-PII (anonymous client IDs only, never IPs)

That fourth point is rare and wonderful. This is a community platform that
believes **transparency is a feature**. We'll build the modlog in Part 11 and
see exactly how every moderator action becomes a permanent, public record.

### The architecture at a glance

Here's the "big picture" diagram from `README.md`. Don't worry about
understanding every line yet — I want you to see the *shape* of the system:

```
User/Script → Axum HTTP server (:8000)
        │
        ├─► Redis ─────────────── (rate limiter token bucket, shadowban)
        ├─► PostgreSQL ────────── (works, fic metadata, social, search, requests, lists)
        ├─► Ollama ────────────── (nomic-embed-text: roadmap consensus, auto-tagger, rec embeddings)
        ├─► Scraper subsystem ─── (FanFicFare CLI: 40+ sites; AO3/FFN blocked from this host)
        ├─► EPUB/TXT/MD builder ── (pure Rust, epub-builder + txt/md export)
        ├─► Calibre sidecar ───── (MOBI/PDF/AZW3 conversion via Docker)
        ├─► Rec strategy registry (REC_ENGINE_MODE=pluggable: RRF-blended
        │    strategies — cooccur/decay/embeddings/mf/hybrid/author_graph/
        │    tag_graph/sequential/clusters/bandit/external; legacy mode
        │    preserves the current engine exactly)
        └─► Filesystem cache ─── (hash-based directory tree)
        └─► ServeDir fallback ─── (frontend/build SPA, single origin)
```

Read it top to bottom. **One user or script hits one HTTP server** — the Axum
server on port 8000 — and that server talks to a whole *constellation* of
systems behind the scenes: a Redis cache for rate limiting, a PostgreSQL
database for all the data, Ollama running local AI models for embeddings and
text generation, a scraper subsystem that goes out and fetches stories from
other sites, an EPUB builder written in pure Rust, a Calibre sidecar for the
formats that are too heavy to build ourselves, and a filesystem cache so we
never do the same work twice.

Two details in this diagram are worth pausing on:

- **"AO3/FFN blocked from this host"** — the real deployment can only reach
  RoyalRoad, Quotev, and Wattpad (AO3 returns error 525, FFN returns 403).
  This is the messy reality of production: sometimes the sites you need to
  talk to won't talk back. We'll build graceful fallbacks anyway.
- **"frontend/build SPA, single origin"** — the frontend is a *single-page
  application* (SPA) whose compiled files are served *by the same Rust
  server* that serves the API. One port, one origin, no CORS headaches in
  production. We'll see how that works in Chapter 3.

💡 **Key Concept — One origin, many systems**
FicHub is a **monolith with a rich back end**: one server process that
orchestrates many external systems. That's a deliberate architecture
decision. When your frontend and API live on the same origin
(`fichub.example.com/api/*` and `fichub.example.com/`), browsers treat them
as one application — no cross-origin security dance, no separate deployment
for the UI, one binary to ship. Modern "microservices" get a lot of press,
but most real-world products start exactly like this: one well-organized
server that knows how to delegate.

### The unified works model

Here's the first *real* piece of domain thinking in FicHub, and it's
brilliant: **a story is not a URL. A story is a work.** The same fanfiction
posted on AO3, FanFiction.net, and a forum is *one story* with *three
sources*. The README calls this the "Unified Works Model":

- **works** table: auto-increment PK, canonical title/author, description
- **fic_info**: sources linked to works via `work_id` FK
- **Auto-merge**: exact title+author match with word count tolerance
- **Curator proposals**: community-driven merge/split with voting
- **Reputation**: contributors earn points, auto-promote to curator

We'll see the actual SQL for `works` in a moment (Chapter 3), and we'll spend
real time with auto-merge and curator proposals in later parts. For now,
grab this idea: **identify the thing, not the link**. It's the kind of
decision that separates an app that works from an app that works *well*.

### A tour of the feature highlights

`README.md` lists about fifteen feature highlights. Skim them with me, and
notice how many of them are things *you* will build in this book:

- **5-star ratings + in-depth reviews** — positive-only public surface; no
  dislikes shown; reviews feed the recommendation engine
- **Web reader** (`/read/[urlId]`) — typography prefs, chapter nav, scroll
  progress, position save, "Next Up" panel
- **Fic Requests** (`/requests`) — a prompt board where readers post ideas
  and writers answer with works
- **Reading lists + shelves** (`/lists`, `/shelves`) — curated bundles
- **Follows + updates feed** — follow fics, authors, and users
- **Series & author pages** (`/series/[id]`, `/authors/[id]`)
- **RSS/Atom feeds** — `/feed.xml` and per-fic feeds
- **PWA offline reader** — a service worker that caches reader responses
- **Roadmap consensus** — a MaxDiff/Elo arena where users vote on what to
  build next, powered by pgvector embeddings
- **Advanced search** — boolean AND/OR/NOT, phrases, fielded search, typo
  tolerance
- **Ask the Archive** (`/ask`) — natural-language queries turned into search
  filters by a local LLM
- **Similar-fic suggestions** — community-voted recommendations
- **A pluggable recommendation platform** — ten strategies behind one knob
- **Transparent modlog** and **non-PII usage analytics**
- **Anti-bot** — honeypots, tiered rate limits, proof-of-work challenges

That's not a list of features; that's a **curriculum**. Every chapter of this
book maps to one of these bullets. By the end, you'll have touched every
single one of them.

### The two biggest ideas to take away

Before we move on, two things to remember for the whole book:

1. **FicHub is a work-centric system.** The `works` table is the heart of the
   platform; everything else — sources, bookmarks, ratings, comments,
   requests — hangs off it. When in doubt about where data lives, look for
   the work.
2. **FicHub is honest by design.** The modlog is public. Analytics never
   store IPs. Scraper failures are visible. Building transparent systems is
   *easier* than building secretive ones, and it makes users trust you.

🧪 **Try It Yourself — Read the README like a detective**
Open `README.md` in the repo root and find: (a) the exact `cargo run`
command in Quick Start, (b) the name of the environment variable that
controls the server port, and (c) the three things the "Quick Start"
section lists as prerequisites. Then look at the API endpoints table and
find the endpoint that exports an EPUB — it takes a `?q=` query parameter.
Write down the URL you'd use to export a fic.

⚠️ **Watch Out — The README lies in small ways (on purpose)**
The README says the server starts on `:8000 (or $PORT)`, and the
`docker-compose.yml` sets `PORT: 8004`. The `.env.example` says `PORT=3000`.
Which is it? **All of them** — the port comes from the environment, and
different environments set different values. This is your first lesson in
production reality: configuration lives outside the code. Never assume a
port; read the config. We'll build our own `.env` in Chapter 4.

---

## Chapter 2 — Your Toolkit: Rust, Axum, SvelteKit, PostgreSQL, Redis, Ollama

Every craftsperson has a toolbox. Before we touch a single file in depth,
let's get to know the six tools we'll be using — and *why* FicHub's authors
chose exactly these six. This chapter is about the *why*, so that when you
see the *how* in later parts, it clicks instead of confusing you.

### 2.1 Rust — the language of the back end

The backend is written in **Rust**. If you've only written JavaScript or
Python so far, Rust will feel strange for a week and then feel amazing.
Here's why it's a perfect fit for a platform like this:

- **It's fast.** Rust compiles to native machine code. A Rust server handles
  thousands of requests per second on modest hardware — the kind of
  hardware a self-hoster actually has (this project literally runs on a
  ThinkCentre, a small desktop PC).
- **It's safe.** The compiler *refuses to build* code with use-after-free
  bugs, data races, or null-pointer dereferences. Whole categories of bugs
  that plague other languages are impossible here.
- **It's honest.** No hidden exceptions, no implicit `null`. If a function
  can fail, it says so in its return type, and you handle it.

Let's look at the real `Cargo.toml` — Rust's manifest file, where every
dependency is declared. I'll quote the top of it exactly as it appears in
the repo:

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"
description = "Self-hosted fanfiction download server (fichub.net replacement)"
# The web server (src/main.rs) is the binary `cargo run` should pick when no
# --bin is given. The [[bin]] entries below are one-shot CLI tools. Explicit
# default-run keeps `cargo run` (and the hermes verify boot phase) from
# erroring on multiple binaries.
default-run = "fichub"
```

Line by line:

- `name = "fichub"` — the crate (Rust's word for a package) is called
  `fichub`. This is also the binary name.
- `version = "0.1.0"` — semantic versioning. 0.x means "we're still
  iterating, don't expect stability yet".
- `edition = "2024"` — the Rust language edition. Editions are how Rust
  evolves without breaking old code; 2024 is the newest.
- `description` — one line that tells you exactly what this is.
- `default-run = "fichub"` — this is a *fascinating* production detail.
  The repo contains **seven** binaries (one web server plus six CLI tools).
  When you type `cargo run`, Cargo needs to know *which* binary you mean.
  Without `default-run`, `cargo run` would refuse to run and ask you to
  disambiguate. With it, `cargo run` always boots the server. The comment
  even mentions the "hermes verify boot phase" — an automated check that
  boots the server during development. We'll meet the six CLI tools in
  Chapter 3.

The `[dependencies]` section is a beautiful crash-course in what a modern
Rust web service needs. Let me show you the highlights (trimmed to the
important lines):

```toml
# Web framework & runtime
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.7", features = ["cors", "compression-gzip", "trace", "fs"] }

# Database
sqlx = { version = "0.9", default-features = false, features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate", "tls-rustls-ring", "derive", "macros"] }

# Redis
redis = { version = "1.4", features = ["aio", "tokio-comp"] }

# HTTP client
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }

# HTML parsing
scraper = "0.27"

# EPUB generation
epub-builder = "0.8"

# Template engine
tera = "2"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Let me translate the key ones into plain English:

- **`axum`** — the web framework. It's the thing that turns "a URL arrives"
  into "your function runs". We chose Axum because it's built on `tokio` and
  `tower`, it's fast, and its API is delightfully small. We'll spend many
  chapters inside it.
- **`tokio`** — the async runtime. Rust can run *millions* of concurrent
  lightweight tasks on a handful of OS threads, and Tokio is what makes that
  possible. `features = ["full"]` means "give us everything".
- **`tower` / `tower-http`** — middleware infrastructure. Middleware is code
  that wraps your routes: logging, CORS headers, gzip compression. You'll
  see the `TraceLayer` and `CorsLayer` in Chapter 4.
- **`sqlx`** — the database library. Two things make it special: it talks to
  PostgreSQL *natively* (no ORM magic hiding your SQL), and its `migrate`
  feature applies your `migrations/` folder automatically on startup. Also
  notice the features list: `runtime-tokio`, `postgres`, `migrate`,
  `tls-rustls-ring`… every capability is explicitly opted in. Rust culture
  is "opt in to exactly what you need".
- **`redis`** — the Redis client, with `aio` (async) support.
- **`reqwest`** — the HTTP client the scraper subsystem uses to fetch pages
  from AO3, FFN, and friends. Note `default-features = false` plus explicit
  features: they chose `rustls-tls` (a pure-Rust TLS implementation) over
  the default `native-tls`. Again: explicit > implicit.
- **`scraper`** — an HTML parsing library. This is how we read the messy
  markup of other people's websites.
- **`epub-builder`** — a pure-Rust EPUB generator. One crate gives us a
  whole export format.
- **`serde` + `serde_json`** — serialization. `serde` is Rust's
  industry-standard way to turn structs into JSON and back. You'll see
  `#[derive(Serialize)]` on a *lot* of types in this codebase.

And down below, the two features that made me smile:

```toml
# v2: Auth
jsonwebtoken = "9"
bcrypt = "0.16"
axum-extra = { version = "0.10", features = ["typed-header"] }
```

`jsonwebtoken` (JWTs for login sessions) and `bcrypt` (password hashing) are
exactly the right tools for authentication — and the comment "v2: Auth" tells
you this was added as a second wave of features. Real projects evolve in
waves. We'll build all of this in Part 7.

💡 **Key Concept — The Cargo.toml is a map of the system**
Before reading a single line of Rust code, you can learn 80% of a project's
architecture by reading its `Cargo.toml`. Every dependency is a promise
about what the system does: `reqwest` + `scraper` means "we fetch and parse
web pages"; `epub-builder` means "we generate ebooks"; `jsonwebtoken` +
`bcrypt` means "we have authentication". Whenever you join a new Rust
project, start with the manifest — it's the table of contents of the code.

🧪 **Try It Yourself — Count the binaries**
Open `Cargo.toml` and scroll to the `[[bin]]` entries. Count them, and note
each tool's name and source file (they all live in `src/bin/`). Can you
guess what `compute-stats` does from its name alone? Check your guess by
reading the first comment block of `src/bin/compute_stats.rs`.

### 2.2 Axum — the web framework (a first look)

Axum deserves its own mention because it *is* the back end. Its core idea is
brilliantly simple: **a handler is just an async function that takes the
request pieces it needs and returns a response.** Axum figures out the rest
through types.

Here's the real health-check handler from `src/routes/health.rs` — the
simplest endpoint in the whole codebase, and the perfect first taste of
Axum:

```rust
pub async fn health_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HealthQuery>,
) -> impl IntoResponse {
    let mut db_ok = true;
    let mut redis_ok = true;

    // Check PostgreSQL if not skipped
    if !params.skip_db {
        db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&state.db)
            .await
            .is_ok();
    }
    ...
    let status = if db_ok && redis_ok {
        "ok".to_string()
    } else if !db_ok && !redis_ok {
        "error".to_string()
    } else {
        "degraded".to_string()
    };

    // Return 503 if not fully healthy
    let status_code = if status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        status,
        db: db_ok,
        redis: redis_ok,
        version: "0.2.0".to_string(),
    };

    (status_code, Json(response))
}
```

Don't worry about the details yet (we dissect `AppState` in Part 2). Just
notice the *shape*: the handler **declares what it needs** — `State` (shared
application state) and `Query` (URL parameters) — and Axum's type system
hand-wires them. If you've written Express or Flask, this is the same idea
with compiler-checked wiring. We'll live inside Axum for the next 11 parts.

### 2.3 SvelteKit 5 — the frontend

The frontend is a **SvelteKit 5 single-page application**. Svelte is the
framework where *the compiler does the work*: instead of shipping a big
runtime that diffs the DOM in the browser, Svelte compiles your components
into tiny, efficient JavaScript at build time. SvelteKit 5 is the
meta-framework built on top of it — it gives you routing, layouts, and
server-side rendering out of the box.

Let me show you the real `frontend/package.json` (trimmed to the scripts):

```json
{
  "name": "fichub-frontend",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:e2e": "vitest run --config vitest.e2e.config.ts",
    "coverage": "vitest run --coverage",
    "test:coverage": "vitest run --coverage",
    "test:e2e:browser": "playwright test"
  }
}
```

Notice a few things:

- **`dev`** runs `vite dev` — the development server with hot module
  replacement. This is what you'll run in Chapter 4.
- **`build`** runs `vite build` — producing the static files in
  `frontend/build/` that the Rust server serves in production.
- **Three different test layers**: `vitest run` (fast unit tests), a
  separate E2E config (`vitest.e2e.config.ts`), and `playwright test`
  (browser automation). This project takes testing seriously — CI fails if
  coverage on `src/lib/**` drops below 85% lines. We'll feel that pressure
  in Part 12.

And here's the key architectural decision, in `frontend/svelte.config.js`:

```js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      strict: false,
    }),
  },
};

export default config;
```

`adapter-static` means "render everything as static files". No Node server
needed for the frontend — just HTML, CSS, and JS in a `build/` folder. And
`fallback: 'index.html'` means: for any route we didn't pre-render, serve
`index.html` and let the client-side router take over. That's the "SPA"
piece — and it's why the Rust server can serve the whole frontend with a
single `ServeDir` line. The two halves of FicHub are built separately and
**joined at deployment**: `npm run build` writes files, `cargo run` serves
them.

💡 **Key Concept — Static adapter + SPA fallback**
A "single-page application" doesn't mean "one file". It means the browser
loads one HTML shell plus JavaScript, and then *JavaScript* decides what to
render as you click around — no page reloads. The `fallback: 'index.html'`
setting is what makes deep links like `/read/abc123` work: the server
always hands over the same shell, and the router figures out which screen
to show. Your "next page" is instant because it was already downloaded.

### 2.4 PostgreSQL — the memory that never forgets

All the *data* — users, works, ratings, comments, requests — lives in
**PostgreSQL 16**, an open-source relational database that's been evolving
for three decades. FicHub uses a few of its superpowers:

- **JSONB columns** — store flexible, semi-structured data (like raw scraped
  metadata) right next to relational rows.
- **Full-text search** — the `tsvector` generated columns you'll see in the
  schema give FicHub fast text search without a separate search engine.
- **pgvector** — the extension that turns PostgreSQL into a *vector
  database*, storing the 768-dimensional embeddings that Ollama produces so
  the recommendation engine can find "similar" stories. (We build this in
  Part 9.)
- **Migrations** — schema changes are versioned files in `migrations/`
  applied automatically at startup by sqlx.

The README says it best: PostgreSQL holds "works, fic metadata, social,
search, requests, lists" — the entire soul of the platform.

### 2.5 Redis — the memory that's in a hurry

**Redis** is an in-memory key-value store: a database that lives in RAM
instead of on disk, so reads and writes are microseconds fast. FicHub uses it
for things that need *speed* or *volatility*:

- **Rate limiting** — a token-bucket limiter per IP address (the "tiered"
  system from the architecture diagram).
- **Shadowbans** — quietly slowing down suspected bots instead of blocking
  them outright.
- **Job queues** — the bookmark-import worker drains a Redis list named
  `bookmark_import_queue`.

Notice the *division of labor*: PostgreSQL is the source of truth (durable,
queryable), Redis is the hot cache and the coordination layer (fast,
ephemeral). Good architects don't pick one database — they pick the right
tool per job. We'll get deep into the limiter in Part 3.

### 2.6 Ollama — the local brain

**Ollama** runs large language models on your *own machine*, no cloud, no API
keys, no per-token billing. FicHub uses it in three places:

1. **Roadmap consensus** — embedding feature requests with `nomic-embed-text`
   (768 dimensions) so the platform can cluster and rank them.
2. **Ask the Archive** — converting a natural-language query like "completed
   slow-burn Dramione over 50k, no major character death" into structured
   search filters.
3. **The auto-tagger** — suggesting tags for scraped fics with a chat model.

Here's the client code's own description, from `src/services/ollama.rs`:

```rust
//! Ollama client — local embeddings + small-model text generation.
//!
//! Ollama runs on localhost:11434 (already used by the QA triage worker).
//! The Roadmap Consensus Engine uses `/api/embeddings` (nomic-embed-text,
//! 768-d); comment moderation triage uses `/api/generate` (llama3.1:8b) for
//! cheap one-shot classification. Keeping this as a thin wrapper avoids
//! adding heavy Rust ML dependencies; the model is called over HTTP with the
//! shared reqwest client.
```

That comment is a masterclass in engineering honesty: "a thin wrapper avoids
adding heavy Rust ML dependencies". Instead of pulling in massive ML crates,
FicHub just makes an HTTP call to a local server. And notice how *graceful*
the failure mode is — a failed embedding "should not fail the suggestion
submission — the raw text is still stored". Real systems degrade
gracefully, and this one does.

⚠️ **Watch Out — Ollama is optional at runtime**
Ollama powers *features*, not the core. The server will start fine without
Ollama running — the LLM-dependent endpoints just fall back (Ask the
Archive has a "graceful fallback when the model is down", per the README).
When you first run FicHub in Chapter 4, don't panic if Ollama isn't
installed. For now, focus on PostgreSQL and Redis — those are the
non-negotiable ones. We'll install Ollama properly in Part 9.

### Your toolkit, summarized

| Tool | Role | Why it was chosen |
|------|------|-------------------|
| **Rust** | Backend language | Fast, memory-safe, honest errors |
| **Axum** | Web framework | Minimal, fast, built on Tokio/Tower |
| **SvelteKit 5** | Frontend framework | Compiler-based, tiny bundles, SPA + static adapter |
| **PostgreSQL 16** | Main database | Relational power + JSONB + pgvector + full-text search |
| **Redis** | In-memory store | Microsecond-speed rate limiting, queues, shadowbans |
| **Ollama** | Local AI | Embeddings + LLM features, no cloud, no API keys |

That's the stack. Six tools, each doing what it does best, all connected by
one HTTP server. In the next chapter, we'll walk through the repository and
see where each of these tools lives in the file tree.

🧪 **Try It Yourself — Spot the tool in the tree**
Using the table above, look at the top-level directory listing of the repo
(`ls` in the terminal) and match each item to a tool. Where does the
Rust code live? Where does the frontend live? Where do database schema
files live? Write your answers down — then read Chapter 3 and grade
yourself.

---

## Chapter 3 — The Repo Tour: Every Directory, Mapped

Grab a coffee (or cocoa — this is a cozy book). This chapter is a walk
through the entire repository, top to bottom, so that every later chapter
can say "open `src/routes/requests.rs`" and you'll know exactly where that
is and what neighborhood you're in.

The repo root is `/personal/documents/code/rust/fichub` (on your machine it
might be wherever you cloned it — every path in this book is relative to
the repo root unless stated otherwise). The `.gitignore` file tells you
what's *not* tracked: `/target` (Rust build output), `frontend/build/`,
`frontend/node_modules/`, `.env`, `tmp/`, `cache/`. Everything we're about
to map is the tracked source — the *actual project*.

A quick snapshot of the size of the thing we're learning:

- **~43,600 lines of Rust** across `src/` (20+ modules, 6 CLI tools)
- **34 SQL migrations** (~1,900 lines of schema)
- **178 source files** in `frontend/src/`
- **43 integration test files** in `tests/`
- **~207,000 total tracked lines** in the repo

Don't be intimidated — those numbers are *why* this book exists. Nobody
learns a 200k-line codebase by staring at it. You learn it by walking
through it, room by room, and that's exactly what we're doing.

### 3.1 The two halves of the monorepo

Open the repo and look at the top level. You'll see two big source trees
sitting side by side:

- **`src/`** — the entire Rust back end (server, scrapers, exports,
  search, recommender, database layer, CLI tools).
- **`frontend/`** — the entire SvelteKit front end (`src/`, config,
  tests, and the `build/` output that the Rust server serves in
  production).

This layout is called a **monorepo**: one repository holding multiple
projects that are developed and deployed together. The back end speaks HTTP
JSON; the frontend speaks JavaScript; the contract between them is the API
documented in `README.md`'s endpoint table. You'll feel this split in
every chapter — but you'll also feel why keeping them together works: a
schema change lands in the same commit as the code that uses it.

Let's start in the Rust half, because that's where the server lives.

### 3.2 `src/` — the Rust back end

Here's the real top of `src/main.rs` — the file that starts everything.
It's only 42 lines, and the first 21 of them are a *map* of the entire
back end:

```rust
pub mod body_cache;
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod fic_suggestions;
pub mod frontend;
pub mod heal;
pub mod ingest;
pub mod limiter;
pub mod modlog;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod services;
pub mod tags;
pub mod works;
```

That's Rust's module declaration syntax: every `pub mod X;` says "this
project has a module named X, and it lives in `src/X.rs` or `src/X/`".
Read the list as an index of the whole back end: `body_cache`, `cache`,
`config`, `db`, `error`, `export`, `fic_suggestions`, `frontend`, `heal`,
`ingest`, `limiter`, `modlog`, `recommender`, `routes`, `scrape`, `search`,
`server`, `services`, `tags`, `works`. Twenty modules. We'll visit every
single one before this book ends. (`src/lib.rs` declares the same list —
that's the library form of the crate, so integration tests in `tests/` can
import it.)

Now the tree, module by module:

- **`src/main.rs`** — the entry point: loads `.env`, sets up logging,
  builds `Config`, calls `server::run`. We dissect it in Chapter 4.
- **`src/lib.rs`** — the library root (same modules, plus test re-exports).
- **`src/server.rs`** — *the* heart: the `AppState` struct (all shared
  state), the boot sequence, and `build_router` — the function that
  registers every HTTP route (~255 of them).
- **`src/config.rs`** — the `Config` struct and `from_env()`: every
  environment variable, every default, every knob. 808 lines of pure
  configuration.
- **`src/error.rs`** — `AppError`: the application-wide error type and its
  conversion into HTTP responses. Every handler returns `Result<...,
  AppError>`.
- **`src/db/`** — the database layer: `mod.rs` (the connection pool +
  `sqlx::migrate!` runner), `models.rs` (row structs), `queries.rs`
  (**2,699 lines** — the giant query module), `reviews.rs`.
- **`src/routes/`** — one file per feature area: `export.rs`, `meta.rs`,
  `search.rs`, `reader.rs`, `social.rs`, `comments.rs`, `reviews.rs`,
  `requests.rs`, `lists.rs`, `series.rs`, `authors.rs`, `follows.rs`,
  `feed.rs`, `updates.rs`, `roadmap.rs`, `admin.rs`, `analytics.rs`,
  `modlog.rs`, `honeypot.rs`, `pow.rs`, `kindle.rs`, `reports.rs`,
  `user_export.rs`, `work_proposals.rs`, `quests.rs`, `badges.rs`,
  `trending.rs`, `locales.rs`, `auto_tag.rs`, `cache_download.rs`,
  `download.rs`, `upload.rs`, `health.rs`, plus subfolders `rss/` and
  `opds/`.
- **`src/scrape/`** — the scraper subsystem: `mod.rs` (core types and the
  `SiteScraper` trait), `registry.rs` (which scraper handles which URL),
  `compat_fichub_net.rs`, and `sites/` — one scraper per site: `ao3.rs`,
  `ffnet.rs`, `fictionpress.rs`, `adultfanfiction.rs`, `hpfanfic.rs`,
  `royalroad.rs`, `xenforo.rs`, and `fanficfare.rs` (the catch-all).
- **`src/search/`** — the search engine: `parser.rs` (the boolean query
  parser), `builder.rs`, `routes.rs`, `ask.rs` (the LLM-powered
  natural-language search), `suggest.rs`, `tags.rs`.
- **`src/recommender/`** — the pluggable recommendation platform:
  `registry.rs`, `strategy.rs`, and one module per strategy (`cooccur`,
  `decay`, `embeddings`, `mf`, `hybrid`, `author_graph`, `tag_graph`,
  `sequential`, `clusters`, `bandit`, `external`, `curator`), plus
  `ranker.rs` (RRF blending), `signals.rs`, `worker.rs` (the batch
  training pipeline), `engine.rs` (the legacy engine), `routes.rs`.
- **`src/export/`** — the format builders: `epub.rs`, `html_bundle.rs`,
  `txt.rs`, `md.rs`, `convert.rs` (Calibre sidecar for MOBI/PDF/AZW3),
  `fallback.rs`.
- **`src/limiter/`** — the tiered rate limiter: `mod.rs` (the `Tier`
  enum and `RateLimiter` trait) and `redis_bucket.rs` (the Redis token
  bucket implementation).
- **`src/services/`** — supporting services: `ollama.rs` (the local LLM
  client), `mailer.rs` (SMTP for Send-to-Kindle), `bookmark_import.rs`,
  `comment_triage.rs`, `auto_tagger.rs`, `pow.rs` (proof-of-work).
- **`src/heal/`** — the self-healing scrape agent: `classifier.rs`,
  `snapshot.rs`, `agent.rs`, `store.rs`.
- **`src/cache/`** — the filesystem cache (`disk.rs`, `mod.rs`).
- **`src/body_cache.rs`** — the fic-body cache: every scraped story body
  saved as JSON to `BODY_CACHE_DIR` — "the site is a cache of all
  gathered fanfiction".
- **`src/works/`, `src/tags/`, `src/ingest/`, `src/frontend/`** — the
  unified-works model, the community tag system, manual fic uploads, and
  frontend-serving constants.
- **`src/bin/`** — the six CLI tools: `assign_quests.rs`,
  `compute_stats.rs`, `compute_leaderboards.rs`, `bot_scorer.rs`,
  `backfill_scores.rs`, `seed_roadmap.rs`. Each is a separate binary you
  run by hand or from a cron job — batch jobs that recompute things (stats,
  leaderboards, bot scores) without touching request handlers.

Whew. Before you panic: you are *not* expected to remember this list. The
point of this chapter is that the names are *meaningful* — the module tree
is the feature list, organized the way the code is organized. Every future
chapter re-opens the exact files it needs.

💡 **Key Concept — Module names are navigation**
In a well-organized codebase, the directory tree is a map. `src/routes/`
is where HTTP handlers live; `src/db/queries.rs` is where SQL lives;
`src/scrape/sites/` is where "one file per website" lives. When you need
to find "where is X handled?", the folder structure answers before you
even open a file. As you read this book, practice *predicting* file
locations from feature names — it's a superpower in interviews and in
real codebases alike.

### 3.3 The migration story: `migrations/`

The `migrations/` folder is the database's diary: 34 numbered SQL files,
each one a step in the platform's evolution. The naming is a history
lesson all by itself — read these aloud:

```
001_initial_schema.sql
002_add_client_tracking.sql
003_follows_notifications_gamification_vector_translations.sql
004_shelves_reading_status.sql
005_manual_uploads.sql
006_chapter_translations.sql
007_admin.sql
008_author_merging.sql
009_bot_tracking.sql
010_search_analytics.sql
011_roadmap_consensus.sql
012_search_typo_tolerance.sql
013_add_kindle_email.sql
014_feedback_rework.sql
015_auto_tagger.sql
016_roadmap_consensus_statuses.sql
017_follow_updates.sql
018_fic_requests.sql
019_reading_lists.sql
020_series_authors.sql
021_comment_triage.sql
022_user_reports.sql
023_translation_review.sql
024_rating_verification.sql
025_tag_score_fixes.sql
026_per_fic_suggestions.sql
027_rec_platform.sql
028_i18n_seed.sql
029_scrape_failures.sql
030_agent_runs.sql
031_curator_content_overrides.sql
032_curator_fix_proposals.sql
033_usage_events.sql
034_modlog.sql
```

Read it as a product story: the platform starts minimal (`001`), adds
client tracking (`002`), then a *huge* social wave (`003`), then shelves,
uploads, admin, author merging, bot tracking, search analytics, the
roadmap consensus engine, typo tolerance, Kindle, feedback rework, the
auto-tagger, fic requests, reading lists, series and authors, comment
triage, user reports, translation review, rating verification, per-fic
suggestions, the recommendation platform, i18n seeds, scrape failure
tracking, the self-healing agent, curator overrides and fix proposals,
usage analytics, and finally the modlog (`034`). **You can see the product
grow in the file names** — that's the power of versioned migrations.

How are they applied? Automatically, at startup, by `src/db/mod.rs` — the
code we saw earlier. sqlx reads the folder, checks which migrations are
already applied (it records them in a `_sqlx_migrations` table, with
checksums), and applies the new ones in order, inside transactions. The
README has a hard-won warning: **never hand-insert rows into
`_sqlx_migrations`** — sqlx validates each migration's checksum against
the recorded value and panics with `Migrate(VersionMismatch(N))` if they
differ. (It happened with migration 008 once, and it broke startup until
the bogus row was deleted so sqlx could re-apply the file.) The schema is
*code*, and it's *verified*.

### 3.4 The frontend tree: `frontend/`

Now the other half. `frontend/` is a standard SvelteKit project:

- `frontend/src/` — all source (178 files).
- `frontend/src/routes/` — one folder per URL path: `+layout.svelte` (the
  app shell), `+page.svelte` (the home page), and feature folders —
  `read/` (the reader), `search/`, `ask/`, `bookmarks/`, `lists/`,
  `shelves/`, `follows/`, `updates/`, `feed/`, `requests/`, `roadmap/`,
  `leaderboard/`, `badges/`, `quests/`, `trending/`, `tropes/`,
  `stats/`, `modlog/`, `curator/`, `work-proposals/`, `series/`,
  `authors/`, `notifications/`, `admin/`, `blind-date/`, `reading/`, and
  a catch-all `[...slug]/`.
- `frontend/src/lib/` — everything that *isn't* a page: API clients,
  components, stores, i18n, PWA, utilities (this is the folder CI
  enforces coverage on).
- Config and tests: `svelte.config.js`, `vite.config.ts`,
  `playwright.config.ts`, `vitest.e2e.config.ts`, `e2e/`, `src/**/*.test.ts`.
- `frontend/build/` — *generated*, not source: the output of `npm run
  build`, which the Rust server serves in production. It's in
  `.gitignore`, so it won't exist until you build it.

SvelteKit's routing rule is delightfully literal: **the filesystem is the
route table.** `frontend/src/routes/search/+page.svelte` renders at
`/search`. A `+layout.svelte` wraps every page beneath it. And because the
adapter is static (Chapter 2), all of this compiles to plain files in
`build/` — which is why the Rust server can serve the entire frontend with
one `ServeDir` line.

Let's meet the app shell, `frontend/src/routes/+layout.svelte` — the file
that defines the tabbed UI every visitor sees (this is a trimmed excerpt;
we'll read the whole thing in Part 12):

```svelte
<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import DownloadTab from '$lib/components/DownloadTab.svelte';
  import RecommendationsTab from '$lib/components/RecommendationsTab.svelte';
  import HomeDashboard from '$lib/components/HomeDashboard.svelte';
  import AuthBar from '$lib/components/AuthBar.svelte';
  import LocaleSelector from '$lib/components/LocaleSelector.svelte';
  import NotificationBell from '$lib/components/NotificationBell.svelte';
  import NavDropdown from '$lib/components/NavDropdown.svelte';
  import OfflineIndicator from '$lib/components/OfflineIndicator.svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { exportUserData } from '$lib/api/social';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { i18n, t, initI18n, setLocale } from '$lib/i18n/index.svelte';

  let { children } = $props();

  // Determine if we're on a route page (needs {children}) or a tab page
  const routePages = ['/search', '/ask', '/leaderboard', '/bookmarks', '/fic/', '/notifications', '/follows', '/updates', '/badges', '/trending', '/stats', '/roadmap', '/tropes', '/blind-date', '/feed', '/quests', '/read/', '/work/', '/curator', '/requests', '/series', '/authors', '/lists', '/shelves', '/admin', '/admin/auto-tag', '/admin/comment-triage', '/admin/blacklist', '/admin/stats', '/curator/flags', '/work-proposals'];
  let isRoutePage = $derived(routePages.some(p => $page.url.pathname.startsWith(p)));

  type Tab = 'home' | 'download' | 'recs';
  let activeTab = $state<Tab>('home');

  const discoverLinks = [
    { href: '/trending', label: t('nav.trending'), icon: '🔥' },
    { href: '/leaderboard', label: t('nav.rankings'), icon: '🏆' },
    { href: '/tropes', label: t('nav.tropes'), icon: '🧭' },
    { href: '/ask', label: t('nav.askTheArchive'), icon: '🗣️' },
    { href: '/roadmap', label: t('nav.roadmap'), icon: '🗺️' },
    { href: '/blind-date', label: t('nav.blindDate'), icon: '🎲' },
    { href: '/requests', label: t('nav.requests'), icon: '🙋' },
    { href: '/work-proposals', label: t('nav.workProposals'), icon: '🗳️' },
  ];
</script>
```

This is Svelte 5 *runes* in the wild — `$state`, `$derived`, `$props` are
the new reactive primitives, and `t('nav.trending')` shows that every label
is a translation key. Read the imports and you've read the app's DNA:
`AuthBar` (login state), `LocaleSelector` (six languages),
`NotificationBell` (alerts), `OfflineIndicator` (PWA), `i18n` + `t`
(internationalization), and `auth` from a runes-based store. The
`routePages` array is the *frontend's* route table — the list of real
pages that get the `{children}` layout treatment. We'll build the whole
`i18n` system in Part 12.

And the root page, `frontend/src/routes/+page.svelte`, is famously tiny:

```svelte
<script lang="ts">
  // The root page is intentionally empty: the tabbed UI lives in +layout.svelte.
  // This page renders nothing so the layout's Download tab is the default view.
</script>
```

That's it. Four lines, and the comment explains the architecture: the
layout owns the tabs; the page renders nothing. Minimalism with a comment
isn't laziness — it's design, documented.

Now the *contract* between the two halves. Open
`frontend/src/lib/api/client.ts` — the API client for the Rust backend:

```ts
// API client for the FicHub Rust backend (v0 API).
// All requests go to relative /api/* paths so they work behind the
// same-origin Rust server or any nginx proxy.

import type { ExportResponse, RecommendationsResponse } from './types';

const BASE = '/api';

// Client ID management for anonymous usage tracking
function getClientId(): string {
  const STORAGE_KEY = 'fichub_client_id';
  let clientId = localStorage.getItem(STORAGE_KEY);
  if (!clientId) {
    // Generate UUID v4
    clientId = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
    localStorage.setItem(STORAGE_KEY, clientId);
  }
  return clientId;
}
```

Three details to treasure:

1. **`const BASE = '/api'`** — a *relative* path. The frontend and API
   share one origin, so the client never needs a full URL. This is the
   "single origin" decision made concrete.
2. **Anonymous client IDs, not IPs** — the app generates a UUID v4, stores
   it in `localStorage`, and sends it with every request. This is the
   foundation of the "non-PII usage analytics" promise. Even *anonymous*
   tracking is designed to avoid personal data.
3. **`request<T>` wraps everything** — the exported functions build on a
   tiny core that adds the `X-Client-ID` header, throws a typed
   `ApiError` on failure, and parses JSON:

```ts
async function request<T>(path: string, init?: RequestInit): Promise<T> {
  // Add client ID header to all requests
  const headers = new Headers(init?.headers);
  headers.set('X-Client-ID', getClientId());

  const res = await fetch(`${BASE}${path}`, { ...init, headers });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new ApiError(res.status, text);
  }
  return (await res.json()) as T;
}
```

And the exported function that starts the download flow (and its
metadata-only sibling):

```ts
/** GET /api/epub?q=<url> — export a fic and return download URLs. */
export async function fetchExport(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/epub${buildQuery({ q: url })}`);
}

/** GET /api/meta?q=<url> — fetch fic metadata without downloading. */
export async function fetchMeta(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/meta${buildQuery({ q: url })}`);
}
```

TypeScript *documents the API surface*: the comment tells you the exact
endpoint and its parameters; the return type `ExportResponse` (defined in
`types.ts`) tells you the JSON shape. When the Rust side changes, the
TypeScript side is where you'll feel it — and the tests will catch you.

⚠️ **Watch Out — There are TWO route tables in this repo**
The back end's route table lives in Rust (`src/server.rs`), the
frontend's in filesystem folders (`frontend/src/routes/`). When
something 404s, ask yourself *which* router rejected it. A URL like
`/api/bookmarks` is handled entirely by Rust. A URL like `/bookmarks` is
a SvelteKit page that calls Rust underneath. Mixing the two mental
models is the #1 junior confusion in full-stack codebases — you now know
better.

### 3.5 `tests/` — the back end's security guard

The `tests/` folder holds **43 integration test files** — Rust programs
that boot the real server against a test database and hammer every API
area. The names are the feature map again: `search_api.rs`, `social_api.rs`,
`requests_api.rs`, `roadmap_api.rs`, `modlog_api.rs`, `rec_strategies.rs`,
`rec_embeddings.rs`, `honeypot_api.rs`, `pow_api.rs`, `blind_date_api.rs`,
`kindle_api.rs`, `user_export_api.rs`, `bookmark_csv_api.rs`… and the
harness itself in `integration.rs`. These tests are the reason the platform
can grow 34 migrations deep without collapsing. We study the strategy in
Part 13.

### 3.6 The rest of the top level

- **`docs/`** — the full documentation site (mdBook): `docs/src/*.md` is
  the readable source (`what-is-fichub.md`, `quickstart.md`, `searching.md`,
  `features.md`, `transparency.md`, `anti-bot.md`…), built into
  `docs/book/`. Plus the big `SPECIFICATION.md` and `FICHUB_DESIGN.md`.
- **`docker/` + `docker-compose.yml`** — the Calibre sidecar container
  (MOBI/PDF/AZW3 conversion) and the containerized app.
- **`Dockerfile`, `deploy.sh`** — deployment assets.
- **`qa/`** — the QA bug-triage harness (`qa/run.js`).
- **`rec-engines/`, `templates/`, `scripts/`, `bin/`** — supporting bits.
- **`books/`** — where *you* are right now. This book lives in the same
  repository it teaches. That's very FicHub.

### The map in one table

| You need… | Look in… |
|-----------|----------|
| The server entry point | `src/main.rs` |
| Shared state & the router | `src/server.rs` |
| All config knobs | `src/config.rs` |
| Database pool + migrations | `src/db/mod.rs` |
| Every SQL query | `src/db/queries.rs` |
| The schema story | `migrations/001…034*.sql` |
| Request handlers | `src/routes/*.rs` |
| Scrapers | `src/scrape/` + `sites/` |
| Export formats | `src/export/` |
| Search parser | `src/search/parser.rs` |
| Recommendation strategies | `src/recommender/` |
| Rate limiting | `src/limiter/` |
| CLI tools | `src/bin/` |
| The SPA pages | `frontend/src/routes/` |
| Frontend logic & API clients | `frontend/src/lib/` |
| Frontend build output (served!) | `frontend/build/` |
| Integration tests | `tests/` |
| The docs site | `docs/src/` |

🧪 **Try It Yourself — Draw the request path**
Pick any feature page from the frontend routes list (say `/requests`).
Draw a diagram of what happens when someone visits it: which SvelteKit
route folder renders it, which API client module it imports, which Rust
handler serves the `/api/requests` route, which query module it calls,
and which table in `migrations/` holds the data. You don't need to read
all the code — just trace the names. This "name tracing" skill will make
every later chapter faster.

---

## Chapter 4 — Running It for the First Time

Time for the moment of truth: **boot the whole platform.** By the end of
this chapter you'll have the API responding, the frontend dev server
running, and a health check that proves the entire stack is alive. This is
the chapter where "their project" becomes "your project".

### 4.1 The prerequisites

The README's Quick Start is blunt about what you need:

> Prerequisites: PostgreSQL, Redis, Ollama (for consensus/auto-tagger), Rust
> toolchain, Node.

Note the honest ordering: **PostgreSQL and Redis are non-negotiable.** The
server refuses to start without them (we'll see the exact code that
refuses, in a moment). Ollama is needed only for the AI features. Rust
toolchain and Node are the two compilers/builders — one for the back end,
one for the frontend. If you don't have them yet, this is the moment to
install them (and remember to run `npm install` in `frontend/` before the
frontend commands below).

### 4.2 The `.env` file — configuration without code

FicHub follows the Twelve-Factor convention: **configuration lives in the
environment, not in the source.** The repo ships a template, `.env.example`,
with *safe* placeholder values:

```bash
DATABASE_URL=postgres://fichub:***@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=./cache
SECONDARY_CACHE_DIR=
EXPORT_VERSION=1
DYNAMIC_RATE_LIMIT=true
NODE_NAME=orion
CALIBRE_CONTAINER=
PORT=3000
FRONTEND_DIR=./frontend/build
TMP_DIR=./tmp
RUST_LOG=info,fichub=debug
```

Your first move: **copy it and edit it.**

```bash
cp .env.example .env     # Edit with your DB/Redis URLs
```

Now set your real values: your PostgreSQL URL (with your username and
password), your Redis URL (default `redis://localhost:6379` is almost
always right for a local install), and whatever port you want (`3000` or
`8000` are both fine locally). **Never commit `.env`** — the repo's
`.gitignore` lists `.env` in its first lines, because it contains secrets
(the real deployment's `.env` holds a `JWT_SECRET`, DB credentials, and
API keys).

⚠️ **Watch Out — `PORT=3000` in the example, `:8000` in the README, `8004` in Docker**
We flagged this in Chapter 1. The `.env.example` says `3000`, the README
says the server starts on `:8000 (or $PORT)`, and `docker-compose.yml`
sets `PORT: 8004`. There is no contradiction — the port is *always* read
from the environment at boot, and each deployment picks its own. If you
change `PORT` in your `.env`, the server listens there. When something
"won't start", checking *which port the config actually asked for* is
always step one.

### 4.3 How the configuration is actually read — `config.rs`

Now the good stuff. The entire boot sequence starts in `src/main.rs` — 42
lines, the whole server in miniature:

```rust
#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

Walk through it with me:

1. **`#[tokio::main]`** — the attribute that turns `main` into an async
   entry point running on Tokio's runtime. This one line is why `.await`
   works everywhere in FicHub.
2. **`dotenvy::dotenv().ok()`** — loads the `.env` file into the process
   environment *if present*. The `.ok()` is a lovely idiom: "ignore the
   error if there's no .env" — the environment may already be set (that's
   how systemd deployments work: no `.env`, just real env vars).
3. **Tracing setup** — the logging system. `EnvFilter::try_from_default_env()`
   reads `RUST_LOG` if set; otherwise it falls back to `"info,fichub=debug"`
   — exactly the fallback string that's in `.env.example`.
4. **`let config = config::Config::from_env();`** — *the* configuration
   object, built from environment variables.
5. **`server::run(config).await`** — hand the config to the server and
   never return.

Everything interesting — and everything *strict* — is in
`Config::from_env`. Here is the top of the function, exactly as it appears
in the repo:

```rust
pub fn from_env() -> Self {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let redis_url = std::env::var("REDIS_URL")
        .expect("REDIS_URL must be set");

    let cache_dir = std::env::var("CACHE_DIR")
        .unwrap_or_else(|_| "./cache".to_string());

    let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    let export_version = std::env::var("EXPORT_VERSION")
        .unwrap_or_else(|_| "1".to_string())
        .parse()
        .unwrap_or(1);
    ...
}
```

Look at the *pattern* — three different strategies, and each one teaches
you a Rust idiom:

- **`expect("DATABASE_URL must be set")`** — *required*. If this env var
  is missing, the program **panics with your message before the server
  starts**. No database URL, no server. This is the code that enforces
  "PostgreSQL is non-negotiable".
- **`unwrap_or_else(|_| "./cache".to_string())`** — *optional with a
  default*. No `CACHE_DIR`? Use `./cache` and move on.
- **`.ok().filter(|s| !s.is_empty()).map(PathBuf::from)`** — *optional and
  possibly empty*. `SECONDARY_CACHE_DIR` is a real `Option<PathBuf>`:
  missing or empty both mean "no secondary cache". This is Rust's way of
  saying "this might just not exist, and that's fine".

The `Config` struct itself documents every knob with doc comments. Here's
a sample of the struct's fields (from `src/config.rs`):

```rust
/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    /// Where scraped fic bodies are cached as JSON blobs (the site is a
    /// cache of all gathered fanfiction). Defaults to the attach drive:
    /// /public/literature/fichub/bodies.
    pub body_cache_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    ...
}
```

The struct has **dozens** of fields (the recommender section alone has
twenty-plus `rec_*` knobs, the rate limiter has `rl_*` knobs, the
proof-of-work system has `pow_*` knobs). Every knob in that struct is a
story you'll read in a later part. For now, notice the elegance: the
entire platform's behavior — ports, caches, strategies, limits — is one
typed struct, loaded from the environment, documented in place.

💡 **Key Concept — Fail fast, fail loud**
`expect("DATABASE_URL must be set")` is *deliberate*: a server that boots
without its database is a server that fails mysteriously at 3 a.m. under
load. FicHub's philosophy is to refuse to start, with a clear message,
the instant its requirements aren't met. This is called **fail fast** —
and it's the opposite of "let it crash later". When you write your own
services, decide which config is *required* and make the program refuse
to start without it. Your future self will thank you at 3 a.m.

### 4.4 The boot sequence — what `server::run` does

Now let's see what happens *after* config. `server::run` in `src/server.rs`
is where the platform assembles itself, one connection at a time. Here's
the real code (trimmed):

```rust
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    // Dedicated connection for the health endpoint. Health must never share
    // the main multiplexed connection: the bookmark-import worker parks an
    // unbounded BRPOP on it, and a PING sent behind that block would time out
    // (health would report redis:false while Redis is perfectly healthy).
    let health_redis = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis for health checks");

    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    ...
    // Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
        .await
        .expect("Server error");
}
```

Notice the *boot order* — it's not random, it's dependency order:

1. **PostgreSQL pool** (via `db::init_pool`, which also *runs migrations*
   — we saw that in Chapter 3).
2. **Redis connection** — and not one but *two*: a main multiplexed
   connection and a dedicated one for health checks. The comment explains
   the subtle production bug this avoids: the bookmark-import worker
   parks an unbounded `BRPOP` on the shared connection, so a health-check
   `PING` queued behind it would time out and report `redis:false` even
   when Redis is perfectly healthy. **This is a real bug that was found
   and fixed — and the fix is documented in the code.** Never delete such
   comments.
3. **HTTP client** — with a 30-second timeout and an honest user agent:
   `"fichub.net/0.1.0"`.
4. **Scraper registry**, then the rate limiter, the recommendation
   engine, the strategy registry, the training-pipeline worker (a
   background task that re-trains every `REC_TRAIN_EVERY_H` hours), the
   bookmark-import worker (also backgrounded), and the shared `AppState`.
5. **Bind and serve** — `0.0.0.0:{port}` means "listen on every network
   interface", and `axum::serve` runs forever.

Every `.expect(...)` in this sequence is a *boot-time requirement*. If
PostgreSQL isn't reachable, the server dies instantly with "Failed to
connect to database" — again, fail fast. If you see that message in your
terminal, your database isn't up, and the fix is on the infrastructure
side, not the code side.

⚠️ **Watch Out — Background workers are part of the app**
`tokio::spawn` starts tasks that run *forever, in the same process*: the
rec-training pipeline and the bookmark-import worker. They're not
separate services — they're threads inside your server. If one of them
panics, the whole process can go down with it. When you see `tokio::spawn`
in production code, ask: "what happens if this task never returns?" — and
notice how carefully FicHub's workers are structured.

### 4.5 The router — where `/api/health` gets its address

Everything so far has been *setup*. The router is the actual map of the
server. `build_router` in `src/server.rs` chains ~255 route registrations
into one `Router`. Here's the real beginning of it:

```rust
/// Build the Axum router with all routes
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    // Redirect handler functions (avoid closures with async blocks)
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    Router::new()
        // API routes
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/epub", get(routes::export::epub_handler))
        .route("/api/meta", get(routes::meta::meta_handler))
        .route("/api/remote", get(remote_handler))
        .route("/api/health", get(routes::health::health_handler))
        ...
```

The syntax is the Axum idiom you'll see a thousand times in this book:
`Router::new().route("/path", get(handler))` — a path, a method, a handler
function. `get(...)` is how you say "this handler answers GET requests"
(`axum::routing::post(...)` says POST, and so on). Each route is a
promise: "when a request with this path arrives, run this function". The
whole platform — exports, search, auth, social, admin, feeds — is a chain
of promises like these, and you'll read the full chain together in Part 2,
Chapter 7.

And the *very last* link in the chain is the frontend. After all the API
routes, the router catches everything else and serves the SPA:

```rust
        // Static frontend
        .fallback_service(
            crate::frontend::cache_headers::CacheHeadersLayer
                .layer(
                    ServeDir::new(&frontend_dir)
                        .append_index_html_on_directories(true)
                        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
                ),
        )

        // Middleware
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::routes::analytics::track_usage,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
```

Three layers to love:

- **`ServeDir::new(&frontend_dir)`** — serves the *built* frontend
  (`frontend/build/`) as static files. The SPA you'll run in a moment, at
  the end of this chapter, is exactly what this serves in production.
- **`fallback(ServeFile::new(frontend_dir.join("index.html")))`** — the
  SPA fallback: unknown paths get `index.html`, and the client-side
  router decides what to show. Same-origin SPA, one port, done.
- **Middleware**: `track_usage` (the non-PII analytics from Chapter 1),
  `TraceLayer` (request logging), `CorsLayer::permissive()` (open CORS —
  fine for a public API; the real origin security comes from the API
  design, not CORS).

### 4.6 `cargo run` — start the back end

Let's actually do it. In the repo root:

```bash
cargo run
```

What happens (with `RUST_LOG=info,fichub=debug`, which you have in `.env`):

1. Cargo checks your `Cargo.lock` and downloads/compiles any missing
   crates. **The first build takes a while** — Rust compiles every
   dependency from source. Subsequent builds are fast.
2. `main` loads `.env`, builds `Config`, and `server::run` connects to
   PostgreSQL (applying any pending migrations — watch the log for
   "Database migrations applied") and Redis.
3. The router assembles, the workers spawn, and you'll see:
   `Listening on 0.0.0.0:8000` (or your port).

Then open a second terminal and poke it:

```bash
curl http://localhost:8000/api/health
```

If the whole stack is alive, you get the health response we built in
Chapter 2 — the JSON with `"status": "ok"`, plus `db: true`, `redis:
true`, and a version. If PostgreSQL isn't running, you'll instead get the
panic message from `server::run`: `Failed to connect to database`. The
fail-fast philosophy at work — the error message *tells you* what to fix.

🧪 **Try It Yourself — Three probes, one platform**
With the server running, run these three commands and compare what you
get:

```bash
curl http://localhost:8000/api/health      # the health JSON (db + redis)
curl http://localhost:8000/api/            # the API docs index
curl http://localhost:8000/                # the SPA shell (HTML!)
```

Notice: two of these come from the same process. The API docs at `/api/`
and the entire frontend at `/` are served by the same Rust binary on the
same port. That's the single-origin architecture made tangible. Then try
`curl http://localhost:8000/api/health?skip_db=true` — the query
parameters you saw in `HealthQuery` (`skip_db`, `skip_redis`) are real,
live switches for debugging.

### 4.7 `npm run dev` — start the frontend

The back end is up. Now the frontend dev server. Open `frontend/` and run:

```bash
cd frontend
npm run dev
```

`vite dev` starts the SvelteKit development server on **port 5173** —
with hot module replacement, so every save you make in `frontend/src/`
shows up instantly in your browser. But the interesting part is in
`frontend/vite.config.ts`, which we quoted in Chapter 2:

```ts
server: {
    port: 5173,
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://localhost:8004',
        changeOrigin: true,
      },
    },
  },
```

**The dev proxy.** The frontend runs on 5173; the Rust API runs on 8004
(in this repo's dev setup — remember, port comes from config!). The Vite
dev server *proxies* every request to `/api/*` to
`http://localhost:8004`, so the frontend's `fetch('/api/...')` calls work
without a single code change or CORS header. The TypeScript client from
Chapter 3 uses relative `/api/*` paths — and the proxy is why that works
in development. In production, the proxy isn't needed at all, because the
Rust server serves the built frontend directly on the same port.

⚠️ **Watch Out — The proxy target is config-dependent**
`vite.config.ts` proxies to `http://localhost:8004` — but your Rust
server might be listening on `3000` or `8000`, depending on your `.env`.
If the frontend loads but every API call fails with a network error,
check (a) which port your Rust server actually bound ("Listening on
0.0.0.0:PORT" in the log) and (b) whether the proxy target matches it.
This is *the* classic local-dev mismatch in this repo, and now you know
exactly where to look: one line in `vite.config.ts`.

### 4.8 What you should see

Open `http://localhost:5173` in a browser (or `http://localhost:8000`
directly if you want the production-style single-origin experience). You'll
be greeted by the FicHub home screen — the tabbed layout from
`frontend/src/routes/+layout.svelte`, which imports the big components we
mapped in Chapter 3: `DownloadTab`, `RecommendationsTab`, `HomeDashboard`,
`AuthBar`, `LocaleSelector`, `NotificationBell`, `NavDropdown`,
`OfflineIndicator`.

Try the **Download** tab: paste a story URL (from a reachable site like
RoyalRoad — remember, AO3/FFN are blocked from the production host, and
the same limitations apply to yours) and watch the frontend call
`fetchExport` → `GET /api/epub?q=<url>` → the Rust export pipeline. You
just used the full stack. It's real, it's yours, and you understand every
hop of that request now.

💡 **Key Concept — Dev mode ≠ production, on purpose**
In development you have *two* servers (Vite on 5173 + Rust on your
configured port) joined by a proxy. In production you have *one* (Rust
serving `frontend/build`). This is normal and healthy: dev mode gives you
hot reload and instant feedback; production gives you a single deployable
artifact. The same code, two arrangements — and the relative `/api/*`
paths are what make both work without changes.

### 4.9 Troubleshooting the first boot

Here are the four most likely things to go wrong, and the exact messages
that tell you what's wrong:

| Symptom | Message you'll see | Fix |
|---------|--------------------|-----|
| PostgreSQL isn't running | `Failed to connect to database` (panic) | Start PostgreSQL (`sudo systemctl start postgresql`), check the URL in `.env` |
| Redis isn't running | `Failed to connect to Redis` (panic) | Start Redis (`redis-server` or `sudo systemctl start redis`), check `REDIS_URL` |
| Port already in use | `Failed to bind to address` | Change `PORT` in `.env` (or kill the process using the port) |
| Frontend can't reach API | Network errors in the browser console | Check the proxy target in `vite.config.ts` matches your Rust port |

Every single one of these failures is *loud* — no silent half-start, no
mystery. That's fail-fast, and it's also good UX for a developer: the
terminal tells you exactly which subsystem to fix.

🧪 **Try It Yourself — Break it on purpose**
Now that everything works, *break it deliberately*. Stop PostgreSQL, run
`cargo run`, and watch the panic message. Start it again, stop Redis, and
do it again. Then change `PORT` in `.env` to something new and confirm
the log says `Listening on 0.0.0.0:<newport>`. You've now seen the
failure modes *before* they surprise you in the wild — that's what
experience is.

### 4.10 A map for the rest of the book

You've now touched every layer: the config that shapes the server, the
main function that boots it, the router that maps URLs, the two
databases, the two-route-table frontend, and the proxy that joins them in
dev. Hold onto this feeling of "the whole thing runs on my machine" —
it's the grounding for everything that follows.

Here's the road ahead, in one breath: in **Part 2** we'll rebuild
`main.rs` and `server.rs` line by line and meet `AppState`, the
dependency-injection heart of the platform. In **Part 3** we'll configure
every knob and read all 34 migrations. **Part 4** teaches the scrapers;
**Part 5** the export pipeline; **Part 6** search and the reader;
**Part 7** users, auth, and the social layer; **Part 8** requests, lists,
series, feeds, and the roadmap arena; **Part 9** the pluggable
recommendation platform; **Part 10** the LLM features; **Part 11** admin,
analytics, and the anti-bot war; **Part 12** the frontend deep dive; and
**Part 13** takes everything to production and ships it.

But before the code — a word about how this book works, what it expects
from you, and what you'll become by the time you finish it. One final
chapter to set the ground rules for the journey ahead: **Chapter 5 —
How to Use This Book: A Reader's Contract**. In it, we'll agree on what
you'll do (read code, run code, break code, re-read code), what you'll
need (curiosity, a terminal, and patience with compiler errors), and what
you'll become (someone who can open any unfamiliar repository and find
their way around). This book is a 100,000-word promise, and that chapter
is how we both keep it.

Then, with the map in our hands and the platform running on your machine,
we'll take our first real steps into the code — starting with the moment
your computer boots a web server from scratch. See you there.

