# Feature Specification: Forum Depth Phase 2

**Feature Branch**: `001-forum-depth-phase2`
**Created**: 2026-08-24
**Status**: Draft
**Input**: User description: "C - P4" → Forum Depth Phase 2 per docs/NEXT-STEPS.md + docs/IMPLEMENTATION-PLAN-NEXT-STEPS.md + docs/brainstorm-02-roadmap-status.md §P4

## User Scenarios & Testing

### User Story 1 - Read-state badges on forum topics (Priority: P1)

A logged-in reader browses `/forum` or `/forum/{categorySlug}` and sees at a glance which topics have unread posts since their last visit. Opening a topic marks it read; returning to the list clears its unread badge. Anonymous users see no badges.

**Why this priority**: Restores core Threadlight promise (read-state) and drives return visits. Independent of moderation tooling.

**Independent Test**: As user A, post in topic T. As user B (who previously read T), reload forum list — T shows unread count/badge. Open T — badge clears on next list load.

**Acceptance Scenarios**:
1. **Given** user has read topic T to post N, **When** a new post N+1 appears, **Then** list shows unread indicator for T.
2. **Given** user opens T and scrolls to latest, **When** returning to `/forum`, **Then** T shows no unread indicator.
3. **Given** anonymous user, **When** viewing forum lists, **Then** no read-state indicators appear.

---

### User Story 2 - Pin / Lock affordances (Priority: P1)

Curators can pin a topic to top of its category and lock a topic to prevent new posts. Pins sort first; locked topics show lock badge and disable reply/new-post controls.

**Why this priority**: Minimum viable curation for announcements/rules threads; already implied by existing category/topic model.

**Independent Test**: As curator, pin topic T → it appears first in category listing regardless of bump time. Lock T → non-curator reply attempt blocked with explainable message.

**Acceptance Scenarios**:
1. **Given** curator on topic T, **When** toggling Pin, **Then** T appears pinned (with badge) atop its category list.
2. **Given** locked topic T, **When** non-curator tries to post, **Then** post is rejected and UI shows locked state.
3. **Given** locked topic T, **When** curator posts, **Then** post succeeds (curator override where policy allows, or blocked consistently — behavior documented).

---

### User Story 3 - Metamod queue polish (Priority: P2)

Curators triage moderation actions via `/forum/metamod` more efficiently: filterable queue, single-action audit view, and clear unfair-rate feedback. Existing `forum-core` metamod rolling unfair-rate + cooldowns remain authoritative.

**Why this priority**: Completes Threadlight F6–F7 loop; depends only on existing metamod tables.

**Independent Test**: Create 5 moderation actions, metamod 3 as fair/unfair, verify rolling unfair-rate updates and queue filters hide already-reviewed items.

**Acceptance Scenarios**:
1. **Given** mixed metamod queue, **When** filtering by "unreviewed", **Then** only unreviewed actions appear.
2. **Given** a moderation action, **When** opened, **Then** auditor sees anonymized context and can submit fair/unfair verdict once.
3. **Given** a curator with high unfair-rate, **When** viewing modlog, **Then** cooldown indicator reflects current rate per existing policy.

---

### User Story 4 - Mod-power user tools polish (Priority: P3)

Curators have concise per-user moderation context (recent grants, reasons, rate) accessible from topic/post actions, reducing context-switching.

**Why this priority**: Nice-to-have polish; no new permissions.

**Independent Test**: From a post's moderate menu, open user mod history drawer and verify recent moderation points + reasons visible.

**Acceptance Scenarios**:
1. **Given** user U with 2 prior moderation grants, **When** curator opens U's mod drawer, **Then** last N grants with reasons are listed.

### Edge Cases

- What happens when a topic has 0 posts visible to user (all deleted/hidden)? List still shows topic but read-state reflects visible tip.
- How does system handle pin overflow (many pinned topics)? Pinned section capped and paginated, then unpinned follow.
- What happens when two curators toggle pin/lock concurrently? Last write wins; UI re-fetches and shows current state.
- How does read-state handle deleted posts? Deletion does not retroactively mark topic unread; position tracks last read post id, not count.
- What happens when anonymous user later logs in? Read-state bootstraps from first visit (no historical backfill).
- How does metamod handle low-entropy queues (<3 items)? Queue still renders, filters show empty states, no division-by-zero in rate.

## Requirements

### Functional Requirements

- **FR-001**: System MUST persist per-user per-topic last-read position (topic_id + post_id or timestamp) and expose it to forum list/detail endpoints for authenticated users.
- **FR-002**: Forum list endpoints MUST return unread indicator (boolean or count) per topic for authenticated callers; anonymous callers receive no read-state.
- **FR-003**: Opening a topic MUST advance the caller's last-read position to the latest visible post (or explicit post id) without requiring a separate write call beyond the topic fetch.
- **FR-004**: System MUST support pin and lock booleans on forum topics, persisted and returned in topic list/detail responses.
- **FR-005**: Pinned topics MUST sort before unpinned topics within a category (pinned block first, then recency/bump order); locked state MUST be visible in list and detail.
- **FR-006**: Locked topics MUST reject new posts from non-curators (and enforce consistently for curators per documented policy); UI MUST disable/hide reply affordances when locked.
- **FR-007**: Pin/lock mutations MUST be restricted to curators/admins (level-gated) and audited via existing modlog/curator action log where applicable.
- **FR-008**: Metamod queue MUST be filterable (e.g., unreviewed/reviewed, fair/unfair) and paginated; already-reviewed items MUST not reappear as actionable.
- **FR-009**: Metamod detail MUST show anonymized moderation context and allow a single fair/unfair verdict per reviewer; duplicate verdicts MUST be rejected.
- **FR-010**: System MUST surface rolling unfair-rate and cooldown status derived from existing `forum-core` metamod tables; no new scoring formula.
- **FR-011**: Frontend MUST render archive-skin parity for all new badges/indicators (pin, lock, unread) and gate modern chrome off archive mode.
- **FR-012**: All new read-state writes MUST be idempotent and bounded (one row per user/topic, upsert).
- **FR-013**: Existing forum routes (`/forum/board/{slug}.{id}` canonical + legacy `/{categorySlug}/{topicId}`) MUST continue to work; pin/lock/read-state MUST not break topic slug resolution.

### Key Entities

- **ForumTopicReadState**: Per-user per-topic cursor (user_id, topic_id, last_read_post_id, updated_at). One row per pair, upsert.
- **ForumTopic**: Existing topic extended with `is_pinned` (bool), `is_locked` (bool), plus existing `topic_slug`, `category_slug`, `bumped_at`. Pin/lock are curator-mutable.
- **ModerationAction / MetamodVerdict**: Existing moderation grant + anonymized audit verdict; queue is a view over unreviewed actions with reviewer eligibility.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Logged-in user can identify unread topics from the forum list in under 5 seconds (badge visible without opening each topic).
- **SC-002**: Opening a topic clears its unread indicator on next list load in 100% of cases for that user.
- **SC-003**: 100% of pin toggles reflect in category ordering on next list fetch; pinned topics never intermix with unpinned by recency.
- **SC-004**: 100% of post attempts to locked topics by non-curators are rejected with an explainable locked message; no orphan posts created.
- **SC-005**: Curators can triage 20 metamod items with filters in under 3 minutes (queue correctly hides reviewed items, no duplicate verdict accepted).
- **SC-006**: Archive skin shows pin/lock/unread indicators with equal information as modern skin, verified by visual QA on both skins.

## Assumptions

- `forum-core` crate remains the authoritative engine for read-state, moderation, and metamod scoring; this feature adds persistence/UI, not new scoring.
- Auth is existing session/JWT; anonymous = no read-state, no pin/lock mutation.
- Level gate for pin/lock is curator (level ≥10) and admin (≥100) per existing gates; not a new role.
- Read-state tracks post id, not char offset; compatible with soft-deleted/hidden posts.
- No email/push notifications added in this slice; follow notifications remain out of scope.
- Deployment follows `CARGO_TARGET_DIR=/media/... cargo build` → `psql` migrations → `/tmp` frontend build → `rsync --delete` → curl sweep → push.

## Out of Scope

- New recommendation/leveling logic, invite cohorts, private DMs, walled-garden app, spendable currency.
- Full-text search changes or body search overhaul (P3).
- Uptime probe / backup cron (already done).
- Extension platform v3.1 theme/recipe sharing (separate spec).

## Dependencies

- Existing tables: `forum_topics`, `forum_posts`, `forum_categories`, `forum_topic_read_state` (to be created if missing), moderation/metamod tables in `forum-core`.
- Frontend: SvelteKit 5 runes, `TopicThread.svelte`, forum list pages, `forum.ts` API client, `page.test.ts` (not `+page.test.ts`).
- House rules: no emoji in UI text; `anyhow` error handling, no `unwrap()` in prod, `ArchiveButton` for archive CTAs.

