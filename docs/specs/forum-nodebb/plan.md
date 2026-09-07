# Forum NodeBB parity — implementation plan (integration-first rewrite)

**Status:** PLANNING → Phase 4 rewrite (2026-09-07). Phase 3 DONE (migrations
072–079 recovered from live DDL, commit `f4cc60f`). Moderation pivot landed
(`708cfed`: trust ladder runs moderation, metamod answers 410). This rewrite
changes the Phase 4 architecture: **notifications, the flag/report queue,
blocks, uploads, and drafts are site-wide primitives — the forum consumes
them instead of duplicating them.** Uncommitted WIP (flood control +
`FORUM_MIN_POST_LEN`) must land or stash before starting.
**Supersedes:** `plan.v1-2026-09-07.md.bak` (kept for reference).
**Spec dir:** `/home/alvaro/code/rust/ficnexus/docs/specs/forum-nodebb/`
**Read order:** this file → §1 principles → §3 lanes → `data-model.md` → `contracts/`
**Author:** Cline, 2026-09-07. Every anchor re-verified against the codebase today.

## 0. Ground truth (verified 2026-09-07 — trust these over any doc)

**Site primitives that already exist (the reason for this rewrite):**

| Fact | Anchor |
|---|---|
| Site notifications ship: `GET /api/notifications` + `/api/notifications/unread-count`, table `notifications(user_id, notification_type, title, body, link, reference_type, reference_id, is_read)` | `src/routes/notifications.rs`, `src/server.rs:1002-1006`, `migrations/001_initial.sql` |
| Frontend notification inbox + nav entry already exist — no new UI needed | `frontend/src/routes/notifications/+page.svelte`, `frontend/src/routes/+layout.svelte:265` |
| Canonical producer pattern for site notifications (INSERT with link/reference fields) | `src/db/queries/social.rs:101` |
| `forum.rs` ALREADY writes one site notification (row :144) — precedent exists | `src/routes/forum.rs:144` |
| Site report queue is polymorphic and **already whitelists `forum_post` + `forum_topic`**; triage + resolve + weight columns all live | `src/routes/reports.rs:75,134,149,390`; `user_reports` extended by `078_forum_topics_posts_extend.sql`; consumed by trust metrics `src/routes/trust.rs:113` |
| Site upload route is work/epub-specific (multipart pattern exists, but no generic image service) | `src/routes/upload.rs:62,209,213`; `ls src/services/` has no image/media module |
| No site DMs exist — messaging tables are forum-prefixed but unused by any Rust code | grep-verified; DDL fully specified in `076_forum_messaging.sql` |

**Forum state (unchanged from v1 plan):**

| Fact | Anchor |
|---|---|
| Crate `crates/forum-core` (workspace member), own migrations via `sqlx::migrate!("./migrations")` | `crates/forum-core/src/store/pg.rs:45-49` |
| Migrations 072–079 applied; **next free number 090+** (080 is a permanent gap); **Phase 4 needs zero new DDL** | `ls migrations/` |
| 47 handlers in `forum.rs` (3686 lines) incl. tags, FTS, moderation; `082_forum_parity.sql` | `src/routes/forum.rs` |
| Messaging DDL: `forum_rooms` (name NULL for DMs / set for group rooms, `last_activity_at`), `forum_room_members` (`role owner/moderator/member`, `notify_level all/mentions/none`, `is_muted`), `forum_messages` (`body` markdown, `payload` JSONB, `search_vector`, `is_hidden`), `forum_user_blocks(blocker_id, blocked_id)` | `migrations/076_forum_messaging.sql` |
| `forum_drafts` (:5), `forum_uploads` (:20), `forum_notifications` (:42, `scheduled_at` for future publish, static-predicate partial index) | `migrations/077_forum_drafts_uploads_notifications.sql` |
| Trust gates: `assert_min_trust`, `flag_weight`, `PUBLISH_MIN_TRUST=2`, `RESOLVE_MIN_TRUST=5` | `src/services/trust.rs:39-127` |
| Admin gate pattern: `FORUM_ADMIN_LEVEL` env, default 100 | `src/routes/forum.rs:69-71,359` |


## 1. The pivot: forum as consumer of site primitives

v1 planned parallel forum universes (`forum_notifications` with its own UI, a
forum-only flag queue, forum-only uploads). The codebase already ships the
site-wide versions. New rule, per feature:

| Feature | v1 plan | This plan |
|---|---|---|
| Notifications | Lane E: `forum_notifications` table + `/api/forum/notifications` + forum inbox + bell | **Use site `notifications` + `/api/notifications` + existing bell.** Forum events (reply, mention, poll end) INSERT rows via the `social.rs:101` pattern. |
| Flags/reports | Lane D: forum flag queue routes + UI | **Use site `POST /api/reports`** (already accepts `forum_post`/`forum_topic`). Forum UI gets a Report button; queue stays site-level. |
| DMs | Lane C: "forum messaging" | **Site-wide DMs.** Tables stay (076, generic enough), routes at `/api/messages`, frontend at `/messages`, reachable from profiles. |
| Uploads | Lane F: forum image upload | **Site-wide `/api/uploads`** (images for posts, later avatars/works). `forum_uploads` serves as the table (§6 rename decision). |
| Drafts | forum-only drafts | **Site-wide drafts** keyed by context (`forum_topic`, `forum_post`, later `comment`…), served at `/api/drafts`. |
| Polls / Groups | forum-only | Stay forum-scoped; poll-close/group events notify via site notifications; group rooms later reuse messaging. |
| Realtime (Phase 5) | forum WS | Site-wide WS `/ws` with channels `topic:{id}`, `room:{id}`, `user:{id}` — DMs, notifications, forum all fan out through it. |

**Why:** two inboxes and two report queues would split moderator attention
and double the trust-metric surface (trust.rs already consumes
`user_reports`). Integration was the explicit direction.

**Scope consequence:** v1 Lanes D & E shrink to wiring + buttons; freed
effort moves into messaging (site DMs) and uploads (site service).

## 2. Phase roadmap (rewritten)

| Phase | Scope | Status |
|---|---|---|
| 1–2 | research + contracts (v1) | done in v1 — `contracts/*.md` authoritative **except** `forum-flags-moderation.md` (superseded by Lane 2) and `forum-messaging.md` route paths (now `/api/messages/*`) |
| 3 | migrations 072–079 recovered | ✅ DONE (`f4cc60f`) — **Phase 4 needs zero new DDL** |
| **4** | **Lanes 1–7 below (integration-first)** | ← current |
| 5 | site-wide realtime: WS `/ws` + Redis pubsub + SSE fallback | after 4 |
| 6 | PWA + theming (v1 scope) | after 5 |
| 7 | NodeBB cutover via importer (v1 scope) | last |

## 3. Phase 4 lanes (each = one PR-sized task)

### Lane 1 — Site-wide drafts + composer integration (start here; smallest)

`forum_drafts` exists (077:5). No DDL change — read 077:5-18 for the exact
key columns before writing queries.

1. `src/routes/drafts.rs` (NEW): `GET /api/drafts?context=` (own list),
   `PUT /api/drafts/{context}/{ref}` (upsert),
   `DELETE /api/drafts/{context}/{ref}`. Auth via `AuthUser`; user_id in
   every WHERE.
2. Composer (`frontend/src/lib/forum/composer.ts`): autosave every 30s + on
   blur; restore prompt on mount. Contexts: `forum_topic`, `forum_post`.
3. Tests: axum-test route handlers; composer autosave vitest.

### Lane 2 — Flags: wire forum into the site queue (replaces v1 Lane D)

Nothing to build server-side (verified: reports.rs:75 whitelist, :149
triage, :390 resolve; weights via trust.rs:47; consumed by trust.rs:113).

1. Frontend Report button on topic + post actions → site report dialog with
   `target_type` prefilled (find the existing dialog: grep `target_type`
   in frontend/src — comments already report).
2. Owner notification on report: check what comments do today and match
   exactly — do not invent a new notification kind.
3. Mod surface: ensure `forum_post`/`forum_topic` rows resolve to deep
   links (`link` from reference_id); add resolution if missing.
4. Tests: e2e — TL1 reports a post → TL5 sees + resolves it in the queue.
5. Mark `forum-flags-moderation.md` superseded by this lane.


### Lane 3 — Site-wide DMs (replaces v1 Lane C; biggest new surface)

DDL from 076 is final. Routes are site-level, sibling of notifications.rs:

1. `src/routes/messages.rs` (NEW):
   - `GET /api/messages/rooms` — mine, ordered by `last_activity_at DESC`
   - `POST /api/messages/rooms` `{user_id}` → find-or-create DM room
     (`forum_rooms.name IS NULL`, `is_group FALSE`, two
     `forum_room_members` rows); rely on a unique check, handle the race
   - `GET /api/messages/rooms/{id}/messages?cursor=` — member-only
     (non-member → 404 per repo convention; check how forum.rs treats
     missing vs forbidden and match)
   - `POST /api/messages/rooms/{id}/messages` — enforce
     `forum_user_blocks` both directions + the flood gate (reuse the WIP
     flood control once it lands; do not duplicate a second limiter)
   - `PATCH /api/messages/rooms/{id}` — mute / `notify_level` / leave
   - Group rooms (`is_group TRUE`): defer to Lane 3b after Lane 5 groups.
2. Notifications: on new message, INSERT site `notifications` row (type
   `message`, link `/messages/{roomId}`) for members whose `notify_level`
   allows — social.rs:101 pattern. No email in this lane.
3. Frontend `/messages` (NEW top-level route, NOT under `/forum`): room
   list + thread + composer. Profile "Message" button →
   `/messages?to={userId}` (grep `frontend/src/routes/people/` for the
   profile page actions row).
4. Blocks UI: `/forum/blocks/` already exists — relocate to
   `/settings/blocks` or redirect; do not duplicate (§6.3).
5. Tests: find-or-create race, block enforcement, `notify_level` filtering,
   cursor pagination.

### Lane 4 — Site-wide uploads (generalizes v1 Lane F)

`forum_uploads` (077:20) becomes the generic table (name kept, §6.2).

1. `src/routes/uploads.rs` (NEW): `POST /api/uploads` (multipart; PNG/
   JPEG/WebP/GIF; 10 MiB cap), static `GET /uploads/{id}`, `DELETE
   /api/uploads/{id}` (owner or staff). Files under
   `{config.cache_dir}/uploads/{yy}/{mm}/{id}.{ext}` (cache_dir pattern:
   upload.rs:213).
2. Processing: `image` crate — cap 2000px, strip metadata, 400px thumb.
   Verify `image` is already in `Cargo.toml` before adding.
3. Trust gate: `assert_min_trust(db, user, PUBLISH_MIN_TRUST)` on POST
   (trust.rs:91, constant trust.rs:39).
4. Forum composer: paste/drag-drop → `/api/uploads` → `![](url)` insert.
5. Tests: multipart round-trip, size/type rejection, ownership on DELETE.


### Lane 5 — Groups + privileges (v1 Lane A, unchanged)

Forum-scoped; nothing site-wide here. Follow the full step list in v1 §3
Lane A (`plan.v1-2026-09-07.md.bak`): handlers + `/api/forum/groups` CRUD +
membership, per-category × group privilege checks on topic/post creation,
admin gate via `FORUM_ADMIN_LEVEL` (forum.rs:69-71).

### Lane 6 — Polls (v1 Lane B, unchanged)

Forum-scoped. Follow v1 §3 Lane B: `forum_polls`/`forum_poll_options`/
`forum_poll_votes` handlers + vote endpoints + reusable PollBar component.
One change vs v1: **poll-close notifications go through the site
notification producer** (Lane 1 pattern), never `forum_notifications`.

### Lane 7 — Scheduled topics + NodeBB importer (v1 Lane G, unchanged)

Follow v1 §3 Lane G: cron binary for `scheduled_at` (077:55 static-predicate
partial index) + `crates/forum-import` CLI. Importer must map NodeBB
notification history → site `notifications` rows (type `forum_import`) so
imported users wake up with a unified inbox, and skip writing
`forum_notifications` entirely.

## 4. Verification (per lane + final)

1. Per lane: `cargo check -p fichub` while iterating; `just test` +
   `just clippy` before marking done; `cargo sqlx prepare` after query
   changes (live DB — same flow as v1 §4).
2. curl each new endpoint: `{err:0,…}` envelope, 400-on-auth per repo
   convention.
3. Frontend: `just test-frontend` + `just test-e2e`.
4. Pivot-specific integration checks:
   - forum reply → site bell badge increments
     (`/api/notifications/unread-count` consumer already in the layout).
   - report a forum post from the topic page → appears in the site queue;
     TL5 resolves it.
   - DM from a profile → both users see the thread at `/messages`; a
     blocked user cannot send; muted member gets no notification row.
   - paste an image in the composer → `/api/uploads` stores it → renders
     in markdown preview.
5. Tick lanes off in §2/§3 and update the status line as they land.

## 5. Out of scope (v1 §5 plus pivot additions)

- No new migrations in Phase 4 (090+ reserved; Phase 5 WS may need one for
  site-level tables if any).
- Email digests for messages/notifications (mailer.rs exists; wiring later).
- Group rooms inside messaging (Lane 3b, after Lane 5).
- NodeBB widget/theme parity; reputation/gamification UI (v1 §5).

## 6. Legacy + decision log for the pivot

1. `forum_notifications` (077:42): **do not reference from new code** —
   site `notifications` is canonical. Do not drop the table yet; file a
   Phase 5+ follow-up migration to drop it once zero code paths write it.
2. `forum_uploads` keeps its name in Phase 4 (a rename is DDL churn with
   no behavior gain); data-model.md must note it is the generic uploads
   table. Revisit during Phase 6.
3. `forum_user_blocks` is the site-wide block list; `/settings/blocks` (or
   a redirect from the existing `/forum/blocks/`) is its UI. If works/
   comments need blocks later, reuse this table — never create
   `user_blocks` alongside it.
4. v1 lane-letter map for reading `plan.v1-2026-09-07.md.bak` and the
   contracts: A→5, B→6, C→3, D→2, E→dropped (§1 row 1), F→4, G→7.
   Contracts `forum-messaging.md` (route paths) and
   `forum-flags-moderation.md` (whole file) are superseded by §3 Lanes 3
   and 2 respectively; all other contracts remain authoritative.

## 7. Changelog

- 2026-09-07: **Integration-first rewrite** per user direction
  (notifications, messaging, flag queue, uploads, drafts = site-wide, not
  forum-specific). Verified before writing: site notifications ship
  (`routes/notifications.rs`, `server.rs:1002-1006`, frontend bell
  `+layout.svelte:265`, producer `db/queries/social.rs:101`, and
  `forum.rs:144` already writes a site notification); the site report
  queue already whitelists `forum_post`/`forum_topic` (`reports.rs:75`)
  with triage (:149), resolve (:390), and trust-metric consumption
  (`trust.rs:113`) — so v1 Lanes D & E duplicated shipped systems and were
  replaced by wiring lanes (2 and §1). DMs generalized to `/api/messages`
  + `/messages` (Lane 3); drafts/uploads generalized over the 077 tables
  (Lanes 1, 4). Phase 4 requires zero new DDL. v1 plan archived at
  `plan.v1-2026-09-07.md.bak`; lane-letter map in §6.4.

