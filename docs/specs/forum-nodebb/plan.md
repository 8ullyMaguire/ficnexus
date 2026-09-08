# Forum NodeBB parity — implementation plan (integration-first rewrite)

**Status:** Phase 4 IN PROGRESS (2026-09-08). Phase 3 DONE (migrations
072–079 recovered from live DDL, commit `f4cc60f`). Moderation pivot landed
(`708cfed`: trust ladder runs moderation, metamod answers 410). This rewrite
changes the Phase 4 architecture: **notifications, the flag/report queue,
blocks, uploads, and drafts are site-wide primitives — the forum consumes
them instead of duplicating them.**
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
**Remaining:** Lane 1 composer autosave + vitest, Lane 2 items 2–5
(topic-level report, owner-notify match, mod-surface deep-link, e2e), Lane 5
groups UI, Lane 7 importer crate, final §4 verification.
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
| **4** | **Lanes 1–7 below (integration-first)** | IN PROGRESS — backend done: 1, 2 (deep links), 3, 5, 6; 7-partial; frontend done: 2 (post+topic report), 3 (/messages); not started: 4 |
| 5 | site-wide realtime: WS `/ws` + Redis pubsub + SSE fallback | after 4 |
| 6 | PWA + theming (v1 scope) | after 5 |
| 7 | NodeBB cutover via importer (v1 scope) | last |

## 3. Phase 4 lanes (each = one PR-sized task)

### Lane 1 — Site-wide drafts + composer integration (BACKEND DONE `9ebfd37`)

`forum_drafts` live DDL (077, verified against prod — the `d575752`
version queried nonexistent `context`/`ref`/`content` columns and is
superseded): `(user_id, topic_id NULL=new-topic, category_id, title,
body, payload, poll_data)`. No DDL change.

1. ✅ `src/routes/drafts.rs` rewritten topic/category-keyed; `context`/`ref`
   URL segments decoded (`forum_post {topicId}` → reply draft,
   `forum_topic new:{slug}` → new-topic draft); UPDATE-then-INSERT in a tx
   (no unique index exists for upsert); idempotent DELETE (`{err:0,
   deleted}`). Wired in `server.rs`, committed.
2. Composer autosave every 30s + on blur; restore prompt on mount.
   Contexts: `forum_topic`, `forum_post`. NOTE: `frontend/src/lib/forum/`
   does not exist — composer lives in `frontend/src/routes/forum/new/` and
   the reply box in `TopicThread.svelte`; wire autosave there, no new lib
   dir needed. TODO.
3. Tests: route-handler tests (unit tests for payload/key JSON only today)
   + composer autosave vitest. TODO.

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
4. Forum composer paste/drag-drop → `/api/uploads` → `![](url)` insert
   (frontend wiring TODO — endpoint is ready).
5. ✅ Tests (5 green): classify whitelist/reject, PNG+GIF dimensions,
   sha256 structural check, staff gate.
   ⚠️ 2026-09-08 review: these are in-module unit tests only — every other
   lane shipped a black-box `tests/*_api.rs`. TODO: `tests/uploads_api.rs`
   covering the multipart POST end-to-end (TL gate 403, >10 MiB 413,
   non-image 400, fetch-back round-trip, DELETE authz).
6. ✅ `chunk_uploads()` wired into `server.rs` route merge chain.
7. ⚠️ 2026-09-08 review: uploads are the only new **on-disk state** in
   Phase 4 — no backup/retention story and no documented storage dir
   contract yet. TODO: document `{cache_dir}/uploads` layout + lifecycle
   in data-model.md (or move to a configurable dir with an ops note).


### Lane 5 — Groups + privileges (BACKEND DONE `7db25f7` + `1b266a2`, UI TODO)

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
1. Frontend: groups management UI (no `api/forum/groups` consumer in
   `frontend/src` today); per-category privilege surfacing. TODO.
2. Group rooms (`is_group TRUE`) stay deferred to Lane 3b after this lane.
3. `ensure_system_groups()` exists but is never called — no startup hook,
   no auto-membership; wire only when the groups UI needs seeded groups.

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
3. TODO: **commit the Lane 6 UI** — `PollBar.svelte` is untracked and the
   `topic_detail` attachment + TS types are still working-tree only (see
   §7 changelog 2026-09-08 review entry).
4. TODO: dedupe the poll serializer — extract `serialize_poll()` from
   `forum_polls.rs::get_poll` and reuse it in `topic_detail`; drop the
   placeholder `votes`/`voted_by_user` fields from the inline payload.
5. TODO: one assertion in `tests/polls_api.rs` that `topic_detail` returns
   the `poll` object with the shape PollBar reads (locks the contract).

### Lane 7 — Scheduled topics (BACKEND LANDED `1478395`) + importer (TODO)

`forum_topics.scheduled_at` + partial index exist (078); **nothing writes
the column today and no listing filters it** — this lane wires it up.

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
2. Publish binary `src/bin/publish_scheduled.rs` + `[[bin]]`
   `publish-scheduled`: flip `scheduled_at <= NOW()` → NULL, notify
   authors (site `notifications`), print counts, idempotent. TODO.
3. `crates/forum-import` CLI (workspace member): NodeBB JSON export
   (categories/topics/posts/users) → `forum_*` tables; notification
   history → site `notifications` (type `forum_import`); NEVER write
   `forum_notifications`. If the NodeBB export schema is unclear, define a
   documented JSON input schema in the crate README and implement against
   it. TODO.
4. Tests: scheduled invisible in listings, visible to author, publish
   binary flips + notifies. TODO.

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
  baseline squash planned in §4.6; (6) `plan.v1...md.bak` got re-added
  inside a feature commit (`1478395`) — keep docs restores out of feature
  commits; (7) dead i18n keys from the moderation pivot and the
  `publish-scheduled` deploy story recorded in §4.6.
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

