# FicNexus — specification: `forum_api` fixtures that predate real validation

Status: active. Written 2026-09-26, before any code change.
Plan: `docs/PLAN-forum-fixtures.md`.

## 1. Problem

`tests/forum_api.rs` reports 19 failures. Most are `PoisonError` cascades; the
run shows **10 real assertions**, in five distinct groups. None of the five is
an authorization defect, and none is a production bug. Each is a fixture that
no longer satisfies a validation rule the product enforces, so the test never
reaches the thing it was written to test.

The two rules involved are real, current, and working:

| rule | where | value |
|---|---|---|
| minimum post body length | `Config::from_env`, `FORUM_MIN_POST_LEN` | 8 chars |
| post rate limit | `Config::from_env`, `FORUM_POST_DELAY_SECS` | 10 seconds |

Confirmed from `src/config.rs:1189-1195`:

```rust
let forum_min_post_len = std::env::var("FORUM_MIN_POST_LEN")
    .ok().and_then(|s| s.parse().ok()).unwrap_or(8);
let forum_post_delay_secs = std::env::var("FORUM_POST_DELAY_SECS")
    .ok().and_then(|s| s.parse().ok()).unwrap_or(10);
```

The `forum_min_post_len: 0` at `src/config.rs:554` is the struct-literal
default in a different constructor, not the value `from_env` produces. Reading
the wrong one of those two is how this looks like a missing rule.

## 2. The five groups

### 2.1 Bodies shorter than the minimum (4 tests)

`validate_forum_body` (`src/routes/forum.rs:340-344`) rejects a body under the
minimum with `400 "body too short (min 8 characters)"`. Fixtures in the test
use one-to-four character bodies: `"x"`, `"b"`, `"body"`.

Affected:
- `f3_edit_post_author_window_and_mod` - expects 200, gets 400
- `f5_ban_enforcement` - expects **403**, gets 400
- `f5_ban_lift` - expects **403**, gets 400

The two ban tests are the important ones. They assert that a banned user is
blocked, and they get a 400 about body length instead. **The ban is never
evaluated**, because `validate_forum_body` runs inside `update_topic` /
topic creation before or alongside the ban check. A test that asserts "banned
users are blocked" while passing a body too short to parse is not testing the
ban, and it has been silently not testing it.

This is the same shape as the `uploads_api` trust-gate class: a validation rule
fires first, the test fails on the wrong thing, and the real subject is never
reached.

**Fix:** lengthen the fixture bodies past the minimum. Do not relax
`FORUM_MIN_POST_LEN`, and do not relax `validate_forum_body`. The rule is
product policy; the fixture is wrong.

### 2.2 Flood control shared across tests (2 tests)

`f3_reply_creates_post_and_notifies_follower` and
`f4_search_finds_topic_and_post` both fail with:

```
"flood control: wait 10s before posting again"
```

The limiter is real and keyed per user. Tests in one suite share users, so a
test that posts twice within 10 seconds trips it. This is order-dependent, not
gate-related.

**Fix:** read the limiter before changing anything. If the window is
configurable by environment variable, set it low in
`scripts/run_db_suites.sh`. If it is not, give each test its own user. Do
**not** disable the limiter - it is the only assertion that the limiter works.

### 2.3 A test for a feature that was never implemented (1 test)

`f3_edit_topic_author_window_and_mod` backdates the topic one hour and asserts
the author gets 403:

```
author expired: {"err":0,"id":46,"msg":"Topic updated"}
  left: 200
 right: 403
```

**There is no author edit window in this codebase.** Verified three ways:

- `update_topic` (`src/routes/forum.rs:2725+`) selects
  `title, body, created_at::text` into `cur`, and **`cur.2` is never read** -
  `grep -c 'cur.2'` over the whole handler returns 0.
- `grep -rn 'edit_window' src/` returns nothing. No config field, no constant,
  no check.
- `git log -S'edit_window' -- src/` returns nothing, so it never existed in the
  squashed history either.

The handler's actual rule is: author or mod may edit; non-authors get an edit
proposal. There is no time limit for anyone.

**This is the one judgement call in this cycle, and it is not mine to make
alone.** Two readings:

- *The test is right and the feature is missing.* Authors can edit a topic
  forever, which lets someone rename a topic days later and quietly change what
  people clicked. Mods are unaffected. If the product wants an author window,
  this is a missing feature and the test correctly caught it.
- *The feature was dropped deliberately and the test is stale.* Then the test
  protects a capability that no longer exists, and asserting 403 forever is
  wrong.

There is no documentation either way: `grep -i 'edit window'` over
`docs/design/SPECIFICATION.md` and `README.md` returns nothing.

**Per R5 of `docs/specs/failing-suite-classification.md`, this is not to be
resolved by editing either side.** It needs an owner decision, and it is
recorded here as a finding. Everything else in this spec is mechanical and can
proceed independently of it.

### 2.4 Missing seed data (2 tests)

- `anonymous_gets_401_on_all_f3_routes` - `topic detail: topic not found`
- `anonymous_gets_401_on_all_forum_routes` - `topics: category not found`

Both expect 200/400 from routes that 404 or 400 because the category or topic
was never seeded. Distinct from 2.1: nothing is being rejected for length, the
fixture rows simply do not exist.

**Fix:** seed what the test reads. Check first whether the test is asserting
anonymous access to *seeded* content and the seed silently no-ops.

### 2.5 Trust level seeded at 0 (1 test)

`f5_report_forum_target` expects 200 and gets:

```
Filing reports requires trust level 1 (Basic) or higher; you are at 0 (New).
```

The gate is `trust_level >= 1` and works correctly. The test's user is seeded
at 0. Same class as `uploads_api`: the gate is right, the fixture is wrong.

**Fix:** seed at trust_level 1 or above.

## 3. Requirements

**R1.** No fixture body may be shorter than `FORUM_MIN_POST_LEN`. Lengthen
them; never lower the minimum.

**R2.** The ban tests must actually exercise the ban. After the fix, a banned
user's request must be rejected *as banned* - the assertion on `msg` is the one
that proves it, and it is already there.

**R3.** No test may depend on execution order within the suite.

**R4.** The trust-gate test must seed a user at the level the gate requires.

**R5.** The author edit window (2.3) is **not** resolved in this cycle. No edit
to either the handler or the test until an owner decides whether the feature
should exist. The test is left failing, and that is recorded rather than hidden.

**R6.** Every change is classified in its commit message as either a stale
fixture or a product decision.

## 4. Non-goals

- Changing `FORUM_MIN_POST_LEN`, `FORUM_POST_DELAY_SECS`, or any validation.
- Making the suite green. One test stays red on purpose (R5), so the count will
  not reach 57/57 and that is the correct outcome.
- `admin_api`, `requests_api` and the other 18 failing suites.

## 5. Acceptance

1. `f5_ban_enforcement` and `f5_ban_lift` fail with `"Banned from the forum"`,
   not with a body-length error. This is the acceptance criterion that matters
   most: it proves the test reached its subject.
2. The flood-control tests pass, twice in a row, and in isolation.
3. `forum_api` improves by at least 9 of the 10 real assertions.
4. The remaining failure is exactly `f3_edit_topic_author_window_and_mod`, and
   the plan records why.
5. 935 unit tests pass; all suites compile.
