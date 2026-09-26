# FicHub — Rust Backend + SvelteKit SPA

Self-hosted fanfiction archive and download server (fichub.net replacement).  
Scrapes fanfiction sites, generates EPUB, HTML, MOBI, PDF, AZW3, TXT, Markdown. Serves a full community platform via HTTP API: user accounts, bookmarks, 5-star ratings, reviews, threaded comments, Fic Requests (prompt board), community forum with site-wide leveling, reading lists, follows + updates feed, series/author pages, RSS/Atom feeds, in-browser reader, roadmap consensus, anti-bot defense, **progression system** (100 levels, ability tree, rank-gated features), **recipe builder** (user-composed recommendation blends), **theme design tokens** (CSS custom properties with presets and import/export), **trust levels** (Discourse-style participation axis with community self-moderation), and an **extension marketplace** (unified gallery for themes, skins, recipes, layouts, and views).

**Repository**: https://opencommit.eu/MagicZhang/fichub.git (public mirror)  
**Stack**: Rust + Axum 0.8 + SQLx 0.9 + PostgreSQL + Redis + Ollama (embeddings) + SvelteKit 5 (frontend)

## Inspiration

FicHub draws inspiration from the following archive-related projects:

- [ficdb.net](https://www.ficdb.net) — fanfiction database and archive
- [relatedrepos.com/gh/FicHub/fichub.net](https://relatedrepos.com/gh/FicHub/fichub.net) — related-repository discovery for the original fichub.net
- [relatedrepos.com/gh/Zeks/flipper](https://relatedrepos.com/gh/Zeks/flipper) — related-repository discovery for Flipper
- [relatedrepos.com/gh/JimmXinu/FanFicFare](https://relatedrepos.com/gh/JimmXinu/FanFicFare) — related-repository discovery for FanFicFare, the fanfiction download tool

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
        │    preserves the current engine exactly; recipes override per-user)
        ├─► Progression engine (100 levels, 10 ranks, XP events,
        │    ability tree, feature gates, theme/layout/widgets)
        ├─► Trust engine (7-level participation axis, auto-promotions,
        │    report auto-triage, weekly moderation digest)
        ├─► Extension marketplace (unified gallery: themes, skins,
        │    recipes, layouts, views — install / rate / remix)
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
- **API route-walk** (`qa/api-walk.js`) — scriptable e2e audit that enumerates every route from the router, asserts auth gates, and flags 5xx/stub responses (97/97 green).
- **Community forum** (`/forum`) — categories, topics, posts, follows, notifications, read state, FTS search with snippets, **moderation points** (queue, reports-as-signal, curator fast-hide), **metamoderation** (anonymized audit, unfair-rate, cooldowns), **site-wide leveling** (0–100 levels + exp), invites + registration applications, user blocks; topics get canonical **slug URLs** (`/forum/board/{topic_slug}.{topic_id}`, legacy `/forum/{cat}/{id}` still works). Engine = the `forum-core` crate; contract in `docs/FORUM-API-CONTRACT.md` + `docs/SPEC-COMMUNITY-PLATFORM.md`.
- **Chat bot** (`fanfic-archivist`) — Socrates-style recommendations in Discord / Telegram / Matrix: `!recs`, `!fresh`, `!gems`, `!roll` + pagination, `/search`, `/ask`, `/quote`, `/download` (EPUB/MOBI/PDF), bookmarks/ratings/kudos, fic requests board, roadmap voting. Thin client over the REST API (never touches the DB directly); see [Integrations](./docs/src/integrations.md) + the [forum announcement](https://fichub.polarisocial.xyz/forum/board/the-fichub-discord-bot-socrates-style-recommendations-from-a-2.2).
- **Command palette** (`Ctrl+K`/`Cmd+K`) — jump to any page or search the in-app docs from anywhere; **help modal** — every docs section opens in-app with an "Ask the docs" box that answers questions in plain English.
- **Bookmarklet** — one-click download from any story page (grab it from the Download tab).
- **Send-to-Kindle** — email an EPUB of any fic to your Kindle address.
- **Badges & streaks** — achievements (curator milestones, login streaks, reading milestones) issued via the progression engine; consecutive-day reading/download streaks with milestones ("You've read 1.2M words this year").
- **Leaderboards** — top contributors by exp, reading stats, and community activity.
- **Work proposals** — community-driven merge/split proposals for duplicate works, peer-voted.
- **Self-healing scraper telemetry** — scrape-failure tracking with domain health + retry (see `docs/src/self-healing.md`).
- **Nightly QA harness** — `qa/run.js` deterministic local-first checks (API walk 97/97 green) + CI coverage gates.
- **Progression system** (v3, `/features`) — 100 levels, 10 ranks with data-driven thresholds, XP engine (12 event types), ability tree UI with activate/deactivate, rank-gated features (26 gateable items), onboarding banner for new users, level-up notifications, admin feature editor with adoption analytics.
- **Recipe Builder** (`/settings/recipes`) — user-composed recommendation blends (strategy weights, filters, boosts), activate/publish/gallery/install flow, active recipe overrides rec engine per-request. Plugin manifest for shareable recipes/themes/layouts.
- **Trust system** — a Discourse-style 7-level participation axis (TL0 New → TL6 Near-admin) earned from reading + community signals, separate from rank/reputation. Trust sandboxes brand-new accounts (TL0 can read but not flag), gates publishing to the shared galleries (TL2+), weights user reports by reporter trust, and lets staff-granted **Community Moderators (TL5+)** resolve reports — moderation load stays under ~15 min/week. Own dashboard at `/settings/trust`.
- **Community self-moderation** — user reports (`POST /api/reports`) carry the reporter's trust weight and are auto-triaged per target into `pending` / `needs_admin` / `auto_hidden` (overwhelming weight) with a transparent modlog entry on escalation; admins see everything in the weekly digest.
- **Extension marketplace** (`/marketplace`) — one unified gallery for themes, skins, recipes, layouts, and views: trust-gated publishing, per-category slugs, one-click (idempotent) install, 1–5 star ratings with running average, and **remix** (copy any public extension as your own private draft).
- **Entity recommendations** (`GET /api/v0/recommendations/entities`) — similar tags/fandoms, authors, collections, and users, reusing the materialized signal graphs (tag Jaccard, author graph, shared reading-list items, shared positive signals).
- **Theme Design Tokens** (`/settings/theme`) — CSS custom properties with 5 presets (Dark, Light, High Contrast, Sepia, Dyslexia-Friendly), accent/bg/surface/text/muted colour pickers, font selector, density slider, corner radius, reader theme (font/width/height), import/export JSON, live preview. Svelte 5 runes, SSR-safe.
- **Widget-composed home dashboard** (rank 5+) — CSS grid with edit mode, widget registry, default layout factory. Custom saved views (rank 7+): save, pin, delete search views.
- **OPDS catalog** (`/opds/*`) — subscribe to the archive from e-reader apps.
- **Blind Date** — random-fic discovery button (hides title/fandom until reveal).
- **Trending tags & popular fics** — time-decayed trending sort on search + home.
- **Search suggestions + tag autocomplete** — as-you-type tag completion with usage counts, and typo-tolerant suggestions.

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
| GET | `/api/features` | Feature registry: all features with unlock levels + user can/cannot |
| POST | `/api/features/{key}/activate` | Activate a feature for the current user |
| POST | `/api/features/{key}/deactivate` | Deactivate a feature for the current user |
| GET | `/api/recipes` | List user's recipes |
| POST | `/api/recipes` | Create a recipe `{name, description, blend, filters, boost}` |
| PATCH | `/api/recipes/{id}` | Update a recipe |
| DELETE | `/api/recipes/{id}` | Delete a recipe |
| POST | `/api/recipes/{id}/activate` | Set a recipe as active (mutual exclusion) |
| GET | `/api/recipes/active` | Get the user's active recipe |
| GET | `/api/recipes/gallery` | Browse public recipes |
| POST | `/api/recipes/{id}/install` | Install a recipe from the gallery |
| POST | `/api/recipes/{id}/publish` | Publish a recipe to the gallery (TL2+) |
| GET | `/api/me/trust` | My trust level, metrics, next-level hint |
| GET | `/api/admin/trust` | Trust-level distribution (role ≥ 10) |
| PUT | `/api/admin/trust/{id}` | Set a user's trust level (role ≥ 10) |
| GET | `/api/admin/digest` | Weekly moderation digest (role ≥ 10) |
| POST | `/api/reports` | File a report `{target_type, target_id, reason}` (TL1+) |
| GET | `/api/admin/reports` | List reports `?status=open\|resolved\|dismissed\|all` |
| POST | `/api/admin/reports/{id}/resolve` | Resolve/dismiss a report (role ≥ 10 **or** TL5+) |
| GET | `/api/extensions` | Extension marketplace gallery (`?kind=&q=&limit=`) |
| GET | `/api/extensions/kinds` | Supported extension kinds |
| GET | `/api/extensions/{id}` | Fetch one extension (private = owner/admin) |
| POST | `/api/extensions` | Publish an extension (TL2+; `?upsert=1` to update) |
| POST | `/api/extensions/{id}/install` | Install (idempotent) |
| POST | `/api/extensions/{id}/rate` | Rate 1..5 (updates running average) |
| POST | `/api/extensions/{id}/remix` | Copy a public extension into a private draft |
| GET | `/api/v0/recommendations/entities` | Entity recs `?kind=tag\|fandom\|author\|collection\|user&seed=&limit=` |

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

See [`docs/brainstorm-01-system-overview.md`](./docs/brainstorm-01-system-overview.md) for the complete system specification — frontend, backend, database schema, deployment, API endpoints, and technical debt inventory.

## Testing & Coverage

Current state, measured 2026-09-26:

| suite | result |
|---|---|
| unit (`--lib`) | **935 pass, 0 fail** |
| DB-gated integration (56 suites) | **31 pass, 25 fail** |

The integration suites need PostgreSQL (with `pgvector`) and Redis. A scratch
database can be built from `migrations/` alone:

```bash
# Backend (Rust)
cargo test --lib                 # unit tests (fast, no DB)

# DB-gated integration tests: provision first, then run
bash scripts/provision_test_db.sh        # idempotent; prints DATABASE_URL
bash scripts/run_db_suites.sh            # runs all 56 serially, tallies results

cargo llvm-cov --lib             # line coverage report (install: cargo install cargo-llvm-cov)
```

`run_db_suites.sh` uses `--test-threads=1` deliberately: the suites serialise on
their own `Mutex`, delete their own seed rows by unique prefix, and assume they
are alone in the database.

**25 integration suites still fail.** Most are role/level gate assertions left
over from the `role` → `trust_level` refactor, which need per-test judgement
rather than a bulk fix. One is blocked by a known production bug:
`src/services/embedding_dedupe.rs` joins `rec_embeddings.work_id` (`varchar`) to
`works.id` (`integer`), so `candidate_pairs` throws on every call. See
[`docs/specs/missing-reference-data.md`](./docs/specs/missing-reference-data.md)
§5.2 and [`docs/specs/db-gated-suites.md`](./docs/specs/db-gated-suites.md).

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
./deploy.sh            # build on gamingpc + sync binary to /opt + migrate + restart
./deploy.sh --skip-build   # skip build, just sync + migrate + restart
```

**The release binary runs from `/opt/fichub/fichub` (local disk), NOT from the
NFS repo** — since 2026-08-14, to stop SIGBUS crashes when the NFS/mergerfs
mount wedges. `deploy.sh` copies the fresh binary to `/opt/fichub/` before
restarting, so always use it rather than a bare `systemctl restart fichub`.

The `fichub.service` systemd unit runs the release binary on port 8000
(see `PORT` in `/opt/fichub/.env`), serving the API (`/api/*`), docs
(`/docs/*`), and the static SvelteKit frontend (`FRONTEND_DIR` → absolute path
in `/opt/fichub/.env`) on one port. The public domain `fichub.polarisocial.xyz`
is fronted by Cloudflare.

Full layout, env notes, self-heal watchdogs, and the sw.js fix:
see [`docs/DEPLOYMENT.md`](./docs/DEPLOYMENT.md).

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
