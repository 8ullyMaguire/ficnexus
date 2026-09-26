# The 18 failing suites: six seed the wrong column, and the gates are correct

Date: 2026-09-26 · Status: spec written, not yet implemented
Baseline: `39 pass, 18 fail, 0 config-missing, 57 suites` (commit `b93cbc9`)

## 1. What the triage measured

I ran each of the 18 failing suites against a freshly provisioned database and
captured the assertion, rather than inferring the cause from suite names. The
earlier cycle's note records that this specific mistake — inferring from names —
produced a wrong diagnosis, so the bar was the actual panic text.

The 18 are not 18 defects. They fall into four groups:

| group | suites | signature |
|---|---|---|
| A. seeds `role`, gated on `trust_level` | 6 | 403 "Curator trust level required" / "Admin access required" |
| B. genuine 500 from the handler | 2 | `{"err":-1,"msg":"database error"}` |
| C. assertion on a field the flow no longer returns | 3 | `left: None, right: Some(...)`, `unwrap()` on `None` |
| D. needs individual judgement | 7 | mixed 400/401/403/404, no single cause |

## 2. Group A: the fixtures write a column nothing reads

`users` has **both** `role smallint` and `trust_level smallint` (verified via
`information_schema`), plus `level smallint` and `is_admin boolean`. All four
are live.

**The trust gates read `trust_level`, not `role`:**

- `src/routes/work_proposals.rs:279` — `get_user_role` does
  `SELECT trust_level::text FROM users WHERE id = $1`, then classifies
  `"admin" => 3`, `"moderator" => 2`, `"curator" => 1`, numeric passthrough,
  `None => 0`. Gate at `:41` and `:168` requires `>= 1`.
- `src/routes/comments.rs:474` — reads `trust_level`, requires `>= 3`.

**The suites seed only `role`:**

    INSERT INTO users (username, password_hash, role) VALUES ($1,'test-hash',$2)
    ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role

So the seeded user gets `role = 1` and `trust_level = 0` (the column default),
and every curator-gated endpoint returns 403. The suite is not testing the
gate. It is testing a column the gate does not read.

**Six of the failing suites have this shape:**

    work_proposals_api   social_api        tags_api
    search_analytics_api translation_review_api  curator_fix_api

**16 suites in total seed `role` without `trust_level`**; the other 10 pass only
because they are not behind a trust gate, so the missing column never gets read.
Those 10 are latent and will break the moment their endpoint gains a gate.

### Proven, not inferred

Adding `trust_level` to the seed and changing **nothing else** — no production
code, no assertion:

    work_proposals_api:  0 passed, 2 failed  ->  2 passed, 0 failed

The gate was never wrong. The fixture was.

### Why this went unnoticed, and why it is a *test* defect not a schema one

This is the third instance of the same root cause. The session note
`2026-09-26-failing-suite-classification.md` records: "There is exactly one
genuine trust-gate failure in the whole set: `work_proposals_api`, 2 tests."
That was also wrong — it counted one suite where there are six, because the
cause looked like a per-suite assertion problem rather than a fixture-column
problem shared across them.

The tempting wrong fix is to "unify" the schema — drop `role`, or make the gate
read `role`. **Do not.** `users.role` is read by live production code:
`src/db/queries/proposals.rs:93` (`is_senior_curator`) and
`src/routes/subsystems.rs:475` documents that approving a subsystem applicant
flips `users.role` to 1. Two columns, two meanings, both live. The fixture is
the thing that is wrong.

### An incidental dead-code finding, explicitly out of scope

`src/db/queries/proposals.rs:91-100`, `is_senior_curator`, does
`SELECT role FROM users` and matches the result against the **strings**
`"admin" | "moderator"`. The column is `smallint`, so `role::text` is always
digits and this can never return true. It is re-exported at
`src/db/queries/mod.rs:120` and **never called**, so it is not a live bug.

Recorded, not fixed: it is unreachable, so fixing it is a separate decision
about whether the function should exist at all. Fixing its body would be
guesswork about intent.

## 3. Group B: two genuine 500s

- `series_api::author_detail_aggregates_works_and_stats` — author detail returns
  `{"err":-1,"msg":"database error"}` where 200 is expected.
- `social_api::notifications_flow` — the same, on notification prefs.

Both are the handler failing, not the fixture. Each needs its own root-cause
pass; grouping them with the trust work would hide two real defects behind a
bulk edit.

## 4. Group C: assertions on a field that is no longer returned

- `search_analytics_api::search_flow_logs_search_query_row` — asserts
  `main_char_attr` is `Some("Harry Potter|Dark Harry Pot…")`, gets `None`.
- `search_analytics_api::search_analytics_returns_aggregates` — `unwrap()` on
  `None` at `tests/search_analytics_api.rs:300`.
- `tags_api::search_autocomplete_resolve` — panics at
  `tests/tags_api.rs:446:43`.

`left: None, right: Some(...)` is exactly the shape of a refactor that stopped
writing a field, and it is also the shape of a genuine regression where the
write was lost. That distinction cannot be made from the assertion text; it
needs the handler read. Deliberately not decided here.

## 5. Group D: seven suites needing individual judgement

`admin_api` (400 vs 200, 401 vs 403), `rss_api` (401 vs 200, twice),
`requests_api` (2 vs 1, 1 vs 0), `forum_api` (3 assertions), `auto_tag`,
`bookmark_csv_api`, `comment_triage_api` (403 "Admin access required" — an
`is_admin` gate, a *third* mechanism, neither `role` nor `trust_level`).

`requests_api`'s `left: 2, right: 1` and `left: 1, right: 0` are counts. A
count assertion is the most likely in this set to be a stale expectation after
a fixture that no longer seeds what it used to — and equally likely to be a real
duplication bug. Not decided here.

## 6. Scope, in order

1. **Group A**, the six suites plus the ten latent ones. One defect, one
   mechanism, proven. This is the whole of the first change.
2. Group B, each root-caused separately.
3. Group C, handler read per case, then a decision per assertion.
4. Group D, one suite per pass.

Group A first, and alone. A change that fixes 6 suites and quietly nudges 12
others is not reviewable, and the last bulk pass over these suites is what
produced two wrong diagnoses.

## 7. What the first change must satisfy

- The 6 failing suites pass, with no production code touched.
- The 10 latent suites are corrected in the same way, on the same principle, so
  the next gate added does not break them.
- Each corrected fixture states which gate it is satisfying and why, so the next
  reader does not have to re-derive it.
- A suite that seeds a curator user must not pass merely because `role` is set.
  If `trust_level` is removed from the gate the suite should fail — that is what
  makes the fixture a test rather than a guess.
- `cargo test --lib` stays at 935, and `cargo build` is unchanged.
- `users.role` and `users.trust_level` are both still written where production
  reads them.

## 8. Addendum: there are four trust columns, not two

Measured on the provisioner-built schema:

    is_admin  boolean
    level     smallint
    role      smallint   (default 0)
    trust     integer
    trust_level smallint
    trust_loss_count integer

Four of them can express a privilege level, and the gates do not agree:

| mechanism | gates reading it | evidence |
|---|---|---|
| `trust_level` | **50** | `src/routes/admin.rs:48`, `comments.rs:474`, `work_proposals.rs:279` |
| `is_admin` | 2 | `src/recommender/routes.rs:541,608` |
| `trust` | 3 | `src/routes/features.rs:196,272,371` — `COALESCE(trust, 0)`, requires >= 4 |
| `role` | 2 | `src/db/queries/proposals.rs:93` (dead code), `src/routes/subsystems.rs` |

`features.rs` is the outlier: it is the only place that reads `trust` rather
than `trust_level`, and it is the only gate in the set that documents its own
threshold in the error string ("Admin access required (trust >= 4)").

I expected `trust` not to exist and went looking for a missing-column bug. It
does exist (`integer`), and the query returns no rows rather than an error, so
**there is no live bug here.** Recording it because the next person to read
`features.rs` will reasonably assume `trust` is a typo for `trust_level`, and
whether that is intended is a product question this spec cannot answer.

This is also why the fixture fix in group A must set `trust_level` and *not*
try to set every trust-ish column: a fixture that satisfies four mechanisms at
once passes today and tells you nothing about which gate it is exercising.
