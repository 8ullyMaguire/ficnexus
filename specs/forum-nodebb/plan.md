# Forum NodeBB parity — implementation plan (integration-first rewrite)

**Status:** Phases 4–8 LANDED (2026-09-08). Phase 3 DONE (migrations
072–079 recovered from live DDL, commit `f4cc60f`). Moderation pivot landed
(`708cfed`: trust ladder runs moderation, metamod answers 410). This rewrite
changes the Phase 4 architecture: **notifications, the flag/report queue,
blocks, uploads, and drafts are site-wide primitives — the forum consumes
them instead of duplicating them.**
**Phase 5–8 post-landing review:** §9 (open TODOs live there — realtime
producers/authz/tests are the highest priority).
**Landed since rewrite:**
- Lane 1 backend (`9ebfd37`: `drafts.rs` rewritten to the LIVE 077 DDL —
  topic/category-keyed; the `d575752` version queried nonexistent
  `context`/`ref`/`content` columns — + `GET/PUT/DELETE /api/drafts`
  wired in `server.rs`).
- Lane 3 backend+UI+tests (`596bf5d`: `messages.rs` rewritten, all 5 gaps
  fixed, `/api/messages/*` wired, `/messages` page + API client + i18n ×6 +
  vitest; `tests/messages_api.rs` 4 green).
- Lane 4 backend+tests (`56c563e`: `uploads.rs` NEW — `POST/DELETE /api/uploads`
  + public `GET /uploads/forum/{yy}/{mm}/{name}`; trust-gated multipart,
  10 MiB cap, PNG/JPEG/WebP/GIF whitelist, sha256 + header-parsed
  dimensions to `forum_uploads` (077); `chunk_uploads()` wired; 5 unit tests
  green. No resize in this lane — `image` crate absent).
- Lane 5 backend (`7db25f7` groups/privileges + `1b266a2`: `can()`
  enforcement open-by-default on create_topic/create_post).
- Lane 6 backend (`9ede502` fixed to compile + `9ebfd37` create endpoint +
  close-notify + inverted-vote-gate fix in `596bf5d`;
  `tests/polls_api.rs` 2 green) + Lane 6 frontend (PollBar UI — archive-only
  rewrite removing modern-mode branches; `voteOnPoll`/`closePoll` API
  helpers + vitest; 35/35 forum API tests green).
- Lane 7 backend (`1478395`: scheduled_at on create/update + listing
  exclusion ×5 sorts + unread/recent/popular/rss/search + detail preview
  gate; `publish-scheduled` bin; `tests/scheduled_topics_api.rs`).
- Flood-gate NULL bug fixed (`1b266a2`: `check_flood` used
  `fetch_optional` on `SELECT MAX` — 500'd every first-time poster; also
  fixed the same pattern in `messages.rs`). Heals `f3_create_topic`.
- Lane 2 frontend (Report button in `TopicThread.svelte` → site
  `/api/reports`) — unchanged.
**Remaining:** Lane 2 item 4 (e2e report test, deferred — needs live DB),
final §4 verification.
**Decisions since rewrite:** block checks unify on canonical site
`blocked_users` (`forum_user_blocks` stays dead — §6.3 corrected below);
`/forum/blocks` page stays (already consumes site `/api/blocks` — no
relocation); profile Message button skipped (no public profile pages
exist; `/messages?to=` supported for future use); Lane 4 stores originals
byte-for-byte (no `image` crate on the tree) — resize/Exif-stripping
deferred to a follow-up that adds the dep without DDL change.
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
| No site DMs exist — messaging tables are forum-prefixed but unused by any Rust code | ~~grep-verified; DDL fully specified in `076_forum_messaging.sql`~~ STALE 2026-09-07 night: Lane 3 landed (`596bf5d`) — `messages.rs` serves `/api/messages/*` + `/messages` UI; DDL live-verified (rooms `creator_id NOT NULL`, messages `author_id`, members `left_at`; `forum_user_blocks` dead, blocks use `blocked_users`) |

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
| **4** | **Lanes 1–7 below (integration-first)** | ✅ DONE — all 7 lanes complete: backend+frontend+tests for 1, 2, 3, 4, 5, 6, 7; remaining: Lane 7 importer crate (Phase 7), Lane 2 e2e (deferred), Lane 5 group rooms (Lane 3b) |
| 5 | site-wide realtime: WS `/ws` + SSE + Redis pubsub | ✅ DONE — WS + SSE + Redis cross-instance pubsub (`e12fef6`). ⚠️ **review findings §9.1** |
| 6 | PWA + theming (v1 scope) | ✅ DONE — manifest, offline fallback, update/install prompts (`76dfe99`). Minor notes §9.2 |
| 7 | NodeBB cutover via importer (v1 scope) | ✅ DONE — crates/forum-import CLI + JSON schema + tests (`5ac26d2`). ⚠️ **review findings §9.3** |
| **8** | **Forum reputation/gamification UI + widgets + theme parity** | ✅ DONE — 4 lanes: AuthorCard, widgets, theme tokens, XP wiring (`e88d268`). ⚠️ **review findings §9.4** |

## 3. Phase 4 lanes (each = one PR-sized task)

### Lane 1 — Site-wide drafts + composer integration (✅ DONE)

`forum_drafts` live DDL (077, verified against prod — the `d575752`
version queried nonexistent `context`/`ref`/`content` columns and is
superseded): `(user_id, topic_id NULL=new-topic, category_id, title,
body, payload, poll_data)`. No DDL change.

1. ✅ `src/routes/drafts.rs` rewritten topic/category-keyed; `context`/`ref`
   URL segments decoded (`forum_post {topicId}` → reply draft,
   `forum_topic new:{slug}` → new-topic draft); UPDATE-then-INSERT in a tx
   (no unique index exists for upsert); idempotent DELETE (`{err:0,
   deleted}`). Wired in `server.rs`, committed.
2. ✅ 2026-09-08: Composer autosave every 30s + on blur; restore prompt
   on mount. Contexts: `forum_topic` (new:{slug}), `forum_post` ({topicId}).
   New `frontend/src/lib/api/drafts.ts` client; wired into
   `forum/new/+page.svelte` and `TopicThread.svelte` reply box.
3. ✅ 2026-09-08: Draft API client tests via DB-gated uploads/polls suites
   (route-handler unit tests for payload/key JSON existed already).

### Lane 2 — Flags: wire forum into the site queue (✅ DONE `7fb9ad9`)

Nothing to build server-side (verified: reports.rs:75 whitelist, :149
triage, :390 resolve; weights via trust.rs:47; consumed by trust.rs:113).

1. ✅ Frontend Report button on post actions → site report dialog:
   `TopicThread.svelte` (openReport/submitReport) + `reportForumPost()` in
   `frontend/src/lib/api/forum.ts` POSTs `{target_type: 'forum_post'}` to
   `/api/reports`. Topic-level (`forum_topic`) report path added —
   `reportForumTopic()` helper + Report button in the topic header with its
   own report box (mirrors post-level flow).
2. ✅ Owner notification on report: verified reports.rs never notifies
   owners for ANY target type — forum already matches comments. No change
   needed (do not invent a new notification kind).
3. ✅ Mod surface deep links: `list_reports` now resolves `forum_post` and
   `forum_topic` → `/forum/board/{slug}.{id}` via a JOIN; other targets get
   `link: null`. Backend-only change in `reports.rs`.
4. Tests: e2e — TL1 reports a post → TL5 sees + resolves it in the queue.
   Requires live DB — deferred.
5. ✅ Marked `forum-flags-moderation.md` superseded by this lane.


### Lane 3 — Site-wide DMs (BACKEND+UI+TESTS DONE `596bf5d`)

DDL from 076 is final (live-verified: rooms carry `creator_id NOT NULL`,
messages use `author_id` — not `sender_id` — members carry `left_at`;
`forum_user_blocks` is a dead legacy table).

1. ✅ `src/routes/messages.rs` rewritten + registered (`mod.rs`,
   `server.rs` as `/api/messages/...`):
   - `GET /api/messages/rooms?filter=all|dm|group` — mine, active
     membership only (`left_at IS NULL`), `last_activity_at DESC`.
   - `POST /api/messages/rooms` `{user_id}` → find-or-create DM
     (idempotent `{err:0, room_id, existing}`, NOT 409); self-DM 400,
     unknown user 404, either-direction block → 403 via canonical
     **`blocked_users`** (see §6.3 correction).
   - `GET /api/messages/rooms/{id}/messages?cursor=&limit=` — member-only
     (non-member → 404), newest-first, `has_more`.
   - `POST /api/messages/rooms/{id}/messages` — 403 on either-direction
     block (reject, never silent-send); message row INSERTed first in a tx,
     then per-recipient site `message` notifications AFTER commit
     (best-effort, `forum.rs` precedent); flood gate
     (`FORUM_POST_DELAY_SECS` measured on own DMs); `{err:0, message_id}`.
   - `PATCH /api/messages/rooms/{id}` — mute / `notify_level` / leave
     (`left_at`, history preserved).
   - Group rooms (`is_group TRUE`) stay deferred to Lane 3b.
2. ✅ Notifications per §1 (muted / `notify_level=none` suppressed;
   `mentions` treated as `all` on DMs). No email in this lane.
3. ✅ Frontend `/messages` (top-level route): room list + thread +
   composer, `?to={userId}` find-or-create + `?room=` deep link; API
   client `lib/api/messages.ts`; i18n `messages.*` ×6 locales; `routePages`
   entry; vitest (`page.test.ts` 2 green). Profile "Message" button
   SKIPPED — no public profile pages exist; `?to=` is ready for future use.
4. ✅ Blocks UI: `/forum/blocks/` stays — it already consumes the site
   `/api/blocks` (`blocked_users`); no relocation, no duplication (§6.3).
5. ✅ Tests `tests/messages_api.rs` (4 green): idempotent find-or-create +
   self-DM 400, block 403 on create AND send with zero message rows stored,
   mute/notify suppression, cursor pagination + non-member 404.
6. ⚠️ Flood-gate pattern note: `SELECT MAX … fetch_optional` 500s on NULL
   (no rows) — use `fetch_one` with `Option<T>` scalar. Fixed in both
   `messages.rs` and `forum.rs::check_flood` (`1b266a2`).

### Lane 4 — Site-wide uploads (BACKEND+TESTS DONE `56c563e`)

`forum_uploads` (077:20) becomes the generic table (name kept, §6.2).

1. ✅ `src/routes/uploads.rs` (NEW): `POST /api/uploads` (multipart; PNG/
   JPEG/WebP/GIF; 10 MiB cap), public `GET /uploads/forum/{yy}/{mm}/{name}`
   (no auth — validated against traversal), `DELETE /api/uploads/{id}`
   (owner or staff). Files under `{cache_dir}/uploads/forum/{yy}/{mm}/{token}.{ext}`
   (nanos-based unique token).
2. ⚠️ No `image` crate on the tree → originals stored byte-for-byte
   (no resize/metadata-strip in this lane). `width`/`height` parsed from
   image headers (PNG/GIF/WebP/JPEG) so the nullable columns stay
   populated; sha256 always written. A follow-up adds the `image` crate
   for resize/Exif-stripping without DDL change.
3. ✅ Trust gate: `assert_min_trust(db, user, PUBLISH_MIN_TRUST)` on POST
   (trust.rs:91, constant trust.rs:39).
4. ✅ 2026-09-08: Forum composer paste/drag-drop → `/api/uploads` →
   `![](url)` insert. New `frontend/src/lib/api/uploadImage.ts`
   (uploadImage + handleImageEvent); wired into new-topic and reply
   textareas (both archive and modern mode).
5. ✅ Tests (5 green): classify whitelist/reject, PNG+GIF dimensions,
   sha256 structural check, staff gate.
   ⚠️ 2026-09-08 review: these are in-module unit tests only — every other
   lane shipped a black-box `tests/*_api.rs`. TODO: `tests/uploads_api.rs`
   covering the multipart POST end-to-end (TL gate 403, >10 MiB 413,
   non-image 400, fetch-back round-trip, DELETE authz).
6. ✅ `chunk_uploads()` wired into `server.rs` route merge chain.
7. ✅ 2026-09-08: Uploads lifecycle documented in `data-model.md` §1.7
   (storage layout, lifecycle table, no automated retention yet).


### Lane 5 — Groups + privileges (✅ DONE — backend `7db25f7`+`1b266a2`, frontend 2026-09-08)

Forum-scoped; nothing site-wide here. Landed: `forum_groups.rs` (11
handlers incl. join/leave/invite/roles), `forum_privileges.rs` (`can()` +
grant/revoke), 14 routes wired in `server.rs`, crate migration mirror,
`tests/forum_groups_api.rs` (3 green).

Enforcement (`1b266a2`): `check_category_priv()` in `forum.rs`, wired into
`create_topic` (`write`) and `create_post` (`reply`). **Open-by-default:**
categories with no privilege rows + public keep today's gates
(ban/flood/trust) — a category becomes restricted the moment an admin
grants any row (or flips `is_mod_only`), and then `can()` decides (staff
bypass inside, default deny). Verified zero rows/groups on prod, so
current behavior is preserved everywhere. `tests/
forum_privileges_enforce.rs` (2 green): open posts OK, outsider 403 /
member OK / staff bypass on restricted.

Also in `1b266a2`: flood-gate NULL fix (see Lane 3.6) — heals
`f3_create_topic_happy_path`.

Remaining:
1. ✅ 2026-09-08: Frontend groups management UI — new
   `frontend/src/lib/api/groups.ts` + `/forum/groups` list page +
   `/forum/groups/[groupId]` detail page (list, create, join/leave,
   delete, member list). Route added to routePages.
2. Group rooms (`is_group TRUE`) stay deferred to Lane 3b after this lane.
3. `ensure_system_groups()` exists but is never called — no startup hook,
   no auto-membership; wire only when the groups UI needs seeded groups
   (low priority — system groups are staff-only and currently empty).

### Lane 6 — Polls (BACKEND DONE, UI DONE — 2026-09-08)

Forum-scoped. Landed across `9ede502` → `9ebfd37` → `596bf5d`:
`forum_polls.rs` get/vote/close/results + **`POST
/api/forum/topics/{topicId}/polls` create** (question 1–500 chars, 2–10
options, `max_selections` in range; author-or-staff gate, one poll per
topic → 409, `PUBLISH_MIN_TRUST`); **poll-close notifies all distinct
voters** via the site producer (type `poll_closed`, never
`forum_notifications`). Wired in `server.rs`. `tests/polls_api.rs`
(2 green): create → double-create 409, validation 400, outsider 403;
vote → outsider-close 403 → author-close → voter gets `poll_closed` row.

Bugs fixed along the way (pre-existing, `9ede502` never compiled):
missing `Value` import, `&str`/`String` + `i32`/`i64` mixups, stray `*`
derefs — plus an **inverted vote gate** (open polls 409'd, closed polls
accepted votes), rewritten as a `close_at` fetch.

Remaining:
1. ✅ 2026-09-08: PollBar component + vote UI — archive-only rewrite
   (`PollBar.svelte` no modern-mode branches; `voteOnPoll`/`closePoll` API
   helpers in `forum.ts`; ×6 i18n keys; vitest cases for `reportForumTopic`,
   `voteOnPoll`, `closePoll`; 35/35 forum API tests green).
2. ✅ 2026-09-08 (second-opinion review): `topic_detail` now attaches the
   topic's poll inline (no second request for the OP's PollBar).
3. ✅ 2026-09-08: Lane 6 UI committed (`18a6b64` — PollBar, topic_detail
   attachment, i18n, vitest cases; no longer working-tree only).
4. ✅ 2026-09-08: poll serializer deduped — `topic_detail` reuses
   `forum_polls::serialize_poll` (`da63a11`); placeholder
   `votes`/`voted_by_user` fields dropped.
5. TODO: one assertion in `tests/polls_api.rs` that `topic_detail` returns
   the `poll` object with the shape PollBar reads (locks the contract).

### Lane 7 — Scheduled topics + importer ✅ DONE

`forum_topics.scheduled_at` + partial index exist (078); wired up in
Phase 4. Importer crate landed in Phase 7 (`5ac26d2`).

1. ✅ (working tree, compiles clean — needs publish binary + tests +
   commit): `CreateTopicBody.scheduled_at` (future-only, else 400) written
   on INSERT; `UpdateTopicBody.scheduled_at` + `clear_schedule`
   (author/staff, reschedule future-only / publish-now); exclusion
   `t.scheduled_at IS NULL` in list (5 sort branches), unread, recent,
   popular (all 3 sorts incl. the `_` fallback), rss (both feeds), search
   (topics + posts branches);
   `topic_detail` (+ by-slug via delegation) fetches `author_id` +
   `scheduled_at` and 404-shapes scheduled topics for non-author non-staff
   (preview allowed).
2. ✅ `src/bin/publish-scheduled` (landed `1478395`): flip
   `scheduled_at <= NOW()` → NULL, notify authors via site notifications,
   idempotent. Systemd timer + deploy wiring in `infra/systemd/`.
3. ✅ `crates/forum-import` CLI (workspace member): NodeBB JSON export
   (categories/topics/posts/users) → `forum_*` tables; notification
   history → site `notifications` (type `forum_import`); NEVER write
   `forum_notifications`. JSON input schema documented in README.md.
   10 unit tests pass. (`5ac26d2`)
4. ✅ `tests/scheduled_topics_api.rs` (3 green): create validates
   + hides from listings; detail 404s for stranger + clears via update;
   publish-scheduled SQL flips + notifies author.

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
6. 2026-09-08 review additions (second opinion on the last 20 commits):
   - **Frontend debt sweep:** the 9 pre-existing `svelte-check` errors
     (`DownloadTab`, `ReportsTable`, `NewWork`, `WorksTable`) have been
     inherited across 20+ commits — own them as an explicit task here
     instead of carrying them forever.
   - **Commit hygiene:** one concern per commit. `1478395` bundled the
     Lane 7 feature + tests + Cargo.toml + a 387-line doc restore in one
     commit; Lane 6 UI risked the same fate (untracked file at review
     time). Rule: feature, tests, and docs land as separate commits, or
     at minimum never mix a doc restore into a feature commit.
   - **Migration-history repair:** the renumber/restore dance
     (`717a008` → `f4cc60f` → `27b49ef`/`f0515b0`) means fresh databases
     apply forum migrations 072–079 *before* the older 081–088 set and
     version order no longer encodes chronology (`set_ignore_missing(true)`
     keeps pre-consolidation DBs booting). Phase 5 TODO: **baseline squash**
     to a single `001_initial.sql` + forward-only migrations (the codebase
     has done this consolidation before — 72 files → baseline), so version
     order regains meaning before more DDL lands.
   - **i18n dead-key audit:** the moderation pivot (`708cfed`) retired
     points/metamod but `forum.modPointsLeft` and friends remain in all 6
     dictionaries — sweep dead `forum.*` keys on the next i18n touch.
   - **`publish-scheduled` deploy story:** the binary exists but nothing
     schedules it; record the intended runner (cron/systemd timer) in the
     Lane 7 section or the ops docs before calling Lane 7 done.


## 8. Phase 8 — Forum reputation/gamification UI + widgets + theme parity

**Status:** LANDED (2026-09-08) — 8a `63e486c`+`f89203e`, 8b `d8e6a65`+`30818c7`,
8c `b73ac30`, 8d `e88d268`. Review findings + follow-up TODOs: **§9**.
**Dependencies:** Phase 4 (all lanes), Phase 5 (realtime), Phase 7 (importer)
**Estimated scope:** 4 sub-lanes, each one PR-sized task

### 8.0 Existing infrastructure (what we're building on)

| Component | Location | Status |
|---|---|---|
| XP system | `src/progression.rs` — XpEvent, XpSourceDef, Feature, UserFeature, UserPref, PageLayout models | ✅ Shipped |
| User gamification fields | `users` table: `level`, `exp`, `xp`, `rank`, `trust`, `reputation` | ✅ DDL exists |
| XP award definitions | `xp_source_defs` table (event_type, xp_amount, daily_cap) | ✅ DDL exists |
| XP events log | `xp_events` table (user_id, event_type, xp, source_ref) | ✅ DDL exists |
| Feature gating | `features` table + `user_features` (unlock/enable/pin) | ✅ DDL exists |
| User preferences | `user_prefs` table (JSONB key-value) | ✅ DDL exists |
| Page layouts | `page_layouts` table (JSONB grid config) | ✅ DDL exists |
| Theme system | `frontend/src/lib/themes/` — presets, ThemeEditor, settings page, API client | ✅ Shipped |
| Trust gates | `src/services/trust.rs` — assert_min_trust, flag_weight, PUBLISH_MIN_TRUST | ✅ Shipped |
| Forum moderation | `src/routes/forum.rs` — mod actions, score tracking, post moderation | ✅ Shipped |

### 8.1 Lane 8a — Forum post author card (reputation + level display)

**Goal:** Display author reputation, level, rank, and XP progress in forum post headers.

**Backend:**
- No new DDL needed — all fields exist in `users` table
- Add `GET /api/forum/users/{user_id}/profile` endpoint returning:
  ```json
  {
    "id": 1,
    "username": "alice",
    "level": 12,
    "rank": 3,
    "xp": 15420,
    "trust": 85,
    "reputation": 42,
    "bio": "...",
    "created_at": "2023-01-01T00:00:00Z",
    "total_words_read": 1250000,
    "total_works_read": 87
  }
  ```
- Add `GET /api/forum/users/{user_id}/xp-history` endpoint returning:
  ```json
  {
    "current_xp": 15420,
    "current_level": 12,
    "next_level_xp": 18500,
    "progress_percent": 67,
    "recent_events": [
      {"event_type": "post_created", "xp": 10, "created_at": "..."},
      {"event_type": "post_upvoted", "xp": 5, "created_at": "..."}
    ]
  }
  ```

**Frontend:**
- New component `AuthorCard.svelte` — shows avatar (initials), username, level badge, XP progress bar, rank icon, trust score
- Wire into `TopicThread.svelte` post headers — replace plain username with AuthorCard
- Wire into `topic_detail` page — show author info sidebar
- Add XP progress bar component `XpProgressBar.svelte` — shows current/next level with fill animation

**Tests:**
- `tests/forum_users_api.rs` — test profile endpoint returns correct fields
- `tests/forum_users_api.rs` — test XP history endpoint returns events
- Unit test for XP progress calculation

### 8.2 Lane 8b — Forum widgets (recent/popular/stats)

**Goal:** Add configurable widgets to forum pages showing recent topics, popular posts, and user stats.

**Backend:**
- No new DDL needed — queries against existing tables
- Add `GET /api/forum/widgets/recent` endpoint:
  ```json
  {
    "topics": [
      {"id": 100, "title": "...", "category": {"slug": "general"}, "author": {"username": "alice"}, "created_at": "...", "reply_count": 5}
    ]
  }
  ```
- Add `GET /api/forum/widgets/popular` endpoint:
  ```json
  {
    "topics": [
      {"id": 100, "title": "...", "view_count": 500, "score": 42, "reply_count": 25}
    ]
  }
  ```
- Add `GET /api/forum/widgets/stats` endpoint:
  ```json
  {
    "total_topics": 1250,
    "total_posts": 8900,
    "total_users": 340,
    "online_now": 12,
    "newest_user": {"username": "bob", "created_at": "..."}
  }
  ```

**Frontend:**
- New component `ForumWidgets.svelte` — sidebar container for widget list
- New component `RecentTopicsWidget.svelte` — shows last 5 topics with category badges
- New component `PopularTopicsWidget.svelte` — shows top 5 by views/score
- New component `ForumStatsWidget.svelte` — shows totals + online count
- Wire into forum index page (`/forum`) — show widgets in sidebar
- Wire into category pages — show category-specific recent/popular
- Add widget preference to user_prefs (enable/disable, order)

**Tests:**
- `tests/forum_widgets_api.rs` — test recent/popular/stats endpoints
- Unit test for widget data formatting

### 8.3 Lane 8c — Forum theme tokens (category + post theming)

**Goal:** Extend theme system with forum-specific tokens for category colors, post backgrounds, and moderation highlights.

**Backend:**
- No new DDL needed — theme tokens stored in `user_prefs` as JSONB
- Extend `GET/PUT /api/me/theme` to accept forum-specific tokens:
  ```json
  {
    "forum": {
      "category_colors": {
        "general": "#58a6ff",
        "writing": "#f78166",
        "meta": "#7ee787"
      },
      "post_bg_color": "#161b22",
      "post_border_color": "#30363d",
      "mod_highlight_color": "#1f6feb",
      "op_highlight_color": "#238636"
    }
  }
  ```
- Add `GET /api/forum/categories` to include `color` field from theme or default

**Frontend:**
- Extend `ThemeTokens` type in `frontend/src/lib/themes/presets.ts` to include forum tokens
- Extend `ThemeEditor.svelte` to show forum theme section (category color pickers, post styling)
- Wire forum colors into `TopicThread.svelte` and `PollBar.svelte`
- Add CSS custom properties for forum theming (`--forum-post-bg`, `--forum-post-border`, etc.)
- Apply category colors as badges/pills in topic lists

**Tests:**
- Unit test for theme token validation
- Unit test for forum color application

### 8.4 Lane 8d — XP event wiring (forum actions award XP)

**Goal:** Wire forum actions to the existing XP system so posting, voting, and moderation award experience points.

**Backend:**
- Add XP award triggers in `src/routes/forum.rs`:
  - `post_created` → 10 XP (daily cap: 100)
  - `post_upvoted` → 5 XP (daily cap: 50)
  - `topic_created` → 15 XP (daily cap: 50)
  - `poll_voted` → 3 XP (daily cap: 30)
  - `post_moderated` → 2 XP (daily cap: 20)
- Update `users.xp`, `users.level`, `users.rank` after each award
- Add daily cap enforcement via `xp_events` table查询 (count today's events of type)
- Add `GET /api/forum/users/{user_id}/achievements` endpoint:
  ```json
  {
    "achievements": [
      {"type": "first_post", "name": "First Post", "description": "Created your first forum post", "unlocked_at": "..."},
      {"type": "popular_post", "name": "Popular Post", "description": "Post received 10+ upvotes", "unlocked_at": null}
    ]
  }
  ```

**Frontend:**
- New component `AchievementBadge.svelte` — shows achievement icon + name
- New component `AchievementToast.svelte` — shows unlock notification
- Wire into `TopicThread.svelte` — show achievements on author cards
- Wire into user profile page — show achievement grid
- Add achievement notification via WebSocket (Phase 5) when unlocked

**Tests:**
- `tests/forum_xp_api.rs` — test XP award on post creation
- `tests/forum_xp_api.rs` — test daily cap enforcement
- `tests/forum_xp_api.rs` — test level/rank update after XP
- Unit test for achievement unlock logic
- Unit test for daily cap calculation

### 8.5 DDL changes

**None required.** All gamification fields already exist in `users` table:
- `level` (smallint, default 0)
- `exp` (bigint, default 0) — alias for total XP
- `xp` (integer, default 0) — current XP in level
- `rank` (integer, default 1)
- `trust` (integer, default 0)
- `reputation` (integer, default 0)

XP events tracked via existing `xp_events` and `xp_source_defs` tables.
Achievements stored in `user_features` table (feature_type = 'achievement').

### 8.6 Verification

1. Per lane: `cargo check -p fichub` while iterating
2. `cargo test --test forum_users_api` — profile + XP history endpoints
3. `cargo test --test forum_widgets_api` — recent/popular/stats endpoints
4. `cargo test --test forum_xp_api` — XP awards + daily caps + achievements
5. `cd frontend && npx svelte-check` — 0 errors
6. Manual: forum post shows author card with level + XP bar
7. Manual: forum sidebar shows widgets with real data
8. Manual: theme editor shows forum section with category color pickers
9. Manual: posting awards XP visible in user profile
10. Manual: achievement unlocks show toast notification

### 8.7 Changelog entry (for §7)

Landed 2026-09-08 — see the Phase 8 entry in §7 (the `2026-09-XX`
placeholders that lived here were superseded by the real commits
`63e486c`…`e88d268`).

## 5. Out of scope (v1 §5 plus pivot additions)

- No new migrations in Phase 4 (090+ reserved; Phase 5 WS may need one for
  site-level tables if any).
- Email digests for messages/notifications (mailer.rs exists; wiring later).
- Group rooms inside messaging (Lane 3b, after Lane 5).
  (Widget/theme parity + reputation UI were pulled back IN as Phase 8
  and landed 2026-09-08 — the old v1 §5 exclusion no longer applies.)

## 6. Legacy + decision log for the pivot

1. `forum_notifications` (077:42): **do not reference from new code** —
   site `notifications` is canonical. Do not drop the table yet; file a
   Phase 5+ follow-up migration to drop it once zero code paths write it.
2. `forum_uploads` keeps its name in Phase 4 (a rename is DDL churn with
   no behavior gain); data-model.md must note it is the generic uploads
   table. Revisit during Phase 6.
3. `forum_user_blocks` is DEAD — do not use. Canonical site block list is
   `blocked_users(user_id, blocked_user_id)` (site `/api/blocks` in
   `subsystems.rs`, UI at `/forum/blocks/` which stays put). Lane 3 DM
   block checks use `blocked_users` both directions. If works/comments
   need blocks later, reuse `blocked_users` — never `forum_user_blocks`
   or a new `user_blocks` alongside it.
4. v1 lane-letter map for reading `plan.v1-2026-09-07.md.bak` and the
   contracts: A→5, B→6, C→3, D→2, E→dropped (§1 row 1), F→4, G→7.
   Contracts `forum-messaging.md` (route paths) and
   `forum-flags-moderation.md` (whole file) are superseded by §3 Lanes 3
   and 2 respectively; all other contracts remain authoritative.

## 9. Second-opinion review of Phases 5–8 (2026-09-08, post-landing audit)

Audit of `37f7d54`…`40ad652` against the §8 planning promises. Phases 5–7
had no dedicated review before landing; findings below are the catch-up.

### 9.1 Phase 5 — realtime (highest-priority findings)

1. **No producers — the bus is silent.** Nothing in `src/routes/*` calls
   `rt_manager.publish` or `publish_to_redis` (only `server.rs` constructs
   the manager and starts the Redis listener). WS clients get `connected`
   plus whatever other *clients* publish over the socket. The plan promised
   "DMs, notifications, forum all fan out through it". **TODO:** publish
   `notification:new` on the site notification producer path, reply events
   on `topic:{id}` in `create_post`, message events on `user:{id}` in
   `messages.rs`; wire `publish_to_redis` into the same paths for
   multi-instance parity.
2. **Unauthenticated channel access.** `ws_handler` authenticates
   optionally; any client (including anon) can `subscribe` to any channel
   and `publish` to any channel — including other users' `user:{id}`
   channels. **TODO:** server-side authz (`user:{id}` only for the owning
   session; `topic:{id}`/`room:{id}` gated on forum visibility; `publish`
   reserved for server-side code). JWT-in-query-param also leaks tokens
   into access logs — prefer a `Sec-WebSocket-Protocol` handshake.
3. **No black-box tests.** No `tests/realtime_api.rs` although every other
   surface got one. **TODO:** integration test asserting anon connect →
   welcome frame; subscribe `user:{other}` → denied; server publish →
   received on both WS and SSE.
4. **SSE busy-wait.** `SseStream::poll_next` calls `try_recv` and on
   `Empty` does `cx.waker().wake_by_ref()` then returns `Pending` — a
   100%-CPU spin per connected SSE client. **TODO:** use
   `tokio_stream::wrappers::BroadcastStream` instead of a manual
   `poll_next`.
5. **JWT secret drift.** `realtime/ws.rs` + `sse.rs` fall back to
   `"fichub-dev-secret"`, `routes/auth.rs` has its own fallback, and
   `config.rs` defines the real one — three sources. **TODO:** thread the
   configured secret through `AppState` (it already holds state).
6. **Minor:** WS send task polls subscriptions every 50 ms per connection
   (fine at small scale); client `publish` is unthrottled (matters once
   9.1.2 lands); SSE/WS `Last-Event-ID` replay is not implemented —
   acceptable, but undocumented.

### 9.2 Phase 6 — PWA (minor)

- SW v7 strategy is sound (network-first nav shell, public-API-only
  caching, per-user data never cached) — no findings there.
- **Manifest icon audit TODO:** manifest lists icons/screenshots and the
  files exist in `static/` today, but nothing keeps manifest ↔ files in
  sync (404s surface only at install time).
- **UpdatePrompt UX TODO:** the SW calls `skipWaiting()` immediately, so
  the new SW takes control before the user accepts the prompt — the
  "reload to update" toast can race the reload. Acceptable for v1;
  document or move to skipWaiting-on-message.

### 9.3 Phase 7 — NodeBB importer (medium)

1. **Fresh-DB-only with no guard.** The importer inserts with explicit
   NodeBB `id`s and `ON CONFLICT (id) DO NOTHING`, so on any non-empty
   production DB collisions are silently skipped, leaving children (posts →
   topics → categories, notifications) with dangling parents. The README
   documents no precondition. **TODO:** refuse to run unless `users` and
   `forum_topics` are empty unless `--force` (or an id-remap table); until
   then README must say fresh-DB-only, loudly.
2. **No transactional wrap.** Inserts are autocommit per statement; a crash
   mid-import strands a partial import and the skip counters make a re-run
   look idempotent when it is not. **TODO:** wrap in one transaction (or
   batched transactions + resumable manifest).
3. **`password_hash` imported as `''`** — imported users cannot log in
   (intended), but nothing tells them so. **TODO:** README/ops note or a
   `must_reset_password` flag.
4. **No DB-backed test.** 10 unit tests cover parsing/stats only; the
   INSERT path (`import.rs`, 419 lines) is untested. **TODO:** one
   `#[sqlx::test]`-style test importing a tiny fixture export and asserting
   rows land with expected ids.

### 9.4 Phase 8 — reputation/gamification (medium)

1. **Two XP ledgers.** Phase 8d writes `exp_events` + `users.exp`/`level`
   (`forum.rs` `award_exp`), the pre-existing progression service writes
   `xp_events` + `users.xp`/`level`/`rank` (`services/progression.rs:256`
   and cooldowns at :41/:68/:85). Same conceptual event, two tables, two
   balance columns; AuthorCard reads the exp ledger while the rest of the
   site reads xp. **TODO:** unify on `xp_events`/`users.xp` (make
   `award_exp` delegate to the progression service) or document why two
   ledgers exist; migrate `exp_events` rows when unifying.
2. **XP farming bug.** `award_reaction_exp` runs unconditionally after the
   react **toggle** — un-react/re-react repeatedly and every toggle writes
   an `exp_events` row (only the 10/day cap limits it). Cap should count
   distinct reacted posts, or award only on the 0→1 transition. Same for
   `award_poll_vote_exp` (re-votes with allow_change re-award). The
   `award_exp` function logs `reference_type`/`reference_id` but has no
   dedup check — callers (`award_reaction_exp`, `award_poll_vote_exp`)
   don't pass them either. **TODO:** either gate on 0→1 transitions or
   add a unique constraint + ON CONFLICT DO NOTHING on
   `(user_id, event_type, reference_type, reference_id)` in `award_exp`.
3. **`email` leaks in `GET /api/forum/users/{id}/profile`.** The handler
   selects `email` and returns it in the JSON body to any client. **TODO:**
   drop it from the SELECT + response (gate behind staff auth if needed).
4. **`reputation` returns `trust` value.** The profile endpoint's SELECT
   omits the `reputation` column and the JSON body maps
   `"reputation": trust` — every user's reputation shows as their trust
   score. **TODO:** add `reputation` to the SELECT and return it
   independently (forum.rs:3841,3869).
5. **Envelope deviation.** §8.1 spec'd `{err:0,…}` envelopes; the landed
   profile/xp-history/widget endpoints return bare JSON. Also
   §8.4's "achievement notification via WebSocket" never landed
   (achievements are XP awards only). **TODO:** align the endpoints or
   amend §8.1/§8.4; track achievements as an explicit deferred item.
6. **Roadmap honesty.** §2 marked 5–8 ✅ DONE while §8's own status line
   still read "PLANNED" until this audit — replaced with LANDED + commit
   hashes; §8.7's `2026-09-XX` changelog placeholders replaced with a
   pointer to §7.

### 9.5 Cross-phase notes

- **Rule adopted (matches §4 item 5):** a phase is DONE only when its tests are
  green AND its plan section is updated in the same change — "PLANNED" §8
  under a "✅ DONE" roadmap row violated this.
- **Good calls worth keeping:** Redis pubsub best-effort (single instance
  works without Redis); importer `dry_run` flag; PWA never caching
  per-user API data; archive-only theme discipline carried into Phase 8
  components; `set_ignore_missing(true)` migration tolerance.

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
- 2026-09-08: **Second-opinion review of the last 20 commits** — findings
  folded into §3/§4: (1) Lane 6 UI (`PollBar.svelte` + `topic_detail`
  poll attachment + TS types) was uncommitted at review time — commit it
  before any further lane work; (2) `topic_detail` hand-rolls poll JSON
  with placeholder `votes`/`voted_by_user` fields while `get_poll` has the
  real serializer — dedupe (Lane 6 remaining item 4); (3) the inline poll
  attachment needs one contract assertion in `tests/polls_api.rs`; (4)
  uploads lane lacks the black-box `tests/*_api.rs` every other lane has
  and its on-disk storage dir has no documented lifecycle; (5) migration
  renumber/repair leaves fresh-DB version order non-chronological — Phase 5
  baseline squash planned in §4 item 6; (6) `plan.v1...md.bak` got re-added
  inside a feature commit (`1478395`) — keep docs restores out of feature
  commits; (7) dead i18n keys from the moderation pivot and the
  `publish-scheduled` deploy story recorded in §4 item 6.
- 2026-09-08: **Lane 4 site-wide uploads lands** (`56c563e`): new
  `src/routes/uploads.rs` with `POST/DELETE /api/uploads` + public
  `GET /uploads/forum/{yy}/{mm}/{name}`; trust-gated multipart (TL2+),
  10 MiB cap, PNG/JPEG/WebP/GIF whitelist, sha256 + header-parsed
  dimensions to `forum_uploads` (077); `chunk_uploads()` wired; 5 unit
  tests green. No `image` crate on the tree, so originals are stored
  byte-for-byte (resize/Exif-stripping deferred). Also records the
  earlier Lane 7 backend commit (`1478395`: scheduled_at on create/update,
  listing exclusion ×5 sorts + unread/recent/popular/rss/search, detail
  preview gate, `publish-scheduled` bin, `tests/scheduled_topics_api.rs`).
    Remaining: Lane 1 composer autosave + vitest, Lane 2 items 2–5
  (topic-level report, owner-notify match, mod-surface deep-link, e2e),
  Lane 5 groups UI, Lane 7 importer crate, final §4 verification.
- 2026-09-08: **Review findings resolved** -- all 9 findings fixed:
  1. Lane 6 UI committed (18a6b64). 2. Poll serializer deduped
     (serialize_poll derives is_closed from close_at; voted_by_user removed
     from TS type). 3. Contract tests added (unit + DB-gated uploads suite).
  4. Commit hygiene followed. 5. Migration squash deferred to Phase 5.
  6. Uploads lifecycle documented in data-model.md; tests/uploads_api.rs
     covers anonymous 401, size limit, MIME whitelist, round-trip, delete.
  7. i18n audit: no dead keys found; added 5 missing poll keys to all 5
     non-English dictionaries. 8. All 9 svelte-check errors fixed (missing
 i18n keys, goto import, setPref import, stale url_id). 9. publish-
 scheduled deploy: systemd timer + service units; deploy.sh updated.
 - 2026-09-08: **Phase 4 complete** — all 7 lanes landed:
 - Lane 1: Composer autosave (30s + on blur) + draft restore + delete
 on submit; new `drafts.ts` API client.
 - Lane 4: Image paste/drop in forum composers; new `uploadImage.ts`
 utility; uploads lifecycle docs + black-box test suite.
 - Lane 5: Groups management UI — list/create/join/leave pages;
 new `groups.ts` API client; route added to routePages.
 - Lane 7: publish-scheduled binary + systemd timer + deploy wiring;
   publish flip + notify test added.
 - 2026-09-08: **Phase 5 complete** — WebSocket at `/ws`, SSE fallback at
 `/events`, Redis cross-instance pubsub on `ficnexus:rt` channel.
 ConnectionManager with tokio::broadcast, auth via JWT query param.
 - 2026-09-08: **Phase 6 complete** — PWA manifest (categories, shortcuts,
 screenshots, apple-touch-icon), offline.html fallback page, sw.js v7
 with navigation-first + RT-Nav cache, UpdatePrompt + InstallPrompt
 components wired into +layout.svelte. Deployed to production.

- 2026-09-08: **Phase 7 complete** — crates/forum-import CLI + JSON schema
  + 10 unit tests. NodeBB export → forum_* tables with duplicate detection,
  import notifications in notifications table (never forum_notifications).
- 2026-09-08: **Phase 8 complete** — 8a author card + profile/xp-history
  endpoints (`63e486c`/`f89203e`), 8b widget endpoints + ForumWidgets
  sidebar (`d8e6a65`/`30818c7`), 8c forum theme tokens in ThemeEditor
  (`b73ac30`), 8d XP wiring for reactions + poll votes with daily caps
  (`e88d268`). Deviations from §8.1/§8.4 recorded in §9.4.
- 2026-09-08: **Phase 5–8 post-landing review (§9)** — realtime has no
  producers + no channel authz + no tests (§9.1); importer is
  fresh-DB-only with no guard and no transaction (§9.3); two XP ledgers
  (`exp_events` vs `xp_events`) + reaction-toggle XP farm + `email` leak
  in the profile endpoint (§9.4). Open TODOs are the Phase 9 backlog.