# Part 34 — Moderation and Admin

> **What ships in Aug 2026:** moderation on FicHub is a three-ring circus: an always-on **trust ladder** that gates who can publish/flag (Part 21), a **transparent modlog** that anyone logged-in can read, and a **curator consensus hub** where the community votes on fixes. Admins (role ≥ 10) keep the lights on with an analytics dashboard, an auto-tagger, a bot-scorer, bulk actions, and a weekly moderation digest. Reports funnel in through `POST /api/reports` and are auto-triaged by reporter trust weight.
>
> Everything that changes state writes a row to the **modlog** (`src/modlog.rs`). The log is public-read — moderation is transparent by design.

This part builds the moderation + admin surface: the trust ladder, modlog, curator consensus, admin analytics, auto-tagger, bulk actions, reports, and bot-scorer. Backend route first, frontend page second, then the real implementation. Kid-friendly, with **Try It Yourself** exercises and **Watch Out** callouts.

---

## 34.1 The Trust Ladder (the first gate on publishing)

Before anything is moderated it's *gated*. The trust system (`src/services/trust.rs`) is a Discourse-inspired 7-level axis, separate from role/rank/reputation:

| Level | Name | Who gets it | What it unlocks |
|---|---|---|---|
| TL0 | New | default | reads only — can't flag |
| TL1 | Basic | 5 works entered + 30k words read | can flag, can install remix |
| TL2 | Member | 5 active days + 25 works + 100k words + 1 forum post | can **publish** to the gallery (`/api/extensions`, `/api/recipes/:id/publish`) |
| TL3 | Regular | 15 days + 100 works + 400k words + 10 posts | — |
| TL4 | Elder | 30 days + 300 works + 1M words + 50 posts | — |
| TL5 | Community Moderator | staff-designated (weekly digest) | can **resolve reports** (`RESOLVE_MIN_TRUST = 5`) |
| TL6 | Near-admin | staff-designated | — |

The two constants that matter here both live in `src/services/trust.rs`:

```rust
// src/services/trust.rs (lines 21-43)
/// The minimum trust level required to publish a shareable object
/// (skin / recipe / theme / layout). Publishing is a "social write": the
/// object becomes visible to everyone, so it is gated above the bare write
/// tier. TL0/1 can draft privately; TL2+ may publish.
pub const PUBLISH_MIN_TRUST: i16 = 2;

/// Minimum trust to resolve other users' reports (the Community Moderator
/// human layer).
pub const RESOLVE_MIN_TRUST: i16 = 5;
```

💡 **Key Concept — trust gates *writes* to shared space; role gates *admin deletes/bans*.** Trust controls participation (publishing, flagging, resolving reports). Role (an explicit admin column) controls the irreversible: hiding fics, banning users, changing roles. The two systems don't overlap — trust never grants moderation, it only shapes how much weight your reports carry and what write surfaces are open to you.

⚠️ **Watch Out** — trust level is cached on the `users` row (`trust_level` column, updated by the hourly promotion pass in `services/trust.rs::run_trust_promotion`). It's *not* recomputed on every request for speed, so if a test account should have hit TL2 but the badge shows TL1, run a manual promotion pass or just refresh `/api/auth/me` — the value on the row is what the gate checks.

### Trust promotion: the weekly <15 min human layer

`services/trust.rs::run_trust_promotion` (line 303) runs hourly. It bumps TL0→TL4 automatically where thresholds are met, and collects **TL5 candidates** (the top of the auto-promotion range) into a list. Those candidates surface in the weekly moderation digest (next section) — an admin clicks once to confirm instead of eyeballing 50 stats. That's the "<15 min/week" human layer the spec calls for.

---

## 34.2 Modlog: every action, readable by anyone logged in

**Migration 034** (modlog) added an `INSERT INTO modlog` for every admin/curator action. The table is simple — `actor_id`, `actor_username`, `action`, `target_type`, `target_id`, `details` (JSON), `created_at` — and the `record` helper in `src/modlog.rs` is **best-effort**: a modlog write that fails never fails the action itself (errors are just `tracing::warn!`'d).

⚠️ **Watch Out** — modlog writes are deliberately fire-and-forget. If you see an action in the UI but a gap in `/api/modlog`, the action succeeded; the log insert just raced a closed transaction. Don't retry the write thinking it didn't happen.

```rust
// src/modlog.rs (lines 12-37)
/// Record a moderation action. Never fails the caller (errors are logged).
pub async fn record(
    db: &sqlx::PgPool,
    actor_id: Option<i32>,
    actor_username: Option<String>,
    action: &str,
    target_type: &str,
    target_id: &str,
    details: Value,
) {
    let res = sqlx::query(
        "INSERT INTO modlog (actor_id, actor_username, action, target_type, target_id, details)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(actor_id).bind(actor_username).bind(action)
    .bind(target_type).bind(target_id).bind(details)
    .execute(db).await;
    if let Err(e) = res {
        tracing::warn!("modlog record failed for action '{action}': {e}");
    }
}
```

### The route

`GET /api/modlog` (registered in `src/server.rs` line 805) is readable by **any logged-in user** (via `crate::modlog::require_logged_in(&auth)` in `src/routes/modlog.rs` line 21 — not role-gated). Query params `?limit=50&action=ban_user` narrow the feed. The handler returns a flat list: `id, actor_username, action, target_type, target_id, details, created_at`.

### What gets recorded

Every mutating admin/curator action calls `record` or `record_json` (`src/modlog.rs` line 40). The instrumented action set includes:

| Action | Where it's recorded |
|---|---|
| `set_user_role` | `src/routes/admin.rs` → `set_user_role` (line 294) |
| `toggle_ban` (ban/unban) | `src/routes/admin.rs` → `toggle_ban` |
| `approve_upload` | `src/routes/admin.rs` line 153 |
| `reject_upload` | `src/routes/admin.rs` line 173 |
| `approve_translation` | `src/routes/admin.rs` line 524 |
| `reject_translation` | `src/routes/admin.rs` line 556 |
| `hide_comment` / `delete_comment` | wherever comment moderation lives |
| `blacklist_fic` / `blacklist_author` | `src/routes/admin.rs` |
| `create_alias` / `merge_tags` | tag-management handlers |
| `delete_tag` | tag-management handlers |
| `resolve_flag` | flag resolution |
| `propose_fix` / `vote_fix` (with outcome) | `src/routes/curator_content.rs` |
| `trust_level_change` | `src/services/trust.rs` → `set_trust_level` (line 240) |
| `reputation_award` | `src/routes/admin.rs` → `rep_award_handler` |

⚠️ **Watch Out** — modlog actions are **strings**, not an enum. That makes the table flexible but means a typo in the action literal (e.g. `"set_user_role"` vs `"setUserRole"`) silently creates a new, unfilterable bucket. The API tests in `src/routes/api_contract_tests.rs` are your friend here — they assert on the literal strings, so a stray rename breaks the build.

🧪 **Try It Yourself** — read the modlog anonymously:

```bash
# Log in first to get a token, then:
curl -H "Authorization: Bearer <token>" \
  "https://fichub.net/api/modlog?limit=10&action=trust_level_change"
```

Pick an `action` from the table above and confirm the entry shows up. Notice that `actor_username` is stored even if the actor is later deleted — that's intentional for audit history.

---

## 34.3 Weekly Moderation Digest & Trust Promotion (`/api/admin/digest`)

The digest is the admin's "one page to rule them all" — and it's also the **trust TL5 confirmation surface**. It lives in `src/routes/trust.rs` (line 95) and is gated at `role >= 10`:

```rust
// src/routes/trust.rs (lines 95-130, excerpt)
/// GET /api/admin/digest — render the weekly moderation digest as JSON for
/// the admin page (role >= 10).
pub async fn admin_digest(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    if auth.role < 10 {
        return Err(AppError::Forbidden("Admin only".into()));
    }
    let contested: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, target_type, status, auto_status FROM user_reports
         WHERE auto_status IN ('auto_hidden','needs_admin') OR status = 'open'
         ORDER BY created_at DESC LIMIT 40",
    ).fetch_all(&state.db).await?;
    let (spam_auto, spam_total): (i64, i64) = sqlx::query_as(
        "SELECT
           COUNT(*) FILTER (WHERE auto_status IN ('auto_hidden','needs_admin')),
           COUNT(*)
         FROM user_reports WHERE created_at > NOW() - interval '7 days'",
    ).fetch_one(&state.db).await?;
    // ... trust_events_7d, TL5 candidates ...
```

It returns four buckets:

1. **`contested_reports`** — reports that need a human (auto-status `auto_hidden` or `needs_admin`, or still `open`).
2. **`spam_auto_hidden_7d` / `spam_total_7d`** — the 7-day spam auto-hide ratio (so you can sanity-check the bot-scorer without diving into raw rows).
3. **`trust_events_7d`** — every trust-level change in the last week, with `from`/`to`/`reason`/`by_user_id` — this is where you confirm TL5 candidates.
4. **TL5 candidates** — the exact list `run_trust_promotion` produced; the admin clicks to confirm each into TL5 (Community Moderator).

⚠️ **Watch Out** — the digest only runs for `role >= 10`. TL5 (Community Moderator) can *resolve* reports but cannot *confirm TL5 candidates* — that admin authority is never delegated, not even to Near-admin (TL6).

---

## 34.4 Reports: `POST /api/reports` + Trust-Weighted Triage

The front door for community moderation is `POST /api/reports` (`src/routes/reports.rs`). A report is auto-triaged at write time using the **reporter's trust weight**:

```rust
// src/services/trust.rs (lines 21-56)
/// Flag weight granted per trust level (reporter trust → effective weight):
/// TL1=1, TL2=1, TL3=2, TL4=3, TL5=5, TL6=5. TL0 may not flag.
pub fn flag_weight(level: i16) -> i32 {
    match level {
        0 => 0, 1 | 2 => 1, 3 => 2, 4 => 3, 5 | 6 => 5,
        _ => 1,
    }
}
```

That weight drives the triage: a single TL1 flag → `pending` (a human TL5+ must look). A TL4+ flag (weight 3+) on something already at `pending` with accumulating weight → promoted to `needs_admin`. And a flood of high-trust flags, or a flag on a freshly-registered account, can auto-triage to `auto_hidden` (the item is hidden pending review, reversible). The report then lands in the next digest run for the admin queue.

The triage states live in the `user_reports` row: `status` = `open` / `pending` / `needs_admin` / `resolved`, and `auto_status` = `pending` / `auto_hidden` / `needs_admin`.

🧪 **Try It Yourself** — flag a fic:

1. Log in, open any work page.
2. Click the flag icon, pick "Spam / scrape," submit.
3. Open `/modlog` in a new tab — do you see a `create_alias` or anything? (You shouldn't *yet* — reports aren't modlog actions. They appear in admin's `/api/admin/digest` instead.)

---

## 34.5 Curator Consensus Hub (`/curator/consensus` + `GET /api/curator/consensus`)

The consensus hub (`src/routes/consensus.rs`) is a single read-only feed that **normalizes every community-vote surface** into one list so curators don't have to context-switch between five different pages. Each item carries a `kind` tag:

| `kind` | Backed by |
|---|---|
| `content` | `curator_fix_proposals` (peer-voted body fixes) |
| `metadata` | `curator_metadata_proposals` |
| `flag` | `tag_flags` (open / resolved) |
| `alias` | `tag_aliases` |
| `author_merge` | `author_merge_proposals` |
| `roadmap` | `feature_clusters` (formerly admin-dashboard-only; migrated to public-read in **053**) |

The handler (line 44) is gated with `crate::modlog::require_logged_in(&auth)` — i.e. **any logged-in user** can read it.

⚠️ **Watch Out** — the consensus feed was originally curator-only and returned `403 Forbidden` for non-curators (role < 5). That was a regression fixed so the feed is now public-read (logged-in). The route registration in `src/server.rs` (line 332) is the spot to confirm: `.route("/api/curator/consensus", get(crate::routes::consensus::consensus_feed))`. If you re-introduce a `require_curator` check, you'll silently break the public-read transparency the spec relies on.

The query shape mirrors `/api/curator/approvals` (one unified queue) but is broader in scope — consensus includes *roadmap* and *tag flags* which the approvals queue doesn't. Filter chips (`?type=&status=&q=`) are deep-linkable; the frontend at `/curator/consensus` builds its URL from the selected chip set so users can bookmark a filtered view.

---

## 34.6 Curator Approvals Queue (`/curator/approvals` + `GET /api/curator/approvals`)

Where consensus is the *feed* of what's being voted on, `/api/curator/approvals` (`src/routes/collections.rs` line 889, `curator_approvals_handler`) is the *action queue*. It aggregates pending items across every curation type:

- collection add-requests (`collection_item_requests`),
- content-fix proposals (`curator_fix_proposals`),
- metadata proposals (`curator_metadata_proposals`),
- comment triage rows (`comment_triage`),
- forum edit proposals (`forum_edit_proposals`).

```rust
// src/routes/collections.rs (lines 894-907, excerpt)
/// GET /api/curator/approvals?status=&type=&curator=
/// Unified curator Approvals queue: pending items across every curation type
/// (collection add-requests, content-fix proposals, metadata proposals, comment
/// triage rows, forum edit proposals). Each item carries a `type` tag and — where
/// applicable — a live community vote tally plus the viewer's own vote. The
/// `pending_counts` object always reports the pending total per type (independent
/// of the status/type filters) so the frontend can show live badges.
///
/// `curator` filters by the submitting/authoring username (ILIKE). Comment
/// triage rows have no status column (they are deleted on resolution), so they
/// only surface under `status=pending`.
```

Each item returns a `type` tag plus live vote tallies (`votes_for` / `votes_against` and `my_vote`). The response always includes a `pending_counts` object — one per type, independent of the filters — so the frontend can show live badge numbers without a second request. Gated at `auth.role < 5` → `Forbidden` (curator+).

---

## 34.7 Admin UI: Translations, Metadata Correction, Moderation Queue

The admin UI is three surfaces under `/admin/`:

### A. Moderation Queue — `/admin/moderation` → `GET /api/admin/moderation/queue`

`src/routes/admin.rs` line 43 (`mod_queue`). Lists `works` where `is_visible = FALSE` and `fic_info.source_type IN ('manual_epub','manual_text','import')`. 20/page by default. Each row carries `work_id, title, author, description, source, uploader, source_type, created`.

The approve/reject pair is right after (lines 90–143):

- `POST /api/admin/moderation/approve/{work_id}` — flips `is_visible = TRUE`, runs the **v3 XP** path: awards a qualified-publish bonus (≥5k words), auto-claims work bounties (`services::bounties::auto_claim_work_bounties`), and grants completion bonuses. Every approve is modlog'd as `approve_upload`.
- `POST /api/admin/moderation/reject/{work_id}` — modlog'd as `reject_upload`.

⚠️ **Watch Out** — approval triggers XP awards via `services::progression::award_xp`. Those XP grants are *not* modlog'd individually (only the approval is), but the XP row lands in `reading_stats` / `user_xp` and the user's next `/api/auth/me` refresh reflects it.

### B. Translation Review — `/admin/translations`

`POST /api/admin/translations/{id}/approve` and `/reject` and `/edit`. Approve/reject call `services::translations` and write `approve_translation` / `reject_translation` to the modlog (lines 524, 556). The list view is a paged query (`TranslationListParams`: page, per_page, status) over the `translations` table.

### C. Metadata Correction — `/admin/metadata`

`POST /api/admin/metadata/{work_id}` lets an admin correct canonical metadata (title, author, warnings, etc.). The mutation is logged as a `modlog::record_json(...)` call with `action = "metadata_correction"` and the changed-field set in `details`. It does *not* go through the curator-fix proposal pipeline — it's an immediate admin override.

---

## 34.8 Admin Analytics (`/admin/analytics`, role ≥ 10)

Analytics landed in two commits — the `usage_events` table shipped in **migration 033**, and the tracking middleware + dashboard routes (`src/routes/analytics.rs`) are wired in `src/server.rs` lines 797–802:

```rust
// src/server.rs (lines 797-802)
.route("/api/analytics", get(crate::routes::analytics::analytics_handler))
.route("/api/analytics/user/{client_id}", get(crate::routes::analytics::user_stats_handler))
.route("/api/reading/analytics", get(crate::routes::analytics::personal_reading_handler))
.route("/api/authors/{id}/analytics", get(crate::routes::analytics::author_analytics_handler))
.route("/api/admin/analytics", get(crate::routes::analytics::admin_analytics_handler))
.route("/api/admin/endpoint-usage", get(crate::routes::analytics::endpoint_usage_handler))
```

### The middleware (zero-PII)

`src/routes/analytics.rs::track_usage` (line 149) is an Axum middleware running on every request that carries an `X-Client-ID` header. It classifies the request as a **view** or an **action** via `is_action(path)` — action prefixes include `/api/epub`, `/api/download`, `/api/meta`, `/api/comments`, `/api/votes`, `/api/curator`, `/api/auth/login`, `/api/rate`, `/api/review`, etc. (the full list is `ACTION_PREFIXES`, lines 116–140; anything not matching is a view). The event row is **spawned on a background task** so it never adds latency.

💡 **Key Concept — zero PII by design.** The middleware stores *only* `client_id`, `user_id` (nullable), path, action/view flag, and `X-Client-ID`. No URLs with query strings, no IP, no user agent. `client_id` is the app's anonymous install ID (UUID v4), not an email or cookie. The row shape is `usage_events(user_id, client_id, event_type, path, created_at)`.

`GET /api/admin/analytics` (`admin_analytics_handler`) aggregates into:

- unique daily / weekly / monthly visitors,
- **active vs view-only users** (action count > 0 vs view-only),
- an action timeline (events-per-day for the last 30 days),
- popular fics by views/downloads over 7d and 30d,
- format breakdown (EPUB vs text vs HTML) over 30d.

### Search → export conversion (`GET /api/admin/search-analytics`)

Admin-only (role ≥ 10). Turns the question "when someone searches, do they download?" into a conversion funnel: query-term → click-through → `/api/download` or `/api/epub`. The handler lives at the tail of `src/routes/admin.rs` and is registered in `src/server.rs` line 858.

⚠️ **Watch Out** — search analytics reads from `usage_events` joined against the search log table. If you ran the book's earlier "build your own search" chapter, the funnel will be empty until users actually search *and* export. The numbers look alarmingly zero at first.

---

## 34.9 Auto-Tag Admin Routes (`/api/admin/auto-tag/*`)

FicHub auto-tags new fics with **zero-shot classification via embeddings**. A fic's `description` + `title` is embedded through Ollama (`nomic-embed-text`, 768-dim) and compared with cosine similarity against the embedded canonical freeform tags in `tag_embeddings`. Tags at ≥ 0.75 similarity (`SIMILARITY_THRESHOLD`, `src/services/auto_tagger.rs` line 27) are attached as `is_machine_suggested = TRUE` and land in the **review queue** — never directly. An admin approves (promotes to a regular tag) or dismisses (deletes the row).

The routes all gate on `user.role >= 10` (same guard as the rest of `src/routes/admin.rs`) and live in `src/routes/auto_tag.rs`:

| Route | Handler |
|---|---|
| `POST /api/admin/auto-tag` | `auto_tag_fic` — recommend + insert suggestions for one `url_id` |
| `POST /api/admin/auto-tag/backfill` | `backfill_tag_embeddings` — embed the canonical tag corpus (idempotent; run once after deploy) |
| `GET /api/admin/auto-tag/queue` | `tag_suggestion_queue` — `is_machine_suggested = TRUE AND reviewed_at IS NULL` |
| `POST /api/admin/auto-tag/approve/{url_id}/{tag_id}` | promote suggestion → regular tag |
| `POST /api/admin/auto-tag/dismiss/{url_id}/{tag_id}` | delete the suggestion row |

⚠️ **Watch Out** — the classifier embeds **description + title only**. Chapter text lives in the export cache (`cache::disk`) on disk, not in the DB, so the `first_chapter` field on `TagSuggestion` is reserved but not yet wired (see `src/services/auto_tagger.rs` lines 13–14). Don't file bugs about "it didn't suggest a tag that's only in chapter 3" — that's the known limitation.

---

## 34.10 Bulk Admin Actions + Bot-Scorer

Two ops-heavy features shipped together in **commit `b017681`**:

### Bulk admin actions (`src/routes/bulk.rs`)

A single handler that fans one request out across many targets — approve 50 pending uploads, reject a batch of translations, or bulk-reassign authors. The route is mounted in `src/server.rs` and delegates to `services::bulk`. Each individual action inside the batch is still modlog'd (so you can see which row in a batch succeeded vs. skipped), but the HTTP envelope is one `POST /api/admin/bulk` with a JSON array body.

⚠️ **Watch Out** — bulk actions are **atomic at the request level**: if row 23 of a 50-row batch hits a constraint error, the *entire batch* returns a 4xx with the failing index — it does **not** partially commit. If you need all-or-nothing, wrap your client call in a retry that re-sends the successful prefix.

### Bot-scorer + `/admin/bots` (`src/bin/bot_scorer.rs`)

An hourly cron job (`src/bin/bot_scraper.rs` runs via the s6 supervision tree — see `hermes-s6-container-supervision` for the daemon config) that scores new uploads for spam-likeness and writes the result into a **Redis shadowban** set keyed by `url_id`. `GET /admin/bots` lets a moderator query a work's bot-score and see the shadowban reason without the user knowing their uploads are being quietly deprioritized in search and recs.

💡 **Key Concept — shadowban, don't delete.** A bot-scored work isn't hidden from the user who uploaded it; it's just removed from *recommendations, search, and the gallery* until a human clears it. That keeps the spammer guessing (and burning effort) while keeping the public archive clean. The modlog still records the score for audit.

---

## 34.11 Trust + Role Authority: who can do what

| Action | Required authority |
|---|---|
| Flag a work | TL1+ (`flag_weight` > 0) |
| Publish to gallery (`POST /api/extensions`, `PUT /api/recipes/:id/publish`) | TL2+ (`PUBLISH_MIN_TRUST`) |
| Approve/reject a manual upload | role ≥ 10 |
| Approve/reject a translation | role ≥ 10 |
| Correct canonical metadata | role ≥ 10 |
| Hide / delete comment | role ≥ 10 |
| Ban / unban a user | role ≥ 10 |
| Set user role (`PUT /api/admin/users/{id}/role`) | role ≥ 10 |
| Resolve someone else's report | TL5+ (`RESOLVE_MIN_TRUST`) |
| Confirm a TL5 candidate | role ≥ 10 (the digest path) |
| Award admin-only XP / reputation | role ≥ 10 |
| Open the auto-tagger review queue | role ≥ 10 |
| Read `/api/modlog` | **any logged-in user** |
| Read `/api/curator/consensus` | **any logged-in user** |
| Read `/api/curator/approvals` | curator (role ≥ 5) |
| Read `/api/admin/analytics` | role ≥ 10 |

⚠️ **Watch Out** — `role` and `trust_level` are different columns on `users`. Promoting someone to TL5 does **not** give them `role >= 5`; it gives them the trust weight to resolve reports, but curator tools (the approvals queue) still require the explicit curator role. The two are deliberately orthogonal.

---

## 34.12 Verifying the Contract: `cargo test` + `sqlx prepare --check`

All moderation + admin routes are covered by **API contract tests** in `src/routes/api_contract_tests.rs`. These aren't unit tests of logic — they assert on the *shape* of the response and the *literal strings* the handlers use (modlog actions, field names, error messages). If you rename an action or change a response key, `cargo test` breaks here first.

```rust
// src/routes/api_contract_tests.rs
// Example: asserts the modlog list endpoint returns the documented shape
// and that 'approve_upload' is a recorded action string.
```

⚠️ **Watch Out** — the contract tests use a throwaway test database, so they run `sqlx::query!` macros at *compile* time against whatever database is configured. Before pushing any migration change, run both:

```bash
cargo test
sqlx prepare --check          # fails the build if migrations + query! drift
cargo clippy --all-targets    # catches the Option<Option<T>> borrow pitfalls
```

`sqlx prepare --check` is the one that bites people: it re-runs every `query!`/`query_as!` against the migration set and fails if the SQL in a handler no longer matches the schema. If you added a column to `modlog` or `user_reports` and forgot to bump `prepare`, CI goes red on a machine that hasn't run your migration locally.

---

## 34.13 Exercises

**🧪 Try It Yourself — Read the modlog as a regular user.**

1. Log in as any normal account (not an admin).
2. Visit `https://fichub.net/modlog`. You should see the feed load — if you get a `403`, you've hit the regression from §36.5; the route must call `modlog::require_logged_in`, not `require_curator`.
3. Filter to `action=trust_level_change` and confirm the TL0→TL1 promotion you'll earn by flagging the spam fic in the next exercise shows up here.

**🧪 Try It Yourself — Flag + watch the digest pick it up.**

1. Find a work you suspect is a scrape (no proper chapter structure).
2. Click **flag → Spam / scrape**. (This calls `POST /api/reports`.)
3. If you're TL4+, the flag alone may push the report into `needs_admin`. If you're TL1, it stays `pending` — you'll need a TL4+ friend to flag the same work.
4. (Admin only) hit `https://fichub.net/admin/digest` and confirm your report id appears in `contested_reports`.

⚠️ **Watch Out** — flagging the same work twice from the same account is a no-op (the `user_reports` table has a `(reporter_id, work_id)` uniqueness constraint). To accumulate weight you need *different* accounts or higher trust tiers, not repeated clicks.

---

## 34.14 Recap

- **Trust ladder (7 levels)** in `src/services/trust.rs` gates writes: TL2+ publishes (`PUBLISH_MIN_TRUST`), TL5+ resolves reports (`RESOLVE_MIN_TRUST`). Role ≥ 10 gates irreversible admin actions. They're orthogonal.
- **Modlog** (migration 034, `src/modlog.rs`) records *every* admin/curator action as a best-effort `INSERT INTO modlog`. `GET /api/modlog` is **public-read** (any logged-in user) for transparency.
- **Weekly digest** (`GET /api/admin/digest`, `src/routes/trust.rs`) surfaces contested reports, 7-day spam ratio, trust events, and TL5 candidates for admin confirmation.
- **Reports** (`POST /api/reports`) auto-triage by reporter trust weight into `pending` / `needs_admin` / `auto_hidden`.
- **Curator consensus hub** (`GET /api/curator/consensus`, `src/routes/consensus.rs`) is a normalized feed over fix proposals, tag flags, aliases, author merges, and roadmap clusters — public-read by any logged-in user.
- **Curator approvals queue** (`GET /api/curator/approvals`, `src/routes/collections.rs`) aggregates pending items across all curation types with live vote tallies.
- **Admin analytics** (`GET /api/admin/analytics`, `src/routes/analytics.rs`) + tracking middleware (`usage_events` table, migration 033) + search→export conversion (`GET /api/admin/search-analytics`) — role ≥ 10, zero-PII by design.
- **Auto-tagger** (`src/routes/auto_tag.rs` + `src/services/auto_tagger.rs`) uses Ollama embeddings at 0.75 cosine threshold; suggestions land in a review queue, never directly.
- **Bulk actions** (commit `b017681`) fan one request across many targets, each sub-action still modlog'd.
- **Bot-scorer** (`src/bin/bot_scorer.rs`, hourly) writes a Redis shadowban; `/admin/bots` surfaces scores.
- **Contract tests** in `src/routes/api_contract_tests.rs` + `sqlx prepare --check` guard against migration/schema drift.

> *The customization platform (Part 21) and the moderation/admin surface (this part) are both gated by the same trust ladder. The next time you see a "Publish" button disabled, you'll know it's not a bug — it's the trust system keeping the gallery spam-free until you've earned your place.*
