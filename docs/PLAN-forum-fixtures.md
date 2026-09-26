# FicNexus — implementation plan: `forum_api` fixtures

Companion to `docs/specs/forum-fixtures.md`. Executable from this text alone.

**One test is deliberately left failing** (§Step 5, and R5 of the spec). The
suite will not reach 57/57 and that is the intended outcome.

## Step 1 — lengthen fixture bodies (fixes 4 tests)

**File:** `tests/forum_api.rs`.

`validate_forum_body` rejects bodies under `FORUM_MIN_POST_LEN` (default 8) with
`400 "body too short (min 8 characters)"`. Replace every fixture body shorter
than 8 characters **in the tests listed below only** - not globally, because
some short bodies are deliberate, testing empty or whitespace input:

- `f3_edit_post_author_window_and_mod` - the edit body
- `f5_ban_enforcement` - `"x"` at the banned topic creation
- `f5_ban_lift` - the blocked-while-banned body

Do **not** touch lines 827, 845, 851, 928, 936, 944, 953 - those assert that
empty or whitespace bodies are rejected, which is correct behaviour worth
keeping.

**Do not** set `FORUM_MIN_POST_LEN=0` and **do not** weaken
`validate_forum_body`. The minimum is product policy.

**This is the step that matters most.** `f5_ban_enforcement` asserts
`StatusCode::FORBIDDEN` and additionally checks `b["msg"] == "Banned from the
forum"`. It currently fails with a body-length 400, so **the ban is never
evaluated**. After this step the test must fail, if it fails at all, with the
ban message - which proves it reached its subject.

## Step 2 — flood control (fixes 2 tests)

**File:** `scripts/run_db_suites.sh`, plus `tests/forum_api.rs` if needed.

`check_flood` (`src/routes/forum.rs:470-498`) has two documented escapes:

- `auth.trust_level >= 5` returns immediately
- `delay <= 0` returns immediately, where `delay` is
  `config.forum_post_delay_secs`

The window is config-driven, so set it for the test run rather than weakening
the limiter:

```bash
export FORUM_POST_DELAY_SECS=0
```

`check_flood` then returns `Ok` and the two tests stop failing on a shared
rate limit. **This disables the limiter for the test run, not in production.**
It also means `forum_api` no longer asserts that flood control works. If that
coverage matters, keep one dedicated test that sets the window to a non-zero
value and asserts the rejection, and let the rest run with it disabled. Say so
in the commit message either way - a test suite that silently stopped covering
a limiter is how a limiter regresses unnoticed.

**Verify the fix is real, not a suppression.** After the change, run the two
tests and confirm they now pass because the limiter allowed them, not because
the test stopped posting.

## Step 3 — seed the `general` category (fixes 2 tests)

**File:** `tests/forum_api.rs`.

`anonymous_gets_401_on_all_forum_routes` reads
`/api/forum/topics?category=general` and fails with `category not found`. The
test never seeds a category with slug `general`; it assumes one exists. That is
the same defect as `uploads_api`'s hardcoded user id 1 - a test that only passes
on a database someone prepared by hand.

Add `seed_category(&db, "general", "General").await` before the read. Do the
same for `anonymous_gets_401_on_all_f3_routes`, which fails with
`topic not found`: seed the topic it then reads.

## Step 4 — trust level for the report test (fixes 1 test)

**File:** `tests/forum_api.rs`, `f5_report_forum_target`.

`Filing reports requires trust level 1 (Basic) or higher; you are at 0 (New).`
The gate is `trust_level >= 1` and works. Seed that user at 1 or above.

Note the JWT's `trust_level` is not what the gate reads - `reports.rs` calls
`fetch_trust_level` from the database, the same pattern as `uploads_api`. Seed
the database row.

## Step 5 — do NOT touch the author edit window

`f3_edit_topic_author_window_and_mod` stays failing. The spec establishes there
is no edit window in the codebase: `cur.2` (`created_at`) is fetched by
`update_topic` and never read, `grep -rn 'edit_window' src/` is empty, and
`git log -S'edit_window' -- src/` is empty too.

Editing the test to accept 200 would silently delete the only record that the
capability was ever wanted. Editing the handler to add a window would be
inventing product policy. **Leave it red and say why**, in the commit and in
the session note.

## Step 6 — gate

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
cargo build                                   # clean
cargo test --lib                              # 935 passed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'   # 0
FORUM_POST_DELAY_SECS=0 cargo test --test forum_api -- --include-ignored --test-threads=1
```

Expected: 9 of the 10 real assertions fixed. `forum_api` still reports failures
only from `PoisonError` cascades plus the one deliberate red.

## Step 7 — commit

One commit, with the classification in the message per R6:

- stale fixtures: bodies, `general` category, trust level, flood-control window
- deliberately unchanged: the author edit window, and why

Tag `forum-fixtures-2026-09-26`, sync, commit the session note to `~/secondbrain`.

## Rollback

Test-file edits plus one environment default in a script. No production code,
no migration, no schema change.
