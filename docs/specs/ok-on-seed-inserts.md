# `.ok()` on seed inserts: 233 places a broken fixture fails silently

Date: 2026-09-27 · Status: finding, not fixed
Found while: `docs/specs/forum-api-five-failures.md`, finding 6

## The pattern

    sqlx::query("INSERT INTO forum_post_votes (post_id, user_id, value) VALUES ($1, $2, 1)")
        .bind(reply_id)
        .bind(id2)
        .execute(&db)
        .await
        .ok();

`.ok()` discards the `Result`. If the table does not exist, a column is wrong,
or a constraint fires, the insert simply does not happen and the test carries on
to an assertion about data that was never written.

`forum_post_votes` **is not a table in this schema**. That test had been
asserting a seed that silently never occurred, and its `vote_score == 1`
assertion failed with a 0 that pointed nowhere near the cause.

## Why it matters more than a style complaint

A seed that fails loudly is a broken test, which is cheap. A seed that fails
quietly is a **test that measures nothing while appearing to measure something**,
and it reports a misleading number when it finally fails. Every fix I attempted
on that assertion was aimed at the wrong line until I printed the database's own
count:

    DIAG tid=1 visible_posts=2 reply_count=1 vote_score=0

Two posts, the count correct — so the 0 belonged to a different assertion
entirely. Without that print I would have kept "fixing" the count until some
unrelated thing gave.

The general hazard: **a hand-inserted fixture row and the product's own query are
two sources of truth, and only one of them is exercised.** When they disagree,
`.ok()` guarantees the disagreement is invisible.

## Scale

    forum_api          152
    requests_api        40
    reader_api          33
    body_search_api      4
    admin_api            2
    content_scan_api     2
    ───────────────────────
    TOTAL              233  across 6 suites

## Not a blanket `.ok()` → `.expect()` conversion

Some of these are **deliberate and correct**: cleanup teardown runs after a test
that may already have failed, and a `DELETE` that finds nothing is not a problem.
Converting all 233 to `.expect` would turn passing tests into panicking ones
during teardown, which is strictly worse.

The distinction is intent, and it is not visible at the call site:

- **seed / setup inserts** — must be `.expect`. A missing row makes every
  downstream assertion meaningless.
- **cleanup / teardown deletes** — `.ok()` is right.

So the audit is per-call-site, not per-pattern, and needs someone to read 233
sites with the surrounding test in view. That is a real piece of work, not a
sed.

## Cheapest durable mitigation

Not a mechanical rewrite. A test that asserts on fixture-derived data can prove
its fixture exists with one explicit check, which converts a silent zero into a
named failure at the seed. The `forum_api` vote is now `.expect`; the general
form would be a per-fixture assertion or a small helper that returns the seeded
ids so a later zero can be attributed.

Recorded rather than started, because the honest scope is "read 233 sites and
judge each", and doing that badly is worse than leaving it visible in one place.
