# Research: NodeBB 4.15.1 → Rust Port Inventory & Architecture Decisions

**Status**: Phase 0 output — open questions resolved
**Date**: 2026-08-31
**Reference**: NodeBB source at `~/code/js/NodeBB` (read-only, GPL-3.0 reference only; re-implementation is MIT)

---

## 1. NodeBB Source Inventory → Rust Mapping

| NodeBB Module (`src/`) | Purpose | Key Files | Rust Mapping (forum_core + routes) |
|------------------------|---------|-----------|-------------------------------------|
| **categories/** | Category CRUD, topics listing, recent replies, watch | `create.js`, `data.js`, `index.js`, `topics.js`, `watch.js`, `recentreplies.js` | `forum_core::model::Category`, `src/routes/forum.rs` (categories + topics list), `src/routes/forum_categories.rs` (admin CRUD) |
| **topics/** | Topic CRUD, posts, tags, scheduled, events, merge/fork, follow, bookmarks, unread | `create.js`, `data.js`, `index.js`, `posts.js`, `tags.js`, `scheduled.js`, `events.js`, `merge.js`, `fork.js`, `follow.js`, `bookmarks.js`, `unread.js`, `tools.js` | `forum_core::model::Topic`, `forum_core::model::Post`, `src/routes/forum.rs` (topics/posts), `src/routes/forum_polls.rs` (polls/events), `src/routes/forum_tags.rs` |
| **posts/** | Post CRUD, edit/delete, votes, uploads, quotes, diffs, bookmarks, parse | `create.js`, `edit.js`, `delete.js`, `index.js`, `votes.js`, `uploads.js`, `parse.js`, `diffs.js`, `bookmarks.js`, `tools.js` | `forum_core::model::Post`, `forum_core::model::Vote`, `src/routes/forum.rs` (posts), `src/routes/forum_uploads.rs` |
| **groups/** | Group CRUD, membership, invites, ownership, cover, posts, search | `create.js`, `data.js`, `index.js`, `membership.js`, `invite.js`, `join.js`, `leave.js`, `ownership.js`, `posts.js`, `search.js`, `cover.js`, `update.js` | `forum_groups`, `forum_group_members` tables, `src/routes/forum_groups.rs` |
| **privileges/** | Per-category per-group privileges (read/write/reply/moderate), admin overrides | `categories.js`, `global.js`, `helpers.js`, `posts.js`, `topics.js`, `users.js`, `admin.js` | `forum_privileges` table, `src/routes/forum_privileges.rs`, trust gates in `src/services/trust.rs` |
| **flags** (file) | Reports/flags, weights, queue, auto-hide, resolution | `flags.js` (31.8K) | `forum_flags` + `user_reports` extension, `src/routes/forum_flags.rs`, `flag_weight()` in trust.rs |
| **messaging/** | DM rooms, group rooms, pins, uploads, unread, notifications | `rooms.js`, `data.js`, `create.js`, `index.js`, `pins.js`, `unread.js`, `notifications.js` | `forum_rooms`, `forum_room_members`, `forum_messages`, `user_blocks`, `src/routes/forum_messaging.rs` |
| **notifications** (file) | Notification types, creation, digests, emailer | `notifications.js` (19.8K), `emailer.js`, `digest.js` (in user/) | `forum_notifications` table, `src/routes/forum_notifications.rs`, `forum_scheduled_promote.rs` cron |
| **search.js** | Full-text search (topics, posts, users, groups, tags) | `search.js` (11.8K), category/topics/posts search files | `src/routes/forum_search.rs`, Postgres `tsvector` + GIN indexes, `search_vector` triggers |
| **polls** (in topics/events) | Poll creation, options, votes, scheduled close | `events.js` (in topics/) | `forum_polls`, `forum_poll_options`, `forum_poll_votes`, `src/routes/forum_polls.rs` |
| **rewards/** | Badges/achievements (automated criteria) | `admin.js`, `index.js` | Deferred — native `reputation_events` + trust cover; explicit rewards skipped v1 |
| **uploads/** (posts/uploads.js) | Image upload, resize, storage | `uploads.js` (in posts/), `image.js`, `file.js` | `forum_uploads` table, `src/routes/forum_uploads.rs`, `image` crate resize |
| **socket.io/** | Real-time: posts, typing, presence, notifications, flags | `index.js`, `posts/`, `topics/`, `categories/`, `user/`, `notifications.js`, `modules.js` | `/ws/forum` via `axum::extract::ws`, Redis pubsub fanout (`src/services/forum_pubsub.rs`), SSE fallback |
| **user/** | Profile, blocks, follows, bans, uploads, settings, digest, approval | `data.js`, `profile.js`, `blocks.js`, `follow.js`, `bans.js`, `settings.js`, `digest.js`, `approval.js` | Existing `users` table + `forum_blocks`, `forum_follows` (users), `forum_bans`, `forum_uploads` |
| **events.js** | Global event emitter (NodeBB hook system) | `events.js` (6.3K) | Replaced by `ReputationHook` trait + direct function calls; no plugin registry |

---

## 2. Trust vs Reputation Divergence (Critical Design Decision)

| Axis | NodeBB (Karma/Reputation) | ficnexus (Trust + Reputation) |
|------|---------------------------|-------------------------------|
| **Name** | "Reputation" (single number, unbounded) | **Trust** (0-6, moderation axis) + **Reputation** (leaderboard currency) |
| **Purpose** | Gate all write actions, unlock privileges | Trust = moderation safety gate; Reputation = leaderboard only |
| **Earned by** | Posts, upvotes, accepted answers | Trust: reading + community behavior (auto to TL4); Reputation: forum activity via pluggable hook |
| **Lost by** | Downvotes, flags | Trust: only TL3+ spam revocation (monotonic to TL4); Reputation: never lost (only capped daily) |
| **Gate for flags** | Karma threshold | **TL2+** (`flag_weight(2)=1`, TL0/1 cannot flag) |
| **Gate for publish** | Karma threshold | **TL2+** (`PUBLISH_MIN_TRUST=2`) |
| **Gate for resolve** | Karma + mod status | **TL5+** (`RESOLVE_MIN_TRUST=5`) |
| **Daily cap** | None | **Forum rep daily cap**: `FORUM_REPUTATION_FORUM_DAILY_CAP` (default 20) — excess writes succeed, award 0 rep |
| **Leaderboard** | Karma ranking | `users.reputation` + `reputation_events` (ficnexus-wide) |

**Implementation**:
- `forum_core` exports `ReputationHook` trait (default noop for standalone)
- Embedded: calls `db::queries::update_reputation_and_promote` with `source="forum"`, `amount` from action table
- Redis key for daily cap: `forum:rep:{uid}:{day}` (INCR with TTL 86400)

---

## 3. WebSocket Transport: Axum WS + Redis Pubsub

**Decision**: `axum::extract::ws` (native, no socket.io compatibility layer)

| Concern | Choice | Rationale |
|---------|--------|-----------|
| **Protocol** | Custom JSON frames over WS | Simpler than socket.io binary; SvelteKit client is greenfield |
| **Fanout** | Redis pubsub (channels: `forum:topic:{id}`, `forum:category:{id}`, `forum:room:{id}`, `forum:notify:{uid}`) | Horizontal scale; single-process `broadcast` fallback for dev |
| **Events** | `new_post`, `edit_post`, `delete_post`, `typing_start`, `typing_stop`, `presence_join`, `presence_leave`, `notification`, `unread_update`, `poll_vote`, `flag_update` | Covers NodeBB parity checklist |
| **Auth** | JWT in `Sec-WebSocket-Protocol` or query param; validated on upgrade | Reuses ficnexus `AuthUser` extractor logic |
| **Reconnect** | Client sends `last_seen_post_id` per topic; server replays missed `new_post` events | Cursor-based catch-up (matches REST `?after=` pagination) |
| **Fallback** | SSE endpoint `/api/forum/events` (same Redis channels) | Mobile PWA + proxies that block WS |

**Redis Keyspace** (see data-model.md §Redis):
- `forum:presence:{uid}` → Hash `{topic_id: ts, ...}` (TTL 300s)
- `forum:typing:{topic_id}:{uid}` → String (TTL 3s, refreshed on keystroke)
- `forum:rep:{uid}:{YYYYMMDD}` → Int (daily rep cap counter, TTL 86400)
- `forum:rate:{action}:{uid}` → Int (per-user rate windows, TTL per action)

---

## 4. Markdown Pipeline: marked + dompurify (Client) + ammonia (Server)

| Layer | Library | Config |
|-------|---------|--------|
| **Client render** | `marked` (v14+) + `dompurify` (v3+) | `marked.setOptions({ gfm: true, breaks: true })`; `DOMPurify.sanitize(html, { ADD_TAGS: ['fic-card'], ADD_ATTR: ['data-fic-id'] })` |
| **Server sanitize (stored HTML optional)** | `ammonia` (Rust) | `ammonia::Builder::default().tags(allowed).attributes(allowed).build().clean(&html)` — only if storing rendered HTML (v1: store raw markdown only) |
| **Mentions** | Client: `@username` → link + WS `mention` event; Server: parse mentions on write, create `forum_notifications` | Regex `@([a-zA-Z0-9_-]{3,30})` against `users.username` |
| **Fic cards** | `/fic {url_id}` shortcode → `{type:"work",id:N}` payload in JSONB | Server validates work exists; client renders card component |
| **Quotes** | `quote_of` FK → server fetches quoted post excerpt (first 200 chars) for render | Client shows expandable quote block with link to original |

---

## 5. PWA Strategy: Hand-Rolled `sw.js` (No Workbox)

| Requirement | Implementation |
|-------------|----------------|
| **Offline cache** | `sw.js` caches: `/forum`, `/forum/*` (HTML), `/api/forum/categories`, `/api/forum/topics?category=*`, `/static/*` (CSS/JS/fonts), drafts (IndexedDB sync) |
| **Strategy** | Stale-while-revalidate for GET; queue POST/PATCH/DELETE in IndexedDB → replay on `online` event |
| **Drafts** | `forum_drafts` table (server) + IndexedDB mirror (client); auto-save every 5s while composing |
| **Push** | VAPID (web-push crate); subscription stored in `forum_push_subscriptions`; payload = notification JSON |
| **Manifest** | `/manifest.webmanifest` generated at build (name, icons, start_url=/forum, display=standalone) |
| **Update flow** | `skipWaiting` + `clients.claim`; toast "New version available — refresh" |

**Why not Workbox**: Zero deps, full control over cache keys (forum content is dynamic), smaller bundle, easier to debug in NodeBB-parity testing.

---

## 6. Open Questions Resolved

| Question | Resolution |
|----------|------------|
| NodeBB plugin hooks → Rust? | **No plugin registry**. Native features only. `ReputationHook` is the only extension point (seam documented). |
| Elasticsearch for search? | **No**. Postgres `tsvector` + GIN (title weight 'A', body 'D') per spec NFR-003. |
| S3 uploads v1? | **No**. Local disk `/public/uploads/forum/{yyyy}/{mm}/{uuid}.webp` (NFR). |
| Exp/levels in forum? | **Scrapped**. Trust 0-6 only moderation axis; forum awards rep via hook. |
| socket.io compatibility? | **No**. Custom WS protocol; SSE fallback. |
| Migration strategy? | Additive only (`072_…` onwards). `forum_core` migrations replayed verbatim into FicHub sequence. |

---

## 7. File Size Targets

- `research.md`: ~350 lines (this file)
- `data-model.md`: ~600 lines
- `quickstart.md`: ~250 lines
- Each `contracts/*.md`: ~400 lines (10 files = ~4000 lines total)
- All files < 8K tokens each