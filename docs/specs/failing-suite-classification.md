# FicNexus — specification: the 23 failing DB suites, classified

Status: active. Written 2026-09-26, before any code change.
Plan: `docs/PLAN-failing-suite-classification.md`.

## 0. Correction to an earlier hypothesis

The previous cycle's closing note said the remaining failures were "dominated by
the role/level gate assertions left from the refactor". **That was wrong**, and
it was inferred from suite names rather than from the assertions. Classifying
the actual panics gives a different picture.

There is exactly one genuine trust-gate failure: `work_proposals_api`, 2 tests,
"Curator trust level required". Everything else is a distinct class.

## 1. Measured classification

Taken from real panic output, per suite, `--test-threads=1`:

| class | suites | root cause |
|---|---|---|
| **A. tests assume a database state they do not create** | `uploads_api` (4), `forum_api` (4 of 19) | hardcoded `user_id = 1`; flood control shared across tests |
| **B. missing seed data** | `forum_api` (2), `search_api` (4), `auto_tag` (2), `rec_curator` (1) | fixture rows absent or filtered out |
| **C. the same `fic_info` FK defect as `embedding_dedupe`** | `rec_curator` (1) | `rec_user_signals_work_id_fkey` |
| **D. migration is not idempotent** | `integration` (1 + 6 cascading) | `update_fic_tag_score` already exists |
| **E. genuine behaviour mismatch** | `curator_fix_api` (1), `search_api` (1) | assertion no longer matches behaviour |
| **F. cascade only** | `integration` (6), others | `PoisonError` from a shared `Mutex` |

Counts overlap: `forum_api`'s 19 failures are 4 real assertions plus 15
`PoisonError` cascades. The 23 failing suites contain roughly **20 independent
defects**, not 76.

### A. `uploads_api` — cannot run anywhere but one database state

Four of four failures are `403 Forbidden` where `200` or `400` was expected,
including the happy-path PNG upload.

`upload_image` (`src/routes/uploads.rs:194`) calls:

```rust
assert_min_trust(&state.db, Some(user_id), PUBLISH_MIN_TRUST, "uploading images")
```

`PUBLISH_MIN_TRUST` is `2` (`src/services/trust.rs:44`) and `assert_min_trust`
reads **`users.trust_level` from the database**, not from the JWT
(`src/services/trust.rs:87-98`).

The test does this:

```rust
let user_id: i32 = 1;
let token = auth_header(user_id, "lowtrust");
```

and its own comments show the author reasoning in circles about it:

```
// level=10 is below PUBLISH_MIN_TRUST (2) but wait — level=10 IS TL2+.
// Actually assert_min_trust checks trust_level, not level. Seed a TL0 user.
// Use a user with trust_level=0 if possible; otherwise skip.
// User 1 is likely admin (trust_level >= 2), so this should succeed.
// If it fails with 403, that confirms the trust gate works.
```

The `trust_level: 1` in the test's `auth_header` is irrelevant — the gate reads
the database. On a freshly provisioned database user 1 is
`adminit_autotag_admin` with `trust_level = 0`:

```
 id |       username        | trust_level | level
----+-----------------------+-------------+------
  1 | adminit_autotag_admin |           0 |     0
```

**So the suite depends on a pre-existing user id 1 who happens to be trusted.**
That is not a stale constant; it is a test that cannot pass on a clean database,
which is the state `provision_test_db.sh` creates. Contained to this one file —
`let user_id: i32 = 1;` appears in no other suite.

This one is worth judging on intent rather than mechanically. The test's name
is `upload_size_limit_rejects_over_max`; it wants a 413. It gets 403, because
the user is untrusted. **The gate is working correctly and the test never
reached the thing it was written to test.** The fix is to seed a user at
`trust_level >= 2` explicitly and stop depending on id 1.

### B. `forum_api` — flood control shared across tests

Real assertion failures include:

- `f3_reply_creates_post_and_notifies_follower`: `"flood control: wait 10s
  before posting again"`, expected 200
- `f4_search_finds_topic_and_post`: same
- `f3_edit_post_author_window_and_mod`: `"body too short (min 8 characters)"`
- `f5_ban_enforcement` / `f5_ban_lift`: expected 403, got `"body too short"`

The flood-control ones are a rate limiter keyed on user or client, shared by
tests in the same suite. They are order-dependent, not gate-related. The
`body too short` ones are fixtures with text under the current 8-character
minimum — a validation rule that exists and works; the fixture predates it.

### C. `rec_curator` — the same abandoned `works`-centric model

```
insert or update on table "rec_user_signals" violates foreign key constraint
"rec_user_signals_work_id_fkey"
```

Identical shape to the `embedding_dedupe` finding: the recommender subsystem
keys off `fic_info`, and this test seeds `works`. Already understood and
scoped; it needs the same `fic_info` treatment, not a new investigation.

### D. `integration` — migrations are not idempotent

```
failed to run migrations: ExecuteMigration(Database(PgDatabaseError {
  code: "42723", message: "function \"update_fic_tag_score\" already exists
  with same argument types" ... }))
```

The suite runs the migration set against a database that already has it
applied, and a migration uses `CREATE OR REPLACE FUNCTION` incorrectly or
`CREATE FUNCTION` without `IF NOT EXISTS`. The other 6 failures in the suite are
`PoisonError` from a shared `Mutex` poisoned by this first failure.

**Worth judging carefully:** is a migration supposed to be re-runnable? The
provisioner applies them once. If the test is expected to run against an already
migrated database, idempotency is a real requirement and a migration that is not
satisfying it is a genuine defect — not a test problem. That is a judgement call,
and it is recorded here rather than decided unilaterally.

## 2. Requirements

**R1.** No test may depend on a pre-existing row it did not create. `uploads_api`
must seed its own users at the trust level each test needs, because the gate
reads the database.

**R2.** No test may depend on execution order within its suite. Flood control
means either a distinct identity per test, an explicit limiter reset, or a
configurable limit for the test environment.

**R3.** `rec_curator` is repaired the same way `embedding_dedupe` was, from
`works` to `fic_info`, and both are then consistent.

**R4.** `integration`'s idempotency question is answered by reading what the
migrations actually do, and the answer is applied consistently. If
`CREATE OR REPLACE FUNCTION` is the established idiom in this repository, the
offending migration is defective and is fixed as a migration; if migrations are
documented as single-shot, the test is wrong and is fixed as a test.

**R5.** Assertions that reveal a real behaviour change are not edited to match
current behaviour. Where a test is asserting a capability the product still
intends (`curator_fix_api`'s quorum), the production behaviour is the thing
under examination.

**R6.** Cascading `PoisonError`s are never counted as defects. The count
reported is independent root causes, with the cascade count stated separately.

## 3. Non-goals

- Making the pass count go up. Class A–D are not all equally fixable, and a
  count that improves for the wrong reason is worse than no change.
- Re-opening migration 090-092 or any applied migration.
- The `search_api` character-matching behaviour, which is a real product
  question about whether "Character-A-as-secondary" should match a query for
  Character A. That needs a decision, not a test edit.

## 4. Acceptance

1. `uploads_api` passes on a database created by `provision_test_db.sh`, and
   each test names the trust level it requires.
2. `forum_api` passes in isolation **and** as part of a full run, in both
   orders.
3. `rec_curator` seeds `fic_info` and its FK violation is gone.
4. `integration`'s migration rerun either succeeds or the test is corrected to
   match documented behaviour — with the reasoning written down.
5. 935 unit tests still pass; all 56 suites still compile.
6. Every change is classified by class in the commit message, so the history
   records which findings were test problems and which were product problems.
