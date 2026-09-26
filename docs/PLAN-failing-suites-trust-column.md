# Plan: seed the column the gate actually reads

Spec: `docs/specs/failing-suites-trust-column.md`
Baseline: `39 pass, 18 fail, 0 config-missing, 57 suites` (commit `b93cbc9`)
Target after: `45 pass, 12 fail` — the six group-A suites fixed, the other
twelve untouched.

Only files under `tests/` change. **No production code.** The gates are correct:
50 of them read `users.trust_level`, and the fixtures write `users.role`.

---

## Step 1 — confirm the baseline is still 39 before changing anything

```bash
cd ~/code-local/rust/ficnexus
grep -c FAILED /tmp/suite_results.txt   # 18
```

Do not re-run the full sweep first; it re-provisions a database per suite and
takes about six minutes. The recorded per-suite results are the baseline. Re-run
only if the working tree has changed in a way that would move it.

## Step 2 — the six failing suites: add `trust_level` to the seed

Each of the six has a `seed_user` helper shaped like this:

```rust
sqlx::query(
    "INSERT INTO users (username, password_hash, role) VALUES ($1, 'test-hash', $2)
     ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role",
)
.bind(username)
.bind(role)
```

The gate reads `trust_level` (`src/routes/work_proposals.rs:279`,
`src/routes/comments.rs:474`), which keeps its column default of `0`. So a
user seeded with `role = 1` has `trust_level = 0` and is refused by every
curator endpoint.

Change the statement to write both columns from the same value:

```rust
sqlx::query(
    "INSERT INTO users (username, password_hash, role, trust_level)
     VALUES ($1, 'test-hash', $2, $2)
     ON CONFLICT (username) DO UPDATE SET
       role = EXCLUDED.role, trust_level = EXCLUDED.trust_level",
)
.bind(username)
.bind(role)
```

One `$2` for both, deliberately: the fixture's `role` parameter already means
"privilege level for this test", and the two columns are both live in
production, so both get the value the test intends. Rename the parameter
`role` → `trust_level` at the same time so the call sites read honestly, and
keep writing `role` as well because `src/db/queries/proposals.rs:93` and
`src/routes/subsystems.rs:475` still read it.

**Do not** seed all four trust columns. A fixture that satisfies
`role`, `trust_level`, `trust` and `is_admin` at once passes regardless of which
gate is live, which is how the current fixture hides the problem. Set
`trust_level`, plus `role` only because production still reads it.

Add a doc comment to each `seed_user` naming the gate it satisfies, so the next
reader does not re-derive this:

```rust
/// Seed a user at the given trust level.
///
/// `role` and `trust_level` are separate columns and both are live: the trust
/// gates read `trust_level` (50 of them, e.g. work_proposals.rs:279,
/// comments.rs:474), while proposals.rs:93 and subsystems.rs:475 read `role`.
/// Seeding only `role` left `trust_level` at its default of 0, so every
/// curator-gated endpoint returned 403 and the suite tested a column no gate
/// reads. Both are written here so the fixture cannot drift again.
```

The six files: `tests/work_proposals_api.rs`, `tests/social_api.rs`,
`tests/tags_api.rs`, `tests/search_analytics_api.rs`,
`tests/translation_review_api.rs`, `tests/curator_fix_api.rs`.

**Verify — one suite, the one already proven:**

```bash
cd ~/code-local/rust/ficnexus
db=$(bash scripts/provision_test_db.sh 2>/dev/null | grep -Eo 'postgres(ql)?://[^[:space:]]+' | tail -1)
DATABASE_URL=$db REDIS_URL=redis://127.0.0.1:6379 FORUM_POST_DELAY_SECS=0 \
  CARGO_TARGET_DIR=~/.cargo-target/ficnexus \
  cargo test --test work_proposals_api -- --include-ignored --test-threads=1 2>&1 | tail -4
```

Expect `2 passed; 0 failed`. This exact change was already proven in a
throwaway copy: `0 passed, 2 failed` → `2 passed, 0 failed`, with no production
code touched.

**Verify — the other five, all six in one loop:**

```bash
cd ~/code-local/rust/ficnexus
for n in work_proposals_api social_api tags_api search_analytics_api translation_review_api curator_fix_api; do
  db=$(bash scripts/provision_test_db.sh 2>/dev/null | grep -Eo 'postgres(ql)?://[^[:space:]]+' | tail -1)
  line=$(DATABASE_URL=$db REDIS_URL=redis://127.0.0.1:6379 FORUM_POST_DELAY_SECS=0 \
    CARGO_TARGET_DIR=~/.cargo-target/ficnexus \
    cargo test --test "$n" -- --include-ignored --test-threads=1 2>&1 | grep -E '^test result:' | tail -1)
  printf '%-26s %s\n' "$n" "$line"
done
```

Expect each to show a **higher** pass count than the baseline recorded in
`/tmp/suite_results.txt`, and **zero** new failures. Some will not reach 0
failures — `social_api::notifications_flow` and
`search_analytics_api::search_flow_logs_search_query_row` have causes outside
group A and must still fail. The assertion is that the *trust-gate* 403s are
gone:

```bash
grep -c "Curator trust level required" /home/alvaro/.hermes/profiles/coding/cache/scratch/triage/social_api.log
```

Compare against a fresh capture. `social_api::comments_post_list_hide` must no
longer report `curator hide: 403`.

## Step 3 — the ten latent suites, same fix, on the same principle

Sixteen suites seed `role` without `trust_level`. Ten pass only because no gate
reads the missing column yet, and they break the moment one is added. Fix them
in this change, by the same edit, while the reasoning is fresh:

`analytics_api`, `content_scan_api`, `curator_api`, `curator_fix_api`,
`fic_suggestions_api`, `forum_groups_api`, `heal_api`, `metadata_api`,
`modlog_api`, `roadmap_api`, `search_analytics_api`, `social_api`,
`tag_edit_api`, `tags_api`, `translation_review_api`, `work_proposals_api`.

`forum_groups_api` and `roadmap_api` seed `(username, password_hash, role,
level)` — add `trust_level` alongside, keeping `level`.

**Verify — this is the part that must not regress:**

```bash
cd ~/code-local/rust/ficnexus
cargo test --no-run 2>&1 | grep -cE '^error'
```

Expect `0`. Then confirm the ten previously-passing suites still pass, which is
the real risk of touching passing tests:

```bash
cd ~/code-local/rust/ficnexus
for n in analytics_api content_scan_api curator_api fic_suggestions_api heal_api metadata_api modlog_api forum_groups_api roadmap_api tag_edit_api; do
  db=$(bash scripts/provision_test_db.sh 2>/dev/null | grep -Eo 'postgres(ql)?://[^[:space:]]+' | tail -1)
  line=$(DATABASE_URL=$db REDIS_URL=redis://127.0.0.1:6379 FORUM_POST_DELAY_SECS=0 \
    CARGO_TARGET_DIR=~/.cargo-target/ficnexus \
    cargo test --test "$n" -- --include-ignored --test-threads=1 2>&1 | grep -E '^test result:' | tail -1)
  printf '%-26s %s\n' "$n" "$line"
done
```

Every one of these was passing in the baseline and must still be `ok`. A suite
that goes from `ok` to `FAILED` here means the fixture change altered behaviour
it was relying on, and that is a finding, not a nuisance to work around.

## Step 4 — the assertion that would catch the regression

The fixture can pass for the wrong reason again, so pin the mechanism. In
`tests/work_proposals_api.rs`, the suite already has a test for the gate; make
sure there is one that asserts the *refusal* as well as the grant. A grant test
alone passes whenever the gate is absent:

```rust
#[tokio::test]
#[flavor = "multi_thread"]
async fn split_proposal_requires_curator_trust() {
    // A user at trust_level 0 must be refused. If the gate were removed this
    // would return 201 and the suite would still have a passing "curator can
    // create" test -- which is exactly how the missing column went unnoticed.
    // ...
    assert_eq!(status, 403);
}
```

The `flavor = "multi_thread"` is required, not decorative: `app()` builds a
`RedisBucketLimiter` whose shadowban probe calls `block_in_place`, which panics
on a current-thread runtime. `admin_api` documents this and
`uploads_api` was converted for it in an earlier cycle.

**Verify the test can fail — mutation check:**

```bash
cd ~/code-local/rust/ficnexus
cp tests/work_proposals_api.rs /tmp/wp_fixed.rs
# Remove trust_level from the seed; the refusal test must still pass (the user
# is at 0 either way) but the grant tests must fail.
sed -i 's/role, trust_level) VALUES ($1, .test-hash., $2, $2)/role) VALUES ($1, '"'"'test-hash'"'"', $2)/' /tmp/wp_fixed.rs
cp /tmp/wp_fixed.rs tests/zz_mutant.rs
```

Run `zz_mutant` and confirm the grant tests fail while the refusal test passes.
A mutant where *everything* fails proves nothing; the point is that the
refusal test is insensitive to the fixture and the grant tests are sensitive.
Then delete `tests/zz_mutant.rs`.

## Step 5 — full sweep, and the unit suite

```bash
cd ~/code-local/rust/ficnexus
env -u DATABASE_URL -u REDIS_URL bash scripts/run_db_suites.sh 2>&1 | tail -3
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib 2>&1 | tail -2
```

The sweep must report **at least 45 pass** and **0 config-missing**. If it
reports 39, the fixture change did not reach the suites and something is being
compiled from a stale target dir — that is the first thing to check, not a
reason to adjust expectations.

`cargo test --lib` must stay at **935 passed, 0 failed**, and `cargo build` must
succeed unchanged. No `src/` file is touched in this change, so any difference
here means the diff leaked.

## Step 6 — document

- `docs/sessions/2026-09-26-failing-suites-trust-column.md` — the measurement,
  the proof, and the four-column finding.
- Correct `docs/sessions/2026-09-26-failing-suite-classification.md`, which
  states "There is exactly one genuine trust-gate failure in the whole set:
  `work_proposals_api`, 2 tests." That is wrong on both counts — six suites,
  and the cause is a shared fixture column rather than a per-suite assertion.
  Leaving a wrong diagnosis in place is how the next cycle inherits it.
- Note in the spec's group D that `comment_triage_api`'s 403 is an `is_admin`
  gate, a different mechanism from the `trust_level` ones, and is not fixed here.

Definition of done: Steps 2–5 verified as written, sweep >= 45 with 0
config-missing, 935 unit tests unchanged, `git diff --stat` showing no `src/`
file, spec and the two session notes updated, committed and tagged.

---

## Addendum: the plan's own "one value for both columns" instruction was wrong

The plan said to write `trust_level` from the same `$2` as `role`. Applied
literally that made **four suites worse than the baseline** — the mirror-image
of the bug being fixed:

    work_proposals_api    0 passed, 2 failed  (unchanged)
    social_api            6 passed, 2 failed  ->  1 passed, 7 failed
    tags_api              2 passed, 1 failed  ->  1 passed, 2 failed
    translation_review_api 5 passed, 2 failed -> 0 passed, 7 failed
    curator_fix_api       2 passed, 1 failed  ->  0 passed, 3 failed

The cause, from the panic rather than from reading the diff:

    code: 23514  new row for relation "users" violates check constraint
    "users_trust_level_check"

and the constraint is:

    CHECK (((trust_level >= 0) AND (trust_level <= 6)))

**`trust_level` is bounded 0–6. `role` is an unbounded `smallint`.** They are
not the same domain, so they cannot share a value. 32 call sites across 14
suites legitimately pass `role = 10` — 10 is a valid role and an invalid trust
level. The plan's instruction was to write 10 into a column that forbids 10.

The plan assumed the two columns were interchangeable because both express "a
privilege level". I had measured that they are separate columns and still
assumed a shared domain, which is the specific thing I had not checked.

Fixed by clamping in SQL — `LEAST($2, 6)` — rather than at 32 call sites,
because a fixture that can violate its own table's constraint is a fixture that
will do so again. `content_scan_api`'s literal `10` becomes `10` for `role` and
`6` for `trust_level`.

## Addendum: a parameter rename rewrote SQL column names

The plan also said to rename the helper's `role` parameter to `trust_level` so
call sites "read honestly". Done with a regex over the function body, it
rewrote the **SQL string** too, producing:

    INSERT INTO users (username, password_hash, trust_level, trust_level)
    ON CONFLICT (username) DO UPDATE SET
      trust_level = EXCLUDED.trust_level, trust_level = EXCLUDED.trust_level

A duplicate column, in 16 files — and it hit five suites I had never intended
to touch (`admin_api`, `forum_api`, `reports_api`, `uploads_api`), because the
window used to find "the function containing this INSERT" was a fixed character
count that reached into the next function.

This is the second time in this repository that a broad textual substitution over
a schema where `role` is overloaded has caused damage; the session note
`2026-09-26-failing-suite-classification.md` records the first, where a
role/level regex renamed test *locals* and orphaned six `.bind(role)` calls.

The parameter rename was dropped. It is cosmetic, it was never required by the
fix, and the SQL is the part that has to be right. A change to a test that
renames nothing and fixes something is worth more than one that also tidies.

After both corrections: **no suite is worse than baseline, and
`work_proposals_api` is fully green.** `social_api` 6→7, `tags_api` holds at 2,
`translation_review_api` holds at 5, `curator_fix_api` holds at 2. The remaining
failures in those four are the group B/C causes the spec deliberately excluded.

