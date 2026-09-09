# Requests Spam Guardrails — Plan (draft, needs your sign-off before build)

> **Goal:** make `/requests` un-spammable without killing legitimate use.
> Mirrors the existing site patterns: `070_auto_moderation` grace-period +
> curator-review for body cache, forum mod-points, tiered Redis rate limits,
> and `modlog`. Plan only — no code lands until you approve / edit.

## 1. Audit: what exists today

**Creation (`POST /api/requests`, auth required):**
- Validates `title` non-empty, <=200 chars; `body` <=4000; optional `seed_work_id` must exist.
- No per-user / per-IP rate limit. No honeypot (`website` / `form_opened_at` — register/forum have it, requests does not).
- Archivist LLM runs async after insert (best-effort, never blocks) — not a moderation gate.
- Inserts `fic_requests(user_id, title, body, seed_work_id)` with `status='open'`, `deleted_at=NULL`. Immediate public visibility.
- No duplicate / near-duplicate check (same title+body spam is allowed).
- No trust/role/reputation gate — any `role>=0` authed user can create.

**Deletion:**
- `DELETE /api/requests/{id}` — soft `deleted_at=NOW()` if `owner OR role>=5` (curator). Instant, no grace, no review. No modlog entry today (inconsistency — admin actions modlog, requests don't).
- Same shape for answers (`DELETE .../answers/{aid}`).

**Other writes:**
- Answers capped at 3 per user per request (`ANSWER_CAP`). No time window, but the cap limits answer spam.
- Votes 1/-1/0, toggle, no self-vote. Upvotes on requests (toggle, no self-vote). All check `AuthUser`.
- No tier for `/api/requests` in `tier_for_path` — falls into `Tier::Default` (global bucket only; ~30 burst / 0.116 flow). Auth tier (10/min) is *not* applied to requests, so the only throttle is the shared global bucket + nginx.

**Queue / moderation:**
- No curator queue, no consensus flow. No LLM classification. No `deletion_scheduled_at` or `review_status` on `fic_requests` (unlike `content_scan`).
- `content_scan` pattern *is* available to copy: `classification/confidence → deletion_scheduled_at (72h) → hourly cron `process_expired_deletions` → curator confirm/dismiss → modlog.

**Read path:**
- `GET /api/requests?status=&sort=&page=` is public, 20/page. No abuse there.

**Verdict:** yes, you can spam — N requests per minute per account until the global bucket whines, then you just rotate `X-Client-Id` or slow to ~1/8s and keep going. Low-effort / duplicate / prompt-injection requests go live immediately and only disappear if a curator happens to notice and DELETEs.

## 2. Threats to cover

1. **Burst spam** — 20 junk requests in 10 minutes.
2. **Slow drip** — 50/day of low-effort "fic recs plz" farming activity.
3. **Duplicate / near-duplicate** — same request re-posted to farm upvotes.
4. **Off-topic / adversarial** — non-requests, hate, NSFW, prompt-injection ("ignore previous instructions…") polluting the board.
5. **Sybil** — new accounts created to spam (trust-gate mitigates).
6. **Curator overload** — flagging everything creates a second spam queue for mods.

## 3. Decisions I need from you (pick before build)

| # | Question | Options (recommended first) | Your pick |
|---|----------|------------------------------|-----------|
| D1 | **New-request visibility** | **A) live immediately, flag hides after review** (low friction, current UX) · B) `pending` until first curator/consensus approve (strict, kills drip-spam but adds latency) · C) hybrid: trusted users (TL>=2 or curator) live; new users `pending` |  |
| D2 | **LLM strictness** | **A) flag only high-confidence spam (precision > recall, 0.85)** · B) flag medium+ (0.70, more queue noise) · C) LLM off by default, manual flags only |  |
| D3 | **Grace period length** | **A) 72h like `content_scan`** · B) 24h · C) 7 days |  |
| D4 | **Removal authority** | **A) any curator `role>=5` can confirm/dismiss solo (simple)** · B) **consensus quorum (2 curators, or 3 votes)** — mirrors collection/consensus patterns · C) admin-only (`role>=10`) |  |
| D5 | **Rate limits** | **A) 3/hour + 10/day per user (Redis, configurable env)** · B) 5/hour + 20/day · C) stricter (1/hour) |  |
| D6 | **Duplicate window** | **A) 7-day exact-title duplicate block (same normalized title → 409)** · B) also Ollama embedding near-duplicate (pgvector cosine >0.90) flags · C) none |  |
| D7 | **Where flagged queue lives** | **A) filter on `/api/curator/requests?status=flagged` reusing `fic_requests` columns** · B) dedicated `request_moderation_queue` table |  |

Defaults I will build if you say "go" with no edits: **A** for all (72h, single-curator confirm/dismiss, 3/h+10/d, 7-day exact-title dupe, queue = flag columns on `fic_requests`).

## 4. Design (copy existing patterns, no new infra)

### 4.1 DB — new migration `074_requests_moderation.sql` (advisory, idempotent)

```sql
-- Extend fic_requests with moderation state (mirrors content_scan.deletion_scheduled_at)
ALTER TABLE public.fic_requests
  ADD COLUMN IF NOT EXISTS mod_status text NOT NULL DEFAULT 'clean'
    CHECK (mod_status IN ('clean','flagged','pending_removal','removed')),
  ADD COLUMN IF NOT EXISTS flag_reason text,
  ADD COLUMN IF NOT EXISTS flag_confidence real,
  ADD COLUMN IF NOT EXISTS removal_scheduled_at timestamptz,
  ADD COLUMN IF NOT EXISTS reviewed_by integer REFERENCES users(id),
  ADD COLUMN IF NOT EXISTS reviewed_at timestamptz,
  ADD COLUMN IF NOT EXISTS review_decision text
    CHECK (review_decision IN ('keep','remove') OR review_decision IS NULL);

CREATE INDEX IF NOT EXISTS idx_fic_requests_mod
  ON public.fic_requests (removal_scheduled_at)
  WHERE mod_status = 'pending_removal' AND removal_scheduled_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_fic_requests_flagged
  ON public.fic_requests (mod_status, created_at DESC)
  WHERE mod_status IN ('flagged','pending_removal');

-- Optional: consensus votes when D4=B (quorum mode). Single-curator mode still writes here (1 row = decision).
CREATE TABLE IF NOT EXISTS request_moderation_votes (
  request_id integer NOT NULL REFERENCES fic_requests(id) ON DELETE CASCADE,
  curator_id integer NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  decision text NOT NULL CHECK (decision IN ('keep','remove')),
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (request_id, curator_id)
);
```

`mod_status` vs `status` (`open/answered/closed`) — orthogonal. A request can be `open+flagged`.
`removed` means soft-deleted via moderation (sets `deleted_at` + `mod_status='removed'` for audit).

### 4.2 Rate limits + anti-automation (reuse Redis bucket)

- Map `POST /api/requests` (and maybe `POST /api/requests/{id}/answers`) into a **dedicated bucket keyed by `user_id`**, not just IP — easiest is a small helper in `requests.rs` that checks `rate:tier:requests:user:{id}` before insert.
  - Env (add to `Config`): `REQUESTS_PER_HOUR` (default 3), `REQUESTS_PER_DAY` (10), `REQUESTS_ANSWER_PER_HOUR` (10 — or reuse `ANSWER_CAP` which already bounds total, but add a time window to stop burst).
  - Enforce with the existing Redis Lua (reuse `check_bucket`) or a tiny sql fallback (`SELECT count(*) FROM fic_requests WHERE user_id=$1 AND created_at > now()-interval '1 hour'` — 1 cheap index scan). Redis preferred so it matches the rest of the limiter.
  - Also wire `/api/requests` into `tier_for_path → Tier::Auth` or a new `Tier::Requests` (cleaner: add `Tier::Requests` with 3/hour capacity so global bucket isn't the only defense). I propose `Tier::Requests` so we don't overload Auth semantics.
- **Honeypot**: add `form_opened_at` + `website` fields to `CreateRequestBody` (optional on wire, reject if `website != ""` or `now - form_opened_at < 2s`). Mirrors `auth::handleRegister` — bare curl fails silently.
- **Trust gate (optional, you choose strictness)**: require `users.trust_level >= 1` or `level >= 2` for creation when `REQUESTS_TRUST_GATE=true`. New accounts can still read/vote but not create — spam accounts lose their vector. Keep off by default if you want open-by-default.
- **Duplicate guard**: before insert, normalized-title exact match in last 7 days (`lower(regexp_replace(trim(title),'\\s+',' ','g'))`) → 409 "Duplicate request — see #id". Embedding near-duplicate is a v2 (needs pgvector index on fic_requests, do later if A proves insufficient).

### 4.3 LLM moderation with grace period (copy `content_scan` shape)

**Prompt** (reuse `ollama_chat_model`, best-effort — Ollama down → skip, never blocks creation):

```
You are moderating Fic Requests (prompt board). Classify:

Return ONE line: classification | confidence | reason
classification ∈ {clean, spam, off_topic, low_effort, policy_violation}
confidence 0.0..1.0, reason short phrase.

Title: {title}
Body: {body}
```

Parsing: same `splitn 3, '|'` pattern as `parse_scan_response`. Threshold 0.85 (D2=A).

On `spam|policy_violation` with `conf>=threshold`:
- `mod_status='pending_removal'`, `removal_scheduled_at = now()+grace`, `flag_reason`, `flag_confidence`, modlog `request_auto_flag` (actor `auto-moderator`, grace_hours).
- Request remains visible but gets a banner ("Flagged — removal in {grace}") and is surfaced in the curator queue.
- On `off_topic|low_effort` with high conf → `flagged` (no auto-deletion, just curator queue).

**When the scan runs:** `tokio::spawn` right after the `INSERT ... RETURNING id` in `create_request` (same place Archivist runs), plus a nightly/periodic sweep of recent un-scanned `fic_requests` for the Ollama-down gap. Both paths call a shared `moderate_request_with_grace(db, ollama, req_id, title, body)`.

**Grace expiry cron:** `process_expired_request_removals(db, config)` — same shape as `process_expired_deletions` (select `mod_status='pending_removal' AND removal_scheduled_at <= now()` limit 50, then for each: soft `deleted_at=now()`, `mod_status='removed'`, `review_decision='remove'`, modlog `auto_delete_request`, emit notification to owner + curators). Wired in `server.rs` as an hourly `tokio::spawn` loop beside the existing 3 crons.

**Curator undo:** `POST /api/curator/requests/{id}/dismiss` clears `removal_scheduled_at`, sets `mod_status='clean'` (or `flagged`→`clean`), records `reviewed_by/at`, `review_decision='keep'`, modlog `request_moderation_dismiss`. Available until `removal_scheduled_at` fires; after firing, undelete is admin-only `PUT /api/admin/requests/{id}/restore`.

**Confirm early:** `POST /api/curator/requests/{id}/confirm` instantly `deleted_at=now()`, `mod_status='removed'`, modlog `request_moderation_confirm` (bypasses grace).

### 4.4 Curator queue + consensus (reuse forum/admin patterns)

**Endpoints** (all `role>=5` unless D4=C):

- `GET /api/curator/requests?status=flagged|pending_removal&sort=new&page=` — lists flagged/pending with `flag_reason`, `flag_confidence`, `removal_scheduled_at`, `hours_remaining`, `author` (uses existing `idx_fic_requests_flagged`).
- `POST /api/curator/requests/{id}/confirm` — instant remove (single-curator mode) OR cast `remove` vote (quorum mode).
- `POST /api/curator/requests/{id}/dismiss` — keep / dismiss flag (solo OR `keep` vote).
- `GET /api/curator/requests/{id}/votes` — vote breakdown (quorum mode).
- Reuse `modlog::record` for every transition; add to existing modlog filter UI (no new page needed — `GET /api/modlog?target_type=request` already exists).

**Quorum mode (when you pick D4=B):**
- `request_moderation_votes` accumulates curators' `keep|remove` votes.
- Threshold: 2 `remove` → auto `deleted_at`; 2 `keep` → auto `mod_status='clean'` + clear `removal_scheduled_at`. Mixed 1-1 leaves it pending (third vote decides). Votes are idempotent per curator (PK).
- Grace timer still runs — quorum can resolve before or after expiry; expiry just auto-removes if quorum hasn't `keep`-ed. This mirrors `collection_item_requests` voting shape.

**Single-curator mode (D4=A):** same endpoints, but the first confirm/dismiss wins; votes table stays empty (or 1 row for audit).

### 4.5 Frontend (minimal, archive-mode aware)

- **Create form (`/requests/new`):** add hidden honeypot fields + `form_opened_at` timestamp (JS sets on mount). No UX change otherwise. On 429, show "Too many requests — try again in {retry_after}m." On 409 duplicate, link to the existing request.
- **List (`/requests`):** no change for normal users. Curators see a subtle count badge "{N} flagged" linking to `/curator/requests` (or inline filter `?mod_status=flagged` on `/requests` — pick one; curators route is cleaner).
- **Detail (`/requests/{id}`):** if `mod_status != clean`, banner: yellow `Flagged: {reason} — removal in {hours}h` + curator `Confirm removal` / `Dismiss flag` buttons. Owner always sees status.
- **Curator queue page** (`/curator/requests`): new Svelte route — table of flagged rows (reuse `/curator/*` shell), bulk confirm/dismiss, vote counts in quorum mode. Empty state when clean.
- **SEO / visibility:** flagged/pending_removal requests remain indexable until actually `deleted_at`; if you later pick D1=B (pending-until-approved), flip to hidden — but that's the D1 decision.

### 4.6 Config / env

```
REQUESTS_PER_HOUR=3              # per-user bucket (Redis or DB count)
REQUESTS_PER_DAY=10
REQUESTS_ANSWER_PER_HOUR=10      # per-user answers across all requests
REQUESTS_GRACE_HOURS=72          # removal_scheduled_at delay
REQUESTS_LLM_THRESHOLD=0.85
REQUESTS_TRUST_GATE=false        # when true, requires trust_level>=1 to create
REQUESTS_DUPLICATE_WINDOW_DAYS=7
```

Wire through `Config::from_env` + `.env.example`.

### 4.7 Telemetry & ops

- Every transition modlogged (same `modlog` table the existing admin UI already filters — one filterable page per your product rule).
- Metrics in `/api/admin/stats` or a new `/api/curator/requests/stats`: `flagged_today`, `auto_removed_24h`, `dismissed_24h`, `rate_limited_hits`.
- No new background infra — just one more hourly `tokio::spawn` loop in `server.rs`.

## 5. Implementation steps (after you sign off — in order, each verifiable)

1. **Migration + config** — `074_requests_moderation.sql` + `Config` fields + `.env.example` + `cargo sqlx prepare` sanity.
2. **Rate limits + honeypot + duplicate guard** — enforce in `create_request`/`add_answer`, map `Tier::Requests`, add `tier_for_path` test, return 429 with `retry_after` + 409 for dupes. Manual test: `curl` spam loop → 429.
3. **LLM moderation service** — `src/services/request_moderation.rs` (`moderate_request_with_grace`, prompt, parse, threshold, `upsert` of `mod_status/removal_scheduled_at`, modlog). Spawn from `create_request`. Test: Ollama stub → flagged row.
4. **Grace expiry cron + curators confirm/dismiss** — `process_expired_request_removals` + 4 endpoints + modlog. Test: set `removal_scheduled_at` in past → cron soft-deletes.
5. **Queue UI** — `GET /api/curator/requests` + `/curator/requests` Svelte page with confirm/dismiss; banner on `/requests/{id}`. Empty + flagged states.
6. **(Optional, if you pick D4=B)** quorum votes + `request_moderation_votes` handling + `GET .../votes`.
7. **Limits on answers too** (if not already handled by `ANSWER_CAP`) — per-user answer rate bucket + reuse same LLM path for answer-level spam if you want (defer unless needed).
8. **Tests + docs** — `cargo test` shard for limiter/tier & requests moderation, `cargo sqlx prepare --check`, update `docs/brainstorm-02-roadmap-status.md` + `docs/USER-ACTIONS.md` (canonical docs), conventional commit.

## 6. What I will NOT do unless you say so

- Auto-shadowban authors (too punitive for requests; keep explicit removal).
- Embedding near-duplicate index (v2 — exact-title handles 90% of spam).
- Trust-gate enabled by default (would block legit new users — leave it opt-in).

## 7. Risks & mitigations

- Ollama down → no flagging. Mitigated: creation still throttled by rate limits + honeypot + dupe window; nightly sweep retries moderation.
- LLM false positives → mitigated by high threshold (0.85) + 72h grace + one-click dismiss. Shadow false rate visible in stats.
- Curator queue spam (flagging flood) → mitigated by rate limits being upstream of moderation — fewer requests means fewer flags.
- Migration 074 applied with existing rows: all `mod_status='clean'` by default, so zero behavior change for historical requests.

---

**Tell me:** your picks for D1–D7 (or just "go with defaults" / "change X"), and whether you want steps 1–5 in one go or batched. I'll then implement exactly that slice and not touch anything else until you re-approve.
