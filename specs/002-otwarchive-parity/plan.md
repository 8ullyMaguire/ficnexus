# Implementation Plan: OTW Archive Parity — Friction-Free AO3 Migration

**Branch**: `002-otwarchive-parity` | **Date**: 2026-08-24 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-otwarchive-parity/spec.md` — go through `otwarchive` code and polish FicHub so AO3 users feel zero friction, slice-by-slice, visually as close as possible, without regressing FicHub extras.

## Summary

Deliver AO3 parity as incremental polished verticals (browse/filter/blurb → read/download → kudos/bookmarks/subscriptions+history → comments/inbox → pseuds/co-creators/series/collections → challenges/skins). Each slice uses `otwarchive` `app/views`/`app/controllers` as visual/behavioral reference, maps to FicHub's Rust/Axum+Postgres+SvelteKit stack, and ships with manual screenshot checklist before next slice. Phase 1 (P1 browse+read) is MVP.

## Technical Context

**Language/Version**: Rust 1.85, SvelteKit 2 + Svelte 5 runes, TypeScript 5
**Primary Dependencies**: Axum, sqlx, PostgreSQL 16 + pgvector, Redis, SvelteKit adapter-static, zerafina skin, archive mode
**Storage**: PostgreSQL (works/chapters/bookmarks/comments/tags/series/collections/skins), Redis (sessions/rate-limit), filesystem body cache
**Testing**: `cargo test --lib`, `cargo check`, Vitest `page.test.ts` (not `+page.test.ts`), `svelte-check`, manual screenshot checklists per slice
**Target Platform**: Linux (ThinkCentre), web (desktop + mobile), offline PWA reader
**Project Type**: Web application (backend `src/` + frontend `frontend/`)
**Performance Goals**: Work list <500ms p95, work show <300ms, no layout shift on filter apply
**Constraints**: No emoji in UI text (except 🌐 in LocaleSelector + reaction data), archive skin typography must match AO3, existing FicHub features (forum/requests/recs/progression/translations) must not regress
**Scale/Scope**: 50+ OTW views/controllers/models to audit; 6 stories sliced into 5 deployable increments

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- No new constitution yet (placeholder) — no gates to violate.
- Principles applied: visual/behavioral parity (not Rails port), slice-by-slice shippability, manual QA before next slice, no regression to FicHub extras (additive placement).
- PASS

## Project Structure

### Documentation (this feature)

```text
specs/002-otwarchive-parity/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
src/
├── routes/
│   ├── works.rs          # work index/show, filters, sorts
│   ├── chapters.rs       # chapter nav, entire-work
│   ├── bookmarks.rs      # bookmarks + rec flag
│   ├── kudos.rs          # kudos
│   ├── subscriptions.rs  # subscriptions / history
│   ├── comments.rs       # comments + inbox
│   ├── pseuds.rs         # pseuds (new)
│   ├── series.rs         # series
│   ├── collections.rs    # collections
│   └── skins.rs          # skins (new)
├── db/
│   ├── queries.rs
│   └── models/
└── ...

frontend/
├── src/
│   ├── routes/
│   │   ├── works/[workId]/
│   │   ├── read/[urlId]/
│   │   ├── bookmarks/
│   │   ├── subscriptions/
│   │   ├── inbox/
│   │   ├── pseuds/
│   │   ├── series/[id]/
│   │   └── collections/
│   ├── lib/components/
│   │   ├── WorkBlurb.svelte
│   │   ├── WorkMeta.svelte
│   │   ├── BookmarkForm.svelte
│   │   ├── CommentThread.svelte
│   │   └── SkinPicker.svelte
│   └── lib/api/
└── static/styles/zerafina-skin.css

tests/
├── contract/ (forum/work API contracts)
├── unit/ (api/*.test.ts)
└── manual/ (screenshot checklists per slice in quickstart.md)
```

**Structure Decision**: Web application — backend `src/` + frontend `frontend/` — mirrors existing FicHub layout. No new top-level projects.

## Complexity Tracking

> Fill ONLY if Constitution Check has violations that must be justified

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |

