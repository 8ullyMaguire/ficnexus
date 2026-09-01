# FicHub — Agent Rules (for fixer agents)

## Stack
- Backend: Rust 2024 / Axum 0.8, SQLx (PostgreSQL 16), Redis, Tera templates.
- Frontend: SvelteKit 5 (SPA, adapter-static) built into `frontend/build/`, served by Axum fallback.
- Repo root: `/home/alvaro/documents/code/rust/fichub` (symlink: `/personal/documents/code/rust/fichub`). **Use the `/personal/...` path for build/run.**
- DB: PostgreSQL 16, db `fichub`. Migrations in `migrations/` (sqlx applies on startup).
- Service: `fichub.service` (systemd). Run: `sudo systemctl restart fichub` after `cargo build --release`.
- Test: `cargo test` (integration tests in `tests/integration.rs`), frontend `cd frontend && npm test`. Frontend E2E/integration (renders the real layout + nav and clicks through to every routed page, plus smoke assertions for new features): `cd frontend && npm run test:e2e`. **E2E is fully green** (15/15) — the layout `routePages` list in `frontend/src/routes/+layout.svelte` must include every routed path (`/roadmap`, `/tropes`, `/blind-date`, `/feed`, `/quests`, `/read/`, `/work/`, `/curator`, `/forum`); if you add a new page, add its prefix there or the layout renders the home dashboard instead.
- Coverage gate: `cd frontend && npm run coverage` (vitest v8, thresholds in `frontend/vite.config.ts` — 65% lines / 55% funcs / 60% branches on `src/lib/**`). CI job `frontend-coverage` in `.forgejo/workflows/ci.yml` runs it and fails the build below threshold; the docker job depends on it. Raise thresholds toward 80/70 as lib coverage grows.

## QA harness (local-first, deterministic)
- `qa/run.js` — full deterministic QA: API smoke, link crawl, OPDS checks, browser journeys (Playwright, console/network capture), backend log scan (journalctl). **Run after any change: `node qa/run.js`.** Requires: chromium at `~/.cache/ms-playwright/chromium-1208`, `node qa/` deps installed.
- `qa/api-walk.js` — scriptable API-surface audit. Parses every route from `src/server.rs`, hits each endpoint, asserts auth gates (anonymous 4xx for auth/admin/non-GET), flags 5xx + stub-like responses. **Run against a live backend: `node qa/api-walk.js` (default http://localhost:8000) or `QA_BASE=https://fichub.polarisocial.xyz node qa/api-walk.js`.** Exit 0/1 for CI. **97/97 green.**
- `qa/triage.js` — enrich findings with local LLM (ollama llama3.2:3b). Summaries only; **do not trust its file paths**.
- `qa/gen-issues.js` — emit `qa/reports/ISSUES.md` from `qa/bugs.db`.
- Bug queue: `qa/bugs.db` (SQLite). Fingerprints dedupe; status `open` → `fixed` when regression test passes.

## Rules for the fixer
1. Make the smallest change that fixes the failing check. No unrelated refactors.
2. **Never delete or weaken tests.** Every fixed bug adds a regression check (curl/Playwright/rust test).
3. Do not change migrations/schema without human approval. If a migration already applied to prod is altered, mark in `_sqlx_migrations`.
4. Auth/payment/security logic changes require human review.
5. Verify: `cargo build --release` + `sudo systemctl restart fichub` + `node qa/run.js` → the fingerprint for your bug disappears and no new findings appear.
6. After fixing, run `node qa/triage.js && node qa/gen-issues.js` to refresh ISSUES.md, update status in bugs.db.

## Known quirks
- SQLite: `commit` and `runs` are reserved words; avoid as column/table names.
- OPDS routes need `?token=` for auth; root catalog must include `rel="self"`; content-type must be XML not JSON.
- PG column types: watch for i16 vs INT4 mismatches in `query_as` tuples (root cause of past leaderboard 500s — now fixed).
- The 400-as-401 convention: auth-required endpoints return HTTP 400 with `{"err":401}`. QA treats 500 on these as the bug.
- The 400-as-403 convention: role-gated endpoints return HTTP 400 with `{"err":403}` in the body (e.g. Fic Requests accept/delete).
- axum 0.8 forbids two routes at the same path level with different param names (`{id}` vs `{name}`) — use distinct literal subpaths (e.g. `/api/authors/by-name/{name}`).
- Route test files must be `page.test.ts` (not `+page.test.ts` — breaks the build).
- DB-gated test suites each hold their OWN mutex — combined runs (`cargo test --test a --test b`) can transiently flake from cross-suite seed pollution; run suites individually (`--test-threads=1`) for deterministic results.
- `qa/bugs.db`, `qa/node_modules`, `qa/crawl-cache`, `qa/artifacts` are gitignored (see .gitignore).

## Agent endpoint (self-healing)
- Default LLM endpoint for the fixer/agent loop: CommandCode
  `https://api.commandcode.ai/provider/v1/` (OpenAI-compatible
  chat/completions). Key in env `COMMANDCODE_API_KEY` (or `AGENT_API_KEY`).
  Model: `deepseek/deepseek-v4-flash` (fast, tool-capable) — set
  `AGENT_MODEL` to override.
- Local fallback: `AGENT_OLLAMA_URL` (default http://localhost:11434) with a
  small model (e.g. llama3.2:3b) when the remote API is unreachable.
- The agent loop is gated: `AGENT_ENABLED=false` by default.
- **M1 shipped**: failure telemetry + classifier (transient/blocked/structural/systemic) + fingerprint + debounce + HTML snapshots. `POST /api/admin/heal` diagnose-only.
- **M2 shipped**: on-the-fly scraper creation (`AGENT_USE_ON_FLY=true`). On structural failure, the agent extracts metadata from the HTML snapshot, validates it, and completes the pending export. Autonomy OFF by default.
- **M3 shipped**: bulk admin actions (commit b017681). **Fandom landing pages shipped** (commit 21a0440).

## Site-as-cache + body blobs
- Scraped fic bodies are cached as JSON blobs under `BODY_CACHE_DIR`
  (default `/public/literature/fichub/bodies`, attached drive — NOT the
  DB). Exports reuse the cached body (no re-scrape).
- Curator content-fix endpoints: `PUT/DELETE/GET /api/curator/content/{url_id}`
  (role ≥ 10). Use PUT to fix a wrong scraped body (the actual fic post,
  not comments); DELETE to force a re-scrape.
- Scraper selection: `find_specific_or_fallback` prefers native scrapers
  (e.g. XenForo, AO3); no Python FanFicFare CLI dependency (the
  `fanficfare` fallback is gated behind the crate's `fff-fallback` feature,
  off by default). Add new XenForo boards to `XENFORO_DOMAINS` in
  src/scrape/sites/xenforo.rs.

## Current QA state
- **0 open bugs** (`node qa/report.js` → "total: 0 open").
- Full suite green on main: 582 lib + ~170 DB-gated (admin 13, heal 8,
  requests 11, ask 6, body_search 3, reader 2, **forum_api 49**, …) + 464
  frontend unit + **200+ scraper-crate tests** (`cd scrapers && cargo test`).
- v3 progression/recipe/theme tests included in lib suite (migration 052/053
  tables, recipe CRUD, feature gate `can()` logic, theme token application).