# forum_api cannot be measured until it is isolated

Found 2026-09-26 while implementing the owner's three decisions. This blocks
work on `f3_edit_*` and `f7_level_gate_*`, and it invalidates any count-based
comparison involving this suite.

## 1. The measurement

Same commit, same command, database re-provisioned before each run:

    bash scripts/provision_test_db.sh
    cargo test --test forum_api -- --include-ignored --test-threads=1

| run | failures |
|---|---|
| 1 | 13 |
| 2 | **47** |
| 3 | 12 |

`--test-threads=1` is already serial, and the database is freshly provisioned
each time. Two of run 2's entries are bare `FAILED` lines with no test name,
which is the harness reporting something it could not attribute.

## 2. Why this invalidates the earlier numbers

`forum_api` has been quoted at 24/31, then 42/49, then 42 passed / 7 failed,
then 38/11, then 37/12 across this cycle. **None of those are comparable.** The
suite reads tables that the other 56 suites write, and `run_db_suites.sh` runs
them in a fixed order, so the count depends on everything that ran before it in
that particular invocation.

Concretely, the comparison that looked like a regression:

    parent (b497721~1) on a fresh db -> 13 failures
    head   (b497721)  on a fresh db -> 14 failures
    comm -13 parent head -> f3_delete_post_author_mod_and_non_author, vote_gone
    comm -23 parent head -> 11 tests, including six the is_admin change never
                              touches (topic_slug_by_slug_route_and_detail,
                              trust_metamod_queue_gone, ...)

Eleven "fixes" and two "regressions" in tests that have nothing to do with each
other is the signature of ordering noise, not of a code change. Chasing either
list would have been wasted work.

## 3. The likely mechanism

`wipe_forum_for_users` cleans up per-test, but several tests seed rows through
paths that cleanup does not cover, and the suite's own fixtures use fixed
usernames. Whether a given test sees a clean table therefore depends on which
tests ran before it in *this process*, and the process order is not stable —
`db_guard()` is a `Mutex`, and test registration order across the binary is not
guaranteed identical between builds.

The `user_reports` leak found in `reports_api` is one confirmed instance of this
class: `f5_report_forum_target` left rows behind that another suite then saw.

## 4. Requirements

**R1.** `forum_api` must produce the same failure set on three consecutive runs
from a freshly provisioned database. Until it does, no number from this suite is
reportable.

**R2.** Isolation, not ordering luck. Each test truncates or seeds the tables it
reads, the way `uploads_api` and `integration.rs` already do via `truncate_all()`.

**R3.** Tests that assert a global count (`items.len()`, `topics.len()`) are
incompatible with a shared table and must assert by identity — the same fix
applied to `reports_api`, where `assert_eq!(items.len(), 1)` was seeing 16
foreign rows.

**R4.** `run_db_suites.sh` must re-provision, or each suite must be measured
against a documented starting state. Comparing a suite's result across two
invocations of a shared database is not a measurement.

## 5. What this blocks

- The owner's decision to trim `f3_edit_topic_author_window_and_mod` and
  `f3_edit_post_author_window_and_mod`. The trim itself is clear (remove the
  author-expiry half, keep the moderator half), but its effect cannot be
  verified while the suite's baseline moves.
- `f7_level_gate_admin` / `f7_level_gate_curator`, which are the acceptance
  criteria for the F7 level gate.

## 6. Recommendation

Fix this **before** implementing the remaining two decisions. Both touch
`forum_api`, and neither can be verified against a baseline that swings between
12 and 47. Doing the isolation work first means the F7 gate work lands on a
suite that can actually report whether it worked.

This is a larger piece of work than the other two decisions combined, and it is
the highest-value thing left in this repo: it is the difference between the
suite telling you something and the suite telling you nothing.
