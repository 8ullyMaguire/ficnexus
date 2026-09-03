# Bot + CLI Feature Parity Gap Analysis

Tracks which FicHub site features are missing from the Discord companion bot (`fanfic-archivist/`) and the (not-yet-existing) CLI tool. Updated 2026-08-25.

## Bot API coverage (what fanfic-archivist already supports)

| Area | Bot |
|------|-----|
| Export | `/api/epub`, `/api/meta`, `/api/epub/convert` ✅ |
| Recommendations | `/api/recommendations`, `/api/recommendations/personal`, `/api/recommendations/strategies` ✅ |
| Search | `/api/search`, `/api/search/ask`, `/api/search/body` ✅ |
| Social | Bookmarks (add/remove), ratings (rate work), blocks (add/remove) ✅ |
| Kudos | `/api/kudos/{work_id}` (GET only — count display) ✅ |
| Auth | `/api/auth/login`, `/api/auth/me`, `/api/auth/refresh` ✅ |
| Fic Requests | `/api/requests` (list/create) ✅ |
| Roadmap | `/api/roadmap/consensus` ✅ |
| Feed | `/api/v1/feed` (updates) ✅ |

## Missing from bot (site features not yet wired)

### Missing from frontend API client too (parity gap for both)

| # | Feature | API endpoint | Priority |
|---|---------|-------------|----------|
| 1 | **Send-to-Kindle** | `POST /api/send-to-kindle` | P2 |
| 2 | **OPDS catalog** | `/opds/*` (full catalog) | P3 |
| 3 | **User data export** | `GET /api/user/export` | P2 |
| 4 | **User site credentials** | `/api/user/site-credentials` | P2 |

### Frontend has, bot lacks

| # | Feature | Frontend API | Bot status | Priority |
|---|---------|-------------|------------|----------|
| 5 | **Work detail / single fic** | `GET /api/works/{id}` | Not in bot API client | P1 |
| 6 | **Work stats** | `GET /api/works/{id}/stats` | Not in bot API client | P2 |
| 7 | **User profile** | `GET /api/users/{id}` | Not in bot API client | P2 |
| 8 | **Curator leaderboard** | `GET /api/leaderboard/curators` (global + weekly + monthly) | Not in bot | P2 |
| 9 | **Badges** | `GET /api/badges`, `GET /api/users/{id}/badges` | Not in bot | P3 |
| 10 | **Shelves** | `POST/GET /api/shelves`, add/move/remove works | Not in bot | P3 |
| 11 | **Reading lists (bundles)** | `POST/GET/PUT/DELETE /api/lists` | Not in bot | P3 |
| 12 | **Reading status** | `POST /api/reading/status`, `GET /api/reading/list` | Not in bot | P3 |
| 13 | **Quests** | `GET /api/quests` | Not in bot | P3 |
| 14 | **Reading stats** | `GET /api/users/{id}/reading-stats` | Not in bot | P3 |
| 15 | **Streaking** | `GET /api/users/{id}/streak` | Not in bot | P3 |
| 16 | **Reading history** | `GET/POST/DELETE /api/reading/history*` | Not in bot | P3 |
| 17 | **Reviews** | `GET /api/works/{id}/reviews`, `POST /api/reviews`, `DELETE /api/reviews/{id}` | Not in bot | P2 |
| 18 | **Threaded comments** | `GET/POST /api/works/{url_id}/comments`, edit/hide/delete | Not in bot | P3 |
| 19 | **Notifications** | `GET /api/notifications*`, prefs endpoints | Not in bot | P2 |
| 20 | **Search suggestions** | `GET /api/search/suggest` | Not in bot | P2 |
| 21 | **Similar works** | `GET /api/search/similar/{work_id}`, `GET /api/works/{id}/similar` | Not in bot | P2 |
| 22 | **Random work (surprise me)** | `GET /api/works/random` | Not in bot | P2 |
| 23 | **Readers also bookmarked** | `GET /api/v1/works/{url_id}/also-bookmarked` | Not in bot | P3 |
| 24 | **Blind date** | `GET /api/blind-date`, `/api/blind-date/reveal` | Not in bot | P3 |
| 25 | **Fic suggestions (community)** | `GET /api/fic-suggestions`, vote/remove | Not in bot | P3 |
| 26 | **Work proposals** | `POST/GET /api/work-proposals`, vote | Not in bot | P3 |
| 27 | **Trending** | `GET /api/trending`, `/api/trending/tags`, by-tag | Not in bot | P2 |
| 28 | **Fandom routes** | `GET /api/fandoms`, `GET /api/fandoms/{slug}` | Not in bot | P3 |
| 29 | **Tag routes** | `GET /api/tags`, search, autocomplete, detail | Not in bot | P3 |
| 30 | **Recipe Builder** | `GET/POST /api/recipes*` | Not in bot | P4 |
| 31 | **Modlog** | `GET /api/modlog` | Not in bot | P3 |
| 32 | **User follows** | `GET /api/follows`, check, followers | Bot has `/api/v1/feed` only | P2 |
| 33 | **User-level blocks** | `POST /api/blocks` (user-level, not fic-level) | Bot blocks by url_id only | P3 |

### Forum-specific (frontend has forum API client, bot has nothing)

| # | Feature | API endpoint | Priority |
|---|---------|-------------|----------|
| 34 | **Categories** | `GET /api/forum/categories` | P3 |
| 35 | **Topic list** | `GET /api/forum/topics` | P3 |
| 36 | **Topic detail** | `GET /api/forum/topics/{id}` | P3 |
| 37 | **Topic by slug** | `GET /api/forum/topics/by-slug/{slug}` | P3 |
| 38 | **Create/update/delete topic** | `POST/PATCH/DELETE /api/forum/topics*` | P3 |
| 39 | **Create/update/delete posts** | `POST/PATCH/DELETE /api/forum/posts*` | P3 |
| 40 | **Follow toggle** | `GET/POST /api/forum/topics/{id}/follow` | P3 |
| 41 | **Mark read** | `POST /api/forum/topics/{id}/read` | P3 |
| 42 | **Forum search** | `GET /api/forum/search` | P2 |
| 43 | **Moderation status/queue** | `GET /api/forum/moderation/*` | P4 |
| 44 | **Moderate post** | `POST /api/forum/posts/{id}/moderate` | P4 |
| 45 | **Post moderations history** | `GET /api/forum/posts/{id}/moderations` | P4 |
| 46 | **Post reactions** | `POST /api/forum/posts/{id}/react` | P3 |
| 47 | **Metamoderation** | `GET/POST /api/forum/metamod*` | P4 |
| 48 | **Admin: hide/lock/pin** | `/api/admin/forum/*` | P4 |
| 49 | **Admin: bans** | `/api/admin/forum/bans*` | P4 |

### Self-healing / admin

| # | Feature | API endpoint | Priority |
|---|---------|-------------|----------|
| 50 | **Health** | `GET /api/health` | P1 |
| 51 | **Self-healing diagnose** | `POST /api/admin/heal` | P4 |
| 52 | **Heal extractions** | `/api/admin/heal/extractions*` | P4 |
| 53 | **Content scan** | `/api/admin/content-scan*` | P4 |
| 54 | **Admin moderation queue** | `GET /api/admin/moderation/queue` | P4 |
| 55 | **Analytics** | `/api/analytics*` | P3 |

## Missing from bot — FicHub batch shipped 2026-08-25

Endpoints added on FicHub `main` after this doc's original audit (2026-08-19).
None wired into the bot API client yet.

| # | Feature | API endpoint | Priority |
|---|---------|-------------|----------|
| 56 | **Follow exclusions** | POST/GET/DELETE `/api/follows/{follow_id}/exclusions` | P1 |
| 57 | **Saved-search list / run / alert** | GET/POST/DELETE `/api/search/saved*`, PUT `/api/search/saved/{id}/alert`, POST `/api/search/saved/{id}/run` | P1 |
| 58 | **Saved-search Atom feed** | `/feed/saved/{user_id}/{search_id}` (strips trailing `.xml`) | P2 |
| 59 | **Collection-submission vote** | POST `/api/collections/{id}/requests/{req_id}/vote` | P2 |
| 60 | **Unified curator approvals queue** | GET `/api/curator/approvals` | P2 |
| 61 | **Search v2 fielded operators** | already via `/api/search?q=` — `@field`, `~` fuzzy, wildcards, `with:`/`not-with:`, romship/platship, ranges + `50k` | P2 |

Search v2 needs no new endpoint — the existing bot `!search` already sends
plain `q=` text that the v2 parser resolves. Only UX affordances (slash
completion for `@char`/`words:`) are missing.

## CLI tool — not yet created

The FicHub CLI (`fichub get <url> --format epub`) does not exist. P8#89 in the roadmap plans it. A thin CLI wrapping the same API client would share `fanfic-archivist`'s `FichubClient`. Minimal V1: download (epub/mobi/pdf/azw3) + metadata + search.

## Summary

- **14 features** the frontend API client doesn't support yet (Send-to-Kindle through Modlog + admin endpoints) — these are API endpoints that exist on the server but have no frontend binding
- **35+ features** the bot lacks that the frontend supports — mostly forum, social, and community features
- **6 more** from the 2026-08-25 batch (follow exclusions, saved-search run/alert/feed, collection vote, curator approvals, search v2 operators)
- **No CLI** exists yet at all
- **P1 gaps**: work detail (`/api/works/{id}`), work stats, search suggestions, similar works, random work — basic discovery features a recommendation bot should have
- **P2 gaps**: reviews, notifications, user profiles, kudos toggle (bot only shows count), reading history, search suggestions
- **P3+ gaps**: forums, quests, badges, shelves, reading lists, trending, OPDS, user data export
