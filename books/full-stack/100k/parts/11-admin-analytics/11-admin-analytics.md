# Part 11 — Admin, Analytics & Transparency

> **Part 11 of 13** — By the end of Part 10, FicHub could *do* things: scrape
> a fic, search it, recommend it, translate it, and even diagnose its own
> failures with an LLM agent. But a platform with no one steering it is a
> platform that drifts. Who decides what gets curated? Who stops the bot
> that just downloaded the entire archive? Who can tell an admin — or a
> *reader* — what the staff actually did last week?
>
> This part builds the layer that answers those questions. We start with
> the **admin endpoints** in `src/routes/admin.rs`: the user management and
> ban endpoints, the platform stats rollup, the bot leaderboard, and the
> blacklist (Chapter 47). Then the **anti-bot defense stack**: the hidden
> honeypot and form-timing traps that silently swallow bot registrations
> and comments, the tiered Redis token-bucket rate limiter keyed by both
> IP *and* `(ip, client_id)`, the shadowban set that gives flagged clients
> friction instead of a hard block, and the hashcash-style proof-of-work
> challenge that makes a bulk downloader pay real CPU per request
> (Chapter 48). Then **usage analytics**: the `usage_events` table and the
> middleware that records every request as a *view* or an *action*, so the
> admin dashboard can tell readers from doers without ever storing PII
> (Chapter 49). Then the **modlog**: migration 034, the best-effort
> `record()` helper every mutating action calls, and the `/modlog` page
> that lets *any logged-in user* read exactly what staff did (Chapter 50).
> And finally the **curator content-fix loop**: peer-voted proposals to
> replace a scraped fic's body, a quorum before anything is written, and
> the body cache versions that make every fix reversible (Chapter 51).
>
> The thread through all five chapters is the one we've been pulling since
> Part 10: **power must be structured, gated, measured, and recorded.**
> The model proposes and the system validates; here, the admin acts and
> the *database* remembers. Every endpoint, every trap, and every audit
> trail in this part is the trust layer underneath everything we've
> built so far.

---

# Part 11 — Admin, Analytics & Transparency

> **Part 11 of 13** — By the end of Part 10, FicHub could *do* things: scrape
> a fic, search it, recommend it, translate it, and even diagnose its own
> failures with an LLM agent. But a platform with no one steering it is a
> platform that drifts. Who decides what gets curated? Who stops the bot
> that just downloaded the entire archive? Who can tell an admin — or a
> *reader* — what the staff actually did last week?
>
> This part builds the layer that answers those questions. We start with
> the **admin endpoints** in `src/routes/admin.rs`: the user management and
> ban endpoints, the platform stats rollup, the bot leaderboard, and the
> blacklist (Chapter 47). Then the **anti-bot defense stack**: the hidden
> honeypot and form-timing traps that silently swallow bot registrations
> and comments, the tiered Redis token-bucket rate limiter keyed by both
> IP *and* `(ip, client_id)`, the shadowban set that gives flagged clients
> friction instead of a hard block, and the hashcash-style proof-of-work
> challenge that makes a bulk downloader pay real CPU per request
> (Chapter 48). Then **usage analytics**: the `usage_events` table and the
> middleware that records every request as a *view* or an *action*, so the
> admin dashboard can tell readers from doers without ever storing PII
> (Chapter 49). Then the **modlog**: migration 034, the best-effort
> `record()` helper every mutating action calls, and the `/modlog` page
> that lets *any logged-in user* read exactly what staff did (Chapter 50).
> And finally the **curator content-fix loop**: peer-voted proposals to
> replace a scraped fic's body, a quorum before anything is written, and
> the body cache versions that make every fix reversible (Chapter 51).
>
> The thread through all five chapters is the one we've been pulling since
> Part 10: **power must be structured, gated, measured, and recorded.**
> The model proposes and the system validates; here, the admin acts and
> the *database* remembers. Every endpoint, every trap, and every audit
> trail in this part is the trust layer underneath everything we've
> built so far.

---

## Chapter 47 — Admin Endpoints: Users, Bans, Stats, and Bots

If you have ever run anything on the internet — even a Discord bot with
five friends in it — you know the feeling: the moment you give someone a
"moderator" button, you start worrying about the person holding it.
Admin code is different from every other code we've written in this
book. A search endpoint that's wrong costs you a confused user. An admin
endpoint that's wrong costs you *trust* — someone's account, someone's
content, someone's right to be here at all.

So before we write a single handler, let's look at how FicHub thinks
about admin code, because the shape of `src/routes/admin.rs` is a
statement about how the whole platform should be governed.

## The admin file: one surface, many powers

`src/routes/admin.rs` is 1,695 lines long — the biggest route file in the
project by far — and it is organized into lettered sections that read
like the chapters of a small constitution:

```rust
// ═══════════════════════════════════════════════════════════════════
// A. Moderation Queue
// ═══════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════
// B. Scraper Health Dashboard
// ═══════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════
// C. User Management
// ═══════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════
// D. Platform Analytics
// ═══════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════
// E. Translation review workflow
// ═══════════════════════════════════════════════════════════════════
```

…and so on through rating verification (F), character score fixing (G),
blacklist management, the content-scan queue, search mining, and
cross-site dedupe. Admin power in FicHub is not one giant "god mode"
handler — it's a *set of narrow tools*, each one doing exactly one job,
each one gated, each one recorded.

> 💡 **Key Concept — Least privilege isn't just for security.** Notice that
> admin code is organized by *responsibility*, not by "here's everything
> an admin can do." That's the principle of least privilege applied to
> code structure: a handler that can only ban users is easier to review,
> easier to test, and harder to misuse than a handler that can do
> anything. When you build your own admin surface, resist the urge to
> make one giant `admin_action()` endpoint with a `type` field. Narrow
> endpoints are safer *and* more readable.

Before we dig into handlers, there's one line we're going to see at the
top of *every single one of them*:

```rust
pub async fn admin_users(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<AdminUserParams>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
```

That guard — `if user.role < 10` — is FicHub's entire admin permission
model, and it's worth pausing on because it's so small. Remember from
Part 7: roles are an `i16` on the `users` table, with a documented
ladder: `0 = reader, 1 = curator, 5 = senior_curator, 10 = admin`
(migration 007 spells it out in a column comment). Every admin handler
*re-derives* the check from the authenticated `AuthUser` extractor, which
itself comes from the JWT claims. There is no separate "is this user an
admin?" table lookup, no permission registry, no role hierarchy library.
Ten lines in, and the entire security model is: the token says your
role, and the handler compares it to a constant.

> ⚠️ **Watch Out — The role check is per-handler by design.** Notice that
> FicHub does *not* wrap all admin routes in one middleware that checks
> the role once. That's a deliberate trade: repeating `if user.role < 10`
> in every handler is verbose, but it means a handler can never be
> accidentally mounted outside the admin router and silently lose its
> guard. When you're tempted to centralize authorization into middleware,
> ask yourself: can a future teammate mount this handler somewhere the
> middleware doesn't run? FicHub's answer is "no, because the guard lives
> in the handler itself." Defense lives closest to the code that needs
> it.

The other pattern you'll see in nearly every handler is that the guard
runs *before anything else* — before parsing the body, before touching
the database, before even reading query params. Fail fast and fail
publicly: an anonymous request gets a 403 and never costs a database
round-trip.

## The moderation queue: manual uploads waiting for a human

Section A of the file is the **moderation queue** — manual EPUB uploads
that arrived invisible and are waiting for a staff member to approve or
reject them. Remember from Part 5 that a user can upload their own EPUB
of a fic that our scrapers can't reach; that upload starts with
`is_visible = FALSE`. The queue is the list of everything waiting:

```rust
/// GET /api/admin/moderation/queue — list pending manual uploads
pub async fn mod_queue(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<PageParams>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let per_page = params.per_page.unwrap_or(20).max(1).min(100);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * per_page;

    let rows = sqlx::query_as::<_, (i32, String, String, String, String, String, String, chrono::NaiveDateTime)>(
        r#"SELECT w.id, w.canonical_title, w.canonical_author, fi.description,
                  fi.source, u.username, fi.source_type, fi.created
           FROM works w
           JOIN fic_info fi ON fi.work_id = w.id
           LEFT JOIN users u ON w.uploader_id = u.id
           WHERE fi.source_type IN ('manual_epub', 'manual_text', 'import')
             AND w.is_visible = FALSE
           ORDER BY fi.created DESC
           LIMIT $1 OFFSET $2"#
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;
```

Let's read that query like a detective. The `JOIN fic_info` brings in
the per-source metadata. The `LEFT JOIN users u ON w.uploader_id = u.id`
is a *left* join because a work could exist with a null uploader (an
import that predates user accounts) — and we still want it in the queue,
just with a null `username`. The `WHERE` clause is the entire definition
of "pending": the source type is a manual upload or import, *and* the
work is not visible. `ORDER BY fi.created DESC` puts the newest first —
the queue is a stack, and the newest upload is almost always the one
someone is waiting on.

The pagination line is a pattern worth memorizing, because you'll see it
everywhere in this file:

```rust
let per_page = params.per_page.unwrap_or(20).max(1).min(100);
let offset = ((params.page.unwrap_or(1).max(1)) - 1) * per_page;
```

`unwrap_or` supplies a default, `.max(1)` clamps the floor (page 0 and
page 1 are the same page), and `.min(100)` clamps the ceiling so a
malicious `per_page=999999` can't make the database build a huge result
set. Three method calls turn untrusted user input into a safe bound.
That's the whole job of an admin API in miniature: *take untrusted
input, clamp it, and never let it do anything expensive or destructive.*

> 🧪 **Try It Yourself — clamp your own pagination.** Open any endpoint
> you've written in this book — say, the recommendations list from Part 9.
> Does it clamp `per_page` and `offset`? If not, try this: request it
> with `?per_page=999999999`. If the response is a huge JSON blob or a
> slow query, you've found your first pagination hole. Fix it with the
> three-line clamp above, then add a test that requests `page=0` and
> `per_page=0` and asserts the response is identical to `page=1,
> per_page=20`.

The queue's *decision* endpoints are where the trust model shows. Approve
makes the work visible — but it also rewards the uploader, and it records
the decision in the modlog (which we'll meet properly in Chapter 50):

```rust
/// POST /api/admin/moderation/approve/{work_id} — approve a manual upload
pub async fn approve_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Get uploader_id before making visible
    let uploader: Option<(i32,)> = sqlx::query_as(
        "SELECT uploader_id FROM works WHERE id = $1 AND is_visible = FALSE"
    )
    .bind(work_id)
    .fetch_optional(&state.db)
    .await?;

    let (uploader_id,) = uploader.ok_or_else(|| AppError::NotFound("No pending upload with this ID".into()))?;

    sqlx::query("UPDATE works SET is_visible = TRUE WHERE id = $1")
        .bind(work_id)
        .execute(&state.db)
        .await?;

    // Award uploader XP for approval
    let _ = crate::db::queries::update_reputation_and_promote(&state.db, uploader_id, 25, "upload_approved").await;
    let _ = crate::db::queries::check_and_award_badges(&state.db, uploader_id, "upload_approved").await;

    crate::modlog::record(&state.db, user.user_id, user.username.clone(), "approve_upload", "work", &work_id.to_string(), serde_json::json!({})).await;
    Ok(Json(json!({"err": 0, "msg": "Upload approved and made visible"})))
}
```

There are three details here that separate this from a homework exercise.
First, the *read-then-write*: the handler selects `uploader_id` from a
work that is specifically `is_visible = FALSE`, and turns the absence of
such a row into a `NotFound` error. That single `AND is_visible = FALSE`
in the SELECT means you can't approve a work that's already visible, and
you can't approve a work that doesn't exist — both cases fall out of one
`fetch_optional` plus `ok_or_else`.

Second, the **XP award is best-effort**: both calls are prefixed with
`let _ =` and `.await`, swallowing the result. If reputation awarding
fails, the upload is *still approved* — the primary action must never be
held hostage by a side effect. This "main action is sacred, side effects
are best-effort" ordering is a theme that runs through the whole part,
and it's the same philosophy as the modlog's `record()` we'll see in
Chapter 50.

Third, the **modlog call happens after the mutation succeeded**. If we
recorded the action first and then the UPDATE failed, we'd have a log
entry for something that never happened. Log the truth, after the fact.

The reject endpoint is even simpler — and harsher:

```rust
/// POST /api/admin/moderation/reject/{work_id} — reject a manual upload
pub async fn reject_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Delete the fic permanently
    sqlx::query("DELETE FROM works WHERE id = $1")
        .bind(work_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record(&state.db, user.user_id, user.username.clone(), "reject_upload", "work", &work_id.to_string(), serde_json::json!({})).await;
    Ok(Json(json!({"err": 0, "msg": "Upload rejected and deleted"})))
}
```

A rejected manual upload is *deleted*, not hidden. That's a product
decision encoded in one line: a rejected upload came from outside our
scraper pipeline, so there's no source URL to re-scrape from later —
keeping a half-trusted blob around just to hide it would be worse than
deleting it. But note: the modlog entry survives. Deletion is not
erasure of the *decision*, which is what transparency requires.

> 💡 **Key Concept — Log before you forget; log after you act.** The
> ordering matters: act first, then record. A modlog row for an action
> that failed is a lie, and a platform that lies to itself in its audit
> log will eventually believe it. When you add logging to destructive
> operations, put the log call *after* the mutation, and make the log
> call best-effort so it can never roll back the action it describes.

## User management: search, role, ban

Section C is the classic admin surface: list users, change roles, toggle
bans. Let's look at the search handler's query, because it's a tiny
lesson in how to make a search endpoint that doesn't hurt the database:

```rust
if let Some(ref q) = params.q {
    let rows = sqlx::query_as::<_, (i32, String, i16, i32, Option<i64>, bool, Option<String>)>(
        r#"SELECT id, username, role, reputation, total_words_read, is_banned, locale
           FROM users
           WHERE username ILIKE $1
           ORDER BY id
           LIMIT $2 OFFSET $3"#
    )
    .bind(format!("%{}%", q))
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    return Ok(Json(json!({
        "err": 0,
        "users": rows.into_iter().map(|(id, uname, role, rep, words, banned, locale)| json!({
            "id": id, "username": uname, "role": role, "reputation": rep,
            "total_words_read": words, "is_banned": banned, "locale": locale,
        })).collect::<Vec<_>>(),
        "page": params.page.unwrap_or(1),
    })));
}
```

`ILIKE $1` with `%q%` is a substring search that's case-insensitive —
exactly what you want for "find the user whose name I half-remember."
The `bind(format!("%{}%", q))` is the important part: the *pattern* is
bound as a parameter, never interpolated into the SQL string, so a query
containing `%` or `_` or `'` is just data. This is the same parameterized
query discipline we've used all book — in admin code it matters *more*,
because the data being searched is sensitive and the users doing the
searching have power.

The response shape is a study in restraint. Look at what the admin gets:
id, username, role, reputation, words read, ban status, locale. No email
addresses, no password hashes (obviously), no IPs. The principle is
"minimum viable information for the job." An admin searching for a user
to ban needs to find the right account; they don't need the account's
email. We'll see this "zero-PII by construction" instinct get even more
deliberate in the bot endpoints below.

Changing a role and toggling a ban are two more narrow handlers:

```rust
/// PUT /api/admin/users/{id}/role — change user role
pub async fn set_user_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(user_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let new_role: i16 = payload
        .get("role")
        .and_then(|v| v.as_i64())
        .map(|v| v as i16)
        .ok_or_else(|| AppError::BadRequest(-1, "role field required (0,1,5,10)".into()))?;

    sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
        .bind(new_role)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "set_user_role",
        "user",
        &user_id.to_string(),
        vec![("role", json!(new_role))],
    )
    .await;

    Ok(Json(json!({"err": 0, "msg": "Role updated"})))
}
```

This is the first time we see `record_json`, which is `record` with a
small bag of extra keys. The `vec![("role", json!(new_role))]` means the
modlog row for this action carries *what* the new role is — so the log
isn't just "someone changed a role," it's "alvaro set user 42's role to
10, at this exact time, and here's the proof." We'll dig into the modlog
schema in Chapter 50, but keep your eyes on that details bag; it's what
turns a log from theater into accountability.

The ban toggle is the same shape with a twist:

```rust
crate::modlog::record_json(
    &state.db,
    user.user_id,
    user.username.clone(),
    if banned { "ban_user" } else { "unban_user" },
    "user",
    &user_id.to_string(),
    vec![],
)
.await;
```

The *action name itself* is conditional: `ban_user` or `unban_user`,
never a generic `toggle_ban`. That's a subtle but important decision for
an audit log. If we logged `toggle_ban` with `{"is_banned": true}` in
details, the log would be *accurate* but awkward to query — "show me
every ban in the last month" becomes "show me every toggle_ban where the
detail flag was true." By making the action name the verb, `WHERE action
= 'ban_user'` just works, and the frontend's filter dropdown (we'll see
it in Chapter 50) gets clean, readable options for free.

> ⚠️ **Watch Out — Name your actions as verbs, not endpoints.** When you
> design an audit log, the `action` column is a vocabulary, and every new
> action is a word in that vocabulary. Choose words that read like
> sentences: `ban_user`, `approve_upload`, `blacklist_fic`. Avoid
> endpoint-shaped names (`PUT /api/admin/users/42/ban`) and avoid
> overloaded verbs (`update` with type tags). Your future self, writing
> a "who banned whom this week" query, will thank you.

## Stats: the rollup table that keeps the dashboard fast

Every admin dashboard in the world has the same origin story: "let's
just COUNT(*) a few tables on page load" … and then the COUNTs get slow
and someone builds a rollup table. FicHub's `admin_daily_stats` is that
rollup, and it's worth studying because it's a *design*, not an
afterthought:

```sql
-- from migrations/007_admin.sql
CREATE TABLE IF NOT EXISTS admin_daily_stats (
    date DATE PRIMARY KEY,
    total_users INT4 NOT NULL DEFAULT 0,
    new_users INT4 NOT NULL DEFAULT 0,
    total_works INT4 NOT NULL DEFAULT 0,
    new_works INT4 NOT NULL DEFAULT 0,
    manual_uploads INT4 NOT NULL DEFAULT 0,
    epubs_downloaded INT8 NOT NULL DEFAULT 0,
    words_read INT8 NOT NULL DEFAULT 0,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

One row per day, pre-aggregated. The dashboard reads 30 rows and renders
a table; it never touches `users` or `works` at request time. The row is
filled by a standalone binary, `src/bin/compute_stats.rs`, which is
designed to run from cron once a day:

```rust
// from src/bin/compute_stats.rs — trimmed
let today = chrono::Utc::now().date_naive();

sqlx::query(
    r#"INSERT INTO admin_daily_stats (date, total_users, new_users, total_works, new_works, manual_uploads, epubs_downloaded, words_read)
       VALUES ($1,
           (SELECT COUNT(*) FROM users),
           (SELECT COUNT(*) FROM users WHERE created_at::date = $1),
           (SELECT COUNT(*) FROM works),
           (SELECT COUNT(*) FROM works WHERE created_at::date = $1),
           (SELECT COUNT(*) FROM works w
             LEFT JOIN fic_info fi ON fi.work_id = w.id
             WHERE fi.source_type IN ('manual_epub','manual_text','import')
               AND w.created_at::date = $1),
           (SELECT COALESCE(SUM(CASE WHEN etype = 'epub' THEN 1 ELSE 0 END), 0) FROM export_log WHERE created::date = $1),
           (SELECT COALESCE(SUM(words_read), 0) FROM reading_stats WHERE last_read_at::date = $1)
       )
       ON CONFLICT (date) DO UPDATE SET
           total_users = EXCLUDED.total_users,
           new_users = EXCLUDED.new_users,
           total_works = EXCLUDED.total_works,
           new_works = EXCLUDED.new_works,
           manual_uploads = EXCLUDED.manual_uploads,
           epubs_downloaded = EXCLUDED.epubs_downloaded,
           words_read = EXCLUDED.words_read"#
)
.bind(today)
.execute(&pool)
.await?;
```

Notice the structure: seven `SELECT COUNT(*)` subqueries, each with its
own definition of "new" (`created_at::date = $1`), all collapsed into
one row with an `ON CONFLICT (date) DO UPDATE` upsert, so re-running the
job on the same day *refreshes* the row instead of erroring. That's
idempotency — the cron can fire twice (cron always eventually fires
twice) and the second run is harmless.

Then the endpoint is almost anticlimactic:

```rust
/// GET /api/admin/stats — platform analytics
pub async fn admin_stats(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let daily_stats = sqlx::query_as::<_, (String, i32, i32, i32, i32, i32, i64, i64)>(
        r#"SELECT to_char(date, 'YYYY-MM-DD'), total_users, new_users, total_works,
                  new_works, manual_uploads, epubs_downloaded, words_read
           FROM admin_daily_stats ORDER BY date DESC LIMIT 30"#
    )
    .fetch_all(&state.db)
    .await?;

    // Live totals
    let totals: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM users), (SELECT COUNT(*) FROM works), (SELECT COUNT(*) FROM bookmarks), (SELECT COUNT(*) FROM request_log)"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "daily": daily_stats.into_iter().map(|(d, tu, nu, tw, nw, mu, ed, wr)| json!({
            "date": d, "total_users": tu, "new_users": nu, "total_works": tw,
            "new_works": nw, "manual_uploads": mu, "epubs_downloaded": ed, "words_read": wr,
        })).collect::<Vec<_>>(),
        "totals": {
            "users": totals.0,
            "works": totals.1,
            "bookmarks": totals.2,
            "requests": totals.3,
        },
    })))
}
```

Two queries total: one reads the 30-row rollup, one reads four live
counts. The four live counts are cheap `COUNT(*)`s on tables with
indexes (and the dashboard page caches them in the frontend store
between refreshes). The expensive aggregation work — the kind that would
scan `reading_stats` — happened in the cron job, hours ago, once.

> 💡 **Key Concept — Roll up expensive aggregations, keep cheap counts
> live.** Every analytics feature you'll ever build faces the same
> choice: aggregate on read (simple, slow as data grows) or aggregate on
> write/schedule (complex, fast forever). FicHub splits the difference
> deliberately: the *heavy* numbers (words read, uploads per day) live in
> a cron-filled rollup; the *trivial* numbers (how many users exist) are
> counted live because they're already indexed. When you design your own
> dashboard, decide per-number which bucket it belongs in — and never let
> a dashboard page scan a log table.

## The bot leaderboard: zero-PII behavioral security

Now we get to the section that ties this chapter to the next one.
FicHub's answer to "who's abusing us?" is not a list of IP addresses —
it's a *behavioral leaderboard* keyed by anonymized client IDs. The
route doc comment says it better than I could:

```rust
/// GET /api/admin/bots — top clients by unusual behavior.
///
/// Zero-PII: rows are keyed by anonymized client_id (never raw IPs). Signals:
/// requests/hour, downloads, export_ratio (downloads/requests — a mirror bot
/// approaches 1.0), failed_auths (credential stuffing), and a composite
/// `bot_score` that ranks clients for admin review. Admin sees *what* looks
/// bot-like and can act (shadowban), not *who* the user is.
```

The query reads from `bot_scores`, the hourly aggregate table created in
migration 009 and filled by `src/bin/bot_scorer.rs` — the same
rollup-on-schedule pattern we just saw, applied to security data. The
aggregation is one SQL statement with a personality:

```rust
let rows = sqlx::query_as::<_, (String, i64, i64, f64, i64, i64)>(
    r#"
    SELECT
        client_id,
        SUM(requests)::bigint AS total_requests,
        SUM(downloads)::bigint AS total_downloads,
        CASE WHEN SUM(requests) = 0 THEN 0
             ELSE (SUM(downloads)::float / SUM(requests)::float)
        END AS export_ratio,
        SUM(failed_auths)::bigint AS total_failed_auths,
        COUNT(*)::bigint AS windows
    FROM bot_scores
    WHERE window_start >= now() - ($1 * interval '1 hour')
      AND client_id <> 'anon'
    GROUP BY client_id
    HAVING SUM(requests) >= $2
    ORDER BY
        (CASE WHEN SUM(requests) = 0 THEN 0
              ELSE (SUM(downloads)::float / SUM(requests)::float) END
         + CASE WHEN SUM(failed_auths) > 0 THEN 0.5 ELSE 0 END) DESC
    LIMIT $3
    "#,
)
.bind(window_hours)
.bind(min_requests)
.bind(limit)
.fetch_all(&state.db)
.await?;
```

Let's decode the signals, because each one is a bot *archetype*:

- **`export_ratio`** — downloads divided by requests. A normal reader
  requests a fic page, maybe downloads an EPUB occasionally: ratio well
  under 1. A mirror bot (a site that mirrors our whole archive) has a
  ratio *approaching 1.0* — nearly every request is a download.
- **`failed_auths`** — credential stuffing. A human mistypes a password
  a few times; a stuffing bot tries thousands. Zero failed auths is
  human; *any* cluster of them is suspicious.
- **`windows`** — how many hourly windows the client appeared in. A
  `burst` flag fires when requests are huge but windows are few: 500
  requests in 2 hours is a burst; 500 over 48 hours might be a heavy
  reader.

The `HAVING SUM(requests) >= $2` filters to clients with at least
`min_requests` (default 10) so the leaderboard doesn't show every
one-request visitor. And the `ORDER BY` is a hand-tuned scoring formula:
sort by (export_ratio + 0.5 if any failed auths), descending. It's not a
machine-learned model — it's a transparent, explainable heuristic, which
is exactly what you want when the output is "humans look at this and
decide."

The handler then turns each row into a JSON object with a **composite
bot score** and **flags** — and this is the part I want you to really
study:

```rust
let leaderboard = rows
    .into_iter()
    .map(|(cid, reqs, dl, ratio, fauths, wins)| {
        // Composite bot score 0..1: heavy on download ratio + failed auths.
        let bot_score = (ratio * 0.7 + if fauths > 0 { 0.3 } else { 0.0 }).min(1.0);
        let flags: Vec<&str> = {
            let mut v = vec![];
            if ratio > 0.9 { v.push("mirror"); }
            if fauths > 3 { v.push("stuffing"); }
            if reqs > 500 && wins <= 2 { v.push("burst"); }
            v
        };
        json!({
            "client_id": cid,
            "requests": reqs,
            "downloads": dl,
            "export_ratio": ratio,
            "failed_auths": fauths,
            "windows": wins,
            "bot_score": bot_score,
            "flags": flags,
            "shadowbanned": state.rate_limiter.is_shadowbanned(&cid),
        })
    })
    .collect::<Vec<_>>();
```

The flags are *named thresholds with reasons*: `mirror`, `stuffing`,
`burst`. A flag without a threshold is a vibe; a flag with a threshold
is a spec. `bot_score` is a weighted blend — download ratio is 70% of
the story, failed auths 30% — clamped to 1.0. And `shadowbanned` is a
live Redis probe: `state.rate_limiter.is_shadowbanned(&cid)` tells the
admin, right in the row, whether this client is already in the penalty
box (we'll meet the shadowban mechanism in Chapter 48).

The action endpoints are then almost absurdly small — because the heavy
lifting lives in the limiter:

```rust
/// POST /api/admin/bots/{client_id}/shadowban — add a client to the Redis
/// shadowban set for the configured TTL (friction: stricter download bucket,
/// never a hard block). Zero-PII: only the anonymized client_id is touched.
pub async fn admin_bot_shadowban(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(client_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    if client_id.is_empty() || client_id == "anon" {
        return Err(AppError::BadRequest(1, "invalid client_id".into()));
    }

    state
        .rate_limiter
        .shadowban(&client_id, state.config.rl_shadowban_ttl);
    Ok(Json(json!({ "err": 0, "client_id": client_id, "shadowbanned": true })))
}
```

Note the guard beyond the role check: `client_id` must not be empty and
must not be `"anon"` — you can't shadowban the anonymous bucket that
everyone without a client ID falls into. Shadowbanning `anon` would
throttle half the internet. Then a single call into the limiter, with
the TTL pulled from config (`rl_shadowban_ttl`, default 24 hours). The
admin *doesn't* specify how long or how hard — the platform does. The
admin's power is "flag this client for friction," not "tune the
penalty."

> 🧪 **Try It Yourself — trace a bot through the whole pipeline.** With
> the server running and a couple of test accounts, fire off a burst of
> download requests from one browser session: loop
> `curl -H "X-Client-Id: my-test-client" "http://localhost:8080/api/v0/epub?q=<some-fic-url>"` a dozen times. Then hit `GET /api/admin/bots?min_requests=5` with an admin token and find `my-test-client` in the leaderboard. Check its `export_ratio` — did your loop push it toward 1.0? Now POST to `/api/admin/bots/my-test-client/shadowban`, wait a minute, and re-run the loop. The requests should start hitting the shadowban bucket's stricter limits (and, if PoW is on, demanding a challenge — Chapter 48). You've just watched your own bot get flagged, judged, and penalized by code you wrote.

## The blacklist: refusing content at the source

The last stop on our tour of admin.rs is the **blacklist** — the
content-level counterpart to user bans. FicHub can blacklist a *fic*
(by `url_id`) or an *author* (by `source_id` + `author_id` on a
platform). Remember the reason codes from the export pipeline in Part 5:
5/7/8 are hard blocks, 6 is a greylist. The endpoint takes a reason, not
a boolean:

```rust
/// POST /api/admin/blacklist/fic — add a fic to the blacklist (upsert).
pub async fn blacklist_fic(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<FicBlacklistBody>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    if body.url_id.trim().is_empty() {
        return Err(AppError::BadRequest(-1, "url_id required".into()));
    }
    let reason = body.reason.unwrap_or(5);

    sqlx::query(
        r#"INSERT INTO fic_blacklist (url_id, reason)
           VALUES ($1, $2)
           ON CONFLICT (url_id, reason) DO UPDATE SET updated = NOW()"#,
    )
    .bind(&body.url_id)
    .bind(reason)
    .execute(&state.db)
    .await?;

    crate::modlog::record_json(&state.db, user.user_id, user.username.clone(), "blacklist_fic", "fic", &body.url_id, vec![("reason", serde_json::json!(reason))]).await;
    Ok(Json(json!({"err": 0, "msg": "fic blacklisted", "url_id": body.url_id, "reason": reason})))
}
```

The `ON CONFLICT (url_id, reason) DO UPDATE SET updated = NOW()` upsert
means blacklisting the same fic twice with the same reason just refreshes
the timestamp — idempotent, no duplicate rows. And once again: modlog
after the mutation, with the reason code in the details bag so the log
answers "why" without a second lookup.

The list endpoint shows off the whole surface in one response:

```rust
/// GET /api/admin/blacklist — list all blacklist entries (fics + authors).
pub async fn list_blacklist(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let fics = sqlx::query_as::<_, (String, i32, chrono::NaiveDateTime)>(
        "SELECT url_id, reason, created::timestamp FROM fic_blacklist ORDER BY created DESC LIMIT 500",
    )
    .fetch_all(&state.db)
    .await?;

    let authors = sqlx::query_as::<_, (i64, i64, i32, chrono::NaiveDateTime)>(
        "SELECT source_id, author_id, reason, created::timestamp FROM author_blacklist ORDER BY created DESC LIMIT 500",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "fics": fics.into_iter().map(|(url_id, reason, created)| json!({
            "url_id": url_id, "reason": reason,
            "created": created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        })).collect::<Vec<_>>(),
        "authors": authors.into_iter().map(|(source_id, author_id, reason, created)| json!({
            "source_id": source_id, "author_id": author_id, "reason": reason,
            "created": created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        })).collect::<Vec<_>>(),
    })))
}
```

Two bounded queries (`LIMIT 500`), two lists, one response. The blacklist
is a tool of last resort, so it needs to be *complete* — an admin adding
a blacklist entry must be able to see what's already there — but it also
needs to be *bounded*, because nobody reads a 50,000-row blacklist.

> ⚠️ **Watch Out — bounded lists, always.** Any admin endpoint that
> returns a list should have a hard cap *in the SQL*, not just in the
> pagination parameters: `LIMIT 500` here, `LIMIT 100` in the curator
> proposal lists (Chapter 51), `LIMIT 200` in the comment triage queue.
> A cap in the handler can be bypassed by a future refactor that forgets
> to clamp; a cap in the query cannot. When in doubt, put the bound in
> both places.

## The Command Center: realtime pulse

Before we close the chapter, one more pattern worth stealing — the
**realtime pulse**. The admin dashboard's front page polls a cheap
endpoint every ~5 seconds so staff get a live pulse of the platform
without SSE or WebSocket infrastructure:

```rust
/// GET /api/admin/realtime — cheap live aggregates for the Command Center.
/// Polls lightweight COUNT(*) windows on request_log + bot_scores (indexed by
/// created / window_start) plus a couple of Redis counters. Zero-PII: counts
/// only, never rows. Avoids SSE/WebSocket infra; the UI polls every ~5s.
pub async fn admin_realtime(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Exports + searches in the last 5 minutes (request_log etypes).
    let (exports_5m, searches_5m): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE etype = 'epub')::bigint,
            COUNT(*) FILTER (WHERE etype = 'search')::bigint
        FROM request_log
        WHERE created >= now() - interval '5 minutes'
        "#
    )
    .fetch_one(&state.db)
    .await?;
```

Two `COUNT(*) FILTER` windows over a five-minute slice of `request_log`,
indexed by `created`. The Redis probe at the bottom has a wonderful
piece of institutional memory in its comment — I'll let it speak for
itself:

```rust
    // Redis counters (best-effort). Use the DEDICATED health_redis connection
    // (never shared state.redis): the bookmark-import worker parks an
    // unbounded BRPOP on state.redis, which would make PING time out and
    // report redis:false even when Redis is healthy (see health.rs).
    let mut redis_ok = false;
    let mut redis_mem: i64 = 0;
    if let Ok(()) = redis::cmd("PING").query_async::<String>(&mut state.health_redis.clone()).await.map(|_| ()) {
        redis_ok = true;
        ...
    }
```

That comment is a bug report fossilized into documentation: someone
once saw "redis: false" on the dashboard, spent hours discovering the
BRPOP worker was eating the shared connection's PING, and wrote the
lesson down so nobody re-walks the trap. When you debug something
weird, *write the why into the code*. Your future self is reading.

> 💡 **Key Concept — Polling beats pushing, until it doesn't.** FicHub's
> Command Center polls a cheap endpoint every 5 seconds instead of
> holding WebSockets open. For a handful of admins, that's strictly
> simpler: no connection lifecycle, no reconnect logic, no message
> broker — and a missed poll just shows up on the next tick. Pushing
> (SSE/WebSockets) earns its complexity only when you have *many*
> clients or *sub-second* latency needs. Start with polling; graduate
> when the product demands it.

## What we learned in Chapter 47

Admin endpoints are the same REST handlers we've been writing all book,
with three disciplines bolted on: **guard early** (role check before any
work), **act then record** (mutation first, best-effort modlog after),
and **never let a request scan a log table** (rollups for the heavy
numbers, bounded queries everywhere). And beneath all three, the design
philosophy that makes them coherent: admin power is a *set of narrow,
named, recorded tools* — not a god button.

The bot leaderboard we just met points forward: the *detection* side of
the anti-bot story lives here in admin.rs, but the *enforcement* side —
the rate-limit tiers, the shadowban set, the honeypots, and the
proof-of-work challenge — lives in the limiter and the routes we're
about to read. In Chapter 48 we strap on the full defense stack, and
it's a lot more interesting than blocking IPs.

---

## Chapter 48 — Anti-Bot Defense: Honeypots, Rate Limits, Shadowbans, and Proof-of-Work

Every public web service has an audience it didn't invite. Somewhere,
right now, there is a script that has discovered FicHub's export
endpoint and is downloading the archive one request at a time — or
trying to register a thousand accounts, or stuffing a password list into
the login form. You cannot delete that script. You cannot argue with it.
All you can do is make its behavior *expensive* while keeping the
platform frictionless for humans.

That's the whole of this chapter: **friction, layered.** FicHub defends
itself with four tools that escalate, each one invisible to real users
and each one a wall to a specific class of bot:

1. **Honeypots** — hidden fields and impossible timing that catch
   dumb bots at the form, silently.
2. **Tiered rate limits** — Redis token buckets per IP *and* per
   `(ip, client_id)`, so a human's shared NAT is never punished by their
   neighbors and a bot can't hide behind one.
3. **Shadowbans** — flagged clients get a stricter bucket, not a block,
   so they keep burning bandwidth without realizing they've been caught.
4. **Proof-of-work** — a shadowbanned client must solve a hashcash
   challenge before the export endpoint serves it: friction measured in
   real CPU.

We'll build all four, read the code that enforces them, and end with the
test suite that proves the math.

## Layer 1: the honeypot that never says "caught"

Here's the thing about a CAPTCHA: it works, but it taxes every single
human who passes through it, and it *tells the bot it was caught* — which
just makes the bot authors build better bots. FicHub's honeypot takes
the opposite approach, and the module doc comment in
`src/routes/honeypot.rs` states the philosophy up front:

```rust
//! Honeypot fields + form-timing traps — silent rejection of
//! human-impersonation bots on user-facing forms (registration, comments).
//!
//! The whole point is to stay invisible: a bot that fills the hidden
//! `website` honeypot or submits in under `MIN_FORM_MS` gets a normal,
//! success-looking response, but nothing is persisted. This never teaches
//! the bot that it was caught, and never challenges a real user.
```

"Never teaches the bot that it was caught" — that's the design goal, and
it's harder than it sounds. A bot that gets an error learns to adapt. A
bot that gets a *success* with no side effects just keeps running,
blind, forever, paying full bandwidth for nothing.

The module defines the contract between frontend and backend as three
constants:

```rust
/// Hidden honeypot input name shared by the frontend forms and the backend
/// checks. Both register and comment forms render a CSS-hidden `<input
/// name="website">`; any non-empty value means a bot filled it (humans
/// never see it).
pub const HONEYPOT_FIELD: &str = "website";

/// Hidden form-open timestamp field. The frontend sets
/// `form_opened_at` (epoch ms, `Date.now()`) when the form mounts;
/// the backend uses it to measure how long the form was open before
/// submission.
pub const OPENED_AT_FIELD: &str = "form_opened_at";

/// Submissions that claim the form was open for less than this are
/// rejected silently — a human cannot type a comment or fill a
/// registration form that fast. The frontend forms never submit this
/// early, so no legitimate user is ever affected.
pub const MIN_FORM_MS: u64 = 500;

/// Maximum accepted age of the `form_opened_at` timestamp. Clock skew,
/// cached pages, or a stale timestamp are all bot-ish signals.
pub const MAX_FORM_AGE_MS: u64 = 60 * 60 * 1000; // 1 hour
```

Two traps in four constants. **Trap one, the honeypot field:** every
form renders an input named `website` that CSS hides from humans. A
human never sees it, never fills it, never submits it — so *any*
non-empty value proves the submitter was a script that scraped the form,
saw an input named `website`, and helpfully filled it in. The trap is a
lie that only bots fall for, because only bots read the DOM as a list of
fields instead of as a page.

**Trap two, the timing trap:** the frontend stamps `form_opened_at` with
`Date.now()` when the form mounts, and submits it with the form. A human
takes more than half a second to type a comment — `MIN_FORM_MS = 500`
says so. A script that POSTs the raw form payload without the timestamp
at all, or with a timestamp from a stale cached page (older than one
hour), is not a human. Three failure modes, one verdict.

The inspection function is a pure function — no I/O, no database, just
inputs and a verdict — which makes it trivially testable:

```rust
/// Result of the honeypot/timing inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapVerdict {
    /// Passed all checks — proceed with the real handler.
    Clean,
    /// Honeypot field was filled, or the form was submitted impossibly
    /// fast. Caller must silently swallow the submission (return a
    /// success-looking response, create nothing).
    RejectSilently,
}

/// Inspect a parsed form submission for bot traps.
pub fn inspect_submission(
    website: Option<&str>,
    form_opened_at: Option<&str>,
    now_ms: u64,
) -> TrapVerdict {
    // Honeypot: any non-empty value means a bot filled a field no human
    // can see.
    if website.map(|w| !w.trim().is_empty()).unwrap_or(false) {
        return TrapVerdict::RejectSilently;
    }

    // Timing trap. Absence of the JS-set field means the submission was
    // not made through the real form.
    let opened_at: u64 = match form_opened_at {
        Some(raw) => match raw.trim().parse() {
            Ok(ms) => ms,
            // Unparsable — not generated by our frontend.
            Err(_) => return TrapVerdict::RejectSilently,
        },
        None => return TrapVerdict::RejectSilently,
    };

    // Absurdly old timestamps (cached/stale) are not human typing.
    if now_ms.saturating_sub(opened_at) > MAX_FORM_AGE_MS {
        return TrapVerdict::RejectSilently;
    }

    // Impossibly fast submission.
    let elapsed = Duration::from_millis(now_ms.saturating_sub(opened_at));
    if elapsed < Duration::from_millis(MIN_FORM_MS) {
        return TrapVerdict::RejectSilently;
    }

    TrapVerdict::Clean
}
```

Read the order of checks like a funnel. First the honeypot — cheapest
signal, hardest to argue with. Then: is the timestamp *present*? A
scripted POST with no `form_opened_at` is rejected immediately — this
catches the crudest bots that don't even render the form, and it's also
why the timing trap doubles as a "must use our frontend" gate. Then: is
it *parseable*? Garbage in, rejected. Then: is it *fresh*? (`MAX_FORM_AGE_MS`
— a replayed submission from a cached page). Then: is it *human-speed*?
And only if all four pass does the submission get the `Clean` verdict.

There's a subtle correctness detail in the `saturating_sub` calls: if a
bot sends a timestamp from the *future* (clock skew, or just messing
with us), `now_ms.saturating_sub(opened_at)` is 0 — not a negative
number that would slip through the comparison. Zero elapsed time is
impossibly fast, so a future timestamp is rejected by the same branch
that rejects 100ms submissions. `saturating_sub` turns a whole class of
weird inputs into one safe verdict. The test suite pins this exact case:

```rust
#[test]
fn future_timestamp_is_rejected_as_stale() {
    // opened_at in the future: saturating_sub gives 0 → too fast.
    let now = 1_700_000_000_000u64;
    assert_eq!(
        inspect_submission(Some(""), Some(&(now + 10_000).to_string()), now),
        TrapVerdict::RejectSilently
    );
}
```

> 🧪 **Try It Yourself — the 500ms boundary.** Look at the test
> `too_fast_submission_is_rejected` in `src/routes/honeypot.rs`. It
> asserts that 499ms is rejected and exactly 500ms passes. Change
> `MIN_FORM_MS` to 2000 and re-run `cargo test honeypot` — the boundary
> tests will fail, telling you exactly which expectations encode the
> constant. That's the value of pinning boundary conditions in tests:
> the moment someone tunes the trap, the tests re-ask every behavioral
> question the module ever answered.

Now, the *caller* side is where "silent" gets real. Registration in
`src/routes/social.rs` (the v1 auth handler) runs the check before doing
any work:

```rust
    // ── Honeypot + timing trap (silent rejection) ──────────────────────
    // A bot that fills the hidden `website` field, or submits without the
    // JS-set `form_opened_at` (or impossibly fast), gets a success-looking
    // response and no account. See `crate::routes::honeypot`.
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let verdict = honeypot::inspect_submission(
        body.website.as_deref(),
        body.form_opened_at.as_deref(),
        now_ms,
    );
    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("registration silently rejected (honeypot/timing trap)");
        // Success-looking response with no token — the bot "registered"
        // without creating anything.
        return Ok(Json(json!({
            "err": 0,
            "token": "",
            "user": {
                "id": 0,
                "username": body.username,
                "role": 0,
                "reputation": 0,
                "email": null,
            },
        })));
    }
```

Read that response body again. `err: 0`. An empty token. A user object
with id 0. The bot's "registration" *succeeded* — it just didn't create
anything, and the bot's logic will merrily move on to "login" with a
token that can't possibly work, still not realizing it was caught. A
log line fires (`tracing::warn!`) so the *humans* can see the trap
working, but the bot gets nothing but a warm, empty success.

The comment form gets the identical treatment, in
`src/routes/social.rs`'s `add_comment_handler`:

```rust
    // ── Honeypot + timing trap (silent rejection) ──────────────────────
    // Same trap as registration: a bot that fills `website` or submits too
    // fast gets a success-looking response and the comment is dropped.
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let verdict = honeypot::inspect_submission(
        body.website.as_deref(),
        body.form_opened_at.as_deref(),
        now_ms,
    );
    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("comment silently rejected (honeypot/timing trap)");
        return Ok(Json(json!({
            "err": 0,
            "comment_id": 0,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "username": auth.username,
        })));
    }
```

Same shape: success-looking response, nothing persisted. And because
the frontend `CommentBody` and `RegisterRequest` structs carry
`#[serde(default)] pub website: Option<String>` and the timestamp field,
the *same* JSON contract works for both forms. Two handlers, one trap,
zero CAPTCHAs, zero humans bothered.

> ⚠️ **Watch Out — the honeypot is not a security boundary.** A
> determined attacker reading your JavaScript will notice the hidden
> field, notice the timestamp, and strip both. The honeypot's job is to
> silently filter the *dumb* bots — the ones that make up 95% of the
> noise — for nearly zero cost and zero user friction. The smart bots
> are the job of the next three layers. Never design your defense so
> that the cheap layer is also the *only* layer.

## Layer 2: the tiered token bucket in Redis

The second layer is the rate limiter, and this is where FicHub gets
clever. Open `src/limiter/mod.rs` and you'll find the *contract* — two
traits and the tier vocabulary:

```rust
/// Endpoint-class tiers for the tiered rate limiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// /api/epub, /api/v0/epub, /api/download/*, /cache/*, /api/upload* —
    /// strict (60/hr per IP by default).
    Download,
    /// /api/auth/*, /login, /register — strict (10/min per IP).
    Auth,
    /// /api/search, /docs, /static — very high (1000/min).
    Search,
    /// Everything else — moderate (the legacy global bucket).
    Default,
}
```

The insight here is that not all endpoints are equally dangerous. A
download is expensive (it triggers a scrape and builds an EPUB) and is
what a mirror bot wants — so it gets a *tight* tier. An auth attempt is
what a stuffing bot wants — tight too. A search is cheap and what real
readers do constantly — so it gets a *huge* tier; throttling search
would throttle the actual product. Rate limiting by *endpoint class*
instead of "one limit for everything" is the difference between
protecting the service and strangling it.

The trait adds the shadowban vocabulary to the contract:

```rust
#[async_trait::async_trait]
pub trait TieredRateLimiter: Send + Sync {
    /// Check a request against the tier bucket for `tier`. `client_id` is the
    /// `X-Client-Id` header value when present.
    async fn check(&self, ip: IpAddr, client_id: Option<&str>, tier: Tier) -> TieredRateLimitResult;

    /// Map a request path to its tier.
    fn tier_for_path(&self, path: &str) -> Tier;

    /// True when `client_id` is shadowbanned (in the `fichub:shadowban` set).
    fn is_shadowbanned(&self, client_id: &str) -> bool;

    /// Add a client_id to the shadowban set for `ttl_seconds` (friction, not a
    /// hard block: shadowbanned clients get a much stricter download bucket).
    fn shadowban(&self, client_id: &str, ttl_seconds: u64);

    /// Remove a client_id from the shadowban set (admin un-shadowban).
    fn unshadowban(&self, client_id: &str);
}
```

The concrete implementation is `RedisBucketLimiter` in
`src/limiter/redis_bucket.rs`. The heart of it is a token bucket
implemented in **Lua, executed atomically inside Redis** — this is a
pattern worth understanding deeply, because it's how you do
compare-and-check rate limiting without race conditions:

```rust
        let lua_script = r#"
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
"#;
```

Let's walk the token bucket math, because it's the single most
reusable idea in this chapter. Each bucket has two state fields stored
in a Redis hash: `value` (tokens currently in the bucket) and
`last_drain` (when we last computed the drain). The bucket starts full
(`value = capacity` on first touch). Tokens *refill continuously* —
`new_tokens = min(capacity, value + elapsed * flow)` — where `flow` is
tokens per second. So a bucket with capacity 10 and flow 60/3600 (60
per hour) allows a burst of 10 immediately, then refills one token per
minute. `allowed = new_tokens - requested`: if non-negative, the
request is allowed and we store the drained bucket; if negative, we
return the *wait time* — how many seconds until enough tokens have
refilled — and the caller turns that into a 429 `retry_after`.

Running this in Lua has one enormous advantage: **atomicity**. The
check-and-drain is one `EVALSHA` — Redis executes the whole script
without interleaving other commands, so two simultaneous requests from
the same IP can never both read `value = 1` and both pass. If you
implemented this in Rust with GET + SET you'd have a race condition
that lets a clever bot drain a bucket to negative. The Lua script is
the fix.

> 💡 **Key Concept — Atomic check-and-set with Lua.** Whenever you need
> "read state, decide, write state" to be race-free — rate limits,
> counters, idempotency keys, anything shared — push the decision into
> Redis Lua and call it with `EVALSHA`. One round trip, no locks, no
> races, and Redis guarantees the script runs atomically. It's one of
> the few patterns in distributed systems that is simultaneously simple
> and correct.

The tier parameters come from config, with defaults that encode the
product's priorities. From `src/config.rs`:

```rust
        // ── Tiered rate limiting (anti-bot) ─────────────────────────
        // Tiers are (capacity, flow-tokens/sec). Defaults implement:
        //   download: 10 burst, refills 60/hr  = 60 downloads/hour, burst 10
        //   auth:     10 burst, refills 10/min = 10 auth attempts/minute
        //   search:   1000 burst, refills 1000/min (very high — bots don't hurt)
        //   shadowban download: 5 burst, refills 5/hr (friction, not a hard block)
        let rl_download_capacity = env_f64("RL_DOWNLOAD_CAPACITY", 10.0);
        let rl_download_flow = env_f64("RL_DOWNLOAD_FLOW", 60.0 / 3600.0);
        let rl_auth_capacity = env_f64("RL_AUTH_CAPACITY", 10.0);
        let rl_auth_flow = env_f64("RL_AUTH_FLOW", 10.0 / 60.0);
        let rl_search_capacity = env_f64("RL_SEARCH_CAPACITY", 1000.0);
        let rl_search_flow = env_f64("RL_SEARCH_FLOW", 1000.0 / 60.0);
        let rl_client_bonus_capacity = env_f64("RL_CLIENT_BONUS_CAPACITY", 5.0);
        let rl_client_bonus_flow = env_f64("RL_CLIENT_BONUS_FLOW", 30.0 / 3600.0);
        let rl_nat_multiplier = env_f64("RL_NAT_MULTIPLIER", 4.0);
        let rl_shadowban_capacity = env_f64("RL_SHADOWBAN_CAPACITY", 5.0);
        let rl_shadowban_flow = env_f64("RL_SHADOWBAN_FLOW", 5.0 / 3600.0);
        let rl_shadowban_ttl = std::env::var("RL_SHADOWBAN_TTL")
            .unwrap_or_else(|_| "86400".to_string()) // 24h
            .parse::<u64>()
            .unwrap_or(86400);
        let rl_tiered_enabled = std::env::var("RL_TIERED_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
```

Every knob is an env var with a sane default — `RL_TIERED_ENABLED` can
flip the whole system off for testing (we'll see that path in a
moment), and each tier can be tuned without a redeploy. The shadowban
defaults deserve a second look: `5 burst, refills 5/hr` and a 24h TTL.
A shadowbanned client gets *5 downloads an hour*. A human who was
accidentally flagged can still read — slowly — while a mirror bot's
bandwidth collapses to 5 EPUBs per hour. That's the "friction, not a
block" philosophy made numeric.

The tier mapping is a pure function, which makes it testable in
isolation:

```rust
/// Pick the rate-limit tier for a request path. Pure function so it is
/// trivially unit-testable.
pub fn tier_for_path(path: &str) -> Tier {
    if path.starts_with("/api/epub")
        || path.starts_with("/api/v0/epub")
        || path.starts_with("/api/download")
        || path.starts_with("/cache/")
        || path.starts_with("/api/upload")
        || path.starts_with("/legacy/epub_export")
    {
        Tier::Download
    } else if path.starts_with("/api/auth/")
        || path.starts_with("/login")
        || path.starts_with("/register")
    {
        Tier::Auth
    } else if path.starts_with("/api/search")
        || path.starts_with("/docs")
        || path.starts_with("/static")
    {
        Tier::Search
    } else {
        Tier::Default
    }
}
```

Path prefixes, not exact matches — so `/api/v0/epub?q=...` and
`/api/download/abc` both land in Download without a table of every URL.

## The two-key problem: NAT and the (ip, client_id) bucket

Now the genuinely interesting part of the design. A naive rate limiter
keys on IP alone, and that has two failure modes: a whole office behind
one NAT (or a whole telco behind CGNAT) shares one bucket and gets
throttled for each other's sins; and a bot can churn through IPs. The
client_id header — remember the UUID stored in localStorage from
Part 7 — gives FicHub a stable per-*browser* identity. The design uses
**both keys, in a specific order**, and the `check_tiered` function is
where it happens:

```rust
    /// Enforce the tiered limit for a request. Bucket keys:
    ///   `rate:tier:{tier}:ip:{ip}`
    ///   `rate:tier:{tier}:client:{ip}:{client_id}` (only when client_id present)
    ///
    /// A shadowbanned client_id is routed to the shadowban bucket for the
    /// download tier (its `(ip, client_id)` key uses the stricter params).
    pub async fn check_tiered(
        &self,
        ip: IpAddr,
        client_id: Option<&str>,
        tier: Tier,
    ) -> TieredRateLimitResult {
        if !self.tiered_enabled || !self.dynamic_rate_limit {
            // Test/dev mode: legacy static delay, always allowed.
            let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
            return TieredRateLimitResult::Allowed;
        }

        let is_shadowbanned = match client_id {
            Some(c) => self.is_shadowbanned(c),
            None => false,
        };
        let (_, _, shadow_cap, shadow_flow) = self.tier_params(tier);

        // Per-IP bucket first (the NAT ceiling). Identified clients get a
        // scaled ceiling so one client can't starve the IP.
        let ip_cap = if client_id.is_some() {
            self.nat_scaled_capacity(tier)
        } else {
            let (cap, _, _, _) = self.tier_params(tier);
            cap
        };
        let ip_flow = {
            let (_, flow, _, _) = self.tier_params(tier);
            flow
        };
        let ip_key = format!("rate:tier:{:?}:ip:{}", tier, ip);
        let ip_wait = self
            .check_bucket(&ip_key, ip_cap, ip_flow)
            .await
            .unwrap_or(-1.0);
        if ip_wait > 0.0 {
            return TieredRateLimitResult::Wait(ip_wait.ceil() as u64);
        }

        // Per-(ip, client_id) bucket. The download tier uses the shadowban
        // params when the client is flagged; other tiers use the bonus params.
        if let Some(cid) = client_id {
            let client_key = format!("rate:tier:{:?}:client:{}:{}", tier, ip, cid);
            let (client_cap, client_flow) = if tier == Tier::Download && is_shadowbanned {
                (shadow_cap, shadow_flow)
            } else {
                (
                    ip_cap + self.client_bonus_capacity,
                    ip_flow + self.client_bonus_flow,
                )
            };
            let client_wait = self
                .check_bucket(&client_key, client_cap, client_flow)
                .await
                .unwrap_or(-1.0);
            if client_wait > 0.0 {
                return TieredRateLimitResult::Wait(client_wait.ceil() as u64);
            }
        }

        TieredRateLimitResult::Allowed
    }
```

Let's decode the two-level design carefully, because it solves a real
production problem.

**Level one, the IP bucket — the NAT ceiling.** Every request checks
the IP bucket first. For anonymous requests (no client_id) it uses the
tier's base capacity; for *identified* requests it uses the
`nat_scaled_capacity`, which multiplies the base by `nat_multiplier`
(default 4):

```rust
    /// Effective per-IP bucket when the request carries a client_id: scale by
    /// `nat_multiplier` so one abusive identified client cannot exhaust the
    /// whole IP's allowance, while still capping the IP as a whole.
    fn nat_scaled_capacity(&self, tier: Tier) -> f64 {
        let (cap, _, _, _) = self.tier_params(tier);
        if self.nat_multiplier > 1.0 {
            cap * self.nat_multiplier
        } else {
            cap
        }
    }
```

The reasoning: an IP with *identified* clients on it is probably a
shared NAT (a coffee shop, a university), so its ceiling gets raised —
we don't want three students in a dorm to share one 10-download bucket.
But the IP bucket *still exists*, so a bot farm hammering from one IP
can't do 10 clients × 10 downloads either. The IP is the ceiling; the
multiplier is the acknowledgment that ceilings must breathe for shared
networks.

**Level two, the (ip, client_id) bucket — the per-browser allowance.**
When a client_id is present, the limiter checks a second bucket keyed
by *both* IP and client_id. This bucket gets the base capacity *plus* a
bonus (`client_bonus_capacity`, default 5), because it's the honest
per-user allowance and should be slightly generous. The dual key
prevents the classic spoofing trick: a bot rotating fake client_ids
would get a fresh `(ip, client_id)` bucket per ID, but they'd *still*
hit the IP ceiling at level one, because the IP bucket doesn't care
what ID you claim. Two keys, two jobs: the client bucket measures the
individual, the IP bucket measures the crowd.

And **this is exactly where the shadowban plugs in**:

```rust
            let (client_cap, client_flow) = if tier == Tier::Download && is_shadowbanned {
                (shadow_cap, shadow_flow)
            } else {
                (
                    ip_cap + self.client_bonus_capacity,
                    ip_flow + self.client_bonus_flow,
                )
            };
```

A shadowbanned client_id *in the download tier* gets the shadowban
params (5 burst, 5/hr) instead of the bonus params. Not blocked —
*throttled to a crawl*. And only on the download tier, because
downloads are the expensive resource bots want. The shadowban is one
`if` inside the existing limiter, which is exactly how a good feature
should feel: the infrastructure was already there, the flag just
switches which bucket parameters apply.

> 💡 **Key Concept — Two keys beat one key.** Any rate limiter keyed on
> a single dimension (IP, user ID, API key) has a blind spot: IPs
> punish innocent neighbors, user IDs are forgeable. Keying on
> *both* — a coarse crowd-level key and a fine individual-level key —
> covers both blind spots: the individual key gives honest per-user
> limits, and the crowd key catches rotation. Whenever you design a
> limiter or a cache or a counter, ask: *which two keys would make this
> robust?*

Before moving on, note the fail-open discipline running through this
whole function: every Redis call ends in `.unwrap_or(-1.0)` — meaning
"if Redis hiccups, pretend there's no limit and let the request
through." A rate limiter that blocks everyone during a Redis outage is
worse than no limiter; availability wins, and the bots can have their
one bad minute.

## Layer 3: the shadowban set in Redis

The shadowban set itself is beautifully simple. It's a Redis SET of
client_ids, `fichub:shadowban`, with per-member expiry:

```rust
pub const SHADOWBAN_SET: &str = "fichub:shadowban";
```

The two operations are SADD and SISMEMBER, plus a per-member TTL:

```rust
    /// Blocking `SADD` + per-member TTL (fail-open: `()` on error).
    fn block_on_sadd(&self, client_id: &str, ttl_seconds: u64) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.clone();
        let member_key = format!("{}:member:{}", SHADOWBAN_SET, client_id);
        let mut pipe = redis::pipe();
        pipe.cmd("SADD")
            .arg(SHADOWBAN_SET)
            .arg(client_id)
            .cmd("PEXPIRE")
            .arg(&member_key)
            .arg((ttl_seconds * 1000) as i64);
        let fut = pipe.query_async::<()>(&mut conn);
        let _: Result<(), redis::RedisError> = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        });
        Ok(())
    }
```

The design comment explains the lazy-expiry trick:

```rust
    /// True when `client_id` is in the shadowban set (expiring entries).
    /// Lazy per-member expiry: a member whose per-member TTL key is gone is
    /// pruned on the next probe, keeping SISMEMBER fast and the set bounded.
```

The SET itself never expires — individual members do, via a sidecar
`fichub:shadowban:member:<id>` key with a TTL. `SISMEMBER` stays fast
(no per-member TTL scanning), and stale members get pruned lazily.
This is the pattern for "a set where membership expires": the set holds
membership, a parallel key holds the expiry, and probes are O(1).

The admin's shadowban action from Chapter 47 is just `SADD` with the
configured TTL — which is why the admin handler was so short. And the
whole thing is **fail-open**: `block_on_sismember` returns `false` on
any Redis error, so a Redis hiccup never accidentally treats the whole
internet as shadowbanned:

```rust
    fn block_on_sismember(&self, client_id: &str) -> bool {
        let mut conn = self.redis.clone();
        let mut cmd = redis::cmd("SISMEMBER");
        cmd.arg(SHADOWBAN_SET).arg(client_id);
        let fut = cmd.query_async::<bool>(&mut conn);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        })
        .unwrap_or(false)
    }
```

> ⚠️ **Watch Out — fail-open vs fail-closed is a product decision, not
> an implementation detail.** FicHub's security layers are almost all
> fail-open (a limiter outage means no limiting; a shadowban probe
> outage means no shadowbans) because the cost of a false positive —
> blocking a real reader — is worse than a temporary false negative.
> Some systems must be fail-closed (a firewall, an auth check). Decide
> per-layer, and write the decision into the code comment so nobody
> "fixes" it later.

## Layer 4: proof-of-work — make the bot pay in CPU

Rate limits and shadowbans slow a bot down, but they don't make the bot
*expensive to run*. A mirror bot can run 24/7 at 5 downloads/hour
forever. The final layer changes the economics: **proof-of-work**.

The pure math lives in `src/services/pow.rs`, and it's hashcash — the
1997 anti-spam construction that predates every modern "proof of work"
system you've heard of:

```rust
//! Hashcash-style proof-of-work (PoW) challenges for flagged clients.
//!
//! When a client is shadowbanned/flagged (see `src/limiter/`), the export
//! endpoint refuses to serve the request until the client proves it spent
//! real CPU work. The challenge is a random hex string; the client must find
//! a `nonce` such that `SHA-256(challenge || nonce)` (hex) starts with
//! `difficulty` zero bits (difficulty 16 → the first 4 hex chars are `0000`).
//!
//! This is friction, not a hard block: humans (or a tiny bit of browser JS)
//! never notice a ~65k-hash solve, while bulk bots pay ~1000x per request.
```

Here's the game. The server issues a random challenge: 16 random bytes,
hex-encoded (32 hex chars). The client must find a nonce such that
`SHA-256(challenge || nonce)` starts with a required number of zero
bits. At difficulty 16 (the default), the hash must start with `0000`
— four hex zeros — which is a 1-in-65,536 chance per guess, so the
expected work is ~65,536 hashes. A modern laptop does that in
milliseconds. A bot doing 1,000 downloads pays 65 *million* hashes —
seconds of CPU per download, and it's pure overhead the bot can never
reuse.

The verification is a pure function, and the doc comment shows the
tests are built on *known-good vectors* — precomputed pairs pinned so
the math can never drift:

```rust
/// Verify a `(challenge, nonce)` pair: the SHA-256 hex of
/// `challenge || nonce` must start with `difficulty` zero bits.
///
/// The nonce is the decimal ASCII encoding of an unsigned counter, matching
/// the classic hashcash construction. Returns `false` (never panics) for a
/// non-numeric nonce.
pub fn verify_solution(challenge: &str, nonce: &str, difficulty: u32) -> bool {
    if difficulty == 0 {
        // Zero-bit difficulty is trivially satisfiable by any nonce.
        return nonce.chars().all(|c| c.is_ascii_digit());
    }
    if !nonce.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(challenge.as_bytes());
    hasher.update(nonce.as_bytes());
    let digest = hasher.finalize();
    let hex_digest = hex::encode(digest);
    hex_digest.starts_with(&difficulty_to_hex_prefix(difficulty))
}
```

The prefix mapping handles the bit-to-hex conversion, including the
awkward non-multiple-of-4 case:

```rust
/// The minimum hex prefix a valid solution hash must start with.
///
/// `difficulty` counts leading zero BITS; each hex char is 4 bits, so the
/// required prefix is `ceil(difficulty / 4)` zero chars. Difficulty 16 →
/// `"0000"`. A non-multiple-of-4 difficulty (e.g. 17) still requires
/// `ceil(17/4) = 5` zero hex chars, which is slightly *harder* than 17 bits
/// — an acceptable granularity for a friction layer.
pub fn difficulty_to_hex_prefix(difficulty: u32) -> String {
    let nibbles = difficulty.div_ceil(4);
    "0".repeat(nibbles as usize)
}
```

And the tests — I want you to see how a cryptographic primitive gets
pinned down properly. Known-good vectors, cross-difficulty rejection,
non-numeric nonce rejection, even a NIST reference hash:

```rust
    /// A known-good (challenge, nonce, difficulty) vector computed with the
    /// reference Python implementation. SHA-256("unit-test-challenge241")
    /// starts with `00` (8 zero bits).
    #[test]
    fn verify_known_good_vector_difficulty_8() {
        assert!(verify_solution("unit-test-challenge", "241", 8));
    }

    /// A known-good difficulty-16 vector: SHA-256("fichub-db-test-challenge17269")
    /// starts with `0000` (16 zero bits).
    #[test]
    fn verify_known_good_vector_difficulty_16() {
        assert!(verify_solution("fichub-db-test-challenge", "17269", 16));
    }

    /// A wrong nonce for the same challenge must fail.
    #[test]
    fn verify_rejects_wrong_nonce() {
        assert!(!verify_solution("unit-test-challenge", "242", 8));
    }

    /// Non-numeric nonces are never valid (no panic).
    #[test]
    fn verify_rejects_non_numeric_nonce() {
        assert!(!verify_solution("unit-test-challenge", "abc", 8));
        assert!(!verify_solution("unit-test-challenge", "12a", 8));
        assert!(!verify_solution("unit-test-challenge", "-5", 8));
    }
```

The `"241"` and `"17269"` nonces are the output of a reference solver
run once, pinned forever. If anyone ever changes the hash construction
(swaps SHA-256 for something else, changes the concatenation order),
these tests fail instantly — the fastest possible signal that the
crypto contract broke.

> 🧪 **Try It Yourself — solve a challenge by hand (well, by loop).**
> In a scratch Rust file (or Python), implement the solver: take
> challenge `"fichub-db-test-challenge"`, loop `nonce` from 0 upward,
> hash `challenge + nonce` with SHA-256, and stop when the hex starts
> with `0000`. Time it — it should find `17269` in well under a second
> (Python) or a few milliseconds (Rust). Now raise the target to
> `00000` (difficulty 20) and watch the time jump ~16x. That's the
> entire economics of the layer: the difficulty knob is a CPU tax you
> can set per-client, and it costs you nothing to issue.

The full challenge lifecycle spans three files, and it's worth tracing
end to end because it shows how a small feature threads through the
whole architecture:

**1. Issuing** — `GET /api/pow/challenge` in `src/routes/pow.rs`.
Fail-open first: no client_id, or not shadowbanned, and the response is
`{"err": 0, "not_needed": true}` — normal traffic literally never sees
the feature:

```rust
pub async fn challenge_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let Some(cid) = client_id else {
        return Ok(Json(json!({
            "err": 0,
            "not_needed": true,
            "message": "no client id",
        })));
    };

    // Fail-open: a Redis hiccup means is_shadowbanned() is false, so normal
    // traffic is never accidentally challenged.
    if !state.rate_limiter.is_shadowbanned(&cid) {
        return Ok(Json(json!({
            "err": 0,
            "not_needed": true,
        })));
    }

    let challenge = pow::generate_challenge(
        state.config.pow_difficulty,
        state.config.pow_ttl_secs,
    );
    Ok(Json(json!({
        "err": 0,
        "not_needed": false,
        "challenge": challenge.challenge,
        "difficulty": challenge.difficulty,
        "expires_at": challenge.expires_at,
    })))
}
```

**2. Solving** — `POST /api/pow/solve`. The client sends back the
challenge and its found nonce. Sanity-check the challenge shape (32 hex
chars), verify the solution, and — on success — store a solved marker
in Redis with a TTL so the client only pays once per challenge, not per
request:

```rust
pub async fn solve_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SolveRequest>,
) -> Result<Json<Value>, AppError> {
    // Only shadowbanned clients are allowed to solve (the challenge is
    // meaningless for everyone else).
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(ref cid) = client_id {
        if !state.rate_limiter.is_shadowbanned(cid) {
            return Ok(Json(json!({
                "err": 0,
                "not_needed": true,
            })));
        }
    }

    // Basic sanity: the challenge must be a plausible 32-hex-char value.
    if body.challenge.len() != 32 || !body.challenge.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::RateLimited(0));
    }

    // Remember the challenge we are about to accept so the export gate can
    // tie the solve to the challenge it actually checks.
    if let Some(ref cid) = client_id {
        pow::set_latest_challenge_for_client(&state, cid, &body.challenge);
    }

    let difficulty = state.config.pow_difficulty;
    if pow::verify_solution(&body.challenge, &body.nonce, difficulty) {
        let _ = pow::store_solution(
            &mut state.redis.clone(),
            &body.challenge,
            state.config.pow_ttl_secs,
        )
        .await;
        return Ok(Json(json!({
            "err": 0,
            "solved": true,
        })));
    }

    // Invalid solution → 429 with a fresh challenge (friction, not a block).
    let challenge = pow::generate_challenge(difficulty, state.config.pow_ttl_secs);
    Err(AppError::RateLimitedJson(json!({
        "err": -429,
        "msg": "invalid proof of work",
        "challenge": challenge.challenge,
        "difficulty": challenge.difficulty,
        "expires_at": challenge.expires_at,
    })))
}
```

**3. Enforcing** — the export gate in `src/routes/export.rs`. This is
the line in the sand: a shadowbanned client whose latest challenge
isn't solved gets a 429 *with a fresh challenge embedded in the error
body*, so the client can solve and retry in one round trip:

```rust
    // ── Proof-of-work gate (shadowbanned clients only) ───────────────
    // A flagged client must solve a hashcash-style challenge before the
    // export is served. The challenge is re-issued (429 + fresh challenge)
    // until the client POSTs a valid solve, which is cached in Redis for
    // POW_TTL_SECS (10 min by default) so the follow-up export requests
    // pass without re-solving. Non-shadowbanned clients and requests
    // without a client_id skip this entirely. Redis errors fail open: a
    // hiccup re-issues the challenge instead of un-blocking a bot.
    if let Some(ref cid) = client_id {
        if state.rate_limiter.is_shadowbanned(cid) {
            let last_challenge = pow::latest_challenge_for_client(&state, cid);
            let solved = pow::solution_solved(
                &mut state.redis.clone(),
                last_challenge.as_deref().unwrap_or(""),
            )
            .await;
            if !solved {
                let challenge = pow::generate_challenge(
                    state.config.pow_difficulty,
                    state.config.pow_ttl_secs,
                );
                pow::set_latest_challenge_for_client(&state, cid, &challenge.challenge);
                return Err(AppError::RateLimitedJson(json!({
                    "err": -429,
                    "msg": "proof of work required",
                    "challenge": challenge.challenge,
                    "difficulty": challenge.difficulty,
                    "expires_at": challenge.expires_at,
                })));
            }
        }
    }
```

Then — and only then — the handler proceeds to the tiered rate-limit
check and the actual export. The order matters: **PoW first, then the
bucket.** A bot that hasn't solved never even reaches the bucket, and a
bot that solved once gets through the PoW gate on every follow-up
request (thanks to the Redis-solved marker) and then pays the
shadowban bucket's 5-per-hour toll. Two taxes on the same client,
each one cheap to levy.

> 💡 **Key Concept — Proof of work is a tax, not a lock.** The elegant
> property of hashcash is that cost is *tunable and client-side*: the
> server verifies with one hash, the client pays with 65,536. You can
> set difficulty per client, per endpoint, or per time-of-day without
> touching any bot's code. For a flagged client, the tax makes
> automation uneconomical; for a human, the tax is invisible. Whenever
> you face "how do I stop abuse without punishing users," ask whether
> the abuser's *cost structure* can be changed instead of trying to
> detect them perfectly.

## The layer cake, served

Put it together and the defense reads like an onion, from cheapest to
most expensive, from dumbest bot to smartest:

1. **Honeypot + timing** catches form bots silently at the front door —
   zero cost, zero friction, no CAPTCHAs.
2. **Tiered token buckets** throttle by endpoint class and by two keys,
   so shared NATs breathe and rotation gets caught at the IP ceiling.
3. **Shadowban** downgrades a flagged client's bucket to near-zero
   without ever telling the bot it's flagged.
4. **Proof-of-work** makes every remaining download cost real CPU, paid
   per challenge, verified with one hash.

Every layer is fail-open, every knob is a config env var with a
tested default, and every boundary — the 500ms timing line, the
`0000` prefix, the tier mapping — is pinned by unit tests. That's the
whole story of building anti-bot defense that a real product can live
with: **detect cheaply, throttle fairly, penalize invisibly, and never
let the defense hurt the humans it's protecting.**

In Chapter 49, we turn from defending the platform to *measuring* it:
the `usage_events` pipeline that tells us who's browsing versus who's
acting — still with the same zero-PII discipline you saw in the bot
leaderboard.

---

## Chapter 49 — Usage Analytics: Views vs Actions, Without the PII

Here's a question every platform eventually asks: *are people actually
using this?* Not "how many accounts" — accounts lie; people register and
never return. Not "how many requests" — requests include bots. The
question is: how many *humans* came, and of those, how many *did*
something — downloaded, voted, commented, bookmarked?

FicHub's answer is `usage_events`, and the design decision at its heart
is one of the most instructive in this whole book: **you can measure
engagement without measuring people.** The system tracks an anonymous
client_id — a UUID generated in the browser, stored in localStorage —
and never ties it to an account, an email, or an IP. It records *what
path was hit* and *whether it was a view or an action*. That's the
entire dataset. And from that tiny, PII-free table, the admin dashboard
derives unique visitors, return rates, active-user counts, and the
viewers-vs-doers split that tells you whether the platform is a library
people browse or a product people use.

## The schema: three columns of intent

Migration 033 says it all in its header comment:

```sql
-- Anonymous usage analytics (non-PII). Every API request from the frontend
-- carries an X-Client-ID header (UUID in localStorage) — this table records
-- each request's path + when it happened so admins can see unique
-- daily/weekly/monthly visitors and distinguish view-only visitors from
-- active users (those who performed actions beyond browsing).
CREATE TABLE IF NOT EXISTS usage_events (
    id         BIGSERIAL PRIMARY KEY,
    client_id  TEXT NOT NULL,
    path       TEXT NOT NULL,
    event_type TEXT NOT NULL DEFAULT 'view'
               CHECK (event_type IN ('view', 'action')),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_usage_events_client_created
    ON usage_events (client_id, created_at);
CREATE INDEX IF NOT EXISTS idx_usage_events_created
    ON usage_events (created_at);
CREATE INDEX IF NOT EXISTS idx_usage_events_type_created
    ON usage_events (event_type, created_at);
```

Five columns. `client_id` is the anonymous browser UUID. `path` is the
API path that was hit. `event_type` is the interesting one — a `CHECK`
constraint that forces every row to be either `'view'` or `'action'` —
and `user_agent` is stored raw (truncated at write time, as we'll see)
because knowing "is this traffic mostly mobile Safari or mostly
curl?" is genuinely useful for a platform owner, and a user agent
string is not PII on its own.

The three indexes tell you exactly which queries the table was built to
serve: unique visitors per client over time (`client_id, created_at`),
raw time-series (`created_at`), and the view/action split
(`event_type, created_at`). Indexes are a great way to read a team's
intent: these three indexes say "we will count distinct clients per
bucket, filter by type, and always sort by time."

> 💡 **Key Concept — The CHECK constraint is the schema's personality.**
> `event_type IN ('view', 'action')` means the *database itself*
> refuses to store anything else. A typo in a future code path
> (`event_type = 'actoin'`) fails at INSERT time instead of silently
> creating a third category that breaks every aggregation downstream.
> When a column has a small, closed vocabulary, put the vocabulary in
> the database — it's the cheapest invariant checker you'll ever have.

Where does the client_id come from? The frontend generates it once,
lazily, and reuses it forever — from `frontend/src/lib/api/client.ts`:

```ts
// Client ID management for anonymous usage tracking
function getClientId(): string {
  const STORAGE_KEY = 'fichub_client_id';
  let clientId = localStorage.getItem(STORAGE_KEY);
  if (!clientId) {
    // Generate UUID v4
    clientId = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
    localStorage.setItem(STORAGE_KEY, clientId);
  }
  return clientId;
}
```

Note what this is *not*: it's not a cookie (no server involvement, no
cross-site tracking), it's not tied to login (guests have one too), and
it's not a fingerprint. It's a random UUID that says "this browser has
been here before." That one decision — anonymous, client-side, stable —
is what makes every downstream number in this chapter both useful and
privacy-safe. The same ID is used by the rate limiter and the shadowban
set from Chapter 48, so one identity primitive serves defense *and*
analytics.

## The middleware: record everything, block nothing

The recording happens in Axum middleware — `track_usage` in
`src/routes/analytics.rs` — which is registered in `src/server.rs`
around the whole router:

```rust
        // Middleware
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::routes::analytics::track_usage,
        ))
```

The middleware itself is a masterclass in "instrumentation that can
never hurt the request":

```rust
/// Axum middleware that records a usage event (view or action) per request
/// carrying an X-Client-ID header. Best-effort + non-blocking: the insert is
/// spawned on a background task so it never adds latency to the response.
pub async fn track_usage(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let client_id = req
        .headers()
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() <= 128);

    let path = req.uri().path().to_string();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(256).collect::<String>());

    let db = state.db.clone();
    if let Some(cid) = client_id {
        let event_type = if is_action(&path) { "action" } else { "view" };
        // Fire-and-forget: record after the response, never block the caller.
        tokio::spawn(async move {
            crate::db::queries::insert_usage_event(&db, &cid, &path, event_type, user_agent.as_deref()).await;
        });
    }

    next.run(req).await
}
```

Let me count the ways this middleware protects the request it's
instrumenting:

- **It reads, it never writes to the request.** The original request is
  passed through untouched via `next.run(req).await`.
- **It skips requests without a valid client_id.** The `.filter(|s| !s.is_empty() && s.len() <= 128)` means a missing, empty, or absurdly long header produces no event — and an attacker can't make you store a 10MB header value.
- **It truncates the user agent** at 256 chars with `.chars().take(256)` — defensive against anything a weird client sends.
- **It spawns the insert on a background task.** The database write
  happens *after* the response, on a `tokio::spawn`'d task, so a slow
  insert can never add a millisecond to the user's request. This is the
  fire-and-forget pattern, and it's the single most important line for
  production safety: analytics that slow down the product are analytics
  that get deleted.
- **The insert itself is best-effort**, as we'll see in the query
  function — a failed insert logs a warning and returns.

The classification lives in a tiny pure function with a constant
table — the *definition* of "action" is a curated list of path
prefixes:

```rust
/// Paths that count as ACTIONS (something beyond browsing). Exports,
/// downloads, votes, comments, bookmarks, etc.
const ACTION_PREFIXES: &[&str] = &[
    "/api/epub",
    "/api/meta", // fetch metadata = a concrete lookup action
    "/api/download",
    "/cache/",
    "/api/bookmarks",
    "/api/comments",
    "/api/votes",
    "/api/tags",
    "/api/vote",
    "/api/suggest",
    "/api/requests",
    "/api/recommendations/suggest",
    "/api/recommendations/vote",
    "/api/roadmap/vote",
    "/api/roadmap/suggest",
    "/api/auth/register",
    "/api/auth/login",
    "/api/curator",
    "/api/follow",
    "/api/rate",
    "/api/review",
    "/api/send-to-kindle",
    "/api/ask",
];

fn is_action(path: &str) -> bool {
    ACTION_PREFIXES.iter().any(|p| path.starts_with(p))
}
```

Read that list as a product manifesto. Downloads, exports, votes,
comments, bookmarks, follows, ratings, reviews, registration, login,
the Ask-the-Archive call — everything that *changes state or costs
real work* is an action. Everything else — browsing a work page,
reading search results, fetching metadata for display — is a view. The
distinction is exactly the "library vs product" question from the
opening: a platform full of views is being *read*; a platform with a
healthy action rate is being *used*.

> ⚠️ **Watch Out — a prefix list is a contract, so version it like
> one.** If someone adds a new mutating endpoint (`/api/import`) and
> forgets to add it to `ACTION_PREFIXES`, every call to it is silently
> classified as a *view* — and your active-user numbers quietly
> undercount forever. When you add an endpoint to your own analytics
> classifier, make adding it to the action list part of the definition
> of done. The test suite can help: a test asserting "every mutating
> route path starts with an action prefix" is worth its weight.

And the insert function, in `src/db/queries.rs`, has the same
best-effort religion:

```rust
/// Insert a usage event (best-effort; failures are logged, never fatal).
pub async fn insert_usage_event(
    pool: &PgPool,
    client_id: &str,
    path: &str,
    event_type: &str,
    user_agent: Option<&str>,
) {
    if let Err(e) = sqlx::query(
        "INSERT INTO usage_events (client_id, path, event_type, user_agent)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(client_id)
    .bind(path)
    .bind(event_type)
    .bind(user_agent)
    .execute(pool)
    .await
    {
        tracing::warn!("insert_usage_event failed: {e}");
    }
}
```

No `?` operator, no `Result` returned — the function swallows the error
and logs. The philosophy, stated once and applied everywhere in this
part: **the product must never depend on the analytics.** If
`usage_events` fills up, if the DB connection pool is exhausted, if a
constraint breaks — the worst case is a missing row and a log line,
never a failed request.

> 💡 **Key Concept — Instrumentation must be invisible to the
> instrumented.** Three rules make analytics safe to ship: record
> asynchronously (never block the request), record best-effort (never
> fail the request), and record minimally (never store what you don't
> aggregate). Break any one of them and your metrics will start lying —
> or worse, your product will start failing — and the first thing you'll
> delete is the analytics.

## The aggregations: from rows to insight

The admin dashboard handler (`/api/admin/analytics`, role ≥ 10)
assembles the picture from a set of small, named queries. First the
time-series visitors, at three granularities — day, week, month:

```rust
    let daily = queries::get_daily_unique_visitors(&state.db, 30).await?;
    let weekly = queries::get_weekly_unique_visitors(&state.db, 12).await?;
    let monthly = queries::get_monthly_unique_visitors(&state.db, 12).await?;
```

Each is the same shape of query — `COUNT(DISTINCT client_id)` bucketed
by a date truncation. Here's the daily one:

```rust
/// Unique visitors per day for the last N days (from usage_events).
pub async fn get_daily_unique_visitors(
    pool: &PgPool,
    days: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE(created_at) as day,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY DATE(created_at)
           ORDER BY day DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

And the weekly variant shows a nice detail — the week starts on Monday,
which is the ISO convention `DATE_TRUNC('week', ...)` follows in
PostgreSQL:

```rust
/// Unique visitors per week for the last N weeks (ISO week start Monday).
pub async fn get_weekly_unique_visitors(
    pool: &PgPool,
    weeks: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE_TRUNC('week', created_at)::date as week_start,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' weeks')::INTERVAL
           GROUP BY DATE_TRUNC('week', created_at)
           ORDER BY week_start DESC"#,
    )
    .bind(weeks)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

Three granularities, one question answered three ways: are we growing
day over day, week over week, month over month? The `COUNT(DISTINCT
client_id)` is the key piece — it's *unique* visitors, not requests.
Fifty requests from one reader is one visitor; a metric that counted
requests would be lying about growth.

Then the part that makes this whole chapter worth it — the **views vs
actions** split. Two queries define the two populations:

```rust
/// Active users in a window: distinct client_ids that performed at least one
/// ACTION (non-view event). Returns (active_count, total_events).
pub async fn get_active_users(
    pool: &PgPool,
    days: i32,
) -> AppResult<(i64, i64)> {
    let row = sqlx::query_as::<_, (i64, i64)>(
        r#"SELECT COUNT(DISTINCT client_id) as active_users,
                  COUNT(*) as action_events
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
             AND event_type = 'action'"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}
```

Active users = distinct clients with at least one action. And its
mirror image, the view-only population:

```rust
/// Distinct client_ids that ONLY viewed (no actions) in the window.
pub async fn get_view_only_users(
    pool: &PgPool,
    days: i32,
) -> AppResult<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(DISTINCT client_id)
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
             AND client_id NOT IN (
               SELECT DISTINCT client_id FROM usage_events
               WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
                 AND event_type = 'action'
             )"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}
```

Read that subquery carefully — it's set logic in SQL: "all clients who
appeared in the window" minus "clients who ever performed an action in
the window." The `NOT IN` with a subquery is the subtraction. These two
numbers together — active and view-only — partition the visitor base,
and their ratio is the single most honest engagement metric in the
system. If active users grow while view-only stays flat, the product
is converting. If views balloon but actions don't, you're building an
audience that never engages — which is its own kind of signal.

> 🧪 **Try It Yourself — make your own engagement experiment.** With the
> server running, clear your localStorage (removing `fichub_client_id`)
> and spend five minutes *only browsing*: load the home page, open a few
> fic pages, run a couple of searches. Then check
> `GET /api/admin/analytics` and find your client_id in `recent_events`
> — every entry should be a `view`. Now download an EPUB and post a
> comment, and reload: those should appear as `action` events, and you
> should move from the view-only bucket into the active-users bucket in
> the 1-day window. You've just watched the classifier work on your own
> behavior.

The handler wraps it all up, including totals and a recent timeline:

```rust
    let total_events_7d = queries::get_total_events(&state.db, 7).await?;
    let total_events_30d = queries::get_total_events(&state.db, 30).await?;

    let recent = queries::get_recent_events(&state.db, 50).await?;

    Ok(Json(json!({
        "unique_visitors": {
            "daily": daily.iter().map(|(d, n)| json!({"date": d.format("%Y-%m-%d").to_string(), "visitors": n})).collect::<Vec<_>>(),
            "weekly": weekly.iter().map(|(d, n)| json!({"week_start": d.format("%Y-%m-%d").to_string(), "visitors": n})).collect::<Vec<_>>(),
            "monthly": monthly.iter().map(|(d, n)| json!({"month": d.format("%Y-%m").to_string(), "visitors": n})).collect::<Vec<_>>(),
        },
        "engagement": {
            "active_users": { "1d": active_1d, "7d": active_7d, "30d": active_30d },
            "view_only_users": { "1d": view_only_1d, "7d": view_only_7d, "30d": view_only_30d },
            "action_events": { "7d": actions_7d, "30d": actions_30d },
            "total_events": { "7d": total_events_7d, "30d": total_events_30d },
        },
        "recent_events": recent.iter().map(|(ts, cid, path, etype, ua)| {
            json!({
                "at": ts.to_rfc3339(),
                "client_id": cid,
                "path": path,
                "type": etype,
                "user_agent": ua,
            })
        }).collect::<Vec<_>>(),
    })))
}
```

The `recent_events` timeline is the only place raw paths and client_ids
appear — and it's behind the role-10 wall, showing exactly the
"minimum viable information" discipline from Chapter 47.

> ⚠️ **Watch Out — timelines leak by volume.** The recent-events
> timeline returns the last 50 rows with raw client_ids. That's fine
> behind admin auth, but keep two rules: bound it (50 here), and never
> let a *logged-in user's* account be joined to a client_id in any
> response. The moment you render "user X's email + client Y's browsing
> history" side by side, you've turned an anonymous ID into a
> fingerprint. FicHub never joins `usage_events` to `users` — by
> design, the join would be the privacy breach.

## The dashboard page: bars and engagement cards

The frontend, `frontend/src/routes/admin/analytics/+page.svelte`,
renders all of this with zero chart libraries — CSS bars and a
hand-rolled timeline. The page header states the privacy contract
right on the screen:

```svelte
  <header class="page-head">
    <h1>📊 Usage Analytics</h1>
    <p class="subtitle">
      Non-PII anonymous usage (X-Client-ID). Unique visitors daily/weekly/monthly, plus
      engagement: who performs actions vs who just browses.
    </p>
```

The engagement summary cards give the viewer's answer in four numbers:

```svelte
      <div class="cards">
        <div class="card">
          <h3>👀 Unique visitors (30d)</h3>
          <p class="big">{sum(daily)}</p>
          <p class="muted">distinct anonymous ids</p>
        </div>
        <div class="card">
          <h3>⚡ Active users (30d)</h3>
          <p class="big">{engagement.active_users['30d']}</p>
          <p class="muted">performed an action</p>
        </div>
        <div class="card">
          <h3>👁️ View-only (30d)</h3>
          <p class="big">{engagement.view_only_users['30d']}</p>
          <p class="muted">browsed but never acted</p>
        </div>
        <div class="card">
          <h3>🎯 Actions (30d)</h3>
          <p class="big">{engagement.action_events['30d']}</p>
          <p class="muted">downloads / votes / etc.</p>
        </div>
      </div>
```

Four cards, one sentence each, and an admin can read the health of the
platform in five seconds. The daily bars use a tiny CSS-only bar chart —
`barWidth` computes a percentage, the `.bar-fill` div gets a `width`
style:

```svelte
      {#each daily as d (d.date)}
        <div class="bar-row" title="{d.date}: {d.visitors} visitors">
          <span class="bar-label">{fmtDate(d.date!)}</span>
          <div class="bar-track">
            <div class="bar-fill" style="width: {barWidth(d.visitors, maxVisitors(daily))}"></div>
          </div>
          <span class="bar-val">{d.visitors}</span>
        </div>
      {/each}
```

No charting dependency, no SVG, no canvas — a flex column of divs,
each with a percentage width. For 30 daily points, that's not a
compromise; it's the right tool. The timeline below renders the recent
events with a colored dot per type — green for action, gray for view:

```svelte
      {#each recent as e (e.at + e.client_id)}
        <div class="tl-item">
          <span class="tl-dot {e.type === 'action' ? 'action' : 'view'}"></span>
          <span class="tl-time mono">{new Date(e.at).toLocaleString()}</span>
          <span class="tl-client mono">{shortClient(e.client_id)}</span>
          <span class="tl-path mono">{eventLabel(e)}</span>
          <span class="tl-type {e.type === 'action' ? 'badge-action' : 'badge-view'}">{e.type}</span>
        </div>
      {/each}
```

`shortClient` truncates the UUID to 8 chars for display — a small,
telling detail: even the admin screen shows only a *fingerprint* of the
identity, enough to recognize "this one client again" without ever
displaying the full ID as a first-class citizen of the UI.

> 💡 **Key Concept — Analytics UI is a story, not a dump.** The page
> doesn't show a table of raw events first; it leads with four
> interpreted numbers (visitors, actives, view-only, actions), then the
> trend bars, then the raw timeline as an appendix. When you build
> dashboards, decide the *question* the page answers ("is the platform
> growing and engaging?") and order the components as an argument:
> headline, evidence, appendix. Raw data at the top of a dashboard is
> how dashboards die.

## The honest numbers

There's one more thing worth saying about `usage_events` specifically,
because it's the design constraint that makes all of the above
possible: the numbers are *honest by construction* — not by policy.
There's no anonymization step that could be skipped, no IP field that
could be added later and retroactively joined, no user_id column that
"we're not using yet." The schema is the privacy policy. If a future
developer wants to know which accounts are most active, they have to
*add a column* — a visible, reviewable, deliberate act — rather than
quietly exploiting one that's already there.

That's the same discipline we saw in the bot leaderboard's zero-PII
design in Chapter 47, and we'll see it once more in Chapter 50, where
the transparency story reaches its peak: a modlog that *every logged-in
user* can read, recording every staff action in the open.

---

## Chapter 50 — The Modlog: Every Action, Recorded in the Open

Let's start this chapter with the question that justifies it: **why
would a platform publish its own moderation decisions?**

Not "why would it record them" — every serious platform records staff
actions internally, because you need to investigate abuse, review
decisions, and defend yourself. The unusual choice in FicHub's modlog
is in the module doc comment of `src/modlog.rs`:

```rust
//! Modlog: transparent record of moderator / curator / admin actions.
//!
//! Every mutating admin/curator action calls [`record`] (best-effort, never
//! fatal to the action itself). ANY logged-in user can read the log via
//! `GET /api/modlog` — moderation is completely transparent.
```

*Any logged-in user.* Not admins. Not curators. Any reader with an
account can open `/modlog` and see exactly who banned whom, who
approved which upload, who merged which tags, and when. That's a
product philosophy as much as a technical feature: moderation that
happens in the dark is how platforms lose the trust of the communities
they serve. FicHub's answer is radical transparency — the audit trail
is public, because the *power* it records is exercised on behalf of
the community.

This chapter builds the whole thing: the schema (migration 034), the
`record` helper that every action calls, the read endpoint that any
logged-in user can hit, the admin actions that feed it, and the
frontend page that renders it.

## Migration 034: the ledger

The schema is deliberately minimal — an audit log is a *ledger*, and
ledgers don't need foreign keys to every table in the system:

```sql
-- Moderation log (modlog): a transparent, public-by-default record of every
-- moderator / curator / admin action. ANY logged-in user can read it — the
-- point is that moderation is completely transparent.
CREATE TABLE IF NOT EXISTS modlog (
    id            BIGSERIAL PRIMARY KEY,
    actor_id      INTEGER,
    actor_username TEXT,
    action        TEXT NOT NULL,
    target_type   TEXT NOT NULL DEFAULT '',
    target_id     TEXT NOT NULL DEFAULT '',
    details       JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_modlog_created ON modlog (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_modlog_actor ON modlog (actor_id);
CREATE INDEX IF NOT EXISTS idx_modlog_action ON modlog (action);
```

Six data columns. Let's go through them with the care a ledger
deserves:

- **`actor_id` and `actor_username`** — *both*, even though the ID is
  enough to look up the name. Storing the username denormalized means
  the log stays readable even if a user is deleted or renamed later;
  the ID means you can still join if you need to. An audit log should
  survive the disappearance of the people in it.
- **`action`** — the vocabulary word, the verb we chose in Chapter 47:
  `ban_user`, `approve_upload`, `blacklist_fic`, `propose_fix`,
  `vote_fix`, `delete_body`, and so on.
- **`target_type` / `target_id`** — what the action was *about*: a
  `user` with id 42, a `work` with id 17, a `fic` with a url_id, a
  `translation` with an id. They're TEXT, deliberately — targets can be
  numeric ids, url_ids, or tag names, and TEXT accepts all of them
  without pretending the whole world is one type.
- **`details`** — a JSONB bag for everything else: the new role, the
  reason code, the vote, the proposal id. JSONB keeps the ledger
  flexible (new actions can add new detail keys without a migration)
  while still being queryable with `@>` operators if you ever need to.
- **`created_at`** — with `DEFAULT now()`, so a forgetful caller can't
  even omit the timestamp and get NULL.

The three indexes map to the three ways the log will be read: newest
first (the main view), by actor (who did what), and by action (show me
all the bans). Indexes as intent, again.

> 💡 **Key Concept — An audit log is a ledger, not an entity.** Notice
> what the schema *doesn't* have: no `ON DELETE CASCADE` to `users` (the
> log must outlive the actor), no CHECK constraint on `action` (the
> vocabulary grows organically and enforcing it in code is fine), no
> updated_at column (an audit entry is written once and never changes —
> if a decision was wrong, the *correct response is another entry*,
> e.g. `unban_user`, not an UPDATE). When you design anything
> audit-shaped, remember: append-only, denormalized, and survivable.

## The record helper: best-effort, never fatal

The heart of the module is `record` — the single function every
mutating action calls. Its signature and doc comment tell the design
story:

```rust
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
    .bind(actor_id)
    .bind(actor_username)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(details)
    .execute(db)
    .await;
    if let Err(e) = res {
        tracing::warn!("modlog record failed for action '{action}': {e}");
    }
}
```

Note the return type: `()`, not `Result`. The signature *enforces* the
philosophy — a caller cannot accidentally propagate a modlog failure,
because there is no error to propagate. The `if let Err(e) = res` logs
and moves on. We saw this pattern in the admin endpoints of Chapter 47
("act, then record, best-effort"), and here's the implementation of
that promise: the audit trail is important, but it must never hold the
primary action hostage. If the modlog table is down, a ban still
happens — with a warning in the logs that accountability for that
particular action wasn't recorded.

The module also ships a small companion, `record_json`, for the common
case of "a handful of extra keys in details":

```rust
/// Convenience: record with a JSON object of extra detail keys.
pub async fn record_json(
    db: &sqlx::PgPool,
    actor_id: Option<i32>,
    actor_username: Option<String>,
    action: &str,
    target_type: &str,
    target_id: &str,
    extra: Vec<(&str, Value)>,
) {
    let mut details = serde_json::Map::new();
    for (k, v) in extra {
        details.insert(k.to_string(), v);
    }
    record(
        db,
        actor_id,
        actor_username,
        action,
        target_type,
        target_id,
        Value::Object(details),
    )
    .await;
}
```

The `Vec<(&str, Value)>` parameter is an ergonomic win: callers write
`vec![("role", json!(new_role))]` instead of hand-building a JSON
object. Small API, used twenty times across the codebase, each call a
one-liner at the call site. That's the test of a good helper: the
*boring* way to write the call is also the *correct* way.

There's a small mapping helper too — `actor_from_auth` — which keeps
the "who is acting" question in one place:

```rust
/// Map an actor (AuthUser) to (id, username) for logging.
pub fn actor_from_auth(auth: &crate::routes::auth::AuthUser) -> (Option<i32>, Option<String>) {
    (auth.user_id, auth.username.clone())
}
```

And the guard that makes transparency real — the read side requires
merely being logged in:

```rust
/// Simple guard for "any logged-in user" (not admin/curator-only).
pub fn require_logged_in(auth: &crate::routes::auth::AuthUser) -> Result<(), crate::error::AppError> {
    if auth.user_id.is_none() {
        return Err(crate::error::AppError::BadRequest(401, "Login required".into()));
    }
    Ok(())
}
```

The module even carries unit tests for these tiny pieces — including
the guard rejecting anonymous users:

```rust
    #[test]
    fn require_logged_in_rejects_anonymous() {
        let auth = crate::routes::auth::AuthUser::default();
        assert!(require_logged_in(&auth).is_err());
    }

    #[test]
    fn require_logged_in_accepts_user() {
        let auth = crate::routes::auth::AuthUser {
            user_id: Some(7),
            username: Some("u".into()),
            role: 0,
        };
        assert!(require_logged_in(&auth).is_ok());
    }
```

Note that second test: a *role 0* user passes. The modlog read is not
an admin privilege — that's the whole point, and the test pins it.

> 🧪 **Try It Yourself — the transparency guarantee, tested.** In
> `src/modlog.rs`, the tests assert that an anonymous `AuthUser::default()`
> is rejected but any logged-in user (even role 0) passes. Write one
> more test: `record` never panics when given a closed database pool —
> `PgPool::new_lazy("postgres://invalid")` or a dropped pool, and assert
> `record(...).await` simply returns. You'll be pinning the
> "never fatal to the action" contract as a test, which is exactly how
> you protect a best-effort guarantee from future refactors.

## The read endpoint: GET /api/modlog

The route lives in `src/routes/modlog.rs` — a small file, because the
handler is a thin wrapper over the `list` function:

```rust
/// GET /api/modlog?limit=50&action=ban_user
/// Any logged-in user can read the moderation log (transparency).
pub async fn modlog_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<Value>,
) -> Result<Json<Value>, AppError> {
    crate::modlog::require_logged_in(&auth)?;

    let limit = params
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(50)
        .clamp(1, 200);
    let action_filter = params
        .get("action")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let entries = crate::modlog::list(&state.db, limit, action_filter.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("modlog query failed: {e}")))?;

    let items: Vec<Value> = entries
        .into_iter()
        .map(|e| {
            json!({
                "id": e.id,
                "actor_id": e.actor_id,
                "actor_username": e.actor_username,
                "action": e.action,
                "target_type": e.target_type,
                "target_id": e.target_id,
                "details": e.details,
                "created_at": e.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "entries": items })))
}
```

The handler parses `limit` (clamped to 1–200) and an optional `action`
filter, then delegates to `list`:

```rust
/// Recent modlog entries, newest first.
pub async fn list(
    db: &sqlx::PgPool,
    limit: i64,
    action_filter: Option<&str>,
) -> sqlx::Result<Vec<ModlogEntry>> {
    let rows = if let Some(act) = action_filter {
        sqlx::query_as::<_, ModlogEntry>(
            "SELECT id, actor_id, actor_username, action, target_type, target_id, details, created_at
             FROM modlog
             WHERE action = $2
             ORDER BY created_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .bind(act)
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as::<_, ModlogEntry>(
            "SELECT id, actor_id, actor_username, action, target_type, target_id, details, created_at
             FROM modlog
             ORDER BY created_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(db)
        .await?
    };
    Ok(rows)
}
```

Two queries — one with the filter, one without — because dynamic SQL
(conditionally appending `WHERE`) is exactly the kind of string
building that invites injection and hurts query planning. Two static
queries, both `ORDER BY created_at DESC LIMIT $1`: the log is always
newest-first, always bounded. The `ModlogEntry` struct that rows land
in is `FromRow`-derived and serializes straight into the JSON response:

```rust
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ModlogEntry {
    pub id: i64,
    pub actor_id: Option<i32>,
    pub actor_username: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub details: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

> ⚠️ **Watch Out — public audit logs still need bounds.** Just because
> the log is public doesn't mean it should be paged through
> unlimited — `LIMIT $1` with a clamp of 200 keeps the endpoint cheap
> and the response sane. A public endpoint is a *more* attractive DoS
> target, not less. Every knob a public reader can turn (limit, filter)
> needs the same clamping discipline as the admin endpoints of
> Chapter 47.

## What fills the log: every mutating action, from every subsystem

The modlog isn't one feature — it's a *cross-cutting record* that
twenty call sites feed. Let's sample the vocabulary across the
codebase, because the breadth is the point. From `src/routes/admin.rs`,
the user-management actions we met in Chapter 47:

```rust
    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "set_user_role",
        "user",
        &user_id.to_string(),
        vec![("role", json!(new_role))],
    )
    .await;
```

From the translation review workflow (Part 10's Chapter 45 — the
draft→post-edit→approve state machine):

```rust
    crate::modlog::record(&state.db, user.user_id, user.username.clone(), "approve_translation", "translation", &translation_id.to_string(), serde_json::json!({})).await;
```

From the blacklist (Chapter 47), carrying the reason code:

```rust
    crate::modlog::record_json(&state.db, user.user_id, user.username.clone(), "blacklist_fic", "fic", &body.url_id, vec![("reason", serde_json::json!(reason))]).await;
```

From the comment triage queue — hiding and deleting comments, with the
triage row cleared as a side effect:

```rust
    crate::modlog::record(&state.db, user.user_id, user.username.clone(), "hide_comment", "comment", &comment_id.to_string(), serde_json::json!({})).await;
```

From the tag curator in `src/tags/curator.rs` — alias creation, tag
merges, deletions, and flag resolutions, each with the *other* tag id
in the details so the entry tells the full story of the merge:

```rust
    crate::modlog::record_json(&state.db, user.user_id, user.username.clone(), "create_alias", "tag_alias", &body.alias_name, vec![("canonical_tag_id", serde_json::json!(body.canonical_tag_id))]).await;
    crate::modlog::record_json(&state.db, user.user_id, user.username.clone(), "merge_tags", "tag", &body.source_tag_id.to_string(), vec![("target_tag_id", serde_json::json!(body.target_tag_id))]).await;
```

And from the curator content system we'll build in Chapter 51 — the
proposal and vote actions that will fill the log with `propose_fix`
and `vote_fix` entries carrying proposal ids and verdicts.

That's the audit trail as a *system*, not a feature: user management,
translations, blacklists, comment moderation, tag curation, content
fixes — every subsystem that gives a human power over content or
accounts reports into one ledger. When a reader asks "what did the
staff do this week?", one table, one endpoint, one page answers.

> 💡 **Key Concept — Audit logging is a cross-cutting concern.** The
> modlog works because it's a *convention*, not a framework: one helper,
> one schema, and a rule ("every mutating staff action calls record")
> applied at every call site. There's no middleware magic, no decorator
> registry — just discipline plus a helper that makes the disciplined
> call the easy call. When you add audit logging to your own project,
> resist building a logging framework; build the helper, document the
> rule, and review that new mutating endpoints call it.

## The page: /modlog

The frontend page, `frontend/src/routes/modlog/+page.svelte`, is the
public face of the ledger. Its subtitle states the contract plainly:

```svelte
  <header class="page-head">
    <h1>🛡️ Moderation Log</h1>
    <p class="subtitle">
      Complete transparency: every moderator, curator and admin action, visible to any logged-in user.
    </p>
```

The interesting part of the page is the **action label map** — a
curated translation from the internal vocabulary to human-readable
sentences, plus the filter dropdown built from it:

```ts
  const ACTION_LABELS: Record<string, string> = {
    set_user_role: 'Changed user role',
    ban_user: 'Banned user',
    unban_user: 'Unbanned user',
    approve_upload: 'Approved upload',
    reject_upload: 'Rejected upload',
    approve_translation: 'Approved translation',
    reject_translation: 'Rejected translation',
    hide_comment: 'Hidden comment',
    delete_comment: 'Deleted comment',
    blacklist_fic: 'Blacklisted fic',
    blacklist_author: 'Blacklisted author',
    create_alias: 'Created tag alias',
    merge_tags: 'Merged tags',
    delete_tag: 'Deleted tag',
    resolve_flag: 'Resolved tag flag',
    propose_fix: 'Proposed body fix',
    vote_fix: 'Voted on body fix',
    delete_body: 'Deleted cached body',
  };

  function actionLabel(a: string): string {
    return ACTION_LABELS[a] ?? a;
  }
```

The `?? a` fallback is a lovely robustness touch: if the backend ever
adds a new action the frontend doesn't know about yet, the page shows
the raw action name instead of "Unknown" — the log stays readable even
mid-deploy, when the new backend and old frontend briefly coexist.

The load function is the standard adminFetch pattern we saw in
Chapter 47 (JWT from localStorage, since the backend reads the
`Authorization` header, not cookies), plus the filter query string:

```ts
  async function load() {
    loading = true;
    error = '';
    try {
      const qs = actionFilter ? `?action=${encodeURIComponent(actionFilter)}` : '';
      const res = await adminFetch(`/api/modlog${qs}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      entries = body.entries ?? [];
    } catch (e) {
      error = `Failed to load modlog: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }
```

And the table itself — When, Actor, Action, Target, Details — with the
details bag flattened into a human string:

```svelte
        <tbody>
          {#each entries as e (e.id)}
            <tr>
              <td class="mono">{fmtTime(e.created_at)}</td>
              <td>{e.actor_username ?? '—'}</td>
              <td><span class="badge">{actionLabel(e.action)}</span></td>
              <td class="mono">{e.target_type ? `${e.target_type} ${e.target_id}` : '—'}</td>
              <td class="mono detail">{detailText(e.details)}</td>
            </tr>
          {/each}
        </tbody>
```

The details flattener deserves a look, because JSONB bags are great for
storage but terrible for humans:

```ts
  function detailText(d: Record<string, unknown>): string {
    const parts = Object.entries(d ?? {})
      .filter(([, v]) => v !== null && v !== undefined && v !== '')
      .map(([k, v]) => `${k}: ${String(v)}`);
    return parts.length ? parts.join(' · ') : '';
  }
```

Nulls, undefineds, and empty strings are filtered out, then the rest is
joined with interpuncts: `role: 10 · reason: 5`. A reader scanning the
page sees "alvaro banned user 42 — role: 10 · reason: 5" as a readable
sentence, while the underlying row stays structured. This is the
translation layer every raw-detail design needs.

> 🧪 **Try It Yourself — fill the log.** With two accounts (one admin),
> perform a small tour of actions: change a role, ban and unban a user,
> blacklist a fic, approve or reject a manual upload, merge two tags.
> Then open `/modlog` as a *plain logged-in user* — not the admin — and
> scroll your own history: every action you just took is there, with
> your username, the target, the details, and the timestamp. Now open
> it in a private window without logging in: you should get a
> login-required error, proving the "any logged-in user, not anyone"
> boundary. Transparency has an edge, and the code draws it.

## Transparency as a feature

Here's what I want you to take from this chapter beyond the code: the
modlog is a *product decision* with a technical implementation, and the
technical decisions all serve the product decision. Denormalized
usernames keep the log readable after accounts vanish. Best-effort
recording keeps the log from ever breaking the actions it describes.
Public read access keeps the log honest — when the people being
moderated can read the ledger, the people moderating know their work
is visible. And the "append a new entry, never edit an old one" rule
means the log can't be rewritten to look better; it can only be added
to, which is the only kind of truth an audit trail can have.

The last stop on this part's tour is the system that fills the modlog
with its most interesting entries: the curator content-fix loop in
Chapter 51, where *peer voting* decides whether a proposed fix to a
fic's body actually gets written — power distributed, gated, and
recorded, one proposal at a time.

---

## Chapter 51 — Curator Content Fixes: Peer-Voted Body Corrections

Here's a problem you only discover after you've built a scraper-based
platform: **scrapers are wrong, sometimes.** A forum thread where the
fic is one post among fifty comments. A site redesign that breaks the
chapter extraction for a week, leaving garbage in the body cache. A
rare story format the parser never quite understood. The metadata
might be right, the URL might be right, but the *body* — the actual
story text people came to read — is wrong.

FicHub's answer is the most sophisticated system in this part, and the
most interesting, because it's a *governance* answer as much as a
technical one. A single curator cannot silently replace a fic's body.
Instead, a curator **proposes** a replacement; *other* curators **vote**
on it; and the fix is written only when a **quorum** approves. The
modlog records every step. And because the body cache is
**versioned**, every fix is reversible.

This is the human-in-the-loop pattern from Part 10 applied to content
integrity — "the model proposes, the system validates, a human
decides, and the database records" — with the *crowd of curators*
standing in for the validation layer. Let's read it from the schema
up.

## Migrations 031 and 032: overrides and proposals

Two migrations frame the system. Migration 031 is the **override** —
the emergency valve for "this fic's body is wrong, right now":

```sql
-- Curator content overrides: lets curators fix a fic whose scraped body is
-- wrong (e.g. a forum thread where the fic is one post among comments, or a
-- scraper regression). When an override exists, the export pipeline uses the
-- curator-provided HTML instead of live-scraping.
CREATE TABLE IF NOT EXISTS curator_content_overrides (
    url_id      TEXT PRIMARY KEY,
    body_html   TEXT NOT NULL,          -- full fic body (HTML), one chapter
    chapters    INTEGER NOT NULL DEFAULT 1,
    title       TEXT,
    description TEXT,
    notes       TEXT,                   -- curator note (why the override)
    created_by  INTEGER REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Keyed by `url_id` (one override per fic), storing the full body HTML,
plus a `notes` column for the human explanation. The comment names the
two real-world cases that motivated it: forum threads and scraper
regressions.

Migration 032 is the **peer-voted proposal** system — the careful
version. Instead of one curator writing an override directly, a curator
submits a proposal that *other* curators vote on:

```sql
-- Curator fix proposals: curator-submitted body fixes that other curators
-- vote on. The fix is applied (body blob written) only when approved by a
-- quorum of curators, so a single curator cannot silently replace a fic's
-- content.
CREATE TABLE IF NOT EXISTS curator_fix_proposals (
    id          BIGSERIAL PRIMARY KEY,
    url_id      TEXT NOT NULL,
    body_html   TEXT NOT NULL,          -- proposed replacement body (HTML)
    reason      TEXT NOT NULL DEFAULT '', -- why the current body is wrong
    proposed_by INTEGER NOT NULL REFERENCES users(id),
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','approved','rejected','applied')),
    upvotes     INTEGER NOT NULL DEFAULT 0,
    downvotes   INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at  TIMESTAMPTZ,
    decided_by  INTEGER REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_curator_fix_proposals_status
    ON curator_fix_proposals (status, created_at);

-- Per-curator votes on proposals (one vote per curator per proposal).
CREATE TABLE IF NOT EXISTS curator_fix_votes (
    proposal_id BIGINT NOT NULL REFERENCES curator_fix_proposals(id) ON DELETE CASCADE,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    vote        SMALLINT NOT NULL CHECK (vote IN (1, -1)),  -- 1 up, -1 down
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (proposal_id, user_id)
);
```

The state machine is spelled out by the CHECK constraint: `pending →
approved → applied`, or `pending → rejected`. Notice there's no
`pending → rejected → approved` — once a proposal is decided, it's
decided; if new evidence appears, the correct move is a *new*
proposal. Same append-only discipline as the modlog.

The `curator_fix_votes` table is where the governance lives: the
composite primary key `(proposal_id, user_id)` means *one vote per
curator per proposal* — enforced by the database, not by application
logic — and the `vote` CHECK restricts values to 1 and -1. An
application that implements "one vote each" in code can be bypassed by
a bug; a database that *refuses* duplicate votes cannot.

> 💡 **Key Concept — Enforce governance constraints in the schema.**
> "One vote per curator per proposal" is a *data* rule, so it lives in
> the *data layer* as a composite primary key. "Votes are 1 or -1" is a
> *vocabulary* rule, so it lives in a CHECK constraint. The pattern:
> whatever rule you'd otherwise enforce with an application-level `if`,
> ask whether the database can enforce it structurally. Schema-level
> invariants are the ones that survive refactors.

## The constants and the guard

The route module, `src/routes/curator_content.rs`, opens by declaring
the governance parameters as named constants — because a magic number
in a vote quorum is a bug waiting to be tuned by accident:

```rust
/// Curators needed for a fix to be approved (besides the proposer).
const VOTE_QUORUM: i32 = 2;
/// Minimum net votes (up - down) for approval.
const VOTE_NET_MIN: i32 = 1;

fn require_curator(user: &AuthUser) -> AppResult<i32> {
    let uid = user.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;
    if user.role < 10 {
        return Err(AppError::BadRequest(403, "Curator access required".into()));
    }
    Ok(uid)
}
```

Two knobs, both named, both with doc comments explaining the *policy*
they encode. `VOTE_QUORUM = 2` means "the proposer plus two other
curators must vote before anything is decided." `VOTE_NET_MIN = 1`
means "at decision time, upvotes must exceed downvotes." Together
they encode: *one curator proposes, two more weigh in, and the fix
needs a positive net score to be applied.* No single curator — not even
the proposer — can flip a fic's body alone. The proposal system
structures that guarantee into the constants themselves.

`require_curator` deserves a close read: role ≥ 10. Curators are the
people trusted with tag merges, translation review, and now content
fixes — a role tier *below* admin, which is itself a governance
decision. Admins don't need to do this work; the people who know the
fandom do.

## Proposing a fix

The propose endpoint is the front door. A curator submits a `url_id`,
a replacement `body_html`, and a `reason`:

```rust
#[derive(Debug, Deserialize)]
pub struct ProposeBody {
    /// Full fic body as HTML (one or more chapters separated by
    /// `<hr class="chapter-break">` — each segment becomes a chapter).
    pub body_html: String,
    /// Why the current body is wrong (shown to other curators).
    pub reason: String,
}
```

Note the documented convention in the field comment: multi-chapter
bodies are separated by an `<hr class="chapter-break">` marker, which
the backend splits into chapters. The *contract* is documented at the
type level, so every future caller knows the format without reading
the docs.

```rust
/// POST /api/curator/content/{url_id}/propose — submit a fix proposal.
pub async fn propose_fix(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    Json(body): Json<ProposeBody>,
) -> AppResult<Json<Value>> {
    let uid = require_curator(&auth)?;

    if body.body_html.trim().is_empty() {
        return Err(AppError::BadRequest(-5, "body_html must not be empty".into()));
    }

    let row = sqlx::query(
        "INSERT INTO curator_fix_proposals (url_id, body_html, reason, proposed_by)
         VALUES ($1, $2, $3, $4)
         RETURNING id, status",
    )
    .bind(&url_id)
    .bind(&body.body_html)
    .bind(&body.reason)
    .bind(uid)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(-5, format!("failed to create proposal: {e}")))?;

    let id: i64 = row.get("id");
    let status: String = row.get("status");

    crate::modlog::record_json(&state.db, auth.user_id, auth.username.clone(), "propose_fix", "fic", &url_id, vec![("proposal_id", serde_json::json!(id))]).await;

    Ok(Json(json!({
        "ok": true,
        "proposal_id": id,
        "url_id": url_id,
        "status": status,
        "quorum": VOTE_QUORUM,
    })))
}
```

One validation (non-empty body), one INSERT with `RETURNING` (so the
caller gets the new id and status without a second query), one modlog
entry, one response that tells the proposer the quorum they'll need.
The `"quorum": VOTE_QUORUM` in the response is a nice touch: the
frontend can display "needs 2 more votes" without hardcoding the
policy.

## Voting: the state machine in action

Now the heart of the system — `vote_fix`. This handler is a complete
state machine in one function, and it's worth reading carefully. First,
load the proposal and run the guards:

```rust
/// POST /api/curator/content/proposals/{id}/vote — vote on a pending fix.
/// No self-vote. When quorum is reached: net >= VOTE_NET_MIN → approved +
/// applied (body blob written at next version); net < 0 → rejected.
pub async fn vote_fix(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<VoteBody>,
) -> AppResult<Json<Value>> {
    let uid = require_curator(&auth)?;
    if body.vote != 1 && body.vote != -1 {
        return Err(AppError::BadRequest(-5, "vote must be 1 or -1".into()));
    }

    let row = sqlx::query(
        "SELECT url_id, body_html, proposed_by, status FROM curator_fix_proposals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(-5, format!("failed to load proposal: {e}")))?;

    let Some(row) = row else {
        return Err(AppError::NotFound(format!("proposal {id} not found")));
    };
    let url_id: String = row.get("url_id");
    let body_html: String = row.get("body_html");
    let proposed_by: i32 = row.get("proposed_by");
    let status: String = row.get("status");

    if status != "pending" {
        return Err(AppError::BadRequest(-5, format!("proposal already {status}")));
    }
    if proposed_by == uid {
        return Err(AppError::BadRequest(-5, "cannot vote on your own proposal".into()));
    }
```

Three guards, in order: the vote value must be 1 or -1 (mirroring the
schema CHECK), the proposal must exist (NotFound), the proposal must
still be pending (no zombie voting on decided proposals), and — the
governance heart — **no self-votes**: `if proposed_by == uid`. A
curator cannot approve their own fix. The proposer's opinion is
already baked into the proposal existing at all; their vote would be
double-counted.

Then the vote itself — an upsert, so a curator who changes their mind
can flip their vote (the composite PK from the schema makes this
work):

```rust
    // Upsert the vote (one per curator).
    sqlx::query(
        "INSERT INTO curator_fix_votes (proposal_id, user_id, vote)
         VALUES ($1, $2, $3)
         ON CONFLICT (proposal_id, user_id) DO UPDATE SET vote = EXCLUDED.vote",
    )
    .bind(id)
    .bind(uid)
    .bind(body.vote)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(-5, format!("failed to record vote: {e}")))?;
```

`ON CONFLICT (proposal_id, user_id) DO UPDATE SET vote = EXCLUDED.vote`
is the "one vote per curator, latest wins" pattern: first vote inserts,
later votes update. A curator who initially votes down and then reads
the proposed body more carefully can flip to up — democracy with
reconsideration.

Then the counting — a single aggregate query returns the up/down
tally:

```rust
    let counts = sqlx::query(
        "SELECT
           COALESCE(SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END), 0)::int AS up,
           COALESCE(SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END), 0)::int AS down
         FROM curator_fix_votes WHERE proposal_id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(-5, format!("failed to count votes: {e}")))?;

    let up: i32 = counts.get("up");
    let down: i32 = counts.get("down");
    let total = up + down;
    let net = up - down;
```

The `COALESCE(SUM(...), 0)` guards the empty-votes case (SUM over zero
rows is NULL in Postgres; COALESCE makes it 0). Now the decision:

```rust
    let mut new_status = "pending".to_string();
    let mut applied = false;

    if total >= VOTE_QUORUM {
        if net >= VOTE_NET_MIN {
            new_status = "approved".to_string();
            // Apply: write the proposed body at the next version.
            let version = crate::body_cache::bump_version(&state.config, &url_id);
            let chapters = split_chapters(&body_html);
            crate::body_cache::save_body(
                &state.config,
                &url_id,
                &chapters,
                Some(format!("curator-fix-proposal-{id}")),
                version,
            )
            .map_err(|e| AppError::BadRequest(-5, format!("failed to apply body: {e}")))?;
            applied = true;
            new_status = "applied".to_string();
        } else if net < 0 {
            new_status = "rejected".to_string();
        }
    }
```

This is the whole governance model in six lines:

1. **Quorum first** — `total >= VOTE_QUORUM`. No decision until enough
   curators have weighed in.
2. **Then the net score** — `net >= VOTE_NET_MIN` approves, `net < 0`
   rejects. A 2-2 tie stays pending forever (nobody is unhappy enough
   to break it — which is itself a kind of decision: the proposal
   stalls until someone changes their mind or a new vote arrives).
3. **Approval applies immediately** — the body is written at the
   *next version*, via `bump_version` + `save_body`, with a source
   label that says exactly where it came from:
   `curator-fix-proposal-{id}`.

The versioning is the safety net, and it's worth pausing on. Remember
the body cache from Part 4 — every scraped fic's body is a versioned
JSON blob on disk:

```rust
pub fn bump_version(config: &Config, url_id: &str) -> i32 {
    let v = current_version(config, url_id) + 1;
    let dir = shard_dir(&config.body_cache_dir, url_id);
    let id = url_id.to_ascii_lowercase();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(&id) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    v
}
```

`bump_version` removes the current blob so the *next* save becomes the
new current version — and `save_body` writes atomically (temp file +
rename, remember from Part 4). The practical effect: an applied fix is
a *new* blob with a *new* source, and the old scraped body is gone but
the fix's provenance is stamped in the blob itself. The modlog entry
from `vote_fix` (which we'll see below) ties the whole story together:
who proposed, who voted, what the verdict was.

Then the proposal row is updated with the verdict and the deciding
voter, and the modlog entry records everything:

```rust
    sqlx::query(
        "UPDATE curator_fix_proposals
         SET status = $2, upvotes = $3, downvotes = $4, decided_at = CASE WHEN $2 = 'pending' THEN NULL ELSE now() END, decided_by = CASE WHEN $2 = 'pending' THEN NULL ELSE $5 END
         WHERE id = $1",
    )
    .bind(id)
    .bind(&new_status)
    .bind(up)
    .bind(down)
    .bind(uid)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(-5, format!("failed to update proposal: {e}")))?;

    crate::modlog::record_json(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "vote_fix",
        "fic",
        &url_id,
        vec![
            ("proposal_id", serde_json::json!(id)),
            ("vote", serde_json::json!(body.vote)),
            ("status", serde_json::json!(new_status)),
            ("applied", serde_json::json!(applied)),
        ],
    )
    .await;
```

The `CASE WHEN $2 = 'pending' THEN NULL ELSE now() END` trick keeps the
timestamps honest: a proposal that's still pending keeps `decided_at`
NULL; the moment it's decided, both stamps land in one UPDATE. And the
modlog entry carries the full context — the vote, the resulting
status, whether the body was actually applied. The audit trail for a
content change is complete: `propose_fix` (by the proposer), then one
`vote_fix` per curator, then the final `vote_fix` that tipped the
quorum carries `status: applied`.

The response mirrors the state so the frontend can update in place:

```rust
    Ok(Json(json!({
        "ok": true,
        "proposal_id": id,
        "url_id": url_id,
        "upvotes": up,
        "downvotes": down,
        "total": total,
        "net": net,
        "status": new_status,
        "applied": applied,
        "quorum": VOTE_QUORUM,
    })))
}
```

> ⚠️ **Watch Out — never let a single failure chain corrupt a
> multi-step decision.** In `vote_fix`, the application step (writing
> the body) happens *inside* the quorum branch, and its error is
> returned as a 400. That's a deliberate choice: if writing the body
> fails, the whole vote fails loudly, and the proposal stays pending —
> no state where "the vote said approved but the body was never
> written." When your own workflow has a decision step followed by a
> side-effect step, decide *which failure mode is recoverable* and
> structure the code so partial states can't exist.

## Listing proposals and inspecting bodies

The system also needs its review surface. `list_proposals` returns the
pending queue with the proposer's username joined in, bounded:

```rust
/// GET /api/curator/content/proposals?status=pending — list fix proposals.
pub async fn list_proposals(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;
    let status = q.status.unwrap_or_else(|| "pending".to_string());

    let rows = sqlx::query(
        "SELECT p.id, p.url_id, p.reason, p.proposed_by, p.status,
                p.upvotes, p.downvotes, p.created_at, u.username AS proposer
         FROM curator_fix_proposals p
         LEFT JOIN users u ON u.id = p.proposed_by
         WHERE p.status = $1
         ORDER BY p.created_at DESC
         LIMIT 100",
    )
    .bind(&status)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(-5, format!("failed to list proposals: {e}")))?;
```

`LEFT JOIN users` again — a proposal whose proposer was deleted still
shows up, with a null `proposer` username. The bounded `LIMIT 100` is
the discipline from Chapter 47's blacklist, applied again.

And `get_body` — the inspection endpoint that shows curators what the
*cached* body actually looks like, so they can judge whether a fix is
needed. It returns a preview, not the whole body:

```rust
/// GET /api/curator/content/{url_id} — inspect the cached body blob
/// (chapter count + preview) so curators can see what was gathered.
pub async fn get_body(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;

    let Some(chapters) = crate::body_cache::load_body(&state.config, &url_id) else {
        return Ok(Json(json!({
            "ok": true,
            "url_id": url_id,
            "cached": false,
        })));
    };

    let previews: Vec<String> = chapters
        .iter()
        .map(|c| {
            let text = c.content.chars().take(200).collect::<String>();
            format!("[{}] {}…", c.title, text)
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "url_id": url_id,
        "cached": true,
        "chapters": chapters.len(),
        "previews": previews,
    })))
}
```

Two hundred characters per chapter, prefixed with the chapter title
and an ellipsis — enough for a curator to see "this body is forum
chatter, not story" without dumping a megabyte of HTML. Preview, not
payload. Every UI decision in this chapter is the same instinct: show
enough to decide, never more than needed.

> 🧪 **Try It Yourself — run the whole loop.** This one needs two
> curator accounts (or one curator + the ability to create a second).
> With a fic whose cached body you can inspect via `GET
> /api/curator/content/{url_id}`, propose a fix with a slightly
> modified body via `POST /api/curator/content/{url_id}/propose`.
> Then, as the *other* curator, POST a vote of 1. Watch the response:
> `total: 1, net: 1, status: pending` — quorum not reached. Have a
> third curator vote up, and watch the response flip to
> `status: applied, applied: true`. Now check the modlog: `propose_fix`
> and `vote_fix` entries with the proposal id. Finally, try voting
> again as the proposer and confirm the `cannot vote on your own
> proposal` error. You've just exercised the entire governance
> state machine, end to end.

## The same pattern, one level up: metadata proposals

The body-fix system was so useful that FicHub extended the exact same
pattern to *metadata* — title, author, status, description. Migration
037 adds `curator_metadata_proposals` and `curator_metadata_votes` with
the same shape, and `src/routes/curator_content.rs` implements the
same propose/vote/list handlers with one new twist: the proposal
carries an `old_value` and a `new_value`, and the application is a
different UPDATE per field:

```rust
    if total >= VOTE_QUORUM {
        if net >= VOTE_NET_MIN {
            new_status = "approved".to_string();
            // Apply. title/author/description live on the canonical
            // works row; status lives on fic_info (the fic's own
            // per-source status), updated via the default source.
            let sql = match field.as_str() {
                "title" => "UPDATE works SET canonical_title = $2, updated_at = NOW() WHERE id = $1",
                "author" => "UPDATE works SET canonical_author = $2, updated_at = NOW() WHERE id = $1",
                "description" => "UPDATE works SET description = $2, updated_at = NOW() WHERE id = $1",
                _ => "",
            };
            if field == "status" {
                sqlx::query(
                    r#"UPDATE fic_info SET status = $2, updated = NOW()
                       WHERE id = (SELECT default_source_id FROM works WHERE id = $1)"#,
                )
                .bind(work_id)
                .bind(&new_value)
                .execute(&state.db)
                .await?;
            } else {
                sqlx::query(sql).bind(work_id).bind(&new_value).execute(&state.db).await?;
            }
            applied = true;
            new_status = "applied".to_string();
        } else if net < 0 {
            new_status = "rejected".to_string();
        }
    }
```

Same quorum, same net-score rule, same state machine — but the
application logic knows that `title` lives on `works` while `status`
lives on `fic_info` via the work's `default_source_id`. The system
grew a second application of the governance pattern with maybe thirty
new lines of distinctive code. That's the payoff of a well-shaped
pattern: the *second* instance is nearly free.

> 💡 **Key Concept — A governance pattern is a product, not a
> utility.** The curator content-fix system demonstrates something
> broader: when you build "changes require quorum" once, with clean
> schema constraints, named policy constants, and modlog integration,
> the *next* thing that needs it (metadata!) is mostly copy-adapt.
> Design your first governance workflow as if five more will follow —
> because in a healthy platform, they will.

## Content scan: the machine that finds the problems

One more piece completes the picture, because proposals need problems
to fix. The **content scan** service (`src/services/content_scan.rs`)
uses the local LLM to read every cached body and flag the ones that
aren't story prose — or that carry warnings their metadata denies:

```rust
//! Content scan: verify cached fic bodies with a local LLM.
//!
//! Two checks in one pass over BODIES_DIR/*.json:
//!   1. **Sanity** — is this body actually story prose, or forum
//!      chatter / scrape noise? Auto-flag noise so curators can fix the
//!      body before it gets exported.
//!   2. **Warnings** — does the text contain graphic violence, sexual
//!      content, or profanity? Keeps the `no_warnings` search filter
//!      honest (a work with no archive warning tags should not actually
//!      contain explicit content).
```

The model is asked for one line — `classification | warnings |
confidence | reason` — and the parser never panics on a bad reply:

```rust
/// Parse the model's single-line reply into a [`ContentScanRow`].
/// Never panics; unknown values fall back to unclear/none.
pub fn parse_scan_response(s: &str) -> ContentScanRow {
    let line = s.trim().lines().next().unwrap_or("").trim();
    let parts: Vec<&str> = line.splitn(4, '|').map(|p| p.trim()).collect();
    let classification = match parts.first().copied().unwrap_or("") {
        "story" => CLASS_STORY,
        "noise" => CLASS_NOISE,
        _ => CLASS_UNCLEAR,
    };
    ...
}
```

Results land in `content_scan` (migration 038) with a
`review_status` of pending/confirmed/dismissed — and the admin
endpoints from Chapter 47 (`/api/admin/content-scan`) let curators
review and dispose of each flag, with a modlog entry
(`content_scan_review`) recording the disposition. Scan finds
problems; the proposal system fixes them; the modlog records both.
The detection loop and the fix loop are two halves of one content
integrity system.

## The trust layer, complete

Step back and look at what this chapter — and this part — built. Every
system in Part 11 answers the same question: *how does a platform hold
power safely?* The admin endpoints narrow power into named tools and
record every use. The anti-bot stack prices abuse in friction without
taxing humans. The analytics measure engagement without measuring
people. The modlog publishes the ledger. And the curator system
distributes the most delicate power of all — rewriting what readers
see — across a quorum of peers, with every vote and every application
in the open.

That's the trust layer underneath everything FicHub does: **structure
the power, gate it, measure it, and record it.** The database is the
memory of every decision; the modlog is the public version of that
memory; and the community — readers, curators, admins — is the
audience that keeps it honest.

We've now built the entire platform's machinery: scraping, exports,
search, community, recommendations, AI features, and now the admin and
trust systems that run and protect it all. What's left is to make the
frontend that ties it together feel like a real product — the SvelteKit
deep dive in Part 12, where we dissect the stores, the i18n system,
the PWA offline shell, and the admin UI that renders every page we've
built the APIs for. See you there.
