# FicHub — Complete System Specification

> **Generated**: 2026-07-25 (updated: 2026-08-25 — single-binary Rust stack, search v2 + Ask-the-Archive v2 rewrite, migrations 002–007, current deployment)
> **Author**: hirrolot19
> **Purpose**: Single-source-of-truth for LLM-assisted refactoring and development queries.

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Repository Layout](#2-repository-layout)
3. [Architecture & Data Flow](#3-architecture--data-flow)
4. [Backend: Rust (single axum binary)](#4-backend-rust-single-axum-binary)
5. [Backend: Go v1 (retired, merged into Rust)](#5-backend-go-v1-retired-merged-into-rust)
6. [Frontend: SvelteKit SPA](#6-frontend-sveltekit-spa)
7. [Database Schema](#7-database-schema)
8. [Deployment](#8-deployment)
9. [API Endpoint Reference](#9-api-endpoint-reference)
10. [Data Flow Examples](#10-data-flow-examples)
11. [Current Issues & Technical Debt](#11-current-issues--technical-debt)
12. [Environment Variables](#12-environment-variables)

---

## 1. System Overview

FicHub is a self-hosted fanfiction platform providing URL-to-EPUB export, metadata, caching, and a v2 social/search API (user accounts, bookmarks, ratings, comments, kudos, reviews, forums, collections, follows, quests, saved searches, recommendations, OPDS, advanced search). It is a **single Rust (axum) binary** that serves the API, the docs, and the SvelteKit SPA from one port. There are **no separate Go / threadlight backends** — the v1 API surface (accounts, fics, bookmarks, moderation, search) has been absorbed into this one Rust backend.

- **Backend**: one Rust binary (`/opt/fichub/fichub` in production), axum 0.8 + sqlx 0.9 (PostgreSQL) + redis 1.4. Runs the web server (`src/main.rs`, `[[bin]] name = "fichub"`, `default-run`) plus a set of one-shot CLI helper binaries (see §2).
- **Frontend**: a SvelteKit SPA living **inside the repo** at `frontend/`, built to `frontend/build` and served by the Rust binary via `ServeDir`/`ServeFile` (fallback to `index.html`). Also served: `/docs/*` (mdbook) and `/sw.js` (PWA service worker).
- **Search**: shared `parser → builder` boolean search pipeline ("search v2") used by `GET /api/search`, `POST /api/search/ask`, saved searches, and the watcher. "Ask the Archive" translates natural language to a **v2 query string** via Ollama and feeds it through the exact same pipeline.
- **Deployment host** (2026-08-14+): **ThinkCentre** dev box. The binary runs from **local disk** `/opt/fichub/fichub` (not the NFS repo, to avoid SIGBUS on wedged mounts), served on port `PORT` (8000 in prod), fronted by Cloudflare at `fichub.polarisocial.xyz`. The old Orange Pi deployment is retired.

### Current Deployment Status

| Component | Location | Port | Status |
|-----------|----------|------|--------|
| ficHub single binary (API + docs + SPA) | ThinkCentre `/opt/fichub/fichub` | `PORT` (prod 8000) | ✅ systemd `fichub.service`, `User=alvaro` |
| nginx (reverse-proxies public domain → :8000) | ThinkCentre | 8000 | ✅ fronts `fichub.polarisocial.xyz` |
| PostgreSQL | ThinkCentre | 5432 | ✅ local |
| Redis / Valkey | ThinkCentre | 6379 | ✅ local |
| Ollama (base URL `OLLAMA_URL`) | per-config | 11434 (default) | Used by Ask-the-Archive + embeddings |
| Fanfic-Archivist bot | `fanfic-archivist/` crate | — | Chat-bot integration (Discord/Telegram/Matrix) |

### Key Design Decisions

- **Single binary + single port**: the backend serves `/api/*`, `/docs/*`, and the static SPA together; the public domain is fronted by Cloudflare/nginx. No separate API daemon.
- **Shared search pipeline**: one `parser.rs → builder.rs` grammar is the single search language for search, Ask, and saved searches — they cannot drift.
- **Ask-the-Archive emits a v2 query string** (not the old 5-key `{q,min_words,max_words,complete,source}` JSON contract). `applied_params` in the response is now a *string*. Ollama down / validation failure degrades to a plain search with the raw NL as `q` and `translated: false`.
- **Migrations 002–007** layer new features on the consolidated `001_initial.sql` base (see §7).
- **Deployment from NFS-shared repo to local `/opt`**: repo is NFS-shared between gamingpc (fast build host) and thinkcentre; `deploy.sh` builds on gamingpc and copies the release binary to `/opt/fichub` (local disk) before restarting.
- **Svelte 5 runes** — state management uses `$state`, `$derived`, `$effect`; static adapter with `index.html` fallback.
- **Rust edition 2024**; scrapers are a separate internal crate `scrapers/` (`fanfic-scrapers`, AGPL), and a generic `forum_core/` crate (AGPL) powers the forum engine.

---

## 2. Repository Layout

The FicHub repository is the **root of the whole system** — backend + frontend + scrapers + bot live in one tree.

```
fichub/
├── Cargo.toml                            # [[bin]] fichub (default-run) + 8 helper bins
├── Cargo.lock
├── migrations/
│   ├── 001_initial.sql                   # Consolidated base schema (generated from live prod dump, 8.7k lines)
│   ├── 002_follow_exclusions.sql         # per-follow work/series/fandom exclusions
│   ├── 003_search_v2_polarity.sql        # tags.rel_polarity (rom/plat) generated column
│   ├── 004_search_v2_role_confidence.sql # fic_tags.role_confidence
│   ├── 005_search_v2_counters.sql        # works kudos/comments/bookmarks/hit counts, language, beta, dates
│   ├── 006_collection_submissions.sql    # collection_submission_votes
│   └── 007_saved_searches.sql            # saved_searches + saved_search_matches
├── src/
│   ├── main.rs                           # Server entry point (binary `fichub`); dotenvy + tracing + config + run
│   ├── lib.rs                            # Module declarations (server is a library for integration tests)
│   ├── config.rs                         # Config struct + all env-var loading (§12)
│   ├── server.rs                         # Axum router (all .route() calls) + startup + AppState
│   ├── error.rs                          # AppError enum + IntoResponse
│   ├── frontend/                         # Static SPA serving + cache headers
│   ├── db/{mod,models,queries,reviews}.rs
│   ├── routes/                           # HTTP handlers (one module per feature — see §9)
│   │   ├── *.rs                          # export, meta, cache_download, health, epub-convert, download,
│   │   │                                 # pow, recipes, tags, curator_*, authors, series, search,
│   │   │                                 # social, comments, reviews, bookmarks, kudos, ratings,
│   │   │                                 # follows, updates, forum, collections, lists, shelves,
│   │   │                                 # pseuds, skins, quests, progression, notifications, feed,
│   │   │                                 # requests, docs, badges, analytics, admin, auto_tag, bulk,
│   │   │                                 # modlog, heal, reports, roadmap, features, customization,
│   │   │                                 # locales, trending, fandom, reader, kindle, upload,
│   │   │                                 # user_export, user_credentials, opds/, rss/
│   ├── search/                           # SEARCH V2 PIPELINE
│   │   ├── parser.rs                     # boolean grammar + Field/FieldOp/RangeExpr, tokenizer
│   │   ├── builder.rs                    # dynamic SQL builder (tag resolution, counters, facets)
│   │   ├── routes.rs                     # GET /api/search + shared run_search (parser→builder)
│   │   ├── ask.rs                        # POST /api/search/ask (NL → v2 query string via Ollama)
│   │   ├── ask_cache.rs                  # Ask response cache (1h) + warm-keepalive
│   │   ├── body.rs                       # full-text body search (/api/search/body)
│   │   ├── suggest.rs                    # /api/search/suggest
│   │   ├── tags.rs                       # tag facet helpers
│   │   └── history_chips.rs              # /api/search/history/chips
│   ├── recommender/                      # multi-strategy rec engine (cooccur, embeddings, mf, bandit, hybrid)
│   ├── tags/                             # tag system + curator routes
│   ├── scrape/                           # SiteScraper trait + registry (uses scrapers/ crate)
│   ├── export/                           # EPUB, HTML bundle, TXT, MD, DOCX, FB2, KEPUB, PDF/MOBI/AZW3 (calibre)
│   ├── cache/{mod,disk}.rs               # disk cache + EType enum
│   ├── limiter/{mod,redis_bucket,geoip}.rs # Redis token bucket + MaxMind datacenter blocking
│   ├── heal/                             # self-healing OTF scraper (agent, classifier, extract, snapshot)
│   ├── services/                         # background services (email/kindle, scrapers)
│   ├── works/, progress/, progression/
│   ├── body_cache.rs, crypto.rs, modlog.rs, fic_suggestions.rs, roadmap_seed.rs, ingest/
│   └── bin/                              # HELPER BINS (one-shot CLI tools)
│       ├── assign_quests.rs              # → assign-quests
│       ├── compute_stats.rs              # → compute-stats
│       ├── compute_leaderboards.rs       # → compute-leaderboards
│       ├── bot_scorer.rs                 # → bot-scorer
│       ├── backfill_scores.rs            # → backfill-scores
│       ├── backfill_body_search.rs       # → backfill_body_search
│       ├── seed_roadmap.rs               # → seed-roadmap
│       ├── saved_search_watcher.rs       # → saved-search-watcher (nightly alert feed, ALERT_PER_PAGE=500)
│       └── migrate.rs                    # → migrate (explicit migration runner used by deploy.sh)
├── frontend/                             # SvelteKit SPA (in-repo)
│   ├── src/routes/                       # ~100 routes (admin/, ask, authors/, collections/,
│   │   │                                 # curator/, dashboard, download, fandom/, fic/, follows/,
│   │   │                                 # forum/, history, leaderboard, lists/, modlog,
│   │   │                                 # notifications, quests, reading, read/, recommendations/,
│   │   │                                 # requests/, roadmap, search/ (+body, +syntax, +tags),
│   │   │                                 # series/, settings/, shelves/, stats, tags, trending,
│   │   │                                 # updates, upload, work/, works/, work-proposals/, ... )
│   ├── build/                            # Generated build output (served by the binary; not committed)
│   ├── e2e/, static/, *.config.*ts
├── scrapers/                             # `fanfic-scrapers` crate (AGPL) — site-specific scrapers
├── forum_core/                           # `forum-core` crate (AGPL) — generic forum engine
├── fanfic-archivist/                     # Chat-bot (Discord/Telegram/Matrix) integration crate
├── rec-engines/                          # Python sidecar recipes for the extension platform
├── docs/                                 # mdbook documentation (/docs served by the binary)
│   ├── src/                              # markdown chapters
│   ├── book/                             # built HTML
│   ├── DEPLOYMENT.md                     # current deployment layout (authoritative)
│   └── brainstorm-*.md                   # design/implementation-plan notes
├── bin/, scripts/, docker/, tests/, qa/  # tooling, container (calibre), integration tests, QA harness
├── deploy.sh                             # build-on-gamingpc → sync /opt → migrate → restart → health check
├── justfile, qa.sh, docker-compose.yml, Dockerfile
└── SPECIFICATION.md                      # THIS FILE
```

No other backend codebases exist in the tree. `~/code/go/fichub-frontend/` and `~/code/go/fichub-cli.bak/` referenced by older revisions of this document are historical and no longer used.

---

## 3. Architecture & Data Flow

```
                        ┌──────────────┐
                        │  (Browser)   │
                        └──────┬───────┘
                               │ https://fichub.polarisocial.xyz
                               ▼
                     ┌─────────────────┐
                     │  Cloudflare  →  │ nginx :8000 (reverse proxy)
                     └────────┬────────┘
                              │ proxy to localhost
                              ▼
                     ┌──────────────────────────────┐
                     │  fichub.service (systemd)     │  User=alvaro, /opt/fichub
                     │  single axum binary :PORT     │  prod PORT=8000
                     │  ──────────────────────────── │
                     │  /api/*  /docs/*  /sw.js  SPA │  ServeDir frontend/build
                     └────────┬─────────┬────────────┘
                              ▼         ▼
                     ┌────────────┐ ┌──────────┐
                     │ PostgreSQL │ │ Redis    │   Ollama (OLLAMA_URL)
                     │ :5432      │ │ :6379    │   external for ask/embeddings
                     └────────────┘ └──────────┘
```

**Request flow — web app / API (`GET /` and `GET /api/...`):**

```
Browser → Cloudflare → nginx:8000 → fichub binary :8000
        → /api/*            → axum handler (auth, DB/Redis)
        → /docs/*           → mdbook static
        → /sw.js            → PWA service worker (from FRONTEND_DIR)
        → / (SPA route)     → ServeFile index.html → client-side SvelteKit routing
```

The binary owns the API, the docs, and the SPA all on one port — there is no separate threadlight or Go service. nginx is only a reverse proxy for the public domain.

**Search flow (shared pipeline) — `GET /api/search`, `POST /api/search/ask`, saved-search reruns:**

```
raw string ─→ parser.rs (BooleanExpression) ─→ apply_query_parse → SearchParams
            ─→ builder.rs build SQL (tag name resolution, counters, facets)
            ─→ run_search → envelope {total, page, per_page, results, facets}
        Ask: NL ─→ Ollama (llama3.1:8b) ─→ v2 query string ─→ SAME pipeline
                 ─ fallback on failure: raw NL as `q`, translated:false
```

---

## 4. Backend: Rust (single axum binary)

### Technology Stack

| Component | Crate | Version |
|-----------|-------|---------|
| HTTP framework | axum | 0.8 (`multipart`) |
| Async runtime | tokio | 1 (full) |
| Middleware | tower / tower-http | 0.5 / 0.7 (cors, gzip, trace, fs) |
| Database | sqlx | 0.9 (runtime-tokio, postgres, chrono, uuid, migrate, tls-rustls-ring, derive, macros) |
| Redis | redis | 1.4 (aio, tokio-comp) |
| GeoIP | maxminddb | 0.30 |
| HTTP client | reqwest | 0.12 (json, rustls-tls, cookies) |
| HTML parsing | scraper | 0.27 |
| Scraper library | fanfic-scrapers (local `scrapers/`) | AGPL-3.0 |
| Forum engine | forum-core (local `forum_core/`) | AGPL-3.0 |
| EPUB generation | epub-builder | 0.8 |
| Template engine | tera | 2 |
| Serialization | serde / serde_json | 1 |
| Config | dotenvy | 0.15 |
| Logging | tracing / tracing-subscriber | 0.1 / 0.3 |
| Metrics | axum-prometheus | 0.10 |
| UUID | uuid (v4) | 1 |
| Hashing | md-5, sha2 | 0.11 |
| Dates | chrono (serde) | 0.4 |
| Compression | zip | 8 |
| Auth | jsonwebtoken, bcrypt, axum-extra (typed-header) | 9 / 0.16 / 0.10 |
| Encryption (site creds) | aes-gcm, base64 | 0.10 / 0.22 |
| Email (Send-to-Kindle) | lettre | 0.11 |
| Documents | docx-rs, quick-xml | 0.4 / 0.41 |
| Error ergonomics | thiserror | 2 |
| Misc | sanitize-filename, hex, regex-lite, rand, ipnetwork, urlencoding | — |

### Entry Point (`main.rs`)

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();                       // load .env if present
    tracing_subscriber::fmt().with_env_filter(...).init(); // default "info,fichub=debug"
    let config = config::Config::from_env();
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    server::run(config).await;
}
```

The `fichub` binary is the sole web server (`default-run`). The library crate (`lib.rs` / `src/lib.rs`) re-exports modules so integration tests can link against the server.

### Server Setup (`server.rs`)

Builds an `AppState` with: `config: Config`, `db: sqlx::PgPool`, `redis`, `http_client: reqwest::Client`, scraper registry, cache semaphores, rate limiter, recommender, body cache, heal agent, email/kindle service, etc. The router is assembled in `build_router()` (~900 lines of `.route()` calls — the complete API surface is enumerated in §9). Static assets are served by `ServeDir::new(&frontend_dir)` with fallback to `index.html`, plus `/sw.js` from the same frontend dir.

### API Routes (Axum Router)

The full list of `.route()` paths (verbatim from `src/server.rs`, git-derived) is given in [§9 API Endpoint Reference](#9-api-endpoint-reference). Highlights:

- Export/metadata/cache: `GET /api/epub`, `GET /api/epub/convert`, `GET /api/meta`, `GET /api/remote`, `GET /api/health`, `GET /cache/{etype}/{url_id}[/{fname}]` (either download or trigger export), batch `GET /api/download/author|series`.
- Anti-bot: `GET/POST /api/pow/challenge|solve` (proof-of-work for shadowbanned clients), `GET /api/pow/...`.
- Recommender: `GET /api/recommendations`, `/personal`, `/embeddings`, `/strategies`, `POST /api/recommendations/train`, `/suggest`, `/vote`, `GET /votes`.
- Tags/curator: `/api/tags/*`, `/api/tags/submit|vote|flag|resolve|search|autocomplete`, `/api/curator/*` (alias, merge, flags, content fixes, metadata proposals, author merges, approvals, marginalia).
- **Search v2**: `GET /api/search`, `POST /api/search/ask`, `GET /api/search/body|suggest|history/chips|similar`, saved searches `GET/POST /api/search/saved...`.
- Auth & social: `/api/auth/{register,login,refresh,me}`, `/api/bookmarks*`, `/api/ratings`, `/api/kudos`, `/api/reviews`, `/api/comments`, `/api/works/{id}` + stats + reviews + similar + random, `/api/users/{id}` + badges + reading-stats + streak.
- Follows (incl. **exclusions**, migration 002): `/api/follows`, `/api/follows/{follow_id}/exclusions` (POST/GET/DELETE), `/api/follows/check/...`, `/api/follows/followers/{user_id}`, `/api/v1/follows/{id}/seen`, `/api/v1/updates`.
- Collections (**submissions**, migration 006): `/api/collections/*` incl. `/requests`, `/requests/{req_id}/approve|reject|vote`, challenges, bookmarks.
- Forum, requests/docs, recipes (extension platform), shelves/lists, pseuds, skins, quests/progression/customization (`/api/me/*`, `/api/features*`, `/api/roadmap*`), notifications, feed, badges/leaderboards, locales/translations, trending/fandoms, reader, upload, kindle, analytics, admin (`/api/admin/*`), modlog.
- Feeds/OPDS/docs: `/feed.xml`, `/feed/follows.xml`, `/feed/works/{url_id}`, `/feed/saved/{user_id}/{search_id}` (saved-search Atom feed, migration 007), `/opds/*` (root, new, popular, tags, authors, recommendations, search, shelves, manifest), `/docs/*`, `/sw.js`.
- Legacy redirects: `/legacy/epub_export`, `/changes`, `/popular/` → `/`.

### Search v2 (parser.rs → builder.rs)

The boolean query string grammar (single source of truth, used by search, Ask, saved searches):

```
Plain terms:              harry potter time travel          (words AND together)
Quoted phrase:            "slow burn"
Boolean:                  x AND y | x OR y | NOT | -exclude   (e.g. -angst, NOT with attr:Dark)
Role modifier (@):        @char:Harry  @fandom:Marvel  primary_fandom:X
Character:                char:Hermione (any) / char:Harry (via @) 
Relationship polarity:    romship:A/B  platship:A&B  (and bare ship:)
Fandom / fielded:         fandom:Harry Potter  title:harry  author:jk  description:...
Freeform / attribute:     attr:"Slow Burn"  attr:cozy*  attr:*burn
Warning / category:       warning:"Major Character Death"  category:Gen
Rating:                   rating:Explicit
Status:                   status:complete|ongoing|wip
Counters & ranges:        words:>50k  words:10k-100k  words:5000  kudos:>=100
                          chapters:5-50  chapters:=7  published:2018-2021  published:2020  updated:>2019
Site source:              source:archiveofourown.org  source:fanfiction.net
Language / beta:          language:en  beta:yes
Crossover:                crossover:1   (more than one distinct fandom tag)
Implicit AND w/ negation: char:Harry with attr:"Slow Burn" NOT with attr:Dark
```

Typed fields (enum `Field`): Title, Author, Description, Fandom, PrimaryFandom, Char, Ship, Romship, Platship, Attr, Warning, Category, Rating, Status, Words, Chapters, Kudos, Comments, Bookmarks, Hits, Published, Updated, Language, Beta, Source, Crossover. Values apply via `FieldOp`: Contains, Prefix (`val*`), ContainsWide (`*val`), Range (Eq/Ge/Gt/Le/Lt/Between, with `k` suffixes). Facets are built over fandoms, characters, relationships, freeforms, ratings, warnings.

**Response envelope (GET /api/search):**
```json
{ "total": 0, "page": 1, "per_page": 20, "results": [ ... ], "facets": { ... } }
```
`SearchQueryParams` also accepts personal filters keyed on the signed-in user (e.g. hide read / hide bookmarked / library-only); anonymous searches pass user_id 0 and the builder treats them as no-ops. Every search is logged (best-effort, non-blocking) to `search_queries` for analytics.

### Ask the Archive (`search/ask.rs`)

`POST /api/search/ask` — body `{ "q": "<natural language>" }`, `MAX_ASK_LEN = 500`, `ASK_PER_PAGE = 20`.

- **Prompt** (`build_ask_prompt`): instructs the model (Ollama, default `OLLAMA_CHAT_MODEL`, e.g. `llama3.1:8b`) to translate NL to **ONE bare FicHub v2 search-query string** — no JSON, no markdown/code fences. Rules: always start with topic keywords as plain words; only add a structured operator the request explicitly states; size words map to `words:>` (`long fic`→`words:>100k`, `over 50k`→`words:>50000`, etc.); completion maps only to explicit `status:complete`; source maps only from an explicit site name (`ao3`→`source:archiveofourown.org`, `ffn`→`source:fanfiction.net`); **never invent tag/type/character ids** (tag filters are resolved by name server-side).
- **Validation** (`validate_llm_params`): strips code fences/backticks, rejects multi-line output and junk (must parse to ≥1 usable term), caps length. Returns `None` on unusable output → plain fallback.
- **Translation** (`translate_with_ollama`): calls Ollama with a hard timeout (cold model load can take 10–30s), best-effort — returns `None` on any Ollama error.
- **The v2 query string is fed into the SAME pipeline as `GET /api/search`** (`translated_into_search_params` sets `q: Some(v2_query)`), so an ask matches exactly like typing the string.
- **Response** = the standard search envelope + ask fields:
```json
{ "total": ..., "results": [...], "page": ..., "per_page": ...,
  "translated": true|false,
  "nl_query": "<original NL>",
  "applied_params": "<the v2 query string the UI renders>" }
```
  `applied_params` is a **STRING** (the v2 query) — this replaced the old 5-key JSON contract `{q,min_words,max_words,complete,source}`.
- **Degradation contract**: Ollama down / times out / returns garbage → raw NL search as `q` with `translated:false`; the raw NL is returned as `applied_params` (plain fallback). A translation, once produced, is cached in Redis (1h TTL, hash key) and the model kept warm with `keep_alive`. Every ask is logged to search analytics (`[ask-translated]` / `[ask-plain]` markers).
- `translate_for_request` reuses the same translation for the Fic-Requests archivist answer (never fails, falls back to raw NL).

### Tag role / polarity support (migrations 003, 004)

- `tags.rel_polarity` (generated column): 1 = romantic (`A/B`), 2 = platonic (`A & B`), 0 = relationship tag with neither, NULL = not a relationship tag. Lets `romship:`/`platship:` filter by a plain index seek.
- `fic_tags.role_confidence` (REAL): 1.0 = main/primary (legacy `score >= 10`), 0.5 = secondary (`score > 0`), else 0.0. Lets `@char:` / `@fandom` / `primary_fandom:` express "main character / primary fandom".

### Search v2 counters (migration 005)

`works` gained denormalised columns so search can filter/sort on engagement without live JOINs: `kudos_count`, `comments_count`, `bookmarks_count`, `hit_count`, `language_code`, `beta_status`, `first_published`, `last_updated`.

### Scraper System

`SiteScraper` trait (`scrape/`) + registry; concrete scrapers live in the `scrapers/` crate (`fanfic-scrapers`):

```rust
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

Sites: archiveofourown.org, fanfiction.net, fictionpress.com, adultfanfiction.org, hpfanfic.com, generic XenForo, plus fanficfare for many more. `FicMetadata` carries `url_id` (deterministic SHA-256), title, author, chapters, words, description, published/updated (unix millis), status, source, source_id, author_id, content_hash, extra metadata. A self-healing on-the-fly agent scraper (`src/heal/`) recovers scrape failures using an LLM classifier + snapshot extraction (see `docs/book/self-healing.html`).

### Export System (`src/export/`)

Formats: **EPUB** (`epub.rs`, via epub-builder + MD5), **HTML bundle**, **TXT**, **Markdown**, **DOCX**, **FB2**, **KEPUB** (Kindle), and **PDF/MOBI/AZW3** via the Calibre sidecar (`convert.rs`, `ebook-convert`). Disk cache (`cache/disk.rs`) is a 2-level hash tree per `EType` (`Epub, Html, Mobi, Pdf, Txt, Azw3, Md, ...`); `CacheSemaphores` prevent duplicate concurrent exports. `export_log` / `fic_version_bump` drive cache invalidation; `EXPORT_VERSION` bumps the format.

### Rate Limiter & Anti-Bot

Redis token-bucket (`limiter/redis_bucket.rs`) with per-endpoint-class tiers (`RL_*` env vars), dynamic per-load scaling, client-id-aware keys, NAT multipliers, shadowban buckets, plus MaxMind datacenter-ASN blocking (`MAXMIND_DB`). Proof-of-work challenges (`/api/pow/*`) gate shadowbanned clients.

### Recommender System (`src/recommender/`)

Multi-strategy engine: legacy co-occurrence, embedding-based (`REC_EMBED_MODEL`, default `nomic-embed-text`), implicit matrix factorization (`rec-mf` feature), bandit/UCB, curator-align, hybrid. Strategies selected via `REC_STRATEGIES` (default `cooccur`); periodic training (`REC_TRAIN_EVERY_H`), caching/precompute, external Python sidecar recipes.

### Error Handling (`error.rs`)

`AppError` enum with `IntoResponse`. Common codes: `-1` internal/db, `-5` bad request/not found, `-6` scrape error, `-10` blocked (automated), `-429` rate limited.

---

## 5. Backend: Go v1 (retired, merged into Rust)

> **Status: RETIRED.** The Go v1 API backend (`~/code/go/fichub-cli.bak/`) and its Typesense/asynq stack were **abandoned**; every v1 feature (user accounts, fics CRUD, bookmarks, full-text search, uploads, moderation) was **implemented natively in the single Rust backend** with SQL/PostgreSQL search instead of Typesense. No Go code ships in the current system; `fichub-frontend` also moved into the Rust repo (`frontend/`).

Historical shape (for reference only, no longer present):

- **Router**: go-chi/chi/v5; **DB**: jackc/pgx/v5; **Redis**: go-redis/v9 (asynq queue); **Search**: Typesense; **Auth**: JWT; **Logging**: zerolog.
- Former v1 routes: `/api/v1/auth/*`, `/api/v1/users/*`, `/api/v1/fics*`, `/api/v1/search*`, `/api/v1/modlog*`, `/api/v1/upload`, etc. — the endpoints below now live under the Rust backend (many at `/api/...` without the `v1` prefix; see §9):
  - Auth: `POST /api/auth/register|login|refresh|me` (Rust route).
  - Users/profile/bookmarks: `/api/users/{id}`, `/api/bookmarks*` (Rust).
  - Fics list/detail/download/versions: `/api/works/{url_id}*`, `/api/download/...`, `/api/epub` (Rust).
  - Search: `/api/search` (Rust, boolean v2), `/api/search/suggest` (Rust).
- Former Go middleware (RequestID, RealIP, Logger, Recovery, Compress, CORS, RateLimiter, Auth, ModeratorOnly) — all replicated in Rust (tower-http middleware, axum layer stack).
- Former Tablesense collections / asynq jobs (IndexFic, ProcessUpload, GenerateThumbnail, CleanupExpiredSessions, RecalculateTags, ExportFic) — all handled by Rust tasks/helper bins now.

Nothing in the current tree references `fichub-cli.bak` or Typesense; migrate any expectations of a Typesense JSON API to the SQL-backed `/api/search`.

---

## 6. Frontend: SvelteKit SPA

The frontend lives **in the Rust repo** at `frontend/` and is served by the backend binary.

### Technology Stack

| Component | Library | Version |
|-----------|---------|---------|
| Framework | @sveltejs/kit | ^2.8.0 |
| Svelte | svelte | ^5.1.0 |
| Build tool | vite | ^5.4.0 |
| Adapter | @sveltejs/adapter-static | ^3.0.6 |
| Testing | vitest, @testing-library/svelte, playwright | — |
| Rendering | marked, dompurify | — |
| Linter / formatter | eslint / prettier (repo-level) | — |

### Build Configuration

- Adapter-statickt static with `fallback: 'index.html'`; the SPA is fully pre-rendered + client-side hydration, routed client-side from `index.html`.
- Dev server (`npm run dev`, Vite) — build output goes to `frontend/build`, which the backend serves from `FRONTEND_DIR`.
- **CI**: frontend coverage ≥ 85% lines / 78% funcs / 76% branches on `src/lib/**`; backend coverage via `cargo llvm-cov --lib` ≥ 30%.

### Route Structure (in-repo `frontend/src/routes/`)

A large per-feature route tree (SvelteKit file-based routing). Examples: `+page.svelte` (home), `search/` (+`body`, `+syntax`, `+tags`), `ask/`, `download/`, `dashboard/`, `fic/[urlId]/` (and `work/[workId]/` incl. `/stats`, `/translate`, and `works/[urlId]/`), `authors/`, `fandom/[slug]/`, `fandoms/`, `collections/[id]/`, `lists/[id]/`, `shelves/[id]/`, `series/[id]/`, `follows/`, `updates/`, `feed/`, `requests/`, `forum/` (board/topic/category/metamod/moderate/search), `bookmarks/` (+`search/+server.ts`), `history/`, `reading/`, `read/[urlId]/`, `quests/`, `leaderboard/`, `badges/`, `modlog/`, `notifications/`, `people/`, `stats/`, `tags/`, `trending/`, `work-proposals/`, `curator/` (flags/approvals/consensus/authors/marginalia), `admin/` (users, stats, analytics, moderation, blacklist, bots, translations, bulk-actions, metadata, content-scan, features, scrapers, comment-triage, auto-tag), `settings/` (+`recipes`, +`theme`), `docs/`, `blind-date/`, `upload/`, `roadmap/`, `[...slug]/+page.ts` (catch-all).

### API Client

The SvelteKit app talks to the same-origin Rust API (empty base URL = same origin, so no CORS/proxy in prod). Token/auth handled via localStorage. In dev (`vite.config.ts`) the `/api` proxy targets the running backend.

### Known Frontend Notes

- `package.json` name is `fichub-frontend` (in-repo version; older docs mention Photon branding remnants — those live in code comments only).
- Covered by an extensive Vitest + Playwright suite; `scripts/` helpers rebuild assets.

---

## 7. Database Schema

PostgreSQL. Migrations are applied by **sqlx automatically on service start** (`sqlx::migrate!()`); `deploy.sh` also runs the `migrate` helper bin explicitly. Migrations must be idempotent (`CREATE TABLE IF NOT EXISTS`, `ON CONFLICT DO NOTHING`). Do **not** hand-insert into `_sqlx_migrations` (sqlx validates checksums — a mismatched row causes `Migrate(VersionMismatch(N))` and can break startup).

Migration files: `001_initial.sql` (consolidated base), then **002–007** (new feature layers).

### Migration 001 — Consolidated Base (generated from live prod dump)

`001_initial.sql` is a single consolidated schema (~8.7k lines from a production dump). It defines the ~138 core tables. Notable landmarks:

- **Content/identity**: `works` (canonical work), `fic_info` (cached metadata keyed by SHA-256 `url_id`, FK `work_id`), `fic_works`, `series`, `series_works`, `authors` / `author_profiles` / `author_socials` / `author_merge_proposals`, `pseuds`, `creatorships`.
- **Tags**: `tag_types` (1 fandom, 2 character, 3 relationship, 4 freeform, 5 warning, 6 category, 7 rating), `tags`, `fic_tags` (+`score`), `fic_tag_votes`, `tag_aliases`, `tag_flags`, `tag_embeddings`, `tag_score_fixes`, `tags` polarity/confidence columns from 003/004.
- **Users & social**: `users` (role 0=reader…3=admin, reputation, curator flags), `bookmarks`, `work_ratings`, `work_rating_verifications`, `kudos`, `comments`, `comment_triage`, `reviews`, `reactions`, `follows`, `blocked_users`, `reputation_events`, `xp_events`, `xp_source_defs`, `user_badges`, `badge_definitions`, `user_invites`.
- **Search/suggestions**: `search_queries`, `fic_suggestions`-related, `auto_merge_log`, `fic_blacklist`, `author_blacklist`, `content_scan`.
- **Recommender**: `fic_bookmarks`, `fic_bookmark_cooccur`, `precomputed_recommendations`, `recommendation_suggestions`, `recommendation_votes`, `rec_*` (embeddings, bandit arms, impressions, models, training runs, user clusters, user curator align, user signals, author graph, transitions).
- **Export/cache**: `export_log`, `request_log`, `request_source`, `fic_version_bump`, `pending_exports`.
- **Forum/community**: `forum_*` (categories, topics, posts, follows, read_state, topic_views, post_reactions, edit_proposals, edits queue, metamod_votes, mod_actions, mod_grants, bans), `modlog`, `reports` (`user_reports`), `registration_applications`.
- **Requests/docs/roadmap**: `fic_requests`, `fic_request_answers`, `fic_request_answer_votes`, `fic_request_upvotes`, `doc_sections`, `arena_votes`, `features`, `feature_clusters`, `feature_suggestions`, `roadmap`-related (vote/consensus via `arena_votes` / `feature_suggestions`).
- **Collections/lists/shelves**: `collections`, `collection_item_requests`, `collection_bookmarks`, `challenges`, `challenge_signups`, `challenge_assignments`, `lists` (+items), `shelves`, `opds_shelves`, `opds_shelf_items`, `work_shelves`.
- **Progression/customization**: `user_levels`, `user_features`, `user_widgets`, `user_views`, `user_layouts`, `user_prefs`, `user_daily_progress`, `user_recipes`, `extensions`, `skins`, `user_skins`, `work_skins`, `reading_lists`, `reading_list_items`, `reading_stats`, `reading_history`, `login_streaks`, `daily_quests`, `locale`/`locales`, `translations`, `chapter_translations`, `chapter_translation_versions`, `user_site_credentials`.
- **Admin/analytics/heal**: `usage_events`, `admin_daily_stats`, `log_stats`, `bot_scores`, `scrape_failures`, `heal_extractions`, `agent_runs`, `content_scan` results, `search_mining` outputs(`feature_clusters`), `site_settings`, `curator_content_overrides`, `curator_fix_proposals`, `curator_fix_votes`, `curator_metadata_proposals`, `curator_metadata_votes`, `marginalia`.
- Postgres extensions: `pg_trgm` (fuzzy search), `vector` (embeddings).

(Not every column is re-derived here; the file itself is the source of truth for exact DDL.)

### Migration 002 — Follow Exclusions (`follow_exclusions`)

Per-follow filters that drop specific works, series, or fandoms out of the follow-updates feed. One row per `follows` FK (ON DELETE CASCADE); exactly one of `exclude_work_id` / `exclude_series_id` / `exclude_fandom` must be set (`target_check`), and `exclude_type` ∈ {`work`,`series`,`fandom`}. A work is hidden when any exclusion matches: work_id equality, membership in an excluded series, or any fandom tag equal to the excluded fandom. Index on `follow_id`.

### Migration 003 — Search v2 Relationship Polarity

`tags.rel_polarity SMALLINT GENERATED ALWAYS AS (...)` STORED: 1 = romantic (name contains `/`), 2 = platonic (`&`), 0 = relationship type-3 tag with neither literal, NULL = not a relationship tag. Partial index `idx_tags_rel_polarity ON tags(tag_type_id, rel_polarity) WHERE tag_type_id = 3` → `romship:`/`platship:` filter by index seek.

### Migration 004 — Search v2 Role Confidence

`fic_tags.role_confidence REAL NOT NULL DEFAULT 1.0`, backfilled from legacy `score`: `>=10 → 1.0` (main/primary), `>0 → 0.5` (supporting), else `0.0`. Enables `@char:`/`@fandom`/`primary_fandom` (main/primary role filters).

### Migration 005 — Search v2 Engagement Counters

`works` gains denormalised columns (all with defaults, backfilled): `kudos_count`, `comments_count`, `bookmarks_count`, `hit_count` (INT4), `language_code TEXT DEFAULT 'en'`, `beta_status TEXT DEFAULT 'unknown'`, `first_published TIMESTAMPTZ`, `last_updated TIMESTAMPTZ`. Search sorts/filters on engagement and maps `language:`/`beta:`/`published:`/`updated:` directly.

### Migration 006 — Collection Submissions (community curation)

`collection_submission_votes(id identity PK, request_id FK collection_item_requests ON DELETE CASCADE, user_id FK users ON DELETE CASCADE, vote SMALLINT CHECK vote IN (-1,0,1) DEFAULT 0, created_at, UNIQUE(request_id,user_id))`. Any logged-in user may signal agree (1) / abstain (0) / disagree (-1) on a pending "add work to collection" request. Votes are upserts; the tally shows in the request list and the unified curator Approvals queue (`/api/curator/approvals`). Approve/reject authority is unchanged.

### Migration 007 — Saved Searches + Daily Alerts

- `saved_searches(id BIGSERIAL PK, user_id FK users ON DELETE CASCADE, name, query_text, query_json jsonb, alert_mode CHECK IN ('none','rss') DEFAULT 'none', last_run_at, last_match_count, created_at, UNIQUE(user_id,name))`. Saves a search's name + raw query string + parsed JSON AST.
- `saved_search_matches(search_id FK saved_searches, work_id FK works, first_seen, notified_at, PRIMARY KEY(search_id, work_id))` — newly-seen works for alerting searches, recorded by the nightly watcher and surfaced in the public per-search Atom feed `/feed/saved/{user_id}/{search_id}`.
- Indexes: `idx_saved_searches_user`, partial `idx_saved_searches_alert_mode WHERE alert_mode <> 'none'`.

### Table inventory (001, all ~138 tables)

`admin_daily_stats agent_runs arena_votes ask_translation_cache author_blacklist author_merge_proposals author_profile_links author_profiles author_socials auto_merge_log badge_definitions blocked_users bookmarks bot_scores challenge_assignments challenges challenge_signups chapter_translations chapter_translation_versions collection_bookmarks collection_item_requests comments comment_triage content_scan creatorships curator_content_overrides curator_fix_proposals curator_fix_votes curator_metadata_proposals curator_metadata_votes daily_quests doc_sections exp_events export_log extensions feature_clusters features feature_suggestions fic_blacklist fic_bookmark_cooccur fic_bookmarks fic_info fic_request_answers fic_request_answer_votes fic_requests fic_request_upvotes fic_tags fic_tag_votes fic_version_bump fic_works follows forum_bans forum_categories forum_edit_proposals forum_follows forum_metamod_votes forum_mod_actions forum_mod_grants forum_post_reactions forum_posts forum_read_state forum_topics forum_topic_views heal_extractions kudos leaderboard_monthly leaderboard_weekly locales login_streaks marginalia modlog notification_preferences notifications opds_shelf_items opds_shelves pending_exports precomputed_recommendations pseuds reactions reading_history reading_list_items reading_lists reading_stats rec_author_graph rec_bandit_arms rec_embeddings rec_impressions rec_models recommendation_suggestions recommendation_votes rec_training_runs rec_transitions rec_user_clusters rec_user_curator_align rec_user_signals registration_applications reputation_events request_log request_source reviews schema_migrations scrape_failures search_queries series series_works shelves site_settings skins tag_aliases tag_embeddings tag_flags tags tag_score_fixes tag_types tags translations usage_events user_badges user_daily_progress user_features user_invites user_layouts user_prefs user_recipes user_reports users user_site_credentials user_skins user_views work_proposals work_proposal_votes work_ratings work_rating_verifications works work_shelves work_skins works work_translations xp_events xp_source_defs`

---

## 8. Deployment

> Authoritative source: `docs/DEPLOYMENT.md` (2026-08-14); README "Deployment".

### Host & Layout (ThinkCentre — NOT Orange Pi)

- **Host**: ThinkCentre dev box, `User=alvaro`. Repo is NFS-shared with gamingpc (fast compile host).
- **The service runs from LOCAL disk**, not the NFS repo — since 2026-08-14, to stop **SIGBUS** crashes when the mergerfs/NFS mount wedges.

| What | Where |
|------|-------|
| Binary | `/opt/fichub/fichub` (local NVMe ext4) |
| `.env` | `/opt/fichub/.env` (`FRONTEND_DIR` absolutized) |
| Frontend build (static) | `/personal/documents/code/rust/fichub/frontend/build` (NFS) |
| Caches / tmp / bodies | `/public/literature/...` (NFS) |
| Migrations | `/personal/documents/code/rust/fichub/migrations` (NFS, applied at deploy + auto on start) |
| Repo (source) | `/personal/documents/code/rust/fichub` (NFS, shared with gamingpc) |

- **systemd unit** `fichub.service`: `WorkingDirectory=/opt/fichub`, `EnvironmentFile=/opt/fichub/.env`, `User=alvaro`, runs the release binary on port **`PORT`** (in `/opt/fichub/.env`; prod **8000**). One port serves `/api/*`, `/docs/*`, `/sw.js`, and the static SPA (`FRONTEND_DIR`).
- **Public domain**: `fichub.polarisocial.xyz`, fronted by **Cloudflare** → nginx on :8000 → the fichub binary. nginx is a reverse proxy only.
- **Env var note**: `/opt/fichub/.env` must have an **absolute** `FRONTEND_DIR=`; the repo `.env` uses relative `./frontend/build` — never copy the repo `.env` over `/opt` verbatim.

### Deploy (`./deploy.sh`)

```bash
./deploy.sh                # build on gamingpc + sync binary to /opt + migrate + restart + health check
./deploy.sh --skip-build   # only sync + migrate + restart + health check
```

Flow: build `target/release/fichub` + `migrate` on gamingpc over the shared NFS tree → verify binary fresh → `systemctl stop fichub` → copy binary to `/opt/fichub/fichub` → start → run `./target/release/migrate` (explicit migrations) → `systemctl restart fichub` → health check `localhost:8000/` and `/sw.js` → run QA (`./qa.sh`). **Always** use `deploy.sh`, never a bare `systemctl restart fichub` (the NFS binary is not what the service runs).

Migrations are also applied **automatically on service start** (`sqlx::migrate!()`).

### Self-Heal (NFS-wedge watchdog)

ThinkCentre runs `fichub-selfheal.service/.timer` and `streakforge-selfheal.service/.timer` every 60s. On probe failure they recover the `/personal` mergerfs pool (`umount -l /personal` → restart `mergerfs-personal.service` → restart `nfs-server` → `exportfs -r`) and restart the service. Logs: `/var/log/{fichub,streakforge}-selfheal.log`.

### Saved-Search Watcher

`src/bin/saved_search_watcher.rs` runs nightly (systemd timer) as the `saved-search-watcher` bin, re-runs each `alert_mode='rss'` saved search, diffs against already-seen works, and inserts newly-seen rows into `saved_search_matches` (`ALERT_PER_PAGE = 500`). New matches surface in the public Atom feed `/feed/saved/{user_id}/{search_id}`.

### Frontend rebuild

```bash
cd frontend && npm run build    # writes frontend/build (served by the binary)
```

### Common operations

```bash
# Health / logs
curl http://localhost:8000/api/health
ssh thinkcentre "journalctl -u fichub -n 50 --no-pager"

# QA from dev machine (ThinkCentre binds localhost; tunnel)
ssh -L 18000:localhost:8000 -N -f thinkcentre
QA_BASE=http://localhost:18000 node qa/run.js

# New migration (must be idempotent)
# write migrations/NNN_name.sql, deploy — sqlx applies on start
```

---

## 9. API Endpoint Reference

All endpoints below are `.route()` entries verbatim from `src/server.rs` (axum 0.8 `{param}` syntax). Grouped by feature. (Empty HTTP-method cells mean both GET and the listed verb are registered on the same path.)

### Export, Metadata, Health, Cache, Download

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/` | API docs index |
| GET | `/api/epub?q=<url>` | Export EPUB for a fic URL (main endpoint) |
| GET | `/api/epub/convert` | Convert an export to another format |
| GET | `/api/meta?q=<url>` | Get fic metadata |
| GET | `/api/remote` | Remote client info (IP, port, is_automated) |
| GET | `/api/health` | Health check (DB + Redis ping) |
| GET | `/cache/{etype}/{url_id}/{fname}` | Download cached file (hash) |
| GET | `/cache/{etype}/{url_id}` | Download or trigger export |
| GET | `/api/download/author` | Batch download an author's works |
| GET | `/api/download/series` | Batch download a series |

### Anti-bot (proof-of-work)

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/pow/challenge` | Issue a PoW challenge (shadowbanned clients) |
| POST | `/api/pow/solve` | Submit PoW solution |

### Recommender

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/recommendations` | Get recommendations (query params) |
| GET | `/api/recommendations/personal` | Personal recommendations for the user |
| GET | `/api/recommendations/embeddings` | Embedding-based recommendations |
| GET | `/api/recommendations/strategies` | List active strategies |
| POST | `/api/recommendations/suggest` | Submit a recommendation suggestion |
| POST | `/api/recommendations/vote` | Vote on a suggestion |
| GET | `/api/recommendations/votes` | Get votes |
| POST | `/api/recommendations/train` | Trigger rec-models training (admin) |

### Recipes (extension platform)

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET/POST | `/api/recipes` | List / create recipe blends |
| GET | `/api/recipes/active` | Get authenticated user's active recipe |
| GET | `/api/recipes/gallery` | Browse public recipe gallery |
| PUT/DELETE | `/api/recipes/{id}` | Update / delete recipe |
| POST | `/api/recipes/{id}/activate` | Activate a recipe |
| POST | `/api/recipes/{id}/install` | Install a recipe |
| POST | `/api/recipes/{id}/publish` | Publish a recipe |

### Fic Suggestions, Tags & Curator

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/fic-suggestions` | List fic suggestions |
| GET/POST | `/api/tags/submit` | Submit a tag for a fic |
| POST | `/api/tags/vote` | Vote on a tag |
| POST | `/api/tags/flag` | Flag a tag |
| GET | `/api/tags` | List/search tags |
| GET | `/api/tags/resolve` | Resolve a tag reference |
| GET | `/api/tags/search` | Search tags |
| GET | `/api/tags/autocomplete` | Tag autocomplete |
| GET | `/api/tags/{id}` | Tag detail |
| POST | `/api/curator/alias` | Create tag alias (curator) |
| GET | `/api/curator/aliases` | List aliases |
| DELETE | `/api/curator/aliases/{alias_name}` | Delete alias |
| POST | `/api/curator/merge` | Merge tags (curator) |
| PUT/DELETE | `/api/curator/tags/{id}` | Update / delete tag (curator) |
| GET | `/api/curator/flags` | List unresolved flags |
| POST | `/api/curator/flags/{id}/resolve` | Resolve a flag |
| POST | `/api/curator/content/{url_id}/propose` | Propose a body fix |
| GET | `/api/curator/content/{url_id}` | Get raw body for review |
| DELETE | `/api/curator/content/{url_id}` | Delete a bad body |
| GET | `/api/curator/content/proposals` | List body-fix proposals |
| POST | `/api/curator/content/proposals/{id}/vote` | Vote on a body fix |
| POST | `/api/curator/metadata/propose` | Propose a metadata fix |
| GET | `/api/curator/metadata/proposals` | List metadata proposals |
| POST | `/api/curator/metadata/proposals/{id}/vote` | Vote on a metadata proposal |
| GET | `/api/curator/consensus` | Curator consensus feed |
| GET | `/api/curator/marginalia` | Marginalia queue |
| GET | `/api/curator/approvals` | Unified curator Approvals queue (incl. collection submissions) |

### Authors & Series

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/series/{id}` | Get a series |
| GET | `/api/authors/search` | Search authors |
| GET | `/api/authors/by-name/{name}` | Get author by name |
| GET/PUT | `/api/authors/{id}` | Get / update author profile |
| POST | `/api/authors/{id}/socials` | Add author social link |
| DELETE | `/api/authors/{id}/socials/{social_id}` | Remove author social link |
| POST | `/api/curator/authors/merge` | Propose author merge |
| GET | `/api/curator/authors/pending` | Pending author merges |
| POST | `/api/curator/authors/approve/{proposal_id}` | Approve author merge |
| POST | `/api/curator/authors/reject/{proposal_id}` | Reject author merge |

### Search v2 + Ask the Archive

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/search?q=<v2 query>` | Boolean search (parser→builder), returns `{total,page,per_page,results,facets}` |
| POST | `/api/search/ask` | Natural-language search (Ollama → v2 query string → same pipeline); `translated`, `nl_query`, `applied_params` (string) |
| GET | `/api/search/body` | Full-text body search |
| GET | `/api/search/suggest` | Search suggestions/autocomplete |
| GET | `/api/search/history/chips` | Recent-search chips |
| POST | `/api/search/saved` | Save a search |
| GET | `/api/search/saved` | List saved searches |
| DELETE | `/api/search/saved/{id}` | Delete a saved search |
| PUT | `/api/search/saved/{id}/alert` | Enable/disable nightly alert (`alert_mode`) |
| POST | `/api/search/saved/{id}/run` | Rerun a saved search on demand |
| GET | `/api/search/similar/{work_id}` | Similar works by embedding vector |
| GET | `/api/works/{id}/similar` | Similar works by bookmark co-occurrence |
| GET | `/api/works/random` | Random work |

### Auth, Users, Social, Works, Comments

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| POST | `/api/auth/register` | Register user (JWT) |
| POST | `/api/auth/login` | Login (JWT) |
| POST | `/api/auth/refresh` | Refresh token |
| GET | `/api/auth/me` | Get current user from token |
| GET | `/api/users/me/level` | Current user's forum level |
| POST | `/api/bookmarks` | Add bookmark (work_id) |
| GET | `/api/bookmarks` | List user's bookmarks |
| DELETE | `/api/bookmarks/{work_id}` | Remove bookmark |
| GET | `/api/bookmarks/export` (and `.csv`) | Export bookmarks CSV |
| POST | `/api/bookmarks/import` | Import bookmarks CSV |
| POST | `/api/ratings` | Rate a work |
| GET | `/api/ratings/{work_id}` | Get aggregate ratings |
| POST | `/api/kudos/{work_id}` | Give kudos |
| GET | `/api/kudos/{work_id}` | Get kudos count |
| DELETE | `/api/kudos/{work_id}` | Remove kudos |
| POST | `/api/reviews` | Upsert a review |
| DELETE | `/api/reviews/{id}` | Delete a review |
| GET | `/api/works/{id}/reviews` | List reviews for a work |
| POST | `/api/comments` | Post comment (work_id) |
| GET | `/api/comments/{work_id}` | List comments for a work |
| GET | `/api/works/{id}` | Get a work |
| GET | `/api/works/{id}/stats` | Work stats |
| GET | `/api/my-works` | List current user's works |
| POST | `/api/reactions/{type}/{id}/react`, GET `/api/reactions/{type}/{id}` | Reactions on arbitrary targets |
| GET | `/api/leaderboard/curators` | Top curators by reputation |
| GET | `/api/users/{id}` | User profile |
| POST | `/api/user/export` | Export user data |
| GET/PUT | `/api/user/site-credentials` | Managed site credentials |
| DELETE | `/api/user/site-credentials/{domain}` | Remove site credential |
| GET | `/api/v1/works/{url_id}/also-bookmarked` | Works this URL's bookmarks also bookmark |

### Comments (threaded, `comments.rs`)

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET/POST | `/api/works/{url_id}/comments` | Get / post threaded comments |
| DELETE | `/api/comment/{id}` | Soft-delete comment (owner/curator) |
| PATCH | `/api/comment/{id}/hide` | Hide comment (curator) |
| PATCH | `/api/comment/{id}` | Edit comment |

### Follows & Updates (incl. exclusions — migration 002)

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| POST | `/api/follows` | Follow a target |
| GET | `/api/follows` | List current user's follows |
| DELETE | `/api/follows/{id}` | Unfollow |
| POST | `/api/follows/{follow_id}/exclusions` | Add a follow exclusion (work/series/fandom) |
| GET | `/api/follows/{follow_id}/exclusions` | List exclusions for a follow |
| DELETE | `/api/follows/{follow_id}/exclusions/{exclusion_id}` | Remove an exclusion |
| GET | `/api/follows/check/{target_type}/{target_id}` | Check follow status |
| GET | `/api/follows/followers/{user_id}` | List a user's followers |
| GET | `/api/v1/follows/{id}/seen` | Mark follow seen |
| GET | `/api/v1/updates` | Update feed for follows |
| POST | `/api/v1/works/{url_id}/refresh` | Refresh a fic from source |

### Notifications, Feed, Badges, Leaderboards

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/notifications` | List notifications |
| GET | `/api/notifications/unread-count` | Unread count |
| POST | `/api/notifications/read-all` | Mark all read |
| POST | `/api/notifications/{id}/read` | Mark one read |
| GET/PUT | `/api/notifications/preferences` | Notification preferences |
| GET | `/api/feed` | Personalized activity feed |
| GET | `/api/badges` | List badge definitions |
| GET | `/api/users/{id}/badges` | User's badges |
| GET | `/api/leaderboard/curators/weekly` | Weekly curator leaderboard |
| GET | `/api/leaderboard/curators/monthly` | Monthly curator leaderboard |

### Shelves, Lists, Collections (incl. submissions — migration 006)

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| POST | `/api/shelves`, GET `/api/shelves` | Create / list reading shelves |
| DELETE | `/api/shelves/{id}` | Delete shelf |
| POST | `/api/shelves/add` | Add a work to a shelf |
| DELETE | `/api/shelves/{shelf_id}/works/{work_id}` | Remove work from shelf |
| GET | `/api/shelves/{shelf_id}/works` | List works in shelf |
| POST | `/api/lists`, GET `/api/lists` | Create / list curated lists |
| GET/PATCH/DELETE | `/api/lists/{id}` | Get / update / delete a list |
| POST | `/api/lists/{id}/items` | Add item to list |
| DELETE | `/api/lists/{id}/items/{work_id}` | Remove item from list |
| GET/POST | `/api/collections` | Browse / create collections |
| GET/PATCH/DELETE | `/api/collections/{id}` | Get / update / delete collection |
| GET | `/api/collections/by-slug/{slug}` | Get collection by slug |
| POST | `/api/collections/{id}/items`, DELETE `/api/collections/{id}/items/{work_id}` | Add / remove collection item |
| POST | `/api/collections/{id}/bookmark`, DELETE … | Bookmark / unbookmark collection |
| GET | `/api/collections/{id}/bookmarkers` | List collection bookmarkers |
| GET | `/api/collections/{id}/requests` | List "add to collection" requests |
| POST | `/api/collections/{id}/requests/{req_id}/approve` | Approve a request (owner/curator) |
| POST | `/api/collections/{id}/requests/{req_id}/reject` | Reject a request |
| POST | `/api/collections/{id}/requests/{req_id}/vote` | Vote on a submission (migration 006) |
| GET | `/api/collections/view/{id}` | View collection |
| GET | `/api/collections/{id}/challenge` | Challenge info |
| POST | `/api/collections/{id}/challenge/signup` | Sign up for a challenge |
| POST | `/api/collections/{id}/challenge/assign` | Assign challenge prompts |
| POST | `/api/collections/{cid}/challenge/{aid}/claim` | Claim a challenge assignment |

### Pseuds (creatorships)

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET/POST | `/api/pseuds` | List / create pseuds |
| PATCH/DELETE | `/api/pseuds/{id}` | Update / delete pseud |
| GET | `/api/pseuds/creatorships` | List creatorships |
| POST | `/api/pseuds/invite` | Invite a creatorship |
| POST | `/api/pseuds/creatorships/{id}/approve` | Approve creatorship |
| POST | `/api/pseuds/creatorships/{id}/reject` | Reject creatorship |

### Skins, Reading, Quests, Progression, Customization

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET/POST | `/api/skins` | List / create skins |
| GET/PATCH/DELETE | `/api/skins/{id}` | Get / update / delete skin |
| POST | `/api/skins/{id}/apply` | Apply skin to user |
| GET | `/api/me/skin` | Current user's skin |
| POST | `/api/works/{work_id}/skins/{skin_id}` | Assign skin to a work |
| POST | `/api/reading/status` | Update reading status |
| GET | `/api/reading/list` | Reading list |
| POST | `/api/reading/record` | Record a reading event |
| GET | `/api/reading/history`, POST … | Get / append reading history |
| DELETE | `/api/reading/history/{id}` | Delete history entry |
| POST | `/api/reading/history/clear` | Clear reading history |
| GET | `/api/quests` | Get quests |
| GET | `/api/users/{id}/reading-stats` | User reading stats |
| GET | `/api/users/{id}/streak` | User login streak |
| GET | `/api/me/progression` | Progression / level / XP |
| GET/PUT | `/api/me/prefs` | User preferences |
| GET/PUT | `/api/me/layout/{page}` | Per-page layout prefs |
| GET/POST | `/api/me/views` | Custom views |
| DELETE | `/api/me/views/{id}` | Delete view |
| PUT | `/api/me/views/{id}/toggle-pin` | Pin/unpin view |
| GET/PUT | `/api/me/theme` | Theme |
| GET | `/api/nav` | Nav config |
| GET | `/api/widgets` | Dashboard widgets |
| GET/POST | `/api/features`, `/api/features/available` | Feature flags |
| POST | `/api/features/{slug}/enable` / `/disable` | Toggle feature gates |

### Forum, Requests, Docs

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/forum/categories` | List forum categories |
| GET | `/api/forum/topics` | List topics |
| GET/POST | `/api/forum/search` | Search forum |
| GET | `/api/forum/edits/{target_type}/{target_id}` | Forum edit history |
| GET | `/api/forum/edits/queue`, POST `/api/forum/edits/{proposalId}/review` | Edit review queue |
| (plus topic/post subroutes in `forum.rs`) | | Topics, posts, reactions, metamod, moderation within `/api/forum/*` |
| POST | `/api/requests`, GET `/api/requests` | Create / list fic requests |
| GET/DELETE | `/api/requests/{id}` | Get / delete request |
| POST | `/api/requests/{id}/answers` | Add request answer |
| DELETE | `/api/requests/{id}/answers/{aid}` | Delete answer |
| POST | `/api/requests/{id}/answers/{aid}/vote` | Vote on answer |
| POST | `/api/requests/{id}/upvote` | Upvote request |
| POST | `/api/requests/{id}/accept/{aid}` | Accept an answer |
| GET | `/api/requests/{id}/candidates` | Candidate works for a request |
| GET | `/api/docs/ask` | Docs/FAQ |
| POST | `/api/admin/docs/ingest` | Ingest docs (admin) |

### Site subsystem, Blocks, Registrations

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/site` | Site info |
| POST | `/api/admin/invites`, GET `/api/admin/invites` | Create / list invites |
| POST | `/api/registration-applications` | Apply for registration |
| GET | `/api/admin/registration-applications` | List applications |
| POST | `/api/blocks`, GET `/api/blocks` | Block / list blocked users |
| DELETE | `/api/blocks/{user_id}` | Unblock |

### Reading, Locales, Trends, Fandoms, Work Proposals

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/reader/{url_id}` | Reader view data |
| GET | `/api/locales` | List locales |
| GET | `/api/translations/{locale_code}` | Get UI translations |
| POST | `/api/works/{id}/translations` | Upsert work translation |
| GET | `/api/works/{id}/translations/{locale_code}` | Get work translation |
| GET | `/api/works/{id}/chapter-translations/{locale}`, PUT … | Get / upsert chapter translations |
| GET | `/api/trending`, `/api/trending/tags`, `/api/trending/tag/{type}/{name}` | Trending feeds |
| GET | `/api/fandoms`, `/api/fandoms/{slug}` | Fandom browse |
| POST | `/api/work-proposals`, GET `/api/work-proposals` | Create / list merge-split proposals |
| GET | `/api/work-proposals/{id}`, POST `/api/work-proposals/{id}/vote` | Proposal detail / vote |

### Upload, Kindle, Analytics, Modlog, Send-to-Kindle

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| POST | `/api/upload` | Manual fic upload |
| POST | `/api/upload/{url_id}/update` | Update an uploaded fic |
| DELETE | `/api/upload/{url_id}` | Delete an uploaded fic |
| POST | `/api/send-to-kindle` | Email a fic to Kindle |
| GET | `/api/analytics` | Usage analytics |
| GET | `/api/analytics/user/{client_id}` | Per-client analytics |
| GET | `/api/reading/analytics` | Personal reading analytics |
| GET | `/api/authors/{id}/analytics` | Author analytics |
| GET | `/api/admin/analytics` | Admin analytics |
| GET | `/api/admin/endpoint-usage` | Endpoint usage |
| GET | `/api/modlog` | Moderation log (transparent, any logged-in user) |

### Admin

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/api/admin/*` | Admin dashboards (users, stats, bots, moderation queue, content scan, search mining, dedupe/embeddings, comments triage, blacklist fic/author, approve/reject upload, scraper health, realtime, search analytics, roadmap-consensus, translations, rating checks, characters score, features, reports) |
| POST | `/api/admin/heal` | Trigger self-heal agent |
| POST | `/api/admin/auto-tag` / `backfill` / `queue` / `approve/{url_id}/{tag_id}` / `dismiss/{url_id}/{tag_id}` | Auto-tag pipeline |
| POST | `/api/admin/bulk/refresh` / `bulk/auto-tag`, GET `/api/admin/fics/search` | Bulk admin operations |

### Reports & Roadmap

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| POST | `/api/reports` | Create a report |
| GET | `/api/admin/reports` | List reports |
| POST | `/api/roadmap/suggest` | Suggest a roadmap feature |
| GET | `/api/roadmap/arena` | Request arena |
| POST | `/api/roadmap/vote` | Vote on a suggestion |
| GET | `/api/roadmap/consensus` | Roadmap consensus |

### OPDS Catalog (served by the binary)

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/opds` | Root navigation catalog |
| GET | `/opds/new`, `/opds/popular` | Recent / popular feeds |
| GET | `/opds/tags`, `/opds/tags/{type_id}`, `/opds/tags/{type_id}/{tag_name}` | Tag navigation + fics-by-tag |
| GET | `/opds/authors` | Author list |
| GET | `/opds/recommendations/popular`, `/opds/recommendations` | Recommendation feeds |
| GET | `/opds/search` | Search feed |
| GET | `/opds/shelves`, `/opds/shelf/{shelf_id}` | Reading shelves |
| GET | `/opds/manifest` | OPDS acquisition manifest |

### RSS / Atom Feeds & Static

| Method(s) | Endpoint | Purpose |
|-----------|----------|---------|
| GET | `/feed.xml` | New arrivals feed |
| GET | `/feed/follows.xml` | Follow updates feed (honours exclusions) |
| GET | `/feed/works/{url_id}` | Per-work feed (trailing `.xml` handled) |
| GET | `/feed/saved/{user_id}/{search_id}` | Saved-search alert Atom feed (migration 007) |
| GET | `/sw.js` | PWA service worker (from `frontend_dir`) |
| GET | `/docs/*`, `/` (fallback) | mdbook docs + SPA shell |

### Legacy redirects

`GET /legacy/epub_export`, `/changes`, `/popular/` → redirect to `/`.

### Response Format

- **Success**: direct JSON (no wrapping envelope). Search returns `{total, page, per_page, results, facets}`; Ask adds `translated`, `nl_query`, `applied_params`.
- **Error**:
```json
{"err": -1, "msg": "description"}
```
- Common codes: `-1` internal/db, `-5` bad request/not found, `-6` scrape error (bad gateway), `-10` blocked (automated request), `-429` rate limited.

---

## 10. Data Flow Examples

### EPUB Export Flow

```
1. User visits: GET /api/epub?q=https://archiveofourown.org/works/123456
2. Browser → Cloudflare → nginx:8000 → fichub binary:PORT → /api/epub
3. Handler:
   a. Validates q (non-empty, not automated), rate-limits the IP/client (Redis bucket)
   b. Registry finds a SiteScraper via can_handle()
   c. Scraper fetches HTML from AO3, parses title/author/chapters/words/description/stats
   d. Upserts fic_info (and possibly merged works) in PostgreSQL; checks fic_blacklist / author_blacklist
   e. Fetches all chapters via scraper.fetch_chapters()
   f. Generates the requested format via export/ (epub-builder; calibre for pdf/mobi/azw3)
   g. Stores to disk cache (2-level hash tree); bumps export_log
   h. Logs to request_log; returns a redirect to the cached file or JSON with the download URL
4. Frontend triggers the download.
```

### Search v2 Flow (`GET /api/search`)

```
1. User types a query (e.g. "drarry words:>100k status:complete source:archiveofourown.org")
2. GET /api/search?q=<string> → run_search
3. apply_query_parse → parser.rs builds a BooleanExpression (fields, roles, ranges, negation)
4. builder.rs resolves tag names/ids and counters → parameterised SQL against Postgres (plus facets)
5. Returns {total, page, per_page, results, facets}; the query is logged to search_queries
```

### Ask-the-Archive Flow (`POST /api/search/ask`)

```
1. User writes NL, e.g. "dark harry potter completed over 50k" (body {"q":"..."})
2. Handler checks the 1h Redis cache (hash of nl) — hit → return cached envelope
3. build_ask_prompt → Ollama.generate_json (model warm via keep_alive, hard timeout)
4. validate_llm_params strips fences / rejects multi-line / caps length; must parse to ≥1 term
   → "harry potter @char:harry status:complete words:>50k"
5. translated_into_search_params sets q = that v2 string → run_search (SAME parser→builder as /api/search)
6. Response = search envelope + translated:true, nl_query, applied_params (the v2 query string)
   If Ollama down / times out / returns garbage → plain search with raw NL as q, translated:false,
   applied_params = raw NL. Ask analytics logged ([ask-translated] / [ask-plain]).
```

### Saved-Search Alert Flow (nightly)

```
1. User saves a search (name + query text + parsed JSON AST), optionally alert_mode='rss'
2. Each night saved-search-watcher bin re-runs each alerting search (ALERT_PER_PAGE=500)
3. Diffs current match set vs saved_search_matches; inserts newly-seen (search_id, work_id, first_seen)
4. Public Atom feed /feed/saved/{user_id}/{search_id} surfaces new matches
```

### Follow Exclusion Flow

```
1. User follows a tag/author/series; adds an exclusion POST /api/follows/{follow_id}/exclusions
   with exclude_type ∈ {work, series, fandom} and exactly one target column
2. Follow updates feed (/api/v1/updates, /feed/follows.xml) drops any work whose
   work_id, series membership, or fandom tag matches an active exclusion
```

---

## 11. Current Issues & Technical Debt

### Deployment & Ops

| Issue | Impact | Location |
|-------|--------|----------|
| Binary must run from local `/opt` (not NFS) | SIGBUS on wedged mounts would kill the process | `docs/DEPLOYMENT.md` — mitigated 2026-08-14 |
| NFS/mergerfs watchdogs required | Service depends on `/personal` pool recovering | ThinkCentre self-heal timers |
| `_sqlx_migrations` checksum sensitivity | Hand-edited rows cause `Migrate(VersionMismatch(N))` startup failures | `README`, migrations |
| Env file divergence (repo vs `/opt`) | Copying repo `.env` over `/opt` breaks `FRONTEND_DIR` | `docs/DEPLOYMENT.md` |

### Backend

| Issue | Impact | Location |
|-------|--------|----------|
| Large `001_initial.sql` consolidated dump (~8.7k lines) | Hard to read/diff; `no-transaction` header | `migrations/001_initial.sql` |
| `role_confidence` / `rel_polarity` backfills are one-off `UPDATE`s | Rerun cost on re-apply; idempotency required | `migrations/003,004` |
| Ollama dependency for Ask / embeddings | Behavioral degradation to plain search when Ollama is down (by design) | `src/search/ask.rs` |
| Large monolith router file | ~900-line `build_router()` with hundreds of routes | `src/server.rs` |

### Frontend

| Issue | Impact | Location |
|-------|--------|----------|
| Photon-origin comments/branding in some components | Cosmetic inconsistency | `frontend/src/` |
| Coverage thresholds still being raised toward 95/90 | — | CI config |

Note: earlier revisions of this doc listed "Go backend not deployed" and "nginx → threadlight" as issues — both are resolved: the Go backend was merged into Rust and threadlight is gone.

---

## 12. Environment Variables

### Server (`src/config.rs`, dotenvy; production values live in `/opt/fichub/.env`)

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `REDIS_URL` | Yes | — | Redis connection string |
| `CACHE_DIR` | No | `./cache` | Primary cache directory |
| `SECONDARY_CACHE_DIR` | No | — | Secondary (warm) cache |
| `BODY_CACHE_DIR` | No | `/public/literature/fichub/bodies` | Full-text body cache |
| `EXPORT_VERSION` | No | `1` | Export format version (cache invalidation) |
| `DYNAMIC_RATE_LIMIT` | No | `true` | Enable dynamic rate limiting |
| `NODE_NAME` | No | `orion` | Node identifier for logs |
| `CALIBRE_CONTAINER` | No | — | Calibre Docker container name |
| `PORT` | No | `3000` | HTTP listen port (prod uses 8000) |
| `FRONTEND_DIR` | No | `./frontend/build` | Path to SvelteKit build (absolute in `/opt` .env) |
| `TMP_DIR` | No | `./tmp` | Temp directory for exports |
| `TRUSTED_PROXIES` | No | — | Comma-separated trusted proxy subnets (XFF) |
| `IP_TAG_SOURCES` | No | — | Datacenter IP list sources |
| `MAXMIND_DB` | No | — | MaxMind GeoLite2-ASN `.mmdb` path (datacenter blocking; unset = fail-open) |
| `JWT_SECRET` | Yes (auth) | — | JWT signing secret |
| `OPDS_BASE_URL` | No | — | Canonical OPDS base URL |
| `OPDS_SHELF_TOKEN` | No | — | OPDS shelves token |
| `RUST_LOG` | No | `info,fichub=debug` | Tracing filter |

**Ollama** (Ask-the-Archive + embeddings): `OLLAMA_URL`, `OLLAMA_CHAT_MODEL`, `OLLAMA_EMBED_MODEL`, `SEARCH_MAX_PER_PAGE`.

**Self-heal agent** (`src/heal/`): `AGENT_ENABLED`, `AGENT_API_URL`, `AGENT_API_KEY`, `AGENT_MODEL`, `AGENT_OLLAMA_URL`, `AGENT_USE_ON_FLY`, `AGENT_COOLDOWN_DOMAIN_SECS`, `AGENT_MAX_RUNS_PER_DAY`, `AGENT_EXTRACT_MAX_SNAPSHOT_CHARS`.

**Recommender** (`REC_*`): `REC_STRATEGIES` (default `cooccur`), `REC_ENGINE_MODE`, `REC_DEFAULT_DELAY_SECS`, `REC_SITE_RATE_LIMITS` (JSON), `REC_MAX_FAVOURITE_PAGES`, `REC_MAX_USER_FAVOURITE_PAGES`, `REC_MAX_RECOMMENDATIONS`, `REC_MIN_FAVOURITERS_FOR_COLLAB`, `REC_VOTING_BOOST_GAMMA`, `REC_CACHE_TTL_HOURS`, `REC_SUGGEST_LIMIT_PER_HOUR`, `REC_VOTE_LIMIT_PER_HOUR`, `REC_PRECOMPUTE_ENABLED`, `REC_PRECOMPUTE_INTERVAL_HOURS`, `REC_ENABLE_CROSS_SITE`, `REC_DECAY_HALFLIFE_DAYS`, `REC_EMBED_MODEL` (default `nomic-embed-text`), `REC_EMBED_DIM`, `REC_MF_FACTORS`, `REC_MF_ITERS`, `REC_MF_TRAIN_MIN_SIGNALS`, `REC_TRAIN_EVERY_H`, `REC_BANDIT_SLOTS`, `REC_CURATOR_PRIOR`, `REC_CURATOR_TAU`, `REC_PRIOR_FLOOR`, `REC_SHADOW_MODE`, `REC_STRATEGY_TIMEOUT_SECS`, `REC_EXTERNAL_URL`.

**Proof-of-work**: `POW_DIFFICULTY`, `POW_TTL_SECS`.

**Rate-limit tiers** (`RL_*`): `RL_TIERED_ENABLED`, `RL_DOWNLOAD_CAPACITY`, `RL_DOWNLOAD_FLOW`, `RL_SEARCH_CAPACITY`, `RL_SEARCH_FLOW`, `RL_AUTH_CAPACITY`, `RL_AUTH_FLOW`, `RL_CLIENT_BONUS_CAPACITY`, `RL_CLIENT_BONUS_FLOW`, `RL_NAT_MULTIPLIER`, `RL_SHADOWBAN_CAPACITY`, `RL_SHADOWBAN_FLOW`, `RL_SHADOWBAN_TTL`.

**Tags**: `TAG_AUTO_DELETE_THRESHOLD`, `TAG_HIDDEN_THRESHOLD`, `TAG_SUBMIT_LIMIT_PER_HOUR`, `TAG_VOTE_LIMIT_PER_HOUR`.

**Forum**: `FORUM_ADMIN_LEVEL`, `FORUM_CURATOR_LEVEL`, `FORUM_EXP_MOD_DAILY_CAP`, `FORUM_EXP_MOD_RECEIVED`, `FORUM_EXP_PER_LEVEL`, `FORUM_EXP_POST_CREATE`, `FORUM_EXP_TOPIC_CREATE`, `FORUM_META_*` (window, cooldown, min age/exp/level/posts/rated, pool, ratings, unfair rate, audit window), `FORUM_MOD_MIN_*`, `FORUM_POINTS_PER_WINDOW`, `FORUM_PUBLIC_READ`, `FORUM_WINDOW_HOURS`.

**Kindle/email**: `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASS`, `SMTP_FROM`.

**Misc**: `CURATOR_TOKEN`, `COMMANDCODE_API_KEY`, `AGENT_API_KEY` (see above), `OPDS_BASE_URL`.

---

## Appendices

### A. Related Repositories / Crates

| Component | Path | Language | Purpose |
|-----------|------|----------|---------|
| fichub (the whole system) | `~/code/rust/fichub/` (i.e. `./`) | Rust + SvelteKit | Backend + frontend + scrapers + bot (THIS DOCUMENT) |
| fanfic-scrapers | `./scrapers/` | Rust | Site-specific scrapers (AGPL-3.0), pulled in via `[dependencies]` path |
| forum-core | `./forum_core/` | Rust | Generic forum engine (AGPL-3.0), path dependency |
| fanfic-archivist | `./fanfic-archivist/` | Rust | Chat-bot integration (Discord/Telegram/Matrix) |
| rec-engines | `./rec-engines/` | Python | Optional sidecar recipes for the extension platform |
| Calibre container | `./docker/calibre.Dockerfile` | — | PDF/MOBI/AZW3 conversion sidecar |

Historical (retired): `~/code/go/fichub-cli.bak/` (Go v1 backend), `~/code/go/fichub-frontend/` (old frontend location), `threadlight`, `~/.pi` files — none are in the current architecture.

### B. Common Commands

```bash
# Build the server (default-run = fichub) or a helper bin
cd /personal/documents/code/rust/fichub && cargo build --release --bin fichub            # web server
cargo build --release --bin saved-search-watcher --bin migrate                            # helper bins

# Run (dev)
cargo run                  # runs the fichub bin (PORT / .env from CWD)

# Frontend rebuild (output served by the binary)
cd frontend && npm install && npm run build

# Deploy (build on gamingpc → copy to /opt → migrate → restart → health check + QA)
./deploy.sh                # or ./deploy.sh --skip-build

# Tests / QA
cargo test                 # backend (incl. server::tests, search parser/ask tests)
cd frontend && npm test    # vitest
cargo llvm-cov --lib       # backend coverage (≥30% line)
npm run coverage           # frontend coverage (≥85/78/76 on src/lib)

# Check service
journalctl -u fichub -n 50 --no-pager
curl http://localhost:8000/api/health

# QA from dev machine via tunnel
ssh -L 18000:localhost:8000 -N -f thinkcentre
QA_BASE=http://localhost:18000 node qa/run.js
```

### C. Git History (This Repo)

The repo is the single home for the current system (see `git log` for the full history; the older `SPECIFICATION.md` described a Photon-fork frontend and Go backend in separate trees — superseded). Key lifecycle milestones recorded in `docs/`:

- Deployment moved to local `/opt/fichub` (2026-08-14) to avoid NFS SIGBUS (`docs/DEPLOYMENT.md`).
- Migrations 002–007 added follow exclusions, search v2 (polarity / role confidence / counters), collection submissions, and saved searches.
- Ask the Archive rewritten to emit v2 search-query strings (`src/search/ask.rs`).
- `001_initial.sql` consolidated base schema and `_sqlx_migrations` checksum handling standardized.
