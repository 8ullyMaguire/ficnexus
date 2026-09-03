# Implementation Plan: Forum Depth Phase 2

**Branch**: `001-forum-depth-phase2` | **Date**: 2026-08-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/001-forum-depth-phase2/spec.md`

## Summary

Deliver Threadlight F6–F7 completion: per-user unread badges (via existing `forum_read_state`), curator pin/lock affordances (via existing `forum_topics.status`), and metamod queue polish (filterable queue, single verdict, unfair-rate surfacing). All storage exists; work is API contract completion + SvelteKit UI parity (archive + modern) + tests + docs.

## Technical Context

**Language/Version**: Rust 1.85 (Axum/sqlx) + SvelteKit 2 + Svelte 5 runes + TypeScript 5
**Primary Dependencies**: sqlx/PostgreSQL 16, pgvector, forum-core crate, SvelteKit adapter-node
**Storage**: PostgreSQL 16 (`forum_read_state`, `forum_topics`, `forum_metamod_votes`, `forum_mod_grants`)
**Testing**: cargo test (664+), vitest (703), qa/run.js, svelte-check
**Target Platform**: Linux server (ThinkCentre M720q) + Cloudflare + PWA
**Project Type**: Web application (Rust backend + SvelteKit SPA, single repo)
**Performance Goals**: Forum list <200ms p95 for 100 topics; read-state upsert <20ms; no N+1 on list
**Constraints**: Archive skin parity mandatory; no emoji in UI; no `unwrap()` in prod; `anyhow` errors; `page.test.ts` not `+page.test.ts`; `CARGO_TARGET_DIR=/media/...` for builds; `FRONTEND_DIR=/personal/.../frontend/build`
**Scale/Scope**: ~50 forum routes, ~100 concurrent curators; one row per user/topic for read-state

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution is placeholder (no ratified constraints). Project gates applied instead:

- [x] Spec-first, parallel subagents, orchestrator verifies/commits
- [x] No secrets in repo/specs (no DATABASE_URL, tokens)
- [x] Archive-first, AO3 parity, positive-only public surface preserved
- [x] No new infrastructure (backup/uptime already done) — not violated
- [x] Migration numbering avoids collisions (next is 070+)

GATE PASS.

## Project Structure

### Documentation (this feature)

```text
specs/001-forum-depth-phase2/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── forum-read-state.md
│   ├── forum-pin-lock.md
│   └── forum-metamod-queue.md
└── tasks.md             # Phase 2 output (via /speckit-tasks)
```

### Source Code (repository root)

```text
migrations/
  070_forum_depth_phase2.sql      # only if alters needed (indexes/comments — no new tables required)

src/
  routes/forum.rs                 # list/detail read-state, pin/lock handlers, metamod queue filters
  db/queries.rs                   # read-state queries if extracted
  server.rs                       # no change expected

frontend/
  src/lib/api/forum.ts            # read/mark-read, pin/lock, metamod queue clients
  src/lib/api/forum.test.ts
  src/routes/forum/[categorySlug]/+page.svelte      # unread badges
  src/routes/forum/board/[topicSlug].[topicId]/+page.svelte
  src/lib/components/forum/TopicThread.svelte       # pin/lock badges, locked disable, mark-read on open
  src/routes/forum/metamod/+page.svelte             # filterable queue
  src/routes/forum/moderate/+page.svelte            # mod drawer polish

tests/
  frontend: page.test.ts per route
  backend: cargo test forum::*
```

**Structure Decision**: Web application — backend + frontend in single repo. No new crate. Reuse existing `forum-core` engine and tables.

## Complexity Tracking

No constitution violations. No new project/repo. Reused tables minimize complexity.

