<!-- CONSOLIDATED BRAINSTORM FILE — Generated 2026-08-13, updated 2026-08-17 from repo markdown (docs/src user docs excluded). -->

> FicHub is a self-hosted fanfiction archive + download server (Rust/Axum backend, SvelteKit SPA, PostgreSQL+pgvector, Redis, Ollama). This file is the SYSTEM OVERVIEW: README (features/quickstart/endpoints), the full SPECIFICATION (architecture, routes, schema, scraper system, export system, recommender), the v2 DESIGN doc (normalization, translations, vector search), and the deployment handover. NOTE: SPECIFICATION contains a historical Go-backend section (chi/pgx/Typesense) — the Rust backend is the source of truth; the Go stuff is legacy context only.


---


======================================================================
SOURCE: README.md
======================================================================

# FicHub — Rust Backend + SvelteKit SPA

Self-hosted fanfiction archive and download server (fichub.net replacement).  
Scrapes fanfiction sites, generates EPUB, HTML, MOBI, PDF, AZW3, TXT, Markdown. Serves a full community platform via HTTP API: user accounts, bookmarks, 5-star ratings, reviews, threaded comments, Fic Requests (prompt board), reading lists, follows + updates feed, series/author pages, RSS/Atom feeds, in-browser reader, roadmap consensus, and an anti-bot defense system.

**Repository**: https://opencommit.eu/MagicZhang/fichub.git (public mirror)  
**Stack**: Rust + Axum 0.8 + SQLx 0.9 + PostgreSQL + Redis + Ollama (embeddings) + SvelteKit 5 (frontend)

---

## Quick Start
Prerequisites: PostgreSQL, Redis, Ollama (for consensus/auto-tagger), Rust toolchain, Node.

```bash
cp .env.example .env     # Edit with your DB/Redis URLs
cargo run                # Starts on :8000 (or $PORT)
```

## Architecture
```
User/Script → Axum HTTP server (:8000)
        │
        ├─► Redis ─────────────── (rate limiter token bucket, shadowban)
        ├─► PostgreSQL ────────── (works, fic metadata, social, search, requests, lists)
        ├─► Ollama ────────────── (nomic-embed-text: roadmap consensus, auto-tagger, rec embeddings)
        ├─► Scraper subsystem ─── (fanfic-scrapers crate: native adapters for all 107
        │    FFF-parity sites; login creds per site, no Python CLI dependency)
        ├─► EPUB/TXT/MD builder ── (pure Rust, epub-builder + txt/md export)
        ├─► Calibre sidecar ───── (MOBI/PDF/AZW3 conversion via Docker)
        ├─► Rec strategy registry (REC_ENGINE_MODE=pluggable: RRF-blended
        │    strategies — cooccur/decay/embeddings/mf/hybrid/author_graph/
        │    tag_graph/sequential/clusters/bandit/external; legacy mode
        │    preserves the current engine exactly)
        └─► Filesystem cache ─── (hash-based directory tree)
        └─► ServeDir fallback ─── (frontend/build SPA, single origin)
```

## Feature Highlights

- **5-star ratings + in-depth reviews** — positive-only public surface; no dislikes shown; reviews/reactions feed the recommendation engine (`work_feedback_signals`).
- **Web reader** (`/read/[urlId]`) — typography prefs, chapter nav, scroll progress, position save, "Next Up" panel (next-in-series → top community suggestion → readers-also-bookmarked, backed by real `/api/reader/{url_id}/sequel` + `/related` endpoints).
- **Fic Requests** (`/requests`) — prompt board; works-only answers (work_id, a **pasted fic URL** — URL-ingest — or an **ask result's url_id**), community fit votes, request-level upvotes, requester accept + notifications on answer/accept; "Request similar" button on every fic page. **Ask × Requests**: an empty ask becomes a request in one click; every request page has an "Ask the Archive" box that surfaces in-library matches as one-click answers.
- **Reading lists + shelves** (`/lists`, `/shelves`) — curated bundles, positioned items, blurbs.
- **Follows + updates feed** — follow fics/authors/users; `/api/v1/feed`; refresh-fic re-scrape with follower notifications.
- **Series & author pages** (`/series/[id]`, `/authors/[id]`) — ordered works, next-in-series.
- **RSS/Atom feeds** — `/feed.xml` (new arrivals), `/feed/follows.xml` (token), per-fic feeds.
- **PWA offline reader** — service worker caches reader responses (stale-while-revalidate) + offline banner.
- **Roadmap consensus** (`/roadmap`) — MaxDiff/Elo arena over pgvector-embedded feature clusters (Ollama nomic-embed-text, 768-dim); public consensus leaderboard + most-controversial list.
- **Advanced search** — boolean AND/OR/NOT, phrases, fielded search, main_char_attr (AO3 "Dark Harry" semantics), typo tolerance, tag filters, kudos/comment bounds.
- **Full-text search over fic bodies** (`/search/body`) — quote search ("fics where X says Y") over the cached fic bodies (`body_text_search` tsvector, migration 040, maintained at body-write time + a backfill bin); highlighted `<mark>` snippets. No scraper-only archive offers this.
- **Ask the Archive** (`/ask`) — natural-language queries ("completed slow-burn Dramione over 50k, no major character death") converted by an LLM into search filters, with graceful fallback when the model is down; an empty ask turns into a Fic Request in one click.
- **User-supplied site credentials** (`/settings`) — logged-in users can provide credentials for login-requiring sites (fanfics.me, fictionhunt, inkbunny, sofurry, dokuga) — opt-in, encrypted at rest (AES-256-GCM), 30-day expiry; downloads use the requesting user's creds via `lookup_authed`.
- **Similar-fic suggestions** — on every fic page, suggest in-archive stories (autocomplete) or paste a link to be scraped; readers up/down-vote so the best matches rank first.
- **Pluggable recommendation platform** — strategy registry (co-occurrence, time-decayed SAR, pgvector embeddings, implicit MF, tag/author graphs, sequential, bandit exploration, curator prior) behind a single config knob; today's engine is the default `cooccur` strategy.
- **Transparent modlog** (`/modlog`) — every moderator/curator/admin action recorded and readable by any logged-in user; moderation is fully transparent.
- **Non-PII usage analytics** (`/admin/analytics`) — unique daily/weekly/monthly visitors, active vs view-only users, action timeline, search→export conversion; anonymous client IDs only, never IPs.
- **Admin UI** — `/admin/*` pages for auto-tag review, upload moderation, blacklist, bots, comment triage, scrapers, analytics, **translation review** (approve/reject/edit ML translations) and **metadata correction** (fix canonical title/author/status/description, synced to the default source).
- **Site-as-cache** — every scraped fic body persisted on the attached drive (`BODY_CACHE_DIR`), instant repeat exports, curator peer-voted body fixes.
- **Anti-bot** — honeypot traps, tiered rate limits, Redis shadowban, proof-of-work, hourly bot-scorer, `/admin/bots`.
- **Community forum** (`/forum`) — categories, topics, posts, follows, notifications, read state, FTS search with snippets, **moderation points** (queue, reports-as-signal, curator fast-hide), **metamoderation** (anonymized audit, unfair-rate, cooldowns), **site-wide leveling** (0–100 levels + exp), invites + registration applications, user blocks; topics get canonical **slug URLs** (`/forum/board/{topic_slug}.{topic_id}`, legacy `/forum/{cat}/{id}` still works). Engine = the `forum-core` crate; contract in `docs/FORUM-API-CONTRACT.md` + `docs/SPEC-COMMUNITY-PLATFORM.md`.
- **Command palette** (`Ctrl+K`/`Cmd+K`) — jump to any page or search the in-app docs from anywhere; **help modal** — every docs section opens in-app with an "Ask the docs" box that answers questions in plain English.
- **Bookmarklet** — one-click download from any story page (grab it from the Download tab).
- **Send-to-Kindle** — email an EPUB of any fic to your Kindle address.
- **Badges, quests & streaks** — achievements, monthly reading goals → badges, consecutive-day reading/download streaks with milestones ("You've read 1.2M words this year").
- **Leaderboards** — top contributors by exp, reading stats, and community activity.
- **Work proposals** — community-driven merge/split proposals for duplicate works, peer-voted.
- **Self-healing scraper telemetry** — scrape-failure tracking with domain health + retry (see `docs/src/self-healing.md`).
- **Nightly QA harness** — `qa/run.js` deterministic local-first checks (API walk 97/97 green) + CI coverage gates.
- **OPDS catalog** (`/opds/*`) — subscribe to the archive from e-reader apps.
- **Blind Date** — random-fic discovery button (hides title/fandom until reveal).
- **Trending tags & popular fics** — time-decayed trending sort on search + home.
- **Search suggestions + tag autocomplete** — as-you-type tag completion with usage counts, and typo-tolerant suggestions.
- **API route-walk** (`qa/api-walk.js`) — scriptable e2e audit that enumerates every route from the router, asserts auth gates, and flags 5xx/stub responses (97/97 green).

## Unified Works Model

FicHub uses a work-centric architecture where a "work" is the abstract story
and a "source" is a concrete URL (e.g. AO3, FFN). The same story posted on
multiple sites appears as a single canonical entry.

- **works** table: auto-increment PK, canonical title/author, description
- **fic_info**: sources linked to works via `work_id` FK
- **Auto-merge**: exact title+author match with word count tolerance
- **Curator proposals**: community-driven merge/split with voting
- **Reputation**: contributors earn points, auto-promote to curator

## API Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/epub?q=<url>` | Export EPUB for a fic URL |
| GET | `/api/health` | Health check (DB + Redis ping) |
| GET | `/api/meta?q=<url>` | Get fic metadata |
| GET | `/api/search?q=...` | Advanced search with filters |
| GET | `/api/search/body?q=...` | Full-text search over fic bodies (quote search, `<mark>` snippets) |
| POST | `/api/search/ask` | Ask the Archive: NL query → search filters → results |
| GET | `/api/recommendations?url_id=&n=` | Collaborative-filtering recs |
| GET | `/api/recommendations/personal` | Personal recs (strategy registry when enabled) |
| GET | `/api/v1/works/{url_id}/also-bookmarked` | Co-occurrence rec anchors |
| GET | `/api/fic-suggestions?url_id=` | Per-fic similar-fic suggestions (votes) |
| POST | `/api/fic-suggestions` | Suggest a similar fic (in-DB or URL scrape) |
| GET | `/api/roadmap/consensus` | Public feature consensus leaderboard |
| POST | `/api/bookmarks` | Add bookmark (work_id) |
| GET | `/api/bookmarks` | List user's bookmarks |
| DELETE | `/api/bookmarks/{work_id}` | Remove bookmark |
| POST | `/api/ratings` | Rate a work (1..=5) |
| GET | `/api/ratings/{work_id}` | Get aggregate ratings (positive-only) |
| POST | `/api/reviews` | Upsert in-depth review |
| GET | `/api/works/{id}/reviews` | List reviews |
| POST | `/api/comments` | Post comment (work_id) |
| GET | `/api/comments/{work_id}` | List comments |
| POST | `/api/follows` | Follow user/work/author |
| GET | `/api/follows` | List my follows |
| GET | `/api/v1/feed` | Follow updates feed |
| POST | `/api/requests` | Create fic request |
| GET | `/api/requests` | List requests (top/new; top ranks by upvotes) |
| POST | `/api/requests/{id}/answers` | Add work answer (work_id OR a fic URL — URL-ingest) |
| POST | `/api/requests/{id}/answers/{aid}/vote` | Vote on answer |
| POST | `/api/requests/{id}/upvote` | Upvote a request (toggle, no self-vote) |
| POST | `/api/requests/{id}/accept/{aid}` | Accept answer |
| GET | `/api/requests/{id}/candidates` | Engine suggestions for answering a request (seed work) |
| GET | `/api/modlog` | Public moderation log (any logged-in user; `?action=` filter) |
| GET | `/api/admin/analytics` | Usage analytics dashboard (role ≥ 10) |
| GET | `/api/admin/search-analytics` | Search analytics: zero-result queries, trope popularity, search→export conversion |
| GET | `/api/admin/translations` | Translation review queue (role ≥ 10; approve/reject/edit) |
| GET | `/api/admin/works/{id}/metadata` | Fetch work metadata for correction |
| PUT | `/api/admin/works/{id}/metadata` | Correct canonical work metadata (syncs default source) |
| GET | `/api/curator/content/{url_id}` | Inspect cached body (curator) |
| POST | `/api/curator/content/{url_id}/propose` | Propose a body fix (peer-voted) |
| POST | `/api/curator/content/proposals/{id}/vote` | Vote on a body-fix proposal |
| GET | `/api/series/{id}` | Series detail + works |
| GET | `/api/authors/by-name/{name}` | Author bibliography |
| GET/POST | `/api/lists` | Reading lists |
| GET | `/api/reader/{url_id}` | Reader HTML bundle |
| GET | `/api/reader/{url_id}/sequel` | Next-in-series for "Next Up" (series_works, numeral fallback) |
| GET | `/api/reader/{url_id}/related` | "Readers also bookmarked" co-occurrence |
| PUT/DELETE/GET | `/api/user/site-credentials` | User-supplied site creds (opt-in, 30-day, encrypted) |
| GET | `/feed.xml` | New arrivals Atom feed |
| GET | `/feed/follows.xml` | Follows Atom feed |
| GET | `/api/roadmap/arena` | Roadmap consensus arena |
| POST | `/api/roadmap/vote` | Cast arena comparison |
| GET | `/opds/*` | OPDS catalog feeds |
| GET | `/api/forum/categories` | Forum: list visible categories + topic/unread counts |
| GET | `/api/forum/topics?category={slug}&cursor={id}&limit=` | Forum: topic list by category, cursor-paginated |
| POST | `/api/forum/topics` | Forum: create topic `{title, category_slug, body, payload?}` (generates `topic_slug`) |
| GET | `/api/forum/topics/{topicId}` | Forum: topic detail + OP + posts (`?after={postId}` cursor) |
| GET | `/api/forum/topics/by-slug/{topicSlug}` | Forum: topic detail resolved by slug (same shape as `{topicId}`) |
| PATCH/DELETE | `/api/forum/topics/{topicId}` | Forum: edit/soft-delete topic (author ≤15min or mod) |
| POST | `/api/forum/topics/{topicId}/posts` | Forum: reply `{body, payload?, quote_of?}` |
| PATCH/DELETE | `/api/forum/posts/{postId}` | Forum: edit/soft-delete post (author ≤15min or mod) |
| POST | `/api/forum/topics/{topicId}/follow` | Forum: toggle follow |
| GET | `/api/forum/topics/{topicId}/follow` | Forum: my follow state + follower count |
| POST | `/api/forum/topics/{topicId}/read` | Forum: mark read `{last_read_post_id:N}` |
| GET | `/api/forum/search?q=&category={slug}` | Forum: FTS over topics+posts, snippets |
| GET | `/api/forum/moderation/status` | Forum: my points, window expiry, eligibility |
| GET | `/api/forum/moderation/queue` | Forum: mods-needed posts (reported + low-score first) |
| POST | `/api/forum/posts/{postId}/moderate` | Forum: moderate `{reason}` — server maps reason→delta, spends 1 pt |
| GET | `/api/forum/posts/{postId}/moderations` | Forum: public mod history |
| GET | `/api/forum/metamod/queue` | Forum: N random under-rated actions, moderator anonymized |
| POST | `/api/forum/metamod/{actionId}/vote` | Forum: `{verdict: fair\|unfair\|unsure}` |
| POST | `/api/admin/forum/hide/{postId}` | Forum: level ≥ curator fast-hide, 72h auto-expiry |
| POST | `/api/admin/forum/topics/{topicId}/lock` `/pin` | Forum: level ≥ curator |
| POST/DELETE/GET | `/api/admin/forum/bans` | Forum: scoped bans (forum\|category, scope ≤ own level) |

See [`docs/brainstorm-01-system-overview.md`](./docs/brainstorm-01-system-overview.md) for the complete endpoint reference.

### Supported Sites (Scrapers)

The [`fanfic-scrapers` crate](./scrapers/README.md) ships **native adapters
with full FanFicFare adapter parity — all 107 real FFF sites**. The Python
FanFicFare CLI dependency has been **removed**: the `fanficfare` fallback
adapter is gated behind the crate's `fff-fallback` feature (off by default);
FicHub registers native adapters only.

- **Major archives**: Archive of Our Own + OTW siblings (adastrafanfic,
  cfaa, squidgeworld, superlove), FanFiction.net, FictionPress, RoyalRoad,
  ScribbleHub, FimFiction, Literotica, Wattpad, DeviantArt, FicBook,
  Quotev, AsianFanFics, Syosetu, Kakuyomu, MediaMiner, ChiReads,
  PMDFanFiction, SpiritFanfiction, Fanfictions.fr, Twisting the Hellmouth
- **Forum/XenForo boards**: SpaceBattles, SufficientVelocity,
  QuestionableQuesting, theforce.net, fiction.live, AlternateHistory,
  Althistory, The Sietch
- **eFiction + variants**: the classic eFiction family (19 archives) and
  the `viewstory.php?sid=` eFiction-variant family (26 archives: psychfic,
  wolverineandrogue, sycophanthex sites, walkingtheplank, themasque,
  ksarchive, twilighted, whofic, …)
- **WordPress-novel sites**: novelfull + family; StoriesOnline family
  (storiesonline, scifistories, storyroom)
- **Adult-gated archives** (require the `is_adult` flag or site login):
  fanfics.me, fictionhunt, inkbunny, sofurry, dokuga, asexstories,
  aneroticstory, mcstories, hentaifoundry, bdsmlibrary, readonlymind,
  utopiastories
- **And many more bespoke adapters**: HPFanficArchive, adultfanfiction.org,
  LCFanFic, FireflyFans, FicWad, FictionMania, FanFiktion.de,
  TouchFluffyTail, FanFicAuthors, PhoenixSong, StoriesOfArda, Fictionalley,
  NovelAll, MassEffect2.in, … (see [`scrapers/README.md`](./scrapers/README.md)
  and [`scrapers/FFF_PARITY.md`](./scrapers/FFF_PARITY.md))

> **Host limitation**: from this host, outbound fetches to some sites are
> blocked (AO3 returns 404 + bot-challenge, FFN 403 Cloudflare) — a strategic
> gap tracked in the roadmap (user-supplied cookie ingestion, P1). All other
> sites are reachable via their native adapter.

## Spec

See [`docs/SPEC-COMMUNITY-PLATFORM.md`](./SPEC-COMMUNITY-PLATFORM.md) for the
community-platform spec (forum + moderation + leveling), and
[`docs/brainstorm-01-system-overview.md`](./brainstorm-01-system-overview.md)
for the full system overview — architecture, schema, API endpoints, and
deployment notes.

## Testing & Coverage

```bash
# Backend (Rust)
cargo test --lib                 # unit tests (fast, no DB)
cargo test -- --include-ignored  # + DB-gated integration tests (needs .env DB/Redis)
cargo llvm-cov --lib             # line coverage report (install: cargo install cargo-llvm-cov)

# Frontend (SvelteKit / vitest)
cd frontend
npm test                         # unit tests
npm run test:e2e                 # E2E/integration suite (real layout + nav)
npm run coverage                 # coverage report + gate (thresholds in frontend/vite.config.ts)

# QA harness (local-first, deterministic; see docs/AGENTS.md)
node qa/run.js
```

**CI coverage gate** (`.forgejo/workflows/ci.yml`):
- `frontend-coverage` — `npm run coverage`; fails below 85% lines / 78% funcs / 76% branches on `src/lib/**` (raised from 65/55/60 in the 2026-08-09 coverage wave when src/lib hit 90.3% lines / 82.66% funcs).
- `backend-coverage` — `cargo llvm-cov --lib`; fails below 30% line coverage.
- Both publish reports as downloadable artifacts (`frontend-coverage`, `backend-coverage`) and a combined summary on each run. Raise thresholds as coverage grows toward the 95/90 target.

## Deployment
```bash
cargo build --release
sudo systemctl restart fichub.service   # User=alvaro, WorkingDirectory=/personal/documents/code/rust/fichub
```

The `fichub.service` systemd unit runs the release binary on port 8000
(see `PORT` in `.env`), serving the API (`/api/*`), docs (`/docs/*`), and
the static SvelteKit frontend (`FRONTEND_DIR=./frontend/build`) on one
port. The public domain `fichub.polarisocial.xyz` is fronted by Cloudflare.

Frontend rebuild (after `frontend/src` changes):
```bash
cd frontend && npm run build    # writes frontend/build (served by the binary)
```

### Database migrations
Migrations live in `migrations/` and are applied by sqlx **automatically on
service start** (`sqlx::migrate!`). Never insert rows into
`_sqlx_migrations` by hand: sqlx validates each migration's checksum against
the recorded value and panics with `Migrate(VersionMismatch(N))` if they
differ (this happened with migration 008 — a manually inserted row with an
empty checksum broke startup until it was deleted so sqlx could re-apply the
file). If you hit that, verify the file is what you want, then:
```sql
DELETE FROM _sqlx_migrations WHERE version = N;
-- restart the service; sqlx re-applies migrations/NNN_*.sql (keep it idempotent)
```
New migrations must be idempotent (`CREATE TABLE IF NOT EXISTS`,
`ON CONFLICT DO NOTHING`) so a re-apply is safe.

## License
AGPL-3.0-only


======================================================================
SOURCE: docs/SPECIFICATION.md
======================================================================

# FicHub — Complete System Specification

> **Generated**: 2026-07-25 (updated: 2026-08-17 — unified Rust+SPA stack on :8000, migrations 001–053, feature wave: feedback rework, web reader, roadmap consensus, Fic Requests M1-M3 (upvotes, URL-ingest, notifications), reading lists, series/authors, RSS/Atom, PWA offline, self-healing M1+M2 (on-the-fly scraper), site-as-cache body blobs, usage analytics, transparent modlog, admin UI (auto-tag/moderation/translations/metadata correction), API route-walk audit, FULL FanFicFare adapter parity in the `fanfic-scrapers` crate (v0.10.0, 107 native sites + login/is_adult support), contributing docs chapter, kudos, v3 progression system + extension platform, recipes/themes, FFN metadata fallback)
> **Author**: hirrolot19
> **Purpose**: Single-source-of-truth for LLM-assisted refactoring and development queries.

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Repository Layout](#2-repository-layout)
3. [Architecture & Data Flow](#3-architecture--data-flow)
4. [Backend: Rust (Primary API)](#4-backend-rust-primary-api)
5. [(Historical) Go v1 Backend](#5-backend-b-go-v1-api)
6. [Frontend: SvelteKit SPA](#6-frontend-sveltekit-spa)
7. [Database Schema](#7-database-schema)
8. [Deployment](#8-deployment)
9. [API Endpoint Reference](#9-api-endpoint-reference)
10. [Data Flow Examples](#10-data-flow-examples)
11. [Current Issues & Technical Debt](#11-current-issues--technical-debt)
12. [Environment Variables](#12-environment-variables)

---

## 1. System Overview

FicHub is a self-hosted fanfiction archive combining a **single Rust/Axum backend** with a **SvelteKit SPA frontend**, both in one repo (`/personal/documents/code/rust/fichub/`):

- **Rust backend** — serves the full API (EPUB export, scraping, metadata, caching, recommendations, OPDS, tags, search, user accounts, bookmarks, ratings, reviews, threaded comments, leaderboards, follows, notifications, Fic Requests, reading lists, series/authors, RSS/Atom feeds). Runs on **port 8000** via systemd (`fichub.service`). Single binary, no external job queue.
- **SvelteKit SPA frontend** (`frontend/`) — Svelte 5 runes, built via `@sveltejs/adapter-static` (SPA mode, `ssr=false`) into `frontend/build/`, served by the Rust binary's `ServeDir` fallback.
- **PostgreSQL** — `fichub` database, migrations **001–053** (sqlx-managed).
- **Redis/Valkey** — rate limiting (token bucket), shadowban, caches.
- **Ollama** — local `nomic-embed-text` (768-dim) for roadmap consensus embeddings + auto-tagger.
- **Domain**: fichub.polarisocial.xyz (this host, `fichub.service` on :8000).

### Current Deployment Status

| Component | Location | Port | Status |
|-----------|----------|------|--------|
| Rust backend + SPA | This host (fichub.polarisocial.xyz) | 8000 | ✅ Running (systemd `fichub.service`) |
| Database (PostgreSQL) | localhost | 5432 | ✅ Running |
| Redis / Valkey | localhost | 6379 | ✅ Running |
| Ollama (embeddings) | localhost | 11434 | ✅ Running (nomic-embed-text) |
| Public mirror | opencommit.eu/MagicZhang/fichub.git | — | ✅ `github` remote |
| (Historical) Go v1 backend / threadlight | Orange Pi (192.168.1.138) | 8004/8005/8086 | ❌ Not part of this stack — superseded |

### Key Design Decisions

- **Single Rust codebase** — the Go backend (`fichub-cli.bak`) and threadlight are historical; the Rust backend is the one source of truth.
- **SPA mode** — SvelteKit `ssr=false` + `prerender=false`; the backend serves `frontend/build/` via ServeDir with `index.html` fallback.
- **Svelte 5 runes** — all state via `$state`, `$derived`, `$effect` (auth uses a store singleton).
- **sqlx migrations** — schema changes are numbered `001`→`053`; the binary runs migrations on boot (must stay in sync with `_sqlx_migrations`).
- **DB-gated tests** — integration tests marked `#[ignore]`, run with `cargo test --test <suite> -- --include-ignored --test-threads=1`; self-healing seeds (unique prefixes).
- **Delegation-friendly** — features are developed in separate git worktrees (external drive `/media/alvaro/code-worktrees`), merged to `main` only when fully green.
- **Test coverage gate** — frontend `npm run coverage` enforces thresholds on `src/lib` (85% lines / 78% funcs / 76% branches; achieved 90.3% lines 2026-08-09).

---

## 2. Repository Layout

### Frontend: `~/code/go/fichub-frontend/`

```
fichub-frontend/
├── build/                    # Build output (generated)
├── e2e/                      # Playwright E2E tests
│   └── fichub.spec.ts
├── node_modules/             # Dependencies
├── src/
│   ├── app.css               # Global styles (Tailwind)
│   ├── app.d.ts              # App type declarations
│   ├── app.html              # SvelteKit HTML shell
│   ├── hooks.client.ts       # Client hooks
│   ├── hooks.server.ts       # Server hooks
│   ├── lib/
│   │   ├── api/
│   │   │   ├── base.ts                  # Base API URL config
│   │   │   ├── client.svelte.ts         # Legacy API client wrapper
│   │   │   ├── fichub-client.svelte.ts  # Main API client (FichubClient class)
│   │   │   └── fichub-types.ts          # All TypeScript types
│   │   ├── app/
│   │   │   ├── error.ts                 # Error handling utilities
│   │   │   ├── i18n/index.ts            # Internationalisation setup
│   │   │   ├── instance.svelte.ts       # Instance configuration
│   │   │   ├── settings.svelte.ts       # User settings (localStorage-persisted)
│   │   │   ├── theme/
│   │   │   │   ├── presets.ts           # Theme presets
│   │   │   │   └── theme.svelte.ts      # Theme state management
│   │   │   └── util.svelte.ts           # General utilities
│   │   ├── stores/
│   │   │   └── auth.svelte.ts           # Auth store (Auth class with $state)
│   │   └── ui/
│   │       ├── form/                    # Form components (from Photon)
│   │       ├── generic/                 # Generic components (Avatar, Logo, etc.)
│   │       ├── icon/photon.ts           # Photon icon mappings
│   │       ├── info/                    # Info components
│   │       ├── layout/                  # Layout components (Shell, Header, Tabs)
│   │       ├── navbar/                  # Navigation bar + command palette
│   │       ├── shared/                  # Shared UI library ("mono-svelte" alias)
│   │       ├── sidebar/                 # Sidebar component
│   │       ├── text/                    # Text components
│   │       └── util/                    # Utility components
│   ├── params/
│   │   └── integer.ts                   # Route parameter matcher
│   └── routes/
│       ├── +error.svelte               # Error page
│       ├── +layout.svelte              # Root layout (Shell + Navbar + Sidebar)
│       ├── +layout.ts                  # Layout loader
│       ├── +page.svelte                # Home page ("Welcome to FicHub")
│       ├── +page.ts                    # Home page loader
│       ├── [...slug]/+page.ts          # Catch-all route (404 handler)
│       ├── browse/+page.svelte         # Browse fandoms
│       ├── error/+page.ts              # Error page loader
│       ├── fics/[id]/
│       │   ├── +page.svelte            # Fic detail page
│       │   └── +page.ts                # Fic detail loader
│       ├── login/+page.svelte          # Login page
│       ├── popular/
│       │   ├── +page.svelte            # Popular fics page
│       │   └── +page.ts                # Popular fics loader
│       ├── recent/
│       │   ├── +page.svelte            # Recent fics page
│       │   └── +page.ts                # Recent fics loader
│       ├── search/
│       │   ├── +page.svelte            # Search page with filters
│       │   └── +page.ts                # Search loader
│       ├── tags/+page.svelte           # Tags browser
│       └── u/[id]/
│           ├── +page.svelte            # User profile page
│           └── +page.ts                # User profile + bookmarks loader
├── static/
│   ├── favicon.png
│   ├── font/Inter.woff2
│   ├── font/RobotoSlab.woff2
│   ├── img/                            # Logo assets (from Photon)
│   ├── manifest.json
│   └── robots.txt
├── FEATURES.md                         # Feature documentation
├── playwright.config.ts                # Playwright config
├── package.json                        # Dependencies & scripts
├── svelte.config.js                    # SvelteKit config
├── vite.config.ts                      # Vite build config
└── tsconfig.json                       # TypeScript config
```

### Backend Rust: `~/code/rust/fichub/`

```
fichub/
├── Cargo.toml
├── .env                                # Local environment
├── migrations/
│   ├── 001_initial_schema.sql          # Core tables
│   ├── 002_add_client_tracking.sql     # Client tracking
│   ├── 003_follows_notifications_gamification_vector_translations.sql
│   ├── 004_shelves_reading_status.sql
│   ├── 005_manual_uploads.sql
│   ├── 006_chapter_translations.sql
│   ├── 007_admin.sql
│   ├── 008_author_merging.sql
│   ├── 009_bot_tracking.sql
│   ├── 010_search_analytics.sql
│   ├── 011_roadmap_consensus.sql       # feature_clusters + pgvector
│   ├── 012_search_typo_tolerance.sql
│   ├── 013_add_kindle_email.sql
│   ├── 014_feedback_rework.sql         # 5-star ratings, reviews, constructive comments
│   ├── 015_auto_tagger.sql
│   ├── 016_roadmap_consensus_statuses.sql
│   ├── 017_follow_updates.sql
│   ├── 018_fic_requests.sql            # Fic Requests prompt board
│   ├── 019_reading_lists.sql
│   └── 020_series_authors.sql
├── src/
│   ├── main.rs                         # Entry point
│   ├── lib.rs                          # Module declarations
│   ├── config.rs                       # Config struct + env loading
│   ├── server.rs                       # Axum router + server startup
│   ├── error.rs                        # AppError enum + IntoResponse
│   ├── roadmap_seed.rs                 # PROPOSED/SHIPPED feature lists for consensus
│   ├── db/
│   │   ├── mod.rs                      # Pool initialisation
│   │   ├── models.rs                   # DB row structs
│   │   ├── queries.rs                  # SQL queries as functions
│   │   └── reviews.rs                  # work_feedback_signals() rec signals
│   ├── routes/
│   │   ├── mod.rs                      # Route module declarations
│   │   ├── export.rs                   # GET /api/v0/epub handler
│   │   ├── meta.rs                     # GET /api/v0/meta handler
│   │   ├── cache_download.rs           # Cache download handlers
│   │   ├── comments.rs                 # Threaded comments + constructive filter
│   │   ├── social.rs                   # Bookmarks, ratings (1..=5), comments
│   │   ├── reviews.rs                  # In-depth reviews CRUD
│   │   ├── requests.rs                 # Fic Requests (prompt board)
│   │   ├── series.rs                   # Series + author bibliography
│   │   ├── rss/                        # Atom feeds (new arrivals, follows, per-fic)
│   │   ├── follows.rs                  # Follow fic/author/user + updates feed
│   │   ├── notifications.rs            # Notifications
│   │   ├── roadmap.rs                  # Roadmap arena (MaxDiff/Elo)
│   │   ├── auto_tag.rs                 # Auto-tagger
│   │   ├── blind.rs                    # Blind Date
│   │   ├── quests.rs                   # Daily quests
│   │   ├── badges.rs                   # Badges
│   │   ├── trending.rs                 # Trending
│   │   ├── shelves.rs                  # Shelves / reading status
│   │   ├── kindle.rs                   # Send-to-Kindle
│   │   ├── user_export.rs              # Data export
│   │   ├── pow.rs                      # Proof-of-work anti-bot
│   │   ├── health.rs                   # GET /api/health (DB + Redis ping)
│   │   └── api_docs.rs                 # API documentation
│   ├── scrape/
│   │   ├── mod.rs                      # SiteScraper trait + FicMetadata struct
│   │   ├── registry.rs                 # ScraperRegistry
│   │   └── sites/                      # ao3, ffnet, fictionpress, etc.
│   ├── export/
│   │   ├── mod.rs                      # ExportError type
│   │   ├── epub.rs                     # EPUB generation (epub-builder)
│   │   ├── html_bundle.rs             # HTML bundle export
│   │   ├── txt.rs                     # Plain-text export
│   │   ├── md.rs                      # Markdown export
│   │   ├── fallback.rs               # Fallback export (when Calibre unavailable)
│   │   └── convert.rs                  # Calibre sidecar (MOBI/PDF/AZW3)
│   ├── cache/
│   │   ├── mod.rs                      # EType enum, cache helpers
│   │   └── disk.rs                     # Hash-based file cache
│   ├── limiter/
│   │   ├── mod.rs                      # RateLimiter trait
│   │   └── redis_bucket.rs            # Redis token bucket (Lua)
│   ├── recommender/
│   │   ├── mod.rs                     # Module declarations
│   │   ├── engine.rs                   # Co-occurrence query + scoring engine
│   │   ├── worker.rs                   # Background collection worker
│   │   └── routes.rs                   # Recommendation HTTP handlers
│   ├── works/
│   │   └── mod.rs                     # Auto-merge: find_or_create_work()
│   ├── search/
│   │   ├── mod.rs                     # Module declarations
│   │   ├── builder.rs                 # Dynamic SQL query builder (tsquery, main_char_attr)
│   │   └── routes.rs                  # Advanced search handler
│   ├── tags/                           # Tag system (v3)
│   ├── opds/                           # OPDS catalog feeds
│   └── frontend/                       # cache_headers + static serving
├── bin/
│   ├── seed_roadmap.rs                 # cargo run --bin seed-roadmap
│   ├── bot_scorer.rs                   # hourly bot-scorer
│   └── (other binaries)
├── docs/                               # Documentation (mdbook + root md)
│   ├── book.toml                      # mdbook config
│   ├── build.sh                       # Build docs + copy to frontend
│   ├── src/                           # Markdown chapters
│   ├── SPECIFICATION.md               # This document
│   ├── STATUS.md                      # Session status
│   └── DOCS_META.md                   # Docs build facts + improvement candidates
├── frontend/                           # SvelteKit SPA (built output served by Rust)
├── docker/                             # Docker files (historical)
├── templates/                          # Tera templates (internal HTML)
└── tests/                              # Integration tests (DB-gated, #[ignore])
```

### Backend Go: `~/code/go/fichub-cli.bak/`

```
fichub-cli/
├── cmd/fichub-cli/main.go              # Entry point
├── internal/
│   ├── config/                         # Config loading
│   ├── handler/
│   │   ├── handler.go                  # Router + route registration
│   │   ├── fic_handler.go              # Fic CRUD handlers
│   │   ├── user_handler.go             # User auth/profile handlers
│   │   ├── search_handler.go           # Search handlers
│   │   ├── upload_handler.go           # Upload handlers
│   │   ├── mod_handler.go              # Moderation handlers
│   │   └── response.go                 # Response helpers
│   ├── middleware/                      # Auth, CORS, rate limit, logging
│   ├── repository/                     # Database access layer
│   ├── service/                        # Business logic layer
│   ├── storage/                        # File storage
│   ├── typesense/                      # Typesense search client
│   └── jobs/                           # Async job processing (Asynq/Redis)
├── migrations/                         # SQL migrations
├── Dockerfile                          # Container build
├── docker-compose.yml                  # Stack compose
├── go.mod / go.sum
└── sqlc.yaml                           # SQLC config
```

---

## 3. Architecture & Data Flow

```
                         ┌──────────────┐
                         │   Browser    │
                         │ (User)       │
                         └──────┬───────┘
                                │ :8000 (fichub.polarisocial.xyz)
                                ▼
                     ┌─────────────────────┐
                     │  Rust backend       │  ← single binary (fichub.service)
                     │  axum router        │
                     └──────┬──────────────┘
                            │
            ┌───────────────┼──────────────────┐
            ▼               ▼                  ▼
     ┌────────────┐  ┌────────────┐    ┌──────────────┐
     │ PostgreSQL │  │ Redis/Valkey│   │   Ollama     │
     │ :5432      │  │ :6379       │   │ :11434       │
     │ fichub DB  │  │ rate limit, │   │ nomic-embed  │
     │ (mig 001-37)│ │ shadowban   │   │ 768d embeds  │
     └────────────┘  └────────────┘    └──────────────┘
            │
            ▼
     ┌──────────────────────────────────────────────┐
     │ Static SPA (frontend/build) + cached exports │
     │ (ServeDir fallback + disk cache)             │
     └──────────────────────────────────────────────┘
```

**Request flow — SPA + API (single origin):**

```
Browser → :8000/                    → ServeDir serves frontend/build/index.html
Browser → :8000/requests            → ServeDir fallback → index.html → client-side routing
Browser → :8000/api/requests        → axum router → JSON
Browser → :8000/feed.xml            → axum router → application/atom+xml
Browser → :8000/fic/<url_id>        → SPA route → client fetches /api/v1/works/<url_id>
```

**Export flow:**

```
Client → GET /api/v0/epub?q=<url> → scraper registry (can_handle) → scrape
       → upsert fic_info → check fic_blacklist/author_blacklist
       → fetch chapters → generate EPUB → disk cache (hash tree)
       → log to request_log → redirect to cached file / JSON download URL
```

---

## 4. Backend: Rust (Primary API)

The Rust backend (`src/`) is the **single source of truth** — all endpoints, migrations, and business logic. Section 5 (Go) is retained for historical reference only.

### Technology Stack

| Component | Crate | Version |
|-----------|-------|---------|
| HTTP framework | axum | 0.8 |
| Async runtime | tokio | 1.x (full features) |
| Database | sqlx | 0.9 (postgres, tls-rustls-ring) |
| Redis client | redis | 1.4 (aio, tokio-comp) |
| HTTP client | reqwest | 0.12 (rustls-tls) |
| HTML parsing | scraper | 0.27 (CSS selectors) |
| EPUB generation | epub-builder | 0.8 |
| Template engine | tera | 2.1 |
| Serialization | serde / serde_json | 1.x |
| Config | dotenvy | 0.15 |
| Logging | tracing / tracing-subscriber | 0.1 / 0.3 |
| Metrics | axum-prometheus | 0.10 |
| UUID | uuid (v4) | 1.x |
| Hashing | md-5, sha2 | 0.11 |
| Dates | chrono (serde feature) | 0.4 |
| Compression | zip | 0.8 |

### Entry Point (`main.rs`)

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(...)
        .init();
    let config = config::Config::from_env();
    server::run(config).await;
}
```

### Server Setup (`server.rs`)

Builds an `AppState` with:
- `config: Config`
- `db: sqlx::PgPool`
- `redis: redis::aio::MultiplexedConnection`
- `http_client: reqwest::Client`
- `scraper_registry: Arc<ScraperRegistry>`
- `cache_semaphores: CacheSemaphores`
- `rate_limiter: Box<dyn RateLimiter>`
- `recommender_engine: RecommendationEngine`
- `collection_worker: CollectionWorker`

### API Routes (Axum Router)

```
GET  /api/                         → API docs
GET  /api/health                   → Health check (DB + Redis ping)
GET  /api/v0/epub?q=<url>          → Export EPUB (main endpoint)
GET  /api/v0/meta?q=<url>          → Get fic metadata
GET  /api/v0/remote                → Remote info (IP, port, is_automated)

GET  /cache/{etype}/{url_id}/{fname}  → Download cached file with hash
GET  /cache/{etype}/{url_id}          → Download or trigger export

GET  /api/v0/recommendations?url_id=&n=&site_domain=   → Get recommendations
POST /api/v0/recommendations/suggest   → Submit recommendation suggestion
POST /api/v0/recommendations/vote      → Vote on a suggestion
GET  /api/v0/recommendations/votes     → Get votes

User Accounts (auth):
POST /api/auth/register             → Register user (returns JWT)
POST /api/auth/login                → Login (returns JWT)
GET  /api/auth/me                   → Get current user from token

Social (work_id-based):
POST /api/bookmarks                 → Add bookmark (work_id)
GET  /api/bookmarks                 → List user's bookmarks
DELETE /api/bookmarks/{work_id}     → Remove bookmark
POST /api/ratings                   → Rate a work (work_id)
GET  /api/ratings/{work_id}         → Get aggregate ratings
POST /api/comments                  → Post comment (work_id)
GET  /api/comments/{work_id}        → Get comments for a work
DELETE /api/comment/{id}            → Soft delete comment (owner/curator)
PATCH /api/comment/{id}/hide        → Hide comment (curator)
GET  /api/leaderboard/curators      → Top curators by reputation
GET  /api/users/{id}                → User profile
POST /api/reputation/events         → Log reputation event (admin)

Threaded Comments (url_id-based, legacy):
POST /api/v1/works/{url_id}/comments → Post comment or reply
GET  /api/v1/works/{url_id}/comments → Get threaded comments (flat list)

Ratings & Reviews (positive-only, migration 014):
POST /api/ratings                   → Rate a work (1..=5; legacy -1 internal-only)
GET  /api/ratings/{work_id}         → Aggregate (avg, count, likes, review_count, distribution; no dislikes)
POST /api/reviews                   → Upsert in-depth review (rating 1..=5, title, body, constructive)
DELETE /api/reviews/{id}            → Soft delete review (owner/curator)
GET  /api/works/{id}/reviews        → List reviews (constructive only in public lists)

Work Proposals (curator only):
POST /api/work-proposals            → Create merge/split proposal
GET  /api/work-proposals            → List pending proposals
GET  /api/work-proposals/{id}       → Proposal detail with votes
POST /api/work-proposals/{id}/vote  → Cast vote (1=approve, 0=retract, -1=disapprove)

Follows (migration 017):
POST /api/follows                   → Follow a user/work/author
GET  /api/follows                   → List my follows
DELETE /api/follows/{id}            → Unfollow
GET  /api/follows/check/{type}/{id} → Check follow state
GET  /api/v1/feed                   → Updates feed (works I follow)
GET  /api/follows/followers/{user_id} → Followers list
POST /api/v1/works/{url_id}/refresh → Re-scrape + fic_version_bump + notify followers

Fic Requests (migration 018):
POST /api/requests                  → Create request (auth; title/body/seed_work_id)
GET  /api/requests?status=&sort=&page= → List (open|answered|closed; top|new)
GET  /api/requests/{id}             → Detail + answers with net scores + my_vote
DELETE /api/requests/{id}           → Soft delete (owner/curator ≥5)
POST /api/requests/{id}/answers     → Add work answer {work_id?, url?, pitch} (+3 cap/user, UNIQUE dup; URL-ingest via scraper registry + find_or_create_work, e92de9a)
DELETE /api/requests/{id}/answers/{aid} → Soft delete (owner/curator)
POST /api/requests/{id}/answers/{aid}/vote → Vote 1|-1|0 (toggle/retract, no self-vote)
POST /api/requests/{id}/upvote      → Request-level upvote toggle {enabled} (no self-vote; migration 035, 6b86164)
POST /api/requests/{id}/accept/{aid} → Requester accept → status='answered' (notifies answer author)
GET  /api/requests/{id}/candidates  → Engine suggestions from the request's seed work (implemented dd64fe5)

Series & Authors (migration 020):
GET  /api/series/{id}               → Series detail + ordered works + next-in-series
GET  /api/authors/search?q=         → Search author profiles
GET  /api/authors/by-name/{name}    → Author bibliography (canonical name keyed)
GET  /api/authors/{id}              → Author profile (socials, linked accounts)
PUT  /api/authors/{id}              → Update author profile
POST /api/authors/{id}/socials      → Add social
DELETE /api/authors/{id}/socials/{sid} → Remove social
GET  /api/curator/authors/*         → Curator author merge workflows

RSS/Atom feeds:
GET  /feed.xml                      → New arrivals feed (application/atom+xml)
GET  /feed/follows.xml              → Follows feed (requires ?token=)
GET  /feed/works/{url_id}           → Per-fic feed (bare param; handler strips .xml)

Reading Lists & Shelves (migrations 019, 004):
GET/POST /api/lists                 → List/create reading lists
GET/PUT/DELETE /api/lists/{id}      → List detail / update / delete
POST /api/lists/{id}/items          → Add item (positioned, blurb)
DELETE /api/lists/{id}/items/{iid}  → Remove item
GET  /api/shelves?token=X           → List shelves (OPDS-style)
GET  /api/shelf/{id}?token=X        → Shelf contents

Advanced Search:
GET  /api/v0/search?q=&include_tags=&exclude_tags=&include_any_tags=
     &min_words=&max_words=&min_chapters=&max_chapters=
     &complete=&source=&date_from=&date_to=&sort=
     &primary_tag=&min_comments=&min_kudos=&no_warnings=&tag_ids=
     (main_char_attr applies freeform attributes to the MAIN character;
      boolean AND/OR via tsquery; typo tolerance)

Roadmap Consensus (migrations 011, 016):
GET  /api/roadmap/arena             → MaxDiff/Elo arena state
POST /api/roadmap/vote              → Cast a comparison
GET  /api/roadmap/features?status=  → Feature clusters (open/shipped/...)
POST /api/roadmap/features          → Suggest a feature
GET  /api/roadmap/leaderboard       → Elo rankings
GET  /admin/consensus               → Admin consensus dashboard
GET  /tropes                        → Trope browser

Reader (in-browser):
GET  /api/reader/{url_id}           → Full HTML bundle for a work
GET  /api/reader/{url_id}/meta      → Reader metadata
GET  /api/reader/{url_id}/sequel    → Next-in-series detection
GET  /api/reader/{url_id}/related   → Related works
GET  /api/v1/works/{url_id}/also-bookmarked → Co-occurrence rec anchors

Anti-bot / moderation:
POST /api/admin/bots                → Bot management
GET  /api/admin/bots                → Bot list
GET  /api/admin/realtime            → Realtime search analytics
GET  /api/admin/search-analytics    → Search analytics
POST /api/admin/bots/{id}/shadowban → Redis shadowban
GET  /api/v0/tags                   → List tags (submitted via POST /api/v0/tags/submit)
POST /api/v0/curator/*              → Tag curation workflows

Tags & Curator:
POST /api/v0/tags/submit            → Submit tag
POST /api/v0/tags/vote              → Vote on tag
POST /api/v0/tags/flag              → Flag tag
GET  /api/v0/tags                   → List tags
POST /api/v0/curator/alias          → Create alias (curator)
POST /api/v0/curator/merge          → Merge tags (curator)
DELETE /api/v0/curator/tags/{id}    → Delete tag (curator)
GET  /api/v0/curator/flags          → List flags (curator)
POST /api/v0/curator/flags/{id}/resolve → Resolve flag (curator)

OPDS Catalog:
GET  /opds                          → Root catalog
GET  /opds/new                      → Recent feed
GET  /opds/popular                  → Popular feed
GET  /opds/tags                     → Tag types
GET  /opds/tags/{type_id}           → Tags by type
GET  /opds/tags/{type_id}/{name}    → Fics by tag
GET  /opds/authors                  → Author list
GET  /opds/search                   → Search feed
GET  /opds/shelves                  → Shelf list
GET  /opds/shelf/{shelf_id}         → Shelf contents

Legacy redirects:
GET  /legacy/epub_export  → Redirect to /
GET  /fic/{url_id}        → Redirect to /
GET  /changes             → Redirect to /
GET  /popular/            → Redirect to /

Fallback: ServeDir (SvelteKit static build)
```

### Scraper System

**Trait**: `SiteScraper` in `scrapers/` (the `fanfic-scrapers` crate, AGPL-3.0). FicHub's `src/scrape/` is a thin compat layer re-exporting the crate + FicHub's i16 tag scores / sha256 url_id.

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    fn requires_login(&self) -> bool { false }
    async fn login(&self, client: &reqwest::Client, creds: &SiteCredentials)
        -> Result<(), ScrapeError>;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

**Registered scrapers** (in `registry.rs` — native scrapers preferred;
`find_specific_or_fallback` skips any catch-all registered at index 0):
1. Native adapters from the `fanfic-scrapers` crate: **full FanFicFare
   adapter parity — all 107 real FFF sites** (crate v0.10.0, AGPL-3.0).
   Families: AO3 + OTW siblings (5), eFiction (19), eFiction-variant
   `viewstory.php?sid=` (26), XenForo1+2 (8), StoriesOnline (3),
   WordPress-novel, plus bespoke adapters (FFN/FictionPress, RoyalRoad,
   ScribbleHub, FimFiction, Literotica, Wattpad, DeviantArt, FicBook,
   Quotev, AsianFanFics, Syosetu, Kakuyomu, MediaMiner, ChiReads,
   PMDFanFiction, SpiritFanfiction, Fanfictions.fr, Twisting the Hellmouth,
   HPFanficArchive, adultfanfiction.org, LCFanFic, FireflyFans, FicWad,
   FictionMania, FanFiktion.de, TouchFluffyTail, Fanfics.me, FanFicAuthors,
   FictionHunt, InkBunny, SoFurry, Dokuga, PhoenixSong, StoriesOfArda,
   Fictionalley, NovelAll, MassEffect2.in, ReadonlyMind, UtopiaStories,
   ASexStories, AnEroticStory, MCStories, HentaiFoundry, BDSMLibrary).
   Login-requiring: fanfics.me, fictionhunt, inkbunny, sofurry, dokuga
   (`SiteScraper::login` + `SiteCredentials`, cookie store).
   Adult-gated (need `is_adult`): readonlymind, utopiastories, asexstories,
   aneroticstory, mcstories, hentaifoundry, bdsmlibrary.
   Coverage/update workflow: `scrapers/FFF_PARITY.md` (baseline v4.60.0).

**`FicMetadata` struct** (scrape output):
```rust
pub struct FicMetadata {
    pub url_id: String,         // deterministic SHA-256 hash
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,         // unix millis
    pub updated: i64,           // unix millis
    pub status: String,         // ongoing, complete, hiatus, cancelled
    pub source: String,         // original URL
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

**`url_id` generation**: SHA-256(`{source_id}:{story_id}`) → first 12 hex chars.

### Export System

**EPUB** (`export/epub.rs`):
- Uses `epub-builder` crate with `ZipLibrary`
- Creates a UUID-named temp directory
- Generates introduction page with metadata table
- Writes each chapter as an XHTML file
- Computes MD5 hash of the output
- Returns `(path_to_epub, md5_hex)`

**HTML bundle** (`export/html_bundle.rs`):
- Wraps chapters in a single HTML file with inline CSS

**Plain text** (`export/txt.rs`):
- Exports fic as plain text with chapter headings

**Markdown** (`export/md.rs`):
- Exports fic as Markdown with chapter headings

**Fallback** (`export/fallback.rs`):
- Used when Calibre sidecar is unavailable; generates EPUB directly

**Calibre sidecar** (`export/convert.rs`):
- Calls `ebook-convert` via `tokio::process::Command`
- Converts EPUB → MOBI, PDF, or AZW3
- Runs in a separate container identified by `calibre_container` config

### Caching System

**Disk cache** (`cache/disk.rs`):
- Hash-based directory tree (2-level: `{cache_dir}/{etype}/{url_id[:2]}/{url_id[2:]}/`)
- Stores generated EPUBs and other exports
- Versioned via `export_log` table for cache invalidation
- `CacheSemaphores`: per-url_id `tokio::sync::Mutex` to prevent duplicate concurrent exports

**Cache types** (`EType` enum): `Epub`, `Html`, `Mobi`, `Pdf`, `Txt`, `Azw3`, `Md`

### Rate Limiter

**Redis token bucket** (`limiter/redis_bucket.rs`):
- Lua script executed via `EVAL` on Redis
- Token bucket per IP: configurable `capacity` and `flow` (tokens/sec)
- `dynamic_rate_limit`: when true, adjusts limits based on upstream load
- Datacenter IP detection: loads IP ranges from configured sources
- Global and per-IP rate limits
- Returns wait time when rate limited

### Recommender System

**Engine** (`recommender/engine.rs`):
- Collaborative filtering based on bookmark/favourite co-occurrence
- Queries `fic_bookmark_cooccur` table for pairwise work co-occurrence
- Tag-based fallback when co-occurrence data is sparse
- Voting boost: community suggestions weighted by upvotes/downvotes
- Configurable parameters: gamma (voting boost), max recommendations, min favouriters
- Precomputed recommendations stored in `precomputed_recommendations` table
- Cache TTL for precomputed results

**Worker** (`recommender/worker.rs`):
- Background collection worker that scrapes user favourites from supported sites
- `SiteFetcher` trait + `PerSiteRateLimiter` for rate-limited scraping
- Supports AO3 and FF.net for favourite collection
- Atomic CAS-based rate limiting per site
- Collects favouriters → collects their favourites → computes co-occurrences
- Periodic precomputation on configurable interval

**Routes** (`recommender/routes.rs`):
- `GET /api/v0/recommendations?url_id=&n=20&site_domain=` — get recommendations
- `POST /api/v0/recommendations/suggest` — submit a recommendation suggestion
- `POST /api/v0/recommendations/vote` — upvote/downvote a suggestion
- `GET /api/v0/recommendations/votes` — get vote status

### Error Handling

**`AppError` enum** (in `error.rs`):
```rust
pub enum AppError {
    BadRequest(i32, String),
    RateLimited(u64),
    NotFound(String),
    Internal(String),
    ScrapeError(String),
    ExportError(String),
    Database(String),
    CacheError(String),
}
```

Each variant implements `IntoResponse` returning appropriate HTTP status codes and JSON error bodies. Common error codes:
- `-1`: Internal / database error
- `-5`: Bad request / not found / export failed
- `-6`: Scrape error (bad gateway)
- `-10`: Blocked (automated request)
- `-429`: Rate limited

Automatic `From` conversions for: `std::io::Error`, `sqlx::Error`, `redis::RedisError`, `reqwest::Error`, `serde_json::Error`.

---

## 5. Backend B: Go (v1 API)

> **⚠️ HISTORICAL — superseded by the Rust backend (section 4).** The Go codebase lives at `~/code/go/fichub-cli.bak/` and is retained for reference only. Not deployed, not part of the current stack.

### Technology Stack

| Component | Library | Notes |
|-----------|---------|-------|
| HTTP router | chi (go-chi/chi/v5) | Compositional middleware |
| Database | pgx (jackc/pgx/v5) | PostgreSQL driver + pool |
| Redis | go-redis/redis/v9 | Caching, rate limiting, job queue |
| Search | Typesense | Full-text search engine |
| Auth | JWT (custom middleware) | Access + refresh tokens |
| Jobs | asynq (hibiken/asynq) | Redis-based async job queue |
| Logging | zerolog (rs/zerolog) | Structured JSON logging |
| Config | Custom | YAML + env overlay |
| Validation | (manual struct validation) | |

### Architecture (Go Backend)

The Go backend follows clean architecture layers:
```
Handler (HTTP) → Service (Business Logic) → Repository (Database)
                                        ↕
                              Typesense Client
                                        ↕
                              Storage (Files)
```

### API Routes (v1)

```
POST   /api/v1/auth/register       → Register user
POST   /api/v1/auth/login          → Login (returns JWT)
POST   /api/v1/auth/refresh        → Refresh token
POST   /api/v1/auth/logout         → Logout (revoke token)

GET    /api/v1/users/{id}          → Get user profile
PUT    /api/v1/users/{id}/profile  → Update profile (auth required)
GET    /api/v1/users/{id}/bookmarks      → List bookmarks
POST   /api/v1/users/{id}/bookmarks      → Add bookmark
GET    /api/v1/users/{id}/history        → Get reading history

DELETE /api/v1/bookmarks/{id}      → Delete bookmark

GET    /api/v1/fics                → Search/list fics
GET    /api/v1/fics/{id}           → Get fic detail
GET    /api/v1/fics/{id}/download  → Download fic (format param)
GET    /api/v1/fics/{id}/versions  → List versions
GET    /api/v1/fics/{id}/tags      → List tags for fic
POST   /api/v1/fics                → Create fic (auth)
POST   /api/v1/fics/{id}/fork     → Fork fic (auth)
POST   /api/v1/fics/{id}/tags     → Suggest tag (auth)
GET    /api/v1/fics/{id}/recommendations → Get recommendations
POST   /api/v1/fics/{id}/chapters → Create chapter (auth)
PUT    /api/v1/fics/{id}/chapters/{chapter} → Update chapter (auth)
DELETE /api/v1/fics/{id}           → Delete fic (mod only)

POST   /api/v1/upload              → Upload file (auth)

GET    /api/modlog                 → List moderation log (any logged-in user; ?action= filter)
GET    /api/admin/analytics        → Usage analytics dashboard (role ≥ 10)
GET    /api/admin/search-analytics → Search analytics + search→export conversion (role ≥ 10)
GET    /api/admin/translations     → Translation review queue (role ≥ 10; ?status=draft|approved|rejected)
POST   /api/admin/translations/{id}/approve | /reject | /edit → Review ML translations (role ≥ 10)
GET    /api/admin/works/{id}/metadata → Fetch work metadata (role ≥ 10)
PUT    /api/admin/works/{id}/metadata → Correct canonical metadata: {title?, author?, status?, description?} (role ≥ 10; syncs default source)

GET    /api/v1/reports             → List reports
POST   /api/v1/reports             → Create report (auth)

GET    /api/v1/search              → Search fics
GET    /api/v1/search/autocomplete → Autocomplete suggestions
POST   /api/v1/searches            → Save search (auth)
```

### Data Types (from Go backend schema)

Key database models include:
- `users` — id, username, password_hash, bio, created_at, updated_at, role
- `fics` — id, title, author_id, fandom, rating, word_count, status, language, summary, cover_url, created_at, updated_at
- `chapters` — id, fic_id, title, content (HTML), position, word_count
- `bookmarks` — user_id, fic_id, created_at
- `tags` — id, name, category, canonical
- `fic_tags` — fic_id, tag_id
- `mod_actions` — id, action, admin_id, target_type, target_id, reason, created_at
- `reports` — id, reporter_id, target_type, target_id, reason, status, created_at
- `sessions` — id, user_id, token_hash, expires_at, created_at

### Middleware Stack

1. `RequestID` — unique request ID per request
2. `RealIP` — trust X-Forwarded-For / X-Real-IP
3. `Logger` — zerolog structured request logging
4. `Recovery` — panic recovery
5. `Compress` — gzip response compression (level 5)
6. `CORS` — configurable CORS origins
7. `RateLimiter` — Redis-based rate limiting
8. `Auth` — JWT verification (Bearer token)
9. `ModeratorOnly` — role-based access control

### Search (Typesense)

- Full-text search via Typesense
- Collections: fics, users, tags
- Autocomplete support
- Faceted search by fandom, rating, status, language
- Fallback to degraded mode when Typesense is unavailable
- Async indexing via asynq job queue

### Async Jobs (Asynq)

- `IndexFic` — index/update fic in Typesense
- `ProcessUpload` — process uploaded file (parse, extract metadata)
- `GenerateThumbnail` — generate cover thumbnails
- `CleanupExpiredSessions` — periodic session cleanup
- `RecalculateTags` — recalculate tag counts
- `ExportFic` — async EPUB/PDF generation

---

## 6. Frontend: SvelteKit SPA

### Technology Stack

| Component | Library | Version |
|-----------|---------|---------|
| Framework | SvelteKit | ^2.69.2 |
| Svelte | svelte | ^5.56.4 |
| Build tool | vite | ^8.1.4 |
| CSS | Tailwind CSS | ^4.3.2 |
| Adapter | @sveltejs/adapter-static | ^3.0.10 |
| Icons | @xylightdev/svelte-hero-icons | ^2.2.10 |
| UI lib | mono-svelte (local alias) | — |
| Test runner | Playwright | — |
| Linter | eslint | ^9.39.2 |
| Formatter | prettier | ^3.9.5 |

### Build Configuration

**`package.json` name**: `photon-lemmy` (inherited from Photon fork, not updated)

**`svelte.config.js`**:
```js
adapter: staticAdapter({ fallback: 'index.html', precompress: false }),
alias: {
  'mono-svelte': 'src/lib/ui/shared',
  'svelte-hero-icons': 'node_modules/@xylightdev/svelte-hero-icons',
  $comp: 'src/lib/components',
},
csp: { directives: { 'script-src': ['self'] } }
```

**`vite.config.ts`**:
```ts
plugins: [sveltekit(), tailwindcss()],
build: { sourcemap: true },
server: {
  port: 8006,
  proxy: { '/api': { target: 'http://127.0.0.1:8005', changeOrigin: true } },
}
```
Note: the dev server proxies `/api` to the Rust backend (the SPA and API are served from one origin on :8000 in production; in dev, `vite` proxies `/api` to the backend port).

### Route Structure

| Route | File | Type | Purpose |
|-------|------|------|---------|
| `/` | `+page.svelte` | Static | Home page with welcome message, links to Search, Popular, Recent, Tags |
| `/browse` | `browse/+page.svelte` | Client-fetched | Browse fandoms by tag category |
| `/fics/[id]` | `fics/[id]/+page.svelte` | SSR/CSR | Fic detail page |
| `/fics/[id]` | `fics/[id]/+page.ts` | Loader | Fetches fic data from API |
| `/login` | `login/+page.svelte` | Client | Login form with auth store |
| `/popular` | `popular/+page.svelte` | SSR/CSR | List popular fics |
| `/popular` | `popular/+page.ts` | Loader | Fetches popular fics from API |
| `/recent` | `recent/+page.svelte` | SSR/CSR | List recent fics |
| `/recent` | `recent/+page.ts` | Loader | Fetches recent fics from API |
| `/search` | `search/+page.svelte` | SSR/CSR | Search with filters (query, fandom, rating, page) |
| `/search` | `search/+page.ts` | Loader | Fetches search results |
| `/tags` | `tags/+page.svelte` | Client-fetched | Browse all tags by category |
| `/u/[id]` | `u/[id]/+page.svelte` | SSR/CSR | User profile + bookmarks |
| `/u/[id]` | `u/[id]/+page.ts` | Loader | Fetches user + bookmarks from API |
| `/[...slug]` | `[...slug]/+page.ts` | Catch-all | 404 / catch-all handler |
| `/error` | `error/+page.ts` | Static | Error page loader |

### API Client (`fichub-client.svelte.ts`)

**Class**: `FichubClient`

**Features**:
- Configurable base URL (defaults to empty string = same origin)
- JWT token management via localStorage (`fichub_token` key)
- Automatic `Authorization: Bearer <token>` header injection
- `Content-Type: application/json` for POST/PUT/PATCH
- Redirect handling (follows 3xx Location headers)
- Error wrapping: `FichubError` class with status + body
- Singleton export: `export const fichub = new FichubClient()`

**Methods**:
```typescript
listFics(params?)         → GET /api/v1/fics
getFic(id)                → GET /api/v1/fics/:id
getDownloadUrl(id, fmt?)  → GET /api/v1/fics/:id/download
search(params)            → GET /api/v1/search
login(username, pass)     → POST /api/v1/auth/login
register(username, pass)  → POST /api/v1/users
logout()                  → clear token
getUser(id)               → GET /api/v1/users/:id
getBookmarks(userId)      → GET /api/v1/users/:id/bookmarks
addBookmark(userId, ficId) → POST /api/v1/users/:id/bookmarks
suggestTags(ficId, tags)  → POST /api/v1/fics/:id/tags
getRecommendations(ficId) → GET /api/v1/fics/:id/recommendations
listTags(params?)         → GET /api/v1/tags
getModlog(page?)          → GET /api/v1/modlog
voteModlog(actionId, vote)→ POST /api/v1/modlog/:id/vote
```

### Type Definitions (`fichub-types.ts`)

**Core types**:
```typescript
type Rating = 'general' | 'teen' | 'mature' | 'explicit'
type FicStatus = 'completed' | 'in_progress'
type TagCategory = 'fandom' | 'character' | 'relationship' | 'genre' | 'warning'
type ModlogVote = 'fair' | 'unfair'
```

**Data interfaces**:
- `Author` — id, name
- `Chapter` — id, title, content (HTML), position, word_count
- `Tag` — id, name, category, canonical, count
- `FicSummary` — id, title, author, fandom, rating, word_count, status, language, summary, tags, chapter_count, created_at, updated_at, cover_url?
- `FicDetail` — id, title, author (Author), fandom, rating, word_count, status, language, summary, tags, chapters (Chapter[]), chapter_count, created_at, updated_at, cover_url?
- `UserProfile` — id, username, bio?, created_at
- `Bookmark` — fic_id, fic_title, added_at
- `ModAction` — id, action, admin (AdminInfo), target, reason, created_at

**Response wrappers**: `PaginatedFics`, `FicDetailResponse`, `SearchResponse`, `UserProfileResponse`, `BookmarkListResponse`, `SuccessResponse`, `RecommendationsResponse`, `TagListResponse`, `ModlogResponse`

**Request types**: `LoginRequest`, `RegisterRequest`, `AddBookmarkRequest`, `SuggestTagsRequest`, `ModlogVoteRequest`

**Query params**: `ListFicsParams`, `SearchQueryParams`, `ListTagsParams`

### Auth Store (`auth.svelte.ts`)

**Class**: `Auth`

**State** (via Svelte 5 `$state`):
```typescript
#state = $state<AuthState>({
  user: UserProfile | null,
  loading: boolean,
  error: string | null,
})
```

**Features**:
- Reactive getters: `user`, `loading`, `error`, `isLoggedIn`
- localStorage persistence key: `fichub_auth`
- Automatic hydration on construction
- Token check: if user is persisted but token was cleared externally → auto-logout
- Methods: `login()`, `register()`, `logout()`, `clear()`, `clearError()`
- Singleton: `export const auth = new Auth()`

### Settings Store (`settings.svelte.ts`)

- Schema-driven settings with defaults
- localStorage persistence
- Supports: `font` (inter/system/browser/serifs), `language`, `useRtl`, `expandSidebar`
- Deep merge utility for loading partial saved settings
- `$effect.root` for auto-save on any settings change

### Theme System

- `theme.svelte.ts` — reactive theme state with color scheme (light/dark/system)
- `presets.ts` — theme colour presets
- Tailwind dark mode via `class` strategy
- Applied via `document.documentElement.classList` manipulation in layout

### UI Component Library ("mono-svelte")

**Alias**: `mono-svelte` → `src/lib/ui/shared`

**Form components** (`src/lib/ui/form/`):
- `Duration`, `FreeTextInput`, `ImageAttachForm`, `ImageInputModal`, `ImageInputUpload`, `ImagePreviewInput`, `Link`, `ObjectAutocomplete`, `Switch`, `TabButton`

**Generic components** (`src/lib/ui/generic/`):
- `Avatar`, `Blobs`, `Entity`, `EntityHeader`, `ExpandableImage`, `Fixate`, `ItemList`, `Logo`, `Skeleton`

**Info components** (`src/lib/ui/info/`):
- `ErrorContainer`, `LabelStat`, `Placeholder`, `ProgressBar`

**Layout components** (`src/lib/ui/layout/`):
- `CommonItem`, `CommonList`, `EndPlaceholder`, `Header`, `InvertedCorner`, `Pageination`, `SearchBar`, `Shell`, `TabbedLayoutShell`, `Tabs`
- Re-export: `src/lib/ui/layout/index.ts`

**Navbar** (`frontend/src/lib/components/`):
- `AuthBar.svelte` — auth status + user menu
- `NavDropdown.svelte` — main navigation
- `NotificationBell.svelte` — unread notifications
- `LocaleSelector.svelte` — language switcher
- `CommandPalette.svelte` — command palette (Ctrl+K / Cmd+K): page actions +
  docs-map.json search (opens the help modal for doc entries)
- `DocLink.svelte` — inline "?" help link → opens the exact docs section
- `HelpModal.svelte` — in-app docs section renderer + "Ask the docs" box
  (`/api/docs/ask` retrieval with citations)

**Shared** (`src/lib/ui/shared/`):
- `badge/Badge.svelte`
- `button/Button.svelte`, `ButtonGroup.svelte`
- `disclosure/Disclosure.svelte`, `Expandable.svelte`
- `forms/FileInput.svelte`, `Label.svelte`, `Switch.svelte`, `TextArea.svelte`, `TextInput.svelte`, `helper.ts`, `select/Option.svelte`, `select/Select.svelte`
- `loader/Spinner.svelte`, `TextLoader.svelte`
- `materials/Material.svelte`
- `modal/Modal.svelte`, `ModalContainer.svelte`, `modal.ts`
- `note/Note.svelte`
- `popover/Menu.svelte`, `MenuButton.svelte`, `MenuDivider.svelte`, `Popover.svelte`, `Portal.svelte`
- `search/Search.svelte`
- `toast/Toast.svelte`, `ToastContainer.svelte`, `toasts.ts`
- `util/RelativeDate.svelte`, `time.ts`

**Sidebar** (`src/lib/ui/sidebar/`):
- `Sidebar.svelte`, `SidebarButton.svelte`

### Root Layout (`+layout.svelte`)

```
<Shell>
  <ToastContainer />
  <ModalContainer />
  
  {#snippet sidebar}  → <Sidebar />
  {#snippet main}     → <main>{@render children}</main>
  {#snippet navbar}   → <Navbar />
  {#snippet suffix}   → "FicHub" footer
</Shell>
```

Features:
- nProgress loading bar (200ms delay)
- Font class switching via settings
- Content-Security-Policy via svelte.config.js

### Page Details

**Home** (`+page.svelte`):
- `<svelte:head>` sets title: "FicHub — Fanfiction Hub"
- Welcome message + description
- CTA buttons: Search Fics (primary), Browse Fandoms (secondary)
- Grid of 3 cards: Popular (Fire icon), Recent (Clock icon), Tags (Tag icon)

**Browse** (`browse/+page.svelte`):
- Loads tags filtered by `fandom` category via `fichub.listTags({ category: 'fandom' })`
- Displays tag buttons that navigate to search with fandom filter
- `GlobeAlt` icon

**Fic Detail** (`fics/[id]/+page.svelte`):
- Loader fetches `fichub.getFic(id)`
- Displays full fic info: title, author, fandom, rating badge, word count, chapters
- Download button (format selector: epub, pdf, mobi, txt, html)
- Chapter list with expandable content
- Recommendations section

**Login** (`login/+page.svelte`):
- Username + password form
- Uses `auth.login()` from auth store
- Redirects to `/` on success
- Error display
- Loading spinner

**Popular** (`popular/+page.svelte`):
- Loader: `fichub.listFics({ sort: 'popular', limit: 50 })`
- Grid of fic cards with: title, author, fandom, rating badge, word count, summary, time ago
- Rating badge colour coding: general→green, teen→yellow, mature→orange, explicit→red
- Word count formatting (K/M suffixes)

**Recent** (`recent/+page.svelte`):
- Loader: `fichub.listFics({ sort: 'recent', limit: 50 })`
- Same card layout as popular
- Time ago helper: just now, Xm ago, Xh ago, Xd ago, then full date

**Search** (`search/+page.svelte`):
- Loader reads query params from URL: `q`, `fandom`, `rating`, `page`
- Local state for search form inputs bound to `$state()`
- `$derived` results from `data.results`
- Filter controls: fandom dropdown, rating selector, pagination
- Results displayed as fic cards

**Tags** (`tags/+page.svelte`):
- Loads all tags via `fichub.listTags()`
- Groups tags by category (fandom, character, relationship, genre, warning)
- Category labels for display
- Clicking a tag navigates to search with that tag

**User Profile** (`u/[id]/+page.svelte`):
- Loader fetches `fichub.getUser(id)` + `fichub.getBookmarks(id)`
- Profile header: avatar (or initial), username, join date, bookmark count
- Bio section
- Bookmarks list with fic links
- Empty state: "No bookmarks yet."
- Not-found state: "User not found"

### Known Frontend Issues

1. **Photon branding remnants**: `package.json` still says `"photon-lemmy"`, some references to Photon in comments.
2. **`mono-svelte` alias**: The shared UI library is aliased as `mono-svelte` which is not a real npm package — it's a local path alias. This works with the SvelteKit alias config but may confuse tooling.
3. **Svelte 5 warnings**: The build produces several `state_referenced_locally` warnings (capturing initial values in `$state()` instead of `$derived`). These are non-blocking but indicate suboptimal reactivity.
4. **No loading/error states on some pages**: Some pages use `#await` blocks but not all have proper loading skeletons.
5. **`base.ts` and `client.svelte.ts`**: These are legacy Photon files that reference Lemmy/PieFed APIs. They're unused by the FicHub pages but still in the tree — candidates for deletion.

---

## 7. Database Schema

### PostgreSQL — Core Tables (Migration 001)

**`works`** — canonical work (unified works model, Migration 003):
```sql
id              SERIAL PRIMARY KEY
canonical_title TEXT NOT NULL
canonical_author TEXT NOT NULL
description     TEXT DEFAULT ''
default_source_id VARCHAR(128) REFERENCES fic_info(id)
created_at      TIMESTAMPTZ DEFAULT NOW()
updated_at      TIMESTAMPTZ DEFAULT NOW()
```

**`auto_merge_log`** — tracks automatic merge operations:
```sql
id              SERIAL PRIMARY KEY
source_url_id   VARCHAR(128) NOT NULL REFERENCES fic_info(id)
target_work_id  INT4 NOT NULL REFERENCES works(id)
confidence      REAL NOT NULL
merged_at       TIMESTAMPTZ DEFAULT NOW()
```

**`request_source`** — tracks sources of requests (automated vs human):
```sql
id          BIGSERIAL PRIMARY KEY
created     TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
is_automated BOOLEAN DEFAULT FALSE
route       TEXT
description TEXT
UNIQUE(is_automated, route, description)
```

**`request_log`** — logs every export request:
```sql
id              BIGSERIAL PRIMARY KEY
created         TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
source_id       BIGINT REFERENCES request_source(id)
etype           TEXT NOT NULL
query           TEXT NOT NULL
info_request_ms INT4 NOT NULL
url_id          TEXT
fic_info        TEXT (JSON string)
export_ms       INT4
export_file_name TEXT
export_file_hash TEXT
url             TEXT
INDEX: (url_id, etype, created)
INDEX: (created) WHERE export_file_name IS NOT NULL AND etype = 'epub'
```

**`fic_info`** — cached fanfiction metadata:
```sql
id                VARCHAR(128) PRIMARY KEY  (SHA-256 hash, 12 hex chars)
work_id           INT4 REFERENCES works(id)  (unified works FK)
created           TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
updated           TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
title             TEXT NOT NULL
author            TEXT NOT NULL
author_url        TEXT
author_local_id   TEXT
chapters          INT4 NOT NULL
words             INT8 NOT NULL
description       TEXT NOT NULL
fic_created       TIMESTAMPTZ NOT NULL
fic_updated       TIMESTAMPTZ NOT NULL
status            TEXT NOT NULL               -- ongoing, complete, hiatus, cancelled
source            TEXT NOT NULL               -- original URL
extra_meta        TEXT
raw_extended_meta TEXT
source_id         INT8                        -- scraper source ID
author_id         INT8
content_hash      VARCHAR(256)
```

**`export_log`** — cache tracking for generated exports:
```sql
url_id     VARCHAR(128) REFERENCES fic_info(id)
version    INT NOT NULL
etype      TEXT NOT NULL
input_hash TEXT NOT NULL
export_hash TEXT NOT NULL
created    TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
UNIQUE(url_id, version, etype, input_hash)
```

**`fic_blacklist`** — blacklisted fics:
```sql
url_id  VARCHAR(128) REFERENCES fic_info(id)
created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
reason  INT NOT NULL DEFAULT 1
UNIQUE(url_id, reason)
```

**`author_blacklist`** — blacklisted authors:
```sql
source_id INT8 NOT NULL
author_id INT8 NOT NULL
created   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
updated   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
reason    INT NOT NULL DEFAULT 1
UNIQUE(source_id, author_id, reason)
```

**`fic_version_bump`** — cache invalidation:
```sql
id    VARCHAR(128) PRIMARY KEY
value INT
```

### PostgreSQL — Recommender Tables (Migration 002)

**`fic_works`** — site-specific work metadata:
```sql
url_id                VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id)
site_domain           VARCHAR(255) NOT NULL
site_work_id          VARCHAR(255) NOT NULL
favouriter_count      INT4 NOT NULL DEFAULT 0
first_favourite_scraped  TIMESTAMPTZ
last_favourite_scraped   TIMESTAMPTZ
last_cooccur_update      TIMESTAMPTZ
UNIQUE(site_domain, site_work_id)
INDEX: (favouriter_count), (site_domain)
```

**`fic_bookmarks`** — user → work bookmark mapping:
```sql
user_hash  VARCHAR(64) NOT NULL             -- SHA-256 of user profile URL
url_id     VARCHAR(128) NOT NULL REFERENCES fic_info(id)
site_domain VARCHAR(255) NOT NULL
first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
PRIMARY KEY (user_hash, url_id)
INDEX: (user_hash), (url_id), (site_domain)
```

**`fic_bookmark_cooccur`** — pairwise co-occurrence counts:
```sql
work_a     VARCHAR(128) NOT NULL REFERENCES fic_info(id)
work_b     VARCHAR(128) NOT NULL REFERENCES fic_info(id)
site_domain VARCHAR(255) NOT NULL
cooccur_count INT4 NOT NULL DEFAULT 1
last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
PRIMARY KEY (work_a, work_b)
CHECK (work_a < work_b)
INDEX: (work_a), (work_b), (site_domain)
```

**`recommendation_suggestions`** — community-submitted recommendations:
```sql
id              BIGSERIAL PRIMARY KEY
url_id          VARCHAR(128) NOT NULL REFERENCES fic_info(id)
suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id)
submitted_by_ip INET NOT NULL
comment         TEXT
created         TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
UNIQUE(url_id, suggested_url_id, submitted_by_ip)
```

**`recommendation_votes`** — votes on suggestions:
```sql
suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id)
voter_ip     INET NOT NULL
vote         SMALLINT NOT NULL CHECK (vote IN (-1, 1))
created      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
PRIMARY KEY (suggestion_id, voter_ip)
```

**`precomputed_recommendations`** — materialised recommendation cache:
```sql
url_id            VARCHAR(128) NOT NULL REFERENCES fic_info(id)
recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id)
score             REAL NOT NULL
rank              SMALLINT NOT NULL
computed_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
PRIMARY KEY (url_id, recommended_url_id)
INDEX: (url_id, rank)
```

### Go Backend — Expected Tables

### PostgreSQL — v2 Tables (Migrations 005-007, updated by 003_unified_works)

**`users`** — User accounts:
- id, username, password_hash, email, role (0=reader, 5=curator, 10=admin)
- reputation (INT4, default 0)
- curator_since (TIMESTAMPTZ, nullable — when role was promoted to curator)
- curator_status (TEXT, nullable — 'active', 'inactive', 'suspended')

**`bookmarks`** — User bookmarks:
- user_id INT4 NOT NULL REFERENCES users(id)
- work_id INT4 NOT NULL REFERENCES works(id)
- notes TEXT DEFAULT ''
- is_private BOOLEAN DEFAULT FALSE
- created_at TIMESTAMPTZ DEFAULT NOW()
- PRIMARY KEY (user_id, work_id)

**`work_ratings`** — 5-star ratings (migration 014 rework; positive-only public):
- user_id INT4 NOT NULL REFERENCES users(id)
- work_id INT4 NOT NULL REFERENCES works(id)
- rating SMALLINT NOT NULL CHECK (rating IN (-1, 1, 2, 3, 4, 5))  -- 1..=5 public; -1 legacy internal rec signal
- created_at TIMESTAMPTZ DEFAULT NOW()
- PRIMARY KEY (user_id, work_id)

**`reviews`** — In-depth reviews (migration 014):
- id SERIAL PRIMARY KEY
- user_id INT4 NOT NULL REFERENCES users(id)
- work_id INT4 NOT NULL REFERENCES works(id)
- url_id VARCHAR(128) REFERENCES fic_info(id)
- rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5)
- title TEXT DEFAULT ''
- body TEXT NOT NULL
- constructive BOOLEAN NOT NULL DEFAULT TRUE
- created_at / updated_at TIMESTAMPTZ
- UNIQUE (user_id, work_id)

**`comments`** — Threaded comment system:
- id SERIAL PRIMARY KEY
- work_id INT4 REFERENCES works(id)  (unified works FK)
- url_id VARCHAR(128) REFERENCES fic_info(id)  (source-specific, legacy compat)
- user_id INT4 NOT NULL REFERENCES users(id)
- parent_id INT4 REFERENCES comments(id)
- body TEXT NOT NULL
- constructive BOOLEAN NOT NULL DEFAULT TRUE  (migration 014; public lists filter constructive=TRUE)
- created_at TIMESTAMPTZ DEFAULT NOW()
- updated_at TIMESTAMPTZ DEFAULT NOW()
- deleted_at TIMESTAMPTZ  (soft delete)
- is_hidden BOOLEAN DEFAULT FALSE  (curator hide)
- Indexes: top-level newest, parent oldest, work_id, url_id; partial index on constructive

**`work_proposals`** — Curator merge/split proposals:
- id SERIAL PRIMARY KEY
- proposer_id INT4 NOT NULL REFERENCES users(id)
- action_type TEXT NOT NULL  -- 'merge' or 'split'
- source_work_id INT4 REFERENCES works(id)  (merge target)
- target_work_id INT4 REFERENCES works(id)  (merge source)
- work_id INT4 REFERENCES works(id)  (split target)
- details JSONB
- status TEXT NOT NULL DEFAULT 'pending'  -- pending, accepted, rejected
- created_at TIMESTAMPTZ DEFAULT NOW()
- closed_at TIMESTAMPTZ

**`work_proposal_votes`** — Votes on proposals:
- proposal_id INT4 NOT NULL REFERENCES work_proposals(id)
- voter_id INT4 NOT NULL REFERENCES users(id)
- vote SMALLINT NOT NULL CHECK (vote IN (-1, 0, 1))
- created_at TIMESTAMPTZ DEFAULT NOW()
- PRIMARY KEY (proposal_id, voter_id)

**`reputation_events`** — Tracks reputation changes:
- id SERIAL PRIMARY KEY
- user_id INT4 NOT NULL REFERENCES users(id)
- event_type TEXT NOT NULL
- delta INT4 NOT NULL DEFAULT 0
- created_at TIMESTAMPTZ DEFAULT NOW()

**`tag_types`** — Tag type enum (1=fandom, 2=character, 3=relationship, 4=freeform, 5=warning, 6=category, 7=other)

**`tags`** — Canonical tags: id, name, tag_type_id, description

**`fic_tags`** — Fic-tag junction: url_id, tag_id, added_by_ip, score

**`follows`** (migration 003/017) — Follow fic/author/user:
- id SERIAL PRIMARY KEY
- user_id INT4 NOT NULL REFERENCES users(id)
- target_type TEXT NOT NULL  -- 'user' | 'work' | 'author'
- target_id INT4 REFERENCES works(id)  -- for work follows
- author_name TEXT  -- for author follows
- created_at TIMESTAMPTZ DEFAULT NOW()
- UNIQUE (user_id, target_type, COALESCE(target_id, 0), COALESCE(author_name, ''))

**`fic_requests`** (migration 018) — Fic Requests prompt board:
- id SERIAL PRIMARY KEY
- user_id INT4 NOT NULL REFERENCES users(id)
- title TEXT NOT NULL (the prompt, ≤200 chars)
- body TEXT NOT NULL DEFAULT '' (≤4000 chars)
- seed_work_id INT4 REFERENCES works(id)
- status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','answered','closed'))
- created_at / updated_at TIMESTAMPTZ
- closed_at TIMESTAMPTZ, deleted_at TIMESTAMPTZ
- accepted_answer_id INT4 REFERENCES fic_request_answers(id)
- INDEX (status, created_at DESC)

**`fic_request_answers`** (migration 018) — Works-only answers:
- id SERIAL PRIMARY KEY
- request_id INT4 NOT NULL REFERENCES fic_requests(id)
- user_id INT4 NOT NULL REFERENCES users(id)
- work_id INT4 NOT NULL REFERENCES works(id)
- pitch TEXT NOT NULL DEFAULT '' (≤500 chars)
- source TEXT NOT NULL DEFAULT 'user'  -- 'user' | 'auto' (engine-seeded)
- created_at TIMESTAMPTZ, deleted_at TIMESTAMPTZ
- UNIQUE (request_id, work_id)

**`fic_request_answer_votes`** (migration 018) — Community fit votes:
- answer_id INT4 NOT NULL REFERENCES fic_request_answers(id)
- user_id INT4 NOT NULL REFERENCES users(id)
- vote SMALLINT NOT NULL CHECK (vote IN (-1, 1))
- created_at TIMESTAMPTZ
- PRIMARY KEY (answer_id, user_id)

**`fic_request_upvotes`** (migration 035) — Request-level upvotes:
- request_id INT4 NOT NULL REFERENCES fic_requests(id) ON DELETE CASCADE
- user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE
- created_at TIMESTAMPTZ
- PRIMARY KEY (request_id, user_id); no self-vote (handler-enforced)
- INDEX (user_id)

**`modlog`** (migration 034) — Transparent moderation log:
- id BIGSERIAL PRIMARY KEY
- actor_id INT4, actor_username TEXT, action TEXT, target_type TEXT, target_id TEXT
- details JSONB, created_at TIMESTAMPTZ
- INDEXES on (created_at DESC), (action)

**`usage_events`** (migration 033) — Zero-PII usage analytics:
- id, client_id TEXT (anonymous X-Client-ID), path TEXT, event_type TEXT (view|action)
- created_at TIMESTAMPTZ; INDEX on (client_id, created_at)

**`scrape_failures` / `agent_runs`** (migrations 029-030) — Self-healing telemetry:
- scrape_failures: url, error, classification, fingerprint, snapshot, debounce
- agent_runs: agent id, action, status, details, created_at

**`reading_lists`** (migration 019) — Curated bundles:
- id SERIAL PRIMARY KEY
- user_id INT4 NOT NULL REFERENCES users(id)
- title TEXT NOT NULL
- description TEXT DEFAULT ''
- is_public BOOLEAN DEFAULT TRUE
- created_at / updated_at TIMESTAMPTZ

**`reading_list_items`** (migration 019) — Positioned list entries:
- id SERIAL PRIMARY KEY
- list_id INT4 NOT NULL REFERENCES reading_lists(id) ON DELETE CASCADE
- work_id INT4 NOT NULL REFERENCES works(id)
- position INT4 NOT NULL DEFAULT 0
- blurb TEXT DEFAULT ''
- added_at TIMESTAMPTZ DEFAULT NOW()
- UNIQUE (list_id, work_id)

**`series`** (migration 020) — Series:
- id SERIAL PRIMARY KEY
- title TEXT NOT NULL
- description TEXT DEFAULT ''
- created_at / updated_at TIMESTAMPTZ

**`series_works`** (migration 020) — Series membership (ordered):
- id SERIAL PRIMARY KEY
- series_id INT4 NOT NULL REFERENCES series(id) ON DELETE CASCADE
- work_id INT4 NOT NULL REFERENCES works(id)
- position INT4 NOT NULL DEFAULT 0
- UNIQUE (series_id, work_id)

**`feature_clusters`** (migration 011/016) — Roadmap consensus:
- id SERIAL PRIMARY KEY
- title TEXT NOT NULL
- representative_text TEXT NOT NULL UNIQUE (seed key)
- embedding vector(768)  -- pgvector, Ollama nomic-embed-text
- status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','shipped','rejected','deferred'))
- elo REAL NOT NULL DEFAULT 1500
- vote_count INT4 NOT NULL DEFAULT 0
- created_at / updated_at TIMESTAMPTZ

**`bot_scores`** / `search_queries` (migration 009/010) — Anti-bot + search analytics:
- bot_scores: client_id, score, last_seen, is_bot
- search_queries: query, results, created (hourly bot-scorer processes)


The Go backend (`fichub-cli`) uses its own set of tables for the v1 API. These are defined in `migrations/001_initial.up.sql` (not included in this repo but expected):
- `users` — user accounts with JWT auth
- `fics` — fanfiction works with full metadata
- `chapters` — fic chapter content
- `bookmarks` — user bookmarks
- `tags` / `fic_tags` — tag taxonomy and mapping
- `mod_actions` — moderation log
- `reports` — user reports
- `sessions` — JWT session tracking

---

## 8. Deployment

### Host

- **Host**: This machine (fichub.polarisocial.xyz)
- **OS**: Linux (kernel 7.0.0-28-generic)
- **User**: `alvaro`
- **Repo**: `/personal/documents/code/rust/fichub`

### Running Services

| Service | Port | Status | Binary |
|---------|------|--------|--------|
| fichub (Rust + SPA) | 8000 | running, systemd `fichub.service` | `target/release/fichub` |
| PostgreSQL | 5432 | running | system postgresql |
| Redis/Valkey | 6379 | running | system valkey |
| Ollama | 11434 | running | `nomic-embed-text` model |
| bot-scorer | — | systemd timer (hourly) | `target/release/bot-scorer` |

### Rust Backend Service

**Systemd service**: `fichub.service`
- Binary: `/personal/documents/code/rust/fichub/target/release/fichub`
- Working directory: `/personal/documents/code/rust/fichub/`
- Environment file: `/personal/documents/code/rust/fichub/.env`
- Serves: axum router + `ServeDir` fallback on `frontend/build/` (SPA), with `index.html` fallback for client-side routes.

### Deployment Workflow

1. **Build frontend**: `cd frontend && npm ci && npm run build` → `frontend/build/`
2. **Build Rust backend**: `cargo build --release` (unset `CARGO_TARGET_DIR` if a worktree left it set — it redirects the build!)
3. **Apply migrations**: the binary runs sqlx migrations on boot (`migrations/001..053`); psql-applied DDL must be recorded in `_sqlx_migrations` or the service fails with `VersionMismatch`.
4. **Restart service**: `sudo systemctl restart fichub`
5. **Verify**: `curl -s localhost:8000/api/health`, `curl -sI localhost:8000/feed.xml` (expect `application/atom+xml`)
6. **Push mirror**: `git push github main` (public mirror at opencommit.eu/MagicZhang/fichub.git)

**Gotchas**:
- A leftover `CARGO_TARGET_DIR=/home/alvaro/<wt>-target` env (from subagent worktrees) silently rebuilds into the WRONG target dir → the service keeps serving the old binary. `unset CARGO_TARGET_DIR` before `cargo build --release`.
- `feed.xml` returning HTML = old binary (route not compiled in) — check binary mtime after build.
- psql-created tables need `ALTER TABLE ... OWNER TO fichub` or sqlx migrations fail; psql-applied DDL must also be mirrored in `_sqlx_migrations` (delete stale rows and let the binary re-apply `IF NOT EXISTS` migrations).

---

## 9. API Endpoint Reference

### Frontend → API (single origin :8000)

The SPA calls the same origin's `/api/...` routes (the Rust backend serves both). The table below documents the v1-style endpoints for completeness; the live backend is the Rust one.

| Method | Endpoint | Purpose | Frontend Caller |
|--------|----------|---------|-----------------|
| GET | `/api/v1/fics` | List/search fics | `listFics()` |
| GET | `/api/v1/fics/:id` | Get fic detail | `getFic(id)` |
| GET | `/api/v1/fics/:id/download` | Download fic | `getDownloadUrl(id, fmt)` |
| GET | `/api/v1/fics/:id/versions` | List versions | (not called from frontend) |
| GET | `/api/v1/fics/:id/tags` | List fic tags | (not called from frontend) |
| POST | `/api/v1/fics` | Create fic | (not called from frontend) |
| POST | `/api/v1/fics/:id/tags` | Suggest tags | `suggestTags(ficId, tags)` |
| GET | `/api/v1/fics/:id/recommendations` | Get recs | `getRecommendations(ficId)` |
| POST | `/api/v1/auth/login` | Login | `login(username, password)` |
| POST | `/api/v1/auth/register` | Register | `register(username, password)` |
| POST | `/api/v1/auth/refresh` | Refresh token | (not called from frontend) |
| POST | `/api/v1/auth/logout` | Logout | (not called from frontend) |
| GET | `/api/v1/users/:id` | Get user | `getUser(id)` |
| GET | `/api/v1/users/:id/bookmarks` | List bookmarks | `getBookmarks(userId)` |
| POST | `/api/v1/users/:id/bookmarks` | Add bookmark | `addBookmark(userId, ficId)` |
| GET | `/api/v1/search` | Search fics | `search(params)` |
| GET | `/api/v1/tags` | List tags | `listTags(params)` |
| GET | `/api/v1/modlog` | List mod log | `getModlog(page)` |
| POST | `/api/v1/modlog/:id/vote` | Vote on mod action | `voteModlog(actionId, vote)` |

### Rust API (single origin :8000)

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/` | API documentation |
| GET | `/api/v0/epub?q=<url>` | Export EPUB for a fic URL |
| GET | `/api/v0/meta?q=<url>` | Get fic metadata |
| GET | `/api/v0/remote` | Get remote IP info |
| GET | `/api/health` | Health check (DB + Redis ping) |
| GET | `/api/v0/recommendations?url_id=&n=&site_domain=` | Get recommendations |
| POST | `/api/v0/recommendations/suggest` | Submit recommendation |
| POST | `/api/v0/recommendations/vote` | Vote on suggestion |
| GET | `/api/v0/recommendations/votes` | Get votes |
| POST | `/api/v0/tags/submit` | Submit a tag for a fic |
| POST | `/api/v0/tags/vote` | Vote on a tag |
| POST | `/api/v0/tags/flag` | Flag a tag |
| GET | `/api/v0/tags?url_id=X` | Get all tags for a fic |
| GET | `/api/v0/search` | Advanced search with tag filters |
| POST | `/api/v0/curator/alias` | Create tag alias (curator) |
| POST | `/api/v0/curator/merge` | Merge tags (curator) |
| DELETE | `/api/v0/curator/tags/:id` | Delete tag (curator) |
| GET | `/api/v0/curator/flags` | List unresolved flags (curator) |
| POST | `/api/v0/curator/flags/:id/resolve` | Resolve flag (curator) |
| GET | `/cache/:etype/:url_id/:fname` | Download cached file |
| GET | `/cache/:etype/:url_id` | Download or trigger export |
| POST | `/api/auth/register` | Register user (returns JWT) |
| POST | `/api/auth/login` | Login (returns JWT) |
| GET | `/api/auth/me` | Get current user from token |
| POST | `/api/bookmarks` | Add bookmark (work_id) |
| GET | `/api/bookmarks` | List user's bookmarks |
| DELETE | `/api/bookmarks/{work_id}` | Remove bookmark |
| POST | `/api/ratings` | Rate a work (work_id) |
| GET | `/api/ratings/{work_id}` | Get aggregate ratings |
| POST | `/api/comments` | Post comment (work_id) |
| GET | `/api/comments/{work_id}` | Get comments for a work |
| DELETE | `/api/comment/{id}` | Soft delete comment (owner/curator) |
| PATCH | `/api/comment/{id}/hide` | Hide comment (curator) |
| GET | `/api/leaderboard/curators` | Top curators by reputation |
| GET | `/api/users/{id}` | User profile |
| POST | `/api/work-proposals` | Create merge/split proposal (curator) |
| GET | `/api/work-proposals` | List pending proposals |
| GET | `/api/work-proposals/{id}` | Proposal detail with votes |
| POST | `/api/work-proposals/{id}/vote` | Cast vote (curator) |
| POST | `/api/ratings` | Rate a work (1..=5) |
| GET | `/api/ratings/{work_id}` | Aggregate ratings (positive-only, no dislikes) |
| POST | `/api/reviews` | Upsert in-depth review |
| DELETE | `/api/reviews/{id}` | Soft delete review |
| GET | `/api/works/{id}/reviews` | List reviews |
| POST | `/api/follows` | Follow user/work/author |
| GET | `/api/follows` | List my follows |
| DELETE | `/api/follows/{id}` | Unfollow |
| GET | `/api/follows/check/{type}/{id}` | Check follow state |
| GET | `/api/v1/feed` | Follow updates feed |
| POST | `/api/v1/works/{url_id}/refresh` | Re-scrape + notify followers |
| POST | `/api/requests` | Create fic request |
| GET | `/api/requests` | List requests (status/sort/page) |
| GET | `/api/requests/{id}` | Request detail + answers + my_vote |
| DELETE | `/api/requests/{id}` | Soft delete request |
| POST | `/api/requests/{id}/answers` | Add work answer (work_id or URL) |
| DELETE | `/api/requests/{id}/answers/{aid}` | Soft delete answer |
| POST | `/api/requests/{id}/answers/{aid}/vote` | Vote on answer (1/-1/0) |
| POST | `/api/requests/{id}/upvote` | Upvote request (toggle) |
| POST | `/api/requests/{id}/accept/{aid}` | Accept answer (requester) |
| GET | `/api/requests/{id}/candidates` | Engine candidates (login-gated, seed work) |
| GET | `/api/series/{id}` | Series detail + works |
| GET | `/api/authors/search?q=` | Search authors |
| GET | `/api/authors/by-name/{name}` | Author bibliography |
| GET | `/api/authors/{id}` | Author profile |
| GET/POST | `/api/lists` | Reading lists |
| GET/PUT/DELETE | `/api/lists/{id}` | List detail/update/delete |
| POST | `/api/lists/{id}/items` | Add list item |
| DELETE | `/api/lists/{id}/items/{iid}` | Remove list item |
| GET | `/api/reader/{url_id}` | Reader HTML bundle |
| GET | `/api/reader/{url_id}/meta` | Reader metadata |
| GET | `/api/reader/{url_id}/sequel` | Next-in-series |
| GET | `/api/reader/{url_id}/related` | Related works |
| GET | `/api/search/body` | Full-text search over fic bodies (quote search) |
| PUT | `/api/user/site-credentials` | Store user site creds (opt-in, 30-day) |
| DELETE | `/api/user/site-credentials/{domain}` | Remove user site creds |
| GET | `/api/user/site-credentials` | List configured creds (never passwords) |
| GET | `/api/v1/works/{url_id}/also-bookmarked` | Co-bookmarked rec anchors |
| GET | `/feed.xml` | New arrivals Atom feed |
| GET | `/feed/follows.xml` | Follows Atom feed (token) |
| GET | `/feed/works/{url_id}` | Per-fic Atom feed |
| GET | `/api/roadmap/arena` | Roadmap arena state |
| POST | `/api/roadmap/vote` | Cast arena comparison |
| GET | `/api/roadmap/features` | Feature clusters |
| POST | `/api/roadmap/features` | Suggest feature |
| GET | `/api/roadmap/leaderboard` | Elo rankings |
| GET | `/api/admin/bots` | Bot list |
| POST | `/api/admin/bots/{id}/shadowban` | Shadowban bot (Redis) |
| GET | `/api/admin/realtime` | Realtime analytics |
| GET | `/api/admin/search-analytics` | Search analytics |

### OPDS Catalog (port 8000, served by Rust binary)

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/opds` | Root navigation catalog |
| GET | `/opds/new?page=N` | Recent fics |
| GET | `/opds/popular?page=N` | Popular fics by word count |
| GET | `/opds/tags` | List all tag types |
| GET | `/opds/tags/{type_id}` | List canonical tags of that type |
| GET | `/opds/tags/{type_id}/{tag_name}?page=N` | Fics with that tag |
| GET | `/opds/authors?page=N` | List authors with fic counts |
| GET | `/opds/authors?author=Name&page=N` | Author's works |
| GET | `/opds/recommendations/popular` | Top recommendations |
| GET | `/opds/recommendations?url_id=X` | Per-work recommendations |
| GET | `/opds/search` | OpenSearch description |
| GET | `/opds/search?q=...&include_tags=...` | Faceted search results |
| GET | `/opds/shelves?token=X` | List reading shelves |
| GET | `/opds/shelf/{id}?token=X` | Shelf contents |

### Response Format

**Success**: Direct JSON object (no wrapping envelope)
**Error**:
```json
{"err": -1, "msg": "description"}
```

Common error codes:
- `-1`: Internal / database error
- `-5`: Bad request / not found
- `-6`: Scrape error (bad gateway)
- `-10`: Blocked (automated request detected)
- `-429`: Rate limited

---

## 10. Data Flow Examples

### EPUB Export Flow (Rust v0)

```
1. User visits: GET /api/v0/epub?q=https://archiveofourown.org/works/123456
2. Rust handler (single origin :8000):
   a. Validates q parameter (non-empty, not automated)
   b. Registry finds Ao3Scraper via can_handle()
   c. Scraper fetches HTML from AO3
   d. Parses title, author, chapters, words, description, stats
   e. Upserts fic_info row in PostgreSQL
   f. Checks fic_blacklist and author_blacklist
   g. Fetches all chapters via scraper.fetch_chapters()
   h. Generates EPUB via epub.rs (epub-builder)
   i. Stores to disk cache (hash-based directory)
   j. Logs to request_log table
   k. Returns redirect to cached file or JSON with download URL
4. Frontend receives download URL and triggers download
```

### Search Flow (Frontend → API)

```
1. User types query on /search page
2. Frontend calls: fetch('/api/search?...')
3. Request to: GET /api/search?q=harry+potter&include_tags=...&sort=...
4. Rust backend: builder.rs compiles tsquery (AND/OR, main_char_attr, typo tolerance) → PostgreSQL
5. Returns: { items: [...], total: 42, page: 1 }
6. Frontend renders fic cards in grid layout
```

### Login Flow

```
1. User submits username + password on /login page
2. auth.login(username, password) calls the API client
3. POST /api/auth/login with JSON body { username, password }
4. Backend validates credentials, returns JWT token + user profile
5. Client stores token to localStorage (key: "fichub_token")
6. auth store sets user state from the response
7. auth store persists user to localStorage
8. Page redirects to "/"
9. Subsequent API calls include Authorization: Bearer *** header
```

---

## 11. Current Issues & Technical Debt

### Frontend

| Issue | Impact | Location |
|-------|--------|----------|
| Reader page test flake in parallel runs (jsdom `goto` in keydown handler) | CI noise | `read/[urlId]/page.test.ts` |
| `$page?.url` guards needed in unit tests (no router) | Test ergonomics | `requests/new/+page.svelte` |
| No loading skeletons on all pages | UX gap | Various pages |
| PWA offline covers reader only; full app-shell offline pending | Offline UX | `sw.js` |
| Fic Requests answer form uses Work ID input (autocomplete for fics is M2) | UX | `requests/[id]/+page.svelte` |

### Rust Backend

| Issue | Impact | Location |
|-------|--------|----------|
| `url_id` is only 12 hex chars (SHA-256 truncated to 6 bytes) | Collision risk | `scrape/mod.rs` |
| No connection pooling health checks | Reliability | `server.rs` |
| Calibre sidecar assumes container name | Fragile | `config.rs`, `export/convert.rs` |
| Rate limiter Lua script not optimised | Performance | `limiter/redis_bucket.rs` |
| No request timeout middleware | Reliability | `server.rs` |
| No structured error logging for all error types | Debugging | `error.rs` |
| `CacheSemaphores` never cleaned up | Memory leak | `cache/mod.rs` |
| No pagination on request_log queries | Performance | `db/queries.rs` |
| Recommender worker has no circuit breaker | Reliability | `recommender/worker.rs` |
| Cross-binary DB-gated test pollution (each suite has its own mutex; combined runs can flake) | CI noise | `tests/*.rs` |
| `/api/requests/{id}/candidates` is an M2 stub (returns empty) | Feature gap | `routes/requests.rs` |
| Fic Requests M2/M3: URL-ingest answers, auto-seeded candidates, notifications | Feature gap | — |

> **Resolved 2026-08-11**: candidates is a real engine endpoint (engine
> suggestions from the seed work, login-gated — dd64fe5); URL-ingest answers
> shipped (e92de9a); M3 upvotes + notifications shipped (6b86164). The two
> rows above are historical.

### Architecture / Deployment

| Issue | Impact | Location |
|-------|--------|----------|
| Historical Go backend (`fichub-cli.bak`) + threadlight docs linger in repo | Confusion | `docs/`, `docker/` |
| Public mirror (`github`) needs manual `git push` after merges | Sync burden | — |
| Forgejo internal remote (192.168.1.138:3000) unreachable | CI unavailable | — |
| Send-to-Kindle requires SMTP credentials | Feature gated | `routes/kindle.rs` |
| AO3/FFN scraping blocked from this host (only RoyalRoad/Quotev/Wattpad reachable) | Content gap | `scrape/` |
| No CI/CD pipeline | Manual deployment | — |
| No monitoring/alerting | Blind operation | — |
| Database password in .env (not in production secrets) | Security | `.env` files |

---

## 12. Environment Variables

### Rust Backend

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `REDIS_URL` | Yes | — | Redis connection string |
| `JWT_SECRET` | Yes | — | JWT signing secret |
| `CACHE_DIR` | No | `./cache` | Primary cache directory |
| `SECONDARY_CACHE_DIR` | No | — | Secondary (warm) cache |
| `EXPORT_VERSION` | No | `1` | Export format version |
| `DYNAMIC_RATE_LIMIT` | No | `true` | Enable dynamic rate limiting |
| `NODE_NAME` | No | `orion` | Node identifier for logs |
| `CALIBRE_CONTAINER` | No | — | Calibre Docker container name |
| `TMP_DIR` | No | `./tmp` | Temp directory for exports |
| `PORT` | No | `8000` | HTTP listen port |
| `FRONTEND_DIR` | No | `./frontend/build` | Path to SvelteKit build |
| `OPDS_BASE_URL` | No | — | Public base URL for OPDS links |
| `OPDS_SHELF_TOKEN` | No | — | Token for OPDS shelf feeds |
| `CURATOR_TOKEN` | No | — | Curator bootstrap token |
| `OLLAMA_URL` | No | `http://localhost:11434` | Ollama endpoint |
| `OLLAMA_EMBED_MODEL` | No | `nomic-embed-text` | Embedding model |
| `POW_DIFFICULTY` / `POW_TTL_SECS` | No | — | Proof-of-work anti-bot params |
| `TRUSTED_PROXIES` | No | — | Comma-separated trusted proxy IPs |
| `IP_TAG_SOURCES` | No | — | Datacenter IP list sources (legacy flat-file path; superseded by `MAXMIND_DB`) |
| `MAXMIND_DB` | No | — | Path to MaxMind GeoLite2-ASN `.mmdb` — rate limiter blocks datacenter/hosting ASNs (1h retry-after); unset/unreadable = fail-open |
| `REC_*` | No | various | Recommender configuration (see `config.rs`) |
| `RUST_LOG` | No | `info,fichub=debug` | Tracing filter |

### Go Backend

| Variable | Required | Description |
|----------|----------|-------------|
| `FICHUB_CONFIG` | No | Path to config file |
| `FICHUB_SERVER_PORT` | No | HTTP port (default 8005) |
| `FICHUB_DATABASE_HOST` | Yes | PostgreSQL host |
| `FICHUB_DATABASE_PORT` | No | PostgreSQL port (default 5432) |
| `FICHUB_DATABASE_USER` | Yes | PostgreSQL user |
| `FICHUB_DATABASE_PASSWORD` | Yes | PostgreSQL password |
| `FICHUB_DATABASE_NAME` | Yes | PostgreSQL database name |
| `FICHUB_REDIS_HOST` | Yes | Redis host |
| `FICHUB_REDIS_PORT` | No | Redis port (default 6379) |
| `FICHUB_JWT_SECRET` | Yes | JWT signing secret |
| `FICHUB_STORAGE_PATH` | Yes | File storage path |
| `FICHUB_TYPESENSE_API_KEY` | Yes | Typesense API key |

---

## Appendices

### A. Related Repositories

| Repository | Path | Language | Purpose |
|-----------|------|----------|---------|
| fichub (Rust) | `/personal/documents/code/rust/fichub/` | Rust + SvelteKit/TS | The full stack (backend + SPA in one repo) — THIS DOCUMENT |
| Public mirror | `github` remote → opencommit.eu/MagicZhang/fichub.git | — | Read-only public mirror |
| Forgejo (internal) | 192.168.1.138:3000 (unreachable) | — | Internal CI/remote |
| (Historical) fichub-cli | `~/code/go/fichub-cli.bak/` | Go | v1 API backend — superseded |
| (Historical) fichub-frontend | `~/code/go/fichub-frontend/` | SvelteKit/TS | Old split frontend — superseded |
| (Historical) threadlight | `~/code/rust/threadlight/` | Rust | Social platform — superseded |

### B. Common Commands

```bash
# Build frontend
cd /personal/documents/code/rust/fichub/frontend && npm ci && npm run build

# Build Rust backend (unset CARGO_TARGET_DIR if a worktree left it set!)
cd /personal/documents/code/rust/fichub && unset CARGO_TARGET_DIR && cargo build --release

# Restart + verify
sudo systemctl restart fichub
curl -s localhost:8000/api/health
curl -sI localhost:8000/feed.xml   # expect application/atom+xml

# Run DB-gated integration tests (all suites, serialised)
cd /personal/documents/code/rust/fichub && set -a && . ./.env && set +a
cargo test --test requests_api -- --include-ignored --test-threads=1
cargo test --test search_api -- --include-ignored --test-threads=1

# Run lib unit tests + frontend
cargo test --lib
cd frontend && npm test && npm run test:e2e

# Frontend coverage gate
cd frontend && npm run coverage

# Seed roadmap consensus (needs Ollama + .env)
cd /personal/documents/code/rust/fichub && set -a && . ./.env && set +a && cargo run --bin seed-roadmap

# Rebuild + serve docs (mdbook)
cd docs && ./build.sh

# Push public mirror
git push github main

# Check service logs
journalctl -u fichub -n 50 --no-pager
```

### C. Git History (Frontend)

```
61f79ca (HEAD -> main, origin/main) feat: FicHub frontend SPA (forked from Photon)
  - Complete fork of Photon/Xyphyn adapted for FicHub
  - Full UI component library + FicHub API client + all routes
  - Clean root commit (orphan branch, no broken ancestry)
```

---

# 13. Session Update — 2026-08-07 (CURRENT LIVE STATE)

> This section documents the actual current state of the system as of the most
> recent engineering session. Sections 1-12 above describe the historical
> design (including the older Go backend / Orange Pi deployment); this section
> is what a fresh reader (human or chatbot) should treat as authoritative for
> what runs today.

## 13.1 Current deployment topology

- **Host**: this machine (M720q), NOT the Orange Pi (that deployment is legacy).
- **Backend**: Rust/Axum binary `fichub`, systemd service `fichub.service`.
  - Port **8000**, `EnvironmentFile=.env`, `User=alvaro`.
  - `WorkingDirectory=/personal/documents/code/rust/fichub` (symlink to
    `/home/alvaro/documents/code/rust/fichub`).
  - Serves the API AND the built SvelteKit frontend (`frontend/build/`) via
    ServeDir fallback — no separate frontend server.
  - `Environment=PATH=.../home/alvaro/.local/bin:...` (systemd default PATH
    excludes ~/.local/bin, which is where `fanficfare` lives).
- **Frontend**: SvelteKit 5, adapter-static, built into `frontend/build/`
  (gitignored). Rebuild: `cd frontend && npm run build`.
- **Database**: PostgreSQL 16, db `fichub`, sqlx migrations `migrations/001`-`053`.
  Access as superuser: `sudo -u postgres psql -d fichub` (peer auth).
- **Redis**: used for rate limiting + the suggest cache.
- **External domain**: https://fichub.polarisocial.xyz (nginx → :8000).
- **Docs**: mdBook in `docs/` → built to `frontend/static/docs/` (served) +
  `docs/book/` + `docs/FicHub_Docs.epub`. Build: `cd docs && bash build.sh`
  (requires `mdbook` + `mdbook-epub` in `~/.local/share/cargo/bin`).
- **EPUB deliverable** (user-facing): `/tmp/FicHub_Docs.epub` + repo
  `docs/FicHub_Docs.epub`.

## 13.2 Scraping pipeline

- **FanFicFare is the PRIMARY scraper** (`src/scrape/registry.rs`):
  `Command::new("fanficfare")` — installed via `pipx install fanficfare`
  (PEP 668 blocks bare pip), CLI at `~/.local/bin/fanficfare`.
- Native Rust scrapers are fallbacks. `extract_tags` maps FanFicFare metadata
  → structured tags (genre/freeformtags/extratags → freeform type 4,
  characters → type 2, ships → type 3, warnings → type 5, category → type 6,
  fandoms → type 1, rating → freeform). CSV split → trim → dedupe by
  (name, type), capped at 50.
- **Main-character scoring (NEW)**: `ExtractedTag` has a `score` field. The
  FIRST-listed character is the main character (score 10), secondary chars 1;
  first-listed ship = primary pairing (5), others 1. `upsert_fic_tag` writes
  score (`GREATEST` on conflict). Powers the `main_char_attr` search.
- **Site reachability from this host (2026-08-12)**: royalroad 200, quotev
  200, wattpad 200, fichub.net 200; AO3 404+bot-challenge (was 525
  Cloudflare block — now a soft block), fanfiction.net 403 Cloudflare,
  spacebattles 403, fimfiction 403. So content ingestion is limited to
  RoyalRoad/Quotev/Wattpad from here.

## 13.3 Content

- The archive holds ~18-20 real RoyalRoad fics (populated via
  `scripts/populate_royalroad.py`, which scrapes the best-rated list through
  `POST /api/epub?q=<url>` with polite delays). This is intentional production
  content (NOT seed data).
- Tags: freeforms (Adventure, Fantasy, ...), warnings (Graphic Violence,
  Sexual Content, Profanity), categories (Original), all from RoyalRoad
  genres. Fandoms = 0 (correct — RoyalRoad uses genres, not fandoms).
- **File locations rule (user preference)**: fics saved by fichub go to
  `/public/literature/fichub` (service TMP_DIR=`/public/literature/fichub/tmp`);
  anything else goes to `/tmp`. Never leave fic EPUBs in the repo root
  (`*.epub` is gitignored).

## 13.4 Search system (advanced)

- **Endpoint**: `GET /api/search` with boolean query support.
- **Parser** (`src/search/parser.rs`): `AND`/`OR`/`NOT`, implicit AND between
  words, quoted phrases, fielded terms (`title:`, `author:`, `fandom:`),
  exclusion (`-term`), nested parens. Emits `to_tsquery` (raw, non-stemmed
  terms where intentional).
- **Query params**: `q`, `include_tags` (AND, format `type_id:name`),
  `include_any_tags` (OR), `exclude_tags` (AND NOT), `min_words`, `max_words`,
  `min_chapters`, `max_chapters`, `complete`, `source`, `date_from`,
  `date_to`, `sort`, `primary_tag`, `min_comments`, `min_kudos`,
  `no_warnings`, `tag_ids`, `relationship_characters`, `main_char_attr`,
  `page`, `per_page`.
- **`relationship_characters=Harry Potter`** — finds fics with ANY relationship
  tag whose name contains "Harry Potter" (include-any semantics).
- **`main_char_attr=Character|Attribute`** (NEW, the flagship feature):
  finds fics where the HIGHEST-SCORED character tag is `Character` (the MAIN
  character) AND the freeform `Attribute` tag is present. E.g.
  `Harry Potter|Dark Harry Potter` = fics STARRING Harry with the Dark tag —
  NOT fics where Harry is a side character. This solves the AO3 "dark harry
  potter" search gap.
- **Facets**: fandoms, characters, relationships, warnings, categories,
  freeforms, statuses — returned with counts, filterable by clicking.
- **Suggested filters**: `GET /api/search/suggest` — popular tags (GROUP BY
  usage) + personalized re-rank (`?personal=1`, JWT) with `reason:
  popular|for_you`; 5-min cache (`AppState.suggest_cache`).
- **Frontend**: `/search` page with main search bar, quick filter chips,
  suggested-filter chips (top 8, deduped vs active, hidden on error),
  advanced filters toggle (incl. Main-Character Attribute input), facet
  sidebar, active filter chips + Clear all, pagination.

## 13.5 Home dashboard (NEW)

- `/` (default tab) = Home dashboard:
  - Compact download input (paste URL → Download, hands off to the Download
    tab via sessionStorage + custom event).
  - **Recommended for you** (left): from `GET /api/recommendations/personal` —
    personalized by the user's bookmarks + downloads; shows "Because you
    bookmarked: ..." label. Fallback chain: personal → trending → popular →
    recently added → hint card ("Bookmark a few fics to personalize").
  - **Trending this week** (right): from `/api/trending`.
  - `Promise.allSettled` — each section loads independently, never a
    page-level error box.

## 13.6 Personalization

- `GET /api/recommendations/personal` (auth): keys on `bookmarks.user_id`
  (the SHIPPED bookmark table — NOT the legacy `fic_bookmarks`/user_hash,
  which is worker-only).
- Signals: bookmarks + downloads (request_log url_id hits in 90 days).
- Gate: < 3 combined signals → `enough_data:false` (200, hint card).
- Scoring: `tag_overlap` (Jaccard) + `personal_score` + recency popularity in
  `src/recommender/engine.rs`. SQL-side candidate generation (top-tag join,
  excludes known), Rust scoring, top 20.

## 13.7 Admin & roles

- **Role scale** (the REAL one, code-wide): `0=regular, 1=trusted, 5=curator,
  10=admin`. (The comment in auth.rs was fixed to match; do NOT use 0-3.)
- **Admin dashboard**: `/admin` (stats dashboard, moderation, scraper health,
  users role/bans) + `/api/admin/*` (stats, moderation queue, scraper-health,
  users CRUD). Guard: role ≥ 10 (frontend layout redirect + backend 403).
- **Curator**: `/curator/authors` (author merge proposals) + `/api/curator/*`
  (alias, merge tags, flags, author merges). Guard: role ≥ 5.
- **Admin user**: promoted to role 10 (was 0 → unreachable). Navbar now links
  Admin Dashboard + Curator for role ≥ 10.

## 13.8 OPDS

- OPDS feeds: `/opds`, `/opds/new`, `/opds/popular`, `/opds/tags`,
  `/opds/authors`, `/opds/search`, `/opds/shelves`, `/opds/recommendations`.
- **Auth-required feeds** (e.g. `/opds/shelves` with `OPDS_SHELF_TOKEN`, default
  `fichub`): missing/invalid token → 200 OPDS XML "authentication required"
  catalog (rel=self, entry with `?token=` instructions) — NOT a JSON 404.
- OPDS readers use `?token=`. Content-type must be XML/Atom.

## 13.9 QA harness

- `qa/` (committed): `qa.sh` (entry), `qa/run.js` (deterministic API smoke +
  link crawl + OPDS checks + browser journeys + journalctl log scan),
  `qa/triage.js` (local ollama triage, 0 tokens), `qa/gen-issues.js` →
  `qa/reports/ISSUES.md`, bug queue `qa/bugs.db` (SQLite, fingerprint-deduped).
- `AGENTS.md` (now at `docs/AGENTS.md`) documents fixer rules.
- **Nightly**: cron job `fichub-nightly-qa` at 4am (local-only).
- All previously-open bugs (3 P0 leaderboard 500s + 5 P1 OPDS/log/console)
  are FIXED (regression tests in `tests/leaderboard_api.rs`, `tests/opds_api.rs`),
  verified live, and marked closed in bugs.db. ISSUES.md now shows 0 issues.

## 13.10 Testing

- `cargo test` (lib): 400+ unit tests (parser 46+, builder, tags, recommender,
  scrape incl. main-character scoring).
- DB-gated integration tests (run with `. ./.env && cargo test --test <name>
  -- --include-ignored --test-threads=1`): `tests/search_api.rs` (13+ incl.
  main_char_attr ×2), `tests/recommender_personal.rs` (4),
  `tests/leaderboard_api.rs` (3), `tests/opds_api.rs` (3).
- Frontend: 121 tests / 14 files (vitest + testing-library), `npm run build`
  green.
- Conventions: DB tests use a global Mutex (poison-tolerant `db_guard()`),
  unique seed names (`searchit_*`, `SearchTest *`), cleanup after each test.

## 13.11 Known issues / future work

- **Existing DB rows** scraped before main-character scoring have score=0 for
  all tags → `main_char_attr` won't match them until re-scraped or backfilled.
- AO3/FFN host-blocked from this machine (AO3 404+challenge / FFN 403
  Cloudflare) — future: cf_clearance, cookie ingestion, proxy, or
  fichub.net-mediated ingestion.
- Frontend suggestions use `localStorage.getItem('fichub_token')` to decide
  `personal=1` — the auth store may use a different mechanism; verify.
- nginx config may still reference the old port (cosmetic; the service is on
  :8000).
- Advanced-search niche coverage (AO3-style) is being expanded; the flagship
  main_char_attr case + any-character generalization are covered.

## 13.12 Repo hygiene

- Root markdown files (AGENTS, FICHUB_DESIGN, IDEAS, SPECIFICATION, STATUS,
  TODO, WORKFLOW) moved to `docs/` (README.md stays at root).
- `docs/src/*.md` (mdBook source) is the canonical user docs; `docs/` root
  files are engineering notes.
- Plans live in `.hermes/plans/` (untracked working artifacts).
- `STATUS.md` at `docs/STATUS.md` tracks the current session's done/in-progress
  work.


======================================================================
SOURCE: docs/FICHUB_DESIGN.md
======================================================================

# FicHub v2 Architecture — Design Document

> **Status: ✅ IMPLEMENTED** (August 2026)
>
> The core of this design — the **unified works model** (Section 1 + Section 8) —
> has been fully implemented. The `works` table, auto-merge heuristics, curator
> proposals/voting, and social table migration are all live. Sections on
> translations (3), vector search (4), and gamification (5) are now
> **partially implemented** with full API and frontend support.
>
> See `references/unified-works-schema.md` for the current canonical schema.

> **Goal**: Evolve FicHub from a download tool into a full community platform with normalized metadata, translations, gamification, vector search, and features that 70%+ of users expect — all running on an Orange Pi 5 (4GB RAM).

---

## Table of Contents
1. [Database Normalization](#1-database-normalization)
2. [Metadata Blob + Curated Normalization](#2-metadata-blob--curated-normalization)
3. [Translations](#3-translations)
4. [Vector Embedding Search](#4-vector-embedding-search)
5. [Gamification & Leaderboards](#5-gamification--leaderboards)
6. [Features for 70%+ Users](#6-features-for-70-users)
7. [Orange Pi Constraints](#7-orange-pi-constraints)
8. [Migration Path](#8-migration-path)

---

## 1. Database Normalization

### Current State (v1)
A single `fic_info` table with flat columns. Tags are in a separate normalized system (003_tagging.sql). Search uses PostgreSQL `tsvector` on title + description.

### Implemented State (v2) ✅
```
works                          ← Core normalized work
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │ ← SERIAL PK (not url_id — simplified from design)
│ canonical_title TEXT         │ normalized title
│ canonical_author TEXT        │ normalized author
│ description TEXT             │ curated description
│ default_source_id TEXT       │ FK to fic_info(id) — preferred source URL
│ created_at TIMESTAMPTZ      │
│ updated_at TIMESTAMPTZ      │
└─────────────────────────────┘

fic_info                        ← Source-level metadata (gained work_id FK)
┌─────────────────────────────┐
│ id (PK)                     │ url_id hash (12 hex chars)
│ title TEXT                   │ original scraped title
│ author TEXT                  │ original scraped author
│ description TEXT             │ original scraped description
│ ...
│ source TEXT                  │ full URL
│ work_id INTEGER FK→works(id) │ ← NEW: link to normalized work record
└─────────────────────────────┘

auto_merge_log                  ← Tracks automatic merge operations
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │
│ source_url TEXT              │ URL that triggered the merge
│ matched_work_id INTEGER FK  │ work it was merged into
│ confidence FLOAT             │ merge confidence score
│ created_at TIMESTAMPTZ      │
└─────────────────────────────┘

work_proposals                  ← Curator merge/split proposals
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │
│ proposer_id INTEGER FK      │ user who proposed
│ action_type TEXT             │ 'merge' | 'split'
│ source_work_id INTEGER FK   │ work to merge FROM
│ target_work_id INTEGER FK   │ work to merge INTO
│ work_id INTEGER FK           │ work being split (for splits)
│ details JSONB                │ proposal details
│ status TEXT                  │ 'pending' | 'accepted' | 'rejected'
│ created_at TIMESTAMPTZ      │
│ closed_at TIMESTAMPTZ       │
└─────────────────────────────┘

work_proposal_votes             ← Voting on proposals
┌─────────────────────────────┐
│ proposal_id INTEGER FK      │ → work_proposals(id)
│ user_id INTEGER FK           │ → users(id)
│ vote SMALLINT                │ 1=approve, 0=retract, -1=disapprove
│ voted_at TIMESTAMPTZ        │
│ PK (proposal_id, user_id)   │
└─────────────────────────────┘

reputation_events               ← Gamification foundation
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │
│ user_id INTEGER FK           │ → users(id)
│ event_type TEXT              │ e.g. 'curation_approve'
│ delta INTEGER                │ points (+/-)
│ created_at TIMESTAMPTZ      │
└─────────────────────────────┘
```

> **Design deviation**: The original design proposed `url_id` as the works PK
> and a `raw_json` JSONB column on fic_info. The implementation simplified this:
> `works` uses a SERIAL id (simpler joins, no hash deps), and `raw_json` was
> deferred (the scraper already stores structured data in fic_info columns).
> `curated_status` was also deferred — proposals/votes handle curation instead.

### Key Design Decisions

**1. Two-tier metadata (raw + curated) — IMPLEMENTED**
- `fic_info` keeps the ORIGINAL scraped data as-is (title, author, description, etc.)
- `works` table stores the CURATED, normalized version
- `fic_info.work_id` links each source record to its parent work
- All user-facing queries go through `works` (the normalized view)
- Curators propose changes via `work_proposals` + voting (not direct edits)

**2. Auto-merge on scrape — IMPLEMENTED**
- `find_or_create_work()` runs when a fic is scraped
- Exact title+author match (case-insensitive, trimmed) + word count within ±5%
- High confidence (≥0.9) → auto-merge immediately
- Medium confidence → create pending proposal
- New works are created automatically when no match is found

**3. Relationship to current tag system — PRESERVED**
- Current `tags`, `tag_types`, `fic_tags` tables remain unchanged
- Tags are associated with fic_info records (source-level), not works
- The existing tag voting (003_tagging.sql trigger) continues unchanged
- Future work could add work-level tag normalization via junction tables

**4. Curated status workflow — DEFERRED**
```
// Original design proposed this workflow:
0 = RAW:    Newly scraped, not touched by curator
1 = PARTIAL: Some fields normalized
2 = FULL:   All fields normalized and verified
3 = VERIFIED: Reviewed by trusted curator, locked

// Actual implementation: deferred — curation handled via
// work_proposals + voting instead. A curated_status column
// may be added in the future if editorial workflows are needed.
```

---

## 2. Metadata Blob + Curated Normalization

> **Status: DEFERRED** — `raw_json` column was not added to fic_info. The scraper
> already stores structured fields (title, author, description, words, chapters, etc.)
> in fic_info's existing columns. A JSONB blob may be added later if full scraper
> response archival becomes necessary.

### Raw JSON Blob (Planned, Not Implemented)

Add to `fic_info`:
```sql
ALTER TABLE fic_info ADD COLUMN raw_json JSONB;
```

Contains everything the scraper found:
```json
{
  "scraped_at": "2024-01-15T10:30:00Z",
  "url": "https://archiveofourown.org/works/12345678",
  "html_title": "The Story Title",
  "meta_tags": {"author": "AuthorName", "..."},
  "chapter_titles": ["Chapter 1", "Chapter 2"],
  "full_html": null,
  "scraper_version": "1.2.0"
}
```

`full_html` is NULL by default — only stored when a curator explicitly requests it for debugging.

### Curation Interface

| Tool | Description |
|------|-------------|
| **Field Editor** | Inline edit title, author, description, language, rating, status |
| **Tag Merger** | Search existing canonical tags, merge duplicates |
| **Tag Suggestion** | AI/algorithm suggests tags based on raw_json keywords |
| **Bulk Operations** | Apply changes across multiple works from same author/series |
| **Diff View** | See raw vs curated side-by-side before approving |

### Curation Permissions

| Role | Can edit own | Can approve edits | Can lock |
|------|-------------|-------------------|----------|
| New User | No | No | No |
| Reader | No | No | No |
| Curator | Yes (pending approval) | No | No |
| Senior Curator | Yes (instant) | Yes | No |
| Admin | Yes (instant) | Yes | Yes |

---

## 3. Translations

> **Status: ✅ IMPLEMENTED** — Locales, translations, and work_translations tables are live.
> See migration `003_follows_notifications_gamification_vector_translations.sql`.

### Database Schema

```sql
CREATE TABLE locales (
    id SERIAL PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,           -- "en", "es", "fr", "ja", "pt-BR"
    name TEXT NOT NULL,                  -- "English", "Español", ...
    is_rtl BOOLEAN DEFAULT FALSE
);

-- Static UI translations
CREATE TABLE translations (
    id BIGSERIAL PRIMARY KEY,
    locale_code TEXT REFERENCES locales(code),
    namespace TEXT NOT NULL,             -- "ui.download.title", "search.labels.fandom"
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(locale_code, namespace, key)
);

-- Dynamic content translations (fic metadata)
CREATE TABLE work_translations (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) REFERENCES works(url_id) ON DELETE CASCADE,
    locale_code TEXT NOT NULL REFERENCES locales(code),
    title TEXT,
    summary TEXT,
    -- Translated by
    translated_by INT4 REFERENCES users(id),
    translated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, locale_code)
);
```

### Translation Workflow

1. User with translator role opens a work page
2. Clicks "Translate" button → form shows original text alongside editable fields
3. Submits translation → queued for review (or auto-approved for trusted translators)
4. Translated version becomes available when locale matches user preference

### Locale Detection

- User setting (stored in profile)
- Browser `Accept-Language` header fallback
- URL prefix: `/es/works/{url_id}` (optional, configurable)

---

## 4. Vector Embedding Search

> **Status: PARTIALLY IMPLEMENTED** — Schema includes `vector(384)` column on works
> (requires pgvector extension). Embedding generation and hybrid search are future work.

### Constraint: Orange Pi 5 (4GB RAM)

This rules out:
- Running a full LLM locally (would use 4GB+ just for the model)
- HuggingFace transformers (too heavy)
- ONNX Runtime with large models

### Feasible Options

**Option A — pgvector with small model (RECOMMENDED)**

Use `all-MiniLM-L6-v2` via `transformers.js` or Rust's `fastembed` crate.
- 384-dim float vectors → ~1.5KB per row
- IVFFlat index with 100 centroids → <100ms ANN search on 100k rows
- Quantize to halfvec (768 bytes per row) or binary (48 bytes per row)
- Total vector storage for 200k fics: ~150MB (float), ~10MB (binary)

```sql
-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE works ADD COLUMN embedding vector(384);
ALTER TABLE works ADD COLUMN embedding_updated_at TIMESTAMPTZ;

-- IVFFlat index for approximate nearest neighbor search
CREATE INDEX idx_works_embedding ON works 
    USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- Or HNSW index (faster, more memory)
-- CREATE INDEX idx_works_embedding ON works USING hnsw (embedding vector_cosine_ops);
```

**Option B — Hybrid search (full-text + vector)**

Use PostgreSQL tsvector as primary search (already implemented in 003_tagging.sql) AND pgvector as a secondary ranker. Combine scores:

```sql
SELECT w.*, 
    ts_rank(w.text_search, to_tsquery('english', 'harry potter')) AS text_score,
    (1 - (w.embedding <=> query_embedding)) AS vector_score,
    (ts_rank(w.text_search, to_tsquery('english', 'harry potter')) * 0.6 + 
     (1 - (w.embedding <=> query_embedding)) * 0.4) AS combined_score
FROM works w
ORDER BY combined_score DESC
LIMIT 20;
```

**Option C — Embedding generation service (EDA APPROACH)**

Generate embeddings via a small standalone service:
- Tiny Rust binary using `fastembed` crate with `all-MiniLM-L6-v2`
- Model file: ~80MB on disk
- RAM usage: ~200MB while processing
- Processing time: ~200ms per fic (5 fics/sec)
- Batch processing: 100 fics in ~5 seconds
- Backfill 100k fics: ~5.5 hours (acceptable for a one-time job)

```bash
# Install model
curl -LO https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx

# Process embeddings in background (cron job, low priority)
./fichub embed --batch 100 --sleep-ms 50
```

### Search Architecture

```
User Query
    │
    ├── Text Query ──→ ts_rank(text_search, query)
    │                       │
    │                       ▼
    │               Combined Score ←── (0.6 × text + 0.4 × vector)
    │                       │
    │                       ▼
    │               Sorted Results
    │
    └── Syntax Query ──→ parseSearchQuery() → filter by tags/fields → sort
```

### Performance Budget

| Operation | Current | With pgvector | Notes |
|-----------|---------|---------------|-------|
| Full-text search | 5-15ms | 5-15ms | Unchanged |
| Vector search | N/A | 20-80ms | IVFFlat index |
| Hybrid search | N/A | 25-95ms | Combined |
| Embedding (batch) | N/A | 200ms per fic | Background job |
| RAM usage | ~1.7MB | +200MB | Model-only, not per-request |

---

## 5. Gamification & Leaderboards

> **Status: ✅ IMPLEMENTED** — `reputation_events` table, full badge system
> (18 badge definitions, auto-award on events), daily quests, login streaks,
> reading stats, and weekly/monthly leaderboards are all live.

### Reputation System

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE,
    role SMALLINT DEFAULT 0,             -- 0=reader, 1=curator, 2=senior, 3=admin
    reputation INT4 DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE reputation_events (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id),
    event_type TEXT NOT NULL,             -- "curation_approve", "translation", "suggestion_accept"
    points INT4 NOT NULL,
    reference_type TEXT,                  -- "work", "tag", "translation", "suggestion"
    reference_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE user_badges (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id),
    badge_type TEXT NOT NULL,             -- "curator_10", "translator_50", "voter_100"
    earned_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, badge_type)
);
```

### Reputation Point Values

| Action | Points | Badge Thresholds |
|--------|--------|------------------|
| Suggest a tag | +3 | 10 suggestions → "Tag Enthusiast" |
| Vote on tag | +1 | 100 votes → "Active Voter" |
| Curation approved | +25 | 10 approved → "Curator Apprentice" |
| Curation rejected | -5 | 50 approved → "Senior Curator" |
| Translation submitted | +15 | 10 translations → "Translator Novice" |
| Translation approved | +35 | 50 translations → "Polyglot" |
| Suggestion accepted | +20 | — |
| Daily login streak | +2/day | 7 days → "Streak Starter", 30 → "Dedicated" |
| Report bug | +10 | — |

### Leaderboards

| Leaderboard | Refresh | Scope |
|-------------|---------|-------|
| Top Curators (week) | Every hour | Weekly XP |
| Top Curators (all-time) | Daily | Total XP |
| Top Translators | Daily | Translation count |
| Most Active Voters | Hourly | Vote count |
| Rising Stars | Daily | New users with fastest rep growth |
| Works Most In Need | On-demand | Uncurated works by age/popularity |

### API Endpoints

```
GET /api/v1/leaderboard/curators?period=week&limit=20
GET /api/v1/leaderboard/translators?locale=es&period=all
GET /api/v1/users/{id}/reputation
GET /api/v1/users/{id}/badges
POST /api/v1/users/{id}/claim-badge/{badge_type}
```

### Daily Quests (Engagement)

| Quest | Reward |
|-------|--------|
| Curate 3 works today | +50 XP, "Curator" badge progress |
| Vote on 10 tags | +20 XP |
| Translate 1 summary | +40 XP |
| Suggest 5 tags | +15 XP |
| Login 7 consecutive days | +100 XP, streak badge |

---

## 6. Features for 70%+ Users

> **Status: IN PROGRESS** — Core social features (bookmarks, ratings, comments)
> and export system are live. Leaderboards, reading stats, and notifications
> are future work.

Based on typical fanfiction platform expectations + HN-style social features:

### Tier 1: Core (95%+ would want)
- ✅ **User accounts** — login/logout/profile
- ✅ **Bookmark/favorites** — save works to personal list
- ✅ **Search** — already exists, enhanced with vector
- ✅ **Download to EPUB/HTML/MOBI/PDF/AZW3/TXT/MD** — already exists
- ✅ **Filter by tags/status** — already partially exists
- ✅ **Dark mode** — already exists
- ✅ **Responsive mobile design** — already exists

### Tier 2: Expected (80%+ would want)
- ✅ **Reading history** — recently viewed works
- ✅ **Collections/lists** — create shared reading lists (004_shelves.sql exists)
- ✅ **RSS/OPDS** — subscribe to updates (already exists)
- ✅ **Related works** — recommendations (already exists)
- ✅ **Rating system** — like/dislike works
- ✅ **Comments** — discuss works
- ✅ **Author pages** — browse by author

### Tier 3: Addictive (70%+ would want)
- ✅ **Weekly/daily popular** — trending works this week
- ✅ **New works feed** — recently added works
- ✅ **Reading stats** — words read, fics completed
- ✅ **Custom recommendations** — based on reading history
- ✅ **Social features** — follows, notifications
- ✅ **Third-party API** — integrate with external tools
- ✅ **Random work button** — discovery
- ✅ **Work series tracking** — series with multiple parts

### Implementation Priority

| Phase | Features | Est. Backend | Est. Frontend | Risk |
|-------|----------|-------------|---------------|------|
| **P1** | User accounts, bookmarks, collections | 2 weeks | 1 week | Low |
| **P2** | Comments, ratings, reading history | 3 weeks | 1 week | Low |
| **P3** | Leaderboards, badge system, daily quests | 2 weeks | 2 weeks | Low |
| **P4** | Vector search, curation workflow | 3 weeks | 2 weeks | Medium |
| **P5** | Translations, author pages, reading stats | 2 weeks | 2 weeks | Medium |
| **P6** | Notifications, follows, series tracking | 2 weeks | 1 week | Medium |

---

## 7. Orange Pi Constraints

> **Status: REFERENCE** — Constraints unchanged. Budget estimates still valid.

### Memory Budget (4GB total)

| Component | RAM | Notes |
|-----------|-----|-------|
| OS + services | ~500 MB | Armbian, systemd basics |
| PostgreSQL | ~200 MB | Shared_buffers=256MB |
| Redis | ~50 MB | Default config |
| Rust backend (v2) | ~5 MB | Static binary, no heavy deps |
| Embedding model (runtime) | 0 MB | Only loaded during batch jobs |
| SvelteKit SPA | ~30 MB | Static files, served by Rust |
| **Available for features** | **~3.2 GB** | Growth room |

### What WON'T Work

- Running a full LLM (even llama 3B would consume >80% RAM)
- ONNX runtime with large models (>500MB)
- Real-time embedding at request time (batch only)
- ElasticSearch (too heavy, PostgreSQL full-text is sufficient)
- Image processing pipeline (thumbnails, OCR)

### What WILL Work

- pgvector with binary quantization (48 bytes per row)
- Lightweight ONNX model via `fastembed` or `ort` crate (200MB RAM, disposed after batch)
- PostgreSQL full-text search as primary index
- Background jobs for embedding generation (cron, low priority)
- Cached leaderboards (refresh every 5 min via cron)

### Storage Budget

| Data | Size Estimate |
|------|---------------|
| 200k works metadata | ~200 MB |
| 200k vectors (binary) | ~10 MB |
| Tag system (current) | ~5 MB |
| User accounts (10k) | ~10 MB |
| Bookmarks (500k) | ~50 MB |
| Comments (100k) | ~50 MB |
| Translations (50k) | ~10 MB |
| **Total** | **~335 MB** |

Well within the Orange Pi's available storage.

---

## 8. Migration Path

> **Status: Phase 0 IMPLEMENTED** — `002_unified_works.sql` has been applied.
> The remaining phases are future work.

### Phase 0: Schema Migration (v1 → v2 tables) ✅ DONE

Migration file: `migrations/002_unified_works.sql`

```sql
-- 1. Create works table with SERIAL PK (simplified from original url_id design)
CREATE TABLE works (
    id SERIAL PRIMARY KEY,
    canonical_title TEXT,
    canonical_author TEXT,
    description TEXT,
    default_source_id TEXT REFERENCES fic_info(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Add work_id FK to fic_info (links source → work)
ALTER TABLE fic_info ADD COLUMN work_id INTEGER REFERENCES works(id);
CREATE INDEX idx_fic_info_work_id ON fic_info(work_id);

-- 3. Bootstrap existing fic_info rows into works
--    Uses title+author matching to deduplicate:
--    - Exact match (case-insensitive, trimmed) → merge into existing work
--    - Word count within ±5% → high confidence auto-merge
--    - No match → create new work

-- 4. Create auto-merge log
CREATE TABLE auto_merge_log (
    id SERIAL PRIMARY KEY,
    source_url TEXT NOT NULL,
    matched_work_id INTEGER NOT NULL REFERENCES works(id),
    confidence FLOAT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 5. Create proposal/vote tables
CREATE TABLE work_proposals ( ... );
CREATE TABLE work_proposal_votes ( ... );

-- 6. Create reputation_events table
CREATE TABLE reputation_events ( ... );

-- 7. Migrate social tables to work_id
--    bookmarks, work_ratings, comments, opds_shelf_items
--    all gain work_id column, FK → works(id)
```

### What was NOT done (compared to original design)

- ❌ `raw_json JSONB` column on fic_info (deferred)
- ❌ `curated_status` column on works (deferred — proposals handle this)
- ❌ Junction tables (work_fandoms, work_characters, etc.) — deferred
- ❌ `source_version` column on works — not needed yet

### Phase 1: User System (non-breaking) ✅ DONE

- Users, auth, sessions — implemented
- Bookmarks — implemented (work_id-based)
- Collections — extend existing opds_shelves (implemented)

### Phase 2: Social & Gamification (partially done)

- Comments — ✅ implemented
- Ratings — ✅ implemented
- reputation_events — ✅ implemented (foundation)
- Leaderboards, badges, daily quests — ❌ future work

### Phase 3: Vector Search (future work) ❌

- pgvector extension — requires PostgreSQL restart
- Backfill embeddings via background job (cron, 5 fics/sec)
- Modify search to include vector score (behind feature flag)

### Phase 4: Translations & Curation (future work) ❌

- Locales, translations — new tables
- Work translations — new table
- Curation UI — new frontend pages
- Curation workflow — modifies works.curated_status

### Rollback Strategy

- All new tables are ADDITIONS — no existing tables are modified or dropped
- `fic_info` always retains original data, so rollback = just stop using new tables
- `works` duplicates data from `fic_info` — can be regenerated from `fic_info`
- Old search works unchanged — vector search is additive

---

## Summary Table

| Requirement | Approach | Status | Orange Pi Feasible? |
|-------------|----------|--------|-------------------|
| Normalized DB | works table + fic_info FK | ✅ Implemented | ✅ |
| Auto-merge | title+author matching + confidence | ✅ Implemented | ✅ |
| Curation proposals | work_proposals + voting | ✅ Implemented | ✅ |
| Metadata blob | raw_json JSONB on fic_info | ❌ Deferred | ✅ |
| Curated status workflow | curated_status + curator role | ❌ Deferred | ✅ |
| Social features | bookmarks, ratings, comments | ✅ Implemented | ✅ |
| Reputation foundation | reputation_events table | ✅ Implemented | ✅ |
| Leaderboards & badges | reputation + badge tables | ❌ Future | ✅ |
| Translations | work_translations table | ❌ Future | ✅ |
| Vector search | pgvector +all-MiniLM-L6-v2 | ❌ Future | ✅ (batch) |
| 70%+ features | 15 features across 3 tiers | 🟡 In progress | ✅ |
| Runs on Orange Pi 5 | 335MB storage, ~200MB added RAM | ✅ | — |

**Completed**: Unified works model, auto-merge, proposals/voting, social features (bookmarks, ratings, comments), export system (7 formats).
**Deferred**: raw_json blob, curated_status workflow, junction tables.
**Future**: Translations, vector search, full gamification (badges, leaderboards, daily quests).


======================================================================
SOURCE: docs/deployment-handover.md
======================================================================

# FicHub Deploy Machine (ThinkCentre M720q) — Handover Notes

Written 2026-08-08 when switching to gamingpc as the coding machine. This box is
now **deploy-only**: run the service, DB, Redis, Ollama, Cloudflare tunnel, and
nightly jobs. Do dev/build on gamingpc.

## Machine layout

- Host: `M720q` (thinkcentre), user `alvaro`, Linux Mint. LAN IP `192.168.1.13`.
- Repo (NFS-shared, single tree): `/personal/documents/code/rust/fichub`
  - Also reachable via symlink `/home/alvaro/code/rust/fichub` (same inode).
  - GamingPC mounts `/personal` over NFS and sees this same tree.
- Binary + data survive reboots (on /personal NFS pool, 5T, 11% used).
- `.env` lives at `/personal/documents/code/rust/fichub/.env` (gitignored).
  - Source it before running any bin: `set -a && . ./.env && set +a`

## Services (systemd)

| Unit | Purpose | Status |
|------|---------|--------|
| `fichub.service` | Rust Axum API :8000 (EPUB export, scrape, OPDS) | active |
| `cloudflared.service` | Cloudflare Tunnel (public domain) | active |
| `postgresql@16-main.service` | DB `fichub` | active |
| `redis-server.service` | cache / rate limits | active |
| `ollama.service` | embeddings (nomic-embed-text) | active |
| `fichub-bot-scorer.timer` | hourly bot scoring | active |
| `fichub-leaderboards.timer` | nightly 01:00 | active |
| `fichub-quests.timer` | nightly 00:05 | active |
| `fichub-stats.timer` | nightly 01:00 | active |
| `fichub-db-backup.timer` | nightly 03:30 pg_dump → NFS | **added 2026-08-08** |

Health: `curl -s localhost:8000/` → 200. Logs: `journalctl -u fichub -n 50`.

## Deploy (from gamingpc or here)

```bash
cd /personal/documents/code/rust/fichub
./deploy.sh              # build release on gamingpc + restart + health check
./deploy.sh --skip-build # binary already fresh; just restart + verify
```

Because the repo + target dir are NFS-shared, a build on gamingpc writes the
binary into the same tree the service runs. `deploy.sh` ssh's to gamingpc
(`unset CARGO_TARGET_DIR` — the stale-target trap), rebuilds `target/release/fichub`,
restarts `fichub.service`, and curls the root.

**CARGO_TARGET_DIR trap:** before any release build in the main repo, `unset
CARGO_TARGET_DIR`. A worktree-exported value redirects the build to the wrong
dir and leaves the served binary stale (playbook §1).

**Docs deploy** (no binary rebuild needed): the mdbook source lives in
`docs/`; rebuild with `docs/build.sh` (mdbook + mdbook-epub in
`~/.local/share/cargo/bin`) + `docs/build_docs_map.py`. The generated HTML
lands in `frontend/static/docs/` (committed) and `frontend/build/docs/`
(served from disk). After committing, just restart the service to serve the
new static docs:
```bash
ssh thinkcentre "cd /personal/documents/code/rust/fichub && sudo systemctl restart fichub"
```

## Daily jobs fixed 2026-08-08

- `fichub-stats.service` — was dying at 01:00 every night:
  - `compute_stats.rs` counted `works.source_type` (no such column — it's on
    `fic_info`) → fixed with `LEFT JOIN fic_info fi ON fi.work_id = w.id`.
  - `compute_stats.rs` counted `export_log.format` (no such column — it's
    `etype`) → fixed to `etype = 'epub'`.
  - Same latent bug fixed in `src/routes/admin.rs` (mod queue used `w.source_type`).
  - Verified: run writes a row into `admin_daily_stats` (was empty for weeks).
- `fichub-quests.service` — was dying with "must be owner of table
  feature_clusters" (psql-applied migration ownership trap). Fixed by the
  2026-08-08 rebuild; verified exits 0.

## Backups

- New: `fichub-db-backup.timer` → 03:30 nightly `pg_dump` of DB `fichub` to
  `/personal/software/backup/fichub/pg_fichub_<stamp>.sql.gz`, keeps 14.
  - Script: `/usr/local/sbin/fichub-db-backup.sh`
  - Restore: `zcat <file> | sudo -u postgres psql fichub`
- Legacy one-off dump: `/personal/software/backup/white-desktop-20260801/pg_fichub.sql`

## tmp / SSD-life

- `/tmp` is a **4G tmpfs** (RAM) — the ENOSPC trap. Do NOT put build trees or
  worktrees there.
- Worktrees + cargo targets live on the external/NFS pool:
  `/media/alvaro/code-worktrees/*` (targets via `.cargo/config.toml` or
  `CARGO_TARGET_DIR=/home/alvaro/<wt>-target`).
- Generic tool temp (node, cc, etc.) is redirected to `/media/alvaro/tmp`
  via `TMPDIR` export in the real zsh config:
  `$ZDOTDIR/zshrc.local` (`/personal/documents/code/misc/dotfiles/config/zsh/zshrc.local`).
  Your shell is **zsh**, and `ZDOTDIR` points at the dotfiles repo, NOT `~/.zshrc`.
- Cleanup habit: `du -sh /tmp/* | sort -rh | head` — anything big is a leftover.

## Dev on gamingpc (compile host)

- `ssh gamingpc` — Ryzen 7 5700G, 16 threads, 30G RAM, mounts `/personal` NFS.
- Repo at `/personal/documents/code/rust/fichub` (same shared tree).
- `cargo build --release` there is ~2.5-3x faster than this box.
- The stack (Postgres/Redis/Ollama/service + DB-gated tests) stays HERE. Only
  compile happens on gamingpc.

## Git

- Remotes: `origin` (git.polarisocial.xyz), `forgejo` (192.168.1.138), `github`
  (opencommit.eu mirror, inline token in URL — never in git config).
- Mirror rule: after merging to main, `git push github main`.
