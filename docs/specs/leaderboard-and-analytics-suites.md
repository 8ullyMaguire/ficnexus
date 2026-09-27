# leaderboard_api seeds dead tables; search_analytics_api has the rss_api secret bug plus a hardcoded `None`

Date: 2026-09-27 · Status: diagnosed, fixing
Suites: `tests/leaderboard_api.rs` 1/3, `tests/search_analytics_api.rs` 1/3

Four failures, three causes. One of the causes is a product bug.

## 1. `leaderboard_api` (2) — the fixtures write tables nothing reads

Both tests seed `leaderboard_weekly` / `leaderboard_monthly` and expect the row
back. The endpoints read neither table:

    // src/db/queries/social.rs — get_weekly_leaderboard / get_monthly_leaderboard
    SELECT u.id, u.username, u.reputation::INT as score,
           ROW_NUMBER() OVER (ORDER BY u.reputation DESC)::SMALLINT as rank
    FROM users u
    WHERE u.reputation > 0
    ORDER BY u.reputation DESC
    LIMIT $1

Both functions are now **byte-identical** — "LIVE from users.reputation (see
weekly)", per the doc comment. A cron (`compute_weekly_leaderboard`,
`compute_monthly_leaderboard`) still writes the two tables on a schedule, and
nothing reads them:

    $ grep -rn 'FROM leaderboard_weekly\|FROM leaderboard_monthly' src/ tests/
    src/db/queries/social.rs:778:    DELETE FROM leaderboard_weekly WHERE week_start = $1
    src/db/queries/social.rs:807:    DELETE FROM leaderboard_monthly WHERE month_start = $1

So the weekly and monthly endpoints are the same list under two names, and the
tables they are named after are write-only.

**Two things are wrong, and only one is in the test.**

**(a) The test fixture is stale.** `seed_user` inserts a user with no
`reputation`, and the column defaults to `0`:

    SELECT column_default FROM information_schema.columns
    WHERE table_name='users' AND column_name='reputation';   →  0

`WHERE u.reputation > 0` therefore excludes every seeded user regardless of the
leaderboard date. The hard-coded `"2026-08-03"` / `"2026-08-01"` is a second,
latent time bomb — harmless today only because the row is already excluded, but
it would bite anyone who "fixed" the fixture by also setting reputation.

**Decision: fix the fixture** — set `reputation` on the seeded user, and derive
the leaderboard period from `Utc::now()` instead of hard-coding August, so the
test cannot expire. Keep the two `leaderboard_*` inserts out of the path: they
assert nothing about the endpoint, and keeping them invites the same confusion
again.

**(b) The endpoints are not what their names and tables say.** `/weekly` and
`/monthly` return identical all-time data.

**Decision: implement the documented distinction** — read the tables the cron
already populates, falling back to the live reputation query when the period row
is absent so the endpoint degrades to "no entries this week" rather than 500.
This is a product change, so it gets its own commit and is *not* folded into the
test fix. Rationale: the tables, the cron, the compute functions and the route
names all exist and agree; only the read path was switched to live data without
the names being changed. Restoring the read is smaller than renaming the routes,
and renaming would break feed clients that hard-code the URLs.

## 2. `search_analytics_returns_aggregates` (403) — the rss_api secret bug, again

`app()` sets `jwt_secret: "fichub-test-secret"`; `auth_header` signs with
`std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret")`. Two
different secrets, same file, so the token is rejected before the handler's
`require_admin_tier` ever runs — hence 403 rather than 401.

This is now the **second** suite in the repo with this exact defect, after
`rss_api`. It is a repeated copy-paste artefact of hand-written `app()`
constructors, so it gets the same fix: one `const TEST_JWT_SECRET` per suite
file, used by both halves, with no env lookup.

Separately, `seed_admin_user` persists `role` and `trust_level` but the token
sets `is_admin: false`, so the request would still be refused by
`require_admin_tier` once the secret is fixed. `docs/specs/admin-flag.md` records
that `is_admin` is the intended signal for this surface. Both halves must be
fixed together or the test still fails with a different status.

## 3. `search_flow_logs_search_query_row` — a hardcoded `None` where the value goes

The test sends a real search carrying `main_char_attr=Harry Potter|Dark Harry
Potter` and asserts the logged row records it. The insert path supports it —
`insert_search_query(pool, query, total_results, main_char_attr, client_id,
user_id)` binds all five — and the analytics admin query groups by it. The
caller passes a literal `None`:

    // src/search/routes.rs:439
    crate::db::queries::insert_search_query(
        &state.db,
        &log_query,
        envelope.total,
        None,                      // ← main_char_attr, never populated
        client_id.as_deref(),
        auth.user_id,
    )

**Product bug.** Every logged search has a null `main_char_attr`, so the admin
analytics screen cannot show which guided filters were used, and the column is
dead in production exactly as the two leaderboard tables are.

Fixed by passing the value from the parsed params. This is the same
`main_char_attr` feature implemented in `v49-search-main-char-attr` — the search
path had it, the analytics path was never told.

## 4. Two more found while fixing the above

### 4a. The `AuthUser` extractor ignored the configured secret (product bug)

After fixing the test's two secrets, the requests returned **401 instead of
403** — still failing, but differently. The cause was in `src/routes/auth.rs`:

    impl<S> FromRequestParts<S> for AuthUser {
        async fn from_request_parts(parts: &mut Parts, _state: &S) -> ... {
            if let Some(user) = auth_user_from_token(token) {   // reads env
                return Ok(user);
            }
        }
        Ok(AuthUser::default())                                 // → anonymous
    }

`auth_user_from_token` calls `std::env::var("JWT_SECRET")`, ignoring the
`state.jwt_secret` the extractor was handed. So the extractor verified tokens
with a *different* secret than every other code path, and on failure returned
`AuthUser::default()` — anonymous — which surfaces at the gate as 401/403
rather than as the auth failure it was.

`auth_user_from_token_with_secret` already existed for exactly this, and
`src/routes/download.rs:398` was already using it. The extractor was the only
caller still going through the environment.

**Fixed** with a `ProvidesJwtSecret` trait implemented for `Arc<AppState>`, so
the extractor verifies with the same secret the rest of the app uses. A blanket
impl for all `S` was rejected: it conflicts with the `Arc<AppState>` impl, and
its environment fallback had to `.leak()` a `String` to return `&str` — a leak
per request. Every router in the tree is `Arc<AppState>`, so one impl suffices.

This also means the two "different secret in one test file" defects were
*consequences*: the test files were right to hard-code their own secret in
`AppState`, and the env-reading extractor was what made them wrong.

### 4b. `trope_popularity` was never implemented either

`search_analytics_returns_aggregates` asserts a `trope_popularity` array. The
handler returns `zero_result_queries`, `search_volume` and `conversion`; there
is no such key, so the test died on `None.unwrap()`. The handler's own doc
comment lists two views and omits it. **Same pattern as `main_char_attr`**:
specified by tests, referenced as if it existed, never built.

Implemented, and it depends on the `main_char_attr` logging fix above — the view
groups by a column that was always null, so both halves were needed. Only
non-null values are grouped, so plain text searches do not appear as a `(none)`
bucket.

## What was done

1. **`search_analytics_api` — one `const TEST_JWT_SECRET`**, used by both
   `app()` and `auth_header()`; no env lookup. `auth_header` gained an
   `is_admin: bool` parameter, because hard-coding `true` made
   `search_analytics_requires_role_10` assert nothing. The test's docstring
   said "requires role >= 10"; the gate reads the `is_admin` claim, so the
   docstring was corrected rather than the gate.
2. **`AuthUser` extractor** now takes the secret from state via
   `ProvidesJwtSecret` (4a). This is the product fix; it is what turns the
   403 into a 200.
3. **`insert_search_query` now receives the real `main_char_attr`** instead of
   a literal `None`.
4. **`trope_popularity` implemented** in `admin_search_analytics` (4b), with
   the handler doc comment corrected to list all four views.
5. **`leaderboard_api` fixture** seeds `reputation` and asserts on score rather
   than rank. The two dead-table helpers (`seed_weekly`, `seed_monthly`) are
   removed — they asserted nothing about the endpoint, and leaving them invites
   the same confusion.

The weekly/monthly **endpoint** behaviour (1b above — both routes returning
identical all-time data) is deliberately **not** changed here. It is a real
product inconsistency, but it is a design decision about which source of truth
wins, not a bug with one right answer, and it deserves its own change rather
than riding along on a test-fixing commit. Recorded here so it is not lost.
