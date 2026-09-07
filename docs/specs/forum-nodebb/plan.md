# Forum NodeBB parity — implementation plan

**Status:** Phase 3 DONE (2026-09-07). Migrations 072–079 were applied to prod but the `.sql` files were missing from the repo; recovered from live DDL and re-recorded in `_sqlx_migrations`. Phases 4–7 remain (this file, §2). **Critical finding:** no Rust code currently references `forum_groups`, `forum_polls`, `forum_rooms`, `forum_messages`, `forum_notifications`, `forum_uploads`, `forum_user_blocks`, `forum_drafts`, or `forum_privileges` (grep-verified) — the tables are real and the DB is consistent, but handlers/routes/frontend do not exist yet.
**Spec dir:** `/home/alvaro/code/rust/ficnexus/docs/specs/forum-nodebb/` (spec.md, data-model.md, contracts/, research.md, quickstart.md)
**Read order for a new dev:** this file → `spec.md` → `data-model.md` → `contracts/*.md` → `quickstart.md`
**Author:** Cline, 2026-09-06. Every anchor verified against the codebase the same day.

> **Why this replaces the old plan:** the previous version described `forum_core/`
> as a to-be-created directory with migrations starting at `072_`. The codebase
> has moved on: the crate lives at `crates/forum-core/` (workspace member #3),
> migrations have reached `089_`, and 47 forum handlers, `082_forum_parity.sql`,
> and much of the frontend already ship. What remains is the half that does NOT
> exist anywhere in `src/` or `crates/forum-core/src/` (verified by grep):
> groups, privileges, flags, polls, messaging/DMs, notifications, uploads,
> realtime, and the importer.

## 0. Ground truth (verified 2026-09-06 — trust these over any doc)

| Fact | Anchor |
|---|---|
| Forum domain crate is `crates/forum-core` (workspace member; dep alias `forum-core = { path = "crates/forum-core" }`, root `Cargo.toml:46`) | `crates/forum-core/src/{lib,model,store,store/pg,moderation,search}.rs` |
| Crate runs its **own** migrations via `sqlx::migrate!("./migrations")` relative to its manifest dir | `crates/forum-core/src/store/pg.rs:45-49` |
| Crate's own migration: `crates/forum-core/migrations/202608140001_forum_core.sql` | creates the base `forum_*` tables standalone |
| Embedded app migrations are at **089** (`089_forum_topic_tags_trigger_fix.sql`); next free numbers are **090+** | `ls migrations/` |
| Base forum tables live in `migrations/001_initial.sql` (categories/topics/posts/votes/follows/read_state/bans/metamod/mod_grants/reactions/edit_proposals/topic_views) | `grep "CREATE TABLE public.forum_" migrations/001_initial.sql` |
| Parity additions so far: `082_forum_parity.sql` adds `forum_topic_tags`, `forum_user_prefs`; tags handlers exist (`set_topic_tags`, `get_topic_tags`, 5-tag cap) | `migrations/082_forum_parity.sql`, `src/routes/forum.rs:1931-1952` |
| `src/routes/forum.rs` = 3686 lines, **47 handlers**, including moderation, metamod, edit queue, RSS, prefs, level | `grep -c "pub async fn" src/routes/forum.rs` |
| FTS search already uses `to_tsquery` + `ts_headline` + `ts_rank` over `search_vector` via `crate::search::parser` | `src/routes/forum.rs:3228+` |
| Trust system is TL0-6, `trust.rs` has `flag_weight()`, `PUBLISH_MIN_TRUST=2`, `RESOLVE_MIN_TRUST=5`, `assert_min_trust()` | `src/services/trust.rs:25-127` |
| Admin gate: `FORUM_ADMIN_LEVEL` env, default 100 | `src/routes/forum.rs:69-71` |
| Auth extractor is `AuthUser` | `src/routes/auth.rs:209` |
| Reputation fn exists: `update_reputation_and_promote` | `src/db/queries/reputation.rs:8` |
| Axum 0.8 with `multipart` feature; **no `ws` feature yet**; no pubsub code anywhere in `src/` | `Cargo.toml:22` |
| sqlx 0.9 with macros + migrate features; **no `.sqlx/` offline dir** — use `just check`/`just test` and `DATABASE_URL` env | root `Cargo.toml:30` |
| Frontend already ships: `forum/page.test.ts`, `[categorySlug]`, `board/[topicSlug].[topicId]`, search, recent/unread/popular, moderate/metamod/invites/blocks, `new/`, `apply/`; `lib/api/forum.ts` (757 lines) + `forum.test.ts`; components `forum/{PassageContextCard,ReactionPicker,TopicThread}` + `ForumBottomNav.svelte`; `lib/pwa/{register,strategies}` + `static/manifest.webmanifest` + `static/sw.js` | `frontend/src/...` |
| **Does not exist anywhere yet** (grep-verified): groups, privileges, flags/reports UI+API, polls, rooms/messages/DMs, notifications, uploads, WS/pubsub, NodeBB importer, scheduled topics | — |

## 1. Conventions (from AGENTS.md + repo patterns — enforced in review)

- No `unwrap()` in production code — `?` or `.expect("reason")` (AGENTS.md:20).
- Conventional commits (feat/fix/chore/docs).
- All API JSON is `{ "err": 0, … }` envelope; auth failures are **400** with
  `{"err":…}` per repo convention (see `src/error.rs` `AppError` + existing
  forum handlers).
- Every migration file **must** end with
  `ALTER TABLE <table> OWNER TO fichub;` and use `CREATE TABLE IF NOT EXISTS`
  (see `082_forum_parity.sql` for the exact house style).
- Route handlers follow the existing pattern: `AuthUser` extractor → trust
  gate via `assert_min_trust`/`FORUM_ADMIN_LEVEL` → sqlx query → envelope.
- Frontend: SvelteKit + Svelte 5 runes, `page.test.ts` per route, `t()` i18n
  dictionaries, vitest (`npm run test` / `npm run test:watch`).
- Test commands: `just test` (`cargo test --lib`), `just test-db`,
  `just test-frontend`, `just check`, `just clippy` (see `justfile`).
- Keep every new pure function unit-tested; integration tests go in `tests/`
  (49 files there already — copy the `*_api.rs` pattern).
## 2. Remaining work — phase by phase

Every phase is a separate PR-sized unit. Order: 3 (schema) → 4 (API) →
5 (realtime) → 6 (frontend). Within Phase 4 the lanes are independent.

### Phase 3 — Migrations 090–098 (the missing half of the schema)

The DDL for every table below **already exists in prose** in
`data-model.md` §1–§3 (columns, indexes, search-vector triggers). Transcribe
it into house-style SQL: copy `migrations/082_forum_parity.sql` style —
`CREATE TABLE IF NOT EXISTS`, explicit FKs, `ALTER TABLE … OWNER TO fichub;`
at the end of each file.

| File | Creates | Source of truth |
|---|---|---|
| `migrations/090_forum_groups.sql` | `forum_groups`, `forum_group_members` | data-model.md §1 |
| `migrations/091_forum_privileges.sql` | `forum_privileges` (category × group matrix) | data-model.md §1 |
| `migrations/092_forum_polls.sql` | `forum_polls`, `forum_poll_options`, `forum_poll_votes` | data-model.md §1 |
| `migrations/093_forum_messaging.sql` | `forum_messages_rooms`, `forum_room_members`, `forum_messages`, `user_blocks` | data-model.md §1 |
| `migrations/094_forum_drafts_uploads.sql` | `forum_drafts`, `forum_uploads` | data-model.md §1 |
| `migrations/095_forum_notifications.sql` | `forum_notifications`, `forum_notification_digests` | data-model.md §1 |
| `migrations/096_forum_scheduled_topics.sql` | `forum_topics.scheduled_at` nullable column | data-model.md §1 |
| `migrations/097_user_reports_extend.sql` | adds `weight`, `auto_status`, `resolved_by`, `resolved_at` to `user_reports` (base table `001_initial.sql:3418`) | data-model.md §2 |
| `migrations/098_forum_search_triggers.sql` | `search_vector` tsvector columns + triggers on topics/posts/users/groups | data-model.md §3 |

Rules:
- `user_reports` is ALTERed, never dropped — it has live rows.
- Mirror each new table into `crates/forum-core/migrations/` as one new file
  (e.g. `20260906000001_forum_groups.sql`) so the crate boots standalone
  (`sqlx::migrate!("./migrations")` at `store/pg.rs:49`); the crate copy must
  not reference ficnexus-only tables.
- Verify: `sqlx migrate run` on a scratch DB, then `just check` (sqlx macros
  compile against the live schema).

### Phase 4 — Backend API (the bulk of the remaining work)

Register every new route in `src/server.rs` beside the existing forum block
(`chunk_forum_activitypub()`, `server.rs:1060+` — follow its nested-router
style). Handlers live in new `src/routes/forum_*.rs` files; the auth, trust,
and error-envelope patterns are identical to `src/routes/forum.rs`.

**Lane A — groups + privileges** (`forum_groups.rs`, `forum_privileges.rs`;
contracts/forum-groups-privileges.md)

- `GET/POST /api/forum/groups`, `GET/PATCH/DELETE /api/forum/groups/{id}`,
  `POST/DELETE /api/forum/groups/{id}/members` (+ list members),
  `GET/PUT /api/forum/categories/{id}/privileges` (per-group matrix).
- CRUD via sqlx against the Phase-3 tables; membership and category changes
  recompute the effective privilege matrix for affected categories.
- Helper: `pub async fn can(db, user_id, category_id, action) -> bool` —
  user's groups ∩ category matrix; resolve per request, no global cache.
- Trust gates: group **create** ≥ `PUBLISH_MIN_TRUST` (trust.rs:39);
  privilege **edit** admin-only (`FORUM_ADMIN_LEVEL`, forum.rs:69).
- Tests: unit for the matrix-resolution pure fn; integration
  `tests/forum_groups_api.rs` copying an existing `tests/*_api.rs` file.

**Lane B — polls** (`forum_polls.rs`; contracts/forum-polls-events.md)

- `PUT /api/forum/topics/{id}/poll` (author or admin), `GET …/poll`,
  `POST …/poll/vote` (single vote per user; revote = update), `DELETE …/vote`.
- Counts via `COUNT(*) GROUP BY option_id`; result visibility per contract.
- Trust: vote ≥ `PUBLISH_MIN_TRUST`; create author/admin-gated.
- Integration test `tests/forum_polls_api.rs`: create → vote → revote →
  counts; double-vote rejected.

**Lane C — messaging/DMs** (`forum_messaging.rs`; contracts/forum-messaging.md)

- Rooms (DM = exactly 2 members, group rooms ≥ 2), members, messages,
  `user_blocks`. Blocks win over every message path.
- `GET/POST /api/forum/rooms`, `GET /api/forum/rooms/{id}/messages?before=`
  (cursor pagination, 50/page), `POST /api/forum/rooms/{id}/messages`,
  `POST/DELETE /api/forum/users/{id}/block`.
- Helper `is_blocked(a, b)` shared with notifications fan-out (Lane E).
- Trust: messaging ≥ TL1 unless the contract says otherwise — contract wins.
- Tests: DM create idempotent (same pair → same room), block prevents send,
  cursor pagination stable.

**Lane D — flags/reports** (`forum_flags.rs`; contracts/forum-flags-moderation.md)

- `POST /api/forum/flags` (report post/topic/user), `GET /api/forum/flags`
  (staff queue, `RESOLVE_MIN_TRUST` = 5, trust.rs:43),
  `POST /api/forum/flags/{id}/resolve`.
- **Weighted auto-status:** on report insert, weight =
  `flag_weight(reporter.trust_level)` (trust.rs:47) combined with the
  reporter's accumulated `user_reports.weight`; when a post's total crosses
  the threshold in `spec.md`, set `forum_posts.status='hidden'` through the
  same transition logic as `forum_core::moderation`. Persists into the
  extended `user_reports` columns (migration 097).
- Tests: 3× TL2 reporters auto-hide a post; TL5 resolve unhides.

**Lane E — notifications** (`forum_notifications.rs`; contracts/forum-notifications.md)

- Table `forum_notifications` (recipient, actor, kind, payload JSONB,
  read_at) + digest watermark table (data-model §5).
- Fan-out write paths: reply-to-your-topic, @mention parsed from post body,
  followed-topic got a new post, poll ended, room message (goes through
  Lane C's `is_blocked`), badge/grant (reuse existing metamod grant flow).
- `GET /api/forum/notifications?unread=`, `POST …/read-all`,
  `GET …/unread-count` (bell polling endpoint).
- Digest: extend the cron stub `src/bin/forum_scheduled_promote.rs` to also
  send daily digests using watermark `forum:digest:last_sent`.
- Tests: reply → row exists; read-all → unread-count 0; blocked user →
  no row.

**Lane F — uploads + drafts** (`forum_uploads.rs`; contracts/forum-uploads.md)

- `POST /api/forum/uploads` — axum `multipart` (feature already enabled,
  Cargo.toml:22). Per contract: image-mime allowlist, ≤5 MiB, per-user daily
  cap via Redis `forum:upload:{user}:{date}` (data-model §5).
- Resize/normalize with the `image` crate (research.md decision) before
  storing under `cache/uploads/` (add `.gitignore` entry for contents).
- Serving: `GET /api/forum/uploads/{id}` streaming handler OR existing
  static service — the contract file is the authority; implement what it
  says.
- Drafts ride along: `GET/PUT /api/forum/drafts/{topic_id?}` against
  `forum_drafts` (composer autosave, Phase 6 uses this).

**Lane G — scheduled topics + NodeBB importer** (`src/bin/` + new crate)

- `src/bin/forum_scheduled_promote.rs`: loop/cron promoting
  `forum_topics WHERE scheduled_at <= now()` from scheduled → published
  (transition via `forum_core::moderation` logic).
- Importer: new **workspace member** `crates/forum-import` — add to
  `workspace.members` + `[workspace.dependencies]` exactly the way
  `forum-core` is declared (`Cargo.toml:16` and `:46`). CLI parses a NodeBB
  JSON dump (shape in `research.md`), upserts into Phase-3 tables through
  sqlx. Ship with `--dry-run` as the default; integration test feeds a
  minimal fixture dump from `tests/fixtures/nodebb/`.

### Phase 5 — Realtime (WS + SSE + presence)

Nothing realtime exists yet (no `ws` feature, no pubsub code — verified).
Build in this order:

1. **Enable the transport:** add `"ws"` to the axum features list
   (`Cargo.toml:22`). Add `tokio-tungstenite` is NOT needed — axum's `ws`
   feature suffices.
2. **Pubsub helper:** `src/services/forum_pubsub.rs` — thin wrapper around
   `redis::aio::PubSub`: `publish(channel, json)` and
   `spawn_subscriber(channels, tx: tokio::sync::mpsc::Sender<Event>)`.
   Channels per data-model §5: `forum:topic:{id}`, `forum:category:{id}`,
   `forum:room:{id}`, `forum:notify:{user_id}`.
3. **WS endpoint:** `src/routes/forum_ws.rs` → `GET /ws/forum` via
   `axum::extract::ws::WebSocketUpgrade`. On connect: auth (same
   cookie/JWT as HTTP), subscribe the socket to the user's channels
   (rooms joined, topics followed, own notify channel). One tokio task per
   socket reading from the pubsub receiver; one task reading client frames
   (typing pings, channel joins).
4. **Emit side:** in the Phase-4 mutation handlers (post create, room
   message, poll vote), `PUBLISH` a JSON event after the DB write commits.
   Event shape documented in `contracts/forum-ws-sse.md`.
5. **Presence/typing:** `SETEX forum:typing:{topic}:{user} 3` + publish
   `typing_start/stop` (clients debounce 500ms); presence via
   `SADD forum:presence:{room}` with TTL heartbeat refresh. New
   `src/services/forum_presence.rs` wraps these.
6. **Degraded mode:** if Redis is down, fall back to a single-process
   `tokio::sync::broadcast` fanout and log it (health endpoint reports
   degraded). Code the seam now: an enum `Fanout::Redis(PubSub) |
   Fanout::Local(broadcast::Sender<Event>)` chosen at startup.
7. **SSE fallback:** `GET /api/forum/stream?channel=…` for anonymous
   read-only streams (no typing, no DMs) — `axum::response::sse::Sse`
   with the same pubsub backing.
8. **Tests:** pubsub round-trip against a real Redis (mark `#[ignore]` +
   run in `just test-db`); WS auth-reject test; typing TTL expiry test.
   `contracts/forum-ws-sse.md` defines every event payload — implement
   exactly those.

### Phase 6 — Frontend (SvelteKit) + PWA completion

What exists already: category grid + `board/` topic page, search,
recent/unread/popular, mod pages, `lib/api/forum.ts` (757 lines),
`TopicThread`/`ReactionPicker`/`PassageContextCard` components,
`ForumBottomNav`, PWA `register`/`strategies` + `manifest.webmanifest` +
`sw.js`. What's missing is the UI for the Phase-4/5 features:

1. **Groups UI:** `frontend/src/routes/forum/groups/` — list (cards),
   detail (members + join/leave), per-category privilege editor
   (admin-only guard client-side; server re-checks everything).
   `+page.svelte` + `page.test.ts` each, like existing routes.
2. **Poll UI:** `PollBar.svelte` in `lib/components/forum/` — render poll
   options, vote/revote, live results via WS event; embed inside the
   existing `board/[topicSlug].[topicId]` thread above the post stream;
   poll creation form inside the existing composer flow (`forum/new`).
3. **Messaging UI:** `frontend/src/routes/forum/messaging/` — room list +
   thread view; composer wired to `POST /rooms/{id}/messages`; live updates
   via WS room channel; block/unblock controls on user cards.
4. **Notifications UI:** bell in the header (patch the existing layout
   component) polling `unread-count`; `frontend/src/routes/forum/
   notifications/` inbox with mark-all-read; WS notify channel updates the
   badge live.
5. **Flags UI:** report button on posts/topics (dialog → `POST /api/forum/
   flags`); staff queue page `frontend/src/routes/forum/flags/` gated by
   the existing mod-page pattern (`moderate/` route as reference).
6. **Uploads UI:** drag-drop + paste handler in the composer → multipart
   `POST /uploads` → insert markdown image snippet; preview thumbnails.
7. **WS client lib:** `frontend/src/lib/forum/ws.ts` — reconnect with
   backoff, `?after=` catch-up on reconnect, typed event union mirroring
   `contracts/forum-ws-sse.md`. `sse.ts` fallback for anonymous viewers.
   Keep `lib/api/forum.ts` as the only REST surface; add WS as a sibling
   module, don't entangle them.
8. **PWA completion:** extend existing `sw.js` (don't replace — it already
   works) to cache `GET /api/forum/categories|topics|topics/{id}` and
   last-read topic pages; queue failed forum POSTs in IndexedDB until
   reconnect; add forum shortcuts to `manifest.webmanifest` (data-model
   §5 / spec.md PWA section list exactly what to cache).
9. **i18n:** every new string through the existing `t()` dictionaries in
   `lib/i18n/` (en first).
10. **Tests:** `page.test.ts` per new route + component tests for
    `PollBar` etc. (vitest, `npm run test`); mock the API layer the same
    way `lib/api/forum.test.ts` does.

### Phase 7 — NodeBB data migration + cutover (operator task, not code)

Run only after Phases 3–6 ship:

1. Dump NodeBB (`mongodump` → JSON per research.md mapping).
2. `cargo run -p forum-import -- --dry-run dump.json` → fix mapping errors.
3. Real run against staging; verify: topic/post/user counts match the dump,
   slugs resolved, tags attached, `search_vector` populated (run a few
   `search_forum` queries).
4. Cut over: DNS/proxy switch, keep NodeBB read-only for 2 weeks, then
   decommission. Rollback = repoint DNS (no data loss either way).

## 3. Execution order & dependency graph

```
Phase 3  migrations 090-098        ← everything depends on this
Phase 4  Lanes A-F (parallel)      ← each lane = one PR, independent
         Lane G (importer + cron)  ← can start anytime after Phase 3
Phase 5  realtime                  ← needs Phase 4 emit-points (lane C/E first)
Phase 6  frontend                  ← trails Phase 4/5 lane by lane:
                                      groups UI after A, polls after B,
                                      messaging after C, flags after D,
                                      notifications after E, uploads after F,
                                      ws.ts after Phase 5
Phase 7  cutover                   ← after everything
```

Junior-dev sanity rules:
- One lane per PR; never mix a migration with frontend code.
- Every PR green on `just check && just test && just test-frontend`.
- `cargo sqlx prepare`-style drift is not in play (no `.sqlx/` dir) — but
  `just check` fails if sqlx macros can't see the schema, so migrations must
  be applied to your dev DB before compiling new queries.

## 4. Verification checklist (per phase + final)

1. Phase 3: `sqlx migrate run` twice (idempotent), `just check` compiles,
   `psql -c '\dt forum_*'` shows all expected tables; standalone crate
   boots: `cargo run -p forum-core --example standalone` (see quickstart.md §3).
2. Phase 4 per lane: `just test` (unit), new `tests/forum_*_api.rs` green
   (`just test-db`), `just clippy` clean, curl each new endpoint with a
   logged-in cookie and confirm the `{err:0,…}` envelope + 400-on-auth shape.
3. Phase 5: two browser tabs show live post/typing/presence; kill Redis →
   degraded fanout logs and messages still flow single-process; SSE stream
   works logged-out.
4. Phase 6: `just test-frontend` + `just test-e2e` green; each new page
   renders logged-in and logged-out; PWA: airplane-mode reload of a
   previously-read topic still renders from SW cache.
5. Phase 7: staging counts match dump; sample topic deep-links resolve;
   search returns hits for imported content.
6. Update this file's status line and tick phases off in §2 as they land.

## 5. Out of scope (explicit)

- Rewriting the existing 47 handlers in `forum.rs` — they already match the
  contracts; touch them only if a contract test fails.
- Reputation/exp systems (data-model §4 ships the hook only; amounts are
  env-configurable per §9) — no gamification UI in this port.
- Mobile-native anything — PWA only.
- Migrating NodeBB *widgets/themes* — the AO3 skin replaces them by design.

## 6. Changelog

- 2026-09-06: Full rewrite. Previous plan predated the codebase: it
  described `forum_core/` at the repo root with migrations starting at 072
  and every backend lane as "NEW". Verified ground truth (§0) instead:
  crate lives at `crates/forum-core` (workspace member), migrations at 089,
  47 forum handlers + FTS search + tags + moderation/metamod already ship,
  and several "to build" frontend routes already exist. Remaining scope is
  now exact: migrations 090-098, seven API lanes, realtime from zero,
  frontend for the missing features, importer + cutover. All sibling spec
  docs (spec.md, data-model.md, contracts/, research.md, quickstart.md)
  remain authoritative for details and are referenced per task.
- 2026-09-07: Phase 3 (migrations) verified already applied to prod.
  Recovered missing `.sql` files 072–079 from live DDL (`pg_dump`),
  committed them, deleted the stale `_sqlx_migrations` rows, and
  re-ran `fichub migrate` — all 38 migrations apply cleanly. Two issues
  found and fixed during recovery:
  1. `forum_topic_tags` uses a `(topic_id, tag text)` shape in prod,
     not the `(topic_id, tag_id)` normalized shape from data-model.md
     §1.3 — the Rust handlers use plain tag strings and 089 explicitly
     drops the triggers that referenced `tag_id`. Migration 074 written
     to match reality.
  2. The data-model partial-index predicates `WHERE scheduled_at > NOW()`
     are not valid (`NOW()` is volatile). Replaced with a static
     `WHERE scheduled_at IS NOT NULL` predicate; the cron query applies
     `scheduled_at <= NOW()` at scan time (same selectivity).
  The `fichub` role did not exist in this DB — created it as
  `NOSUPERUSER NOCREATEDB NOCREATEROLE INHERIT LOGIN` to match the
  `OWNER TO fichub` lines in the existing migrations. **Phase 3 is
  done; Phases 4–7 (handlers, realtime, frontend, importer) remain.**

