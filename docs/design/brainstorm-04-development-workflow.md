<!-- CONSOLIDATED BRAINSTORM FILE — do not edit by hand. Generated 2026-08-13 from repo markdown (docs/src user docs excluded). -->

> HOW WE WORK + QA + DOCS: AGENTS.md (agent conventions, scraper-selection notes, current QA state), WORKFLOW.md (bug-hunting pipeline qa.sh loop, fingerprint-deduped bugs.db), DOCS_META.md (how the mdbook docs are built/served), and the current ISSUES.md (bug queue — empty, 0 open).


---


======================================================================
SOURCE: docs/AGENTS.md
======================================================================

# FicHub — Agent Rules (for fixer agents)

## Stack
- Backend: Rust 2024 / Axum 0.8, SQLx (PostgreSQL 16), Redis, Tera templates.
- Frontend: SvelteKit 5 (SPA, adapter-static) built into `frontend/build/`, served by Axum fallback.
- Repo root: `/home/alvaro/documents/code/rust/fichub` (symlink: `/personal/documents/code/rust/fichub`). **Use the `/personal/...` path for build/run.**
- DB: PostgreSQL 16, db `fichub`. Migrations in `migrations/` (sqlx applies on startup).
- Service: `fichub.service` (systemd). Run: `sudo systemctl restart fichub` after `cargo build --release`.
- Test: `cargo test` (integration tests in `tests/integration.rs`), frontend `cd frontend && npm test`. Frontend E2E/integration (renders the real layout + nav and clicks through to every routed page, plus smoke assertions for new features): `cd frontend && npm run test:e2e`. **E2E is fully green** (15/15) — the layout `routePages` list in `frontend/src/routes/+layout.svelte` must include every routed path (`/roadmap`, `/tropes`, `/blind-date`, `/feed`, `/quests`, `/read/`, `/work/`, `/curator`); if you add a new page, add its prefix there or the layout renders the home dashboard instead.
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
- The agent loop is gated: `AGENT_ENABLED=false` by default; diagnose-only
  this pass (`POST /api/admin/heal` reports what it would do, no writes).

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
- **Author batch download**: `GET /api/download/author?url=<profile_url>`
  supports AO3 and XenForo (QQ, SB, SV) author pages. SSE streaming at
  `/api/download/author/stream` for progress events.

## Current QA state
- **0 open bugs** (`node qa/report.js` → "total: 0 open").
- Full suite green on main: 582 lib + ~120 DB-gated (admin 13, heal 8,
  requests 11, ask 6, body_search 3, reader 2, …) + 464 frontend unit
  + **200 scraper-crate tests** (`cd scrapers && cargo test`).


======================================================================
SOURCE: docs/WORKFLOW.md
======================================================================

# FicHub Bug-Hunting Workflow (local-first, converges to zero)

Goal: find and fix every bug, cheaply. Deterministic checks discover facts locally (0 cloud tokens);
the fixer (cloud agent or human) only gets clean, deduped, reproducible issues. The loop repeats
until the harness finds no more bugs.

## Pipeline

```
./qa.sh --loop --autofix   (discover → local triage → local autofix → rediscover,
                            repeat until 0 open bugs or no progress)
        ↓
qa/bugs.db              (SQLite queue, fingerprint-deduped)
        ↓
qa/reports/ISSUES.md    (fixer-ready issue reports)
        ↓
Local autofix (qa/autofix.js): qwen2.5-coder:7b proposes minimal patch,
deterministic gates: file scope + cargo build + route smoke → fixed-pending-verify
        ↓
Remaining bugs → Fixer agent (cloud): read ISSUES.md + AGENTS.md,
one root cause at a time, add failing regression test first, smallest fix
        ↓
./qa.sh                 (verify: fingerprints disappear, no new ones)
        ↓
Update bug status in bugs.db (open → fixed), regen ISSUES.md
        ↓
repeat until ./qa.sh --loop exits with 0 open bugs
```

## Convergence rule

A bug is fixed only when its fingerprint stops appearing in qa/run.js output AND a
regression check (curl/Playwright/rust test) is committed. Re-running the harness must
never add the same fingerprint back.

## Files

- qa/run.js        — deterministic checks (API smoke, link crawl, OPDS, browser journeys, journalctl scan)
- qa/triage.js     — local LLM summaries (ollama llama3.2:3b); paths hallucinate, use grep ground truth
- qa/gen-issues.js — ISSUES.md from bugs.db
- qa/report.js     — dump queue
- qa/autofix.js     — local LLM autofix (qwen2.5-coder:7b): minimal patch + cargo build + route smoke gates
- qa.sh            — entry point; --loop re-runs until 0 open bugs, --autofix enables local fixing
- AGENTS.md        — fixer rules (inject into cloud agent context)

## Commands

    ./qa.sh                    # one discovery pass
    ./qa.sh --triage           # + local LLM summaries
    ./qa.sh --issues           # + regenerate ISSUES.md
    ./qa.sh --autofix          # + local LLM autofix attempt (P0/P1)
    ./qa.sh --loop --autofix   # converge: discover → fix → rediscover until 0 open (max 10 passes)
    node qa/report.js          # dump queue
    node qa/report.js --json   # machine-readable queue

## Automation

- Nightly cron `fichub-nightly-qa` 04:00 local runs `./qa.sh --triage --issues --autofix --loop`
  (local-only, no cloud tokens; output saved in cron log).

## Handoff to cloud fixer (costs tokens — use sparingly)

Point it at qa/reports/ISSUES.md + AGENTS.md. It must: reproduce, add failing test, fix
smallest, run ./qa.sh, confirm fingerprint gone, keep the test, mark bug fixed in bugs.db.
Never send the whole codebase or raw logs.


======================================================================
SOURCE: docs/DOCS_META.md
======================================================================

# FicHub Docs — How They're Built & How to Improve Them

> **Purpose of this file**: everything a contributor (human or AI) needs to
> know about the FicHub documentation system — what exists, how it's built,
> where it's served, and what could be improved. Use this as context when
> suggesting docs improvements (e.g. adding search, download buttons, better
> EPUB output).

## 1. Overview

FicHub's user-facing documentation is a set of **Markdown chapters** in
`docs/src/`, built into two formats:

1. **HTML website** — served at `/docs/` on the live site (built with
   **mdbook v0.5.4**).
2. **EPUB e-book** — `docs/FicHub_Docs.epub` (built locally with
   mdbook's epub backend; CI builds a *separate* version with pandoc).

The docs are also copied into the frontend's static directory so the SvelteKit
app serves them at `/docs/`.

## 2. Directory layout

```
docs/
├── book.toml          # mdbook config (title, html theme, epub output, header js)
├── build.sh           # local build: mdbook build → copy HTML to frontend/static + build, copy EPUB
├── src/               # ⚠️ THE SOURCE OF TRUTH — markdown chapters
│   ├── SUMMARY.md     # table of contents (chapter order)
│   ├── intro.md
│   ├── quickstart.md  # 🆕 added 2026-08-08 — 5-minute start guide
│   ├── what-is-fichub.md
│   ├── downloading.md
│   ├── searching.md   # the deep search-syntax guide
│   ├── bookmarks.md
│   ├── ratings.md
│   ├── comments.md
│   ├── recommendations.md
│   ├── profile.md
│   ├── features.md
│   ├── faq.md
│   ├── anti-bot.md    # operator-facing (how bots are stopped)
│   └── contributing.md # 🆕 contributor onboarding (junior devs)
├── book/              # mdbook build output (gitignored)
│   ├── html/          #   → HTML site
│   └── epub/          #   → EPUB ("FicHub Docs.epub")
├── theme/             # (empty dir — unused custom theme hook)
├── fichub-header.js   # injected into every docs page: FicHub navbar + back link
├── FicHub_Docs.epub   # the deliverable EPUB (gitignored)
├── WORKFLOW.md        # (unrelated: QA bug-hunting workflow — not docs)
├── FICHUB_DESIGN.md   # (architecture notes — not user docs)
├── SPECIFICATION.md   # (1847-line technical spec — not user docs)
├── STATUS.md          # (session status — not user docs)
├── IDEAS.md, TODO.md  # (scratch — not user docs)
```

Other doc-like things (NOT part of the mdbook):
- `src/routes/api_docs.rs` — serves an API reference at `/api/` (separate from
  the mdbook; plain generated HTML).
- `README.md` at repo root.

## 3. How it's built (local)

Run from `docs/`:

```bash
mdbook build          # produces book/html (HTML site) + book/epub (EPUB)
bash build.sh         # mdbook build + copy HTML → frontend/static/docs + frontend/build/docs,
                      # copy EPUB → ./FicHub_Docs.epub
```

`book.toml` highlights:

```toml
[book]
title = "FicHub Docs"
authors = ["FicHub Community"]
language = "en"

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = "https://opencommit.eu/MagicZhang/fichub"
additional-js = ["fichub-header.js"]   # injects the FicHub navbar into every page

[output.epub]         # (empty = defaults) — mdbook-epub backend produces the EPUB
```

`fichub-header.js` injects a dark sticky bar at the top of every docs page
with: FicHub logo → "Back to FicHub" → (spacer) → Download / Search /
Leaderboard links. It runs client-side.

## 4. How it's served on the live site

- The SvelteKit frontend serves `frontend/static/docs/**` as static files.
  After `bash build.sh`, the generated HTML lives in
  `frontend/static/docs/` (tracked in git) and `frontend/build/docs/`
  (gitignored build output).
- URL: `https://<host>/docs/` (e.g. `http://localhost:8000/docs/`).
- The mdbook HTML includes **elasticlunr client-side search**
  (`searchindex-<hash>.js` + `elasticlunr-*.min.js`), so the HTML site already
  has a search box (top-left, magnifier icon).
- The EPUB is NOT served by the site — it's a build artifact you download
  from the repo/CI.

## 5. How it's built (CI)

`.forgejo/workflows/docs-epub.yml` runs on pushes/PRs touching `docs/src/**`
or `docs/book.toml`:

1. Install **pandoc**.
2. Run pandoc with a **HARDCODED chapter list**:
   `intro what-is-fichub downloading searching bookmarks ratings comments
   recommendations profile features faq` (12 chapters).
3. Upload `docs/FicHub_Docs.epub` as a CI artifact (30-day retention).

⚠️ **Known divergence (2026-08-08)**: the CI pandoc command is stale:
- It does NOT include `quickstart.md` (added 2026-08-08).
- It does NOT include `anti-bot.md` (operator docs).
- It only produces the EPUB — no HTML site (local mdbook does that).
- It uses pandoc, so its EPUB styling/struct differs from the local mdbook-epub
  output. Two EPUBs, two generators, can drift.

## 6. Current docs contents (as of 2026-08-12)

| Chapter | Audience | Covers |
|---------|----------|--------|
| intro | users | what FicHub is, chapter index |
| **quickstart** | users | 5-min start: download, account, bookmark, search, offline; cheat-sheet |
| what-is-fichub | users | problem/solution, supported sites (107+ native adapters), free/open-source |
| downloading | users | flow, formats table, bookmarklet, rate-limit note, troubleshooting |
| searching | users | full search syntax: boolean AND/OR/NOT, quotes, fields, facets, chips, advanced filters, main_char_attr |
| bookmarks | users | save/view/private/remove, why bookmark (recommendations) |
| ratings | users | like/dislike, how ratings feed recommendations |
| comments | users | how to comment, community rules |
| recommendations | users | recs tab, story-page recs, home "Recommended for you", suggestions/voting |
| profile | users | profile fields, reputation, roles, leaderboard |
| features | users | download/search/social/technical feature list (incl. 107+ sites) |
| faq | users | general/download/account/search/troubleshooting Q&A |
| anti-bot | operators | bot-defense architecture (principles, layers, what to avoid) |
| contributing | contributors | junior-dev onboarding: stack, repo layout, first-run setup, dev loop, testing, QA harness, PR flow, docs contribution |

The writing style is friendly, emoji-accented, zero-PII, and aimed at
non-technical readers ("Paste the URL and click Download"). The
contributing chapter is the one exception — it targets junior developers.

## 7. Ideas already floating around (improvement candidates)

- **EPUB/HTML download button on the docs site** — a button per chapter or
  for the whole book so readers can grab a portable copy.
- **Search** — the HTML site already has elasticlunr search; the EPUB does not
  (EPUB readers have their own search). Could be improved: add a search page
  or ensure the search index covers the new chapters.
- **Consistency between the two EPUB generators** (mdbook-epub vs CI pandoc) —
  decide on ONE; the pandoc path is stale and missing chapters.
- **Include the missing chapters in CI** (quickstart, anti-bot) or make the CI
  build use mdbook too.
- **Serve the EPUB from the site** (e.g. `/docs/FicHub_Docs.epub`) instead of
  only as a CI artifact / repo file.
- **Version the docs** — link a "Last updated" or version marker per chapter.
- **A `print.html`** (mdbook already generates one) — could be exposed as a
  "Download as HTML" option.
- **Docs for the docs** — this file; also worth keeping `book.toml` comments
  and build instructions in `AGENTS.md`.

## 8. Useful commands

```bash
cd docs && mdbook build          # local build (HTML + EPUB)
cd docs && bash build.sh         # build + deploy to frontend static dirs
mdbook serve --open              # live-reload preview server
```

## 9. Git notes

- `docs/src/**` + `frontend/static/docs/**` are **tracked** (committed).
- `docs/book/` (mdbook output) and `docs/FicHub_Docs.epub` are **gitignored**
  (regenerated by `build.sh` / CI).
- The last docs rewrite (2026-08-08, commit `e406f78`) added Quickstart and
  refocused chapters on platform usage.


======================================================================
SOURCE: qa/reports/ISSUES.md
======================================================================

# FicHub Bug Queue — 2026-08-07

Generated by qa/ harness (deterministic checks + local LLM triage).
Queue DB: qa/bugs.db · Re-run: `node qa/run.js && node qa/triage.js && node qa/gen-issues.js`

## Summary

| Severity | Count |
|----------|-------|

---
