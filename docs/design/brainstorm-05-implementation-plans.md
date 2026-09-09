<!-- CONSOLIDATED BRAINSTORM FILE — Generated 2026-08-13, updated 2026-08-17 from repo markdown (docs/src user docs excluded). -->

> PREVIOUS IMPLEMENTATION PLANS (2026-08-07) — the richest brainstorm material: personalized home + search chips, admin dashboard improvements, prioritized improvement plan, anti-bot hardening, and this-agent-lane notes (commits + decisions). Each is a concrete task-by-task plan written for the Hermes coding agent.


---


======================================================================
SOURCE: .hermes/plans/2026-08-07_130500-personalized-home-search-chips.md
======================================================================

# Personalized Home + Smarter Search Chips — Implementation Plan (v2, corrected)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task, **Task 0 first**.

**Goal:** Fix 3 shipped regressions (leaderboard 500, bell icon, dropdown style), then build a personalized home dashboard (trending + personal recs) and smarter search filter chips.

**Architecture:** Backend endpoints in Rust/Axum (existing recommender + search modules), frontend SvelteKit 5 (HomeDashboard component + search page chips). Personalization keys on the **auth'd `bookmarks` table** (JWT user), not the legacy anonymous `fic_bookmarks`/`user_hash` (see corrected facts below).

**Tech Stack:** Rust + Axum 0.8 + SQLx 0.9 + PostgreSQL + Redis; SvelteKit 5 + Vitest.

---

## Corrected facts (verified against the live repo/DB — differs from v1/v2)

| Assumption in v2 | Reality (verified) |
|---|---|
| Tables named `fics`, `fic_id`, `community_score` | Real: `fic_info` (id = url_id VARCHAR), `fic_tags` (url_id, tag_id, score), no `community_score` column (it's computed via subquery in queries.rs / `src/bin/compute_leaderboards.rs`) |
| `fic_bookmarks` keyed by user_hash (worker-fed) | Real: `fic_bookmarks(user_hash, url_id)` exists but is fed by the **recommender worker** (AO3 profile import). The **shipped** bookmark API uses the `bookmarks` table: `bookmarks(id, user_id INT4 FK users, url_id, notes, is_private, UNIQUE(user_id,url_id))` via `AuthUser` (JWT). **Personalization must key on `bookmarks.user_id`.** |
| `request_log` has `path` | Real: `request_log` columns: id, created, source_id, etype, query, info_request_ms, url_id, fic_info, export_ms, export_file_name, export_file_hash, url, ... No `path` — downloads identified by `url_id IS NOT NULL` + `export_file_name` or `etype`. |
| Leaderboard failure unknown | **Root cause found:** `get_weekly_leaderboard` / `get_monthly_leaderboard` in `src/db/queries.rs` decode `(i32, String, i32, i16)`, but `COALESCE(lb.rank, 0)` promotes SMALLINT+INT4 literal → **INT4**; sqlx fails decoding column 4 as i16. Live repro: `/api/leaderboard/curators/weekly` → `{"err":-1,"msg":"database error"}` while `/api/leaderboard/curators` → `{"err":0,"leaderboard":[]}`. Verified `COALESCE(NULL::smallint, 0)` = integer; `::smallint` cast fixes it. |
| i18n keys per locale | LocaleSelector lists locales from `/api/auth/locale` / `listLocales()`, stores `fichub_locale` in localStorage. **No i18n key system exists** — the app is hardcoded English. So "add i18n keys" is not a small step; see Task 5 note. |

---

## Task 0: Fix shipped regressions (pre-work — do first)

**Objective:** Fix the leaderboard 500, the unconstrained bell SVG, and the dropdown style mix.

### 0a. Leaderboard 500 — root cause fix + regression test

**Files:**
- Modify: `src/db/queries.rs` lines ~1031-1050 (`get_weekly_leaderboard`) and ~1054-1072 (`get_monthly_leaderboard`)
- Test: `tests/leaderboard_api.rs` (new, DB-gated, follows `tests/search_api.rs` conventions)

**Step 1: Fix the SQL**

Change `COALESCE(lb.rank, 0)` → `COALESCE(lb.rank, 0)::SMALLINT` in **both** queries:

```rust
// before
COALESCE(lb.rank, 0) as rank
// after
COALESCE(lb.rank, 0)::SMALLINT as rank
```

**Step 2: Verify locally**

Run: `cargo build 2>&1 | tail -3` then restart + curl:
```bash
cargo build --release && sudo systemctl restart fichub.service
curl -s 'http://localhost:8000/api/leaderboard/curators/weekly'
curl -s 'http://localhost:8000/api/leaderboard/curators/monthly'
```
Expected: both return `{"err":0,"leaderboard":[...]}` (may be empty list — fine).

**Step 3: DB-gated regression test**

Create `tests/leaderboard_api.rs` (mirror `tests/search_api.rs`: static DB_LOCK, `#[ignore]`, seed a user + a `leaderboard_weekly` row, call the router, assert 200 + `err:0` + shape). Run:
```bash
set -a; . ./.env; set +a
cargo test --test leaderboard_api -- --include-ignored --test-threads=1
```
Expected: PASS.

**Step 4: Commit**
```bash
git add src/db/queries.rs tests/leaderboard_api.rs
git commit -m "fix(api): leaderboard weekly/monthly 500 from SMALLINT COALESCE type promotion"
```

### 0b. Bell SVG sizing

**Files:** the user-menu markup in `frontend/src/routes/+layout.svelte` (or wherever the NotificationBell is embedded) + `frontend/src/lib/components/NotificationBell.svelte`

**Step 1:** Check if notifications are a shipped feature (`src/routes/notifications.rs`, `/api/notifications` route exists — it does). Since it ships, **size the SVG**, don't delete: add `width="16" height="16"` (or a `.menu-icon { width:16px; height:16px; }` class) to the bell's root `<svg>`, matching other menu icons. Also add `flex-shrink:0`.

**Step 2:** Verify: `cd frontend && npm run build` passes.

### 0c. Unify dropdown item styles

**Files:** `frontend/src/routes/+layout.svelte` (Discover/Library/user menus), `frontend/src/lib/components/NavDropdown.svelte` (already has `.dd-item` — the fix is to use it everywhere)

**Step 1:** Replace the bordered-button class on Recommendations/Suggestions items with the same `.dd-item` class used by Trending/Rankings links. One style for all menu items. If the tabs need visual distinction, add a `--tab` modifier to `.dd-item`, not a different element style.

**Step 2:** Verify: `npm test` (AuthBar/NavDropdown tests) + `npm run build`.

### 0d. Frontend error UX for leaderboard (reuse pattern later)

**Files:** `frontend/src/routes/leaderboard/+page.svelte`

**Step 1:** Replace the lone red error box with: error message + **Retry** button + a muted "You can also try the other period" hint. This is the error pattern Task 3's HomeDashboard will use.

**Step 2:** Verify: `npm run build`.

**Commit (0b-0d together):**
```bash
git add frontend/src/routes/+layout.svelte frontend/src/lib/components/NavDropdown.svelte frontend/src/lib/components/NotificationBell.svelte frontend/src/routes/leaderboard/+page.svelte
git commit -m "fix(ui): size menu bell icon, unify dropdown item styles, leaderboard retry UX"
```

---

## Task 1: Backend — `GET /api/recommendations/personal`

**Objective:** Personalized recommendations from the user's `bookmarks` (auth'd) + download activity.

**Files:**
- Modify: `src/recommender/routes.rs` (add handler + scoring)
- Modify: `src/recommender/engine.rs` (shared scoring helper)
- Modify: `src/server.rs` (route registration)
- New: `tests/recommender_personal.rs`

**Step 1: Identity helper (resolves the open question)**

Add `resolve_identity(auth: &AuthUser) -> Option<i32>` → `auth.user_id` (the `bookmarks.user_id` key). **Do NOT use `fic_bookmarks`/user_hash** — that's worker-only. Bookmarks API (the shipped feature) already uses `AuthUser.user_id`; align recs to it. Anonymous users get `None` → endpoint returns `enough_data:false` with a hint (no anonymous personalization for now — cleaner than the dual-key mess; can add client_id later).

**Step 2: Shared scoring in `src/recommender/engine.rs`**

```rust
/// Jaccard overlap of two tag-id sets, normalized to [0,1].
pub fn tag_overlap(a: &[i32], b: &[i32]) -> f64 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let sa: std::collections::HashSet<i32> = a.iter().copied().collect();
    let sb: std::collections::HashSet<i32> = b.iter().copied().collect();
    let inter = sa.intersection(&sb).count() as f64;
    inter / (sa.union(&sb).count() as f64)
}

/// personal_score = tag_overlap + 0.2 * popularity_norm
pub fn personal_score(overlap: f64, popularity: f64) -> f64 { overlap + 0.2 * popularity }
```

**Step 3: Failing unit tests first**

```rust
#[test]
fn tag_overlap_prefers_shared_tags() {
    let a = vec![1, 2, 3];
    let b = vec![1, 2, 9];
    let c = vec![9, 10, 11];
    assert!(tag_overlap(&a, &b) > tag_overlap(&a, &c));
}
```
Run: `cargo test --lib recommender` → expect FAIL (not yet defined), then PASS after Step 2.

**Step 4: Handler**

`personal_recommendations_handler(State, AuthUser) -> Json<Value>`:
1. `let user_id = auth.user_id.ok_or(401)?` (or return `enough_data:false` if you prefer 200-with-flag; **recommend 200 + `enough_data:false`** so the home page can render a hint card without an auth round-trip).
2. Fetch user's bookmarked url_ids: `SELECT url_id FROM bookmarks WHERE user_id = $1 LIMIT 100`.
3. Fetch their top tag ids: `SELECT ft.tag_id, COUNT(*) c FROM bookmarks b JOIN fic_tags ft ON ft.url_id = b.url_id WHERE b.user_id = $1 GROUP BY ft.tag_id ORDER BY c DESC LIMIT 20`.
4. **Gate:** if bookmark count + download count < `PERSONAL_RECS_MIN_SIGNAL` (default 3, config-gated via `Config`), return `{"err":0,"enough_data":false,"recs":[],"based_on":[]}`.
5. Download signal: `SELECT DISTINCT url_id FROM request_log WHERE url_id IS NOT NULL AND (export_file_name LIKE '%.epub' OR etype IN ('download','export')) AND created > now() - interval '90 days' AND url_id NOT IN (bookmarked)` — union with bookmarks for the signal count.
6. **Candidates in SQL** (not whole-table scoring):
```sql
SELECT f.url_id, f.title, f.author,
       (SELECT COUNT(*) FROM fic_tags ft WHERE ft.url_id = f.id) AS tag_count
FROM fic_info f
JOIN fic_tags ft ON ft.url_id = f.id
WHERE ft.tag_id = ANY($1)               -- user's top tags
  AND f.url_id <> ALL($2)               -- exclude bookmarked/downloaded
GROUP BY f.id
ORDER BY COUNT(DISTINCT ft.tag_id) DESC
LIMIT 200;
```
   (Use actual column names from `fic_info`; check `\d fic_info` for title/author/updated columns.)
7. Score in Rust: for each candidate, fetch its tag ids, `personal_score(tag_overlap(user_tags, fic_tags), popularity_norm)` where popularity = comment/kudos-like count (check `fic_info` columns) normalized by max.
8. Response: `{"err":0,"enough_data":true,"recs":[RecResult...],"based_on":[{title,url_id}...]}` (based_on = up to 3 bookmarked fics' titles).

**Step 5: Register route** in `src/server.rs`:
```rust
.route("/api/recommendations/personal", get(crate::recommender::routes::personal_recommendations_handler))
```

**Step 6: DB-gated integration test** `tests/recommender_personal.rs`:
- Seed user + 0 bookmarks → `enough_data:false`.
- Seed user + 3 bookmarks (with tags) + a candidate fic sharing a tag → `enough_data:true`, recs exclude bookmarked, `based_on` populated.
- Follow `tests/search_api.rs` conventions (static lock, `#[ignore]`, cleanup).

Run: `cargo test --lib recommender && cargo test --test recommender_personal -- --include-ignored --test-threads=1` → all PASS.

**Step 7: Commit**
```bash
git add src/recommender/routes.rs src/recommender/engine.rs src/server.rs tests/recommender_personal.rs
git commit -m "feat(recs): personalized recommendations endpoint from auth bookmarks + downloads"
```

---

## Task 2: Backend — `GET /api/search/suggest`

**Objective:** Popular + personalized filter suggestions for the search page chips.

**Files:**
- Modify: `src/search/routes.rs` (handler)
- Modify: `src/server.rs` (route)
- Modify: `src/search/tags.rs` (dedupe/rank helpers)
- Test: extend `tests/search_api.rs`

**Step 1: Failing unit test**

```rust
#[test]
fn suggest_ranks_popular_tags_first() { /* seed 3 tags with usage 5/3/1, assert order */ }
```
Run: `cargo test --lib search::routes` → FAIL, then PASS.

**Step 2: Handler**

`search_suggest_handler(State, Query, Option<AuthUser>) -> Json<Value>`:
- Params: `q` (prefix), `tag_type_id`, `personal` (0/1).
- Popular query (GROUP BY, no correlated subquery):
```sql
SELECT t.id, t.name, t.tag_type_id, COUNT(ft.url_id) AS usage_count
FROM tags t
LEFT JOIN fic_tags ft ON ft.tag_id = t.id
WHERE ($1::int IS NULL OR t.tag_type_id = $1)
  AND ($2::text IS NULL OR t.name ILIKE $2 || '%')
GROUP BY t.id
HAVING COUNT(ft.url_id) > 0
ORDER BY usage_count DESC, t.name
LIMIT 20;
```
- `personal=1` + authed: fetch user's top tags (same query as Task 1), re-rank by `tag_overlap`; tag each item `reason: "popular" | "for_you"`.
- **Cache:** popular (non-personal) result in `AppState` via `tokio::sync::Mutex<Option<(Instant, Vec<Suggestion>)>>`, TTL 300s (tags change rarely; endpoint hit on every keystroke).
- Response: `{"err":0,"suggestions":[{id,name,tag_type_id,usage_count,reason}]}`.

**Step 3: Register route** in `src/server.rs`.

**Step 4: Tests** — unit (order, prefix, type filter, dedupe) + extend `tests/search_api.rs` (seed tags, call endpoint, assert order + reason field).

Run: `cargo test --lib search::routes && cargo test --test search_api -- --include-ignored` → PASS.

**Step 5: Commit**
```bash
git add src/search/routes.rs src/search/tags.rs src/server.rs tests/search_api.rs
git commit -m "feat(search): popular + personalized filter suggestion endpoint with cache"
```

---

## Task 3: Frontend — Home dashboard

**Objective:** Home page shows a compact download input + personal recs (left) + trending (right), with 4-level fallback and per-section resilience.

**Files:**
- Modify: `frontend/src/routes/+layout.svelte` (add `'home'` tab, default it; logo → home)
- Create: `frontend/src/lib/components/HomeDashboard.svelte`
- Create: `frontend/src/lib/api/recommendations.ts` (fetchPersonalRecs)
- New: `frontend/src/lib/components/HomeDashboard.test.ts`

**Layout:**
```
[ Paste a fanfiction URL (AO3, FFN…) ......... ] [⬇ Download]
┌ Recommended for you ───────────┐ ┌ Trending this week ────────┐
│ card — "Because you bookmarked X" │ │ card …                        │
└────────────────────────────────┘ └─────────────────────────────┘
(hint card if !enough_data: "Bookmark a few fics to personalize")
```

**Step 1: API module** `frontend/src/lib/api/recommendations.ts`:
```ts
export interface PersonalRecsResponse {
  err: number; enough_data: boolean;
  recs: RecResult[]; based_on: { title: string; url_id: string }[];
}
export async function fetchPersonalRecs(): Promise<PersonalRecsResponse> {
  return request('/recommendations/personal');  // request() from client.ts, credentials: include
}
```

**Step 2: HomeDashboard.svelte**
- Compact download input at top: URL input + Download button → `goto('/')` with `activeTab='download'` + prefill the URL (or reuse DownloadTab's submit logic; check `DownloadTab.svelte`'s handler).
- `onMount`: `const [trendingRes, recsRes] = await Promise.allSettled([fetchTrending(7,10), fetchPersonalRecs()]);` — each section independently: loading skeleton → data → fallback → quiet empty. **Never a page-level error box.**
- Left section fallback chain: `recsRes.enough_data && recs.length` → personal; else trending; else popular (community score — check `fic_info` has a usable column or use trending); else recently added; else hint card.
- "Because you bookmarked/downloaded: {based_on titles}" label.
- Right section: trending, independent of left.

**Step 3: Wire into +layout.svelte**
- `type Tab = 'home' | 'download' | 'recs' | 'sugg'`; `activeTab = $state<Tab>('home')`.
- `<main>`: `{#if activeTab === 'home'} <HomeDashboard /> {:else if ...}` for existing tabs.
- Brand link click → `goTab('home')` (or `/`).

**Step 4: Component test** — vi.mock `fetchPersonalRecs`: `enough_data:true` → "Recommended for you" + based_on label; `false` → fallback hint; one endpoint rejects → other section still renders.

**Step 5: Verify** — `cd frontend && npm test && npm run build` → all pass.

**Step 6: Commit**
```bash
git add frontend/src/routes/+layout.svelte frontend/src/lib/components/HomeDashboard.svelte frontend/src/lib/api/recommendations.ts frontend/src/lib/components/HomeDashboard.test.ts
git commit -m "feat(ui): home dashboard with download input, personalized recs, and trending"
```

---

## Task 4: Frontend — suggested chips + clear-all

**Objective:** "Suggested filters" chip row from `/api/search/suggest` (top 8, deduped vs active, hidden on error) + "Clear all".

**Files:**
- Modify: `frontend/src/lib/api/search.ts` (fetchSearchSuggestions)
- Modify: `frontend/src/routes/search/+page.svelte`
- Test: `frontend/src/lib/api/search.test.ts` + static assertions

**Step 1: API fn**
```ts
export interface SearchSuggestion { id: number; name: string; tag_type_id: number; usage_count: number; reason: 'popular' | 'for_you'; }
export async function fetchSearchSuggestions(personal: boolean): Promise<SearchSuggestion[]> {
  return request(`/search/suggest${personal ? '?personal=1' : ''}`);
}
```

**Step 2: Suggested chips row**
- New `$state` `suggestions: SearchSuggestion[]`.
- After each successful search: fetch suggestions (personal=1 if authed, else popular); **on error: silently hide the row** (set `suggestions = []`).
- Render top 8, **excluding** any suggestion already in `filters` (dedupe by name+type).
- Chip: `<button class="chip" aria-pressed="false" onclick={applySuggestion(s)}>{s.name}<span class="badge">{s.reason === 'for_you' ? 'for you' : 'popular'}</span></button>`.
- Click → map via existing facet mapping (type 1→include_tags fandom, 2→character, 3→relationship, 4→freeform, 5→warning, 6→category) → re-search.

**Step 3: Clear-all**
- In the active-chips row, show **"Clear all"** only when `activeChips().length >= 1`; onclick → `filters = defaultFilters(); doSearch();`.

**Step 4: Tests** — search.test.ts: suggestion→filter query mapping; static assertions on +page.svelte (suggestions state, fetchSearchSuggestions import, conditional Clear all).

**Step 5: Verify** — `npm test && npm run build` → all pass.

**Step 6: Commit**
```bash
git add frontend/src/lib/api/search.ts frontend/src/routes/search/+page.svelte frontend/src/lib/api/search.test.ts
git commit -m "feat(ui): suggested search filter chips with dedupe + clear-all"
```

---

## Task 5: Docs, deploy, verify

**Step 1: Docs** — update `docs/src/searching.md` (suggested filters) + `docs/src/intro.md` (home dashboard). Rebuild: `cd docs && bash build.sh` (mdbook; copies to frontend/static/docs + epub).

**Step 2: i18n note (corrected):** the app has **no i18n key system** — strings are hardcoded English (LocaleSelector persists a locale but doesn't translate UI). So instead of adding translation keys, use consistent hardcoded English and **flag i18n as a follow-up** (out of scope unless the user wants to build the key system now). Document this in the plan's open questions.

**Step 3: Full suite:**
```bash
cd /personal/documents/code/rust/fichub
cargo test
set -a; . ./.env; set +a
cargo test --test search_api --test recommender_personal --test leaderboard_api -- --include-ignored --test-threads=1
cd frontend && npm test && npm run build
```
Expected: all green.

**Step 4: Deploy + verify (incl. Task 0 regression):**
```bash
cargo build --release && sudo systemctl restart fichub.service
curl -s 'http://localhost:8000/api/leaderboard/curators/weekly'   # 200, err:0 (was 500)
curl -s 'http://localhost:8000/api/leaderboard/curators/monthly'  # 200, err:0
curl -s 'http://localhost:8000/api/search/suggest'                # 200, suggestions
curl -s 'http://localhost:8000/api/recommendations/personal'      # 200, enough_data:false (anon)
curl -s -o /dev/null -w '%{http_code}\n' 'http://localhost:8000/' # 200, dashboard
```

**Step 5: Commit**
```bash
git add docs/ frontend/static/docs
git commit -m "docs: document home dashboard and suggested search filters"
```

---

## Open questions — resolved (this round)

| Question | Decision | Why |
|---|---|---|
| Identity key | `AuthUser.user_id` → `bookmarks` table | The shipped bookmark API uses it; `fic_bookmarks`/user_hash is worker-only legacy. Recs must equal "what the user bookmarked". |
| Anonymous personalization | None for v1 (`enough_data:false` + hint) | Cleaner than dual-key; can add `client_id`-based later |
| Home vs Download-first | Home default **with compact download input embedded** | Both: personalization visible + core action one keystroke away |
| Suggestions always vs empty-only | Always (top 8, deduped, hidden on error) | Discovery aid; empty-results handled by Home fallbacks |
| i18n | **Out of scope** (no key system exists) | Would be a new subsystem; flag as follow-up |

## Files likely to change (summary)

| File | Change |
|---|---|
| `src/db/queries.rs` | **Task 0:** `::SMALLINT` cast on rank COALESCE (weekly + monthly) |
| `tests/leaderboard_api.rs` | new DB-gated regression test |
| `frontend/src/routes/+layout.svelte` | bell sizing, dropdown unify, home tab, logo→home |
| `frontend/src/lib/components/NavDropdown.svelte` | ensure `.dd-item` everywhere |
| `frontend/src/lib/components/NotificationBell.svelte` | 16px SVG |
| `frontend/src/routes/leaderboard/+page.svelte` | retry button + stale hint |
| `src/recommender/engine.rs` | `tag_overlap`, `personal_score` |
| `src/recommender/routes.rs` | `personal_recommendations_handler` |
| `src/search/routes.rs` | `search_suggest_handler` |
| `src/search/tags.rs` | dedupe/rank helpers |
| `src/server.rs` | + 2 routes |
| `tests/recommender_personal.rs` | new DB-gated integration test |
| `tests/search_api.rs` | + suggest endpoint tests |
| `frontend/src/lib/api/recommendations.ts` | new: fetchPersonalRecs |
| `frontend/src/lib/api/search.ts` | + fetchSearchSuggestions |
| `frontend/src/lib/components/HomeDashboard.svelte` | new: home dashboard |
| `frontend/src/lib/components/HomeDashboard.test.ts` | new component test |
| `frontend/src/routes/search/+page.svelte` | suggested chips + clear-all |
| `docs/src/searching.md`, `docs/src/intro.md` | document new features |

## Risks / tradeoffs

- **Task 0 leaderboard fix is the priority** — it's the only currently-500 endpoint and the user explicitly called it out. The `::SMALLINT` cast is minimal and proven (pg_typeof test confirms).
- **Personal recs need data** — with ~9 fics and few real users/bookmarks, `enough_data` will usually be false; the Home hint card + fallback chain is what makes the feature not look broken.
- **Popularity metric**: no `community_score` column — use whatever `fic_info` has (kudos/comment counts or trending-based popularity); verify columns before writing SQL (Step 1 of each backend task should `\d fic_info`).
- **Cache on suggest** prevents per-keystroke DB hammering; invalidate on tag writes (or 5-min TTL is fine).


======================================================================
SOURCE: .hermes/plans/2026-08-07_135000-whats-next-admin-dashboard.md
======================================================================

# What's Next: Admin Dashboard Fix + FicHub Improvement Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task, Task 1 first.

**Goal:** (a) Fix the admin dashboard discoverability/accessibility issue (it's implemented but unreachable), (b) continue the momentum from the personalization work with a prioritized backlog.

**Architecture:** Small, surgical fixes + a prioritized feature backlog. Task 1-3 are correctness/discoverability (cheap, do first); Tasks 4+ are new features.

**Tech Stack:** Rust/Axum + SvelteKit 5 + Postgres (unchanged).

---

## Current State (verified 2026-08-07)

- **Admin dashboard IS implemented**: frontend `/admin` (+page dashboard w/ stats, `/admin/moderation`, `/admin/scrapers`, `/admin/users`) guarded by `admin/+layout.svelte` (`role >= 10`); backend `/api/admin/*` (stats, moderation queue, scraper-health, users CRUD) all guarded `role < 10 → Forbidden`.
- **BUT it's unreachable**:
  1. **No navbar link.** The user menu only shows `/curator/authors` for `isCurator` (`role >= 10`). Nothing links `/admin`.
  2. **The `admin` user has role 0.** `SELECT id, username, role FROM users` → `admin | 0`. So even manually typing /admin redirects home (frontend guard) and /api/admin/* returns 403.
  3. **Chicken-and-egg**: you can't promote the admin via the admin/users page because you can't reach it at role 0. Must be done via SQL.
- **Role scale (verified consistent in code):** 0=Regular, 1=Trusted, 5=Curator, 10=Admin. The `auth.rs:14` comment (`0=reader, 1=curator, 2=senior, 3=admin`) is **stale** — the code uses 5/10 thresholds everywhere (authors.rs `>= 5`, admin.rs `>= 10`, admin/users UI options 0/1/5/10). Fix the comment, not the code.
- **Personalization shipped** (this session): home dashboard (personal recs + trending + compact download), /api/recommendations/personal, /api/search/suggest + suggested chips, leaderboard 500 fix. All tested (399 lib / 18 DB-gated / 121 frontend) and deployed.

---

## Task 1: Promote the admin user to role 10 (unblock admin access)

**Objective:** Make the existing admin dashboard reachable for the actual admin account.

**Step 1: SQL promotion (one-time, not code)**

```sql
UPDATE users SET role = 10 WHERE username = 'admin';
```

Run: `sudo -u postgres psql -d fichub -c "UPDATE users SET role = 10 WHERE username = 'admin';"`
Verify: `sudo -u postgres psql -d fichub -c "SELECT id, username, role FROM users WHERE username='admin';"` → role=10.

**Step 2: Add a DB-gated regression test** that the admin guard accepts role 10 and rejects role 0 (so this can't silently regress):

- File: `tests/admin_api.rs` (new, follows `tests/search_api.rs` conventions: static Mutex, #[ignore], seed user with role 10, call `/api/admin/stats` via a minimal router, assert 200; seed role 0, assert 403).
- Note: `src/routes/admin.rs` handlers use `AuthUser` extractor — the test needs a valid JWT (there's `fichub::routes::auth::create_token` used in recommender_personal.rs tests — reuse that pattern).

**Step 3: Commit**
```bash
git add tests/admin_api.rs
git commit -m "test(admin): role-gated access to admin endpoints (10 ok, 0 forbidden)"
```

**Step 4 (optional, defer if you prefer manual):** a tiny `scripts/promote_admin.sql` documented in README so a fresh deploy can bootstrap the admin role.

---

## Task 2: Link the admin dashboard in the navbar

**Objective:** A role >= 10 user sees an Admin entry point (currently only /curator/authors exists).

**Files:**
- Modify: `frontend/src/routes/+layout.svelte` (the `isCurator` block in the user menu + mobile menu)

**Step 1: Widen the privileged menu**

In the desktop user menu (around line 123) and mobile menu (line 168), replace the single Curator link with a small group:

```svelte
{#if isCurator}
  <a class="dd-item" href="/admin">🛠️ Admin Dashboard</a>
  <a class="dd-item" href="/curator/authors">🛡️ Curator</a>
{/if}
```

**Step 2: Verify** — `cd frontend && npm run build && npm test` (existing 121 tests stay green; add a static assertion if there's a layout test, else rely on build).

**Step 3: Commit**
```bash
git add frontend/src/routes/+layout.svelte
git commit -m "feat(ui): link Admin Dashboard in the user menu for role >= 10"
```

---

## Task 3: Fix the stale role comment in auth.rs

**Objective:** Docs match reality (0/1/5/10 scale), so future work doesn't reintroduce the confusion.

**Files:**
- Modify: `src/routes/auth.rs:14` (`pub role: i16, // 0=reader, 1=curator, 2=senior, 3=admin`)

**Step 1:** Change the comment to the real scale:
```rust
pub role: i16, // 0=regular, 1=trusted, 5=curator, 10=admin (see admin/users UI)
```

**Step 2:** Verify no code depends on the 0-3 scale: `grep -rn 'role.*[0-9]' src/ | grep -v test` — confirm only 5/10 thresholds remain.

**Step 3: Commit**
```bash
git add src/routes/auth.rs
git commit -m "docs(auth): correct role scale comment (0/1/5/10)"
```

---

## Task 4: Personalization polish (build on this session's recs)

**Objective:** Make the shipped personalization actually surface for real users.

**Files:**
- Modify: `src/recommender/routes.rs` (personal_recommendations_handler)
- Modify: `frontend/src/lib/components/HomeDashboard.svelte`

**Step 1: Raise the personalization signal quality**
- Currently: bookmark count + download count >= 3. With the archive at ~9 fics, this rarely fires.
- Improvement: also count **reading activity** — the `reading` table (check schema: `read/[urlId]` route implies a reading_history) and `request_log` url hits. A user who reads 3+ fics (even without bookmarking) gets recs.

**Step 2: Add an explainer chip on the home dashboard** when `enough_data:false`: "Bookmark 3+ fics or read a few to unlock personalized recommendations" (already partially there; make it actionable with a link to /search).

**Step 3: Tests** — extend `tests/recommender_personal.rs` with a reading-signal case.

**Step 4: Commit** — `feat(recs): include reading history in personalization signal`

---

## Task 5: Content growth (the archive is the product)

**Objective:** More fics → better trending, recs, search, and suggest chips.

**Files:**
- Modify: `scripts/populate_royalroad.py` (already exists)
- Create: `scripts/populate_quotev.py`, `scripts/populate_wattpad.py` (both sites reachable per earlier probe: royalroad 200, quotev 200, wattpad 200)

**Step 1: Extend the populate script family** to the reachable sites (Quotev, Wattpad) with polite delays + the same POST /api/epub?q= pattern. Add `--limit` and `--site` flags.

**Step 2: Run a second populate batch** (e.g. 20 more fics across royalroad/quotev/wattpad) → the archive grows to ~30 fics, making trending/personal/suggest meaningful.

**Step 3: Verify** — curl `/api/search/suggest` shows richer suggestions; `/api/trending` has entries; home dashboard personal section fires for a test user with 3 bookmarks.

**Step 4: Commit** — `feat(scripts): populate archive from quotev + wattpad`

---

## Task 6: Notifications UX (the bell is shipped but basic)

**Objective:** The bell shows a count; make the notifications page useful.

**Files:**
- Modify: `frontend/src/routes/notifications/+page.svelte` (check current state)
- Backend: `/api/notifications/*` (list, mark-read) already exist

**Step 1:** Review the notifications page — if it's a bare list, add: unread/read filtering, "mark all read", relative timestamps, and notification-type icons (reply, follow, badge, upload).

**Step 2: Tests** — component test for the filter/mark-all; DB-gated test for the list endpoint shape if missing.

**Step 3: Commit** — `feat(ui): notifications filtering + mark-all-read`

---

## Task 7: Frontend e2e for the new search chips + home dashboard (P2 #8 from earlier)

**Objective:** The frontend tests are component-level; add an e2e-ish flow test.

**Files:**
- Create: `frontend/src/routes/search/+page.test.ts` (if not present) or extend existing

**Step 1:** Test the full flow: type a query → results render → suggested chips appear (mocked fetch) → click a chip → filters update + re-search → Clear all resets.

**Step 2: Commit** — `test(ui): search page flow with suggested chips`

---

## Task 8: Docs refresh (final polish)

**Files:**
- Modify: `docs/src/intro.md` (nav: Admin Dashboard for admins), `docs/src/searching.md` (already updated this session)

**Step 1:** Document the admin dashboard + role scale (0/1/5/10) in a new `docs/src/admin.md` (or the README), rebuild mdbook.

**Step 2: Commit** — `docs: document admin dashboard and role scale`

---

## Suggested prioritization

| Priority | Task | Why |
|---|---|---|
| P0 | Task 1 + 2 | Admin can actually use the shipped dashboard (they asked "was it implemented?") |
| P0 | Task 3 | Kills the stale comment that caused the confusion |
| P1 | Task 4 | Personalization fires for real users |
| P1 | Task 5 | More content → everything else gets better |
| P2 | Task 6 | Notifications are shipped but thin |
| P2 | Task 7, 8 | Coverage + docs polish |

## Open questions

1. **Admin role now**: should the `admin` user be role 10 (Admin) or role 5 (Curator)? Recommend 10 (it's the named admin account). Confirm before Task 1 runs.
2. **Reading history schema**: Task 4 depends on the `reading` table shape — verify columns before writing the signal query (`\d reading`).
3. **Populate scope**: how many fics per site for Task 5? Recommend 10-15/site with polite delays (the earlier 18-fic run took ~14 min).

## Files likely to change (summary)

| File | Task |
|---|---|
| `tests/admin_api.rs` (new) | 1 |
| `frontend/src/routes/+layout.svelte` | 2 |
| `src/routes/auth.rs` | 3 |
| `src/recommender/routes.rs` + `tests/recommender_personal.rs` | 4 |
| `frontend/src/lib/components/HomeDashboard.svelte` | 4 |
| `scripts/populate_*.py` (new quotev/wattpad) | 5 |
| `frontend/src/routes/notifications/+page.svelte` + tests | 6 |
| `frontend/src/routes/search/+page.test.ts` (new) | 7 |
| `docs/src/admin.md` (new), `docs/src/intro.md` | 8 |


======================================================================
SOURCE: .hermes/plans/2026-08-07_160500-improvements-priority.md
======================================================================

# FicHub Improvement Plan — Prioritized (from chatbot suggestions)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Execute the high-value suggestions from the spec review, in priority order, dropping what's not worth it.

**Architecture:** Current system = Rust/Axum `fichub` on :8000 (API + SvelteKit static frontend), Postgres 16 `fichub`, Redis (rate limit + suggest cache), FanFicFare via pipx. All changes are additive; no frontend framework changes.

**Tech Stack:** Rust/Axum, sqlx, Postgres (pg_trgm), SvelteKit 5, systemd, Redis.

---

## Priority order (verified against live state)

### P0 — Fix `main_char_attr` on existing content (BACKFILL)
**Why:** ALL 15 existing `fic_tags` rows have `score=0` (verified). The flagship `main_char_attr` search returns NOTHING for the existing RoyalRoad archive until scores are set. This is the single highest-value fix.

**Task 1: Backfill script** (`scripts/backfill_tag_scores.rs` or a `cargo run --bin backfill_scores`)
- Files: create `src/bin/backfill_scores.rs` (or `scripts/`), modify none
- Approach: query distinct `fic_info` ids → for each, call `extract_tags` (or reuse the existing tag data) → set the first character tag's `fic_tags.score` to 10, others 1; first ship 5, others 1.
- **Simpler alternative (recommended):** Since tags are already in the DB with known types, write a SQL-only backfill: for each fic, find its character tags (type 2) and set the lexicographically/chronologically-first to 10, rest 1; first ship (type 3) to 5, rest 1. This matches the scrape-time rule (first-listed = main) IF insertion order reflects metadata order (it does — `fic_tags` has no explicit order column, but `created_at` + insertion order is a good proxy; verify with a sample).
- Test: after backfill, `SELECT url_id, COUNT(*) FILTER (WHERE score=10) FROM fic_tags WHERE tag_type_id=2 GROUP BY url_id` shows exactly 1 main char per fic; run the `main_char_attr` live check against a real fic.
- Commit: `feat(db): backfill main-character tag scores for existing archive`

### P1 — ServeDir cache headers (10-line win, big bandwidth saving)
**Why:** No `Cache-Control` headers on the SPA (verified). Returning users re-download the whole bundle every visit.

**Task 2: Cache-header middleware** in `src/server.rs`
- Files: `src/server.rs` (wrap `ServeDir` with a middleware)
- `/frontend/build/_app/immutable/*` → `Cache-Control: public, max-age=31536000, immutable`
- `index.html` + `/docs/*` → `Cache-Control: no-cache, no-store, must-revalidate`
- Everything else → `public, max-age=3600`
- Verify: `curl -I localhost:8000/` shows no-cache; `curl -I localhost:8000/_app/immutable/<file>` shows immutable; `curl -I localhost:8000/docs/searching.html` shows no-cache.
- Commit: `perf(server): immutable cache headers for hashed assets, no-cache for html`

### P2 — Trigram fuzzy search (typo tolerance)
**Why:** `pg_trgm` NOT enabled (verified). Misspelled queries ("Hary Pottr") return nothing. AO3 users expect forgiving search.

**Task 3: Enable pg_trgm + fallback ILIKE matching**
- Files: new migration `migrations/009_pg_trgm.sql` (CREATE EXTENSION IF NOT EXISTS pg_trgm + GIN indexes on `fic_info.title`/`author` with `gin_trgm_ops`), `src/search/builder.rs` (when a `q` term has no exact matches, add a `title ILIKE '%term%'` OR `author ILIKE '%term%'` fallback with lower relevance)
- Note: GIN index on title/author of a small archive is cheap; pg_trgm ILIKE fallback gives typo tolerance without a heavy engine.
- Test: `cargo test --lib search::` + a DB-gated test seeding "Harry Potter" and searching `q=Hary Pottr` → matches.
- Commit: `feat(search): pg_trgm trigram indexes + ILIKE fallback for typos`

### P3 — Trope/Attribute browser (`/tropes` route)
**Why:** `main_char_attr` is the flagship feature but has zero discoverability — users must know the `Character|Attribute` syntax and type it manually.

**Task 4: `/tropes` page** (frontend + backend)
- Backend: `GET /api/tropes` — materialized view / query of highest-scoring (character × freeform-attribute) intersections from `fic_tags` (e.g. "Dark Harry Potter", "Competent Naruto"), top N by count. Endpoint in `src/routes/` + registered in server.rs.
- Frontend: `frontend/src/routes/tropes/+page.svelte` — grid/list of chips; clicking a trope navigates to `/search?main_char_attr=Char|Attr`.
- Test: backend unit + frontend test (mock fetch).
- Commit: `feat(tropes): browse main-character attribute combos`

### P4 — Send-to-Kindle / e-reader integration
**Why:** OPDS already exists; e-reader delivery is the natural next step for a download service.

**Task 5: `/api/send-to-kindle` endpoint**
- Files: new `src/routes/send.rs`, config for SMTP relay
- POST url + kindle email → generate EPUB → email via SMTP (background job). Auth required (bookmarks user).
- Test: unit test for the email-building + a mocked SMTP.
- Commit: `feat(send): email EPUBs to e-reader addresses`

---

## Dropped (not worth it — verified)

| Suggestion | Why dropped |
|---|---|
| Archive Go backend dirs | ~/code/go/* are OUTSIDE the repo (not committed); low confusion risk. Optional cleanup, not code. |
| Rename photon-lemmy / mono-svelte | Already done: package.json is `fichub-frontend`, no mono-svelte alias, no base.ts/client.svelte.ts remnants (verified). |
| Fix state_referenced_locally warnings | Only 1 occurrence (`let query = $state('')` in authors page — not a loader pattern); cosmetic, not a bug. |
| Dedicated reader mode | `/read/[urlId]` route ALREADY EXISTS (verified). |
| Virtual scrolling | Nice-to-have; archive is ~20 fics — premature optimization. |
| Service worker offline caching | adater-static; the EPUB files are served from /public/literature, not in the SPA bundle; complex for marginal benefit now. |
| FlareSolverr for AO3/FFN | NO DOCKER on this host (verified). Would need manual install + personal.ini plumbing; high effort. Defer until Docker exists or Cloudflare blocks more sites. |
| Prometheus/Grafana dashboards | axum-prometheus is NOT in the stack (verified — no /metrics endpoint). Full monitoring infra is overkill for a single-host service; defer. |
| Redis token-bucket "optimization" + per-user rec caching | The suggest cache already exists; rec caching per user is a 15-min TTL addition — small but low-priority; fold into P4 if time. |
| OPDS acquisition links | Worth verifying (quick): check `/opds` feeds already emit acquisition links — likely done. Defer if present. |
| CI/CD pipeline | .forgejo/workflows/qa.yml already added this session (doctor gate). Deploy automation is a future task. |

---

## Verification (each task)
- Backfill: psql check (1 main char per fic) + live `main_char_attr` against a real fic
- Cache headers: curl -I checks
- pg_trgm: cargo test + DB-gated typo test
- Tropes: cargo test + frontend test + live /api/tropes
- Send-to-kindle: cargo test + live 401-unauthed check

## Risks / open questions
- **Backfill order proxy**: `fic_tags.created_at` reflects insertion order but is not guaranteed to match metadata order if tags were inserted in a different sequence. Mitigation: sample 2-3 fics and eyeball; if wrong, fall back to re-scraping via extract_tags (slower but exact).
- **pg_trgm index size**: trivial for ~20 fics; scales fine to 100k.
- **Send-to-kindle SMTP**: needs credentials — ask user before wiring a real relay; ship the endpoint with a mock/stub SMTP in tests.
- **Tropes materialized view**: keep it a plain query first (small data); materialize later if slow.


======================================================================
SOURCE: .hermes/plans/2026-08-07_170000-anti-bot-hardening.md
======================================================================

# Anti-Bot Hardening Plan — Human-Impersonation & Bad-Faith Bots, Zero Friction

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Deter human-impersonating and bad-faith bots (mirror bots, credential stuffers, spam accounts, tag/comment spammers) with layered, behavior-based controls that never challenge a normal reader.

**Architecture:** Extend the existing Redis token-bucket limiter + request_log into a behavioral detection pipeline. Everything keys on behavior, not identity; friction only appears after a bot-like pattern is observed. Existing building blocks: `src/limiter/redis_bucket.rs` (per-IP + global buckets on export), `request_log` (per-request client_id + etype + timing), admin ban endpoints.

**Tech Stack:** Rust/Axum, Redis (buckets + aggregates), Postgres (request_log, new bot_scores table), SvelteKit (admin bot dashboard).

---

## Task 1: Real client IPs (X-Forwarded-For)

**Objective:** Rate limits and bot detection must see the real client IP, not nginx's 127.0.0.1.

**Files:**
- Modify: `src/server.rs` (ConnectInfo usage), `src/limiter/redis_bucket.rs` (IP key source)
- Test: unit test in `redis_bucket.rs` (IP extraction from XFF string)

**Step 1: Failing test** — a helper `extract_client_ip(xff: Option<&str>, remote: SocketAddr) -> IpAddr` that returns the FIRST untrusted XFF hop when present, else remote_addr. Test: XFF "203.0.113.7, 10.0.0.1" → 203.0.113.7; no XFF → remote_addr.

**Step 2:** Implement helper (trust XFF only for configured proxy subnets — nginx on 127.0.0.1; parse leftmost untrusted hop).

**Step 3:** Wire into limiter's IP key + request_log logging (add `ip` column to request_log? See Task 4 — the log currently has no ip column; add it in the same migration).

**Step 4:** `cargo test --lib limiter::` green.

**Step 5:** Commit `feat(anti-bot): honor X-Forwarded-For for real client IPs`

## Task 2: Tiered rate limits per endpoint class

**Objective:** Downloads strict, auth strict, search/docs loose — so bots hitting the expensive/abusive endpoints are throttled while normal reading is untouched.

**Files:**
- Modify: `src/limiter/redis_bucket.rs` (per-endpoint buckets), `src/server.rs` (middleware keyed by route path), `src/routes/export.rs` + `src/routes/auth.rs` (apply)
- Test: unit tests for bucket math per tier

**Step 1:** Define tiers (config-driven):
- `download`: 60/hour, 300/day, burst 10 (per IP)
- `auth`: 10/min per IP (login/register)
- `search/docs/static`: very high (1000/min) — bots don't hurt here
- default: moderate

**Step 2:** Middleware that picks tier by path prefix and calls `check_bucket` with that tier's params (per-IP AND per `(ip, client_id)` keys to avoid punishing shared NAT).

**Step 3:** Keep existing global bucket as the top-level circuit breaker.

**Step 4:** Tests: bucket refill math for each tier; `(ip, client_id)` key distinctness.

**Step 5:** Commit `feat(anti-bot): tiered per-endpoint rate limits keyed by ip+client_id`

## Task 3: Behavioral aggregates job (bot_scores)

**Objective:** Compute per-IP/per-client behavioral signals on a schedule so detection is a DB query, not live scanning.

**Files:**
- Create: `migrations/010_bot_scores.sql` (`bot_scores` table: ip, client_id, window_start, requests, downloads, failed_auths, export_ratio, primary key (ip, client_id, window_start))
- Create: `src/bin/bot_scorer.rs` (or a cron task) — aggregates request_log hourly: requests/hour, downloads/hour, failed-auth count, export:request ratio
- Modify: `src/db/queries.rs` (aggregate queries)
- Test: seed request_log rows → run scorer → assert bot_scores rows

**Step 1:** Migration: `bot_scores` table + index on (ip, window_start).

**Step 2:** Scorer binary: group request_log by (ip, client_id, hour) → upsert aggregates.

**Step 3:** `cargo test` for the aggregate query (seed 100 requests, 99 downloads → export_ratio 0.99).

**Step 4:** Commit `feat(anti-bot): hourly behavioral aggregate scorer`

## Task 4: Bot detection signals + admin dashboard

**Objective:** Turn bot_scores into flags an admin can SEE and act on; export:request ratio + failed-auth bursts trigger shadow-flags (never blocks).

**Files:**
- Modify: `src/routes/admin.rs` (new endpoints: top IPs by requests, by download ratio, by failed-auths), `src/db/queries.rs`
- Create: `frontend/src/routes/admin/bots/+page.svelte` (table: IP, requests/hr, downloads, ratio, failed-auths, shadow-flag badge)
- Test: admin API test (DB-gated) + frontend test

**Step 1:** Backend: `GET /api/admin/bots?window=1h` — top 100 by requests + ratio + failed-auths, with a `flagged` boolean when ratio > 0.9 or failed-auths > 5.

**Step 2:** Frontend `/admin/bots` page — sortable table, flag badges, link to ban.

**Step 3:** Tests (admin API DB-gated + vitest component).

**Step 4:** Commit `feat(anti-bot): admin bot dashboard with behavior flags`

## Task 5: Honeypots + form timing traps (zero friction)

**Objective:** Catch human-impersonation bots on forms (register, comment) without challenging anyone.

**Files:**
- Modify: `frontend/src/routes/register/+page.svelte` (if exists) + `src/routes/auth.rs`, `src/routes/social.rs` (comment)
- Test: unit test for the honeypot rejection logic

**Step 1:** Hidden honeypot field in registration + comment forms (CSS-hidden input `website`). Backend rejects submissions where it's non-empty — silently (200 OK, no account created) to avoid teaching the bot.

**Step 2:** Form timing trap: store `form_opened_at` (client JS sets it); backend flags submissions < 2s after page load as low-trust (shadow-flag the client_id, don't block).

**Step 3:** Backend: honeypot + timing checks in auth/social handlers; shadow-flag via request_log.

**Step 4:** Tests: honeypot-filled form → no account, no error; fast submit → flagged.

**Step 5:** Commit `feat(anti-bot): honeypot fields + form-timing traps for impersonation bots`

## Task 6: PoW challenge for flagged clients (optional, deferred)

**Objective:** A tiny hashcash challenge ONLY for clients that already triggered shadow-flags — humans never notice, bulk bots pay heavily.

**Files:**
- Modify: `src/server.rs` (middleware), `Cargo.toml` (sha2 already in tree?)
- Test: unit test for challenge verify

**Step 1:** Generate challenge (random nonce + required leading-zero bits). Flagged clients get `X-PoW-Challenge` header; valid `X-PoW-Solution` (sha256(nonce+client_id+sol) with N zero bits) grants a 10-min `X-PoW-Token`.

**Step 2:** Middleware: only challenge when client is flagged in bot_scores; verify token before download endpoints.

**Step 3:** Tests + commit `feat(anti-bot): proof-of-work challenge for flagged clients only`

## Task 7: Headless-browser flagging (not blocking)

**Objective:** Detect automation WITHOUT blocking the power-user audience (OPDS readers, CLI tools are legitimate).

**Files:**
- Modify: `frontend/src/routes/+layout.svelte` or a small client script, `src/routes/admin.rs` (flag ingestion)
- Test: unit test

**Step 1:** Client-side: send `navigator.webdriver`, missing `navigator.languages`/plugin array as a `X-Client-Capabilities` header.

**Step 2:** Backend: log it in request_log as a flag column (`is_headless`); show in bot dashboard.

**Step 3:** NEVER block on it — just flag (the doc explicitly warns against blocking automation).

**Step 4:** Commit `feat(anti-bot): headless-browser capability flag (log-only)`

---

## Verification (per task + final)

- After Task 1: `curl -H 'X-Forwarded-For: 203.0.113.7'` → request_log shows the real IP (check via admin/log query)
- After Task 2: loop 200 downloads from one IP → blocked at ~60/hr; search 500× → not blocked
- After Task 3: seeded request_log → bot_scores rows with export_ratio 0.99
- After Task 4: `/api/admin/bots` shows the flagged IPs; UI renders badges
- After Task 5: honeypot-filled registration → 200 + no user row; fast comment → shadow-flagged
- After Task 6: flagged client gets X-PoW-Challenge; valid solution → token; normal client → no challenge
- After Task 7: Playwright/headless session → `is_headless` flag in dashboard; real browser → absent
- **Always**: `node qa/run.js` green; no new QA findings; a normal journey (search → open → download 1 EPUB) NEVER sees a challenge

## Risks / trade-offs / open questions

- **X-Forwarded-For trust**: trusting XFF lets a client spoof it if nginx doesn't overwrite. Mitigation: only trust XFF from the nginx proxy subnet (127.0.0.1 / docker network). Confirm nginx sets `proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for` (it does by default in this deployment).
- **request_log has NO ip column today** (verified) — Task 1/3 need a migration to add `ip inet` + backfill from XFF if available (it isn't logged historically → only new rows get IPs; acceptable).
- **PoW is the only "friction" control** — keep it OFF by default (config flag) until abuse actually appears; the dashboard data should drive when to enable.
- **Honeypot UX**: hidden inputs must be `display:none` AND `tabindex=-1` AND `autocomplete=off` so real users (and screen readers) never interact.
- **False positives on ratio**: a legit power user mirroring their own bookmarks hits ratio > 0.9. Mitigation: flag, never block, at Layer 2; admin reviews the dashboard.

## Files likely to change (summary)
- `src/server.rs`, `src/limiter/redis_bucket.rs`, `src/routes/export.rs`, `src/routes/auth.rs`, `src/routes/social.rs`, `src/routes/admin.rs`, `src/db/queries.rs`
- `migrations/010_bot_scores.sql` (+ ip column migration)
- `src/bin/bot_scorer.rs` (new)
- `frontend/src/routes/admin/bots/+page.svelte` (new), register/comment forms
- `Cargo.toml` (sha2 if PoW added — check existing deps first)


======================================================================
SOURCE: .hermes/plans/2026-08-07_220000-this-agent-lane.md
======================================================================

# Notes — 2026-08-07/08 (this agent's lane) — FINAL

## Committed on agent/fichub-lane (4 commits ahead of main, main is ancestor → ff possible)
1. `8064d57` feat(static): ServeDir cache headers (other agent's commit, on my branch, not on main)
2. `f7b1ef1` test(works): extract pure auto-merge decision functions + 17 unit tests
3. `e746439` feat(search): pg_trgm typo-tolerant fuzzy fallback (initial ILIKE version)
4. `a319022` fix(search): pg_trgm word_similarity fuzzy fallback + boolean-op guard

## Fuzzy search — final resolution (subagent deleg_a4be7df8 + my fixes)
The subagent proved pure ILIKE cannot fix typos ('hary' ⊄ 'harry'). Final design:
- builder.rs: `word_similarity(term, fi.title/author) > 0.35` ORs, emitted in BOTH
  tsquery branches (parsed_tsquery is set for ALL bare text queries, so the
  plainto-only fallback was unreachable → THE bug).
- routes.rs: zero-result retry gated on explicit boolean ops (AND/OR/NOT in raw q)
  + advanced filters, NOT parsed_tsquery.is_some() (always true → would block all).
- migration 012: pg_trgm + GIN gin_trgm_ops indexes on fic_info(title, author).
  Applied to DB manually; checksum recorded as SHA-384 (sqlx uses sha384 of file).
- DB-gated test: 'Hary Pottr' → 'Harry Potter' ✓ (34/34 search_api, 452 lib).

## Seed/backfill
- 61 feature clusters seeded (ids 35-96), embeddings via nomic-embed-text, committed 758fbbc.
- P0 tag backfill: NOT APPLICABLE (RoyalRoad fics have no char/relationship tags).

## Blocked
- Merge to main: blocked by other agent's dirty src/server.rs (differs between
  branches; checkout would overwrite their WIP). Wait for them to commit, then
  `git checkout main && git merge agent/fichub-lane --ff-only`.
- Do NOT apply migration 011 (other agent's consensus engine lane).
