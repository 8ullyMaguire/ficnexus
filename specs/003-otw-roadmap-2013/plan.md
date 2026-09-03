# Implementation Plan: OTW Roadmap 2013 Gaps

**Branch**: `003-otw-roadmap-2013` | **Date**: 2026-08-24 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-otw-roadmap-2013/spec.md` — builds every missing OTW Roadmap 2013 item not covered by `002-otwarchive-parity`.

## Summary

Deliver the 2013 Roadmap's remaining milestones as friction-free AO3 parity: media-type + fiction-type posting/filtering (0.9), work relationships (remix/translation/inspired/gift) (0.9), expanded subscriptions/history (0.9–0.10), anonymous posting + drafts + prologue/epilogue chapters (0.10), private messaging (0.10), fully translatable chrome + OTW footer layout (0.8–0.10), admin role separation + wrangling + Open Doors imports + collections/challenges polish (0.9–0.10), and public API + WCAG + test coverage (1.0). Each user story ships as an independent polished slice, verified by screenshots against `/personal/documents/code/ruby/otwarchive` views and by automated page/route/API tests. Existing FicHub extras (forum/Ask/requests/recs/progression/Marginalia/OPDS/PWA) are preserved additively.

## Technical Context

**Language/Version**: Rust 1.85 (Axum + sqlx), TypeScript 5 / SvelteKit 2 (Svelte 5 runes, adapter-static) — same as 002

**Primary Dependencies**: Axum, sqlx, PostgreSQL 16 + pgvector, Redis, Ollama (optional), SvelteKit, Vite, Vitest

**Storage**: PostgreSQL (migrations in `migrations/`), Redis (subscriptions/inbox rate-limit, cache), filesystem object store for imports

**Testing**: `cargo test --lib` + `cargo test` (route tests), `vitest` with `page.test.ts` conventions (not `+page.test.ts`), `svelte-check --output human`, axe/WCAG manual checks

**Target Platform**: Linux server (ThinkCentre M720q, `fichub.service` port 8000, `FRONTEND_DIR=/personal/documents/code/rust/fichub/frontend/build`), browser SPA via `adapter-static` + `sw.js`

**Project Type**: Web application — Rust backend + SvelteKit frontend (single repo, two deploy artifacts)

**Performance Goals**: P1 API routes p95 < 200ms (cached reads hit Redis/pg), work search p95 < 500ms for 50k works, no N+1 on relationship/subs queries; frontend TTI < 2s on archive skin

**Constraints**: NFS `fuse.mergerfs` on `/personal` (use `/media/alvaro/code-worktrees/fichub-target` for Cargo, `/tmp/fichub-frontend-build` for frontend); `GIT_OBJECT_DIRECTORY=/home/alvaro/.cache/git-objects-fichub`; no emoji in UI text; `anyhow` + `?`/`expect`, no `unwrap`; `FRONTEND_DIR` is `/personal/.../frontend/build`; `FICHUB_SKIP_MIGRATIONS=1` in prod (migrations applied via `psql -v ON_ERROR_STOP=1`); backend auth is authoritative

**Scale/Scope**: 8 user stories across ~8 slices; ~10 new tables/columns, ~12 new API groups, ~15 new pages/components; must remain slice-deployable without breaking 002's browse/filter/read flows

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution at `.specify/memory/constitution.md` is still the placeholder template (no ratified principles to enforce). Evaluated against project conventions instead:

- [x] Slice-deployable: each story has an **Independent Test** and no cross-slice hard dependency except where noted in Dependencies.
- [x] Backend-authoritative auth/permissions: FR-005/FR-008/FR-010 require server-side checks; plan keeps frontend controls non-authoritative.
- [x] No scope creep into build/infra: uptime/backup/cache-refresh schedulers are explicitly out of scope (per spec Assumptions).
- [x] Accessibility + test gates: FR-014 requires axe + automated tests per slice before next slice — plan enforces it.
- [x] Existing features preserved: FR-013 — plan segregates new tables/flags so forum/Ask/requests/recs/progression/Marginalia/OPDS/PWA are never migrated away.

**Result**: PASS — no unjustified violations.

## Project Structure

### Documentation (this feature)

```text
specs/003-otw-roadmap-2013/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── api-v1.md
│   ├── work-relationships.md
│   ├── drafts-anon-chapters.md
│   └── messaging.md
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
├── routes/               # Axum handlers: work_media.rs, work_relationships.rs,
│                         # subscriptions.rs, history.rs, drafts.rs, anon.rs,
│                         # chapters_prologue.rs, messages.rs, admin_roles.rs,
│                         # wrangling.rs, imports.rs, api_v1.rs (+ existing)
├── models/               # Domain structs extracted from handlers where needed
├── services/             # Business logic reused across routes
├── db/                   # sqlx pools
└── lib.rs / server.rs

migrations/
├── 070_work_media_types.sql
├── 071_work_relationships.sql
├── 072_subscriptions_history.sql
├── 073_drafts.sql
├── 074_anon_collections.sql
├── 075_chapters_prologue.sql
├── 076_messages.sql
├── 077_admin_roles.sql
└── 078_api_keys.sql

frontend/
├── src/lib/api/          # workMedia.ts, relationships.ts, subscriptions.ts,
│                         # drafts.ts, messages.ts, apiV1.ts
├── src/lib/i18n/dictionaries/  # new keys in *.ts (en/de/es/fr/pt-BR/zh)
├── src/routes/
│   ├── works/ + search/  # media/fiction filters
│   ├── work/[id]/relationships/
│   ├── users/[id]/subscriptions/ + readings/
│   ├── drafts/
│   ├── messages/ + inbox/
│   └── admin/roles/ + wrangling/ + imports/
└── static/ + src/lib/ui/archive/  # skins/footer/layout polish
```

**Structure Decision**: Web application — keep existing Rust backend + SvelteKit frontend layout. No new top-level projects; migrations stay sequential and small to avoid `_sqlx_migrations VersionMismatch` drift.

## Complexity Tracking

> Fill ONLY if Constitution Check has violations that must be justified

No violations to justify.

## Phases (slice order — matches spec User Stories)

1. **Slice 1 — Media & fiction type** (US1, FR-001): posting field + `/works` filter facet, URL-persisted.
2. **Slice 2 — Work relationships** (US2, FR-002): `related_works` + reciprocal display + search facet.
3. **Slice 3 — Subscriptions & History** (US3, FR-003/004): tag/pseud/author subscriptions + `readings` polish.
4. **Slice 4 — Anon + Drafts + Prologue/Epilogue** (US4, FR-005/006/007): anon collection flag, drafts autosave, chapter markers.
5. **Slice 5 — Private messaging** (US5, FR-008): DMs with block/rate-limit/report.
6. **Slice 6 — I18n + skins/footer** (US6, FR-009): all chrome translatable, OTW footer map.
7. **Slice 7 — Admin roles + wrangling + Open Doors/imports** (US7, FR-010/011): role-gated routes, canonical merges, batched idempotent imports.
8. **Slice 8 — Public API + a11y/test package** (US8, FR-012/014): `/api/v1` OpenAPI, axe, coverage gates.

Post-design Constitution re-check: PASS — structure remains single-project with per-slice migrations and tests.
