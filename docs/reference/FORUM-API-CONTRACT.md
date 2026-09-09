# Forum API Contract — shared reference for F2+ subagents

> Ground truth for the forum routes. Every milestone subagent implements against
> this + `docs/SPEC-COMMUNITY-PLATFORM.md` (v2 — Slashdot-style moderation points +
> metamoderation + site-wide leveling) + `docs/THREADLIGHT-FEATURES.md`.

## Conventions (match existing FicHub routes — see `src/routes/requests.rs`)

- Response shape: `{"err": 0, ...}` on success; errors are `AppError`
  (`Unauthorized` → HTTP 400 with `{"err":401}`; `Forbidden` → 400 with
  `{"err":403}` — repo convention, see AGENTS.md).
- Auth: `AuthUser` extractor (`src/routes/auth.rs`). `auth.user_id: Option<i32>`,
  `auth.role: i16` (role 0/1/5/10). **NOTE: a site-wide leveling system (0-100
  levels + exp) replaces `role` in a LATER milestone (F7). Until then, use the
  existing `role >= N` checks exactly like the rest of the app — do NOT build
  role-specific abstraction; the leveling sweep will replace them.** Roles:
  0 reader, 5 curator, 10 admin (migration 007).
- All handlers: `pub async fn name(auth: AuthUser, State(state): State<Arc<AppState>>, ...) -> Result<Json<Value>, AppError>`.
- Register every route in `src/server.rs` `build_router()` (the `qa/api-walk.js`
  auto-discovers from there).
- Frontend: SvelteKit route files must be `page.test.ts`; new routes must be
  added to `frontend/src/routes/+layout.svelte` `routePages`.
- i18n via `t()` from `$lib/i18n/index.svelte`; new strings go in the
  dictionaries.

## Shared types (crate `forum-core` — generic over `ActorId`, FicHub uses i32)

See `docs/SPEC-COMMUNITY-PLATFORM.md` §4 for the full DDL. Key points:
- `forum_categories(id, slug, title, description, position, is_mod_only, created_at)`
- `forum_topics(id, category_id, author_id, title, body, payload JSONB, status, view_count, last_post_id, last_activity_at, created_at, updated_at, deleted_at, is_hidden, search_vector, topic_slug TEXT NULL)`
  - `topic_slug` (migration 047): nullable, unique via partial unique index
    `idx_forum_topics_slug ON forum_topics (topic_slug) WHERE topic_slug IS NOT NULL`.
    Canonical format `{slugified-title}-{id}`; generated in `create_topic`; **immutable**
    (title edits do NOT rewrite it). Legacy rows backfilled once at server startup
    (`backfill_topic_slugs`, best-effort, never fatal).
- `forum_posts(id, topic_id, author_id, body, payload JSONB, quote_of, edited_at, deleted_at, is_hidden, created_at, search_vector)`
- `forum_post_votes(post_id, user_id, value SMALLINT CHECK(value IN (-1,1)), created_at, PK(post_id,user_id))`
- `forum_follows(user_id, topic_id, created_at, PK(user_id,topic_id))`
- `forum_read_state(user_id, topic_id, last_read_post_id, updated_at, PK(user_id,topic_id))`
- `forum_bans(id, user_id, category_id NULL=global, reason, banned_by, expires_at, created_at)`

## Route table (all auth-gated; milestone in parens)

| Method | Path | Purpose | Milestone |
|---|---|---|---|
| GET | `/api/forum/categories` | list visible categories + topic/unread counts | F2 |
| GET | `/api/forum/topics?category={slug}&cursor={id}&limit=` | topic list by category, cursor-paginated | F2 |
| POST | `/api/forum/topics` | create topic `{title, category_slug, body, payload?}` | F3 |
| GET | `/api/forum/topics/{topicId}` | topic detail + OP + posts (`?after={postId}` cursor) | F2/F3 |
| GET | `/api/forum/topics/by-slug/{topicSlug}` | topic detail resolved by slug (same shape as `{topicId}`) | F7 |
| PATCH | `/api/forum/topics/{topicId}` | edit title/body (author ≤15min or mod) | F3 |
| DELETE | `/api/forum/topics/{topicId}` | soft-delete topic (author or mod) | F3 |
| POST | `/api/forum/topics/{topicId}/posts` | reply `{body, payload?, quote_of?}` | F3 |
| PATCH | `/api/forum/posts/{postId}` | edit post (author ≤15min or mod) | F3 |
| DELETE | `/api/forum/posts/{postId}` | soft-delete post (author or mod) | F3 |
| POST | `/api/forum/topics/{topicId}/follow` | toggle follow (returns `{following, follower_count}`) | F3 |
| GET | `/api/forum/topics/{topicId}/follow` | read my follow state + follower count | F3 |
| POST | `/api/forum/topics/{topicId}/read` | `{last_read_post_id:N}` mark read | F4 |
| GET | `/api/forum/search?q=&category={slug}` | FTS over topics+posts, snippets | F4 |
| POST | `/api/forum/categories` | create category (level ≥ admin) | F2 |
| PATCH | `/api/forum/categories/{id}` | edit/reorder category (level ≥ admin) | F2 |
| GET | `/api/forum/moderation/status` | my points, window expiry, eligibility | F5 |
| GET | `/api/forum/moderation/queue` | mods-needed posts (reported + low-score first) | F5 |
| POST | `/api/forum/posts/{postId}/moderate` | `{reason}` — server maps reason→delta, spends 1 pt | F5 |
| GET | `/api/forum/posts/{postId}/moderations` | public mod history (mirrors modlog) | F5 |
| GET | `/api/forum/metamod/queue` | N random under-rated actions, moderator anonymized | F6 |
| POST | `/api/forum/metamod/{actionId}/vote` | `{verdict: fair\|unfair\|unsure}` | F6 |
| POST | `/api/admin/forum/hide/{postId}` | level ≥ curator fast-hide, 72h auto-expiry | F5 |
| POST | `/api/admin/forum/topics/{topicId}/lock` `/pin` | level ≥ curator | F5 |
| POST | `/api/admin/forum/bans` | `{user_id, scope: forum\|category, category_id?, reason, expires_at?}` (scope ≤ own level) | F5 |
| DELETE | `/api/admin/forum/bans/{id}` | lift ban | F5 |
| GET | `/api/admin/forum/bans` | list bans (active + expired) | F5 |

**Scoped bans:** `scope=forum` blocks all forum writes; `scope=category` blocks one category.
`expires_at` NULL = indefinite ban, set = timeout. Never touches the FicHub account (no account
ban/shadowban) — read + fic features keep working. Every action → modlog with scope + reason.

**Topic slugs & board URLs:** every topic gets a `topic_slug` at creation (`{slugified-title}-{id}`,
e.g. `f3-first-topic-42`). Canonical read URL: `/forum/board/{topic_slug}.{topic_id}`.
Legacy `/forum/{categorySlug}/{topicId}` continues to work. The frontend resolves by slug first,
falling back to the numeric id when the slug is stale/unknown (renamed topic, legacy link).
`topic_slug` is `null` for legacy rows until startup backfill.

**v2 changes from v1:** NO ±1 post votes (mod points are the only score system —
`forum_posts.score` starts 0, +1 base if author exp ≥ 100, adjusted only by mod
actions). Vote endpoints are gone. Moderation/metamod endpoints are new.

## Existing wiring to reuse (exact signatures)

- Notifications: `crate::db::queries::create_notification(pool, user_id: i32, notification_type: &str, title: &str, body: Option<&str>, link: Option<&str>, reference_type: Option<&str>, reference_id: Option<&str>) -> AppResult<i64>` — `notification_type` is free text; use `forum_reply` / `forum_mention`. Routes already exist: `/api/notifications`, `/api/notifications/unread-count`, `/api/notifications/preferences` (GET/PUT).
- Modlog: `crate::modlog::record_json(db, actor_id: Option<i32>, actor_username: Option<String>, action: &str, target_type: &str, target_id: &str, extra: Vec<(&str, Value)>)` — `target_type` values `forum_topic` / `forum_post`.
- Rate limiter: `state.rate_limiter.check(ip, client_id, Tier::Default)` — see `src/limiter/mod.rs`; per-user counters via Redis INCR in-handler.
- Honeypot: `src/routes/honeypot.rs` `inspect_form(...)` — composer forms include `form_opened_at` + hidden `website` field.
- Content-scan hook: `src/services/comment_triage.rs` (Ollama classification pattern) — optional in F7.
- Reports: `user_reports` (migration 022) — F7 ALTERs the CHECK to add `forum_topic`/`forum_post`; existing `/api/admin/reports` queue handles them.
- `user_reports` target types today: `('comment','work','user')`.

## Frontend structure to mirror

- `frontend/src/lib/api/requests.ts` — API client pattern (typed functions per endpoint, `authHeaders()`).
- `frontend/src/routes/requests/` — `+page.svelte` (list), `[id]/+page.svelte` (detail, `onMount` + `$state` runes), `page.test.ts` (named exactly this).
- `frontend/src/routes/+layout.svelte` `routePages` list — add `/forum` prefixes.
- NotificationBell: `frontend/src/lib/components/NotificationBell.svelte` — polls unread-count; new notification types just appear.
- Markdown: no server renderer; client-side `marked` + `dompurify` (add to package.json deps — check `frontend/package.json`; `marked`/`dompurify` are NOT currently deps, add them in F3).

## Testing to add per milestone

- Backend: `tests/forum_api.rs` (DB-gated, mutex per suite, `--test-threads=1`); mirror `tests/requests_api.rs` helpers.
- Frontend: vitest `page.test.ts` for each new route + API client test.
- e2e: `frontend/e2e/forum.spec.ts` (Playwright, uses `helpers.ts` `registerUser` + `uniqueUser`).
- QA: routes auto-covered by `qa/api-walk.js` (it parses `src/server.rs`).
