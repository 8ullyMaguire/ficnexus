# forum_api: five failures, five different causes

Date: 2026-09-27 · Status: diagnosed, fixing
Suite: `tests/forum_api.rs` — 44 passed, 5 failed

## A harness discrepancy that had to be explained first

My single-suite helper reported `40 passed; 9 failed` where the canonical sweep
reported `44 passed; 5 failed`. Same flags, same suite. Four tests disagreed.

The cause: `scripts/run_db_suites.sh` exports `FORUM_POST_DELAY_SECS=0` and my
helper did not. With the real 10-second flood-control window, two posts inside it
trip the limiter, and the failure lands on **whichever test posted second** — not
the one that deserves it. That is the runner's own header comment, and the four
extra "failures" were collateral: a test posting a reply twice in quick
succession, tripping flood control, and blaming itself for a limiter it was never
testing.

Fixed the helper to match, and confirmed it now reproduces `44 passed; 5 failed`
exactly. **A debug harness that diverges from the canonical runner is evidence
about the harness.** This is the third time this session that a single
invocation generalised into a conclusion has been wrong.

## The five real failures

### 1. `admin_creates_category_and_list_returns_it` — product is right, test is wrong

    // Slug validation: uppercase rejected
    json!({ "slug": "BAD-SLUG", "title": "Bad" })
    assert_eq!(s, StatusCode::BAD_REQUEST, "bad slug: {b}");
    // actual: 200

`validate_slug` (`src/routes/forum.rs:244`) lowercases **before** validating:

    let s = slug.trim().to_lowercase();
    …
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(AppError::BadRequest("slug must be lowercase alphanumeric + hyphens"));
    }
    Ok(s)

`BAD-SLUG` normalises to `bad-slug` and is accepted, returning 200 with the
canonical slug. That is the better behaviour — a caller who capitalises a slug
gets a working link instead of an error, and the stored value is still
canonical. The test asserted a stricter contract than the product has, and its
comment ("uppercase rejected") describes a policy that was never implemented.

**Decision: keep the product, fix the test.** Uppercase *is* rejected once
normalised — the test just asserted the rejection happens before normalisation
rather than never. The test becomes an assertion that uppercase is *normalised*,
which is the real contract and is worth pinning.

### 2. `topic_list_returns_seeded_topic_with_counts` — product is right, test is wrong

    // A reply from u2 → reply_count 2 (OP + reply)
    assert_eq!(t["reply_count"], 2);   // actual: 1

Every topic-list query computes:

    (SELECT COUNT(*) - 1 FROM forum_posts p
      WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)

`COUNT(*) - 1` — the original post is deliberately excluded. Two posts minus the
OP is 1. The test's own comment reveals it was written against the older
convention where `reply_count` meant "post count".

Counting the OP as a reply would be wrong: a topic with no replies reads as
"1 reply" everywhere in the UI. **Keep the product, fix the test.**

### 3. `trust_metamod_vote_gone` — test router is missing a route

    POST /api/forum/metamod/grants/1/verdict  →  assert 410
    // actual: 404

`metamod_grant_verdict` is retired and returns `AppError::Gone` unconditionally
— correct. The 404 is because the **test's own router never registers that
path**. `tests/forum_api.rs` registers three metamod routes:

    /api/forum/metamod/queue
    /api/forum/metamod/{actionId}/vote
    /api/admin/forum/hide/{postId}          (not metamod)

and omits both `grants/{id}` and `grants/{id}/verdict`, which `src/server.rs`
*does* register. Each suite builds its own mini-router listing the handlers it
exercises, so a route added to the product is not automatically under test — and
here two tests were written against routes the harness could never serve.

### 4. `trust_grant_detail_stays_readable` — same missing route

    GET /api/forum/metamod/grants/{action_id}  →  assert 200
    // actual: 404

Identical cause. The handler is correct — it requires trust >= 3, then reads
`forum_mod_actions` joined to `forum_posts` and returns 400 "grant not found"
when absent, which is the audit-trail-read behaviour the test names.

So 3 and 4 are one fix: register the two missing routes in the test router.

### 5. `anonymous_gets_401_on_all_f3_routes` — status-code shape

    // No topic exists in the seeded DB, so expect 400 "topic not found" rather than 401.
    assert_eq!(b["err"], 400, "topic detail: {b}");   // actual: -1

`b["err"]` is `-1`, not 400. `AppError::BadRequest` maps to HTTP 400 with a
negative body code in this codebase's convention (`{err:-403}` for Forbidden,
`{err:401}` for Unauthorized, and the same shape here), so the HTTP status
assertion above it is right and the body assertion encodes the wrong sign. The
preceding `assert_eq!(s, StatusCode::BAD_REQUEST)` is what the test actually
means; the body check contradicts the convention used by every sibling suite.

## Summary of decisions

| # | test | product | decision |
|---|---|---|---|
| 1 | uppercase slug | normalises, 200 | **test** — assert normalisation |
| 2 | `reply_count` 2 | `COUNT(*)-1` = 1 | **test** — 1 is correct |
| 3 | metamod vote gone | 410, correct | **test router** — route missing |
| 4 | grant detail | 400/200, correct | **test router** — route missing |
| 5 | anonymous err 400 | `-1` | **test** — sign wrong |

Three tests encoded contracts the product does not have and arguably should not
have; one harness was missing routes; one assertion had the wrong sign. **In no
case is the product wrong.** That is worth stating plainly: these are stale tests
against behaviour that was deliberately changed, not defects.

## A sixth failure, hidden behind the fifth

Fixing #2 (`reply_count`) moved the failure to the **next** line, which I had
assumed was fine. It was not:

    // A vote on the reply → vote_score 1
    sqlx::query("INSERT INTO forum_post_votes (post_id, user_id, value) VALUES ($1, $2, 1)")
        …
        .execute(&db)
        .await
        .ok();          // <-- error discarded

**`forum_post_votes` is not a table in this schema.** Confirmed directly:

    SELECT table_name FROM information_schema.tables
     WHERE table_name LIKE 'forum_post%';
    forum_post_reactions
    forum_posts

The insert failed, `.ok()` threw the error away, and `vote_score` read 0 for a
reason that had nothing to do with the endpoint. The test had been asserting a
seed that silently never happened — and because `.ok()` swallows the failure, it
would keep failing with a misleading number rather than telling anyone the seed
was broken.

The list query aggregates `forum_post_reactions` (emoji reactions), so a "vote"
now has to be written as one, with `.expect` so a future schema change is loud.

I only found this by **adding a diagnostic that printed the database's own
count** rather than reasoning about the SQL:

    DIAG tid=1 visible_posts=2 reply_count=1 vote_score=0

Two posts visible, `reply_count` correct at 1 — so the query was fine and the
0 belonged to a different assertion entirely. Without that print I would have
kept "fixing" the count until something unrelated gave. A `COUNT(*)` in the
product and a hand-inserted fixture row are two different sources of truth, and
only one of them had been exercised.

**Generalised:** `.ok()` on a seed insert turns a broken fixture into a
mystery assertion failure. Worth auditing repo-wide; not done here.

## Result

    forum_api: test result: ok. 49 passed; 0 failed; 0 ignored
