# Rebuild Notes — Implementation Guide (junior-dev edition)

> Companion to `docs/REBUILD-NOTES.md` (the *what/why*). This file is the
> *how*: exact files, code sketches, migrations, env vars, tests, and an
> ordered work breakdown. Assume you know Rust + Axum basics and have the
> repo open (`ficnexus/`). Every magic number in the current codebase is
> listed in §3 with its env var — the site is self-hosted, so admins tune
> everything.
>
> Build order: M1 → M2 → M3 → M4 → M5 → M6 → M7. Each milestone compiles,
> passes tests, and ships behind its env toggle with defaults that preserve
> today's behavior where risky, safe-by-default where not.

---

## M1 — WriteGuard middleware (global rate/anti-abuse enforcement)

### M1.1 New tier + bucket classes

File: `src/limiter/mod.rs`. Extend the enum:

```rust
pub enum Tier {
    Download,
    Auth,
    Search,
    /// Community write endpoints (create/update/delete content).
    Write,
    /// LLM / scrape / email endpoints — expensive third-party cost.
    Expensive,
    Default,
}
```

`tier_for_path` in `src/limiter/redis_bucket.rs`: keep the mapping pure and
unit-test each prefix. New mappings (see §4 table for full route list):

```rust
pub fn tier_for_path(path: &str) -> Tier {
    // existing rules first...
    if path.starts_with("/api/search/ask")
        || path == "/api/send-to-kindle"
        || path.ends_with("/refresh")
    { return Tier::Expensive; }
    if write_content_path(path) { return Tier::Write; }
    // ...existing fallback Tier::Default
}

/// All community write endpoints. Kept as an explicit list so a new route
/// that forgets to register is CAUGHT BY THE TEST in M1.5, not in prod.
pub fn write_content_path(path: &str) -> bool {
    const WRITE_PREFIXES: &[&str] = &[
        "/api/requests", "/api/bookmarks", "/api/shelves", "/api/lists",
        "/api/collections", "/api/follows", "/api/reviews", "/api/comments",
        "/api/reactions", "/api/tags/submit", "/api/recipes", "/api/pseuds",
        "/api/skins", "/api/work-proposals", "/api/roadmap/suggest",
        "/api/reports", "/api/registration-applications", "/api/bounties",
        "/api/forum/topics", "/api/forum/posts", "/api/search/saved",
        "/api/works", // delete/translations under /api/works/*
        "/api/kindle", "/api/copyright/notice",
    ];
    WRITE_PREFIXES.iter().any(|p| path.starts_with(p))
}
```

### M1.2 WriteGuard as axum middleware

New file: `src/routes/write_guard.rs`. It runs before every handler, keyed
by BOTH IP and (when authed) user id:

```rust
use axum::{extract::{Request, State}, middleware::Next, response::{IntoResponse, Response}};
use http::StatusCode;
use std::sync::Arc;
use crate::limiter::{client_ip_from_headers, Tier, TieredRateLimitResult};
use crate::server::AppState;

/// Global guard for mutating requests. Checks the tier bucket for the
/// route; the per-user dimension uses the `Authorization` bearer id when
/// present (decoded but NOT validated here — auth extractor still runs).
pub async fn guard(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    if !matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
        return next.run(req).await;
    }
    let path = req.uri().path().to_string();
    let tier = state.rate_limiter.tier_for_path(&path);
    // GET-only Search/Default paths pass through unchanged.
    if tier == Tier::Default && !crate::limiter::write_content_path(&path) {
        return next.run(req).await;
    }

    let xff = req.headers().get("x-forwarded-for").and_then(|v| v.to_str().ok());
    let remote = req.extensions().get::<std::net::SocketAddr>().copied()
        .unwrap_or(([0,0,0,0],0).into());
    let ip = client_ip_from_headers(xff, remote.ip());
    let client_id = req.headers().get("x-client-id").and_then(|v| v.to_str().ok());

    if let TieredRateLimitResult::Wait(secs) =
        state.rate_limiter.check_tiered(ip, client_id, tier).await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(http::header::RETRY_AFTER, secs.to_string())],
        ).into_response();
    }

    // Per-user bucket (logged-in abuse survives IP rotation): key on the
    // bearer token's user id; skip when unauthenticated.
    if let Some(uid) = crate::routes::auth::peek_user_id(&req, &state).await {
        let key = format!("rate:user:{uid}:{tier:?}");
        if let Some(wait) = state.rate_limiter.check_named_bucket(
            &key, user_capacity(tier, &state.config), user_flow(tier, &state.config)
        ).await {
            return (StatusCode::TOO_MANY_REQUESTS,
                    [(http::header::RETRY_AFTER, wait.to_string())]).into_response();
        }
    }
    next.run(req).await
}
```

Implementation notes for you:
- `check_named_bucket` = reuse `RedisBucketLimiter::check_bucket` (already
  exists, make it `pub(crate)`), return `Option<u64>` (ceil of wait).
- `peek_user_id`: decode the JWT from the `Authorization: Bearer` header
  WITHOUT failing (invalid token ⇒ None; the handler's `AuthUser` extractor
  still rejects properly). It's just a bucket key.
- `user_capacity/user_flow` read §3 config (`RL_USER_WRITE_BURST`,
  `RL_USER_WRITE_PER_HOUR`, `RL_USER_EXPENSIVE_BURST`, ...).
- Mount in `src/server.rs` where `track_usage` is mounted (keep order:
  guard AFTER track_usage so blocked requests are still counted as traffic):

```rust
.layer(axum::middleware::from_fn_with_state(state.clone(), crate::routes::write_guard::guard))
.layer(axum::middleware::from_fn_with_state(state.clone(), crate::routes::analytics::track_usage))
```

### M1.3 Fail-fast route coverage (the bug-proofing test)

New file `src/routes/write_guard.rs::tests`:

```rust
#[test]
fn every_mutating_route_declares_a_tier() {
    // Build the router (same fn used by the server), walk its routes via
    // `Router::route_paths()` alternative: iterate a static list generated
    // from server.rs by a build script OR simplest: parse src/server.rs at
    // test time for `.route("PATH", ...post|put|delete(...))` tuples and
    // assert write_content_path() or an explicit exemption covers each.
}
```

Simplest robust version (do this one): read `src/server.rs` in the test,
regex-extract all write-route path strings, and assert
`write_content_path(p) || ALLOWLIST.contains(p)` — the allowlist holds
`/api/auth/*` (Auth tier already), `/api/pow/*`, `/api/admin/*` (role-gated;
still tiered via path rules — admin is a separate exemption). A new POST
route that forgets registration fails CI. This is the structural fix for
"requests board had no guard".

### M1.4 Honeypot on every user-content create form

Reuse `src/routes/honeypot.rs` verbatim — it's good. Add to the JSON bodies
of (see §4 for full list): requests create, answers, comments-on-requests,
reports, roadmap suggestions, work-proposals, registration applications,
collections create, lists/shelves create (optional), recipes create:

```rust
// serde sketch
let hp = honeypot::inspect_submission(body.website.as_deref(),
    body.form_opened_at.as_deref(), now_ms());
match hp {
    TrapVerdict::RejectSilently => return Ok(Json(json!({"err":0,"id":-1,
        "msg":"Request created"}))), // mirror register: fake success, no row
    TrapVerdict::Allow => {}
}
```

Frontend side: copy the pattern from the comment composer
(`routes/social.rs` caller side / `frontend/src/lib/...`): set
`form_opened_at = Date.now()` on mount (the requests detail page already
does for answers — wire it into the POST body), include hidden empty
`website`. Minimum time = `HONEYPOT_MIN_SUBMIT_SECS` config (default 2).

### M1.5 M1 acceptance tests

`tests/write_guard.rs` (integration, uses the real Redis with
`dynamic_rate_limit=false` ⇒ set `RL_TIERED_ENABLED=true` + in-memory Redis
or the existing test limiter harness):
1. 30 POSTs to `/api/requests` from one bucket ⇒ expect ≥1 429 with
   Retry-After.
2. Per-user bucket separate from per-IP: same user, two IPs ⇒ still 429.
3. Route-coverage unit test (M1.3) fails when a route is added bare.
4. Honeypot: POST with `website:"x"` returns 200-shaped JSON but zero rows
   (SELECT count unchanged).

---

## M2 — Config surface (kill the magic numbers)

### M2.1 Pattern to follow

`src/config.rs::Config::from_env()` already parses ~120 env vars with
defaults. Follow the exact pattern used by `rl_download_capacity` (~line
440): read, parse, unwrap_or default, store in the struct, and add the key
to `clear_config_env()` (test hygiene) and `.env.example`.

### M2.2 Complete table (add ALL of these)

Group 1 — tier buckets (defaults preserve current feel; admin can tighten):

| Env var | Default | Meaning / used by |
|---|---|---|
| RL_WRITE_CAPACITY | 20 | burst per key on Write tier |
| RL_WRITE_FLOW_PER_HOUR | 30 | steady-state refill |
| RL_EXPENSIVE_CAPACITY | 5 | burst on LLM/scrape/email endpoints |
| RL_EXPENSIVE_FLOW_PER_HOUR | 20 | refill |
| RL_USER_WRITE_BURST | 10 | per-user-id burst (Write) |
| RL_USER_WRITE_PER_HOUR | 20 | per-user-id steady state |
| RL_USER_EXPENSIVE_BURST | 3 | per-user-id burst (Expensive) |
| RL_USER_EXPENSIVE_PER_HOUR | 10 | per-user-id steady state |

Group 2 — community caps (per user):

| Env var | Default | Meaning |
|---|---|---|
| REQUESTS_PER_HOUR | 3 | fic-request creates (also see requests plan doc) |
| REQUESTS_PER_DAY | 10 | ditto |
| REQUESTS_ANSWER_PER_HOUR | 10 | answers across all requests |
| ANSWER_CAP | 3 | answers per user per request (was `pub const` in code — make env, keep const as fallback) |
| COMMENTS_PER_HOUR | 20 | work comments |
| REVIEWS_PER_HOUR | 10 | work reviews |
| REPORTS_PER_DAY | 20 | abuse reports per reporter |
| WORK_PROPOSALS_PER_DAY | 5 | proposal creates |
| ROADMAP_SUGGEST_PER_DAY | 5 | roadmap suggests |
| SHELVES_MAX | 100 | rows per user |
| LISTS_MAX | 100 | rows per user |
| COLLECTIONS_MAX | 50 | rows per user |
| BOOKMARKS_MAX | 20000 | rows per user (0 = unlimited) |
| FOLLOWS_MAX | 2000 | rows per user |
| FOLLOWS_TOGGLE_COOLDOWN_SECS | 300 | follow→unfollow→follow hysteresis |
| PSEUDS_MAX | 10 | rows per user |
| SKINS_MAX | 20 | rows per user |
| RECIPES_MAX | 50 | rows per user |
| RECIPES_CREATE_PER_DAY | 10 | create window |
| TRANSLATIONS_PER_DAY | 20 | per user |

Group 3 — anti-automation / registration:

| Env var | Default | Meaning |
|---|---|---|
| HONEYPOT_MIN_SUBMIT_SECS | 2 | timing trap floor (currently hardcoded 5s in register path — unify, keep per-route config) |
| SIGNUPS_PER_IP_PER_DAY | 3 | new |
| DISPOSABLE_EMAIL_BLOCKLIST | true | reject listed domains at register/application |
| EMAIL_VERIFY_REQUIRED | false | open-mode default; invite/application mode ignores |
| REG_APPLICATIONS_PER_IP_PER_DAY | 2 | |
| COPYRIGHT_PER_IP_PER_DAY | 5 | replaces the global-count hack |
| COPYRIGHT_GLOBAL_PER_DAY | 200 | |

Group 4 — expensive budgets:

| Env var | Default | Meaning |
|---|---|---|
| ASK_PER_USER_PER_HOUR | 20 | Ollama translate calls |
| REFRESH_PER_USER_PER_HOUR | 5 | external re-scrape triggers |
| KINDLE_PER_USER_PER_DAY | 10 | SMTP sends |
| UPLOADS_PER_USER_PER_DAY | 5 | manual EPUB/text uploads |
| EXPENSIVE_GLOBAL_PER_HOUR | 200 | circuit breaker across all clients |

Group 5 — moderation pipeline (M4):

| Env var | Default | Meaning |
|---|---|---|
| MOD_LLM_ENABLED | true | master switch |
| MOD_CHAT_MODEL | (ollama_chat_model) | classifier model |
| MOD_THRESHOLD_REQUESTS | 0.85 | flag→pending_removal confidence |
| MOD_THRESHOLD_COMMENTS | 0.85 | |
| MOD_THRESHOLD_FORUM | 0.85 | |
| MOD_GRACE_HOURS_REQUESTS | 72 | |
| MOD_GRACE_HOURS_COMMENTS | 72 | |
| MOD_GRACE_HOURS_FORUM | 72 | |
| MOD_MODE_REQUESTS | single | single\|quorum\|admin |
| MOD_MODE_COMMENTS | single | |
| MOD_MODE_FORUM | single | |
| MOD_QUORUM_N | 2 | votes to resolve in quorum mode |
| MOD_TRUST_GATE_REQUESTS | false | require trust_level≥N to create |
| MOD_TRUST_GATE_MIN_LEVEL | 1 | |
| REQUESTS_DUPLICATE_WINDOW_DAYS | 7 | exact-title dupe block |

Group 6 — retention:

| Env var | Default | Meaning |
|---|---|---|
| NOTIF_RETENTION_DAYS | 180 | prune read notifications |
| NOTIF_COALESCE_WINDOW_MINS | 10 | dedupe identical notif rows |
| NOTIF_MAX_PER_USER | 1000 | cap queue depth |
| REQUEST_LOG_RETENTION_DAYS | 90 | |
| AP_INBOX_LOG_RETENTION_DAYS | 30 | |
| RETENTION_RUN_INTERVAL_HOURS | 24 | |

Group 7 — headers/security:

| Env var | Default | Meaning |
|---|---|---|
| CORS_ORIGINS | self | comma list, "" ⇒ same-origin |
| CSP_EXTRA_CONNECT | "" | appended to CSP connect-src |
| SKIN_CSS_ALLOW_REMOTE | false | strip remote url() from skins |
| HSTS_MAX_AGE_SECS | 31536000 | 0 disables |
| ADMIN_ALERT_QUEUE_THRESHOLD | 50 | flag-queue metric alert |

### M2.3 Where each is enforced (code map)

- buckets → M1 WriteGuard (`user_capacity/user_flow`).
- caps → new helper `src/services/community_limits.rs`:

```rust
/// Returns Ok(()) or Err(429-with-retry-after / 409-with-existing-id).
pub async fn assert_row_cap(db: &PgPool, user_id: i32, table: &str,
    max: i64) -> Result<(), AppError> {
    if max <= 0 { return Ok(()); }
    let q = format!("SELECT COUNT(*) FROM {table} WHERE user_id=$1");
    let (n,): (i64,) = sqlx::query_as(&q).bind(user_id).fetch_one(db).await?;
    if n >= max { return Err(AppError::TooManyRequests(
        format!("limit reached: {max}"))); }
    Ok(())
}
```
  (`table` comes from an internal enum, never user input — no injection.)
  Call from each create handler (lists the §4 table), 1–3 lines per route.
- windows (per-hour/day) → same helper with a time predicate:
  `assert_window(db, user_id, table, n, "1 hour")`.
- budgets → `check_budget(state, uid, "ask", per_hour)` uses Redis INCR +
  EXPIRE (cheap) instead of a DB count.

---

## M3 — Expensive-endpoint budgets

### M3.1 Ask (`/api/search/ask`)

`src/search/ask.rs::ask_handler` currently rate-limits at Search tier only.
Change: (a) map it to `Tier::Expensive` in `tier_for_path`; (b) after the
tier check add:

```rust
if let Some(uid) = auth.user_id {
    crate::services::community_limits::check_budget(
        &state.redis, uid, "ask", state.config.ask_per_user_per_hour).await?;
}
crate::services::community_limits::check_global_budget(
    &state.redis, "expensive", state.config.expensive_global_per_hour).await?;
```

`check_budget` = Redis key `budget:{name}:{uid}:{yyyyMMddHH}` INCR, EXPIRE 2h,
429 with Retry-After when over.

### M3.2 Refresh + answer-URL scrape + upload

Same two-liners in `src/routes/updates.rs::refresh_fic_handler`, the URL
branch of `src/routes/requests.rs::add_answer`, and
`src/routes/upload.rs::handle_manual_upload` (budget name `refresh`,
`answer-scrape` folded into Write, `upload` per-day).

### M3.3 Kindle

`src/routes/kindle.rs::send_to_kindle_handler`: budget `kindle` per user/day.
Also validate recipient domain against kindle.email allowlist — prevents
using the relay to mail arbitrary addresses:
`[cfg: KINDLE_DOMAINS=@kindle.com,@free.kindle.com]`.

### M3.4 Copyright fix

`src/routes/copyright.rs::submit_notice`: replace the single global
`SELECT COUNT(*) ... 1 day` with per-IP + per-user (when authed) counters
in Redis (same check_budget helper, key by IP) and keep a global cap. PoW:
reuse `/api/pow/challenge` — require a solved token for anonymous notices
(like exports for shadowbanned clients).

---

## M4 — Unified moderation pipeline

### M4.1 Migration `migrations/074_moderation_targets.sql`

```sql
CREATE TABLE IF NOT EXISTS moderation_targets (
  target_type text NOT NULL,       -- 'request'|'answer'|'comment'|'forum_post'|'review'|'translation'|'upload'
  target_id   text NOT NULL,       -- id as text (different pk types)
  author_id   integer REFERENCES users(id),
  mod_status  text NOT NULL DEFAULT 'clean'
    CHECK (mod_status IN ('clean','flagged','pending_removal','removed')),
  classification text,             -- spam|off_topic|low_effort|policy_violation|...
  confidence  real,
  reason      text,
  flagged_by  text NOT NULL DEFAULT 'llm',  -- 'llm'|user id|'auto'
  removal_scheduled_at timestamptz,
  reviewed_by integer REFERENCES users(id),
  reviewed_at timestamptz,
  review_decision text CHECK (review_decision IN ('keep','remove') OR review_decision IS NULL),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (target_type, target_id)
);
CREATE INDEX IF NOT EXISTS idx_mod_queue
  ON moderation_targets (mod_status, removal_scheduled_at NULLS FIRST, created_at DESC)
  WHERE mod_status IN ('flagged','pending_removal');
CREATE INDEX IF NOT EXISTS idx_mod_author ON moderation_targets (author_id);

-- quorum votes (used when MOD_MODE_<type>=quorum; harmless in single mode)
CREATE TABLE IF NOT EXISTS moderation_votes (
  target_type text NOT NULL,
  target_id   text NOT NULL,
  curator_id  integer NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  decision    text NOT NULL CHECK (decision IN ('keep','remove')),
  created_at  timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (target_type, target_id, curator_id)
);
```

**Why a separate table instead of per-feature columns** (the requests plan
used columns): one queue UI/endpoint/retention/test covers every type; a
new content feature is 1 enum string + 1 classify prompt. The target rows
mirror the feature's own state (`removed` ⇒ the feature sets its own
`deleted_at` via a callback map below).

### M4.2 Service `src/services/moderation.rs`

```rust
pub enum TargetType { Request, Answer, Comment, ForumPost, Review, Translation, Upload }
impl TargetType { pub fn as_str(&self)..; pub fn grace_hours(&self, cfg)..; pub fn threshold(&self, cfg)..;
                  pub fn mode(&self, cfg)..; }

/// Fire-and-forget classify. Ollama down ⇒ no-op (fail-open, rate limits
/// remain). Prompt per type is a const template in this file.
pub async fn classify_and_flag(db,&ollama,cfg, ttype: TargetType, tid: &str,
    author: Option<i32>, text: &str) { /* INSERT ... ON CONFLICT DO NOTHING first */ }

/// Parse `<classification> | <confidence> | <reason>` — same parser as
/// content_scan::parse_scan_response (extract that to moderation::parse_line).

pub async fn resolve(db,cfg, actor: &CuratorActor, ttype, tid, decision) -> Result<()> {
    // single mode: write immediately.
    // quorum mode: insert vote; if keep-count>=N → keep; remove-count>=N → remove.
    // remove ⇒ apply_removal callback; keep ⇒ clear scheduled + status clean.
}

pub async fn apply_removal(db, ttype, tid) {
    // dispatch table: Request/Answer → UPDATE fic_requests/answers SET deleted_at=NOW()
    // Comment → UPDATE comments SET deleted_at; ForumPost → posts hide;
    // Translation → status rejected; Upload → works.is_visible=false.
    // Then modlog::record(actor "auto-moderator" or curator name, "moderation_remove", ttype, tid)
}

/// Hourly cron entry point — mirrors content_scan::process_expired_deletions.
pub async fn process_expired(db, cfg) -> usize {
    // SELECT ... WHERE mod_status='pending_removal' AND removal_scheduled_at<=NOW() LIMIT 50
    // for each: apply_removal + mark reviewed/removed.
}
```

Wire the cron in `src/server.rs` beside the existing auto-delete loop:

```rust
{
    let state = state.clone();
    tokio::spawn(async move {
        loop {
            let n = crate::services::moderation::process_expired(&state.db, &state.config).await;
            if n > 0 { tracing::info!("moderation cron: {n} expired targets removed"); }
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    });
}
```

Hook creation paths (spawn like the Archivist already does in requests.rs):
requests create, add_answer, comments post (social.rs + comments.rs both —
they're two comment systems; moderate both, flag `comments` and
`fic_request_answers`), forum create_post + create_topic (moderation_points
already exist; call classify in addition), reviews upsert, translations
upsert, uploads (the content_scan nightly already covers bodies — fold its
result in as target_type='upload').

### M4.3 Curator queue API (`src/routes/moderation.rs`)

```
GET  /api/curator/moderation?target_type=&status=&page=   role>=5
POST /api/curator/moderation/{type}/{id}/confirm          remove now (bypass grace)
POST /api/curator/moderation/{type}/{id}/dismiss          keep, clear flag
GET  /api/curator/moderation/{type}/{id}/votes            quorum breakdown
```

All write decisions modlog (`record`/`record_json`) — you get the one
filterable modlog page (product rule) for free. Add `target_type=request`
etc. to the existing modlog filter UI (frontend `modlog` page — one
dropdown entry per type).

### M4.4 Frontend

- `frontend/src/routes/curator/moderation/+page.svelte` — table (reuse
  forum `/forum/moderate` pattern): type filter chips, confidence, snippet,
  Confirm/Dismiss buttons, countdown for pending_removal.
- Content banners: where a target is flagged, list/detail pages show
  `Flagged — removal in {h}h` to owner+curators only (API field
  `moderation: {status, removal_scheduled_at} | null` — never to the public;
  public sees content vanish at expiry, not "flagged" stigma beforehand).
  `[cfg: MOD_SHOW_BANNER_PUBLIC=false]`
- page.test.ts per route (skill rule: `page.test.ts` naming).

---

## M5 — Security headers + CORS + skin sanitize

1. `src/server.rs`: replace `CorsLayer::permissive()` with
   `CorsLayer::new().allow_origin(cors_origins).allow_methods(...).allow_headers(...)`
   from `CORS_ORIGINS` (parse: `self` ⇒ reflection off Origin header).
2. New tower middleware `src/frontend/security_headers.rs`: append on every
   response: `Content-Security-Policy` (default-src 'self'; script-src
   'self'; style-src 'self' 'unsafe-inline' — Svelte needs inline styles;
   img-src 'self' data: https:; connect-src 'self' {CSP_EXTRA_CONNECT};
   frame-ancestors 'none'), `X-Frame-Options: DENY`,
   `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`,
   `Strict-Transport-Security` when `HSTS_MAX_AGE_SECS>0`.
3. Skins (`src/routes/skins.rs`): sanitize CSS before store — strip
   `expression(`, `@import`, and (unless `SKIN_CSS_ALLOW_REMOTE=true`) any
   `url(http...`. Tiny regex-lite pass is fine — skins are a trusted-ish
   but admin-visible surface; log every rejection in modlog.

---

## M6 — Auth hardening stage (cookie migration)

Largest change; do LAST, behind a flag.

1. `/api/auth/login` + `/refresh` gain `Set-Cookie: fichub_session=<jwt>;
   HttpOnly; SameSite=Lax; Secure(auto); Max-Age=...` when
   `AUTH_COOKIE_MODE=true` (keep localStorage bearer during transition).
2. `AuthUser` extractor reads cookie-first, bearer-fallback.
3. CSRF: when cookie mode, require `X-FicNexus-CSRF` header matching a
   double-submit cookie (`fichub_csrf`, non-HttpOnly) on mutating verbs.
   Middleware next to write_guard.
4. Refresh rate-limit: move `/api/auth/refresh` into the Auth tier mapping
   (one line in tier_for_path: `/api/auth/` already matches — verify; the
   incident is that the *handler* bypasses `enforce_auth_rate_limit`, which
   is social.rs-local — the global guard now covers it, which is the
   argument for M1 over per-handler calls).
5. Device table (`user_sessions(id,user_id,ua,ip_prefix,created,expires,revoked)`)
   + `GET /api/sessions`, `POST /api/sessions/revoke-all`. ip_prefix = /48
   v6 or /24 v4, never full IP (Zero-PII convention from limiter/mod.rs).
6. Site-credentials at-rest encryption: key from `CREDS_ENC_KEY` (file mode)
   or future KMS; AES-256-GCM via the `aes-gcm` crate; mask GET, modlog
   audit on every scrape that consumes creds.

Keep bearer mode working until cookie mode defaults true; then remove
localStorage token in a follow-up (bump `sw.js` VERSION then — PWA caches).

---

## M7 — Retention crons + metrics

New `src/services/retention.rs`, one function per table:

```rust
pub async fn prune_all(db, cfg) -> usize {
    let mut n = 0;
    n += sqlx::query("DELETE FROM notifications WHERE created_at < NOW() - ($1 || ' days')::interval AND NOT unread")...;
    n += sqlx::query("DELETE FROM ap_inbox_log WHERE created_at < NOW() - ($1 || ' days')::interval")...;
    n += sqlx::query("DELETE FROM request_log WHERE created > NOW() - ($1 || ' days')::interval")...;
    n += sqlx::query("DELETE FROM invites WHERE used_at IS NULL AND expires_at < NOW()")...;
    n
}
```

Daily loop in server.rs (same spawn pattern). Metrics: extend
`/api/admin/stats` (routes/admin.rs) with counters
(`blocked_429_<tier>_24h`, `flagged_24h`, `auto_removed_24h`,
`dismissed_24h`) from Redis INCRs in write_guard/moderation. Alert threshold
`ADMIN_ALERT_QUEUE_THRESHOLD` surfaces as a header on /api/health and a
warning in the digest.

---

## 4. Route → guard map (implementation checklist — tick in PR)

Write routes needing M1 tier + M2 cap + M4 hook. (Generated from
`grep routing::post server.rs`; verify count against
write_guard.rs ALLOWLIST.)

| Route | Guard | Cap/env | Mod hook | Notes |
|---|---|---|---|---|
| POST /api/requests | Write + user | REQUESTS_PER_HOUR/DAY | Request | + honeypot + dupe window |
| POST /api/requests/{id}/answers | Write + user | REQUESTS_ANSWER_PER_HOUR, ANSWER_CAP | Answer | URL branch → Expensive |
| POST /api/requests/{id}/answers/{aid}/vote | Write | vote bucket | — | UNIQUE pk exists |
| POST /api/requests/{id}/upvote | Write | vote bucket | — | |
| POST /api/comments, /api/works/{url}/comments | Write | COMMENTS_PER_HOUR | Comment | comment honeypot exists → unify |
| POST /api/reviews | Write | REVIEWS_PER_HOUR | Review | |
| POST /api/kudos | Write | — | — | UNIQUE good |
| POST /api/bookmarks, import | Write | BOOKMARKS_MAX | — | import → Queue tier (already worker-gated) |
| POST /api/shelves, /api/lists, /api/collections | Write | SHELVES_MAX, LISTS_MAX, COLLECTIONS_MAX | — | |
| POST /api/follows (+exclusions) | Write | FOLLOWS_MAX + cooldown | — | cooldown via `assert_follow_cooldown` (Redis SETNX ttl) |
| POST /api/forum/topics, posts | Write | window caps | ForumPost | mod points exist already |
| POST /api/tags/submit, vote, flag | Write | TAG_SUBMIT/VOTE_LIMIT exist | — | wire config that's parsed but unused? verify |
| POST /api/recipes* | Write | RECIPES_MAX + per-day | — | publish trust gate stays |
| POST /api/pseuds, /api/skins | Write | PSEUDS_MAX, SKINS_MAX | — | skins M5.3 |
| POST /api/work-proposals, vote | Write | WORK_PROPOSALS_PER_DAY | — | |
| POST /api/roadmap/suggest, vote | Write + Expensive | ROADMAP_SUGGEST_PER_DAY | — | suggests hit Ollama embeddings? budget it |
| POST /api/reports | Write | REPORTS_PER_DAY | — | |
| POST /api/registration-applications | Auth tier | REG_APPLICATIONS_PER_IP_PER_DAY | — | + honeypot |
| POST /api/bounties, claim | Write | BOUNTIES_MAX_OPEN | — | verify escrow server-side |
| POST /api/send-to-kindle | Expensive | KINDLE_PER_USER_PER_DAY | — | + domain allowlist |
| POST /api/v1/works/{id}/refresh | Expensive | REFRESH_PER_USER_PER_HOUR | — | |
| POST /api/upload | Expensive/Queue | UPLOADS_PER_DAY | Upload | honeypot exists |
| POST /api/copyright/notice | Expensive + PoW | COPYRIGHT_PER_IP_PER_DAY | — | fix global-count hack |
| POST /inbox, /actor/inbox | Write (per actor) | AP_INBOX_PER_ACTOR_PER_MIN | — | signature ok |
| POST/PUT /api/translations | Write | TRANSLATIONS_PER_DAY | Translation | curator approval already exists — verify server-side |
| DELETE request/answer/comment/review/bookmark | Write | — | — | M4 callbacks already cover |
| POST /api/auth/register, login | Auth | SIGNUPS_PER_IP_PER_DAY (register) | — | honeypot exists |

## 5. Env template (append block to .env.example)

```
# ── Community limits (per-instance) ─────────────────────────────
# Rate buckets (Write/Expensive tiers). Burst + steady-state/hr.
RL_WRITE_CAPACITY=20
RL_WRITE_FLOW_PER_HOUR=30
RL_EXPENSIVE_CAPACITY=5
RL_EXPENSIVE_FLOW_PER_HOUR=20
# Per-user-id buckets (logged-in abuse; IP rotation doesn't help them)
RL_USER_WRITE_BURST=10
RL_USER_WRITE_PER_HOUR=20
RL_USER_EXPENSIVE_BURST=3
RL_USER_EXPENSIVE_PER_HOUR=10
# Community row caps / windows (0 = unlimited)
REQUESTS_PER_HOUR=3
REQUESTS_PER_DAY=10
REQUESTS_ANSWER_PER_HOUR=10
ANSWER_CAP=3
COMMENTS_PER_HOUR=20
REVIEWS_PER_HOUR=10
REPORTS_PER_DAY=20
WORK_PROPOSALS_PER_DAY=5
ROADMAP_SUGGEST_PER_DAY=5
SHELVES_MAX=100
LISTS_MAX=100
COLLECTIONS_MAX=50
BOOKMARKS_MAX=20000
FOLLOWS_MAX=2000
FOLLOWS_TOGGLE_COOLDOWN_SECS=300
PSEUDS_MAX=10
SKINS_MAX=20
RECIPES_MAX=50
RECIPES_CREATE_PER_DAY=10
TRANSLATIONS_PER_DAY=20
# Registration / anti-automation
HONEYPOT_MIN_SUBMIT_SECS=2
SIGNUPS_PER_IP_PER_DAY=3
DISPOSABLE_EMAIL_BLOCKLIST=true
EMAIL_VERIFY_REQUIRED=false
REG_APPLICATIONS_PER_IP_PER_DAY=2
COPYRIGHT_PER_IP_PER_DAY=5
COPYRIGHT_GLOBAL_PER_DAY=200
# Expensive budgets
ASK_PER_USER_PER_HOUR=20
REFRESH_PER_USER_PER_HOUR=5
KINDLE_PER_USER_PER_DAY=10
KINDLE_DOMAINS=@kindle.com,@free.kindle.com
UPLOADS_PER_USER_PER_DAY=5
EXPENSIVE_GLOBAL_PER_HOUR=200
# Moderation pipeline
MOD_LLM_ENABLED=true
MOD_THRESHOLD_REQUESTS=0.85
MOD_THRESHOLD_COMMENTS=0.85
MOD_THRESHOLD_FORUM=0.85
MOD_GRACE_HOURS_REQUESTS=72
MOD_GRACE_HOURS_COMMENTS=72
MOD_GRACE_HOURS_FORUM=72
MOD_MODE_REQUESTS=single
MOD_MODE_COMMENTS=single
MOD_MODE_FORUM=single
MOD_QUORUM_N=2
MOD_TRUST_GATE_REQUESTS=false
MOD_TRUST_GATE_MIN_LEVEL=1
REQUESTS_DUPLICATE_WINDOW_DAYS=7
MOD_SHOW_BANNER_PUBLIC=false
# Retention
NOTIF_RETENTION_DAYS=180
NOTIF_COALESCE_WINDOW_MINS=10
NOTIF_MAX_PER_USER=1000
REQUEST_LOG_RETENTION_DAYS=90
AP_INBOX_LOG_RETENTION_DAYS=30
RETENTION_RUN_INTERVAL_HOURS=24
# Security headers / CORS
CORS_ORIGINS=self
CSP_EXTRA_CONNECT=
SKIN_CSS_ALLOW_REMOTE=false
HSTS_MAX_AGE_SECS=31536000
ADMIN_ALERT_QUEUE_THRESHOLD=50
# Auth stage 2 (M6)
AUTH_COOKIE_MODE=false
```

## 6. Gotchas (repo-specific — read before coding)

- **sqlx offline/compile-time:** any new `sqlx::query!` macro needs the DB
  reachable or `cargo sqlx prepare`; the codebase mostly uses runtime
  `sqlx::query_as::<_,(T,)>` — keep that style for new queries.
- **regex-lite has NO backreferences/lookaround** — parse classifier output
  with `splitn` like `content_scan::parse_scan_response`, never fancy regex.
- **Migrations:** append-only from 074 up; never edit 001_initial.sql; after
  local testing on prod use the skill procedure (delete stale
  `_sqlx_migrations` rows, never hand-INSERT with fake checksums).
- **page.test.ts** naming for SvelteKit route tests (not `+page.test.ts`).
- **i18n:** every new UI string (banner text, queue labels) goes into all 6
  dictionaries (`frontend/src/lib/i18n/dictionaries/*.ts`) in lockstep —
  `sed` bulk edits double commas, grep for `,,` after.
- **Honeypot:** silent-reject must mirror the success response shape or bots
  learn (register returns `{"err":0,...}` fake). For requests: return
  `{"err":0,"id":<negative or real-ish?>}` — use a real-looking id of -1 and
  the frontend redirects to /requests (it already navigates on err=0).
- **Rate limiter test-mode:** `dynamic_rate_limit=false` returns Allowed
  always (static delay). Integration tests for buckets must force
  `RL_TIERED_ENABLED=true` + `DYNAMIC_RATE_LIMIT=true` + a Redis the test
  can flush (`rate:*` keys pattern), or use the existing limiter harness
  (`src/limiter/redis_bucket.rs` unit tests use pure buckets).
- **Zero-PII rule:** ip_prefix only (see `client_ip_from_headers` doc),
  never store raw IPs in new tables (notifications/metrics).
- **429 envelope:** return JSON `{"err":-20,"msg":"rate limited","retry_after":N}`
  with Retry-After header so the SPA toast shows it (api client throws on
  non-2xx — check api/* clients handle retry_after text gracefully).
- **Deploy:** each milestone merges independently; M1+M2+M3 are backend +
  env only; M4 adds tables + one curator page; M5 is header-only (verify
  the SPA still loads — CSP `style-src 'unsafe-inline'` stays); M6 needs
  a staging pass; M7 is ops-only.

## 7. Milestone definitions of done

- **M1:** boot test fails on unrouted write path; 100-route audit PR ticks
  §4; integration 429 test green; no behavior change for real users (verify
  with the weekly-digest stats: 429 count < 0.1% of traffic post-deploy).
- **M2:** every §5 var parsed + round-trip-tested in `config.rs` tests
  (mirror `test_rec_site_rate_limits_*` pattern); `.env.example` updated;
  `clear_config_env()` extended.
- **M3:** budgets green on ask/refresh/kindle/copyright; copyright fix has a
  regression test that one IP can't exhaust the global budget.
- **M4:** requests + comments + forum posts all flow through
  moderation_targets; one queue page; cron removes expired; curator
  confirm/dismiss modlogged; Ollama-down path proven by
  `MOD_LLM_ENABLED=false` test.
- **M5:** securityheaders.com A-grade on prod; skins with `@import` rejected;
  CORS rejects `evil.com` origin.
- **M6:** cookie auth behind flag green in staging; bearer fallback still
  passes the whole api_contract_tests suite.
- **M7:** nightly prune visible in logs; admin stats counters populated;
  queue-depth warning wired to health endpoint.
