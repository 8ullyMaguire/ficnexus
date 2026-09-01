# Research: Forum Depth Phase 2

**Feature**: 001-forum-depth-phase2 | **Date**: 2026-08-24

## Decision 1: Read-state storage — reuse `forum_read_state`

**Decision**: No new table. Reuse `migrations/041_forum_core.sql:forum_read_state(user_id, topic_id, last_read_post_id, updated_at PK(user_id,topic_id))`. Upsert on `GET /api/forum/topics/{id}` or explicit `POST .../read`.

**Rationale**: Table already exists with correct PK and FKs. Matches spec FR-001/012 (one row per user/topic, idempotent). Existing `POST /api/forum/topics/{id}/read` handler confirms pattern; list endpoint can LEFT JOIN to compute unread.

**Alternatives**: New `forum_topic_read_state` table — rejected, duplicates existing.

## Decision 2: Pin/Lock — reuse `forum_topics.status`

**Decision**: No new columns. Reuse `forum_topics.status TEXT CHECK IN ('open','locked','pinned','archived')` + `is_hidden/deleted_at` (041:31-32). Add API `POST /api/admin/forum/topics/{id}/pin|lock` already stubbed in forum.rs header (role≥5); implement ordering as `ORDER BY (status='pinned') DESC, last_activity_at DESC` and expose `status` in list/detail.

**Rationale**: Schema already supports pinned/locked as status values. Indexes `idx_forum_topics_category` + `idx_forum_topics_status` cover queries. No migration needed except optional CHECK extension if both pinned+locked needed (defer — use `status='pinned'` + separate logic or `status` enum expansion).

**Alternatives**: New `is_pinned/is_locked` booleans — rejected, conflicts with existing status enum and would require backfill.

## Decision 3: Metamod queue — reuse `forum_metamod_votes` + `forum_mod_grants`

**Decision**: Keep `forum-core` tables (`044_forum_metamoderation.sql:forum_metamod_votes`, `043_forum_moderation.sql:forum_mod_grants/actions`). Queue is `forum_mod_grants` WHERE not yet voted by caller. Add query params `?filter=unreviewed|reviewed&verdict=fair|unfair` and pagination. Enforce one verdict per (grant_id, reviewer_id) via PK.

**Rationale**: Tables + rolling unfair-rate logic already in `forum-core`. Frontend only needs filterable fetch and single-submit guard.

**Alternatives**: New queue table — rejected.

## Decision 4: Frontend read-state trigger

**Decision**: Mark read on topic detail mount (`onMount` in `TopicThread.svelte` / `board/[topicSlug].[topicId]/+page.svelte`) via `POST /api/forum/topics/{id}/read` with `last_read_post_id = last visible post id`. Debounce 1s, fire-and-forget, no blocking UX.

**Rationale**: Matches existing `POST .../read` contract; avoids double-write on list fetch. Idempotent upsert safe to retry.

**Alternatives**: Mark on list fetch — noisy, marks without reading.

## Decision 5: Archive skin parity

**Decision**: All badges use text labels (`Pinned`, `Locked`, `Unread`) + CSS squares, no emoji/glyphs. Archive header gating via existing `uiMode==='archive'` pattern.

**Rationale**: House rule no emoji; existing `ArchiveButton` + archive CSS pattern proven.

## Open Questions Resolved

- No NEEDS CLARIFICATION in spec; no research unknowns remain.
- Migration 070 only if index needed for `last_read_post_id` or status ordering — verify EXPLAIN, add only if p95 >200ms.

