<!-- CONSOLIDATED BRAINSTORM FILE — Generated 2026-08-13, updated 2026-08-17 from repo markdown (docs/src user docs excluded). -->

> CURRENT STATE + PLANS: the canonical ROADMAP (shipped / in-flight / prioritized suggestions P1-P7 / test coverage / deployment), session-by-session STATUS log, feature notes + TODO, IDEAS brain-dump (starred = high-confidence picks), NEXT.md (current work item: self-healing scraper), USER-ACTIONS runbook (backlog items the user wants done), and the consensus feature seed (original 60 fichub.net issues grouped into clusters).


---


======================================================================
SOURCE: docs/ROADMAP.md
======================================================================

# FicHub — Consolidated Roadmap

> **Canonical planning doc.** Supersedes the old STATUS.md / TODO.md / NEXT.md
> split: this single file tracks what's shipped, what's in flight, and every
> known suggestion/backlog item — with priorities grounded in the current
> codebase state. Feature specs live in `docs/SPEC-COMMUNITY-PLATFORM.md` +
> `docs/FORUM-API-CONTRACT.md`; deep-dive
> references in the `fichub-development` skill; operator docs in `docs/src/`.

---

## 0. Where we are (one paragraph)

|FicHub is a self-hosted fanfiction archive + community platform (Rust/Axum
backend, SvelteKit SPA, PostgreSQL + pgvector, Redis, Ollama). The feature
surface is complete across all major areas — search (boolean + NL ask),
exports (7 formats), web reader, social (bookmarks/ratings/reviews/
comments/follows), community (Fic Requests board, series/authors pages,
RSS feeds, roadmap consensus), recommendations (pluggable strategy
platform + user recipe builder + entity recommendations), AI features (auto-tagger, translations,
self-healing), admin + transparency (modlog, analytics, anti-bot), and a
**progression system** (100 levels, ability tree, rank-gated features,
widget dashboard, theme design tokens, recipe builder), **trust levels**
(Discourse-style 7-level participation axis with community self-moderation),
and an **extension marketplace** (unified gallery for themes, skins, recipes,
layouts, views). The
`fanfic-scrapers` crate has
**full one-to-one adapter parity with FanFicFare** (107/107 real sites as
native Rust scrapers, login + `is_adult` support) and the Python CLI
dependency has been removed. **Author batch download** now supports AO3 +
XenForo (QQ, SpaceBattles, SV) with SSE progress streaming. The site is
close to small-cohort invite readiness; the remaining content-side gap is
outbound reachability from the host (AO3 404+challenge / FFN 403
Cloudflare), tracked as P1 cookie ingestion.

---

## 1. Shipped (all verified, in main)

### Core platform
- Multi-format export: EPUB, HTML, MOBI, PDF, AZW3, TXT, MD (pure-Rust
  builder + Calibre sidecar for MOBI/PDF/AZW3).
- **Author batch download** ✅ — AO3 + XenForo (QQ, SpaceBattles, SV)
  author pages. SSE progress streaming at `/api/download/author/stream`.
  Single file or ZIP based on work count × format count.
- Web reader `/read/[urlId]`: typography prefs, chapter nav, scroll progress,
  position save, "Next Up" panel (next-in-series via real
  `/api/reader/{url_id}/sequel` → top community suggestion →
  readers-also-bookmarked via `/related`).
- Unified works model: `works` + `fic_info` sources, auto-merge (title+author
  + word-count tolerance), curator merge/split proposals with voting.
- Search: boolean AND/OR/NOT, phrases, fielded search, `main_char_attr`
  ("Dark Harry" semantics = main character + attribute), typo tolerance, tag
  filters, kudos/comment bounds, zero-result feedback.
- **Full-text search over fic bodies** (`/search/body`): `body_text_search`
  tsvector (migration 040) maintained at body-write time + backfill bin;
  quote search with `<mark>` snippets. No scraper-only archive offers this.
- Social: accounts, bookmarks, 5-star ratings + reviews (positive-only public
  surface), threaded comments, follows + updates feed, reading lists +
  shelves, series/authors pages, RSS/Atom feeds.
- Fic Requests prompt board: works-only answers (work id, URL-ingest, or
  `url_id`), fit votes, requester accept, request upvotes, notifications on
  answer/accept, `/candidates` engine suggestions.
- Ask the Archive `/ask`: NL query → v2 search-query string via Ollama LLM
  (same language as the advanced search box — no separate 5-key schema),
  graceful fallback when the model is down. **Ask × Requests loop**: empty
  ask → "Turn this into a request" (prefills `/requests/new?q=`); request
  pages have an "Ask the Archive" box that finds matching in-library fics,
  each one click from becoming an answer.
- Roadmap consensus: MaxDiff/Elo arena over pgvector-embedded feature
  clusters; public leaderboard + controversy scatter.
- **Community forum** (Threadlight-style, F1–F7): categories + topics +
  posts + follows + notifications, read state, FTS search with snippets,
  **moderation points** (grants, reasons, queue, reports-as-signal, curator
  fast-hide), **metamoderation** (anonymized audit, rolling unfair-rate,
  cooldowns), **site-wide leveling** (0–100 levels + exp replace
  role/reputation gates; `exp_events` audit trail; `award_exp` on posts),
  invites + registration applications, user blocks, site-info endpoint, and
  **topic slugs** (`{slugified-title}-{id}`) with canonical board URLs
  `/forum/board/{topic_slug}.{topic_id}` (legacy `/forum/{cat}/{id}` still
  works). The `forum-core` crate (generic over actor/score types) is the
  engine; full API contract in `docs/FORUM-API-CONTRACT.md` +
  `docs/SPEC-COMMUNITY-PLATFORM.md`.
- Anti-bot: honeypot traps, tiered rate limits, Redis shadowban, PoW
  challenge, hourly bot-scorer, `/admin/bots`.

### Scraper platform (fanfic-scrapers crate)
- **Full FanFicFare adapter parity — 107/107 real sites** as native Rust
  scrapers across crates.io releases v0.6.0 → v0.10.0: eFiction family (19),
  eFiction-variant `viewstory.php?sid=` family (26), OTW siblings (5),
  XenForo1+2 (8), StoriesOnline (3), WordPress-novel, plus bespoke ports
  (FicWad, FictionMania, FanFiktion.de, TouchFluffyTail, Fanfics.me,
  FanFicAuthors, FictionHunt, InkBunny, SoFurry + turbo-stream decoder,
  Dokuga, PhoenixSong, StoriesOfArda, Fictionalley, NovelAll, MassEffect2.in,
  ReadonlyMind, UtopiaStories, ASexStories, AnEroticStory, MCStories,
  HentaiFoundry, BDSMLibrary). Only FFF's `test1`-`test4` fixtures (internal
  test adapters, not real sites) have no port.
- **Login support**: `SiteScraper::login`/`requires_login` +
  `SiteCredentials` (form POST, Rails CSRF `_token`, inkbunny token,
  SoFurry csrf meta, eFiction form); FicHub enables `cookie_store` +
  `FANFICSCRAPER_<DOMAIN>_USER/_PASS` env creds.
- **`is_adult` gate**: `SiteCredentials::is_adult` + `with_adult()` unlocks
  adult archives (readonlymind, utopiastories, asexstories, aneroticstory,
  mcstories, hentaifoundry, bdsmlibrary); `FANFICSCRAPER_IS_ADULT` env wiring.
- **FanFicFare Python-CLI dependency removed**: `sites/fanficfare.rs` gated
  behind `fff-fallback` feature (off by default); FicHub registers native
  adapters only. Unsupported URLs return `Unsupported` instead of a 60s CLI
  subprocess.
- **Native-scraper routing fix**: `lookup_authed`/`fetch_chapters` now use
  `find_specific_or_fallback()` (native adapter first, catch-all only as last
  resort). Previously the FF catch-all won every lookup via `find_scraper()`;
  RoyalRoad went from ~62s (CLI, broken metadata) to ~280ms native.
- **Crate parity infra**: FFF baseline v4.60.0 tracked with the update
  workflow in `scrapers/FFF_PARITY.md`; version policy: no 1.0 until it's a
  perfect drop-in replacement (user directive).
- **RoyalRoad markup-drift fix**: title/author/chapter selectors updated for
  current site markup (verified live: "Mother of Learning" / nobody103 /
  109 chapters).
- **User-supplied site credentials**: logged-in users can provide credentials
  for login-requiring sites (fanfics.me, fictionhunt, inkbunny, sofurry,
  dokuga) — OPT-IN consent checkbox, stored encrypted at rest (AES-256-GCM
  keyed by JWT secret), 30-day expiry, passwords never returned by any
  endpoint; download path uses the requesting user's creds via
  `lookup_authed`. Env creds still work as the operator path.

### Resilience & transparency
- **Self-healing M1**: `scrape_failures` + `agent_runs` (migrations 029-030),
  pure-Rust classifier (transient/blocked/structural/systemic), fingerprint +
  debounce, HTML snapshots, `POST /api/admin/heal` diagnose-only. Autonomy
  OFF by default (`AGENT_ENABLED=false`); agent = CommandCode API with
  Ollama fallback.
- **Site-as-cache**: every scraped fic body persisted as sharded versioned
  files under `BODY_CACHE_DIR` (`/public/literature/fichub/bodies`,
  `{url_id[0:2]}/{url_id[2:4]}/...`), exports reuse the cache; **curator
  fixes are peer-voted** (migration 032: propose → vote → apply at quorum ≥2,
  no self-vote).
- **Usage analytics**: `usage_events` (migration 033) + middleware recording
  views vs actions (X-Client-ID, zero-PII); `/admin/analytics` shows unique
  daily/weekly/monthly visitors, active vs view-only users, action timeline,
  search→export conversion.
- **Modlog**: migration 034; every admin/curator action recorded;
  `GET /api/modlog` + `/modlog` page readable by ANY logged-in user.
- **API route-walk** (`qa/api-walk.js`): scriptable API-surface audit that
  parses every route from `src/server.rs`, hits each endpoint, asserts gates
  + flags 5xx/stubs. Caught + fixed two real bugs (reading-stats 500 on
  unknown users; analytics 500 on empty request_log).

### Other notable shipped items
- **Technical-debt wave (2ab2b0f)**: real HTTP status semantics (401/403/400
  with negative body `err` codes; NotFound keeps `-5`), url_id 12→16 hex,
  migrations as explicit deploy step (`fichub migrate` + FICHUB_SKIP_MIGRATIONS),
  single frontend auth store (P6#22), comment stale work_id → 400 not 500.
- **Force.net** added as a XenForo site; native scrapers preferred over the
  FF catch-all; browser-UA on XenForo fetches.
- **Backlog paydowns**: CacheSemaphores bounded (10k cap — leak fix);
  search→export conversion view; Fic Requests URL-ingest answers.
- **Admin UI: translation review + metadata correction** (`/admin/translations`
  draft→approve/reject/edit, `/admin/metadata` correct canonical metadata).
- **Redis dashboard fix**: realtime check uses dedicated health_redis (shared
  conn was parked by bookmark-import BRPOP → false "Redis down").
- **Docs**: junior-dev onboarding chapter (`docs/src/contributing.md`) —
  mdbook rebuilt + `build_docs_map.py` regenerated.
- **Advanced Search v2** (2026-08-25 batch): three-tier Simple/Guided/Power
  search UI; structured QP operators (`@field`, `~` fuzzy, wildcard, `with`/
  `not with`, `romship`/`platship` by tag polarity, ranges + `50k` shorthand);
  `tags.rel_polarity` + `fic_tags.role_confidence` + `works` engagement
  counters (migrations 003–005). Full plan in `docs/plans/search-v2-advanced-search.md`.
- **Saved searches + daily alerts** (P8#11): save/list/run/delete/alert-toggle,
  `saved_search_watcher` nightly re-run + match diff, per-search Atom feed (007).
- **Follow Exclusions**: exclude works/series/fandoms from an author follow;
  feed anti-join + `Exclusions.svelte` (002).
- **Collection submissions + unified curator approvals**: community voting on
  collection item submissions (`collection_submission_votes`, 006) + a single
  `/curator/approvals` queue aggregating collection + proposal + triage items
  with type/status filter.
- **Forum "New category" fix**: `forum.close` i18n key ×6; `.new-category`
  form un-hidden in archive mode with AO3-styled submit.

### Operations
- lfm2.5:8b is the canonical Ollama model (benchmark winner 21.3-21.7 tok/s
  on ThinkCentre; supersedes qwen3.5:9b).
- Repo canonical URL: https://opencommit.eu/MagicZhang/fichub (public mirror).
- Dev + deploy machines, mergerfs recovery, NFS git-objects workaround — see
  the `fichub-development` skill references.
- Ops hardening shipped: uptime probe (Hermes cron), body-cache backup
  routine, secrets hygiene.

### FicHub v3 — progressive customization platform (2026-08-17)
- **100-level/10-rank progression system** with XP engine, dual-axis identity
  (rank + trust), 26 gateable features with data-driven thresholds.
- **Ability Tree UI** (`/features`): browse, activate, deactivate features;
  onboarding banner for new users; level-up notifications.
- **Widget-composed home dashboard** (rank 5+): CSS grid with edit mode,
  widget registry, default layout factory.
- **Theme/layout customization** (rank 1+/3+): user prefs API, nav derivation
  from features, widget registry.
- **Custom saved views** (rank 7+): save, pin, delete search views.
- **Admin feature editor** with adoption analytics.
- **Extension platform (v3.1)**: Recipe Builder (user-composed rec blends,
  active recipe overrides rec engine per-request), Plugin Manifest (unified
  shareable objects table), Theme Design Tokens (CSS custom properties +
  5 presets + import/export + live preview).
- DB: migrations 052 (progression) + 053 (recipes/extensions).
- Spec: `docs/v3-customization-spec.md`. Deferred items in roadmap P9.
- All 7 phases (P1–P7) shipped and deployed.

### 2026-08-23 pass — site-quality (docs, consensus, requests, OPDS, symbols, search chips)
- **Curator Consensus Hub** — unified filterable feed at `/curator/consensus`
  backed by `GET /api/curator/consensus?type=&status=&sort=&q=` (public-read
  for logged-in users, modlog-style; admin dashboard links out instead of
  duplicating the panel). Filter chips + search + status/sort deep-linking.
  (commits f7a0f1e, 66ac13e).
- **Consensus auth fix** — `/api/curator/consensus` made readable by any
  logged-in user (was curator-only 403); write/vote paths unchanged. Matches
  `/api/modlog` transparency precedent (66ac13e).
- **Docs hub fix** — `/docs` no longer links to Forgejo HTML pages; left
  column lists in-app mdbook entries from `GET /api/docs/list`, right pane
  renders `GET /api/docs/view?name=…` markdown with `renderMarkdown` +
  archive typography; `?doc=` deep-links; Forgejo raw fallback. mdbook rebuilt
  + `build_docs_map.py` regenerated (581c751).
- **Fic Requests typed answers (backend)** — `fic_request_answers.search_query`
  column + `answer_kind` work|search|llm (migration 062, commit 6652aef);
  `POST /api/requests/{id}/answers` accepts `search_query`, list/get include
  `answer_kind`; LLM auto-answer via existing Ollama ask pipeline attributed
  \"FicHub Archivist\" with graceful fallback when Ollama is down.
- **Request AnswerRow UI** — `AnswerRow.svelte` renders work/search/llm
  answers (search answers as \"Try this search\" cards with live previews,
  llm answers as archivist notes); `frontend/src/routes/requests/[id]/page.test.ts`
  covers the three kinds (in this working tree, to be committed).
- **Search-history chips** — migration 063 adds `search_queries.user_id`
  (nullable FK → users), `insert_search_query`/`recent_search_chips` helpers,
  and `GET /api/search/history/chips` returning top-8 frequent recent queries
  per user; frontend swaps trending chips for personal chips when logged in.
  Cheap P6#23 win — no ML, one GROUP BY (063 in this working tree).
- **OPDS RWPM manifest** — `GET /opds/manifest` returns a minimal Readium Web
  Publication Manifest (`@context`, `metadata`, `links`, `navigation` for 6
  sections) and root Atom feed adds `rel=\"alternate\" type=\"application/webpubmanifest+json\"`
  pointing at it; `src/routes/opds/manifest.rs` + route wired in `server.rs`
  (in this working tree; acquisition-link hash fix still in progress).
- **Symbol squares (AO3-style)** — `symbols.ts` pure logic + `SymbolSquares.svelte`
  + `SymbolsGuideModal.svelte` implementing the 4-square strip (Rating G/T/M/E,
  Category F/F·F/M·Gen·M/M·Multi·Other, Warnings ?/!/🌐/blank, Completion ⏹/✓/blank);
  `cursor: help` + native `title` tooltips, click opens the \"Symbols we use on
  the Archive\" guide; mounted on `WorkBlurb` and the work page with existing
  textual tags preserved underneath (in this working tree).
- **Pass-4 housekeeping** — missing `embedding_recs` module tracked (fresh-clone
  fix), junk dirs gitignored, 8 vitest unhandled rejections in CommentSection
  mocks fixed (5a7d1a1, a01556e); plan file `.hermes/plans/2026-08-23_103420-site-quality-pass4.md` (Parts A–G) drives the remaining forum/OPDS wiring.

### 2026-08-27 pass — trust system, extension marketplace, entity recommendations
- **Trust system (Phase 4)** — Discourse-style 7-level participation axis
  (TL0 New → TL1 Basic → TL2 Member → TL3 Regular → TL4 Elder → TL5 Community
  Moderator → TL6 Near-admin) as a second axis beside rank/reputation
  (migration `013_trust_levels.sql`: `users.trust_level`, `tl0_until`,
  `tl_metrics`). Promotions run from `src/bin/trust_promote.rs` off cached
  engagement metrics; TL5/6 are staff-designated via the weekly digest.
  Endpoints: `GET /api/me/trust` (metrics + next-level hint),
  `GET /api/admin/trust`, `PUT /api/admin/trust/{id}`, `GET /api/admin/digest`.
  Gates: TL0 cannot flag; publishing to shared galleries needs TL2
  (`services::trust::PUBLISH_MIN_TRUST` — recipes, skins, extensions);
  report resolution opens to TL5+ (`RESOLVE_MIN_TRUST`).
- **Report weighting + auto-triage** — reports now carry the reporter's
  trust-derived weight (`flag_weight`: TL1/2 = 1, TL3 = 2, TL4 = 3,
  TL5/6 = 5, TL0 blocked) and every new flag re-aggregates open reports for
  the target into `pending` / `needs_admin` (weight ≥ 3) / `auto_hidden`
  (weight ≥ 6, with a `report_auto_hide` modlog line). `resolve` records
  `resolved_by` / `resolved_at`.
- **Extension marketplace (Phase 2)** — `migrations/014_extensions.sql`
  upgrades the baseline `extensions` table in place (wider kind set adding
  `skin`/`saved_search`/`profile`, per-category unique slugs,
  `is_public`/`is_verified`, first-class `installs`/`rating`, id sequence
  restored, `extension_installs` + `extension_ratings`). New
  `services/extensions.rs` + `routes/extensions.rs` mounted at
  `/api/extensions`: publish (TL2+, `?upsert=1`), gallery (`?kind=&q=&limit=`),
  idempotent install, 1–5 ratings with running average, remix into a private
  draft. Frontend `/marketplace` gallery page + nav entry.
- **Entity recommendations (Phase 3)** —
  `GET /api/v0/recommendations/entities?kind=tag|fandom|author|collection|user&seed=…`
  returns similar tags/fandoms (Jaccard over `fic_tags`), authors
  (`rec_author_graph`), collections (shared `reading_list_items` works) and
  users (shared positive signals) — a pure query layer over the materialized
  graphs, no new training data.
- **Trust frontend (Phase 5)** — `/settings/trust` dashboard (metrics,
  next-level hint, publish/resolve capability gates), `TrustBadge.svelte`
  component, "🛡 Trust" link in the auth bar.
- **Hybrid learning fix** — `ctr_weights` no longer lets an unmeasured
  strategy outrank a strong performer: the no-data component was keeping its
  absolute default share (0.4) while measured CTRs sit on a ~0.3 scale;
  unmeasured components are now imputed at the observed mean CTR. Regression
  tests added; 760 lib tests green. Frontend: 709 vitest passing (8
  pre-existing forum failures untouched by this pass).


---

## 2. In flight / current focus

- **2026-08-23 site-quality pass (Parts A–G)**: ✅ SHIPPED — all tasks landed,
  deployed to ThinkCentre, QA 257/0. Includes OPDS hash links + RWPM manifest,
  consensus hub, typed request answers + ask shim, history chips, symbol
  squares, Marginalia (+ curator page), emoji reactions picker, sequential
  Next Up, recs opt-in toggle, endpoint analytics, ask caching.
  Forward plan: `docs/NEXT-STEPS.md`.

- **Forum Depth Phase 2 (Threadlight F6–F7 completion, 2026-08-24)**: SHIPPED — read-state badges (LEFT JOIN forum_read_state, unread/last_read_post_id on list/detail, debounced POST .../read, archive new chip), pin/lock affordances (forum_topics.status, curator pin/lock toggles, locked 403, pinned ordering), metamod queue polish (filterable unreviewed/reviewed + verdict, pagination next_cursor, 409 single verdict, anonymized grant detail, unfair_rate/cooldown_until surfacing), per-author Mod drawer (GET /api/forum/moderation/user/{id}/grants). Commit cd61b1a; spec specs/001-forum-depth-phase2/ (F1 read-state P1, F2 pin/lock P1, F3 metamod P2, F4 drawer P3); cargo check clean, frontend build ok, vitest 709/715 (1 pre-existing supported_convert_format failure, 5 social.ts status unhandled rejections).
- **Invite readiness for a small cohort**: the F7 invite system ships with
  the leveling milestone (invite codes + registration applications +
  admin invite management). Onboarding flow + seed content + a "new member"
  landing remain.
- **Self-healing M2**: ✅ SHIPPED — on-the-fly scraper creation by the
  agent on structural failure, safety-gated (`AGENT_USE_ON_FLY=true`,
  migrations 050-051, `src/heal/extract.rs`). Autonomy OFF by default.
- **Forum F8**: ✅ SHIPPED — `forum-core` crate published to crates.io
  (v0.1.0), public repo at opencommit.eu (MagicZhang/forum-core).
|- **v3 progression + extension platform**: ✅ SHIPPED — 100-level/10-rank
  progression system, ability tree UI, widget dashboard, recipe builder,
  theme design tokens, plugin manifest (migrations 052-053, commits
  41bd2cd, 6af3254, 18b7354). Extension marketplace (unified gallery,
  trust-gated publish, remix) shipped 2026-08-27 (commit 63e6c66,
  migration 014).
- **FFN metadata fallback**: ✅ SHIPPED — `fichub_net.rs` async client
  for `GET fichub.net/api/v0/epub`, wired into `ffnet.rs` for CF
  challenges and `words==0` parses (commit 99a088a).
- **Registration honeypot fix**: ✅ SHIPPED — frontend sends
  `form_opened_at` + `website:''` to satisfy the timing trap
  (commit b038705).
|- **Progression endpoint SQL fix**: ✅ SHIPPED — `COALESCE(smallint, 0)`
  type mismatch fixed (commit 59454cf).
||- **Reputation XP rewards v3 — A1 idle-tick dedup + B3 admin-award endpoint**: ✅ SHIPPED —
  `POST /api/xp/idle-tick` now deduplicates via a per-user hourly time-bucket
  (`%Y-%m-%dT%H`) as `source_ref`, so the engine's `cooldown_seconds=3600` (seed
  by migration `011_idle_tick_cooldown.sql`) suppresses rapid-repeat farming
  (max 24 awards/day = 48 XP). `POST /api/admin/reputation/award` is live:
  admin-gated (`role >= 10`), allowlists only `Admin:`-prefixed
  `xp_source_defs` rows (`scraper_fix_merged`, `code_pr_core`, `marathon_writer`,
  etc. — 9 rows seeded by `011`), all grants modlogged. Deploy: binary md5 `6f1165a0`
  on thinkcentre, service active, health 200, migration 011 applied (versions 1-8+10+11).
  737 lib tests green. Plan: `docs/plans/reputation-xp-rewards-v3.md` +
  `docs/plans/reputation-xp-remaining-work.md`.
|- **Trust system** ✅ SHIPPED 2026-08-27 (commit 63e6c66) — Discourse-style 7-level
  participation axis (TL0 New → TL6 Near-admin), separate from XP/level/rank. TL0
  sandboxes brand-new accounts (read but not flag); TL2+ gates publishing to shared
  galleries (recipes/skins/extensions); TL5+ Community Moderators resolve reports;
  `role >= 10` remains sole authority for hide/ban/admin actions. Promotion via
  `src/bin/trust_promote.rs` reading engagement signals into cached `tl_metrics`;
  frontend `/settings/trust`, migrations `013_trust_levels.sql`. Reports
  (`POST /api/reports`) auto-triaged by reporter trust weight into `pending` /
  `needs_admin` / `auto_hidden`; weekly moderation digest at `/api/admin/digest`.
  Trust-levels-rework plan doc (`docs/plans/trust-levels-rework.md`) status banner
  updated to SHIPPED.
|- **Badge-wiring fix + codebase audit pass** (commit 591776c) — `POST
  /api/admin/reputation/award` (B3 handler) now calls `check_and_award_badges`,
  so Admin: sources (`marathon_writer` etc.) issue actual `user_badges` records.
  Tier-1 dead code removed: `SuggestionsTab.svelte`, `client-legacy.ts` + its test,
  `RouteHarness` test fixture. Daily-quest gamification loop scrapped (`quests.rs`,
  `user_daily_progress`, `assign_quests` cron, `/quests` route) — duplication of
  reading-history + streak system; kept `/api/reading/*`, `/api/users/{id}/streak`,
  `/api/users/{id}/reading-stats`. Audit: `docs/plans/codebase-audit-low-value-features.md`.
  Deploy: thinkcentre, health 200, md5 `698979b8`.
|- **Extension marketplace** ✅ SHIPPED 2026-08-27 (commit 63e6c66) — unified
  `/marketplace` gallery for themes, skins, recipes, layouts, views: trust-gated
  publishing (TL2+), per-category slugs, one-click idempotent install, 1–5 star
  ratings with running average, remix (copy public extension as private draft).
  `migrations/014_extensions.sql` upgrades the `extensions` table in place; backend
  `services/extensions.rs` + `routes/extensions.rs` at `/api/extensions`; publishing
  gated the same way across recipes/skins/extensions via
  `services::trust::PUBLISH_MIN_TRUST`. Spec: `docs/v3-customization-spec.md` P8
  section updated.
|- **Entity recommendations** ✅ SHIPPED 2026-08-27 (commit 63e6c66) —
  `GET /api/v0/recommendations/entities?kind=tag|fandom|author|collection|user`,
  reusing materialized signal graphs (tag Jaccard, author graph, shared reading-list
  items, shared positive signals). Added to README API table + Feature Highlights.
|- **Fandom landing pages**: ✅ SHIPPED — `/fandoms` landing page + `/api/fandoms` endpoint (commit 21a0440).
|- **Bulk admin actions**: ✅ SHIPPED — bulk operations for fics/authors (commit b017681).
|- **main_char_attr removal**: ✅ SHIPPED — `main_char_attr` column removed; search is now fully tag-based (commit b017681). Search system overhaul now possible.

---

## 3. Suggestions & backlog (prioritized)

> Items are ordered by leverage; each notes what's already true vs what's
> needed.

### P1 — Content bottleneck first

1. **User-supplied cookie ingestion for AO3/FFN.** The host is blocked
   (AO3 404+challenge / FFN 403 Cloudflare from the server; force.net's
   bot-guard beat even Playwright — needs a real human click). The native
   adapter set covers all 107 FFF sites, but the *host's outbound* blocking
   still limits what the archive can ingest from AO3/FFN. Pragmatic fix: a
   "cookie import" flow — user opens the blocked page in their own browser
   once, pastes cf_clearance / session cookie, scraper reuses it. Low-risk
   now: body cache + curator peer-voted fixes make failed scrapes
   recoverable. Without this the archive only grows from reachable hosts.
   **Mitigation in place:** `src/export/fallback.rs` already calls
   `https://fichub.net/api/epub?q=<url>` as a scraping fallback for AO3/FFN
   when the native adapter can't reach the host, so AO3/FFN export errors
   degrade to a secondary scrape rather than a hard failure. Cookie import
   remains the P1 product decision for native ingestion.
2. **Scriptable API-surface e2e (route-walk) in CI.** Boot the server,
   enumerate the router, hit every endpoint with a fixture admin token,
   assert 2xx (or expected 4xx), flag empty/stub responses. (The script
   exists; make it a CI gate.)
3. ~~**Main-char/ship score fixing** (admin UI + backfill IF a per-fic score
   column exists — today `main_char_attr` lives on `search_queries` as a
   FILTER, not a stored per-fic score; see `admin_fix_tag_score`).~~ **DONE (b017681)** — `main_char_attr` column removed; search is now fully tag-based.
4. ~~**Search frontend UI control for `main_char_attr`** ("Main character" +
   "attribute" pickers) — the data-quality lever behind AO3-parity niche
   searches.~~ **N/A — main_char_attr removed (b017681); search system overhaul now possible.**

### P2 — Ops debt

5. **CI/CD + uptime probe.** Internal Forgejo is unreachable; minimum viable
   = external uptime check (UptimeRobot or Hermes cron) alerting to Telegram
   when health flakes or the nightly QA harness (`fichub-nightly-qa`) finds
   new bugs.
6. **Secrets hygiene.** Move the DB password out of `.env` into a 600-perm
   secrets file via systemd EnvironmentFile.
7. **Backup the body cache + EPUBs.** `/public/literature/fichub/bodies` is
   the most valuable data asset — include it in any maintenance routine.

### P3 — Make the rec platform earn its keep

8. **Shadow-run `decay` + `embeddings`** (both built, cheap; golden test
   protects parity). Read `rec_impressions` after a week, promote the winner.
9. **`sequential` Markov in reader "Next Up"** — the reader's Next Up panel
   is now backed by real `/api/reader/{url_id}/sequel` (next-in-series) +
   `/related` (co-occurrence) endpoints; wiring the `sequential` Markov
   strategy's `rank_targets` as an additional Next Up source is the
   remaining step.
10. **Personalized recs by default** — upgrade P3#10 (was deferred). The
    engines (decay, embeddings, MF, bandit) + shadow mode + `/api/recommendations/personal`
    endpoint all exist. Wire the frontend to call `/personal` by default
    when logged in; fall back to generic `/api/recommendations` when logged
    out or when user opts out via a `recs.personalized` pref in `user_prefs`.
    No audience dependency — even a single user benefits from reading-history-
    based recs. Ships with a "Personalized recommendations" toggle in
    settings (default ON for logged-in users). See plan:
    `.hermes/plans/2026-08-19-personalized-recs.md`.

### P4 — Differentiators

11. **Full-text search over fic bodies — SHIPPED 2026-08-12** (61cf1fa,
    migration 040): `body_text_search` tsvector + GIN index, `GET
    /api/search/body` (boolean parser → ts_rank + `<mark>` snippets),
    maintained at body-write time + `backfill_body_search` bin, frontend
    `/search/body` page. No scraper-only archive offers this.
12. **Analytics feedback loop** — `usage_events` + `search_queries`
    recorded; search-to-export conversion view done; keep using zero-result
    queries to prioritize ingestion + auto-tagger.
13. **Per-endpoint usage breakdown** — `usage_events` already records every
    request's `path` + `event_type` (view/action) with an anonymous
    `client_id`; add a derived endpoint-usage view/dashboard ("times each
    endpoint is used": top paths by count, daily/weekly, view vs action,
    search vs export vs reader vs social) so admins see which features are
    actually used. Cheap: one GROUP BY query over the existing table.
13a. **Scrape deduplication pipeline** — before a scrape is processed, deduplicate by:
    (a) URL identity first (`source_url` on `fic_info`), then (b) content hash
    (`body_md5` already stored), then (c) title+author fuzzy match (Levenshtein
    ≤5% of title length). When a content-hash match is found across different
    URLs, compute % similarity on the fly (Jaccard over body word-set via SQL)
    and surface it to curators in the moderation queue: if similarity >
    configurable threshold (default 90%), require curator approval to merge or
    disprove the merge. Eliminates the manual "is this a dupe?" triage for
    90%+ of ingest. **NEW — P4**
13b. **Remove XP/levels/quests system** — trust levels (TL0–TL6, shipped 2026-08-27)
    are the sole promotion axis. Remove the parallel XP/rank/levels/quests
    system (levels 1–100, `award_exp`, `/quests` route, `exp_events` table,
    daily streaks, level-gated features). Trust level replaces ALL role/lvl
    gates in frontend and backend. **NEW — P4 (architectural consolidation)**
13c. **Trust levels replace moderation points on forum** — replace the
    per-category `mod_points` + level-based curator gate with the universal
    trust-level axis: TL5+ Community Moderators resolve reports; TL2+ can
    publish to shared galleries; TL0 sandboxes new accounts. Forum
    moderation actions (pin/lock, metamod verdicts) gated by TL, not by
    ad-hoc `mod_points`. **NEW — P4**
13d. **Trust-level voting on modlog** — allow fair/unfair/unsure votes on
    modlog decisions by any user with trust level > L2 (TL3+). Weight
    curator consensus by voter trust tier (TL5 votes count 3×, TL3 1×).
    Surface agreement % alongside existing verdict counts. **NEW — P4**

### P5 — Curator/admin UI backlog (APIs mostly exist)

14. **Auto-tag review UI** (draft → approve).
15. **Manual fic approval UI** (`/api/admin/moderation/queue` exists).
16. **Metadata correction** (title/author/status/description on works).
17. **Report handling** (Rust-side, beyond the Go schema).
18. **Translation post-edit** (draft → human).
19. **Rating/warning verification** (keep no_warnings search honest).

### P6 — Cheap UX wins

20. ~~**Loading skeletons on all pages**; extend PWA offline beyond the reader.~~ **SHIPPED (partial)** — skeleton states live on `/my-works`, `/collections`, `/collections/[id]`, `/authors` (archive-skeleton-list + skeleton-blurb); remaining pages are follow-ups, not the whole app.
21. **Ask-the-Archive response caching** for common queries + a cron
    keep-warm (cold ~17.6s; `keep_alive: 30m` can lapse).
22. ~~Verify the known mismatch: search suggestions read
    `localStorage('fichub_token')` directly while the auth store may differ —
    personalization could be silently disabled.~~ **DONE (2ab2b0f)** — single
    frontend auth source; suggestions/export use `auth` store + `authHeaders()`.
23. ~~**Search-history chips** — customize the search page's suggested-chip
    row from the user's own search history. Cheap (compute) approach: reuse
    the existing `search_queries` table (add an optional `user_id` column)
    and surface each logged-in user's top-N most frequent recent queries as
    clickable chips (e.g. "dark harry", "dramione", "completed"); no ML or
    embeddings needed — a `GROUP BY query ORDER BY count DESC` per user.
    Guest fallback: the existing global trending chips.~~ **DONE (063)** — `search_queries.user_id` + `GET /api/search/history/chips` (top-8 per user) shipped 2026-08-23 (working tree).
26. **User skins / work skins** — user-composed CSS themes applied to works.
   **SHIPPED** — `skins` table + `user_skins` join in `001_initial.sql`;
   `/settings/theme` for app design tokens, skin-style custom CSS per work
   is the remaining surface (part of the v3.1 extension marketplace skin kind).

### P7 — Bigger bets

24. **Invite the small cohort**: invite codes + registration applications
    ship with F7; the remaining piece is the onboarding flow, seed content
    from body-cache favorites, and a "new member" landing. The governance
    layer (modlog, consensus, transparency) is ready; the cohort is the
    missing piece.
25. **Verify ask-the-archive live translation with lfm2.5:8b** (previous live
    test showed `main_char_attr` null with qwen; fallback plain search
    worked).

### P8 — 50-feature fit audit (batch backlog)

> 2026-08-14: a 50-item feature list was reviewed against this roadmap. Every
> item is captured below. Items already tracked keep their existing P-number;
> items that only lived in the IDEAS/TODO brain-dump are now prioritized here;
> genuinely new items get a P-number in this section. Priority leans on the
> user's "cheapest wins" note: #41/#43/#45/#50 are single queries over tables
> that already exist; #27 is recommender plumbing already present; #4 is the
> stated P1.
>
> 2026-08-15: **OTW Archive fit audit** (second audit below) — surveyed
> github.com/otwcode/otwarchive (cloned to `~/.cache/otwarchive`, symlinked at
> `~/code/rust/otwarchive`) and added the missing features that fit FicHub as
> #52–#72; **brainstorm review** added 22 more as #73–#94.
>
> 2026-08-18: **User-facing priority batch** — 20 features flagged as most
> attractive to end users (emotional hooks, identity, retention, social proof).
> These are the features users would screenshot, share, and return for. Items
> marked **UF** below:

**Download & ingestion**

| # | Feature | Status |
|---|---|---|
| 1 | Batch series/author download (one zip of EPUBs) | ✅ **SHIPPED** — AO3 + XenForo (QQ/SB/SV) author batch download; SSE progress streaming; single file or ZIP based on work count × format count |
| 2 | Download queue with progress (cancel/resume) | 📋 in IDEAS §1; **P8 backlog** |
| 3 | Customizable EPUB (cover, font, notes, chapter toggles) | 📋 in IDEAS §1; **P8 backlog** |
| 4 | Cookie ingestion for AO3/FFN | ✅ already tracked — **P1#1** + USER-ACTIONS #4 |
| 5 | Wayback Machine fallback | 📋 in IDEAS §1 + TODO §Infra + consensus #11; **P8 backlog** |
| 6 | Update watcher (cron re-check ongoing fics → notify + refresh) | **NEW — P8** |
| 7 | Cover image API (`/api` covers for frontends/userscripts) | 📋 consensus seed #8; **P8 backlog** |
| 8 | CORS on API (`Access-Control-Allow-Origin`) | 📋 consensus seed #7; **P8 backlog** |
| 9 | Instance import tool (fichub.net / other FicHub) | 📋 in IDEAS §7; **P8 backlog** |
| 10 | External fanwork aggregation & link-only works | 📋 in IDEAS §1; **P8 backlog** — link-only works + related fanworks (podfics, art, video) |

**Search & discovery**

| # | Feature | Status |
|---|---|---|
| 10 | Search-history chips | ✅ **DONE (063)** — `GET /api/search/history/chips` — was **P6#23** |
| 11 | Saved searches (notify on new match) | ✅ **SHIPPED (007)** — save/list/run/delete/alert-toggle; nightly `saved_search_watcher` re-run + match diff; per-search Atom feed. Timer wired via `scripts/install_systemd_timers.sh` (fichub-saved-search, nightly 02:00) |
| 12 | Main-character/attribute picker UI | ✅ already tracked — **P1#4** |
| 13 | Public community shelves page | 📋 in IDEAS §3; **P8 backlog** |
| 14 | Trope-combo builder (intersect two attributes on `/tropes`) | **NEW — P8** |
| 15 | Quote of the day (from body FTS index) | **NEW — P8** |
| 16 | Cross-fandom bridge (co-bookmark movement) | **NEW — P8** |
| 17 | Mood preset chips (angsty/fluffy/slow-burn facets) | **NEW — P8** |
| 18 | Tag-wrangling assistant (admin synonym/merge suggestions) | **NEW — P8** |
| 19 | "Also bookmarked" anchors in search results | **NEW — P8** |

**Reader & reading experience**

| # | Feature | Status |
|---|---|---|
| 20 | Chapter highlights/annotations (private quotes & notes) | 📋 in TODO §Reader + IDEAS §2; **P8 backlog** |
| 21 | Reading stats dashboard (words, finished, shelf %, monthly recap) | 📋 in IDEAS §4; **P8 UF** — "Spotify Wrapped" for reading |
| 22 | Estimated reading time on cards + reader | **P8 UF** — reduces decision cost; "12 min read" on every card |
| 23 | Text-to-speech mode in web reader | **NEW — P8** |
| 24 | Full app-shell offline PWA | ✅ already tracked — **P6#20** |
| 25 | Chapter-release countdown (cadence of followed in-progress) | **P8 UF** — creates anticipation + return visits |
| 26 | Reader keyboard shortcuts (j/k nav, font keys, bookmark) | **P8 UF** — power users evangelize this; vim of fanfic |
| 132 | **Bookmark/history export** — one-click download of all bookmarks, reading history, ratings, and reviews as CSV/JSON. Reduces lock-in anxiety; users invest more when they know they can leave. Data lives in `bookmarks`, `user_ratings`, `reading_history` tables; export is a SELECT + serialize. | **P8 UF** — reduces lock-in anxiety |

**Recommendations & personalization**

| # | Feature | Status |
|---|---|---|
| 27 | Recommendation explainability ("because you bookmarked X") | 📋 in TODO §Consensus UI; **P8 UF** — users trust recs more when they understand WHY |
| 28 | Rec feedback loop (👍/👎 → bandit weights) | **NEW — P8** |
| 29 | Taste profile page (tag affinities + cluster) | **P8 UF** — identity; users love seeing their taste reflected |
| 30 | "People like you also loved" (clusters strategy on home) | **P8 UF** — social proof + personalization combo |
| 31 | Mood slot ("give me something dark tonight") | **P8 UF** — emotional hook; makes site feel alive |
| 32 | Cold-start onboarding picker (3 picks to seed recs) | **P8 UF** — first impression; "Pick 3 fics you love" |
| 33 | Strategy shadow-run dashboard (per-strategy engagement) | ✅ shadow-run tracked — **P3#8**; dashboard half **P8** |

**Community & engagement**

| # | Feature | Status |
|---|---|---|
| 34 | Reading challenges (monthly goals → badges) | **P8 UF** — habit loops; "Read 3 days in a row! +50 XP" |
| 35 | Comment reactions — dropdown emoji picker with search bar, commonly-used + category sections (not fixed handful always visible); reactions appear on the right of the selector and wrap to next line when many, selector stays anchored (see `docs/REACTION-EMOJI-DESIGN.md`, positive-only policy) | **P8 — spec refined 2026-08-23** |
| 36 | Author claim & verification (badge + metadata fixes) | **NEW — P8** |
| 37 | Fic gifting (rec + personal note to a user) | **P8 UF** — social hook; drives new user acquisition |
| 38 | Reading clubs (private groups + shelf + thread) | **NEW — P8** |
| 39 | Weekly digest (followed fandoms/tags, in-app or email) | **P8 UF** — #1 retention tool; brings people back |
| 40 | Engagement showcase (kudos/comments/bookmark counts, positive-only) | **NEW — P8** |
| 51 | Ghost/absorbed accounts — free usernames of inactive users (re-attribute activity to a site-wide "anonymous" account; threshold config, admin review, never sweep admins) | 📋 FORUM-BRAINSTORM B16; **P8 backlog** |
| 133 | **Emoji reactions redesign** — replace the fixed handful of always-visible reactions with a dropdown picker anchored in place: click opens a search bar with a commonly-used section + categorized emoji grid (like modern pickers); existing reactions render to the right of the selector and wrap to the next line when many exist; selector stays anchored. | 📋 `docs/REACTION-EMOJI-DESIGN.md` + `051_forum_reactions.sql`; **P8 backlog** — Community & engagement |
| 41 | **Tropes bounties** — users stake a small amount of XP or a "bounty pool" (site-wide or per-fandom) to incentivize fics written for specific trope combinations or settings that are underserved. Example: "40 XP bounty on Dark Harry x Draco in a Coffee Shop AU" — the first fic to satisfy the criteria (verified by tags + optional curator nod) wins the pool. Variants: author posts "I'll write X trope if someone stakes 50 XP"; community votes up bounties; time-limited bounties that expire. Pairs with P8#34 reading challenges + P8#29 taste profile (taste affinities suggest which tropes need writers). | **NEW — P8 UF** — positive-only incentive; gamifies content creation, not just consumption |

**OTW Archive fit audit (2026-08-15)** — surveyed `otwarchive` (github.com/otwcode/otwarchive,
cloned to `~/.cache/otwarchive`, symlinked at `~/code/rust/otwarchive`) against FicHub's
existing features. Items below are genuinely missing and fit FicHub's direction
(positive-only, self-hosted, forum-first). Already covered by FicHub and NOT listed:
bookmarks/shelves + want-to-read (Reading model), series + authors + merges (pseud merging),
ratings/reviews (5-star, positive-only), follows (work/author/user), comments, modlog,
tag wrangling assistant, body FTS, quests, collections-as-shelves.

| # | Feature | Status |
|---|---|---|
| 52 | **Kudos** — one-click anonymous appreciation (no text, one per user per work; guest counts separate). Distinct from 5-star reviews. Positive-only, zero effort, the single cheapest engagement win OTW proves. | ✅ **SHIPPED 2026-08-15** (migration 048, `/api/kudos/{work_id}`, work-page button, search `min_kudos` + sort repointed to real kudos) |
| 53 | **Pseuds** (multiple identities per user) — a user writes under several names, each with its own author page + bookmarks; all share one account. Pairs with the existing "propose merging a pseudonym" path. | ✅ **SHIPPED** — `pseuds` table in `001_initial.sql` + `src/routes/pseuds.rs` (list/create/update/delete/invite); author pages render works by pseud. |
| 54 | **Orphaning** — user-initiated "orphan my work/pseud/series" → content moves to a site `orphan_account`, username freed, no data deleted. The voluntary version of B16's forced absorption; same ghost-account machinery. | 📋 otwarchive `orphans_controller.rb` + `User.orphan_account`; **P8 backlog** — pairs with FORUM-BRAINSTORM B16 |
| 55 | **Subscriptions** — per-work / per-author / per-series subscribe (distinct from follow: opt-in email/RSS digest, works + series + user). FicHub has follows but no email/RSS subscription channel. | 📋 otwarchive `subscription.rb`; **P8 backlog** — pairs with RSS feeds |
| 56 | **Mark-for-later** — "Want to Read" shelf exists (P8#24 has toread via `want_to_read`); OTW's dedicated "Mark for Later" home page is the same data, separate surface. Mostly covered; low value to re-add. | ✅ covered by `want_to_read` shelf |
| 57 | **Gift exchanges / fests / challenges** — structured prompt-meme + gift-exchange: signup with offers/requests, tag-set restrictions, matching, anonymous reveals. The richest community feature OTW has; FicHub's consensus/roadmap machinery is adjacent but distinct. Heaviest lift on this list. | 📋 otwarchive `challenge_models/` + `tagset_models/`; **P8 backlog** — big |
| 58 | **User skins + work skins** (custom CSS) — OTW lets users theme the whole site and authors theme individual works. Fits self-hosted users who want their own look; work skins especially (an author customizes how their fic renders). | ✅ **SHIPPED** — `skins` + `user_skins` tables in `001_initial.sql`; `/settings/theme` for app design tokens; skin-kind in the v3.1 extension marketplace (`/marketplace`) covers custom CSS per extension. Work-level skin picker is the follow-up. |
| 59 | **User-level mute/block beyond forum** — OTW has global `block.rb` + `mute.rb` (hide a user's content site-wide, not just in the forum). FicHub's user blocks are forum-only today. | ✅ **SHIPPED (partial)** — `blocked_users` table + `POST/DELETE/GET /api/blocks` (block/unblock/list user-level) shipped; forum-only blocks existed before. Site-wide content hide per blocked user is the follow-up. |
| 60 | **Username/email history** (admin) — track past usernames + emails per user for identity forensics (ban evasion, sockpuppet rings). Cheap: a `user_past_usernames` table mirroring OTW's. | 📋 otwarchive `user_past_username.rb` + `user_past_email.rb`; **P8 backlog** |
| 61 | **Reading history + stats** — per-user reading history (what/when you read), with aggregate reading stats dashboard. P8#21 (reading stats dashboard) is adjacent; add the raw history view. | 📋 otwarchive `reading.rb`; **P8 backlog** — extends P8#21 |
| 62 | **Abuse report queue UI** — OTW's abuse reports + admin queue. FicHub has moderation/flagging but no dedicated abuse-report admin surface. | 📋 otwarchive `abuse_report.rb`; **P8 backlog** — pairs with P5#17 report handling |
| 63 | **Moderated works / manual approval** — OTW lets admins gate works behind review (moderated_work.rb). FicHub has manual fic approval (P5#15) + translation review; this is the general "post goes to queue" gate. | 📋 otwarchive `moderated_work.rb`; **P8 backlog** — extends P5#15 |
| 64 | **Fannish next-of-kin** — designate a trusted user who can take over/delete your works if you die/disappear. Distinctively OTW; cheap data model, meaningful trust feature for a community archive. | 📋 otwarchive `fannish_next_of_kin.rb`; **P8 backlog** |
| 65 | **Gifts** (fic gifted to a user/pseud) — a dedicated "gifted to X" link + gift inbox. Already partially in P8#37 (fic gifting); OTW's version adds the "gift exchange" reveal mechanic. | ✅ covered — P8#37 |
| 66 | **Tag sets / tag-set nominations** — structured collections of tags for challenge signups + community-organized "nominate your tags" drives. Fits the tag-wrangling direction (P8#18). | 📋 otwarchive `tagset_models/`; **P8 backlog** — pairs with P8#18 + #57 |
| 67 | **Site-wide admin announcement banner** — one dismissible banner (e.g. outage, migration, event), cached, "banner seen" per-user. FicHub has no announcement surface today. | 📋 otwarchive `admin_banner.rb`; **P8 backlog** — cheap |
| 68 | **Per-fic hit/view counter** — public view counts on fic cards + reader. OTW's `stat_counter`/`redis_hit_counter` prove the pattern; FicHub tracks `usage_events` but has no public per-fic view count (forum topics have `view_count`, fics don't). | 📋 otwarchive `stat_counter.rb` + `redis_hit_counter.rb`; **P8 UF** — social proof; cheap |
| 69 | **Comment-reply inbox** — unified "replies to your comments" inbox with unread counts + in-reply-to threading. FicHub has notifications/read state; a dedicated comment-reply inbox (OTW `inbox_comment`) is distinct. | 📋 otwarchive `inbox_comment.rb`; **P8 UF** — notification loop keeps people engaged |
| 70 | **Known-issues / status page** — admin-curated "known issues" list with a public status view (title, content, severity/status). Cheap, high trust value for a self-hosted community. | 📋 otwarchive `known_issue.rb` + `known_issues_controller.rb`; **P8 backlog** — cheap |
| 71 | **External works** (record a fic hosted elsewhere) — bookmark/record off-site works (dead link, other archive, original fiction) with author, language, tags; no scraping. Fits the archive direction; pairs with P8#35 fic cards. | 📋 otwarchive `external_work.rb` + `external_works_controller.rb`; **P8 backlog** |
| 72 | **Per-fic language metadata** — OTW's `language.rb` (default English, per-work language, sortable). FicHub has UI i18n (`028_i18n_seed.sql`) but no per-fic language field. | 📋 otwarchive `language.rb`; **P8 backlog** — cheap |
| 131 | **Cross-site source links on fic page** — each `fic_info` source renders as a clickable badge linking to the original fic page on that site (AO3, RR, FFN, etc.). Compact social proof at a glance: "Also on: AO3 (2.1k kudos) · RoyalRoad (4.8k ratings)" where each name links to the original. Each scraper defines `source_url(fic_info_id) -> Option<String>` to construct the URL. No stale-data aggregation — live data on the original site. **P8** |

**Fastest OTW wins (cheap, high fit):** #52 kudos (one-click), #60 username history (admin
table), #64 fannish next-of-kin (data model), #59 user-level mute (extends existing blocks),
#61 reading history (extends P8#21), #67 announcement banner, #68 per-fic hit counter,
#72 per-fic language metadata.

**2026-08-15 brainstorm review** — re-read the FicHub Idea Brainstorm (115 ideas) and
added the ones worth tracking that weren't already captured (⭐ = the brainstorm's own
high-confidence picks). Skipped as already covered: cookie ingestion → P1#1, wayback →
P8#5, main-char picker → P1#4, semantic-cold-start → P8#32, annotations → P8#20, batch
export + queue → P8#1–2, e-reader → IDEAS §1, EPUB styling → P8#3, OPDS → shipped, endpoint
dashboard → P4#13, backup → USER-ACTIONS #3, blocks → P8#59, status page → P8#46/#70,
secrets/uptime → P2#5–6, taste profile → P8#29. Skipped as wild/niche for now: OCR
screenshot ingest, email-to-FicHub, ingestion quests, voice search, parallel view,
spice slider (part of #76), reading-buddy matcher, fic debates, native writing mode,
beta-reader matching, co-authoring, Anki export, federation, most of §11 delight.

| # | Feature | Status |
|---|---|---|
| 73 | **Browser-extension capture** (⭐) — user opens a blocked fic in their own browser, extension POSTs the HTML to FicHub, existing adapters parse it. Sidesteps the AO3/FFN host block entirely; complements P1#1 cookie flow. | 📋 brainstorm §1#2; **NEW — P8** |
| 74 | **"Paste the HTML" manual ingest** — textarea for a raw page, parsed server-side. Zero-dependency version of #73; no extension install. | 📋 brainstorm §1#4; **NEW — P8** |
| 75 | **Semantic/meaning search** (⭐) — NL query embedded against fic vectors (`nomic-embed-text` already resident for the rec platform). Distinct from Ask-the-Archive (LLM → boolean filters); this is embedding-to-embedding over pgvector. | 📋 brainstorm §2#16; **NEW — P8** |
| 76 | **Content shield + spice controls** — collapse explicit/gory passages behind a tap (warnings + LLM triage); global cap on explicitness in recs. | 📋 brainstorm §4#45 + §3#34; **P8 UF** — lets users browse safely in public |
| 77 | **Quote cards** (⭐) — render a favorite line as a shareable image. Builds on the shipped body FTS (`/search/body` + `<mark>` snippets); the natural viral surface. | 📋 brainstorm §4#43; **P8 UF** — Instagram moment for fanfic; free marketing |
| 78 | **Tag wiki** — community definitions of tropes linked from every tag chip; complements admin tag-wrangling (#18), curator-approve entries. | 📋 brainstorm §5#61; **P8 UF** — new readers need this; community knowledge |
| 79 | **Fandom landing pages** — auto-generated per-fandom hub: stats, top fics, trending ships, recent activity. | ✅ **SHIPPED 2026-08-17** (commit 21a0440) — `/fandoms` landing page + `/api/fandoms` endpoint |
| 80 | **Obsidian/Markdown export with frontmatter** — tags, dates, metadata embedded; one fic → vault-ready .md. | 📋 brainstorm §7#78; **NEW — P8** |
| 81 | **Print-ready book PDF** — real book layout (covers, margins), not a converted EPUB. | 📋 brainstorm §7#80; **NEW — P8** |
| 82 | **Public archive stats page** — growth curves, top fandoms, exports/day; zero-PII over `usage_events`. Distinct from the admin endpoint dashboard (P4#13). | 📋 brainstorm §8#82; **NEW — P8** |
| 83 | **Trope popularity over time + fandom health index** — trend graphs per tag; which fandoms grow/die by ingest + read rates. | 📋 brainstorm §8#84/#88; **NEW — P8** |
| 84 | **Metadata open dataset** — monthly dump of metadata (not bodies) for researchers/tooling. | 📋 brainstorm §8#87; **NEW — P8** |
| 85 | **Self-host one-click compose** (⭐) — Docker compose with sane defaults. No Docker on the current host, but it's the adoption lever (fichub.net replacement = instances). | 📋 brainstorm §9#89; **NEW — P8** |
| 86 | **Dead-dove detector** — LLM scan (Ollama + body cache) for content contradicting tags. | 📋 brainstorm §9#92; **NEW — P8** |
| 87 | **Age gate for `is_adult` content** — scrapers already carry the flag; enforce it in the UI. | 📋 brainstorm §9#94; **NEW — P8** |
| 88 | **API tokens for third parties** — scoped keys so external tools build on FicHub (complements P8#8 CORS). | 📋 brainstorm §10#100; **NEW — P8** |
| 89 | **CLI** — `fichub get <url> --format epub` for power users. | 📋 brainstorm §10#101; **NEW — P8** |
| 90 | **Chat bots (Discord/Telegram) + webhooks** — search, lookup, "send me this fic"; fic-updated events to n8n. | 📋 brainstorm §10#97–99; **NEW — P8** |
| 91 | **i18n key system** (⭐) — 6-locale i18n shipped (en/de/es/fr/pt-BR/zh) via `frontend/src/lib/i18n` with per-locale dictionaries (~45K entries each), `LocaleSelector` component in the layout + archive footer, localStorage persistence (`fichub_locale`), browser-language detection, and a backend `PUT /api/auth/locale` flow. UI text is live-translated across routed pages; remaining work is key-coverage completion and per-fic language metadata. | 📋 brainstorm §12#113; **P8 backlog** — coverage completion, not the subsystem |
| 92 | **WCAG audit + RTL end-to-end** — keyboard nav, screen-reader labels, contrast; settings store already has `useRtl`. | 📋 brainstorm §12#114–115; **NEW — P8** |
| 93 | **Fic-to-podcast RSS** — per-fic audio feed where each chapter is an episode (TTS). | 📋 brainstorm §7#77; **NEW — P8** |
| 94 | **Prompt calendar + flash-fic challenges** — daily/weekly prompts feeding Fic Requests; weekly contests with arena voting. | 📋 brainstorm §5#65 + §6#70; **NEW — P8** |
| 95 | **RAM-aware concurrency governor** — track resident memory (RSS) + Ollama load; decide whether another LLM (chat, embeddings, dead-dove scans) can run in parallel or must degrade gracefully. Ollama models are resource-heavy (lfm2.5:8b resident; nomic-embed-text), and multi-user Discord bots / rec engines can OOM the box. Probe: `cgroup` v2 `memory.current` + `memory.max` (or `/proc/meminfo`), plus Ollama `/api/ps` for loaded models; expose `GET /api/health/resource` so the Discord bot can pre-check before spawning an LLM call. | **NEW — P8** (from `fanfic-archivist` spec; bots + rec platform both need it) |
| 96 | **Fic-level moods (⭐)** — classify each work's *tone/mood* on a small axis (e.g. `Neutral/Funny/Shocky/Flirty/Dramatic/Hurty/Bondy`), not author-level. Computed from content signals: genre ratios, description tone, review/kudos sentiment, tag mix. Unlike author-moods (which describe the *author's* average style), fic-moods describe the *story itself* — better for "rec me something dramatic" and for mood-based filtering. Feeds the rec engine (mood-similarity) + search facets + per-fic "mood" chip. | 📋 adapted from **Zeks/flipper** (`rec_calculator_mood_adjusted`, `fic_genre_data.h`); **NEW — P8** |
| 97 | **Author recommendations (⭐)** — recommend *authors* similar to ones I like, or whose works fit my taste. Today the website recommends fics only; there's no "authors like X" surface. Build: author feature vectors from their works (genre mix, mood distribution, fandom spread, popularity band) → author-similarity ranking; plus "authors who wrote fic I've kudos'd/bookmarked" as an implicit taste signal. Deliver: `/api/recommendations/authors?q=<author-or-fic>` + an "Authors you might like" panel + Discord `/authors` command. | 📋 adapted from **Zeks/flipper** (`Interfaces/authors.cpp`, `ffn_authors.cpp`, author-overlap weighting); **NEW — P8** |
| 98 | **Rarity-tier weighted rec strategy** — the Zeks/flipper *weighted* recommender: per-author overlap `ratio` + `matches` + sigma, then weight authors by rarity tier (unique 0.2×matches, rare 0.05×, uncommon 0.005×, common 1). Rare overlap with a big-list author scores higher than a popular author with one common match. Explainable ("you share 5 faves with this author whose list is 90% rare fic") + powers `!sus`-style list-manipulation detection. Add as a pluggable `weighted` strategy beside `mf`/embeddings. | 📋 adapted from **Zeks/flipper** (`rec_calculator_weighted.cpp`); **NEW — P8** |
| 99 | **Audience-genre inference (recommend-by-who-likes-it)** — compute each fic's genre profile *of the people who recommend it* (kudos/bookmark graph): "this fic is funny *to humor-lovers*", not just "declared genre". Cheap SQL over kudos/bookmarks; no ML. Powers "recommend me funny fic" + a per-fic "audience tastes" signal + genre facets. | 📋 adapted from **Zeks/flipper** (`reference_queries/genre_detector.sql`); **NEW — P8** |
| 100 | **Explorer / size / popularity bands as filters** — first-class search/rec filters: popularity band (barely-known / relatively-unknown / popular), size class (small ≤20k / medium ≤100k / big ≤400k / huge), plus per-list ratios (explorer ratio, crossover ratio, unfinished ratio, mature ratio, fandom-diversity). Unlocks "obscure gems" (`!gems`), "short completed fic", etc. | 📋 adapted from **Zeks/flipper** (`fav_list_analysis.h` `ExplorerRanges`/`EntitySizeType`); **NEW — P8** |
| 101 | **Tracked-message bot state machines** — the Zeks/flipper Discord bot persists a state machine *per message* (`tracked_recommendation_list`, `tracked_roll`, `tracked_review`, `tracked_fic_details`, `tracked_similarity_list`, `tracked_help_page`, `tracked_delete_confirmation`), so commands mutate that message in place (re-roll the same list, next/prev by message reference, delete-confirm). More capable than our current button rows; port to `fanfic-archivist`. | 📋 adapted from **Zeks/flipper** (`src/discord/tracked-messages/`); **NEW — P8** (bot) |
| 102 | **Missing bot commands from Flipper** — `!sus` (suspicious-author detection / list-bombing), `!stats` (per-user profile stats), `!fresh` (recency-gated), `!filters` (filter rec stream by liked authors / forced params / reset), `!full-favourites` (browse whole list), `!cutoff`/`!wordcount`/`!year` (range tuning). Add to `fanfic-archivist`. | 📋 adapted from **Zeks/flipper** (`command_generators.h`); **NEW — P8** (bot) |
| 103 | **Author list-manipulation / abuse detection (`!sus`)** — flag authors whose fav-list looks gamed (anomalous overlap, ratio outliers, suspicious vote patterns). Surface on author pages + Discord. Pairs with rarity-tier weighting. | 📋 adapted from **Zeks/flipper** `Sus` command + `CalcWeightingParams` outlier logic; **NEW — P8** |
| 104 | **Per-author profile stats** — aggregate per-author: wordcount/size distribution, fandom diversity, crossover ratio, mood uniformity, popularity band, "explorer" tendency. Powers author pages + `!stats` + author-recs signal. | 📋 adapted from **Zeks/flipper** (`FicListDataAccumulator`/`fav_list_analysis`); **NEW — P8** |

**Cheapest of these (single query / existing infra):** #74 paste-HTML, #82 public stats
(one GROUP BY over `usage_events`), #87 age gate (flag already carried), #89 CLI (thin
wrapper over existing endpoints), #91 i18n key extraction (mechanical, unlocks #92).
**Highest-value rec-platform wins:** #96 fic-level moods (content signals we already
hold — genre, description, reviews, tags), #97 author recommendations (new surface;
author vectors from existing works), #99 audience-genre inference (one SQL over the
kudos/bookmark graph), #100 explorer/size/popularity bands (cheap facets).

**Admin dashboard & engagement metrics**

| # | Feature | Status |
|---|---|---|
| 41 | Per-endpoint usage breakdown | ✅ already tracked — **P4#13** (one GROUP BY over `usage_events`) |
| 42 | Conversion funnel chart (visit → search → read → bookmark → export) | **NEW — P8** |
| 43 | Zero-result query report (top failing queries → ingest) | 📋 P4#12 data exists; **P8** — surface the report |
| 44 | Cohort retention (weekly signup cohorts, return rate) | **NEW — P8** |
| 45 | Bot vs human traffic split (time series from `bot_scores`) | **NEW — P8** (cheap: data exists) |
| 46 | Scraper health board (failures by domain × class + agent run log) | **NEW — P8** (data in `scrape_failures`/`agent_runs`) |
| 47 | Content coverage report (fics by site/fandom/word-count bucket) | **NEW — P8** |
| 48 | Search→export conversion trend (extend view into weekly trend) | **NEW — P8** |
| 49 | Modlog diff view (before/after values on metadata edits) | **NEW — P8** |
| 50 | Feature adoption tracker (% of active users touching a feature in 30d) | **NEW — P8** (cheap: path-prefix over `usage_events`) |
| 105 | Circuit breaker for dead user sidecars (fallback to cooccur) | **NEW — P8** (timeout + breaker in external strategy; guardrails in v3 spec §11.5) |
| 106 | Per-surface rec recipes (home vs fic-page vs Next-Up each use different blends) | **NEW — P8** (extends recipe activation to surface-level override) |
| 107 | Rec subscription (follow another user's recipe, auto-propagate updates) | **NEW — P8** (extends gallery install with sync-on-change) |
| 108 | Theme remixing (fork shared presets, tweak, re-share) | **NEW — P8** (one-click fork from gallery → save as own recipe) |
| 109 | Reduced-motion accessibility theme preset | **NEW — P8** (CSS prefers-reduced-motion; ships alongside high-contrast) |
| 110 | Seasonal event themes (unlock temporarily, collectible achievements) | **NEW — P8** (time-gated unlocks feed progression) |
| 111 | Custom shelf colors and icons (visual library organization) | **P8 UF** — visual organization; feels premium |
| 112 | Command palette for customization (universal launcher for saved views, themes, recipes) | **NEW — P8** (extends existing Ctrl+K palette) |
| 113 | User-remappable keyboard shortcuts (exportable as shareable profiles) | **NEW — P8** (localStorage keymap + JSON export/import) |
| 114 | Skip locked widgets when installing shared layouts | **NEW — P8** (graceful degradation: skip widgets user hasn't unlocked) |
| 115 | Reset-to-default layout button | **NEW — P8** (cheap UX safety net) |
| 116 | Blind-date as unlockable widget (surfaces random hidden gems) | **P8 UF** — serendipity; "Surprise me!" is addictive |
| 117 | Forum posting level gate (anti-spam for fresh accounts) | **NEW — P8** (configurable level threshold, default L3) |
| 118 | Daily quests + reading streaks for XP (gamified engagement loop) | **P8 UF** — habit loops; reading as ritual |
| 119 | Rec explainability (strategy name + reason on every recommendation card) | **P8 UF** — "Because you watched" surface; trust |
| 120 | People-like-you row (taste clusters → explain by group) | **P8 UF** — social proof + belonging |
| 124 | **Reputation + daily login streaks + community bounties** — extend the existing XP/levels system with: (a) daily login streaks — users gain XP for consecutive daily visits, scaling up with streak length (StackExchange-style); (b) forum posting reputation — XP for posts/replies that receive positive feedback (kudos/upvotes), distinct from the existing `award_exp` on generic posts; (c) community bounties on Fic Requests — users stake XP from their reputation pool to incentivize authors answering specific trope requests (e.g. "50 XP bounty on Dark Harry x Draco Coffee Shop AU"), winner takes the pot. This gamifies both content *consumption* (streaks, daily visits) and *creation* (bounties, forum reputation), and pairs with P8#41 (tropes bounties) + P8#117 (forum level gate) + P8#118 (daily quests/streaks). The existing progression system (100 levels, exp_events, award_exp) provides the foundation — this adds the reputation currency layer + bounty mechanics. | **NEW — P8 UF** — retention + content creation incentive loop |
| 121 | Curator prior personal slider (user-adjustable taste shaping) | **NEW — P8** (exposes REC_CURATOR_PRIOR as user pref) |
| 122 | Customization profile export (one portable JSON backup of all prefs) | **NEW — P8** (extends data export with theme/recipe/widget/layout) |
| 123 | Feature toggle API (power users script setup via API tokens) | **NEW — P8** (REST endpoint for programmatic feature activation) |
| 124 | Curator-verified badge in rec marketplace | **NEW — P8** (curator review → verified flag on extensions) |
| 125 | Level gate + curator review for publishing themes/layouts | **NEW — P8** (extends recipe publish gate to themes/layouts) |
| 126 | Widget marketplace with live previews (rendered from layout JSON) | **NEW — P8** (preview before install) |
| 127 | Rec shown-to-engaged ratio as social proof per shared recipe | **NEW — P8** (impressions → engagement ratio from rec_impressions) |
| 128 | Per-feature notification granularity (mute what you disabled) | **NEW — P8** (extends notification prefs with feature-level toggle) |

**Fastest to ship (user-flagged cheapest wins):** #41, #43, #45, #50 (single
queries over existing tables), #27 (already in the recommender), #4 (stated
P1).

### P8.5 — Bot + CLI feature parity (fanfic-archivist + FicHub CLI)
>The Discord companion bot (`fanfic-archivist/`) and the FicHub CLI (not yet
>created — `fichub get <url>`) both consume the public REST API. The bot has
>been shipping for months but lags behind the site's feature set. Full gap
>analysis: `docs/bot-cli-parity-gaps.md`. Each item below is a bot API-client
>method + Discord command (or CLI subcommand) to add.

<table headers>
# Feature API endpoint Priority
131 Work detail — `/work <id>` (Discord) + `fichub work <id>` (CLI), shared presenter | GET /api/works/{id} | ✅ P1 shipped
132 Work stats GET /api/works/{id}/stats P2
133 Reviews list GET /api/works/{id}/reviews P2
134 Kudos toggle — `/kudos <id>` (Discord, auth-gated) + `fichub kudos <id>` (CLI) | POST/DELETE /api/kudos/{work_id} | ✅ P1 shipped
135 Search suggestions GET /api/search/suggest P2
136 Similar works GET /api/works/{id}/similar P2
137 Random work GET /api/works/random P2
138 User profile GET /api/users/{id} P2
139 Curator leaderboard GET /api/leaderboard/curators P2
140 Forum categories+topics GET /api/forum/categories P3
141 Forum topic detail GET /api/forum/topics/{id} P3
142 Forum search GET /api/forum/search P3
143 Send-to-Kindle POST /api/send-to-kindle P2
144 User data export GET /api/user/export P2
145 FicHub CLI (`fichub work/kudos/search/download`) — shares FichubClient with bot | reuses 131-134 | ✅ P1 shipped
</table>

**P1 bot/CLI items shipped:** #131 work detail, #134 kudos toggle, #145 CLI scaffold (fichub work/kudos/search/download).
**P2 bot/CLI:** #132 stats, #133 reviews, #136 similar, #137 random, #139 leaderboard, #143 send-to-kindle, #144 export.
**P3 bot/CLI:** #140-#142 forum browsing.

**User-facing priority batch (UF) — 20 items, sorted by impact:**
Emotional hooks: #31 mood slot, #77 quote cards, #116 blind-date widget.
Identity: #29 taste profile, #21 reading stats ("Spotify Wrapped").
Retention: #39 weekly digest, #25 chapter countdown, #34/#118 challenges/streaks.
Social proof: #68 view counter, #30 "people like you", #120 clusters row.
Trust: #27/#119 rec explainability, #132 data export, #22 reading time.
Community: #37 fic gifting, #69 comment inbox, #78 tag wiki.
Safety: #76 content shield, #26 keyboard shortcuts, #111 shelf colors.

### P9 — Extension platform (v3.1)

> 2026-08-17: The v3 progression system (P1-P5) is shipped. The extension
> platform generalizes the recommender strategy registry + theme system into
> a unified "power ladder" with four tiers: Config, Recipe, Service, WASM.
> Recipe Builder (P6) and Theme Design Tokens (P7) are shipped.
> The following are deferred until demand proves out.

| # | Feature | Why deferred |
|---|---|---|
| 51 | Multi-tenant sidecars (user-hosted rec engines) | Needs trust system + outbound HTTP safety; compute on user's server |
| 52 | WASM sandbox for user scoring functions | Heavy dependency (wasmtime); fuel+memory metering needed |
| 53 | Full marketplace with ratings/reviews + engagement leaderboard | Needs user base first; shadow-run engagement as gate |
| 54 | Nav/layout editor (drag-drop reorder) | Partially built; iterate on existing widget dashboard |
| 55 | Layout presets (mobile vs desktop variants) | Nice but low priority; users can save multiple layouts via user_views |
| 56 | Community recommender leaderboard (Elo from rec_impressions) | Needs rec_impressions data to accumulate first |
| 57 | Recipe → precomputed promotion (batch compute popular shared recipes) | Only when shared recipes get popular enough to warrant batch compute |
| 58 | XP from customization reputation ("your theme installed 50 times") | Makes customization itself a progression loop; needs marketplace first |
| 129 | Seasonal event themes with time-gated unlocks + collectible achievements | Needs progression loop proven first; engagement hook for later |
| 130 | Full marketplace with curator review pipeline + verified badges | Needs user base + rec_impressions data; curator review is heavy |

---

## 4. Test coverage

- Frontend: 74 files / 464 tests (vitest), admin + modlog + analytics pages,
  recipe settings, theme editor.
- Backend: full lib suite ~620 tests; DB-gated suites: admin_api 13/13,
  heal_api 8/8, modlog_api 3/3, curator_fix_api 2/2, analytics_api 2/2,
  fic_suggestions 3/3, ask_archive 6/6, requests_api 11/11, body_search_api
  3/3, reader_api 2/2, rec_* suites,
  roadmap 6/6, integration 31/31. v3: progression (XP, levels, ranks,
  feature gates, recipe CRUD, theme tokens).
- Scraper crate: **~220 adapter tests** (`cd scrapers && cargo test`), each
  site adapter ships can_handle / id-parse / date-parse / body-extract unit
  tests.
- Canonical gate: `hermes verify --save` (detect → build → test → boot →
  readiness). Known quirk: the CLI's evidence recording can fail silently
  (module import inside the CLI); record via the evidence API directly if the
  ledger shows no event for the session.
- NFS shared-target quirk: `target/debug/incremental` rlib fingerprints go
  stale after mount wedges — `rm -rf target/debug/incremental` before
  canonical verify. Verify MUST run with
  `export CARGO_TARGET_DIR=/media/alvaro/cargo-target-sh` (local SSD; the
  NFS target-dir pin was removed — it caused E0463 rlib corruption).

## 5. Deployment

- Live: fichub.polarisocial.xyz (ThinkCentre, systemd fichub.service,
  migrations 1-053 applied). Frontend built to `frontend/build` (served from
  disk; no restart needed for static changes), backend binary at
  `target/release/fichub` + restart.
- Migrations apply on service start. BODY_CACHE_DIR set in the service env
  (default `/public/literature/fichub/bodies`).
- Docs: mdbook source in `docs/`; rebuild with `docs/build.sh` +
  `docs/build_docs_map.py` → copies HTML into `frontend/static/docs`
  (committed) + `frontend/build/docs` (served); restart the service to pick
  up new static docs on the live site.
- Mirror: push after merges to https://opencommit.eu/MagicZhang/fichub
  (remove stale `.git/refs/remotes/github/main.lock` first; NFS git-objects
  workaround — see skill reference).


======================================================================
SOURCE: docs/STATUS.md
======================================================================

# FicHub — Session Status (2026-08-12, feature-complete pass)

> **CONSOLIDATED**: the canonical planning doc is now **docs/ROADMAP.md**
> (shipped / in-flight / prioritized suggestions / test coverage /
> deployment). This file keeps the session-by-session detail below.

## Post-parity feature pass (2026-08-12, events 112-117)

Everything from the brainstorm wave shipped, verified, and deployed:

- **FanFicFare Python-CLI dependency removed** (fc79809): the `fanficfare`
  fallback adapter is gated behind the crate's `fff-fallback` feature (off
  by default); FicHub registers native adapters only. Unsupported URLs now
  return `Unsupported` instead of shelling out to a 60s Python subprocess.
  Also fixed the **native-scraper routing bug** (8128e9c): `lookup_authed`
  used `find_scraper()` which returned the index-0 FF catch-all for every
  URL — switched to `find_specific_or_fallback()` (native first). RoyalRoad
  went from ~62s CLI to ~280ms native; RoyalRoad markup drift fixed
  (fdf9624, `h1.font-white` + `109 Chapters` pill).
- **User-supplied site credentials** (7e8d112): logged-in users can provide
  credentials for login-requiring sites (fanfics.me, fictionhunt, inkbunny,
  sofurry, dokuga) — opt-in consent, AES-256-GCM encrypted at rest, 30-day
  expiry, passwords never returned; download path uses the requesting user's
  creds via `lookup_authed`. Migration 039.
- **Ask × Requests combine** (e0d8c74): an empty ask turns into a Fic
  Request in one click (`/requests/new?q=`); request pages have an "Ask the
  Archive" box with one-click answers via the new `url_id` answer path.
- **Full-text search over fic bodies** (61cf1fa): migration 040
  (`body_text_search` tsvector + GIN), `GET /api/search/body` (boolean
  parser → ts_rank + `<mark>` snippets), maintained at body-write time
  (export + curator hooks) + `backfill_body_search` bin (LTO fix 7314d0d).
- **Reader "Next Up" endpoints** (61cf1fa): `GET /api/reader/{url_id}/sequel`
  (next-in-series via series_works + numeral fallback) and
  `/related` (readers-also-bookmarked co-occurrence) — previously 404, the
  Next Up panel was silently empty.
- **ROADMAP consolidated** (1933a93): dated wave sections merged into
  thematic "Shipped" subsections; no per-date brainstorm sessions.
- Test counts now: FicHub lib 582, DB-gated suites incl. requests_api 11/11,
  body_search_api 3/3, reader_api 2/2, crate 200, frontend 464.
  Canonical verify events 112-117 all `ok: True`.

## Full FanFicFare adapter parity (2026-08-12)

The `fanfic-scrapers` crate reached **one-to-one parity with FanFicFare**:
all **107 real FFF adapters** now have native Rust ports, shipped across
crates.io releases v0.6.0 → v0.10.0. See `scrapers/FFF_PARITY.md` for the
tracked FFF baseline (v4.60.0) + update workflow.

- **Family adapters** (config-driven, DRY): eFiction (19), eFiction-variant
  `viewstory.php?sid=` (26 — biggest cluster, folded wolverineandrogue into
  it), OTW siblings (5), XenForo1+2 (8), StoriesOnline (3), WordPress-novel.
- **Bespoke ports** this campaign: FicWad, FictionMania, FanFiktion.de,
  TouchFluffyTail, Fanfics.me, FanFicAuthors, FictionHunt, InkBunny, SoFurry
  (+ a shared turbo-stream decoder), Dokuga, PhoenixSong, StoriesOfArda,
  Fictionalley, NovelAll, MassEffect2.in, ReadonlyMind, UtopiaStories,
  ASexStories, AnEroticStory, MCStories, HentaiFoundry, BDSMLibrary.
- **Login support** (v0.8.0): `SiteScraper::login`/`requires_login` +
  `SiteCredentials`; FicHub client enables `cookie_store` +
  `FANFICSCRAPER_<DOMAIN>_USER/_PASS` env creds + `lookup_authed` pre-pass.
- **`is_adult` gate** (v0.9.0-v0.10.0): `SiteCredentials::is_adult` +
  `with_adult()` unlocks the 7 adult archives; `FANFICSCRAPER_IS_ADULT` env.
- **Version policy** (user directive): no 1.0 until the crate is a perfect
  drop-in replacement for fanficfare; wave bumps continue.
- **Tests**: crate 215/215 (each adapter: can_handle / id-parse / date-parse
  / body-extract); FicHub lib 575/575; canonical verify events 105-111 all
  `ok: True` (root cause of prior failures = NFS target-dir pin, removed
  2026-08-12; verify MUST run with
  `CARGO_TARGET_DIR=/media/alvaro/cargo-target-sh`).

## Docs: contributing chapter (2026-08-12)

- `docs/src/contributing.md` — junior-dev onboarding (stack, repo layout,
  first-run, dev loop, testing matrix, QA harness, PR flow, cheat-sheet).
- Added to SUMMARY under "For contributors"; DOCS_META updated; mdbook +
  `build_docs_map.py` regenerated (153 docs-map entries); deployed to prod
  (service restart; `/docs/contributing.html` live).

## Transparent modlog (2026-08-11, c961ac8)

Every moderator / curator / admin action is recorded in the `modlog` table
(migration 034) and readable by **ANY logged-in user** via `GET /api/modlog`
and the `/modlog` page (nav: 🛡️ Moderation Log) — complete transparency.

Instrumented actions: set_user_role, ban/unban, approve/reject upload,
approve/reject translation, hide/delete comment, blacklist fic/author,
create_alias, merge_tags, delete_tag, resolve_flag, propose_fix,
vote_fix (with outcome), delete_body.

Tests: modlog_api 3/3 DB-gated (regular user can read, anonymous rejected,
action filter), modlog lib 4/4, frontend page.test.ts 2/2.

## Usage analytics (2026-08-11, 6f27333)

Non-PII anonymous usage tracking. Every frontend request already carries an
`X-Client-ID` header (UUID in localStorage) — a middleware records each
request into `usage_events` (client_id, path, event_type view|action,
user_agent, created_at), fire-and-forget (zero latency).

**Admin dashboard: `/admin/analytics`** (role ≥ 10) shows:
- Unique visitors: daily (30d), weekly (12w), monthly (12m) — bar charts.
- Engagement: active users (performed actions) vs view-only users, for 1d /
  7d / 30d windows, with action events + total events + action rate.
- Recent activity timeline (last 50 events, with path + type + time).

Actions = exports/downloads/votes/comments/bookmarks/etc. (path-prefix
classified); views = plain browsing. API: GET /api/admin/analytics.

Migration 033. Tests: analytics_api 2/2 DB-gated, frontend page.test.ts 3/3.

## Site as a cache of gathered fanfiction (2026-08-11)

FicHub now **persists every scraped fic body** as a JSON blob on the
ThinkCentre-attached drive (`BODY_CACHE_DIR`, default
`/public/literature/fichub/bodies`), NOT in the database. The site is a
cache of all fanfiction it has gathered:

- On a successful export, the scraped chapters are saved to
  `bodies/<url_id>.json` (structured: ordered chapters + metadata).
- On the next export of the same fic, the cached body is used INSTEAD of
  re-scraping — instant, offline, resilient to source changes/blocking.
- Curators can **fix a wrong body** via the content-fix endpoints
  (`PUT/DELETE/GET /api/curator/content/{url_id}`, role ≥ 10): replace the
  cached body with the correct content (e.g. the actual fic post from a
  forum thread, not the comments), or delete it to force a re-scrape.
- Blobs live on the attached drive (e.g. /public/literature/fichub/), so
  the DB stays small and the corpus is portable.

This pairs with the self-healing work: when a scrape fails, the failure is
recorded (scrape_failures) and curators have a tool to correct the result
even after the wrong body was gathered.

## LLM model optimization (2026-08-11, 5961e80)

### Problem
Ask-the-Archive (and comment triage) used `OLLAMA_CHAT_MODEL=qwen3.5:9b`
(6.6GB) on the ThinkCentre (i5-8500T, 6 cores, 15GB RAM). Cold load was
minutes; the live ask-archive timed out and fell back to plain search.

### Benchmark (real tok/s on ThinkCentre, one model loaded at a time)
| Model | Size | Prose tok/s | JSON tok/s | Notes |
|-------|------|-------------|------------|-------|
| **lfm2.5:8b** | 5.2GB (MoE) | **21.6** | **21.7** | WINNER — MoE speed |
| llama3.2:3b | 2.0GB | 12.2 | — | good small fallback |
| qwen3.5:2b | 2.7GB | — | 11.6 | clean JSON |
| ornith:9b | 5.6GB | 4.6 | 4.7 | slow |
| qwen3.5:9b | 6.6GB | ~4.5 | — | old default, slow |
| qwen2.5-coder:7b | 4.7GB | 5.7 | — | slow |

Memory is the binding constraint on M720q: two 5GB+ models loaded together
thrash (8.3GB models + 2.2GB ollama > available). Keep ONE model resident
via keep_alive.

### Changes (5961e80)
- `src/services/ollama.rs`: `generate()` / `generate_json()` now send
  `keep_alive: "30m"` + `think: false`, and strip `<think>…</think>` blocks
  from responses (thinking models emit them even with the option).
- `src/config.rs`: default `OLLAMA_CHAT_MODEL = lfm2.5:8b`.
- Live-verified: `POST /api/search/ask` "completed dark harry potter over 50k
  words" → `{"complete":true, "main_char_attr":"Harry Potter|Dark Harry
  Potter", "min_words":50000}` in **17.6s incl. cold load** (old model:
  timeout → fallback). `translated: true`.

## Pluggable recommendation platform (2026-08-10, wt-rec-platform)

Stage table:

| Stage | Strategy | status |
|-------|----------|--------|
| 0 | `legacy_cooccur` (golden: current engine ≡ strategy) | ✅ shipped |
| 0 | registry + RRF ranker + unified signals | ✅ shipped |
| 1 | `decay` (SAR time-decay + log-likelihood, pure SQL) | ✅ shipped |
| 2 | `embeddings` (pgvector, Ollama nomic-embed-text, content-hash gated) | ✅ shipped |
| 3 | `mf` (Hu–Koren ALS, Cargo feature `rec-mf`) | ✅ shipped (gated) |
| 4 | `hybrid` (MF + embeddings + tag Jaccard blend) | ✅ shipped |
| 5 | `author_graph` (co-bookmark + tag Jaccard) | ✅ shipped |
| 6 | `tag_graph` (meta-path traversal) | ✅ shipped |
| 7 | `sequential` (Markov next-read, Next Up source) | ✅ shipped |
| 8 | `clusters` (k-means over tag affinity) | ✅ shipped |
| 9 | `bandit` (Thompson sampling, impressions, decay) | ✅ shipped |
| 10 | curator prior (5× signals, α blend, alignment tracking) | ✅ shipped |
| 11 | `external` sidecar adapter + `rec-engines/` recipes | ✅ shipped (inert) |

Key points: `REC_ENGINE_MODE=legacy` (default) preserves today's behavior
exactly — golden test `golden_legacy_equals_cooccur_strategy` asserts the
registry-with-only-cooccur output equals the direct engine call. All
strategies ship DISABLED except `cooccur`. Migration 027 creates the
`rec_*` tables (all inert until a strategy runs). Batch pipeline runs every
`REC_TRAIN_EVERY_H` hours via the collection-worker tick, recording
`rec_training_runs`. Admin endpoint `GET /api/recommendations/strategies`
lists enabled state + last run metrics. Docs: `docs/src/rec-engines.md`.

## What's DONE (2026-08-10 sessions)

### Admin auth + polish (fe22185)
- **Admin 403 root cause fixed**: admin/curator pages used raw
  `fetch(..., { credentials: 'include' })` but the backend reads the JWT
  only from the `Authorization: Bearer *** header — every admin call
  authenticated as role 0 → 403. New shared `frontend/src/lib/api/admin.ts`
  (`adminFetch`/`adminJson`) attaches the token from localStorage; all 10
  admin pages + 2 curator pages migrated. Admin/curator vitest 38/38.
- The Admin Dashboard link in the user dropdown is guarded by
  `isCurator` (role ≥ 10), so non-admins never see it; admins no longer
  hit 403s inside.
- **NotificationBell** now shows a visible "Notifications" label next to
  the bell icon (was icon-only).

### Curator/admin hardening wave (5 parallel worktrees → main 5d08076)
1. **Curator hub** — `/curator` landing page (role ≥ 10): cards linking
   Author Merges, Tag Flags, Work Proposals, Upload Moderation, Comment
   Triage, each with live pending counts. Added to routePages + user
   dropdown nav.
2. **Curator auth migration** — `/api/curator/*` (alias, merge, tags/{id},
   flags, flags/{id}/resolve) switched from a shared static bearer token
   (CURATOR_TOKEN, never configured) to role-based auth (role ≥ 10,
   400-as-401/403). The browser curator UI now works with a normal admin
   JWT; the shared admin token is gone. tests updated to role-10 users.
3. **New backend endpoints** (all role ≥ 10, DB-gated tests):
   - `PUT /api/curator/tags/{id}` — edit tag description / tag_type_id
     (canonical column when present; name immutable).
   - `/api/admin/blacklist` (GET list) + `POST /api/admin/blacklist/fic`
     and `/author` — manage fic_blacklist / author_blacklist.
   - `POST /api/admin/moderation/comments/{id}/hide` and `/{id}/delete` —
     admin comment moderation reusing existing hide/delete logic.
   - Work-split proposals verified via `/api/work-proposals`
     (action_type='split') + tests.
4. **Home dashboard declutter** — removed BlindDateCard, RSS subscribe row,
   and Surprise-me button; home is now download input + Recommended +
   Trending. Atom subscribe link moved to the footer.
5. **Admin nav audit + click-through e2e** — new `e2e/admin-nav.spec.ts`
   promotes an admin and clicks through EVERY admin page asserting its
   heading renders (dead-click regression guard). All admin links resolve.

### Roadmap arena fixes (prod bug triage, main 7c6fc18)
6. **Vote FK violation fixed** — anonymous votes used user_id=0 → violated
   `arena_votes_user_id_fkey` (500 "database error" on live site).
   Anonymous votes now persist with NULL user_id (FK-safe) and are keyed
   per-client by client_id for the double-vote guard. Logged-in votes
   unchanged. Regression test `anonymous_vote_persists_with_null_user_id`.
7. **Clustering was completely broken** — TWO stacked bugs:
   - `server.rs` passed `ollama_chat_model` (llama3.1:8b) as the embed
     model → `/api/embeddings` always failed → suggestions never clustered.
   - `roadmap.rs` built the vector literal with `{:?}` → quoted values
     (`["0.26",...]`) → Postgres `vector` rejected the insert.
   Both fixed: embed model = nomic-embed-text, unquoted vector join.
   Suggestions now cluster and spawn/join clusters correctly.
8. **Dark-mode text visibility** — arena card text, suggest textarea, and
   suggest h3 were black on dark backgrounds (invisible). Now use
   `var(--color-text)`.
9. **Full roadmap e2e** — three specs cover the complete lifecycle:
   - `roadmap-arena.spec.ts` — anonymous suggest → arena → vote → Elo
     applied → next set (both auth paths, FK-safe cleanup).
   - `roadmap-lifecycle.spec.ts` — suggest → vote → new set.
   - `requests-roadmap.spec.ts` — arena renders cards or empty state.

## Test coverage (all green on main 5961e80)
- **Frontend unit: 438 tests (66 files)** ✔ (i18n 17, ask 5, fic-suggestions
  15, roadmap 3, layout smoke 4)
- **Backend DB-gated: 41+ tests** across tag_edit (3), work_proposals (2),
  curator (3), tags (3), admin (12), social (7), health (4), requests (7),
  roadmap (6), ask_archive (6), fic_suggestions (3), rec_strategies (4),
  rec_embeddings (2), rec_bandit (3), rec_curator (2) ✔
- **Playwright e2e: 31/31** (auth, admin, admin-nav, curator, fic-reader,
  bookmarks-follows, home-search, requests-roadmap, roadmap-arena,
  roadmap-lifecycle) ✔
- cargo check --all-targets clean; release build ✔
- `hermes verify --save` green (build + test + readiness) recorded 2026-08-11.

## Deployment
- Live: https://fichub.polarisocial.xyz — health ok (db:true, redis:true),
  homepage 200, /ask + /roadmap 200. Migrations 26-28 applied (026
  fic-suggestions, 027 rec platform, 028 i18n seed).
- Ollama: lfm2.5:8b default on ThinkCentre (21.7 tok/s). qwen3.5:0.8b/2b/4b,
  ornith:9b also pulled for comparison; keep_alive 30m keeps the winner
  resident.
- NFS notes: local `CARGO_TARGET_DIR` avoids linker Bus errors on the NFS
  target. **Build on NFS/local ext4 (e.g. /media/alvaro/cargo-target-X),
  NEVER /home/alvaro** (user rule 2026-08-11).

## Next steps (curator/admin backlog, user-requested)
Build UI + tests for:
1. **Auto-tag review UI** — review ML tag suggestions (draft → approve).
2. **Main-char/ship score fixing** — correct misordered scores (drive
   `main_char_attr` search).
3. **Manual fic approval UI** — approve non-scraped submissions
   (`/api/admin/moderation/queue` exists).
4. **Metadata correction** — fix title/author/status/description on works.
5. **Blacklist UI** — DONE (page + tests exist).
6. **Report handling** — user reports (Rust-side, beyond the Go schema).
7. **Translation post-edit** — approve ML translations (draft → human).
8. **Rating/warning verification** — keep no_warnings search honest.
9. Fic Requests M2/M3 (URL-ingest answers, notifications, upvotes) —
   **candidates endpoint shipped (dd64fe5)**: real engine suggestions from
   the request's seed work.

## Backlog items shipped (2026-08-11, dd64fe5)
- CacheSemaphores bounded (leak fix, 10k cap).
- Fic Requests `/candidates` = recommender-engine suggestions (seed work).
- Admin search-analytics: search→export conversion view (searchers /
  exporters / converted, zero-PII).
- Verified: cache 38/38, admin_api 13/13, full hermes verify ok.


======================================================================
SOURCE: docs/TODO.md
======================================================================

# FicHub — Feature Notes & TODO

> **CONSOLIDATED**: the canonical planning doc is now **docs/ROADMAP.md**
> (shipped / in-flight / prioritized suggestions / test coverage /
> deployment). This file keeps the feature-by-feature notes below.
>
> **Updated**: 2026-08-12 — FULL FanFicFare adapter parity (v0.10.0),
> contributing docs chapter. See STATUS.md + NEXT.md.

---

## 🏆 Full FanFicFare adapter parity (2026-08-12)

- `fanfic-scrapers` crate reached **107/107 real FFF adapter parity** (v0.10.0
  on crates.io); the only unported FFF adapters are `test1`-`test4` internal
  test fixtures (not real sites).
- Families: eFiction (19) + eFiction-variant `viewstory.php?sid=` (26) +
  OTW (5) + XenForo1+2 (8) + StoriesOnline (3) + WordPress-novel; bespoke
  ports listed in `scrapers/FFF_PARITY.md`.
- **Login support** (`SiteScraper::login`, `SiteCredentials`,
  `FANFICSCRAPER_*_USER/_PASS`, cookie_store) for fanfics.me, fictionhunt,
  inkbunny, sofurry, dokuga.
- **`is_adult` flag** (`SiteCredentials::is_adult` + `with_adult()`,
  `FANFICSCRAPER_IS_ADULT`) for readonlymind, utopiastories, asexstories,
  aneroticstory, mcstories, hentaifoundry, bdsmlibrary.
- Version stays 0.x until the crate is a perfect drop-in replacement
  (user directive — see ROADMAP §1).
- **Contributing docs chapter** added (`docs/src/contributing.md`) + docs
  rebuilt and deployed.

## 🏛️ Site as a cache of gathered fanfiction (2026-08-11)

- Every scraped fic body is persisted as a JSON blob under `BODY_CACHE_DIR`
  (default `/public/literature/fichub/bodies` on the ThinkCentre-attached
  drive) — NOT in the DB. The site is a cache of everything it has scraped.
- Export reuses the cached body (no re-scrape); curator content-fix
  endpoints (`/api/curator/content/{url_id}`) let curators replace/delete
  the cached body when a scrape gathered the wrong content.
- Related: boards.theforce.net added as a XenForo site (990e94b);
  native scrapers now preferred over the FanFicFare catch-all
  (`find_specific_or_fff`, 3514a3d).

## 🚀 Self-healing for scraping (2026-08-11)

- **Migration 029** `scrape_failures` — failure telemetry (url, domain,
  error_kind, message, html_snapshot_path, fingerprint, resolution).
- **Migration 030** `agent_runs` — agent loop bookkeeping.
- Classifier (transient/blocked/structural/systemic) + debounce + fingerprint
  dedup (reuses `qa/bugs.db` scheme).
- HTML snapshot capture on parse failures → `tests/fixtures/scrape/<domain>/`
  (agent context + future regression fixtures).
- Agent endpoint configurable — CommandCode API
  (`https://api.commandcode.ai/provider/v1/`, key in env) default, Ollama
  fallback. Diagnose-only this pass; `POST /api/admin/heal` manual trigger.
- See NEXT.md for the full milestone plan.

## 🚀 Recommendation platform (2026-08-10)

- Migration 027: `rec_user_signals`, `rec_embeddings` (pgvector 384 + HNSW),
  `rec_models`, `rec_bandit_arms`, `rec_impressions`, `rec_transitions`,
  `rec_author_graph`, `rec_user_clusters`, `rec_training_runs`,
  `rec_user_curator_align`. All inert until a strategy runs.
- `src/recommender/`: `strategy.rs` (trait), `registry.rs` (REC_STRATEGIES),
  `ranker.rs` (RRF + curator prior + bandit slots), `signals.rs` (unified
  view), `legacy_cooccur.rs` (golden default), plus `decay`, `embeddings`,
  `mf` (feature `rec-mf`), `hybrid`, `author_graph`, `tag_graph`,
  `sequential`, `clusters`, `bandit`, `external`, `curator`.
- `REC_ENGINE_MODE=legacy` (default) = today's behavior exactly; golden test
  proves it. Batch pipeline in the worker tick (REC_TRAIN_EVERY_H),
  `rec_training_runs` bookkeeping.
- Admin `GET /api/recommendations/strategies`; pluggable personal handler
  reports `strategies` diagnostics + `curator_alpha`; home dashboard shows
  "Curator's pick" when curator-shaped.
- `rec-engines/` Python sidecar recipes (LightFM, implicit, VW bandit,
  RecBole) — recipes only, `REC_EXTERNAL_URL` opt-in.

## ⭐ Shipped (2026-08-11 wave)

### Transparent modlog (c961ac8)
- `modlog` table (migration 034); every admin/curator action recorded
  (ban, role change, upload/translation approve-reject, comment hide/delete,
  blacklist, tag alias/merge/delete, flag resolve, body-fix propose/vote/
  delete).
- `GET /api/modlog` + `/modlog` page readable by ANY logged-in user.
- Tests: modlog_api 3/3, modlog lib 4/4, frontend 2/2.

### Usage analytics (6f27333)
- `usage_events` (migration 033) + middleware recording views vs actions
  (X-Client-ID, zero-PII). `/admin/analytics` dashboard: unique daily/weekly/
  monthly visitors, active vs view-only users, action timeline.
- Search→export conversion view added (dd64fe5) to search analytics.
- Tests: analytics_api 2/2, frontend 3/3.

### Backlog items (dd64fe5)
- CacheSemaphores bounded (10k cap — leak fix).
- Fic Requests `/candidates` = recommender-engine suggestions from seed work.
- Search→export conversion in admin search-analytics.
- Redis dashboard fix (8051863): realtime check uses dedicated health_redis
  (shared conn was parked by bookmark-import BRPOP → false "Redis down").
- Force.net: boards.theforce.net added as XenForo site; native scrapers
  preferred over FanFicFare catch-all.

## ⭐ Shipped (2026-08-08 wave)

### Fic Requests M1 (prompt board)
- `fic_requests` / `fic_request_answers` / `fic_request_answer_votes` tables (migration 018).
- Works-only answers (UNIQUE per request), +3 answer cap per user, up/down fit
  votes (net score only, no self-vote), requester accept → 'answered'.
- `/requests`, `/requests/new`, `/requests/[id]` pages; "🙋 Request similar" on
  fic pages; Discover nav link.
- Seeded into roadmap consensus (172 open + 16 shipped).

### Feedback rework (migration 014)
- 5-star `work_ratings` (1..=5, legacy -1 kept internal-only), `reviews` table
  (UNIQUE(user_id, work_id)), `comments.constructive` + partial index.
- Public surface is positive-only: no dislikes anywhere; public lists filter
  `constructive = TRUE`; `work_feedback_signals()` feeds the rec engine.

### Web reader v1
- `/read/[urlId]`: typography prefs, chapter nav, scroll progress, position
  save (localStorage keyed `fichub:reader:state:{url_id}`), Next Up panel
  (sequel → community pick → readers-also-bookmarked).

### Roadmap consensus
- MaxDiff/Elo arena (`/roadmap`), pgvector feature_clusters (768-dim,
  Ollama nomic-embed-text), `cargo run --bin seed-roadmap`, statuses
  open/shipped/rejected/deferred (migration 016).

### More shipped this wave
- **Reading lists + shelves** (migration 019) — `/lists`, `/shelves`.
- **Series & author pages** (migration 020) — `/series/[id]`, `/authors/[id]`.
- **RSS/Atom feeds** — `/feed.xml`, `/feed/follows.xml`, per-fic feeds.
- **PWA offline reader** — SW stale-while-revalidate + offline banner.
- **Follows + updates + refresh** (migration 017) — follow fic/author/user,
  `/api/v1/feed`, refresh-fic re-scrape → notify followers.
- **Quick wins** — tag autocomplete w/ counts, hide read/bookmarked,
  search-my-library, Blind Date, default prefs, dynamic facet counts.
- **Kanban all done** — Send-to-Kindle, PWA manifest, Strict Gen, bookmark CSV
  import/export, data export, E2E suite (15/15), Roadmap/Tropes route fix,
  CI coverage gate.

### Advanced search (AO3 niche cases)
- `main_char_attr=Character|Attribute` — freeform attributes apply to the MAIN
  character (first-listed character score=10; secondary=1). Implemented
  (commit a680301) + 20 DB-gated tests (commit ab4c67a).

### Comment moderation triage (migration 021, feat/llm-apps)
- `src/services/ollama.rs::generate` — non-streaming `/api/generate`
  (llama3.1:8b) alongside the existing embeddings call.
- `src/services/comment_triage.rs` — classify posted comments into
  fine/constructive/non-constructive/toxic/spam with reason + confidence;
  defensive parsing, `should_show_publicly()` helper; best-effort fallback to
  Fine/0.0 when Ollama is down (comment posts never fail).
- `comment_triage` table (migration 021), fire-and-forget hook in
  `add_comment_handler`, `GET /api/admin/moderation/comments` admin review
  queue (role ≥ 10, joined with comment body + fic title).
- comment_triage_api 3/3 DB-gated tests + lib unit tests.

---

## Backlog / TODO

### Fic Requests M2
- URL-ingest answers (scraper registry → find_or_create_work; reject
  blacklists; friendly error for AO3/FFN unreachable).
- Auto-seeded candidates: title parsed via `src/search/parser.rs` → top engine
  picks posted as `source='auto'` answers (ask-the-archive NL→filter reuse).
- Rec-engine feedback loop: accepted answers / high-score answers become
  recommendation edges (`request_signals()` alongside `work_feedback_signals()`).

### Fic Requests M3
- Notifications (follow plumbing) when a request gets answers / accepted.
- Request upvotes for board ranking; nullable `category` column if it grows.

### Consensus UI
- Roadmap badge column (endpoint returns status; page column pending).
- Per-card "Because you bookmarked X" anchor on home dashboard
  (data plumbing in `personal_recommendations_handler`).

### Reader
- Full app-shell PWA offline (currently reader-only).
- Chapter-level annotations / quotes.

### Search
- ~~Frontend UI control for `main_char_attr` ("Main character" + "attribute" pickers).~~ **DONE (b017681)** — `main_char_attr` removed; search system overhaul now possible.
- ~~Backfill: if/when a per-fic `main_char` score column exists, backfill it
  (note: today `main_char_attr` lives on `search_queries` as a FILTER, not a
  stored per-fic score — the "score=0 backfill" idea from the backlog review
  was based on a wrong premise; see admin_fix_tag_score for score fixing).~~ **N/A — main_char_attr removed.**
- Reading-history-based personalization signal (currently bookmarks+downloads only).

### Infrastructure
- Wayback/relay fallback chain for AO3/FFN (525/403 from this host).
- CI coverage thresholds → 80/70 as coverage grows. **DONE 2026-08-09** —
  src/lib at 90.3% lines / 82.66% funcs (gate raised to 85/78/76); hold the
  guard while refactoring, and push back toward 95/90 only if it stays cheap.
- Send-to-Kindle needs SMTP credentials from the user.


======================================================================
SOURCE: docs/IDEAS.md
======================================================================

# FicHub — Feature Ideas

> Brain-dump of ideas for making FicHub indispensable to daily fic readers.
> Starred (⭐) items are high-confidence picks that 70%+ of users would genuinely use.
> **Status legend**: ✅ = shipped (2026-08-08 wave), 🚧 = in progress / partial, — = not started.

---

## 1. The Core Promise: Downloading Stories

- **⭐ Broader site support** 🚧 — the holy grail. Wattpad, Quotev, Tumblr fics, fanfiktion.de, Webnovel… every new site brings a flood of users who currently have no good download tool. (Only RoyalRoad/Quotev/Wattpad reachable from this host; AO3/FFN blocked — Wayback/relay fallback is the strategic gap.)
- **⭐ Batch downloads** ✅ — select a whole series on AO3, an author's page, or tick boxes in your bookmarks and hit "download all as EPUB". One click, one zip file. Now supports AO3 + XenForo (QQ, SpaceBattles, SV) with SSE progress streaming.
- **⭐ Send to e-reader** 🚧 — email-to-Kindle is implemented (`kindle.rs`) but gated on SMTP credentials. Kobo's Dropbox integration is a future option.
- **⭐ Download queue + progress** — for big batches, show what's happening, allow cancelling, resume after network hiccups.
- **Customisable EPUB** — choose cover style, font, include/exclude author notes, dedications, tags as chapter, etc.
- **"Download again" instantly** — once a fic is cached, re-download in any format without scraping again, and notify if the source updated. (Refresh-fic re-scrape exists via follows.)
- **External Fanwork Aggregation & Link-Only Works** — FicHub should act as a central hub for *all* fanworks, not just archived text. Two key gaps: (1) Related Fanworks — a single story often inspires podfics, fan videos, art, or derivative fics; users should discover and link these from the original fic's page. (2) Link-Only Works — some authors forbid archiving their text, and some fanworks are inherently non-text (audio dramas, YouTube videos, comics). FicHub should support adding these with *only* metadata (title, author, tags, external URL), enabling full community features (bookmarks, ratings, reviews, comments, recommendations) without storing the file/body. Implementation: extend `works` with `media_type`, `content_policy` (`archived` | `external_only`), `external_url`; new `work_relations` table for source→target links with `relation_type` (`has_podfic`, `video_adaptation`, `inspired_by`, etc.); backend endpoints `POST /api/works/external`, `GET/POST /api/works/{id}/relations`; frontend "Related Fanworks" section on fic detail page; metadata-only scrapers returning `ScrapeError::MetadataOnly`. Aligns with P8#71 (OTW External Works), Unified Works Model, and Recommendation Platform. Open questions: link rot detection, curator burden for user-submitted links, metadata scraping limits for YouTube/SoundCloud.

---

## 2. Your Personal Library

- **⭐ Shelves / Collections** ✅ — `/shelves` + reading status (Want to Read, Currently Reading, Completed, Dropped).
- **⭐ Reading status** ✅ — tracked per fic, filterable.
- **⭐ Chapter tracking** ✅ — the web reader saves your position per fic (scroll + chapter), synced to your account.
- **Private notes** — jot down why you liked it, a favourite quote. Stays inside FicHub, not on the public internet.
- **Import/Export bookmarks** ✅ — CSV import/export shipped (bookmark_csv_api).
- **Search my library** ✅ — quick-wins: "hide read/bookmarked" + search-my-library.
- **Reading lists / bundles** ✅ — curated ordered lists with blurbs (`/lists`, migration 019).

---

## 3. Discovery: Finding Your Next Obsession

- **⭐ Personalised recommendations** ✅ — "Because you bookmarked X" on the home dashboard + per-fic "Readers also bookmarked" anchors.
- **⭐ "Similar fics" on every fic page** ✅ — co-occurrence + recommendations on fic pages.
- **⭐ Follow feeds** ✅ — follow an author/fandom/tag → updates feed (`/api/v1/feed`) + refresh-fic notifications.
- **Fic Requests** ✅ (M1) — prompt board; works-only answers, community fit votes, requester accept. M2 = URL-ingest + auto-candidates; M3 = notifications.
- **Public community shelves** 🚧 — reading lists exist; public discovery page pending.
- **Trending / Popular this week** ✅ — `/trending`.

---

## 4. Reading Inside FicHub (the "maybe I don't need an app" angle)

- **⭐ Built-in web reader** ✅ — `/read/[urlId]` with font size/theme, chapter nav, scroll-position memory, "Next Up" panel.
- **⭐ Offline reading (PWA)** ✅ — service worker caches reader responses (stale-while-revalidate) + offline banner. Full app-shell offline is future work.
- **Chapter navigation** ✅ — next/prev, dropdown, progress restoration on return.
- **Reading stats** — "you've read 300k words this month, 70% complete on your 'To Read' shelf." Gamify without the pressure.

---

## 5. Account & Sync (the glue)

- **⭐ True cross-device sync** ✅ — bookmarks, reading progress, shelves, lists all tied to your account.
- **Default preferences** ✅ — default download format, etc. (`prefs.ts`).
- **Data export/backup** ✅ — one button to download all your data (`user_export.rs`).

---

## 6. Community (lightweight social)

- **⭐ Comments** ✅ — threaded comments; public lists show constructive ones.
- **⭐ Ratings & short reviews** ✅ — 5-star ratings + in-depth reviews; positive-only public surface.
- **Following users** 🚧 — follows exist; public shelves/recommendations opt-in pending.

---

## 7. The Admin Side (for self-hosters)

- **Dashboard** ✅ — `/admin` dashboard, moderation queue, scrapers, users, bots.
- **Bulk actions** ✅ — re-scrape all fics by an author, re-tag a batch (auto-tagger exists; bulk admin UI shipped (commit b017681)).
- **Import from other FicHub instances / fichub.net** — if someone's moving from the old service, a migration tool would be killer.

---

## 8. Usage Analytics & Visitor Tracking

### Client ID System ✅
- **Anonymous client IDs** — UUID on first visit, localStorage, X-Client-ID header.
- **Unique visitor tracking** — total requests vs unique users.
- **Return visitor rate** — 7/30-day returns.

### Per-User Action Tracking ✅
- **Download counts**, **search queries** (anonymized), **fic views**, **bookmark/rating/comment activity**, **session duration**.

### Aggregate Analytics ✅
- **Popular fics this week/month**, **download format breakdown**, **site source breakdown**, **peak usage times** — via `/api/admin/realtime` + `/api/admin/search-analytics` + bot-scorer.

### Admin Dashboard ✅
- Real-time metrics, historical trends, bot list + shadowban, CSV export.

---

## Why these ideas hit the 70% mark

People come to FicHub to **get stories onto their devices**. The list above expands that core loop:

- **Get** → more sites, batch, queue, send to device
- **Organise** → shelves, status, notes, library search
- **Discover** → personalised recs, update notifications, similar fics
- **Read** → web reader, progress sync, offline PWA

Everything else (community, admin tools) sweetens the deal but isn't required for most. The starred items alone would turn FicHub from a handy converter into a central piece of someone's reading life.


======================================================================
SOURCE: NEXT.md
======================================================================

# FicHub — NEXT.md (current work item)

> **CONSOLIDATED**: the canonical planning doc is now **docs/ROADMAP.md**
> (shipped / in-flight / prioritized suggestions / test coverage /
> deployment). This file tracks the single current work item (self-healing
> flagship flow).
>
> Updated: 2026-08-11 — self-healing for scraping. User-clarified flagship
> flow: on scrape failure, call the Hermes agent → it creates a scraper ON
> THE FLY → the site uses it immediately to complete the request → periodic
> rebuild folds generated scrapers into the binary. Safety-gated throughout.
> Work on this; remove this file when done.

## The flagship flow (user's words, clarified)
> "Whenever a user scrapes a fic and it fails, the website makes a call to
> the hermes agent to fix it, it creates a new scraper on the fly and uses
> it. Then from time to time I rebuild the site including those scrapers.
> Only do this if safe."

The site is ALSO a cache of all gathered fanfiction: every scraped body is
persisted as a blob on the attached drive (`/public/literature/fichub/bodies`),
so exports don't re-scrape and curators can correct wrong bodies after the
fact (see "Curator content fix" below).

Concretely, on a `structural` scrape failure (parse/selector mismatch):

1. **Capture** — the failed HTML snapshot + error land in `scrape_failures`
   (milestone 1, in progress).
2. **Agent call** — the site calls the Hermes agent (CommandCode API,
   `deepseek/deepseek-v4-flash` by default) with the snapshot + site context.
   The agent returns structured `FicMetadata` (title/author/chapters/tags)
   extracted from the snapshot — i.e. a **scraper on the fly**, no recompile
   needed.
3. **Use it now** — the site validates the returned metadata and completes
   the user's original export request immediately. The response is cached
   like any normal scrape.
4. **Persist** — the agent's extraction (plus the site profile it inferred)
   is recorded (agent_runs + a `heal_scraper_overrides` table later) so a
   periodic **rebuild** can fold it into the native Rust scraper
   (`src/scrape/sites/<domain>.rs`) with a regression fixture.
5. **Safety gates** — only `structural` failures trigger the agent; blocked
   (403/Cloudflare) and transient never do. Agent output is treated as
   untrusted: validated against FicMetadata shape, all fields sanity-checked
   (title length, chapters>0, URL matches), and never written to the
   filesystem from the running binary. Autonomy ladder: diagnose-only →
   use-on-the-fly-with-validation → fold-into-rebuild (the last one is a
   build-time action, human-triggered).

## Scope

### Milestone 1 — failure telemetry + agent plumbing (IN PROGRESS, subagent)
1. **Migration 029** `scrape_failures` + **030** `agent_runs`.
2. Classifier (transient/blocked/structural/systemic) + fingerprint +
   debounce.
3. Instrument `ScraperRegistry` dispatch + `routes/export.rs`; HTML snapshot
   capture on parse failures.
4. `POST /api/admin/heal?domain=X` — diagnose-only, CommandCode agent call.
5. Config: `AGENT_ENABLED`, `AGENT_MODEL`, `AGENT_API_KEY`, `AGENT_API_URL`,
   `AGENT_OLLAMA_URL`, budgets/cooldowns.

### Milestone 2 — on-the-fly scraper (user's flagship, SHIPPED 2026-08-16)
- On structural failure with `AGENT_ENABLED=true` + `AGENT_USE_ON_FLY=true`:
  the agent extracts metadata from the HTML snapshot → validates it
  (`parse_agent_metadata_json`) → completes the pending export (re-fetches
  chapters using the extraction as a hint; last resort = single chapter
  from the snapshot) → records the generated extraction in `agent_runs`
  (`heal_extract` trigger, status `proposed`).
- `AGENT_USE_ON_FLY=false` default; true = use-on-the-fly-with-validation.
- Admin review: `GET /api/admin/heal/extractions` (untrusted queue) →
  `POST /api/admin/heal/extractions/{id}/trust` (only trusted extractions
  are reused via `find_trusted_extraction_for_url` for replay).
- Pending-request queue (migration 033 `pending_exports`): transient/blocked
  failures persist the user request; `POST /api/admin/heal/replay-pending`
  retries them (completed on success, attempts+1 on failure).
- Migrations: **033** `pending_exports`, **034** `heal_extractions`.
- Config: `AGENT_USE_ON_FLY` (default false), `AGENT_EXTRACT_MAX_SNAPSHOT_CHARS`
  (default 120000).
- `src/heal/extract.rs` — extraction service (parse/validate/store/list/
  trust/find/replay + FicMetadata conversion).

### DONE — body/HTML cache + curator voting (2026-08-11)
- `src/body_cache.rs`: scraped bodies cached as sharded versioned files
  under `BODY_CACHE_DIR` (default `/public/literature/fichub/bodies`):
  `{url_id[0:2]}/{url_id[2:4]}/{url_id}.v{N}.html` (extracted HTML) +
  `.json` (structured chapters). Exports reuse cached body; version bumps
  GC old files. Config `BODY_CACHE_DIR`.
- **Curator fixes require peer voting** (migration 032): propose → other
  curators vote → applied only at quorum (≥2 votes, net ≥ 1). No self-vote.
  Endpoints: POST propose, POST vote, GET proposals (review queue), GET
  inspect, DELETE force-re-scrape.
- Tests: body_cache 5/5 unit, curator_fix_api 2/2 DB-gated, heal_api 8/8.

### Milestone 3 — periodic rebuild folds generated scrapers
- Tool/script (`scripts/fold-scrapers.sh` + docs): read agent_runs where the
  agent produced a site profile, emit a candidate `src/scrape/sites/<domain>.rs`
  + fixture test into a worktree → run tests → propose for human merge.
- Human-triggered only (rebuild is a build-time action).

### Not in scope (later)
- T2 service heal (health watcher → auto-restart), T3 QA-driven regen,
  `/admin/agent` UI panel.

## Test gate
- Classifier unit tests, fingerprint, debounce.
- DB-gated `heal_api`: record_failure, classify, dedupe, admin heal trigger
  (diagnose-only), agent-unavailable → failed run, on-the-fly validation
  (milestone 2), pending-request replay.
- Existing suites stay green.

## Docs to keep current
- docs/AGENTS.md, docs/src/* (user-facing mdBook), docs/FORUM-API-CONTRACT.md,
  docs/SPEC-COMMUNITY-PLATFORM.md, docs/THREADLIGHT-FEATURES.md,
  docs/FORUM-BRAINSTORM.md (idea pool, ranked), this file.


======================================================================
SOURCE: docs/USER-ACTIONS.md
======================================================================

# FicHub — Actions Needing You (from the ROADMAP audit)

*Generated 2026-08-11 after the full ROADMAP completion + e2e verification
pass. Everything below is either ops work that needs your hands/keys, or a
product decision only you can make. Everything scriptable that was worth
doing has been done (see the summary at the end).*

---

## 1. Secrets hygiene (P2#6) — ~15 min

Move the DB password out of `.env` into a 600-perm file via systemd
`EnvironmentFile`. Currently the deploy box's `fichub.service` reads
`.env` from the repo directory, which is on NFS and readable by any user
with repo access.

**Steps (on the deploy box, ThinkCentre):**

```bash
# 1. Create a secrets file with tight permissions
sudo mkdir -p /etc/fichub
sudo touch /etc/fichub/secrets.env
sudo chmod 600 /etc/fichub/secrets.env
sudo chown root:root /etc/fichub/secrets.env

# 2. Move the secrets into it (edit with sudo nano/vim)
#    DATABASE_URL=...  (the full postgres:// URL with password)
#    JWT_SECRET=...    (the live signing secret)
sudo nano /etc/fichub/secrets.env

# 3. Update the systemd unit to load it
sudo systemctl edit fichub.service
#   [Service]
#   EnvironmentFile=/etc/fichub/secrets.env
#   # (remove the same vars from the old .env reference or keep .env for the
#   #    non-secret knobs only — BODY_CACHE_DIR, REC_ENGINE_MODE, etc.)

# 4. Remove the secrets from the repo .env (keep the non-secret knobs)
#    sed -i '/^DATABASE_URL=/d; /^JWT_SECRET=/d' /personal/documents/code/rust/fichub/.env

# 5. Reload + restart + verify
sudo systemctl daemon-reload
sudo systemctl restart fichub.service
curl https://fichub.polarisocial.xyz/api/health   # expect ok/db:true/redis:true
```

## 2. External uptime probe + alerts (P2#5) — 30 min, free

The internal Forgejo is unreachable, so no CI/CD. The minimum viable
safety net is an external uptime check alerting to Telegram when health
flakes.

**Option A — UptimeRobot (simplest, free):**
1. Create an account at uptimerobot.com.
2. Add a monitor: `https://fichub.polarisocial.xyz/api/health` — expect
   HTTP 200, check every 5 min.
3. Add an alert contact: Telegram bot (they give you a bot token to
   configure), or email.
4. Also add a keyword monitor on the JSON body `"ok"` — that catches the
   case where the server returns 200 but DB/Redis are down (the body says
   `"db":false`).

**Option B — Hermes cron (if you want it fully local):**
```bash
# In a Hermes session with the Telegram gateway connected:
hermes cron create "every 5m"
# prompt: "curl https://fichub.polarisocial.xyz/api/health; if the body
# doesn't contain \"ok\" or db/redis aren't true, send an alert."
# deliver: telegram
```

## 3. Backup the body cache + EPUBs (P2#7) — 20 min + disk

`/public/literature/fichub/bodies` is the most valuable data asset — every
scraped fic, on disk, in the attached drive's pool. It is NOT backed up
anywhere today.

**Recommended: a nightly rsync to the SANDISK USB or another machine.**

```bash
# On the deploy box (or a cron there):
#!/bin/bash
# /etc/fichub/backup-body-cache.sh
set -euo pipefail
DEST="${BACKUP_DEST:-/mnt/backup/fichub}"
mkdir -p "$DEST"
rsync -a --delete --info=progress2 \
  /public/literature/fichub/bodies/ "$DEST/bodies/"
rsync -a --info=progress2 \
  /public/literature/fichub/*.epub "$DEST/epubs/" 2>/dev/null || true
echo "backup ok $(date -Is)" >> /var/log/fichub-backup.log
```

Then `chmod +x` + `crontab -e`:
```
0 3 * * * /etc/fichub/backup-body-cache.sh
```

(First run will copy the whole tree; subsequent runs are incremental.
SANDISK USB is at `/media/alvaro/SANDISK` on the dev box — a weekly manual
`rsync` from ThinkCentre to it also works.)

## 4. User-supplied cookie ingestion for AO3/FFN (P1#1) — product decision

The host is blocked at the outbound level: AO3 404+bot-challenge / FFN 403 from the
server. The scraper crate now covers all 107 FFF sites natively (full
adapter parity, v0.10.0), but the *host's outbound* blocking still limits
what the archive can actually ingest from AO3/FFN. The pragmatic fix that
fits the current architecture: a **cookie import flow** — a user opens a
blocked page in their own browser once, pastes the `cf_clearance` /
session cookie into the site, and the scraper reuses it for that site. The
body cache + curator peer-voted fixes make failed scrapes recoverable, so
this is low-risk.

This is a **design + scope decision** (who can submit cookies, expiry,
per-site vs global), so I did not build it unilaterally. Say the word and
I'll implement it: a `scraper_cookies` table (url pattern, cookie header,
added_by, expires_at), a `/settings/cookies` page, and a per-site cookie
injection in the registry lookup.

## 5. Self-healing M2 — on-the-fly scraper generation (safety-gated)

The telemetry + classifier (M1) are live; the agent loop that CREATES a
new scraper on structural failure is documented but OFF (`AGENT_ENABLED=false`).
Enabling it requires the safety review in NEXT.md — it's the one genuinely
autonomous write path in the system. Not flipped on without your explicit
go-ahead.

## 6. Rec platform shadow-run (P3#8) — decision + one config flip

`decay` + `embeddings` strategies are built and covered by the golden
test. To shadow-run them:

```bash
# On the deploy box .env (or /etc/fichub/secrets.env after step 1):
# REC_SHADOW_MODE=true
# REC_STRATEGIES=cooccur:1.0,decay:0.0,embeddings:0.0
#   (all-zero weight except cooccur means "compute, don't serve")
```

Read `rec_impressions` after a week and promote the winner by flipping
weights. I can set this up + monitor it if you want.

---

## What's already DONE (no action needed)

- **API route-walk e2e** (`qa/api-walk.js`) — 97/97 endpoints green; caught
  + fixed 2 real 500s (reading-stats, analytics user stats).
- **Fic Requests M3** — upvotes (migration 035) + notifications on answer/
  accept, shipped.
- **Fic Requests URL-ingest** — paste a URL to answer a request; scraper
  resolves + find-or-creates the work.
- **Admin UI** — translation review (`/admin/translations`) + metadata
  correction (`/admin/metadata`) pages built + tested.
- **Verified already-existing** (stale ROADMAP items): auto-tag review,
  fic approval, blacklist UI, main_char_attr search UI, search→export
  conversion.
- **Docs** — ROADMAP.md updated with the audit-wave shipped list + this
  file.


======================================================================
SOURCE: scripts/consensus_seed.md
======================================================================

# FicHub Consensus Feature Seed

Derived from the 60 open+closed issues on GitHub (FicHub/fichub.net), grouped into
feature clusters for the community consensus voting feature. Users vote on what they
want most/least; each cluster references its source issues.

## 1. Add more supported fanfic sites
Support MediaMiner.org, ScribbleHub, Literotica, SpiritFanfiction, AlternateHistory,
fimfiction.net, tthfanfic.org, and FanficParadise forums.
*(Issues: #51, #48, #44, #36, #17, #4, #39)*

## 2. Reliable story updates & cache refresh
Files don't refresh when a fic is updated without new chapters; provide a way to
invalidate cache for a story and auto-check for updates.
*(Issues: #54, #30, #18, #23, #28, #26, #14, #1, #3, #46)*

## 3. More export formats: Markdown, text, ODT, DOCX
Add Markdown, plain text, ODT and DOCX export options in addition to
EPUB/PDF/MOBI/HTML. *(Issue: #29)*

## 4. Fix HTML export formatting bugs
HTML exports mangle dashes (em/en -> hyphen), drop chapter titles, lose
underline/formatting, and include garbage tags.
*(Issues: #32, #22, #42, #47)*

## 5. Improve EPUB quality
Fix random spaces, support book-style paragraph indentation, and preserve chapter
ordering when chapters are inserted out of sequence.
*(Issues: #6, #21, #45, #46)*

## 6. Fix download failures & blank exports
Address blank downloads, PDF export failures on fanfiction.net, 503 errors, and
'response was not valid' conversion errors.
*(Issues: #50, #57, #49, #40, #53, #52)*

## 7. Enable CORS on API for userscripts
Return Access-Control-Allow-Origin: * on API endpoints so bookmarklets and
userscripts can call FicHub directly. *(Issue: #41)*

## 8. Expose cover images via API
Add cover image support to the API for frontends and third-party apps.
*(Issue: #35)*

## 9. Download entire series (AO3)
Support downloading a whole AO3 series, not just individual works.
*(Issue: #25)*

## 10. Bulk mirroring / database import
Allow bulk mirroring of databases and batch submission of stories FicHub has
trouble with. *(Issues: #8, #10)*

## 11. Wayback Machine fallback
Use Wayback Machine to fetch stories from already-supported sites when the live
site blocks or fails. *(Issues: #37, #27)*

## 12. XenForo threadmark support
Handle Apocrypha, Side Story, and Informational threadmarks on
SpaceBattles/Sufficient Velocity and XenForo forums.
*(Issues: #33, #38, #39)*

## 13. Metadata & description improvements
Include story descriptions in exports and fix wrong author associations.
*(Issues: #19, #7, #2)*

## 14. Contributing guide & docs
Explain how to contribute to the project and document setup requirements.
*(Issues: #43, #9)*

## 15. Cosmetic fixes
Fix timezone display and chapter numbering issues. *(Issues: #13, #15, #16)*
