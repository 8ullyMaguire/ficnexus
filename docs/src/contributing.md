# Contributing to FicHub

Welcome! This guide gets a junior developer from "I cloned the repo" to
"my first merged PR" — with no prior FicHub context required.

FicHub is a self-hosted fanfiction archive + download server (a modern
fichub.net replacement). It scrapes stories from fanfiction sites, builds
EPUB/HTML/PDF/TXT/MOBI/AZW3 files, and runs a community platform on top:
accounts, bookmarks, ratings, reviews, comments, reading lists, follows,
feeds, an in-browser reader, and a recommendation engine.

---

## 1. The stack at a glance

| Layer | Technology | Notes |
|-------|-----------|-------|
| Backend | Rust 2024, Axum 0.8, SQLx 0.9 | `src/` — HTTP API, scraping, exports, social |
| Frontend | SvelteKit 5 (SPA, adapter-static) | `frontend/` — built to `frontend/build/`, served by Axum |
| Database | PostgreSQL 16 + pgvector | Migrations in `migrations/`, applied on startup |
| Cache/limits | Redis | Rate limiter token bucket, shadowban state |
| ML/embeddings | Ollama (nomic-embed-text) | Roadmap consensus, auto-tagger, rec embeddings |
| Scraping | `fanfic-scrapers` crate (no Python dependency) | `scrapers/` — native adapters for all 107 FFF-parity sites |
| Docs | mdBook (Rust) | `docs/` — this site |

> The frontend is a **SPA served from one origin** — Axum serves
> `frontend/build/` as a fallback, so `/docs/` is just static HTML shipped
> inside the SPA.

---

## 2. Repository layout

```
fichub/
├── src/               # Rust backend
│   ├── server.rs      # Axum router: all routes registered here
│   ├── routes/        # one module per feature area (download, auth, social, admin…)
│   ├── scrape/        # scraper subsystem + registry (fanfic-scrapers)
│   ├── heal/          # self-healing classifier (LLM-assisted)
│   ├── limiter/       # Redis token-bucket rate limiter
│   └── epub/          # EPUB/TXT/MD builder (pure Rust)
├── scrapers/          # fanfic-scrapers crate (standalone, published to crates.io)
│   └── src/sites/     # one adapter module per site/family (ao3, ffnet, xenforo…)
├── frontend/          # SvelteKit 5 SPA
│   ├── src/routes/    # +page.svelte files; one route per page
│   ├── src/lib/       # shared components, API client, stores
│   └── static/docs/   # BUILT docs output (mdBook HTML; committed)
├── migrations/        # SQLx migrations 001…N (auto-applied on boot)
├── docs/              # mdBook source (docs/src/*.md + SUMMARY.md)
├── tests/             # Rust integration tests (DB-gated)
├── qa/                # QA harness (node qa/run.js — Playwright + API audit)
├── scripts/           # data/seed/consensus helper scripts (Python)
├── deploy.sh          # build + deploy to the prod machine
└── Cargo.toml         # backend workspace deps
```

---

## 3. First-run setup (5 minutes)

### Prerequisites

- Rust toolchain (`rustup default stable`)
- Node.js 20+ (frontend)
- PostgreSQL 16 running, with a `fichub` database
- Redis running on `localhost:6379`
- Optional but recommended: Ollama (for recs/consensus; the app falls back
  gracefully when offline)

### Backend

```bash
cp .env.example .env        # edit DATABASE_URL, REDIS_URL to match your machine
cargo run                   # starts on :8000 (or $PORT)
```

First boot applies migrations automatically. Check it's up:

```bash
curl http://localhost:8000/api/health
```

### Frontend

```bash
cd frontend
npm install
npm run dev                 # Vite dev server (usually :5173, proxies API)
```

> In production the SPA is built (`npm run build`) into `frontend/build/`
> and served by Axum — so a backend change needs no frontend rebuild, and
> a frontend change needs `npm run build` before it appears on the live
> site.

### The scraper crate (separate, published)

```bash
cd scrapers
cargo test                  # runs the adapter test suite
```

The crate is **general-purpose and published to crates.io**
(`fanfic-scrapers`). It has zero FicHub internals — keep it that way.

---

## 4. Your dev loop

1. **Pick a small issue** (check the consolidated plan in
   `docs/brainstorm-02-roadmap-status.md`).
2. **Create a branch**: `git checkout -b feat/your-feature`.
3. **Make the smallest change that works.** No unrelated refactors.
4. **Test it** (see §5).
5. **Run the QA harness** (§6).
6. **Open a PR** to `main` (§7).

### Golden rules for a junior PR

- **Never weaken or delete a test.** Every fixed bug adds a regression test.
- **Don't change migrations/schema without approval.** If a migration is
  already applied to prod, changing it breaks the checksum check.
- **Auth/payment/security logic requires human review.**
- **Prefer explicit failure over hidden assumptions.** If something can't be
  scraped or parsed, return a clear error — don't silently return empty data.
- **Commit often, small commits.** Conventional Commits:
  `feat:`, `fix:`, `test:`, `docs:`, `chore:`.

---

## 5. Testing

|| Suite | Command | Notes |
||-------|---------|-------|
|| Backend unit + integration | `cargo test` | 582+ lib tests |
|| DB-gated integration | `cargo test --test <suite> -- --test-threads=1` | Run suites individually (they hold their own DB mutex) |
|| Frontend unit | `cd frontend && npm test` | vitest, 464+ tests |
|| Frontend E2E | `cd frontend && npm run test:e2e` | Playwright via vitest config; 15/15 green |
|| Frontend coverage gate | `cd frontend && npm run coverage` | 65% lines / 55% funcs / 60% branches on `src/lib/**` |
|| Scraper crate | `cd scrapers && cargo test` | 200+ adapter tests |

### New feature? New tests, always

- Backend: unit test the handler logic; integration test the endpoint (with
  the DB suite).
- Frontend: a `page.test.ts` next to the route (not `+page.test.ts` —
  that breaks the build). Add the route prefix to the `routePages` list in
  `frontend/src/routes/+layout.svelte` or the layout renders the dashboard
  instead.
- Scraper: every new site adapter ships 4 unit tests (can_handle, id
  parsing, date parsing, body extraction) against real HTML fixtures.

---

## 6. The QA harness (run before every PR)

```bash
node qa/run.js
```

This runs the full deterministic QA: API smoke, link crawl, OPDS checks,
browser journeys (Playwright with console/network capture), and a backend
log scan. It's the closest thing to a staging environment you can run
locally.

Other QA tools:

```bash
node qa/api-walk.js              # scriptable API-surface audit (97/97 green)
node qa/triage.js                # enrich findings with local LLM
node qa/gen-issues.js            # emit qa/reports/ISSUES.md from the bug queue
```

QA state is tracked in `qa/bugs.db` (SQLite). Fingerprints dedupe; status
`open` → `fixed` when the regression test passes.

---

## 7. Submitting a PR

1. `git push origin your-branch` → open a PR on
   `https://opencommit.eu/MagicZhang/fichub` (or the GitHub mirror).
2. **Describe what changed and why**, and paste the test/QA output.
3. CI runs: build, backend tests, frontend tests + coverage, docs EPUB
   build. Make them green.
4. Expect review feedback. Small, well-tested PRs merge fast.

---

## 8. Where things live (feature cheat-sheet)

|| Feature | Backend | Frontend |
||---------|---------|----------|
|| Download/export | `src/routes/download.rs`, `src/epub/` | `src/routes/+page.svelte` (Download tab) |
|| Search | `src/routes/search.rs` (+ pgvector) | `searching.md` docs, search page |
|| Bookmarks | `src/routes/bookmarks.rs` | `src/routes/bookmarks/*` |
|| Ratings/reviews | `src/routes/ratings.rs` | story page components |
|| Comments | `src/routes/comments.rs` | `src/routes/+page.svelte` comments |
|| **Forum** | `src/routes/forum.rs`, `forum_core/` crate, `src/server.rs` forum routes | `src/routes/forum/*` (`/forum`, board, search, moderate, metamod) |
|| Recommendations | `src/routes/recs.rs`, `rec-engines/` | Recommendations tab |
|| **Progression** | `src/services/progression.rs`, `src/routes/progression.rs` | `src/routes/features/`, `src/lib/api/theme.ts` |
|| **Recipes** | `src/services/recipes.rs`, `src/routes/recipes.rs` | `src/routes/settings/recipes/` |
|| **Themes** | `src/routes/social.rs` (PUT /api/me/theme) | `src/lib/themes/`, `src/routes/settings/theme/` |
|| Admin | `src/routes/admin.rs` | `src/routes/admin/*` |
|| Scraping | `src/scrape/`, `scrapers/` | — |
|| Self-healing | `src/heal/` | admin UI |
|| Docs | `docs/` | served at `/docs/` |

---

## 9. Contributing to these docs

The docs are an **mdBook**. To change them:

```bash
cd docs
# edit src/*.md (add a chapter: create the .md + add it to SUMMARY.md)
mdbook build          # produces book/html + book/epub
bash build.sh         # also copies HTML into frontend/static/docs + build/docs
```

Then commit `docs/src/*` **and** the regenerated `frontend/static/docs/*`.

Style guide: friendly, plain language, no emoji (a few UI glyphs like ♡ or →
that name real on-screen elements are fine), zero-PII, aimed at non-technical
readers. The `docs/src/` chapters are the user-facing docs; the consolidated
planning/design docs live in `docs/brainstorm-*.md`.

---

## 10. Mental models that help

- **The site is a cache + export machine first, a community second.**
  Scrape → store body → export on demand. Social features sit on top.
- **The scraper crate is shared.** Improvements to `scrapers/` benefit
  every consumer (and it's published on crates.io — good practice for
  public Rust APIs).
- **Determinism is sacred.** Tests must be runnable locally, no network
  dependence. Fixtures, not live scrapes.
- **Read the docs before guessing.** `docs/brainstorm-01-system-overview.md`
  is the full system spec (architecture, schema, endpoints, design notes);
  `docs/brainstorm-02-roadmap-status.md` is the canonical plan; `docs/AGENTS.md`
  is the agent-rules file (also useful for humans!).

---

## 11. Getting help

- **Repo**: https://opencommit.eu/MagicZhang/fichub (public mirror on
  GitHub too)
- **Issues/roadmap**: see `docs/brainstorm-02-roadmap-status.md`
  (shipped / in-flight / prioritized suggestions)
- **Community**: Discord invite in `docs/src/faq.md`

Welcome aboard — happy hacking!
