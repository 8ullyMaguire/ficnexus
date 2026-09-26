# Two shipped endpoints 500 on every request: columns no migration creates

Date: 2026-09-26 · Status: spec written, not yet implemented
Baseline: `40 pass, 17 fail, 0 config-missing, 57 suites` (commit `331de54`)

## 1. What the triage found

Two of the seventeen remaining failures are genuine server bugs, not stale
assertions and not fixture problems. Both surface identically to the client as

    {"err":-1,"msg":"database error"}

which is `AppError::Database` — the error message is logged nowhere in the test
output, so the failing query is invisible from the assertion alone. Both were
root-caused by running the handler's SQL against the live schema.

| test | endpoint | cause |
|---|---|---|
| `series_api::author_detail_aggregates_works_and_stats` | author bibliography | `author_profiles.badge_text` does not exist |
| `social_api::notifications_flow` | `GET /api/notifications/preferences` | `notification_preferences` is missing 6 selected columns |

## 2. Bug 1 — `author_profiles.badge_text`

`author_profiles` has exactly six columns:

    id, canonical_name, bio, avatar_url, created_at, updated_at

`src/routes/series.rs:252` selects a seventh:

    SELECT avatar_url, bio, badge_text FROM author_profiles
      WHERE canonical_name = $1 ORDER BY updated_at DESC LIMIT 1

so **every author bibliography page 500s**, for every author, including one with
no profile row at all — the column is resolved before any row is examined.

Proven by running the handler's query directly:

    ERROR:  column "badge_text" does not exist
    LINE 1: SELECT avatar_url, bio, badge_text FROM author_profiles WHER...

`grep -rln badge_text migrations/` → nothing. No migration has ever created it.

**The write path is broken too**, which is worse, because it is silent:

    src/routes/authors.rs:213
    UPDATE author_profiles SET badge_text = $1, updated_at = NOW() WHERE id = $2

`PUT /api/authors/{id}` with a `badge_text` field 500s. `update_author_profile`
is registered at `src/server.rs:683` and gated on `trust_level >= 3`, so it is
reachable in production. **No test covers it** — there is no `tests/authors_api.rs`
and no test anywhere references `badge_text`.

The other three profile fields in the same handler (`canonical_name`, `bio`,
`avatar_url`) all exist, so the endpoint works until a curator sets a badge text.

## 3. Bug 2 — `notification_preferences` missing six columns

The table has ten columns:

    user_id, comment_reply, follow_update, work_update, badge_earned,
    curator_promotion, recommendation, email_digest, updated_at

`src/db/queries/social.rs:179-183` selects sixteen, adding:

    comments_on_work, replies_to_comments, kudos_on_work,
    bookmarks_on_work, follows, mentions

Proven:

    ERROR:  column "comments_on_work" does not exist

All six are absent from `migrations/`, and all six are used in **three** places in
the same file — the SELECT at :181, the default-row INSERT at :197, and the
UPDATE at :246. So the whole preferences feature is inoperable: the read 500s,
and the write would too.

Note the shape of the intent: the six extra columns are all per-work
notifications (`comments_on_work`, `kudos_on_work`, `bookmarks_on_work`) plus
`follows` and `mentions`. The code was written for a per-work notification
feature; the migration for it never landed. This is a **feature gap wearing a
bug's clothes**, which decides the fix — see §6.

## 4. The shared root cause

Both are the same failure: SQL in `src/` references columns that no migration
creates. `cargo build` cannot see it, `cargo test --lib` cannot see it, and
`cargo test --no-run` cannot see it. Only a query executed against a migrated
database finds it.

This repository has committed migrations that are applied and immutable, so the
question is not "why is the SQL wrong" but "which side is authoritative".

I ran a static sweep for the class rather than stopping at the two
symptomatic endpoints: extract every snake_case token from SQL in `src/`, resolve
it against the 180-table live schema, and report what does not exist. The sweep
found both bugs, plus a large tail of false positives — JSON object keys,
per-row aliases inside aggregates, and SQL functions (`to_char`, `ts_rank`,
`date_trunc`) that look like columns because they are snake_case.

**The sweep is not reliable enough to act on beyond these two.** Both of these
are proven by execution, and the remaining candidates need each query run
individually — the same way these two were found. Recorded so the next person
does not mistake the sweep for a result.

## 5. What was ruled out first

`series_api`'s URL looked wrong at first: the test calls
`/api/authors/{name}` while `src/server.rs:679` registers
`/api/authors/{id}` (`Path(id): Path<i32>`) and the name-keyed route at
`src/server.rs:675` is `/api/authors/by-name/{name}`.

**The test is right.** It builds its own router at `tests/series_api.rs:124`:

    Router::new()
        .route("/api/series/{id}", get(fichub::routes::series::get_series))
        .route("/api/authors/{name}", get(fichub::routes::series::get_author))

so `/api/authors/SeriesApiTest Person` does reach `get_author`. Changing the
test's URL to `/api/authors/by-name/...` turns the 500 into a **404** — the
proof that the path is not the problem and the handler is. Reverted.

Worth recording: I was one query away from "fixing" a test that was correct and
blaming it for a server bug.

## 6. The decision this spec does not make

For bug 1 the fix is unambiguous and belongs in this change: `badge_text` is
read in one place, written in one place, and nothing else in the codebase knows
the field exists. Either add the column or drop the field. **Dropping it is
safer and smaller** — it is a curator-facing label with no UI, no test, and no
consumer, and adding a migration to store a string nobody reads is the larger
change. Recorded as the recommendation, not silently applied.

For bug 2 the fix is **not** unambiguous, and this is the part that needs an
owner decision:

- The six columns are all per-work notification toggles. Either the feature was
  cut and the code should be trimmed back to the ten columns that exist, or it
  was deferred and a migration is owed. Those are different products.
- Trimming the code is a behaviour change: `GET /preferences` would stop
  reporting six fields the API currently promises, and any client reading them
  breaks silently rather than loudly.
- Adding the columns is additive and breaks nothing, but ships schema for a
  feature nobody has built a UI for.

**This spec does not pick.** It records the finding, proves it, and leaves the
choice where it belongs. `GET /api/notifications/preferences` returning 500 for
every authenticated user is the bug either way; that much is not a decision.

## 7. Scope

**In (bug 1):** remove the `badge_text` reference from `series.rs` and
`authors.rs`, or add the column — owner call, per §6. Add a test that the author
page returns 200 and that the profile update does not 500, since neither is
covered today.

**Out:** the static sweep's other candidates (unproven individually), the
per-work notification feature, and any change to `migrations/`.

Bug 2 is **not** implemented here. It needs the decision in §6 first, and
implementing half of it — fixing one 500 while another of the same class is
documented and untouched — is how the next reader loses track of whether
notification prefs work.
