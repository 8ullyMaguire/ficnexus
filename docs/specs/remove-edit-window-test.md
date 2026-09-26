# FicNexus — specification: remove the author edit-window test

Status: **approved by the owner 2026-09-26.**

## 1. Decision

The feature was never wanted, and never existed. The owner's premise is correct
about production and **incorrect about the tests** — measured, and recorded below.

**Trim the two tests, do not delete them.** Each covers the moderator edit path
as well, which is real coverage.

Not "leave it red": a permanently-failing test is a defect in the suite that
every future run pays for, and it trains the next person to ignore red. Not
`#[ignore]`: that hides it without recording why, and the reason here is a
product decision, not a timing dependency.

## 2. What was checked first

Confirmed the feature genuinely does not exist in production:

- `git log -S'edit_window' -- src/` — empty. The string never appeared in
  production code.
- No `EDIT_WINDOW` / `forum_edit_window` in `Config`.
- `update_topic` (`src/routes/forum.rs:2787`) selects `created_at` into `cur.0`
  and never uses it — the shape of a half-built feature with no second half.
- No `15-minute` or `INTERVAL '15...'` in any source file.

**But the tests are not merely aspirational.** They are written as though the
window exists, and they are **red because it does not**:

- `f3_edit_topic_author_window_and_mod:1516` — `// Age the topic past the
  15-minute window`
- `f3_edit_topic_author_window_and_mod:1546` — `assert_eq!(s, FORBIDDEN,
  "author expired: {b}")`

That 403 *is* the window assertion, and it fails precisely because
`update_topic` ignores `created_at`. So this is not "a test for a feature nobody
wanted" — it is **"a red test describing a feature that was never built"**,
which is a different record and worth stating as such.

## 2a. The two tests, and what survives

| test | line | covers |
|---|---|---|
| `f3_edit_topic_author_window_and_mod` | 1502 | aged author edit → 403 (**the window**); mod edit → 200; fresh author edit → 200 |
| `f3_edit_post_author_window_and_mod` | 1636 | the same three, for posts |

Remove the author-expiry half from each. **Keep** the moderator PATCH, its
`assert_eq!(s, OK, "mod edit")`, and the persistence round-trips attached to it
— that is independent coverage of moderator edit authority, and deleting whole
tests would throw it away. Rename to drop `_author_window` from the name so it no
longer promises a window.

## 3. Requirement

**R1.** The author-expiry assertions are removed from both tests, each is renamed
to match what it now covers, and the commit names this spec — so a future reader
who wants the feature can find the decision rather than rediscover the absence.

**R2.** No production code changes. `git diff --stat src/` must be empty. The
unused `created_at` in the SELECT is left alone: it is unrelated cleanup, and the
next feature needing it would otherwise re-add it.

**R3.** The delta is **−2 failing tests**, with both tests still present and
passing at a narrower scope. A test vanishing from the count means the trim took
the moderator coverage with it.

**R4.** The commit message corrects the record: the feature never existed in
production *and* the tests were written against it and left red.

## 4. If the feature is wanted later

The shape is already known: `Config::forum_edit_window_minutes`, checked in the
update path against the post's `created_at`, which the route already selects. A
test would assert an edit inside the window succeeds and one after it fails with
403. Nothing in this decision blocks that.
