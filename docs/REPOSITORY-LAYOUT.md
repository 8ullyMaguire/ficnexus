# Repository Layout — Proposed Restructuring

> Status: proposal. This file maps every current file/folder to a clean target
> structure, explains what to delete, and why. The `src/` tree is intentionally
> untouched — it's already well-organized.

## Current problems

1. **Root is a junk drawer** — 10+ stray files (`agent.rs`, `config.rs`, `err.txt`,
   translation `.ts` files, markdown one-offs, docker-compose, env example, scripts).
2. **`docs/` mixes genres** — specs, plans, sessions, ideas, brainstorms, posts,
   mdbook source, **mdbook HTML build output** (committed!), duplicate frontend specs,
   and 30+ one-off design docs all at the same level.
3. **Duplicate content** — `docs/frontend-spec/` ≈ `docs/specs/frontend/` (slight
   drift). `docs/book/html/` is committed generated artifacts.
4. **Scratch/build dirs tracked** — `cache/`, `tmp/`, `err.txt`, `__pycache__/`.
5. **Book factory in-repo** — `books/` is a huge generated artifact factory
   (50k/100k/200k-word variants) with its own assembler scripts.
6. **Translation files (`en.ts`, `de.ts`, …) in root** — these are TypeScript i18n
   strings that belong near the frontend.
7. **`rec-engines/` and `qa/` are self-contained sub-projects** sitting at top level.
8. **`deploy/` and `docker/` are thin splits** — better merged into a single `infra/` tree.
9. **`docs/specs/forum-nodebb/`** lives in `docs/`, but newer specs live in `specs/` —
   inconsistent.

---

## Proposed target structure

```
ficnexus/
├── Cargo.toml, Cargo.lock, justfile, README.md
├── .gitignore, .gitattributes
├── .env.example
├── .pre-commit-config.yaml
├── .forgejo/                    # CI workflows
├── .github/skills/
│
├── src/                         # (UNCHANGED — already clean)
│   ├── main.rs, lib.rs, server.rs, …
│   ├── routes/, scrape/, services/, realtime/, …
│   └── bin/
│
├── frontend/                    # (UNCHANGED)
│
├── crates/                      # (UNCHANGED — forum-core, forum-import, scrapers, consensus)
│
├── migrations/                  # (UNCHANGED — main app migrations)
│
├── tests/                       # (UNCHANGED — integration tests)
│
├── qa/                          # (UNCHANGED — self-contained JS harness)
│   ├── package.json, run.js, api-walk.js, bugs.db, …
│   └── reports/                 # gitignored outputs
│
├── scripts/                     # operational scripts (seeders, embedders, fixers)
│   ├── seed/
│   │   ├── seed_feature_clusters.py
│   │   ├── seed_roadmap_clusters.py
│   │   ├── populate_royalroad.py
│   │   └── consensus_seed.json
│   ├── ml/
│   │   └── generate_embeddings.py
│   ├── maintenance/
│   │   ├── fix-migrations.sh
│   │   ├── rank_issues.py
│   │   ├── score_features.py
│   │   └── feature_inventory.py
│   └── scrape/
│       └── scrape_ficai_popular.py
│
├── infra/                       # DEPLOY + DOCKER merged
│   ├── docker/
│   │   ├── Dockerfile                   # (from root)
│   │   ├── docker-compose.yml           # (from root)
│   │   ├── calibre.Dockerfile           # (from docker/)
│   │   ├── cross-build.Dockerfile       # (from docker/)
│   │   └── .dockerignore               # (from root)
│   ├── systemd/
│   │   ├── fichub.service
│   │   ├── publish-scheduled.service    # (from deploy/systemd/)
│   │   └── publish-scheduled.timer
│   └── deploy.sh                       # (from root)
│
├── i18n/                        # translation files (from root .ts)
│   ├── en.ts, de.ts, es.ts, fr.ts, pt-BR.ts
│   └── README.md                # note: these feed the frontend build
│
├── docs/                        # CLEANED — see breakdown below
│   ├── README.md                # index / "what lives where"
│   ├── src/                     # mdbook source (SUMMARY.md, *.md chapters)
│   ├── theme/                   # mdbook theme (CSS/JS)
│   ├── book.toml                # mdbook config
│   ├── specs/                   # MERGED: all feature specs
│   │   ├── forum-nodebb/        # (MOVED from docs/specs/forum-nodebb/)
│   │   ├── 001-forum-depth-phase2/
│   │   ├── 002-otwarchive-parity/
│   │   ├── 003-otw-roadmap-2013/
│   │   └── 004-site-stats/
│   ├── plans/                   # time-stamped implementation plans (KEEP)
│   ├── sessions/                # session summaries (KEEP)
│   ├── adr/                     # Architecture Decision Records (new bucket)
│   ├── design/                  # one-off design docs (brainstorms, audits, UX)
│   └── reference/               # FORUM-API-CONTRACT, DEPLOYMENT, etc.
│
├── rec-engines/                 # recommendation engine recipes (unchanged)
│
└── books/                       # book factory — see "Books decision" below
```

---

## Detailed file mapping

### A. Delete / gitignore (junk & generated)

| Current path | Action | Reason |
|---|---|---|
| `err.txt` | **DELETE** | stale build log |
| `cache/` | **gitcheck + delete** | empty scratch dir |
| `tmp/` | **gitignore** | runtime scratch |
| `scripts/__pycache__/` | **gitignore** | Python bytecode |
| `docs/book/` (html/epub) | **gitignore** | mdbook build output |
| `docs/FicHub_Docs.epub` | **gitignore** or delete | duplicate of book output |
| `.unison..github.*.unison.tmp/` | **gitignore** | unison sync leftover |
| `node_modules/` (root) | verify gitignored | |
| `qa/node_modules/` | verify gitignored | |
| `qa/bugs.db` | verify gitignored | |
| `qa/artifacts/`, `qa/crawl-cache/` | verify gitignored | |

### B. Move to `infra/`

| From | To |
|---|---|
| `Dockerfile` | `infra/docker/Dockerfile` |
| `docker-compose.yml` | `infra/docker/docker-compose.yml` |
| `.dockerignore` | `infra/docker/.dockerignore` |
| `docker/calibre.Dockerfile` | `infra/docker/calibre.Dockerfile` |
| `docker/cross-build.Dockerfile` | `infra/docker/cross-build.Dockerfile` |
| `deploy/systemd/publish-scheduled.service` | `infra/systemd/` |
| `deploy/systemd/publish-scheduled.timer` | `infra/systemd/` |
| `deploy.sh` | `infra/deploy.sh` |

### C. Move to `i18n/`

| From | To |
|---|---|
| `en.ts`, `de.ts`, `es.ts`, `fr.ts`, `pt-BR.ts` | `i18n/en.ts`, … |

> Note: verify the frontend build imports these by path. If they're loaded via
> Vite glob or a webpack alias, update the import path. If they're committed for
> reference only (not imported), note that in `i18n/README.md`.

### D. Move to `scripts/` (subdivided)

| From | To |
|---|---|
| `scripts/consensus_seed.json` | `scripts/seed/consensus_seed.json` |
| `scripts/seed_feature_clusters.py` | `scripts/seed/` |
| `scripts/seed_roadmap_clusters.py` | `scripts/seed/` |
| `scripts/populate_royalroad.py` | `scripts/seed/` |
| `scripts/generate_embeddings.py` | `scripts/ml/` |
| `scripts/fix-migrations.sh` | `scripts/maintenance/` |
| `scripts/rank_issues.py` | `scripts/maintenance/` |
| `scripts/score_features.py` | `scripts/maintenance/` |
| `scripts/feature_inventory.py` | `scripts/maintenance/` |
| `scripts/scrape_ficai_popular.py` | `scripts/scrape/` |
| `scripts/install_systemd_timers.sh` | `scripts/maintenance/` |

### E. Stray root `.md` files → where they belong

| File | Target | Category |
|---|---|---|
| `README.md` | keep in root | project entry point |
| `AGENTS.md` | `docs/reference/AGENTS.md` | agent/operator reference |
| `SPECIFICATION.md` | `docs/design/specification.md` | design doc |
| `FICHUB_DESIGN.md` | `docs/design/site-design.md` | design doc |
| `IDEAS.md` | `docs/design/ideas.md` | design doc |
| `agent.rs` | **DELETE** or `src/bin/agent.rs` if usable | stray Rust snippet |
| `config.rs` | **DELETE** or `src/config.rs` if usable | stray Rust snippet |

> `agent.rs` and `config.rs` in the repo root do NOT compile (Cargo only sees
> `src/`). They are either experiments or were accidentally placed. Confirm and
> delete or relocate into `src/`.

### F. `docs/` cleanup — the big one

#### F1. Merge duplicate frontend specs
- `docs/frontend-spec/` and `docs/specs/frontend/` cover the same ground.
- **Keep** `docs/specs/frontend/` (newer, lives in the canonical `specs/` tree).
- **Diff** the two; port any unique content from `frontend-spec/` into `specs/frontend/`.
- **Delete** `docs/frontend-spec/`.

#### F2. Move `docs/specs/forum-nodebb/` → `docs/specs/forum-nodebb/`
Already there — but it should move to the top-level `specs/` to match the numbered
specs. **Move** to `specs/forum-nodebb/`.

#### F3. mdbook source stays, build output goes
- KEEP: `docs/src/`, `docs/theme/`, `docs/book.toml`, `docs/build.sh`, `docs/build_docs_map.py`
- GITIGNORE: `docs/book/`, `docs/FicHub_Docs.epub`

#### F4. One-off docs → `docs/design/`

Move these brainstorms, audits, and design riffs:
- `ao3-features-gap.md`
- `ARCHIVE-UI-PARITY-PLAN.md`
- `bot-cli-parity-gaps.md`
- `brainstorm-01-system-overview.md`
- `brainstorm-02-roadmap-status.md`
- `brainstorm-03-scraper-rec-platforms.md`
- `brainstorm-04-development-workflow.md`
- `brainstorm-05-implementation-plans.md`
- `ficnexus-feature-audit.md`
- `FORUM-BRAINSTORM.md`
- `FORUM-NODEBB-PARITY.md`
- `frontend-design.md`
- `INTERFACE-STYLES.md`
- `lemmy-rust-announcement.md`
- `missing.md`
- `PROMPT-frontier-llm-fanfiction-platform.md`
- `REACTION-EMOJI-DESIGN.md`
- `REBUILD-NOTES.md`
- `reddit-announcement.md`
- `SITE-PREMISE.md`
- `SPEC-COMMUNITY-PLATFORM.md`
- `SPEC-COMMUNITY-PLATFORM-v2.md`
- `THREADLIGHT-FEATURES.md`
- `v3-customization-spec.md` (also keep a copy or symlink in `docs/src/` for mdbook)
- `xp-removal-todo.md`

#### F5. Reference docs → `docs/reference/`

- `DEPLOYMENT.md`
- `FORUM-API-CONTRACT.md`
- `USER-ACTIONS.md`
- `VERIFICATION-CHECKLIST.md`
- `AGENTS.md` (from root)

#### F6. Ideas → `docs/design/ideas/`

Already partially in `docs/ideas/`. Merge:
- `docs/ideas/ai-features-orange-pi.md`
- `docs/ideas/recommendation-engine-v31-brainstorm.md`

#### F7. Posts → keep `docs/posts/` (already clean)

#### F8. Plans → keep `docs/plans/` (already clean, but prune stale ones)

Plans older than 30 days that are fully executed should be moved to
`docs/plans/archive/` or deleted. Candidates:
- `2026-08-18-permanent-link.md`
- `2026-08-30-session-features.md`
- `code-along-*.md` series (check if executed)

#### F9. Sessions → keep `docs/sessions/` (already clean)

#### F10. ADR bucket → `docs/design/adr/` (new)

Architecture decision records for consequential choices (e.g., "why we chose
forum-core as a separate crate", "why XP ledger split exists").

---

## What stays put (already clean)

- `src/` — well-organized by concern (routes/, scrape/, services/, realtime/, …)
- `crates/` — each crate is self-contained with its own Cargo.toml, migrations, README
- `frontend/` — standard SvelteKit layout
- `migrations/` — chronological SQL migrations
- `tests/` — integration tests
- `qa/` — self-contained JS harness (has its own package.json)
- `rec-engines/` — recommendation engine recipes
- `.forgejo/` — CI workflows
- `.github/skills/` — agent skills
- `.specify/` — speckit tooling (project-scoped config)
- `.hermes/` — hermes agent plans (project-scoped)

---

## Open decisions (confirm before executing)

1. **Books** — `books/` is ~100+ files of LLM-generated tutorials (50k/100k/200k-word
   variants). Options:
   - (a) Move to a separate repo `fichub-books` (recommended — it's a product, not source).
   - (b) Keep but gitignore generated parts, keep only source outlines.
   - (c) Delete if superseded by the mdbook (`docs/src/`).

2. **i18n path** — confirm whether the frontend build imports `en.ts` etc. by
   filesystem path. If yes, update the glob/alias. If they're reference-only,
   moving them is safe.

3. **`agent.rs` / `config.rs`** — open them; if they're usable, move into `src/bin/`
   or `src/`. If they're dead experiments, delete.

4. **Stale plans** — prune `docs/plans/` to keep only active/referenced plans.

5. **`docs/src/people.md`** and **`docs/src/tag-search.md`** — not in SUMMARY.md.
   Either wire them into the book or move to `docs/reference/`.

---

## Execution order

1. **gitignore cleanup** — add `tmp/`, `err.txt`, `cache/`, `docs/book/`, `__pycache__/`.
2. **Delete obvious junk** — `err.txt`, `.unison.*.tmp/`.
3. **Create new dirs** — `infra/`, `i18n/`, `scripts/{seed,ml,maintenance,scrape}`, `docs/{design,reference,adr}`.
4. **Move files** — batch by section (B–F above). Commit after each section.
5. **Update references** — grep for moved paths (CI workflows, imports, links in docs).
6. **Merge duplicate specs** — diff and consolidate `frontend-spec/` vs `specs/frontend/`.
7. **Prune** — archive stale plans, decide on books.
8. **Add `docs/README.md`** — index explaining the new structure.
9. **Full build + QA** — `cargo build --release`, `node qa/run.js`, frontend tests.
