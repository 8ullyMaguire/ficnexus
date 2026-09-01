# Preventing Bots Without Friction

FicHub is an open download service: anyone can paste a URL and get an EPUB. That
openness is the product — but it also attracts scrapers, spam bots, credential
stuffers, and "bad faith" automation (mass-downloading to repost, fake traffic,
SEO spam). This page is the playbook for hardening against bots **without**
friction for real readers.

## Principles

1. **Friction follows behavior, not identity.** Never challenge a normal reader
   who is just downloading one fic. Only introduce a challenge after a pattern
   of bot-like behavior (rate, burst, or ratio anomalies).
2. **Cheap signals first.** Rate limiting, headers, and honeypots cost nothing
   for humans. CAPTCHAs and proof-of-work are last resorts.
3. **Defense in depth.** No single control is enough; layer them so a bot that
   evades one hits the next.
4. **Never block on first contact.** First-time anonymous users are mostly
   real. Flag first, block later.

## What FicHub already has

- **Redis bucket rate limiter** (`src/limiter/redis_bucket.rs`) — per-IP token
  bucket, enforced on download/export endpoints. `dynamic_rate_limit` can be
  enabled per-request. This is the first line: it already stops naive
  mass-download bots.
- **Client-id tracking** (`X-Client-ID` header, `request_log`) — every request
  is logged with source + route + timing. This is the raw material for
  behavioral detection (see below).
- **User accounts with roles** — registered users can be flagged/banned by
  admins (`/api/admin/users/{id}/ban`).

## Layered controls (in order of increasing friction)

### Layer 0 — Observability (no friction)

- Keep `request_log` rich: IP, `X-Client-ID`, user agent, route, timing,
  export file. Add: `X-Forwarded-For` handling behind nginx (currently the
  service may only see the proxy IP — fix this first).
- Ship a **bot dashboard** in `/admin`: top IPs by requests/hour, top
  client-ids, export:request ratio, error-rate spikes. An admin who can *see*
  abuse can respond before it needs automation.
- Add a periodic job that computes per-IP aggregates (requests/hour,
  downloads/hour, failed-auth count) into a `bot_scores` table.

### Layer 1 — Rate limiting (cheap, mostly invisible)

- **Tiered limits per endpoint class:**
  - Static assets, docs, search: high limits (bots don't hurt here).
  - Download/export (`/api/epub`, `/api/download`): the critical one. Suggest
    per-IP: e.g. 60 downloads/hour, 300/day, with a burst of 10.
  - Auth endpoints (`/login`, `/register`): strict, e.g. 10/min per IP —
    stops credential stuffing and account spam.
- **Sliding window over the bucket** (already implemented) — keep it.
- **Rate-limit by `X-Client-ID` too**, not just IP — mobile users on shared
  NAT IPs get unfairly blocked otherwise. Key: `(ip, client_id)`.

### Layer 2 — Behavioral heuristics (still zero friction)

- **Export:request ratio**: a client that requests 500 fics and downloads 499
  EPUBs in an hour is a mirror bot, not a reader. Flag ratio > 0.9.
- **Speed**: > 5 downloads/minute sustained = suspicious. Add a per-client
  burst check before the bucket.
- **Failed-auth bursts**: > 5 failed logins/min from one IP = stuffing. Apply
  a temporary IP block (30 min) + require CAPTCHA after.
- **New-account spam**: accounts created in bursts from one IP, or with
  throwaway email patterns, get a shadow "low trust" flag until they've made
  N legit requests.

### Layer 3 — Light challenges (small friction, only for flagged clients)

- **Proof-of-work (PoW)**: for flagged clients, require a small hashcash-style
  challenge (e.g. SHA-256 prefix of 8-10 zero bits, ~0.1-1s on modern
  hardware). Returns an `X-PoW` token valid for 10 min. Humans never notice;
  bulk bots pay 1000x.
- **JS challenge**: a 1-second JS+canvas proof that a real browser is present
  (like Cloudflare's "checking your browser"). Flagged clients only.
- **Email verification** for registration (already should exist) — raise to
  mandatory if spam accounts appear.

### Layer 4 — CAPTCHA (last resort, real friction)

- Only for clearly abusive patterns: IP that already hit Layer 3 blocks,
  disposable-email signups, comment spam.
- Use a self-hosted option (e.g. turnstile-like or mCaptcha) to avoid
  third-party friction/privacy cost.

## Human-impersonation bots (the tricky ones)

Bots that mimic humans (correct headers, browser fingerprints, human-like
pacing) defeat layers 1-3. Detect via:

- **Behavioral fingerprinting**: mouse/keyboard entropy, scroll patterns,
  request-order randomness. Flag clients with *too-perfect* pacing.
- **Honeypot fields** in forms (a hidden input humans never fill): if it's
  filled, it's a bot — silently drop + shadow-ban the client-id.
- **Timing traps**: a form that a human takes 3+ seconds to fill but a bot
  submits in <300ms is bot-like. Store `form_opened_at` and flag fast submits.
- **Headless-browser detection**: check `navigator.webdriver`, missing
  `navigator.languages`, absent plugin arrays. Flag but don't block — many
  legitimate power users use automation (that's the core FicHub audience!).

## Bad-faith content bots

- **Comment spam**: require the commenter to have a non-spam history (account
  age + N legit requests) before posting links; rate-limit comments;
  honeypot the comment form; admin flag queue for reported comments.
- **Tag/upload spam**: manual-upload moderation queue already exists
  (`/api/admin/moderation/*`) — keep it mandatory for new uploaders until
  trust is earned.
- **Reputation farming**: cap daily reputation gain; require a mix of
  actions, not just one action repeated.

## What to avoid (friction traps)

- **Don't CAPTCHA everyone** — the #1 way to lose the power-user audience.
- **Don't block shared NAT IPs globally** — use `(ip, client_id)` keys.
- **Don't rate-limit reading/search** as hard as downloads.
- **Don't ban on first offense** — shadow-flag + challenge, escalate only on
  repeat patterns.
- **Don't block legitimate automation** — FicHub's whole value is programmatic
  EPUB access (OPDS readers, CLI tools). PoW tokens and API keys for known
  tools are the answer, not IP bans.

## Implementation roadmap

1. **Immediate (this week)**: fix X-Forwarded-For so rate limits see real IPs;
   add per-endpoint-class rate limits (downloads strict, auth strict, search
   loose); add `X-Client-ID` to the rate-limit key.
2. **Short term (1-2 weeks)**: behavioral aggregates job + admin bot dashboard;
   export:request ratio flag; failed-auth burst IP block.
3. **Medium term**: PoW challenge for flagged clients; honeypot fields in
   comment/registration forms; headless-browser flag (not block).
4. **Long term**: CAPTCHA layer for repeat offenders; account-trust tiers.

## Files that would change

- `src/limiter/redis_bucket.rs` — tiered limits, (ip, client_id) keys
- `src/routes/admin.rs` — bot dashboard endpoints (top IPs, ratios)
- `src/bin/` or a cron task — per-IP aggregate scoring
- `frontend/src/routes/admin/*` — bot dashboard UI
- `src/routes/upload.rs`, `src/routes/social.rs` — honeypots, form timers
- `Cargo.toml` — PoW crate (e.g. a small sha256 challenge), if added

## Verification

- Load test with a fake bot (loop 200 downloads) → blocked by Layer 1.
- Real-user journey (search → open fic → download one EPUB) → zero challenges.
- Headless browser (Playwright) → flagged in the bot dashboard, not blocked.
- After each change: `node qa/run.js` still green; no new QA findings.
