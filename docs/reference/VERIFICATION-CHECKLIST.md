# Verification Checklist

> **Purpose:** Step-by-step verification list for every shipped area. Feed this
> (or sections of it) to another chatbot/QA agent to ask about issues one area
> at a time. Each item is a concrete, independently checkable question.

**Legend:**
- `✅` verified working
- `🔲` needs verification
- `⚠️` known issue / needs attention

---

## Section 1: Route Inventory

All frontend routes (`frontend/src/routes/**/+page.svelte`) and their archive-mode
status. Each route should return HTTP 200 and render without errors in both
archive and modern mode.

### 1.1 Forum routes
| Route | Archive-style? | Verified 200? | Notes |
|---|---|---|---|
| `/forum` | ✅ rewritten with archive-main | ✅ 200 | Category listing: archive table dl format |
| `/forum/general` (or any category) | ✅ rewritten | ✅ 200 | Topics listed in dl format, archive-style pagination |
| `/forum/new` | ✅ rewritten cleanly | ✅ 200 | fieldset.dl form, no syntax errors |
| `/forum/board/{slug}.{id}` | ✅ fixed +page.svelte + page-head hidden | ✅ 200 | TopicThread: page-head hidden via `.hidden-archive`, back link added |
| `/forum/search` | — | ✅ 200 | |
| `/forum/apply` | — | ✅ 200 | |
| `/forum/blocks` | — | ✅ 200 | |
| `/forum/invites` | — | ✅ 200 | |
| `/forum/moderate` | — | ✅ 200 | |
| `/forum/metamod` | — | ✅ 200 | |

**Key checks for forum:**
- `topicHref()` in `[categorySlug]/+page.svelte` generates `/forum/board/${tp.topic_slug}.${tp.id}`
- API returns `topic_slug` field for each topic
- Clicking a topic opens TopicThread.svelte with content visible
- Back link in TopicThread archive header resolves correctly
- `.hidden-archive` CSS rule applied to `page-head` (hides in archive mode)

### Table 1.2: Frontend route list
| Route | Has +page.svelte | Archive mode? | page.test.ts? | Verified 200? |
|---|---|---|---|---|
| `/` | ✅ | — | — | ✅ |
| `/ask` | ✅ | — | — | ✅ |
| `/people` | ✅ | ✅ | — | ✅ |
| `/authors` | ✅ | — | — | ✅ |
| `/authors/[name]` | ✅ | — | — | ✅ |
| `/badges` | ✅ | — | — | ✅ |
| `/blind-date` | ✅ | — | — | ✅ |
| `/bookmarks` | ✅ | ✅ | — | ✅ |
| `/collections` | ✅ | — | — | ✅ |
| `/collections/[id]` | ✅ | — | — | ✅ |
| `/curator` | ✅ | — | — | ✅ |
| `/curator/flags` | ✅ | — | — | ✅ |
| `/dashboard` | ✅ | — | — | ✅ |
| `/docs` | ✅ | ✅ | — | ✅ |
| `/download` | ✅ | ✅ | — | ✅ |
| `/fandoms` | ✅ | ✅ | — | ✅ |
| `/fandom/[slug]` | ✅ | ✅ | — | ✅ |
| `/feed` | ✅ | — | — | ✅ |
| `/follows` | ✅ | — | — | ✅ |
| `/history` | ✅ | — | — | ✅ |
| `/leaderboard` | ✅ | — | — | ✅ |
| `/lists` | ✅ | — | — | ✅ |
| `/lists/[id]` | ✅ | — | — | ✅ |
| `/modlog` | ✅ | ✅ | — | ✅ |
| `/notifications` | ✅ | — | — | ✅ |
| `/people` | ✅ | ✅ | — | ✅ |
| `/quests` | ✅ | — | — | ✅ |
| `/reading` | ✅ | — | — | ✅ |
| `/read/[urlId]` | ✅ | — | — | ✅ |
| `/recommendations` | ✅ | — | — | ✅ |
| `/requests` | ✅ | — | — | ✅ |
| `/requests/new` | ✅ | ✅ | — | ✅ |
| `/requests/[id]` | ✅ | — | — | ✅ |
| `/roadmap` | ✅ | ✅ | — | ✅ |
| `/search` | ✅ | ✅ | — | ✅ |
| `/search/body` | ✅ | — | — | ✅ |
| `/search/syntax` | ✅ | — | ✅ | ✅ |
| `/search/tags` | ✅ | ✅ | ✅ | ✅ |
| `/series/[id]` | ✅ | — | — | ✅ |
| `/settings` | ✅ | — | — | ✅ |
| `/settings/recipes` | ✅ | — | — | ✅ |
| `/settings/theme` | ✅ | — | — | ✅ |
| `/shelves` | ✅ | — | — | ✅ |
| `/shelves/[id]` | ✅ | — | — | ✅ |
| `/stats` | ✅ | ✅ | — | ✅ |
| `/tags` | ✅ | ✅ | — | ✅ |
| `/trending` | ✅ | ✅ | — | ✅ |
| `/updates` | ✅ | — | — | ✅ |
|| `/work-proposals` | ✅ | ✅ | ✅ | ✅ |
|| `/work-proposals/new` | ✅ | ✅ | ✅ | ✅ |
|| `/works/[urlId]` | ✅ | ✅ | ✅ | ✅ |
|| `/upload` | ✅ | ✅ | ✅ | ✅ | (NEW — Import Work file ingestion)
|| `/my-works` | ✅ | — | ✅ | ✅ | (NEW — My Works = own uploads only)
|| `/bookmarks/search` | ✅ | ✅ | ✅ | ✅ | (NEW — redirects to /bookmarks?tab=search)

### 1.3: Admin / curator routes (auth-gated)
| Route | page.test.ts? | Notes |
|---|---|---|
| `/admin` | — | Level-gated |
| `/admin/analytics` | — | |
| `/admin/auto-tag` | — | |
| `/admin/blacklist` | — | |
| `/admin/bots` | — | |
| `/admin/bulk-actions` | — | |
| `/admin/comment-triage` | — | |
| `/admin/content-scan` | — | |
| `/admin/metadata` | — | |
| `/admin/moderation` | — | |
| `/admin/scrapers` | — | |
| `/admin/stats` | — | |
| `/admin/translations` | — — | |
| `/admin/users` | — | |
| `/admin/features` | — | |
| `/admin/features/analytics` | — | |
| `/curator/authors` | — | |

---

## Section 2: Archive-Mode Consistency

These checks verify that the AO3 archive style is consistent across all pages.

- [ ] Every page renders inside `<main class="archive-main">` in archive mode
- [ ] Every page has a `<header class="archive-header">` with an `<h1 class="archive-page-title">`
- [ ] Every page has a `<div class="archive-content archive-page">` wrapper
- [ ] No page renders `ArchiveHeader` or `ArchiveFooter` as a component import (they come from `ArchiveLayout` in `+layout.svelte`)
- [ ] No page renders a plain `<header>`/`<footer>` in archive mode (only modern mode)
- [ ] `page-head` elements in TopicThread and other archive pages are hidden via `class:hidden-archive` when `uiMode === 'archive'`
- [ ] `.hidden-archive { display: none !important; }` exists in the CSS (zerafina-skin.css + any component-level styles)
- [ ] Tag limits: no 75-tag limit per work; WorkBlurb truncates at 5 tags per category with "+N more"; full tags on work detail page
- [ ] `/works/{urlId}` route used instead of `/fic/{urlId}` everywhere
- [ ] No emoji in UI text (⚠️, 👍, 💬, ✅ removed from all pages)
- [ ] All `href="/path/{var}"` literal strings are now `href={`/path/${var}`}` (template literals)

---

## Section 3: Navbar & Dropdown Structure

Verify the ArchiveHeader dropdowns are correct:

- [x] **Browse** dropdown contains: Trending, Bookmarks (logged-in only), Fandoms, Lists, Series, Tags, Rankings (leaderboard)
- [x] **Search** dropdown contains: Work Search (`/search`), People Search (`/people`), Tag Search (`/search/tags`), Bookmark Search (`/bookmarks?tab=search`)
- [x] **About** dropdown contains: Docs, Roadmap, Modlog, Statistics, Transparency
- [x] **Post** dropdown: "Import Work" → `/upload` (file ingestion), "Post Work" → `/work-proposals/new` (AO3-style new work form)
- [x] Bookmarks unified: both Browse>Bookmarks (`/bookmarks`) and Search>Bookmark Search (`/bookmarks?tab=search`) are tabs on the same page
- [x] LeaderBoard now reachable via Browse > Rankings
- [x] Curator/Admin nav gated on `auth.level >= 10` / `auth.level >= 100`
- [x] Notifications bell removed from top-right (only in user greeting dropdown)
- [x] All dropdown links use `.archive-dd-item` class
- [x] CSS hover dropdowns work (`.dropdown` + `.dropdown-menu` pattern)
- [x] Footer rewritten with AO3 textured-red archive-footer, locale selector, theme switch
- [x] `zerafina-skin.css` loaded via `ArchiveLayout.svelte` and `skin-zerafina` class on `<body>`
- [x] `ArchiveEmpty.svelte` shared component for empty/loading/error states
- [x] `ArchiveBreadcrumbs.svelte` added for work/pages breadcrumbs

---

## Section 4: Search Features

- [x] `/search` — Work search: modern mode hidden behind `{#if uiMode !== 'archive'}`, archive mode shows `ArchiveWorkSearchForm`
- [x] `/search/tags` — Tag search: AO3-style form with Tag name, Type (7 options), Wrangling status (7 options incl. non-canonical non-syn), Fandom, Sort by, Sort direction
- [x] `/search/tags` results table renders with pagination
- [x] `/search/tags` — `searchTagsAdvanced()` API client works with `q`, `tag_type`, `canonical`, `fandom`, `page`, `limit`, `sort` params
- [x] `/search/tags/page.test.ts` — 5 tests PASS (form render, fandom field, wrangling options, search+results, modern mode)
- [x] `/people` — People search: archive mode with AO3-style search form, alphabetical browse, PeopleBlurb results
- [x] `/bookmarks/search` — Bookmark search: redirects (308) to `/bookmarks?tab=search` (tabs of unified bookmarks page)
- [x] Search dropdown "Edit Your Search" subnav link appears after results
- [x] Search form field matrix (INTERFACE-STYLES.md) matches actual implementation
- [x] Search bar 700ms debounce to prevent burst API calls (hardening)

---

## Section 5: Forum Specific Checks

- [ ] Forum category page `/forum/[categorySlug]` lists topics from API
- [ ] `getForumTopics(categorySlug)` returns 5 topics with `topic_slug` field
- [ ] API endpoint `/api/forum/topics?category=general` returns correct data
- [ ] API endpoint `/api/forum/categories` returns "General" (slug: general, id: 1)
- [ ] API endpoint `/api/forum/topics/by-slug/{slug}` returns topic data
- [ ] Topic URLs: `href={`/forum/board/${tp.topic_slug}.${tp.id}`}` uses template literal, not literal string
- [ ] Board route `/forum/board/[topicSlug].[topicId]/` `+page.ts` resolves slug to topic
- [ ] Board route `+page.svelte` passes resolved `topicId` to `TopicThread`
- [ ] `TopicThread` calls `auth.init()` in `onMount` before rendering
- [ ] TopicThread archive mode: `arc-topic-head` back link href resolves to `/forum/${categorySlug}`
- [ ] `TopicThread.svelte` — `page-head` hidden in archive mode via `.hidden-archive`
- [ ] `forum/new/+page.svelte` — no syntax errors, archive mode uses `fieldset.dl`
- [ ] Forum category `page.test.ts` exists and tests pass (6 tests)

---

## Section 6: Statistics & Roadmap

- [x] `/stats` page: all `href` are valid template literals (no extra backticks)
- [x] `/stats` page has `archive-main` wrapper
- [x] `/stats` page: `{#if auth.isLoggedIn && stats}` properly closed with `{/if}`
- [x] `/stats` page: `dl.work.meta` layout in archive mode
- [x] `/roadmap` page: feature ranking toggle button works in archive mode
- [x] `/roadmap` page: leaderboard shows Elo scores after toggle
- [x] `/modlog` page: no duplicate ArchiveHeader/ArchiveFooter
- [x] `/docs` page: all doc section links resolve (no broken hrefs)

---

## Section 7: Tag System

- [x] Tag type IDs: `fandom=1, warning=2, category=3, character=4, relationship=5, additional=6`
- [x] Tags page `/tags`: `switchMode` fetches all 6 types (1-6) with collision checking
- [x] Tags page uses `Promise.all` for parallel fetching
- [x] Tags cloud: weighted (`cloud1`–`cloud8`) by popularity + opt-in personalization (`personalizedRecs` pref gates data fetch, boosts liked-tag sizes)
- [x] Tag search API `/api/tags/search` accepts: name, fandom, tag_type, wrangling_status, sort_by, sort_direction
- [x] `searchTags()` and `searchTagsAdvanced()` in `src/lib/api/tags.ts`
- [ ] `/tags/search` standalone tag cloud route (planned, not shipped — cloud lives on /tags)

---

## Section 8: Build & Deploy (FINAL — 2026-08-21 22:00)

- [x] `cargo check` — 0 errors, 0 warnings (dev profile)
- [x] `cargo build --release` — succeeds (clean rebuild from `/home/alvaro/code/rust/fichub`, 1m20s incremental)
- [ ] `cargo sqlx prepare --check` (skipped — no schema migrations in this cycle)
- [x] Frontend build from `/tmp/fichub-frontend` (NFS-safe) succeeds via `npm run build`
- [x] `rsync -avz --delete` deploys build to ThinkCentre
- [x] Backend binary deployed to `/opt/fichub/fichub` via `scp → /tmp → systemctl stop → cp → systemctl start`
- [x] `fichub.service` active and running (PID confirmed)
- [x] ThinkCentre server responds on `localhost:8000`
- [x] All 24 routes return HTTP 200 (verified: `/leaderboard /upload /my-works /tags /bookmarks /bookmarks?tab=search /bookmarks/search /docs /stats /dashboard /history /settings /forum /forum/general /search/tags /people /search /requests /roadmap /lists /series /authors /blind-date /feed /updates /profile /curator /admin`)
- [x] API endpoints verified: `/api/reading/history` → 401 JSON `{"err":401,"msg":"Login required"}` (was 500), `/api/my-works` → 401 (NEW), `/api/tags/search` → 200, `/api/forum/topics?category=general` → 200
- [ ] `svelte-check` errors documented (498 pre-existing TS errors, not blocking builds)
- [x] Browser cache: immutable hashed assets (1yr cache) — users may need hard-refresh after deploy
- [x] NFS clean: timestamp junk files deleted, `.svelte-kit`/`node_modules/.vite` removed after builds

---

## Section 9: Known Issues & Gotchas

- [ ] **NFS build recovery**: Builds must run from `/tmp/fichub-frontend/`, never from the NFS-mounted `frontend/` directory (corruption on `.svelte-kit` and `node_modules/.vite`)
- [ ] **Git on NFS**: Requires `GIT_OBJECT_DIRECTORY=~/.cache/git-objects-fichub` + `GIT_ALTERNATE_OBJECT_DIRECTORIES=<repo>/.git/objects`
- [ ] **Honeypot**: Register endpoint requires `form_opened_at` + `website:''` — bare curl fails silently
- [ ] **sqlx inet**: PostgreSQL `inet` columns bound via string + `$N::inet` cast
- [ ] **psql ownership**: All tables need `ALTER OWNER TO fichub`
- [ ] **svelte-check**: 498 pre-existing TS errors — these are type-checking issues, not runtime errors
- [ ] **Cache busting**: After frontend deploy, old UI = browser cache (immutable hashed assets); force refresh or clear cache

---

## Section 10: Test Files

| Test file | Tests | Status |
|---|---|---|
| `frontend/src/routes/search/tags/page.test.ts` | 5 (form render, fandom field, wrangling options, search+results, modern mode) | ✅ 5/5 PASS |
| `frontend/src/routes/my-works/page.test.ts` | 1 (renders My Works heading in archive mode) | ✅ PASS (NEW) |
| `frontend/src/routes/forum/[categorySlug]/page.test.ts` | 6 | ✅ PASS |
| `frontend/src/routes/forum/board/[topicSlug].[topicId]/page.test.ts` | 1 | ✅ PASS |
| `frontend/src/lib/stores/auth.test.ts` | ? | — |
| `frontend/src/lib/stores/auth-flows.test.ts` | ? | — |
| `frontend/src/lib/api/social.test.ts` | ? | ✅ hardening: jsdom mock fix (56abebe) |
| Rust tests | 582 lib + ~170 DB-gated | ✅ green |
| Scraper tests | 200+ | ✅ green |
| QA harness (`node qa/run.js`) | Full suite | ✅ green |
| API audit (`node qa/api-walk.js`) | 97/97 | ✅ green |

**Note:** `npx vitest run` reports 38 failures in pre-existing tests unrelated to this cycle (stale `fic/[urlId]` route tests, jsdom `prefs` tests). NOT introduced by this batch. New/changed routes all green.

---

## Section 11: 2026-08-23 Pass — New Endpoints & UI

| Endpoint / Route | Expected | Notes |
|---|---|---|
| `GET /api/docs/list` | 200 JSON `{"err":0,"docs":[...]}` | public, no auth; backs /docs hub (581c751) |
| `GET /api/docs/view?name=USER-ACTIONS.md` | 200 JSON with `markdown` | rejects `..`/`/`/non-.md (400) |
| `GET /api/curator/consensus?type=&status=&sort=&q=` | 200 (anon 401, logged-in 200) | modlog-style public-read for authed users (f7a0f1e, 66ac13e) |
| `GET /curator/consensus` (frontend) | 200 archive page | filter chips + search + status chips |
| `GET /api/search/history/chips` | 200 (anon 401 or empty, authed returns top-8) | P6#23 — migration 063, working tree |
| `GET /opds/manifest` | 200 `application/webpubmanifest+json` | RWPM manifest with 6 navigation links |
| `GET /opds` (root feed) | 200 Atom | must contain `rel="alternate" type="application/webpubmanifest+json" href="/opds/manifest"` |
| `POST /api/requests/{id}/answers` with `search_query` | 200, persists `answer_kind=search` | migration 062 (6652aef) |
| `GET /api/requests/{id}/answers` | 200 list with `answer_kind` per answer | work|search|llm |
| Symbol squares on `WorkBlurb` + work page | 4 bordered squares, cursor:help, click→guide modal | `symbols.ts` + `SymbolSquares.svelte` (working tree) |

---
