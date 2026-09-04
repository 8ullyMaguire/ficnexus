# Implementation Plan: Forum NodeBB Port — Full Rust+Svelte+Postgres Replacement

**Branch**: `forum-nodebb` | **Date**: 2026-08-31 | **Spec**: `docs/specs/forum-nodebb/spec.md`
**Input**: Feature spec at `/specs/forum-nodebb/spec.md` + NodeBB 4.15.1 at `~/code/js/NodeBB` (read-only)

## Summary

Port NodeBB 4.15.1 to Rust/Axum + SvelteKit 5 + Postgres + Redis, 1:1 parity. Standalone crate (own Axum router, own migrations, pluggable `ReputationHook`) that also mounts into ficnexus at `/forum` + `/api/forum` to replace `forum_core` + `frontend/src/routes/forum/*`. Single ficnexus `users` table, ficnexus trust 0-6 for moderation + ficnexus reputation for leaderboard (no exp/levels) via `update_reputation_and_promote`, AO3 skin over NodeBB information architecture, WebSocket realtime (`/ws/forum` via `axum::extract::ws` + Redis pubsub, SSE fallback), local uploads, full NodeBB parity checklist before done. No NodeJS in production.

## Technical Context

**Language/Version**: Rust 1.82+ (edition 2024), TypeScript 5.6, Svelte 5 runes, Vite 5
**Primary Dependencies**: Axum 0.8 + tokio full, sqlx 0.9 (postgres+chrono+uuid), redis 1.4 (aio tokio), marked + dompurify (markdown), `forum_core` retained as base
**Storage**: PostgreSQL (canonical), Redis (pubsub + presence + rate windows + typing), local disk `/public/uploads/forum/{yyyy}/{mm}/{uuid}`
**Testing**: `cargo test` + `axum-test` for Rust, `vitest run` + `vitest e2e` for SvelteKit, `cargo sqlx prepare --check`
**Target Platform**: Linux (NFS deploy to ThinkCentre `fichub.service`), SvelteKit `adapter-static` SPA
**Project Type**: Web application — backend Axum + SvelteKit SPA + Postgres + Redis + PWA
**Performance Goals**: Topic list <200ms p95, topic+posts <300ms p95, WS fanout <500ms typing, <2s new_post push
**Constraints**: No `unwrap()` in prod (use `?`/`expect("reason")`), `inet` binds via string + `$N::inet`, `ALTER OWNER TO fichub`, `page.test.ts` naming, single `users` table, migrations additive (next `072_…`)
**Scale/Scope**: 20+ API routes, 10+ frontend routes, 15+ DB tables, WS + SSE + PWA, importer CLI

## Constitution Check

No constitution at `.specify/memory/constitution.md` (template only). Enforced gates still apply:
- Library-first: standalone forum crate remains embeddable (generic `ActorId` where sensible, opaque `payload` JSONB preserved).
- CLI surface: importer exposes `cargo run --bin forum-import-nodebb <dump.json>` text in/out.
- Test-first: every route requires contract + integration test; frontend requires `page.test.ts`.
- Observability: structured `tracing` per route, `modlog` for moderation, `trust_events` for level changes.

## Project Structure

### Documentation (this feature)

```
docs/specs/forum-nodebb/
├── spec.md              # feature spec (shipped)
├── plan.md              # this file
├── research.md          # Phase 0 output (open questions resolved)
├── data-model.md        # Phase 1 — entities + DDL deltas
├── quickstart.md        # Phase 1 — how to run standalone + embedded
├── contracts/           # Phase 1 — OpenAPI-style route contracts
│   ├── forum-categories.md
│   ├── forum-topics-posts.md
│   ├── forum-groups-privileges.md
│   ├── forum-flags-moderation.md
│   ├── forum-messaging.md
│   ├── forum-polls-events.md
│   ├── forum-search-tags.md
│   ├── forum-notifications.md
│   ├── forum-uploads.md
│   └── forum-ws-sse.md
└── tasks.md             # Phase 2 (speckit-tasks)
```

### Source Code (repository root)

```
# Backend (new + extended)
forum_core/                              # RETAINED, extended (groups, polls, messaging, uploads, privileges, flags)
├── src/
│   ├── model.rs                        # add Group, GroupMember, Privilege, Poll, PollOption, PollVote, Room, Message, Block, Draft, Upload, Tag
│   ├── store/mod.rs                    # Store trait extended (all new entity ops)
│   ├── store/pg.rs                     # Postgres impl for every new entity
│   ├── moderation.rs                   # trust-aware flag_weight + auto_status thresholds
│   └── search.rs                       # builder extended (tag/author/date filters)
├── migrations/                         # crate-own migrations (also replayed into FicHub's 072_… sequence)
│   ├── 001_forum_groups.sql
│   ├── 002_forum_privileges.sql
│   ├── 003_forum_tags.sql
│   ├── 004_forum_polls.sql
│   ├── 005_forum_messaging.sql
│   ├── 006_forum_uploads_drafts.sql
│   └── 007_forum_notifications.sql
└── Cargo.toml

crates/forum-import/                     # NEW — NodeBB JSON dump importer (optional but required for parity checklist)
├── src/main.rs                        # CLI: reads NodeBB JSON, upserts into forum_* tables
└── Cargo.toml

src/
├── routes/
│   ├── forum.rs                       # EXTENDED — replaces/augments all NodeBB route parity (see contracts/)
│   ├── forum_groups.rs                # NEW — groups CRUD + membership
│   ├── forum_privileges.rs            # NEW — per-category per-group privileges
│   ├── forum_flags.rs                 # NEW — flags/reports (TL2+ gate)
│   ├── forum_messaging.rs             # NEW — DMs + rooms
│   ├── forum_polls.rs                 # NEW — polls
│   ├── forum_search.rs                # NEW — FTS topics+posts+users+groups
│   ├── forum_uploads.rs               # NEW — multipart + image resize
│   ├── forum_notifications.rs         # NEW — notifications + mark-read
│   └── forum_ws.rs                    # NEW — /ws/forum handler + Redis pubsub fanout
├── services/
│   ├── forum_pubsub.rs                # NEW — Redis publish/subscribe helper (topic/category/room/notification channels)
│   ├── forum_presence.rs              # NEW — online presence + typing state
│   └── trust.rs                       # EXTENDED — flag_weight + forum gates (already TL0-6)
├── server.rs                          # EXTENDED — new routes registered + WS route + static compat redirects
├── config.rs                          # EXTENDED — forum points/window/levels + WS/Redis/upload knobs
└── bin/
    ├── forum_import_nodebb.rs         # NEW — binary entry for NodeBB dump importer
    └── forum_scheduled_promote.rs     # NEW — cron for scheduled topics

migrations/
├── 072_forum_groups.sql               # mirror of forum_core migration 001 (groups)
├── 073_forum_privileges.sql
├── 074_forum_tags.sql
├── 075_forum_polls.sql
├── 076_forum_messaging.sql
├── 077_forum_uploads_drafts.sql
├── 078_forum_notifications.sql
└── 079_forum_scheduled_topics.sql

frontend/
├── src/routes/forum/
│   ├── +page.svelte                  # REPLACED — AO3-skinned category grid (NodeBB layout preserved)
│   ├── [categorySlug]/
│   ├── board/[topicSlug].[topicId]/ # REPLACED — topic+posts (WS live, pagination, poll, tags)
│   ├── groups/                       # NEW — groups list/detail/member/privilege
│   ├── tags/[tag]/                   # NEW — tag pages
│   ├── search/                       # REPLACED — FTS with filters + snippets
│   ├── notifications/                # NEW — inbox + bell badge state
│   ├── messaging/                    # NEW — DM rooms + group rooms
│   ├── polls/                        # NEW — poll create/vote UI
│   ├── uploads/                      # NEW — drag-drop + paste + preview
│   └── (+ mod queue pages retained: moderate/metamod/invites/blocks)
├── src/lib/
│   ├── forum/
│   │   ├── ws.ts                    # NEW — WebSocket client (/ws/forum) + reconnect + catch-up
│   │   ├── sse.ts                   # NEW — SSE fallback client
│   │   ├── composer.ts              # NEW — markdown preview + @mention autocomplete + emoji + /fic card
│   │   └── pwa.ts                   # NEW — manifest + service worker + offline cache + VAPID
│   └── components/forum/            # NEW — ForumCard, ComposerBar, BottomNav, PollBar, NotificationBell patch, etc.
└── static/
    ├── manifest.webmanifest           # EXTENDED — PWA scope + forum shortcuts
    └── sw.js                         # NEW — service worker (offline cache + queue)

tests/ (integration)
└── tests/forum_*.rs                  # per-contract integration tests via axum-test
```

**Structure Decision**: Web application layout. Backend extends `forum_core` as embeddable library + `src/routes/forum*.rs` + services; frontend extends `frontend/src/routes/forum/*`. New crate `crates/forum-import` for importer CLI. Single `users` table shared; migrations replayed into main `migrations/` with sequential IDs.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| New `crates/forum-import` | NodeBB parity checklist requires dump importer; keeps main binary lean | Stuffing importer into `fichub` binary bloats dep graph + mixes one-shot CLI with server |
| `forum_core/migrations/` + `migrations/07x_…` duplication | Standalone crate must boot without FicHub DB; embedded mount must run from single FicHub migration sequence | Single location breaks standalone boot; requires either full FicHub DB or manual copy |

## Data Model (summary — full DDL in `data-model.md`)

Existing `forum_core` tables retained (`forum_categories`, `forum_topics`, `forum_posts`, `forum_post_votes`, `forum_follows`, `forum_read_state`, `forum_bans`) plus additive tables; `user_reports` extended with `weight/auto_status/resolved_by/at`; `users.trust_level` 0-6 authoritative and `users.reputation` + `reputation_events` for leaderboard (no new user table, no exp/level columns). New tables: `forum_groups`, `forum_group_members`, `forum_privileges` (category×group), `forum_tags` + `forum_topic_tags` + `forum_category_tags`, `forum_polls` + `forum_poll_options` + `forum_poll_votes`, `forum_messages_rooms` + `forum_room_members` + `forum_messages` + `user_blocks`, `forum_drafts`, `forum_uploads`, `forum_notifications` (with `forum_notification_digests` watermark), `forum_scheduled_topics` (or `forum_topics.scheduled_at` nullable), plus `ReputationHook` trait — not a table — that writes `reputation_events` (`forum_topic_create`, `forum_post_create`, `forum_mod_helpful`) via `db::queries::update_reputation_and_promote` when embedded (noop in standalone dev). FTS: `search_vector` on topics/posts/users/groups maintained at write time (no cron). Redis keys: `forum:topic:{id}`, `forum:category:{id}`, `forum:room:{id}`, `forum:typing:{topic|room}`, `forum:presence:{room|category}`, digest watermark `forum:digest:last_sent`, rep daily cap `forum:rep:{user_id}:{yyyy-mm-dd}` counter.

## API Surface (summary — contracts in `contracts/`)

All routes under `/api/forum` JSON (`{err:0,…}` / `AppError` 401/403 as 400 per repo convention), auth via `AuthUser`, SvelteKit pages under `/forum`. Categories (list/get/create/patch), topics (`?category=&cursor=&limit=` list, detail by id + by-slug, create/patch/delete, pin/lock/move/fork/merge), posts (create/patch/delete, votes/bookmarks, edit history), tags (list, merge, `/forum/tags/{tag}`), groups (`/api/forum/groups` CRUD + join/leave/invite), privileges (`/api/forum/privileges/{categoryId}` get/set), flags (`POST /api/forum/flags` TL2+ gate + weight + auto_status, queue `GET /api/forum/flags/queue` TL5+, resolve), bans/timeouts, search (`GET /api/forum/search?q=&category=&tag=&author=&since=` FTS), follows/watches, read-state, notifications (`/api/forum/notifications` + mark-read), messaging (`/api/forum/messaging/{rooms,messages,blocks}`), polls (`/api/forum/polls/{votes}`), scheduled (cron promote), events, rewards, uploads (multipart + resize), reputation config (`GET /api/forum/reputation/config` read-only env), WS `GET /ws/forum` + SSE `GET /api/forum/stream`, compat 301s (`/category/{id}/{slug}` etc.). Moderation points + metamod retained at `/api/forum/moderation/*` + `/api/forum/metamod/*` exactly as `docs/FORUM-API-CONTRACT.md`.

## Reputation Hook

Forum writes emit `ForumEvent::TopicCreated { user_id, topic_id } | PostCreated { user_id, post_id } | ModHelpful { moderator_id, action_id }`. A `ReputationHook` trait (`async fn awardForumRep(pool, event)`) maps to `update_reputation_and_promote` with deltas `FORUM_REPUTATION_TOPIC_CREATE` (2), `FORUM_REPUTATION_POST_CREATE` (1), `FORUM_REPUTATION_MOD_HELPFUL` (3) and enforces `FORUM_REPUTATION_FORUM_DAILY_CAP` (20) via `reputation_events` count for that day (or Redis `forum:rep:{user}:day` counter). Positive-only; no rep for deletes/flags. Embedded mount wires live hook (writes `users.reputation` → visible on `GET /api/leaderboard/*`); standalone dev wires noop/in-memory collector (crate still boots, tests assert awarded deltas without touching `users`). No exp/levels anywhere.

## Realtime Design

`GET /ws/forum` upgrades via `axum::extract::ws::WebSocketUpgrade` (feature `ws` on Axum). On connect: auth from cookie/JWT (same as HTTP), subscribe via `redis::aio::Connection` PubSub to channels for joined topics/categories/rooms + user notification channel. On topic/category/messaging mutation: handler `PUBLISH` JSON event to Redis; pubsub task fans out to all WS peers subscribed to that channel, plus writes notification row + pushes to user channel. Typing: ephemeral `SETEX forum:typing:{topic}:{user} 3` + publish `typing_start/stop`; clients debounce 500ms. Presence: `SADD forum:presence:{room}` + TTL heartbeat. Degraded mode: if Redis unavailable, single-process `tokio::sync::broadcast` fanout (logged, health reports degraded). SSE fallback at `/api/forum/stream?channel=forum:topic:{id}` for anonymous/read-only (no `typing`, no DM).

## Frontend + PWA + Theming

AO3 skin over NodeBB IA: ficnexus `docs/frontend-design.md` tokens (`--color-bg`, `--color-surface`, etc.) + header/footer/chrome; NodeBB layout (category cards, topic list, post stream, composer, bottom nav, notification bell) preserved. Svelte 5 runes, `+page.svelte` + `page.test.ts` per route, `i18n` via `t()` dictionaries, WS client `src/lib/forum/ws.ts` (reconnect + `?after=` catch-up) + `sse.ts` fallback, composer `src/lib/forum/composer.ts` (CommonMark via `marked` + NodeBB extensions, `@mention` autocomplete via `GET /api/users/search?q=prefix`, emoji via emoji picker, `/fic {url_id}` card via `GET /api/works/{id}`, drafts via localStorage + `POST /api/forum/drafts` backup every 30s). Uploads: drag-drop + paste handler posting `multipart` to `/api/forum/uploads`. PWA: `manifest.webmanifest` (name=ficnexus Forum, scope=/forum, display=standalone, icons 192/512, shortcuts), `sw.js` caches GET `/api/forum/categories|topics|topics/{id}` + last-read topic pages + drafts, queues failed POSTs in IndexedDB until reconnect, VAPID push via `POST /api/forum/push/subscribe` (applicationServerKey), foreground WS takes precedence when online.

## Build + Execution Order (one big port, ≤3 subagents)

Phase 0 — **Research** (`research.md`): NodeBB src inventory (`src/categories|topics|posts|groups|privileges|flags|messaging|notifications|search|polls|events|rewards|uploads|socket.io|widgets`) → Rust mapping, trust gate inventory, WS transport choice (Axum/ws + Redis pubsub chosen), upload resize lib (image crate), markdown pipeline (marked+dompurify), PWA strategy (Workbox-lite vs hand-rolled sw.js).

Phase 1 — **Data model + contracts + quickstart**: `data-model.md` (full DDL deltas, indexes, search_vector triggers, Redis keyspace), `contracts/*.md` (request/response examples + error codes + auth matrix), `quickstart.md` (standalone `cargo run -p forum_core --example standalone` + embedded `cargo run -- middleware mount`, `just` recipes, env vars).

Phase 2 — **Backend** (≤3 subagents, shared context via `plan.md` + `data-model.md` + `contracts/`):
- Lane A: `forum_core` model/store/pg + migrations 001-007 + `Store` trait tests.
- Lane B: `src/routes/forum*.rs` + `src/server.rs` registration + services (pubsub, presence, trust gates) + config.
- Lane C: Messaging + polls + scheduled + events + rewards + uploads (multipart + image resize) + importer crate.

Phase 3 — **Realtime + notifications**: WS handler + Redis fanout + SSE + typing/presence + notification write + digest cron + VAPID push wiring.

Phase 4 — **Frontend**: 10+ SvelteKit routes + lib/forum (ws/sse/composer/pwa) + components + AO3 skin + i18n + `page.test.ts` coverage + manifest/sw.

Phase 5 — **Parity checklist + QA**: run NodeBB parity checklist 100% (browse → write → search/tags → groups/privileges → flags/bans → messaging → polls/scheduled → uploads/composer → realtime → PWA → importer), `cargo test` + `vitest run` + `qa/api-walk.js` + manual click-test via `ssh -L` to ThinkCentre if needed, `cargo sqlx prepare --check`, `just`/`nextest` as applicable.

## Risks + Mitigations

- **NFS git fsync** (`/personal/documents/code`): NodeBB ref already rsynced NFS-safe (`/tmp/NodeBB` → `~/code/js/NodeBB`, `core.fsync false`); all new Cargo work at `~/code/rust/ficnexus` NFS — set `CARGO_TARGET_DIR=/media/alvaro/code-worktrees` per AGENTS.md.
- **WS + Redis pubsub at scale**: fallback single-process `broadcast` keeps dev/single-host alive; add Redis in CI/deploy; load-test WS fanout before declaring P1 done.
- **Markdown parity vs XSS**: use `marked` + `dompurify` (already dep) on client, `ammonia` or equivalent on server for stored HTML; never trust client-rendered HTML as stored value.
- **Trust vs NodeBB karma divergence**: spec-mandated, but document delta in `research.md` + show 403 reason="requires L2 Member" instead of silently failing.
- **Migration ordering**: never rewrite `migrations/001_initial.sql`; new `07x_` only, replay `forum_core` migrations verbatim; `sqlx migrate run` order is filename-sorted — keep `07x` contiguous.
- **Scope creep — 1:1 is large**: plan is one big port but lanes cap ≤3 subagents; main thread holds shared context; deliver in Phase 2-4 order so P1 stories unblock manually while P2 finishes.

## Junior Dev / LLM Execution Guide

1. Read `spec.md` + `plan.md` + `data-model.md` + relevant `contracts/*.md` before touching code; treat `~/code/js/NodeBB/src/**` as read-only reference for behavior, not copy source (GPL).
2. For each lane: write failing test first (`cargo test` or `page.test.ts`), then Rust/Svelte impl, then `cargo sqlx prepare --check` if queries changed, then `cargo test` + `vitest run`.
3. DB: run `sqlx migrate run` against local Postgres; verify with `psql \d forum_*`; add `ALTER OWNER TO fichub` for any new table created (per AGENTS.md).
4. Branch: `feat/forum-nodebb-<lane>` via git worktree under `/media/alvaro/code-worktrees` with `CARGO_TARGET_DIR` pointing there; never work on `main`.
5. Tokens: batch tool calls in scripts; use `caveman` skill (ultra) for comms; max 3 subagents at a time, main thread only for shared context — do not duplicate `plan.md` reads per subagent.
6. Completion: update `NodeBB Parity Checklist` in `spec.md` (100% required), run `qa/api-walk.js` to verify routes in `server.rs`, click-test `/forum` → create topic → reply → flag → WS live on phone width, then merge.

## References

- Spec: `.specify/specs/forum-nodebb/spec.md` (trust 0-6, AO3 skin, single account, 1:1 parity)
- NodeBB 4.15.1: `~/code/js/NodeBB` (NFS-safe) + `/tmp/NodeBB` (local mirror)
- Existing forum: `forum_core` (model/store/pg) + `src/routes/forum.rs` + `docs/FORUM-API-CONTRACT.md` + `docs/SPEC-COMMUNITY-PLATFORM.md` v2
- Trust: `src/services/trust.rs` + `migrations/013_071` (flag_weight, PUBLISH/RESOLVE_MIN_TRUST)
- Frontend design: `docs/frontend-design.md` + `frontend/src/routes/forum/*` + `frontend/src/lib/prefs.ts` + `frontend/src/lib/i18n/`
- Deploy: `AGENTS.md` + `docs/DEPLOYMENT.md` (ThinkCentre scp→/tmp→/opt/ficnexus/*.tmp then mv, `fichub.service` restart, `FRONTEND_DIR=/var/www/ficnexus`)





