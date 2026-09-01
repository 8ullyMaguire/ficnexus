# Implementation Plan — NEXT-STEPS P1–P4

> Companion to `docs/NEXT-STEPS.md`. Concrete work plan for the four
> priority passes left after Site Quality Pass 4 shipped. Follows the
> same spec-first → delegate → verify → commit workflow.

## P1 — Polish debt (half-day, can land as one commit)

| # | Task | Files | Owner |
|---|------|-------|-------|
| 1 | Strip emoji glyphs from UI text: curator hub `✒️`, HomeDashboard `🔥`/`✨`/`🧑‍🏫`/`🔍`/`⬇`, reader toolbar `⬅`/`⬆`. Replace with text labels or CSS-icon classes. | `frontend/src/routes/curator/+page.svelte`, `frontend/src/lib/components/HomeDashboard.svelte`, `frontend/src/routes/read/[urlId]/+page.svelte`, `frontend/src/lib/ui/archive/icons.ts` | 1 subagent |
| 2 | Marginalia e2e hand-test (log + verify against live): create passage thread as level-5, check icon count, tooltip, deep-link highlight, curator page excerpt. No code change if it works; file bugs if not. | n/a (QA) | human |
| 3 | Reactions click-test under scroll + many-reaction wrap + archive-skin variant. Fix anchoring if broken. | `frontend/src/lib/components/forum/ReactionPicker.svelte` | 1 subagent |
| 4 | Document `QA_BASE` in `docs/DEPLOYMENT.md` (tunnel-from-thinkcentre pattern). | `docs/DEPLOYMENT.md` | human |

**Commit**: `chore(ui): strip emoji from UI text + QA_BASE docs — P1 polish debt`

### P2 — (skipped)
Invite cohort / onboarding not needed currently.

## P3 — Backlog picks (parallel small batch, ~2 days)

1. **Search overhaul** — `main_char_attr` is gone; add UI facets (completion, min-words, min-comments, exclude-warnings, tag-type) to `/search` that map 1:1 to existing `SearchFilters`. One subagent on frontend filter panel + `page.test.ts`.
2. **OPDS 2.0 conformance** — run `opds-validator` against `/opds/new` + `/opds/manifest`, fix any failures, add KOReader-compatible `hreflang`/`summary` fields. One subagent.
   
   (Uptime probe + backup cron already set up — no action.)

## P4 — Big rocks (spec-first, then bulk delegate)

1. **Forum depth phase 2** — read-state badges (`forum_topic_read_state` table), pin/lock affordances UI, metamod queue polish. Spec via `speckit-specify`, then 3 subagents (backend table + UI + admin tools).
2. **Extension platform v3.1** — spec per `docs/v3-customization-spec.md` §11. MVP: theme token picker saved to `user_prefs`, single community-shared recipe slot. One subagent after spec approval.

   (OPDS conformance is in P3 — not duplicated here.)

## Workflow
- P1 single-commit (no spec needed — polish).
- P2 skipped.
- P3: all parallel, single-batch delegate with independent deliverables.
- P4: each item → speckit-specify → approval → delegate batch.

Gates per the usual: cargo check/tests, svelte-check 0 errors, vitest green, qa/run.js against deployed. Deploy cadence: each P-level commits independently; deploy together only after P1 + P3 + P4-core clear gates.
