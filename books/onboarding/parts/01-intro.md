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
