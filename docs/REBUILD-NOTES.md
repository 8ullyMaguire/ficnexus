# Rebuild Notes — What I'd Change Building FicNexus From Scratch

> Hardened-architecture audit written after finding the second spammable
> endpoint (fic requests, 2026-08-31; the first was the forum 401-bounce).
> Everything below is grounded in the current codebase. Pair with:
> - `docs/plans/requests-spam-guardrails.md` — the near-term requests fix
> - `docs/plans/rebuild-notes-implementation.md` — junior-dev implementation
>   guide for everything in this file (files, code sketches, env vars, tests)
>
> **Config rule:** this is self-hosted software — every limit, threshold,
> window and toggle below is an env var with a sane default. Nothing is a
> magic number in code. Each item is tagged `[cfg: VAR=default …]`; the full
> config surface is table §9.

## 0. The root pattern behind both incidents

Enforcement is **opt-in per handler**. The tiered limiter, honeypot, and
moderation queue all exist, but every route family decides whether to call
them. export.rs / social.rs / upload / ask / fic_suggestions / tags do;
requests, bookmarks, shelves, lists, follows, reviews, recipes, pseuds,
skins, work-proposals, roadmap, translations do not. server.rs mounts ~150
write routes with only `track_usage` + `TraceLayer` + permissive CORS.

**Rule #1: protection is middleware, not a library call.** All POST/PUT/DELETE
traffic passes a router-level WriteGuard that requires the route to declare a
class. Routes without a declared class fail closed at boot (test asserts it).

## 1. Cross-cutting architecture

- **Global middleware stack:** request-id → body-size limit → WriteGuard
  (rate/anti-abuse) → auth extractor → security headers → trace. Today only
  `analytics::track_usage` is global (server.rs ~1070).
  `[cfg: MAX_BODY_BYTES=2MiB default; per-route overrides]`
- **Rate-limit tiers:** beyond Download/Auth/Search/Default add `Write`
  (community creates), `Expensive` (LLM/scrape/email), `Queue` (enqueueing
  work). Bucket = per-IP **and** per-user-id (today only IP + rotatable
  client_id). `[cfg: RL_WRITE_CAPACITY=20, RL_WRITE_FLOW=30/h, RL_EXPENSIVE_CAPACITY=5, RL_EXPENSIVE_FLOW=20/h, RL_NAT_MULTIPLIER=4]`
- **Expensive-endpoint budgets:** Ollama (`search/ask.rs` is Search tier —
  1000/min of LLM per IP today!), scrapes (`updates.rs::refresh_fic_handler`
  — unlimited re-scrape; `requests.rs::add_answer` URL path), SMTP
  (`kindle.rs` — no per-user cap → relay abuse). Per-user/day budget +
  global circuit breaker, fail closed 429+Retry-After.
  `[cfg: ASK_PER_USER_PER_HOUR=20, REFRESH_PER_USER_PER_HOUR=5, KINDLE_PER_USER_PER_DAY=10, EXPENSIVE_GLOBAL_CIRCUIT_BREAK_PER_HOUR=200]`
- **Security headers layer (missing):** CSP, X-Frame-Options DENY, HSTS,
  Referrer-Policy, X-Content-Type-Options. User skins (skins.rs) are
  arbitrary CSS → sanitize (url() allowlist) or sandbox origin.
  `[cfg: CSP_EXTRA_SRC="", SKIN_CSS_ALLOW_REMOTE=false]`
- **CORS:** `CorsLayer::permissive()` → lock to deployed origins.
  `[cfg: CORS_ORIGINS="self"]`
- **Error envelope:** 429 always carries `retry_after`; 409 carries the
  conflicting resource id; single `AppError` mapper. `[cfg: none]`
- **Idempotency keys** on create-type writes (client `Idempotency-Key`
  header, server dedupe window). `[cfg: IDEMPOTENCY_TTL_MINS=30]`

## 2. Auth & accounts

- **Token storage:** JWT in localStorage (api/social.ts) is XSS-stealable →
  httpOnly SameSite=Lax cookie + CSRF token, or in-memory access token +
  rotating refresh cookie. Background requests must never navigate (the 401
  incident); codify as a test. `[cfg: TOKEN_TTL_DAYS=30, REFRESH_TTL_DAYS=90, COOKIE_SECURE=auto]`
- **argon2id** with yearly cost re-eval; rehash-on-login for legacy hashes.
- **Registration:** keep honeypot + invite/application modes; add per-IP
  signup cap, disposable-email list, email verification before any write.
  `[cfg: SIGNUPS_PER_IP_PER_DAY=3, DISPOSABLE_EMAIL_BLOCKLIST=true, EMAIL_VERIFY_REQUIRED=false (true for open mode), HONEYPOT_MIN_SUBMIT_SECS=2]`
- **Session lifecycle:** wrap `/api/auth/refresh` in the Auth-tier limiter;
  device list + revoke-all. `[cfg: SESSIONS_PER_USER_MAX=10]`
- **Site credentials** (user_credentials.rs): encrypt at rest (KMS-backed
  key), mask on read, audit reads. `[cfg: CREDS_ENC_KEY=file|env|kms]`

## 3. Anti-abuse defaults per write route

Route → guards (details + sketches in the implementation doc):

| Route family | Today | From scratch | Config |
|---|---|---|---|
| `/api/requests` | nothing | 3/h+10/d, honeypot, dupe window, LLM flag+grace | REQUESTS_PER_HOUR=3, REQUESTS_PER_DAY=10, REQUESTS_GRACE_HOURS=72, REQUESTS_LLM_THRESHOLD=0.85, REQUESTS_TRUST_GATE=false, REQUESTS_DUPLICATE_WINDOW_DAYS=7 |
| `/api/requests/{id}/answers` | cap 3/req, scrape on URL | per-user/hour cap + scrape budget | ANSWER_CAP=3, REQUESTS_ANSWER_PER_HOUR=10 |
| bookmarks/shelves/lists/collections | free row spam | count caps + create bucket | SHELVES_MAX_PER_USER=100, LISTS_MAX_PER_USER=100, COLLECTIONS_MAX_PER_USER=50, BOOKMARKS_MAX_PER_USER=20000, WRITE_BUCKET=Write tier |
| follows | unbounded toggle | hysteresis + cap + notif coalescing | FOLLOWS_MAX_PER_USER=2000, FOLLOWS_TOGGLE_COOLDOWN_MINS=5 |
| reviews/comments | comment honeypot only | write bucket + dupe + LLM flag | REVIEWS_PER_USER_PER_HOUR=10, COMMENTS_PER_USER_PER_HOUR=20, COMMENTS_DUPLICATE_WINDOW_MINS=60 |
| `/api/reports` | min-trust 1 | daily cap per user | REPORTS_PER_USER_PER_DAY=20 |
| work-proposals + vote | none | bucket + vote limits + dupe | WORK_PROPOSALS_PER_DAY=5, PROPOSAL_VOTES_PER_DAY=50 |
| `/api/roadmap/suggest` | none | bucket + LLM budget + dupe | ROADMAP_SUGGEST_PER_DAY=5 |
| recipes | publish trust gate only | row cap + bucket | RECIPES_MAX_PER_USER=50, RECIPES_PER_DAY=10 |
| pseuds/skins | none | count caps + CSS sanitize | PSEUDS_MAX_PER_USER=10, SKINS_MAX_PER_USER=20 |
| `/api/translations*` | auth only | write bucket; curator approval server-enforced | TRANSLATIONS_PER_USER_PER_DAY=20 |
| registration-applications | none | IP cap + honeypot + dupe | REG_APPLICATIONS_PER_IP_PER_DAY=2 |
| ActivityPub `/inbox` | signature OK | per-actor bucket + log retention | AP_INBOX_PER_ACTOR_PER_MIN=30, AP_INBOX_LOG_RETENTION_DAYS=30 |
| kudos/ratings | UNIQUE (good) | brigade detection | KUDOS_BRIGADE_WINDOW_MINS=10 |
| bounties | verify escrow | escrowed rep required, zero-sum cancel | BOUNTIES_MAX_OPEN_PER_USER=5 |

- **copyright.rs:** "rate limit" is a single global 1-day count — one
  attacker exhausts the world's DMCA budget. Per-IP + per-user caps + PoW
  for the public form. `[cfg: COPYRIGHT_PER_IP_PER_DAY=5, COPYRIGHT_GLOBAL_PER_DAY=200]`
- **Notification rows:** coalesce per (user,type,reference) in a window;
  retention pruning (today unbounded growth).
  `[cfg: NOTIF_COALESCE_WINDOW_MINS=10, NOTIF_RETENTION_DAYS=180, NOTIF_MAX_PER_USER=1000]`

## 4. One moderation pipeline, not per-feature

content_scan (070) got the shape right: classify → grace → cron →
confirm/dismiss → modlog. From scratch it is ONE service for every content
type (works, requests, comments, forum posts, reviews, translations,
uploads): `moderation_targets` table, one queue UI
(`/curator/moderation?target_type=`), one confirm/dismiss/vote endpoint set,
one modlog convention. Consensus mode (single-curator / quorum-N /
admin-only) is per-type config, not code forks.
`[cfg: MOD_MODE_<TYPE>=single|quorum|admin (default single), MOD_QUORUM_N=2, MOD_GRACE_HOURS_<TYPE> (default 72), MOD_THRESHOLD_<TYPE> (default 0.85), MOD_LLM_ENABLED=true]`

## 5. Data model integrity

- UNIQUE constraints as dedupe documentation on every join/flag table.
- One soft-delete convention (`deleted_at` + moderation state) with a shared
  helper, not per-handler SQL.
- Retention jobs from day one: notifications, ap_inbox_log, request_log,
  analytics aggregates, expired invites, dead queue entries.
  `[cfg: RETENTION_RUN_INTERVAL_HOURS=24, REQUEST_LOG_RETENTION_DAYS=90]`
- Migrations append-only, versioned like code; `migrate` stays an explicit
  deploy step (migrate.rs), server never boot-migrates in prod
  (FICHUB_SKIP_MIGRATIONS=1 in service env).

## 6. Frontend

- Auth store: never navigate on background 401 (test-enforced); polling
  components check `isLoggedIn` before first call (ForumBottomNav lesson).
- Build on local disk in CI, ship only `build/` (the /tmp+rsync dance is an
  NFS workaround, not a design).
- i18n: keep 6-dictionary lockstep + CI drift check (catches the `,,` bug
  class). `[cfg: LOCALES=en,de,es,fr,pt-BR,zh]`
- PWA: generate sw.js VERSION from git sha; strategies single-sourced.

## 7. Testing & QA

- **Authz matrix tests:** every write route × (anon/new/trusted/curator/admin)
  × (valid/spammy) asserting status + side effects.
- **Spam simulation:** N writes per family → assert bucket behavior.
- **Boot-time route-class coverage test:** every write route maps to a
  WriteGuard class or the test fails.
- Contract tests extended with 429/Retry-After and 409-dupes expectations.

## 8. Ops

- Keep explicit migrate + health-gated swap; add auto-rollback on 5xx spike.
- Per-endpoint 429/409/budget metrics in admin stats; flag-queue-size alert
  (queue depth = spam-campaign canary).
  `[cfg: ADMIN_ALERT_QUEUE_THRESHOLD=50]`
- Docs: this file + plans/ live in-repo; `brainstorm-02-roadmap-status.md`
  stays the only status log.

## 9. Full per-instance config surface (summary)

Every number in this doc is an env var — see the implementation doc §3 for
the complete table with defaults, types and where each is parsed
(`Config::from_env` + `.env.example`). Admin-tunable axes, grouped:

1. **Rate limiting** — per-tier capacities/flows, NAT multiplier, shadowban
   params, per-family per-user buckets.
2. **Community limits** — max rows per user (shelves/lists/skins/...),
   create windows, toggle cooldowns, follow caps.
3. **Moderation** — per-type mode (single/quorum/admin), quorum N, grace
   hours, LLM threshold, LLM on/off, trust gates.
4. **Expensive budgets** — ask/refresh/kindle/upload per-user windows +
   global circuit breakers.
5. **Registration/auth** — signup caps, email verification, token TTLs,
   honeypot timing.
6. **Retention** — notifications, logs, inbox, analytics windows.
7. **Public-read toggles** — forum_public_read pattern extended: each
   community feature can be member-only per instance.
