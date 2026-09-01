# Plan: Remove Quests Feature (audit)

## Decision
No "quests" feature exists in the FicHub codebase. Audit findings (2026-08-28):

- `src/routes/quests.rs` — filename only. Contents are reading-stats + reading-streak
  handlers (`get_reading_stats_handler`, `get_streak_handler`, `record_read_handler`).
  No quest entity, model, migration, or route.
- No `quest` table in `migrations/` (001_initial.sql or later).
- No `quest` route, page, store, or i18n key in `frontend/`.
- `books/full-stack/200k/parts/27-reading-quests.md` is a docs chapter about a
  *reading-quests* concept but no backend implements it.

## Action
Nothing to remove. File `src/routes/quests.rs` is misnamed; left as-is to avoid
breaking reading-stats/streak routes. Optional follow-up: rename to
`src/routes/reading.rs` for clarity (out of scope, not done).

## Tracking
All `books/**/*.md` are git-tracked (verified `git ls-files --others` empty).
The one dirty file `books/full-stack/200k/FicHub_full-stack_200k.md` is a
generated book assembly (doc-content refresh) — committed in this batch.
