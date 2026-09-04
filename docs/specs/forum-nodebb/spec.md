# Feature Specification: Forum NodeBB Port — Full Rust+Svelte+Postgres Replacement

**Feature Branch**: `forum-nodebb`
**Created**: 2026-08-31
**Status**: Draft
**Input**: User description: "Fully featured forum on ficnexus — git clone NodeBB to ~/code/js/NodeBB and port all functionality to Rust+Svelte+Postgres to replace current forum, integrate seamlessly into site (AO3 theme) but preserve NodeBB design and PWA feel on phone. One big port, 1:1 parity, single ficnexus account, AO3 colour skin over NodeBB structure, WebSocket realtime, trust levels 0-6, flags only L2+, max 3 subagents, main thread for shared context."

## Overview

Replace the current `forum_core` + `frontend/src/routes/forum/*` implementation with a complete port of NodeBB 4.15.1 (GPL-3.0 reference, MIT re-implementation) into the ficnexus stack: Rust (Axum + sqlx + Postgres) + SvelteKit 5 + Postgres + Redis (pubsub) + existing ficnexus auth/trust/reputation/i18n. NodeBB at `~/code/js/NodeBB` (NFS-safe rsync from /tmp clone, read-only reference; `.git` with `core.fsyncObjectFiles false`) is the functional inventory. No NodeJS runtime in production. Theming: AO3 palette/tokens/header/footer over NodeBB information architecture — desktop AO3 chrome, mobile NodeBB bottom-nav/composer/PWA feel. Forum activity (posts, topics, helpful moderation) awards **reputation** that feeds the ficnexus leaderboard (`users.reputation` + `reputation_events`); there is no exp/level system — trust level (0-6) remains the only moderation axis.

---

## User Scenarios & Testing

### User Story 1 — Browse categories, topics, and posts (Priority: P1)

Reader browses `/forum`, opens a category, paginates topics, opens a topic and paginates posts. Works anonymously for read; write requires login. Unread badges, pin/lock state, tags, view counts visible.

**Why P1**: Core forum loop; everything else depends on it.

**Independent Test**: Anonymous user can list categories → topics → posts without login. Logged-in user sees unread indicators (from `forum_read_state`).

**Acceptance Scenarios**:
1. **Given** anonymous visitor, **When** opening `/forum`, **Then** visible categories + topic lists render with correct sort (pinned first, then bump time) and pagination.
2. **Given** topic with 100 posts, **When** opening `/forum/board/{slug}.{id}`, **Then** posts paginate (20/page default) with jump-to-last-unread when logged in.
3. **Given** logged-in user who has read to post 10, **When** 3 new posts arrive, **Then** category list shows unread count 3 for that topic.

---

### User Story 2 — Create topics and reply (Priority: P1)

Logged-in member composes a topic (title + body markdown + optional tags/category + optional fic card `/works {id}` + optional poll + optional scheduled publish) or replies (body + optional quote + uploads + mentions). Rate-limited by trust level; TL0 sandboxed (no topic create).

**Why P1**: Write path is half the product.

**Independent Test**: L1 user creates topic → appears in category; L1 replies → appears in topic; TL0 attempt rejected with upgrade hint.

**Acceptance Scenarios**:
1. **Given** L1 user, **When** posting topic with markdown + upload, **Then** topic created, search_vector updated, poster auto-follows topic, watchers notified.
2. **Given** user quotes post 5, **When** replying, **Then** reply stores `quote_of` and renders quoted excerpt with link.
3. **Given** TL0 user, **When** trying to create topic, **Then** 403 with "Requires Basic (L1) — keep reading to rise."

---

### User Story 3 — Mentions, notifications, and email digests (Priority: P1)

@mention of a user in a post creates an in-app notification (and email if user opted in + digest batch). Notifications list/page + bell badge + WebSocket push. Digest email cron (daily/weekly) batches unread mentions/replies/follows.

**Why P1**: NodeBB notification parity; drives return visits.

**Independent Test**: User A @mentions B → B gets notification within 2s via WS, badge increments, email queued if enabled.

**Acceptance Scenarios**:
1. **Given** post containing `@alice`, **When** saved, **Then** alice gets `mention` notification; WS pushes; bell count increments.
2. **Given** user watches category, **When** new topic appears, **Then** watcher gets `new_topic` notification.
3. **Given** user with daily digest enabled, **When** cron runs, **Then** email contains batched notifications since last digest (no duplicates).

---

### User Story 4 — Search + tags + bookmarks/follows (Priority: P1)

Full-text search over topics+posts (Postgres tsvector, title-weighted) with snippets, filters by category/tag/author/date. Topic tags (freeform, admin-mergeable). Per-topic bookmarks, follows (watching/tracking/muted), per-category watch levels.

**Why P1**: Discovery; NodeBB parity.

**Independent Test**: Create topic "Hello world" with tag "hp" → search "hello" returns topic with `<mark>` snippet; tag page `/forum/tags/hp` lists it.

**Acceptance Scenarios**:
1. **Given** topics with body "Dark Harry", **When** searching `q=dark harry`, **Then** results ranked title>body, snippets highlighted, paginated.
2. **Given** user follows topic, **When** new reply arrives, **Then** follower gets notification per watch level (watching=notif+unread, tracking=unread only, muted=none).
3. **Given** topic has tags `a,b`, **When** admin merges `b→a`, **Then** topics retagged and tag page reflects merge.

---

### User Story 5 — Real-time: live posts, typing, presence, unread (Priority: P1)

Topic page streams new posts, typing indicators, online presence, and unread counts via Axum WebSocket (Redis pubsub fanout; SSE fallback). Works on mobile PWA.

**Why P1**: NodeBB defining feature; required for 1:1 parity.

**Independent Test**: Two browsers on same topic: A types → B sees typing; A posts → B sees post appended without reload; unread badge updates via WS.

**Acceptance Scenarios**:
1. **Given** two users on topic T, **When** user A starts typing, **Then** user B sees "A is typing…" within 500ms; clears 3s after stop.
2. **Given** user posts to watched topic, **When** post saved, **Then** watchers' unread counts push via WS without refresh.
3. **Given** WS disconnects, **When** client reconnects, **Then** missed posts fetched via `?after={lastPostId}` cursor catch-up.

---

### User Story 6 — Groups, privileges, and moderation (Priority: P1)

Groups (open/closed/hidden/invite-only, per-group ownership) gate privileges per category (read/write/reply/moderate). Flags/reports (TL2+ only) queue to group moderators; reputation-weighted flag power per trust. Bans/timeouts scoped to category or global, expiring. Admin can shadow globally if needed.

**Why P1**: NodeBB groups+privileges+flags parity; ficnexus trust gates replace NodeBB karma.

**Independent Test**: TL1 tries to flag → 403; TL2 flags post → appears in mod queue; TL5 resolves → `auto_status` updated, weight applied.

**Acceptance Scenarios**:
1. **Given** TL2 user, **When** flagging post with reason, **Then** report created with `weight=flag_weight(TL)`, `auto_status=pending`, queued for TL5+ moderators.
2. **Given** group mod, **When** resolving report (hide/dismiss), **Then** post `is_hidden` toggled, modlog entry created, metamod-auditable.
3. **Given** banned user (category scope), **When** trying to post in that category, **Then** 403 scoped; other categories still writable.

---

### User Story 7 — Messaging (chat/DM + rooms) (Priority: P1)

1:1 DMs and group rooms, real-time via same WS, unread per room, block list (`uid:<id>:blocker_uids` parity), pins, uploads, message edit window (15min), room invites.

**Why P1**: NodeBB `src/messaging` parity.

**Independent Test**: A DMs B → B gets WS notification + unread; B blocks A → A can no longer message B.

**Acceptance Scenarios**:
1. **Given** A and B not blocked, **When** A sends DM, **Then** room created or reused, message stored, B gets WS `new_message` + notification.
2. **Given** A blocked by B, **When** A tries to message B, **Then** 403 "User has blocked you."
3. **Given** room with 3 participants, **When** one leaves, **Then** room persists for others; rejoin by invite only.

---

### User Story 8 — Polls, scheduled topics, events, awards (Priority: P2)

Polls attached to OP (single/multiple choice, timed close, vote change allowed until close). Scheduled publish (future `created_at`, invisible until due, cron promotes). Events calendar feed per category. Awards/rewards (badges on post/topic milestones, e.g. 100 upvotes).

**Why P2**: High-value NodeBB extensions; completes parity.

**Independent Test**: Create poll with 3 options → votes counted, result bar rendered, WS live update.

**Acceptance Scenarios**:
1. **Given** poll topic, **When** voting, **Then** vote stored once per user, counts broadcast via WS, change allowed until poll closes.
2. **Given** scheduled topic for +1h, **When** listing category before due, **Then** not visible; after cron promotes, appears and notifies watchers.
3. **Given** award rule "100 upvotes → badge", **When** threshold hit, **Then** award granted and displayed on post/topic.

---

### User Story 9 — Uploads, emoji, markdown, and editor (Priority: P2)

Composer: markdown (CommonMark + NodeBB-compatible extensions), live preview, @mention autocomplete, emoji picker, drag-drop uploads, fic card `/works {id}`, polls, drafts autosave (30s), quote affordance. Uploads stored `/public/uploads/forum/{yyyy}/{mm}/{uuid}` (local), 10MB max, images resized/compressed (same as `src/image.js` parity), avatars same bucket, max 512KB.

**Why P2**: Must match NodeBB composer UX; mobile composer bar required.

**Independent Test**: Compose with markdown `**bold**`, emoji `:smile:`, upload image → preview renders, post saves with parsed HTML cached.

**Acceptance Scenarios**:
1. **Given** composer open, **When** typing `@ali`, **Then** autocomplete shows matching users (prefix search on `username`).
2. **Given** image 8MB dropped, **When** uploading, **Then** server resizes to max 1920px, converts to webp if beneficial, stores, returns URL.
3. **Given** draft typed, **When** 30s passes or page unloads, **Then** draft autosaved (localStorage + server `forum_drafts` backup) and restores on return.

---

### User Story 10 — Moderation queue polish + PWA + importer (Priority: P2)

Flags queue filterable (pending/needs_admin/auto_hidden), modlog parity page, `forum-core` moderation points (5/72h) + metamod (L4+ audit) retained alongside new TL-gated flags. PWA: installable, offline cache (categories + last-read topics + pending drafts), push via VAPID. Admin: importer stub for legacy NodeBB JSON dump.

**Why P2**: Close the loop on trust triage + mobile + migration.

**Independent Test**: Three TL2 flags on same post cross threshold → `auto_hidden` set, visible to TL5 queue, mod resolves → metamod samples action.

**Acceptance Scenarios**:
1. **Given** queue with 10 reports mixed statuses, **When** filtering `auto_status=needs_admin`, **Then** only those appear, paginated.
2. **Given** PWA installed offline, **When** opening cached category, **Then** last-fetched topics render from cache; writes queue until reconnect.
3. **Given** NodeBB JSON dump, **When** running importer CLI, **Then** categories/topics/posts/users mapped into ficnexus Postgres with preserved IDs where safe.

---

### Edge Cases

- Deleted/hidden posts: list still shows topic, read-state uses visible tip, moderator sees collapsed with "show hidden" affordance.
- Concurrent pin/lock: last write wins, WS broadcasts state, clients re-fetch.
- Anonymous read: no read-state, no WS auth, SSE public feed only.
- Trust demotion: only spam reports at TL3+ auto-demote per existing invariant; manual set via `trust_events`.
- Upload abuse: TL0 cannot upload; L1 limited to 2 uploads/post; L4+ 10 uploads/post.
- WS fanout scale: Redis pubsub per topic/category channel; single-process fallback when Redis unavailable (degraded, logged).
- Flag weight 0 (TL0) rejected at API boundary, not silently queued.
- Block parity: `uid:<id>:blocker_uids` stored as `user_blocks(blocker_id, blocked_id)` table, checked on messaging + mention notify.

---

## Standalone Crate + Seamless Integration

**Requirement**: The ported forum MUST be a perfect port of NodeBB on its own crate — it can run standalone (its own Postgres + Redis, its own Axum router) with full NodeBB parity (categories/topics/posts/tags, groups/privileges, flags, search, chat/rooms, polls/scheduled/events/rewards, uploads/emoji/markdown, notifications/digests, WS realtime, PWA, importer) — the *only* deliberate divergences from NodeBB are (1) the moderation system, which uses ficnexus trust levels 0-6 instead of NodeBB karma/reputation, and (2) reputation, which is awarded by the forum (topics/posts/helpful moderation) and feeds the ficnexus leaderboard via `users.reputation` / `reputation_events`. There is no exp/level system. Everything else is 1:1.

**Integration**: The same crate MUST integrate seamlessly into ficnexus to replace the current forum: single `users` table / single ficnexus account (no separate forum users), ficnexus auth (`AuthUser`, `users.id` i32), trust gates (see Trust Mapping below), reputation writes via `db::queries::update_reputation_and_promote`, shared Postgres + Redis, mounted under ficnexus Axum router at `/forum` and `/api/forum`, SvelteKit routes replace `frontend/src/routes/forum/*`, AO3 skin applied at integration layer (tokens/header/footer), migrations additive to ficnexus `migrations/` (never fork `001_initial.sql`), and existing `forum_*` data migratable. When embedded, the forum's own auth/karma code is not used — ficnexus auth+trust+reputation is authoritative.

**Standalone vs embedded**: Standalone is still worth it, but now via a pluggable reputation hook: the forum crate emits `ForumEvent { TopicCreated, PostCreated, HelpfulModeration }` (or a `ReputationHook` trait) that the standalone dev boot wires to an in-memory/noop collector, while the embedded mount wires to `update_reputation_and_promote`. This preserves a runnable crate without coupling it to ficnexus leaderboards, at the cost of one trait indirection per write path.

---

## Requirements

### Functional Requirements

**Browse & structure**
- **FR-001**: List categories with topic/post counts, last-post teaser, unread badges (when authed), sorted by `position`; respects category `is_mod_only` / group read privilege.
- **FR-002**: List topics per category cursor-paginated (`?cursor=&limit=`), pinned first then `last_activity_at` desc; hidden/deleted filtered per viewer.
- **FR-003**: Topic detail returns OP + posts cursor-paginated (`?after={postId}`), with view_count increment (deduped per session/IP window), `topic_slug` immutable `{slugified-title}-{id}`.
- **FR-004**: Anonymous read open for all non-hidden content; write requires `AuthUser` (ficnexus account).

**Write**
- **FR-005**: Create topic `{title, category_slug, body, payload?, tags[], poll?, scheduled_at?}` — TL1+ only; TL0 403 with upgrade hint. Auto-follow for author.
- **FR-006**: Reply `{body, payload?, quote_of?, uploads[]}` — TL1+; quote renders excerpt. Edit window 15min for author, unlimited for TL5+/curator.
- **FR-007**: Soft-delete topics/posts (author or TL5+/mod); `deleted_at` + `is_hidden` flags; modlog entry.
- **FR-008**: Pin/lock topic (TL5+/mod); pinned sorts first; locked rejects non-mod writes.

**Tags, search, bookmarks, follows**
- **FR-009**: Topic tags freeform, admin-mergeable; tag pages `/forum/tags/{tag}` and per-topic tag display.
- **FR-010**: FTS over topics+posts via Postgres `search_vector` (title-weighted), snippets with `<mark>`, filters `category/tag/author/date`.
- **FR-011**: Bookmarks per post (toggle), follows per topic (watching/tracking/muted), per-category watch levels (watching/tracking/muted/ignored).

**Groups & privileges**
- **FR-012**: Groups CRUD (open/closed/hidden/invite-only), membership, ownership, join/leave/invite flows.
- **FR-013**: Privileges per category per group (`read`, `write`, `reply`, `moderate`) — checked on every read/write.
- **FR-014**: Privilege-gated category visibility and write affordances; admin can view all.

**Trust-gated moderation (only divergence from NodeBB)**
- **FR-015**: Flags/reports require TL2+ (`flag_weight` 0 for TL0 rejected 403); weight by trust: TL1/2=1, TL3=2, TL4=3, TL5/6=5.
- **FR-016**: Reports triage `weight` + `auto_status` (pending/auto_hidden/needs_admin/resolved); thresholds auto-hide, TL5+ queue resolves.
- **FR-017**: Keep `forum-core` moderation points (5/72h) + metamod (L4+ audit) alongside flags; sampled actions anonymized.
- **FR-018**: Scoped bans/timeouts (category or global, `expires_at` nullable = ban vs timeout); write routes check scope; read never blocked.

**Messaging**
- **FR-019**: 1:1 DMs + group rooms, real-time WS, unread per room, pins, uploads, edit window 15min, `user_blocks` checked.
- **FR-020**: Block list parity `uid:<id>:blocker_uids` as `user_blocks(blocker_id, blocked_id)`; blocks checked on messaging + mention notify.

**Polls, scheduled, events, rewards**
- **FR-021**: Polls on OP (single/multiple, timed close, vote-change until close), WS live counts.
- **FR-022**: Scheduled publish (future `created_at`, hidden until cron promotes, then notifies watchers).
- **FR-023**: Events per category (calendar feed), rewards/awards (badge rules on milestones).

**Editor, uploads, markdown**
- **FR-024**: Composer markdown CommonMark + NodeBB extensions, live preview, @mention autocomplete (prefix on `username`), emoji picker, drag-drop uploads, fic card `/works {id}`, polls, quote affordance.
- **FR-025**: Uploads `/public/uploads/forum/{yyyy}/{mm}/{uuid}`, 10MB max per file, images resized max 1920px + webp, avatars 512KB; TL0 no uploads, L1 2/post, L4+ 10/post.
- **FR-026**: Draft autosave 30s (localStorage + server `forum_drafts` backup), restores on return.

**Realtime**
- **FR-027**: WS at `/ws/forum` (Axum `extract::ws`), Redis pubsub per `forum:topic:{id}` / `forum:category:{id}` channel; single-process fallback when Redis down (logged degraded).
- **FR-028**: Events: `new_post`, `post_edited`, `post_deleted`, `typing_start/stop`, `presence`, `unread_counts`, `notification`, `poll_vote`, `topic_pinned/locked`. Client catch-up via `?after={lastPostId}`.
- **FR-029**: SSE fallback at `/api/forum/stream` for read-only feeds; anonymous may subscribe to public channels only.

**Notifications & digests**
- **FR-030**: Notifications for mention/reply/new_topic/follow/poll_close/room_message; WS push + bell badge + `/forum/notifications` page; mark-read per id and all.
- **FR-031**: Email via ficnexus mailer (not NodeBB emailer); digest cron daily/weekly batches since last digest, no duplicates, respects user prefs.

**Reputation & leaderboard**
- **FR-038**: Topic create (TL1+) awards `FORUM_REPUTATION_TOPIC_CREATE` rep (default 2) via `update_reputation_and_promote(forum_topic_create)`; visible in leaderboard (`GET /api/leaderboard/*` live over `users.reputation`).
- **FR-039**: Reply (TL1+) awards `FORUM_REPUTATION_POST_CREATE` rep (default 1); daily cap `FORUM_REPUTATION_FORUM_DAILY_CAP` (default 20) per user per day across forum rep sources; positive-only (no rep for deletes/flags).
- **FR-040**: Helpful moderation (resolution surviving L4+ metamod as fair) awards `FORUM_REPUTATION_MOD_HELPFUL` rep (default 3); abused moderation never awards.
- **FR-041**: Reputation hook is pluggable: trait `ReputationHook { async fn award(rep_event_type, delta) }` — embedded mount uses live ficnexus DB (`reputation_events` + `users.reputation`), standalone dev boot uses noop/in-memory collector so crate still boots without ficnexus.

**PWA & theming**
- **FR-032**: PWA installable (manifest + service worker), offline cache categories + last-read topics + pending drafts; writes queue until reconnect; VAPID push for forum notifications.
- **FR-033**: AO3 skin over NodeBB structure: ficnexus header/footer/palette/tokens; NodeBB topic-list/cards/composer/feed layout preserved; mobile bottom-nav + composer bar (NodeBB PWA feel).

**Routing & compat**
- **FR-034**: Keep ficnexus routes `/forum`, `/forum/{categorySlug}`, `/forum/board/{slug}.{id}` canonical + legacy `/{categorySlug}/{topicId}` via 301; add NodeBB-compat aliases `/category/{id}/{slug}` `/topic/{id}/{slug}` `/user/{slug}` 301 to canonical.

**Crate & integration**
- **FR-035**: Standalone crate (own router + migrations + config) that mounts into ficnexus at `/forum` + `/api/forum`; same Postgres/Redis when embedded; no NodeJS in prod.
- **FR-036**: Single `users` table, ficnexus `AuthUser` authoritative; no separate forum users; existing `forum_*` data migratable (IDs preserved where safe).
- **FR-037**: Trust mapping authoritative — no NodeBB karma/reputation code runs when embedded.

### Key Entities

- **ForumCategory** (`forum_categories`): id, slug, title, description, position, `is_mod_only`, `privileges` (via group), `created_at`; sortable, searchable.
- **ForumTopic** (`forum_topics`): id, category_id, author_id, title, body, payload(jsonb), tags, poll_id, status, `is_pinned/is_locked/is_hidden`, view_count, `topic_slug` immutable `{slug}-{id}`, `scheduled_at`, `last_post_id/last_activity_at`, `search_vector`, timestamps.
- **ForumPost** (`forum_posts`): id, topic_id, author_id, body, payload, `quote_of`, `is_hidden`, `edited_at/deleted_at`, `search_vector`, `created_at`.
- **ForumGroup** + **ForumGroupMember** + **ForumPrivilege**: group(id, name, slug, visibility open/closed/hidden/invite-only, owner_id), member(group_id, user_id, role), privilege(category_id, group_id, can_read/write/reply/moderate).
- **ForumTag** + **ForumTopicTag**: tag(name, count), topic_tag(topic_id, tag).
- **ForumVote / Bookmark / Follow / WatchState**: vote(post_id,user_id,value), bookmark(post_id,user_id), follow(topic_id,user_id,state watching/tracking/muted), category_watch(category_id,user_id,level).
- **ForumPoll** + **ForumPollOption** + **ForumPollVote**: poll(topic_id, question, type single/multi, closes_at), option(poll_id, text), vote(poll_id, option_id, user_id).
- **ForumReadState**: (user_id, topic_id, last_read_post_id, updated_at) upsert, one row per pair.
- **UserReport** (extended): existing `user_reports` + `weight`, `auto_status`, `resolved_by/at`; TL2+ gate, flag_weight by trust.
- **ForumBan**: (id, user_id, category_id NULL=global, scope, reason, banned_by, expires_at, created_at).
- **Messaging**: room(id, type dm/group, title), room_member(room_id,user_id), message(id,room_id,author_id,body,payload,is_hidden,edited_at), room_pin, `user_blocks(blocker_id, blocked_id)`.
- **ForumDraft**: (user_id, topic_id NULL=topic-draft, body, payload, updated_at) — server backup for autosave.
- **ForumUpload**: (id, uploader_id, topic_id/post_id nullable, path, mime, bytes, width/height, created_at) on `/public/uploads/forum/…`.
- **Notification**: (id, user_id, type, actor_id, topic_id/post_id/room_id, payload, is_read, created_at) + digest watermark.

### Trust Mapping (ficnexus 0-6; trust is moderation, reputation is leaderboard — only divergences from NodeBB)

| Level | Name | Write | Flag weight | Notes |
|-------|------|-------|-------------|-------|
| 0 | New | read+bookmarks only, no topic/reply, no uploads, no flags | 0 (rejected) | sandboxed |
| 1 | Basic | reply + topic create | 1 | core write |
| 2 | Member | publish tags, flag | 1 | **min to flag** (FR-015) |
| 3 | Regular | larger weight | 2 |  |
| 4 | Elder | + metamod audit | 3 | audit `forum-core` actions |
| 5 | Community Moderator | resolve reports, hide, 5/72h points | 5 | human layer |
| 6 | Near-admin | all except bans/billing | 5 | staff-designated |

Constants: `PUBLISH_MIN_TRUST=2`, `RESOLVE_MIN_TRUST=5`, `flag_weight()` as in `src/services/trust.rs`; TL0/1 can draft privately where applicable; auto-promotion to TL4 monotonic, TL3+ spam demotion only automatic demotion; TL5/6 via admin digest. **No exp/levels** — the scrapped progression system is not used.

### Reputation & Leaderboard Coupling (ficnexus `users.reputation` + `reputation_events`)

Forum activity awards reputation that counts toward the ficnexus leaderboard (live queries `get_weekly_leaderboard`/`get_monthly_leaderboard` over `users.reputation`). Write paths call `db::queries::update_reputation_and_promote` with an event type and delta, which also handles auto-promotion checks:

- `forum_topic_create`: +N rep (config `FORUM_REPUTATION_TOPIC_CREATE`, default 2) — on successful topic create (TL1+).
- `forum_post_create`: +N rep (config `FORUM_REPUTATION_POST_CREATE`, default 1) — on reply (TL1+).
- `forum_mod_helpful`: +N rep (config `FORUM_REPUTATION_MOD_HELPFUL`, default 3) — when a moderator's resolution is deemed helpful / survives metamod (L4+ audit fair).
- Daily cap: `FORUM_REPUTATION_FORUM_DAILY_CAP` (default 20) applies to forum-sourced reputation per user per day — prevents grinding via spam topics; positive-only (never deduct forum rep for downvotes/flags — moderation weight is trust, not rep).
- When embedded, the forum's reputation hook is the live ficnexus `update_reputation_and_promote`; standalone dev boot wires a noop/in-memory hook so the crate still runs without ficnexus DB.

### Non-Functional

- **NFR-001**: WS fanout via Redis pubsub; single-process fallback when Redis down (degraded, logged), no data loss — catch-up via cursor.
- **NFR-002**: No NodeJS runtime in production; NodeBB GPL code is reference only, re-implemented MIT.
- **NFR-003**: Postgres FTS (no Elasticsearch) for search parity; Redis only for pubsub + presence + rate-limit windows.
- **NFR-004**: Mobile parity: bottom nav + composer bar, PWA installable, 60fps scroll on topic lists (virtualized if needed).

---

## Success Criteria

### Measurable Outcomes

- **SC-001**: Anonymous user can browse categories→topics→posts with pagination and correct pin/lock/tag rendering.
- **SC-002**: L1 user can create topic + reply with markdown/mention/upload/quote and see it live via WS on second client without reload; `users.reputation` incremented and visible on `/leaderboard` after refresh.
- **SC-003**: TL1 cannot flag (403); TL2 flag appears in TL5 queue with correct weight; resolution hides or dismisses with modlog + WS broadcast; helpful moderation awards rep (FR-040) counted once.
- **SC-004**: Two users on same topic see typing indicator within 500ms and new posts appended via WS; reconnect catch-up returns missed `?after=` posts.
- **SC-005**: Search `q` returns ranked results with `<mark>` snippets, filtered by category/tag/author/date, paginated.
- **SC-006**: DM + group room: messages real-time, blocks enforced, pins/uploads/edit window respected, unread per room correct.
- **SC-007**: Poll votes counted once, changable until close, WS live bars; scheduled topic hidden until cron promotes.
- **SC-008**: Upload 10MB image resizes to ≤1920px + webp, stored correctly, served; TL0 upload rejected.
- **SC-009**: PWA installs, caches categories + last-read topics + drafts offline, queues writes until reconnect, VAPID push delivers notifications.
- **SC-010**: Standalone crate boots with own router + pluggable reputation hook (noop in dev) and passes same browse/write/realtime checks; embedded mount replaces current forum with no orphan routes and forum rep appears on ficnexus leaderboard.
- **SC-011**: Daily rep cap enforced: creating 30 topics in one day yields at most `FORUM_REPUTATION_FORUM_DAILY_CAP` (default 20) forum rep for that user; excess writes succeed but award 0 rep.

---

## Assumptions

- NodeBB 4.15.1 at `~/code/js/NodeBB` is the parity inventory (GPL reference, not shipped). PRD inventory covers `src/categories|topics|posts|groups|privileges|flags|user|messaging|notifications|search|plugins(captured as native)`.
- Single ficnexus `users` table; ficnexus `AuthUser` + `services::trust` + `db::queries::update_reputation_and_promote` authoritative when embedded; standalone uses `ReputationHook` noop.
- Realtime: Axum WebSocket (`/ws/forum`) + Redis pubsub + SSE fallback; polling not used for parity.
- Uploads: local disk `/public/uploads/forum/{yyyy}/{mm}/{uuid}` (no S3 v1); FicHub mailer for digests; VAPID for push.
- AO3 skin = ficnexus design tokens/header/footer over NodeBB layout; not a NodeBB theme port.
- Migrations additive after `071_trust_preferences.sql` (next is `072_…`); existing `forum_*` rows + `reputation_events` preserved.
- One big port (no phased milestones for parity), executed by ≤3 subagents at a time, main thread for shared context only.
- No exp/levels — scrapped; trust is moderation, reputation is leaderboard; forum awards rep via pluggable hook, daily cap prevents grinding.

---

## NodeBB Parity Checklist (must reach 100% before "done")

Browse: categories tree, topics pinned/locked/moved/forked/merged, posts with diffs, tags, teasers, thumbnails, pagination, unread, recent, sorted, suggested, scheduled. Groups: create/join/leave/invite/ownership, cover, search, membership. Privileges: per-category per-group read/write/reply/moderate, admin overrides. Flags: TL2+ reports, weights, queue, auto_hide/needs_admin thresholds, resolve. Bans: scoped+global, timeouts, expiries. User: blocks (`uid:<id>:blocker_uids`), follows, bookmarks, uploads, cover, settings. Search: FTS topics+posts+users+groups. Messaging: DMs, rooms, pins, uploads, unread, blocks. Polls, events, rewards, crossposts, attachments, notifications (mention/reply/new_topic/room), digests (NodeBB emailer → ficnexus mailer), API routes, WS `socket.io/*` → `/ws/forum` parity. PWA: manifest, offline cache, push. Importer: NodeBB JSON dump → Postgres. Explicitly out for v1: NodeBB plugin hook/filter registry (native features only, seam documented).

---

## References

- NodeBB source: `~/code/js/NodeBB` + `/tmp/NodeBB` (read-only, GPL-3.0)
- Current forum: `forum_core` crate + `frontend/src/routes/forum/*` + `docs/FORUM-API-CONTRACT.md` + `docs/SPEC-COMMUNITY-PLATFORM.md` v2
- Trust: `src/services/trust.rs` (TRUST_NAMES 0-6, flag_weight, PUBLISH_MIN_TRUST=2, RESOLVE_MIN_TRUST=5) + migrations `013_trust_levels.sql` / `071_trust_preferences.sql`
- Design tokens: `docs/frontend-design.md`
- Spec Kit: `.specify/templates/spec-template.md` + `.specify/specs/` (existing `001-forum-depth-phase2` etc.)

---
