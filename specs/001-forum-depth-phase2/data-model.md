# Data Model: Forum Depth Phase 2

**Feature**: 001-forum-depth-phase2 | **Source**: spec.md FR-001..013 + migrations 041/043/044

## Entities (existing — no new tables)

### ForumTopic (extends 041:forum_topics)

| Field | Type | Notes |
|-------|------|-------|
| id | BIGSERIAL PK | |
| category_id | BIGINT FK forum_categories | |
| author_id | INT4 FK users | |
| title | TEXT | |
| body | TEXT | OP markdown |
| status | TEXT CHECK open/locked/pinned/archived | **Pin/lock via status** |
| view_count | BIGINT | |
| last_post_id | BIGINT nullable | bumped on post |
| last_activity_at | TIMESTAMPTZ | ordering key |
| created_at/updated_at/deleted_at | TIMESTAMPTZ | soft delete |
| is_hidden | BOOL | fast-hide |
| search_vector | TSVECTOR | GIN |

**Ordering**: `ORDER BY (status='pinned') DESC, last_activity_at DESC` (FR-005).
**Validation**: status transitions only via curator/admin; `locked` rejects non-curator posts (FR-006).

### ForumReadState (041:forum_read_state)

| Field | Type | Notes |
|-------|------|-------|
| user_id | INT4 FK users PK | part of PK |
| topic_id | BIGINT FK forum_topics PK | part of PK |
| last_read_post_id | BIGINT nullable | cursor, null = never read |
| updated_at | TIMESTAMPTZ | `NOW()` on upsert |

**Constraints**: PK(user_id, topic_id) → one row per pair (FR-012). Upsert: `INSERT ... ON CONFLICT (user_id, topic_id) DO UPDATE SET last_read_post_id=EXCLUDED.last_read_post_id, updated_at=NOW()`.

**Derived**: unread = `last_read_post_id IS NULL OR last_read_post_id < forum_topics.last_post_id` (or compare to max visible post id). List query LEFT JOIN read_state for auth user.

### ForumPost (041:forum_posts)

Unchanged. Used to compute `last_read_post_id` and to enforce locked check before insert.

### Moderation / Metamod (043/044)

- `forum_mod_grants(id, post_id, granted_by, reason, created_at)` — action being audited
- `forum_metamod_votes(grant_id, reviewer_id, verdict fair/unfair, created_at PK(grant_id,reviewer_id))` — one verdict per reviewer (FR-009)
- Rolling unfair-rate computed from `forum_metamod_votes` per reviewer (existing forum-core logic, FR-010).

## Relationships

- User 1—* ForumReadState *—1 ForumTopic
- ForumTopic 1—* ForumPost
- ForumPost 1—* ForumModGrant 1—* ForumMetamodVote

## State Transitions

- Topic status: `open ↔ pinned ↔ locked` (curator only). `archived` out of scope.
- ReadState: `null → post_id` monotonically forward (re-read does not go backward; upsert overwrites with latest).
- MetamodVote: `∅ → {fair,unfair}` once per (grant, reviewer); no update.

## Validation Rules

- Read-state upsert requires auth; anonymous gets no row (FR-002).
- Pin/lock requires level ≥50/100; else 403.
- Locked topic: `INSERT forum_posts` rejected for non-curator (403 + message).
- Metamod duplicate verdict: 409 Conflict.

