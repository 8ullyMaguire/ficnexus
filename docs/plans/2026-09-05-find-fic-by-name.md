# Plan: "Find fic by title + author" (no URL required)

**Audience:** junior dev, one commit per task, each task independently shippable. Written against main @ `5e18ecd` — re-grep before editing.

**Goal:** let a user type `Governor's Gambit by Freefaller on SpaceBattles` anywhere a URL is expected today and land on the right fic — or, when it's not in the archive, funnel into Fic Requests instead of dead-ending.

## What already exists (verified — do NOT rebuild)

| Capability | Where | Note |
|---|---|---|
| Fielded search | `src/search/parser.rs` — `title:harry`, `author:jk`, `fandom:`, `character:`, `relationship:`, **`source:`** operators; `title~x` fuzzy | v2 syntax via `GET /api/search?q=...` |
| Typo tolerance | pg_trgm (`001_initial.sql:15`); used at `src/search/builder.rs:454-480` with `word_similarity(...) > 0.35` | "Govener's Gambit" still matches |
| Full-text body search | `src/search/body.rs` (tsvector) | not needed for find-by-name |
| NL → query | `POST /api/search/ask` (`src/search/ask.rs`) — Ollama, Redis-cached, keyword fallback | use as *fallback only*; regex covers the common case deterministically |
| Fic Requests + candidates | `src/routes/requests.rs` (`CreateRequestBody {title, body, seed_work_id}`) | funnel target |
| Scrape-on-demand by URL | `src/fic_suggestions.rs` POST accepts raw `url`; `src/scrape/registry.rs::lookup` | requires a URL — see Deferred |
| Fuzzy-suggestion pattern | `src/routes/authors_fuzzy.rs` — pure scoring fns + `suggestions` field on 404 | copy this pattern |
| Download input + handoff | `ArchiveHome.svelte` (`sessionStorage 'fichub_dl_url'` → `goto('/download')`) → `frontend/src/lib/components/DownloadTab.svelte` reads it and auto-exports. `detectSite()` exists in `$lib/util` | Task 3 extends this |
| Command palette | `frontend/src/lib/components/CommandPalette.svelte` (Ctrl+K, page actions + doc search) | Task 6 adds find-fic action |
| Canonical works model | `works` table (`001_initial.sql:3783`) resolves multi-source fics | find endpoint dedupes by `work_id` |

**Corrections to the brainstorm doc** (verified against code): fielded search is *not* `?title=&author=` GET params — it's the v2 `q=` syntax; the download input lives in `DownloadTab.svelte` + `ArchiveHome.svelte`, and there is **no** Discord bot and **no** `fichub` CLI in this repo (only backfill/worker binaries in `src/bin/`).

## Tasks

## Task 1: `POST /api/find-fic` — parse, search, disambiguate

**Files:** new `src/routes/find_fic.rs`; register in `src/routes/mod.rs` + `src/server.rs`.

**Behavior:** accepts free text, returns archive matches (canonical works only) plus a normalized interpretation the UI can show ("Interpreted as…", same idea as `/ask`).

**Steps:**

1. **Pure parser** `parse_find_query(input: &str) -> ParsedFindQuery` in `find_fic.rs` (no DB, fully unit-testable). Output struct:
   ```rust
   pub struct ParsedFindQuery {
       pub title: Option<String>,
       pub author: Option<String>,
       pub site: Option<String>,   // canonical site key, e.g. "spacebattles"
       pub raw: String,            // original input for the /ask fallback
   }
   ```
   Rules:
   - If input contains `://` → return `ParsedFindQuery { raw }` with all `None` — the URL path already exists, caller falls through to existing handling.
   - Site aliases first: strip a trailing site mention from the end (` on <site>`, ` from <site>`, ` (<site>)`, ` - <site>`), case-insensitive, via a small `SITE_ALIASES` table: `sb→spacebattles`, `sv→sufficientvelocity`, `qq→questionablequesting`, `ao3→archiveofourown`, `ffn→fanfiction.net`, `rr→royalroad`, `sh→scribblehub`, plus full names. Check `src/scrape/registry.rs` for the canonical site keys and reuse them.
   - **Split on the LAST ` by ` only** (`rsplit_once(" by ")`) — splitting on the first would mis-parse `War by Other Means by Author`. Title = left part; author = right part. If no ` by `, everything is the title.
   - Strip surrounding quotes/asterisks (markdown italics) from both parts; trim.
2. **Handler flow** (`POST /api/find-fic`, body `{ "query": "...", "limit": 5 }`):
   - `limit` default 5, clamp 1–10.
   - Build v2 query string: `title:"{title}" author:"{author}"` (+ ` source:{site}` when present — verify exact operator spelling in `src/search/parser.rs:548` region first). Empty title or author parts are omitted.
   - Call the **same internal path** `src/search/routes.rs` uses for `GET /api/search?q=...` (do not duplicate the SQL pipeline; factor a helper if needed). Map results to canonical works: dedupe on `work_id` (multiple source `fic_info` rows collapse), keep best-ranked row per work.
   - **Never call Ollama here.** If the regex yields no title/author at all, respond with `"parsed": null` + `"fallback": "ask"` and let the client decide to hit `/api/search/ask` (Task 5). Keeps this endpoint fast and deterministic; the `/ask` handler already has its own Redis cache and fallback chain.
   - Response shape:
     ```json
     {
       "err": 0,
       "parsed": { "title": "...", "author": "...", "site": "..." | null },
       "results": [ { "work_id": 1, "url_id": "...", "title": "...", "author": "...",
                      "fandom": "...", "words": 123456, "sources": ["ao3", "sb"], "score": 0.93 } ],
       "suggestions": [],
       "fallback": "ask" | "request" | null
     }
     ```
   - **Zero results → fuzzy suggestions:** query candidates with `word_similarity(fi.title, $title) > 0.35` (same threshold as `builder.rs:458`), rank with the scoring helpers from `src/routes/authors_fuzzy.rs`, fill `suggestions` with the same item shape. `fallback` = `"request"` when the input parsed cleanly but nothing matched.
3. **Ambiguity handling:** return up to `limit` matches ordered by search rank; the client renders them as a disambiguation list with fandom + wordcount (Task 3). No special "exactly one match" logic needed — one result = one row to show.
4. **TDD** (all pure, no DB): parser cases — `"Governor's Gambit by Freefaller on SpaceBattles"`; `"War by Other Means by Freefaller"`; `"*Title* by Author (AO3)"`; `"Just A Title"`; URL input passthrough; alias table coverage; quote/markdown stripping; empty input.
5. **Verification:** `cargo test --lib routes::find_fic` green. Manual curl: `curl -X POST localhost:PORT/api/find-fic -d '{"query":"harry potter by some author"}'`.
6. **Commit:** `feat(find): POST /api/find-fic parses "title by author on site" into fielded search`

---

## Task 2: Wire `/api/find-fic` into the router + rate limit

**Files:** `src/routes/mod.rs`, `src/server.rs`, `src/limiter.rs` (if tiering needed)

**Steps:**

1. Mount `POST /api/find-fic` next to the search routes in `server.rs`. Public (no auth) — logged-out users are the main audience for this.
2. Apply the existing tiered limiter (`src/limiter.rs`, `Tier` / `client_ip_from_headers`) — this is a cheap DB-backed search (≤2 queries: search + optional trgm suggestions), so the generous existing search tier is fine. Match whatever `fic_suggestions` uses.
3. **TDD:** none beyond Task 1's; add one handler test asserting the limiter rejects garbage input gracefully (4xx JSON shape consistent with the rest of the API — follow `AppError`).
4. **Verification:** `cargo test --lib` green; hit the endpoint unauthenticated.
5. **Commit:** `feat(find): mount find-fic route with public rate limiting`

---

## Task 3: Dual-mode download input (frontend)

**Files:** `frontend/src/lib/components/DownloadTab.svelte`, `frontend/src/lib/ui/archive/ArchiveHome.svelte`

**Steps:**

1. In `ArchiveHome.svelte`, `handleDownload()` (~line 19) currently always stores `sessionStorage 'fichub_dl_url'` + `goto('/download')`. Change: if the input does **not** look like a URL (`!/^https?:\/\//i.test(v)` and no dot-separated host), store it instead as `fichub_dl_query` and navigate the same way.
2. In `DownloadTab.svelte` `onMount` (~line 29), add a parallel branch: read `fichub_dl_query` → run the find flow instead of the export flow:
   - `POST /api/find-fic` with the stored text.
   - 1 result → go to the work page. Multiple → disambiguation list (title, author, fandom, words, source badges), styled like `WorkBlurb.svelte`.
   - Zero results + `suggestions` → "Did you mean…" list + **"Request this fic"** button prefilling `/requests/new` via `sessionStorage 'fichub_request_prefill'` = `{title, author, site}` (Task 4).
   - `parsed === null` → one "Ask the Archive" button repeating the query against `/api/search/ask`, rendering the same list from its results.
   - The URL path is untouched: `fichub_dl_url` keeps working exactly as today.
3. Placeholder text: "Read or download a fic — paste a URL, or type *Title by Author*" (ArchiveHome label + DownloadTab input, keep `t()` i18n usage).
4. **TDD:** vitest for the pure helper `isFicUrl(v: string): boolean` (extract to `$lib/util` beside `detectSite`) — `https://…`, `www.x.com/…`, `Governor's Gambit by X`, empty. Component test: mock fetch → single result auto-navigates; zero results shows the request button.
5. **Verification:** `npx vitest run`, `npx svelte-check --threshold error`, `npm run build` (adapter-static). Manual: type a known title into the home box.
6. **Commit:** `feat(download): dual-mode input accepts "title by author" and finds archive matches`

---

## Task 4: Fic Requests auto-suggest at creation time

**Files:** `src/routes/requests.rs`, `frontend/src/routes/requests/new/+page.svelte`

**Why:** the brainstorm's "avoid duplicate requests" goal — someone writing "looking for Governor's Gambit" probably wants a fic we already have.

**Steps:**

1. **Backend:** extract Task 1's search step into a shared helper `pub async fn find_matches(db, parsed: &ParsedFindQuery, limit: usize)` in `find_fic.rs` so both callers share one implementation. In the request-create handler, after insert, run `parse_find_query` over the request `body`; if it yields a title, search and attach `matches: [...]` (same item shape) to the create response JSON. Best-effort: search failure → omit the key, never fail creation.
2. **Frontend prefill:** `/requests/new` reads `sessionStorage 'fichub_request_prefill'` (set by Task 3) on mount: prefill `title` = `{title} by {author}`, `body` = original query + site. Editable; nothing auto-submits.
3. **Debounced live suggest (small):** in `/requests/new`, on title/body blur (not per keystroke), `POST /api/find-fic` with the text; ≥1 match → dismissible info box "N fics in the archive already match — [view]" linking to work pages, capped at 3.
4. **TDD:** handler test: create request whose body names an archived fic → response contains `matches` with that work. Frontend: prefill + dismiss-box component test.
5. **Verification:** `cargo test --lib routes::requests routes::find_fic`; `npx vitest run`; manual flow: type an archived title in a new request → box appears.
6. **Commit:** `feat(requests): auto-suggest matching archive fics when creating a request`

---

## Task 5: "Did you mean?" on failed direct lookups

**Files:** `src/routes/find_fic.rs` (reuses helpers), work-page 404 branch in the frontend

**Why:** piggybacks on pg_trgm; converts dead ends into recoveries. Nearly free after Task 1.

**Steps:**

1. Extract the suggestion query from Task 1 into `pub async fn similar_titles(db, title: &str, limit: usize) -> Vec<SuggestedWork>` in `find_fic.rs` (single SQL: `word_similarity(fi.title, $1) > 0.35` + `ORDER BY similarity(fi.title, $1) DESC LIMIT $2`, only rows with `work_id IS NOT NULL`).
2. Expose `GET /api/find-fic/suggest?title=...&author=...` (public, limiter tier from Task 2) → `{ "err": 0, "suggestions": [...] }`. Used by any 404 UI.
3. Frontend: in the works `+page.svelte` error branch (work load 404), render "Not in the archive — did you mean:" + suggestions + "Request this fic" (reuse Task 4 prefill key with the attempted title). Same for the `/download` export-failure path when the URL no longer resolves.
4. **TDD:** DB-gated test for `similar_titles` (seed two near-titles + one far → correct order). Frontend: error branch renders suggestions from mock.
5. **Verification:** `cargo test --lib routes::find_fic`; manual: visit a bogus work URL → suggestions appear.
6. **Commit:** `feat(find): did-you-mean suggestions on work 404s and failed exports`

---

## Task 6: Command palette find-fic mode

**Files:** `frontend/src/lib/components/CommandPalette.svelte`

**Steps:**

1. Add a third entry kind: when typed text contains ` by ` (or has no `://` but ≥ 4 chars), show a dynamic action "Find fic: “{text}”" above page actions.
2. Selecting it sets `sessionStorage 'fichub_dl_query'` and routes to `/download` — **reuses Task 3's flow entirely**, no new search UI in the palette (the palette stays dumb; DownloadTab owns the find UX).
3. **TDD:** palette component test: typing "governor's gambit by x" surfaces the find action; typing a URL does not.
4. **Verification:** `npx vitest run`; manual Ctrl+K flow.
5. **Commit:** `feat(palette): find-fic action routes typed title+author into the download flow`

---

## Deferred (do not build now)

- **Site-side title search (Tier 2):** `search_by_title` on adapters is blocked in practice by host blocks (AO3/FFN) and only pays off for fics *not* in the archive. Prerequisite is cookie ingestion (P1#1), not adapter code. Revisit after.
- **Discord `/find` and CLI `fichub find`:** no Discord bot or CLI exists in this repo (only background workers in `src/bin/`). Both would be *new* surfaces, not wiring. The HTTP endpoint from Task 1 is the layer they'd consume later.
- **Scrape-on-demand from find-by-name:** scraping requires a URL; name → URL → scrape needs Tier 2 first. Until then, "not in archive" funnels to Fic Requests (Tasks 3/4). This also settles the brainstorm's open question #2: **request-first, no auto-scrape**.

## Open questions → decisions made in this plan

1. **Ambiguity** → show top N with fandom/words/source badges (Tasks 1/3). No forced site scope.
2. **Scrape-on-demand vs request-first** → request-first (see Deferred).
3. **Site aliases** → `SITE_ALIASES` table in `find_fic.rs`, canonical keys reused from `src/scrape/registry.rs`.
4. **Canonical resolution** → dedupe on `work_id` in Task 1; `sources` array shows all mirrors.

## Verification (whole plan)

```bash
cd ~/code/rust/ficnexus && cargo test --lib && cargo clippy --lib -- -D warnings
cd frontend && npx vitest run && npx svelte-check --threshold error && npm run build
```

Manual smoke: home box "Governor's Gambit by Freefaller on SpaceBattles" → disambiguation or direct hit; unknown title → suggestions + request button; new-request page shows the "already in the archive" box; Ctrl+K finds fic; bogus work URL shows did-you-mean.

**Order:** 1 → 2 → 3 → 4 → 5 → 6 (3 depends on 1; 4 on 1+3; 5 on 1; 6 on 3). Total ≈ 2–3 dev-days.
