# FicNexus — implementation plan: remove the author edit-window test

Companion to `docs/specs/remove-edit-window-test.md`. Owner-approved 2026-09-26.

## Step 1 — record the before state

```bash
cargo test --test forum_api -- --include-ignored --test-threads=1 2>&1 \
  | grep -E '^test result'
```

**R3.** Write the pass/fail counts down. The delta from this change must be
attributable to one deleted test and nothing else.

## Step 2 — find the tests

```bash
grep -rn 'edit_window' tests/ src/        # -> NOTHING. The string does not exist.
```

Verified: `edit_window` appears in no test and no source file. The tests assert
the behaviour in prose instead, so search for that:

```bash
grep -rn 'author can edit\|author fresh edit\|within the window' tests/
```

Three hits, in `tests/forum_api.rs`:

| line | text |
|---|---|
| 1567 | `// Fresh topic: author can edit within the window; body-only update OK.` |
| 1589 | `assert_eq!(s, StatusCode::OK, "fresh author edit: {b}");` |
| 1694 | `assert_eq!(s, StatusCode::OK, "author fresh edit: {b}");` |

They belong to exactly two whole test functions, both named for the window:

| test | line | covers |
|---|---|---|
| `f3_edit_topic_author_window_and_mod` | 1502 | aged author edit → 403; **mod edit → 200**; fresh author edit → 200 |
| `f3_edit_post_author_window_and_mod` | 1636 | same three, for posts |

### The premise is half wrong, and the tests say so

The owner was told the feature does not exist. That is **true of production** and
**false of the tests**: they are written as though a window does exist, and they
are red *because it does not*.

`f3_edit_topic_author_window_and_mod` (line 1516):

```rust
// Age the topic past the 15-minute window (the window check for topic
// edits reads the topic's created_at).
```

and line 1546:

```rust
assert_eq!(s, StatusCode::FORBIDDEN, "author expired: {b}");
```

That 403 is exactly the window assertion, and it fails because
`update_topic` (`src/routes/forum.rs:2787`) selects `created_at` into `cur.0` and
never uses it. The "15-minute window" appears in **no source file** — only in the
test's comment.

So these are not aspirational tests for a missing feature. They are **red tests
describing a feature that was never built**, and each also asserts a moderator
edit succeeds, which is real coverage worth keeping.

### What to remove, precisely

Per test, remove the **author-expiry** assertions only:

- the `UPDATE ... created_at = NOW() - INTERVAL '1 hour'` ageing
- the author PATCH and its `assert_eq!(s, FORBIDDEN, "author expired")`
- the "fresh topic: author can edit within the window" block and its assertions

**Keep**, in both tests:

- the moderator PATCH and `assert_eq!(s, OK, "mod edit")` — that is independent
  coverage of moderator edit authority
- the final round-trip SELECTs that assert the new title/body persisted, attached
  to the moderator edit

The result is `f3_edit_topic_author_window_and_mod` renamed to reflect what it
now covers — `f3_edit_topic_moderator` — rather than a name that promises a
window. **R1.**

Do not delete `created_at` from the production SELECT. It is unused today, but
removing it is unrelated cleanup and the next feature to need it would re-add it.

## Step 3 — trim, do not delete whole tests

Remove the author-expiry assertions listed in step 2 from each of the two tests.
Keep the moderator-edit half and its persistence round-trip. Rename each test to
drop `_author_window` from its name. Remove any helper that becomes unused.

Do **not** touch `update_topic` / `update_post`. They work; what does not exist
is the *time limit*. The unused `created_at` in the SELECT stays — see step 2.

**R1.** The commit names this spec and states that the feature was deliberately
not wanted, so someone who wants it later finds the decision instead of
rediscovering the absence.

## Step 4 — verify nothing else moved

```bash
cargo test --test forum_api -- --include-ignored --test-threads=1 2>&1 \
  | grep -E '^test result'
```

**R3.** The count is the before-count minus exactly the number of deleted tests.
Anything else is a mistake — check with `git diff --stat tests/`.

**R2.** `git diff tests/` shows only deletions and the helpers they orphaned.

**Also verify no production behaviour was removed:** `git diff --stat src/` must be
empty for this commit.

**R3 expected delta: −2 failing tests**, not −2 removed tests. Both tests
continue to exist and pass with a narrower scope. If a test disappears from the
count entirely, the trim went too far and took the moderator coverage with it.

**Correct the record in the commit message:** the feature never existed in
production *and* the tests were written against it and left red. That is a
different situation from "aspirational test, feature unwanted", and the next
person reading the history should be able to tell.

## Step 5 — commit

`test(forum): remove the author edit-window test - feature deliberately not wanted`

Tag `remove-edit-window-2026-09-26`. Sync, log to `~/secondbrain`.
