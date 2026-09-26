# FicNexus — specification: the F7 site-wide leveling system is unbuilt

Status: active. Written 2026-09-26, before any code change.
Found while executing `docs/PLAN-forum-fixtures.md`.
Plan: `docs/PLAN-f7-leveling.md`.

## 1. Problem

`forum_api` has 5 failing tests across the F7 group, and **none of them is a
fixture defect.** The F7 leveling system does not exist. Five of its six
assertions test behaviour that was never implemented.

## 2. Evidence

The scaffolding is complete and the implementation is absent. Every layer below
the wiring exists; the wiring does not.

### 2.1 The columns exist

```
$ psql -c "select column_name from information_schema.columns
           where table_name='users' and column_name in ('exp','level')"
exp
level
```

### 2.2 The config exists, with the exact defaults the tests expect

`src/config.rs` has 25 forum exp/level fields. The two the failing tests depend
on, at `src/config.rs:1203-1209`:

```rust
let forum_exp_topic_create = std::env::var("FORUM_EXP_TOPIC_CREATE")
    .ok().and_then(|s| s.parse().ok()).unwrap_or(2);
let forum_exp_post_create = std::env::var("FORUM_EXP_POST_CREATE")
    .ok().and_then(|s| s.parse().ok()).unwrap_or(2);
```

The tests expect 2 and 2. The config defaults to 2 and 2. **The configuration
and the tests agree; nothing consumes either.**

### 2.3 Nothing reads the config

```
$ grep -rn 'forum_exp' src/ | grep -v config.rs | wc -l
0
```

Zero. `forum_exp_topic_create` and `forum_exp_post_create` are dead fields.

### 2.4 Nothing writes the columns

```
$ grep -rn 'SET exp\|exp = exp\|exp +=' src/
(no matches)
```

The only F7 hook in the codebase is at `src/routes/forum.rs:750`:

```rust
// F7: positive mod received → the post author gains +1 exp (capped +3/day).
if delta > 0 {
    let _ = crate::db::queries::update_reputation_and_promote(
        &state.db, author_id, 1, "mod_received").await;
}
```

So **one of six F7 exp sources is wired up.** The five that are not:

| test | source | expected |
|---|---|---|
| `f7_level_exp_from_topic_and_post` | `topic_create` | +2 |
| `f7_level_exp_from_topic_and_post` | `post_create` | +2 |
| `f7_level_level_up_notification` | level-up notification | 1 sent |
| `f7_level_mod_received_exp_capped` | `mod_received` daily cap | +3/day |
| `f7_level_gate_admin` / `f7_level_gate_curator` | level gates | 403 |

### 2.5 The mechanism to build on exists

`update_reputation_and_promote(db, user_id, delta, reason)`
(`src/db/queries/reputation.rs:8`) is real and already used by
`src/routes/admin.rs` for `work_publish` and `work_complete_bonus`, and by
`src/db/queries/social.rs:329`. It takes the `exp` delta and a reason string.
**F7 needs wiring, not a new mechanism.**

## 3. The failing assertions, precisely

- `f7_level_exp_from_topic_and_post`: `exp` is 0, expected 2 after creating a
  topic. The topic was created successfully - the failure is purely that no exp
  was granted.
- `f7_level_level_up_notification`: 0 notifications, expected 1.
- `f7_level_mod_received_exp_capped`: exp 0, expected 3. The `mod_received`
  hook at forum.rs:750 *is* wired, so this one needs its own look - it may be
  the daily cap, or the test's own setup.
- `f7_level_gate_admin`: gets 200, expected 403. A level gate is not enforced.
- `f7_level_gate_curator`: gets 409, expected 403.

## 4. Requirements

**R1.** Creating a topic grants `forum_exp_topic_create` exp to its author,
through `update_reputation_and_promote` with reason `topic_create`.

**R2.** Creating a post grants `forum_exp_post_create` exp to its author,
reason `post_create`.

**R3.** Crossing a level threshold emits exactly one level-up notification, and
does not re-emit for a level already reached.

**R4.** `mod_received` exp is capped at +3 per day per author, as the comment
at forum.rs:750 already claims.

**R5.** Level gates are enforced where the tests expect 403.

**R6.** The values come from `Config`, not from literals. A test must be able to
change `FORUM_EXP_TOPIC_CREATE` and see the behaviour change.

**R7.** A user at level 0 stays at level 0 after 2 exp, as
`f7_level_exp_from_topic_and_post` asserts. The level curve must not promote at
2 exp.

## 5. Why this is not being fixed in the same pass as the fixtures

R5 is the problem. `f7_level_gate_admin` expects 403 and gets 200; that is a
missing permission check, and **who is gated at which level is a product
decision with real consequences** - get it wrong and either legitimate users
are locked out of admin tools, or the gates are cosmetic. I cannot infer the
intended curve from config defaults alone.

R3 is similar but smaller: the notification's exact shape and the level curve
are not derivable from the code.

The exp grants in R1, R2 and R4 are unambiguous - the config names them, the
tests state the expected values, and the mechanism exists. Those can be built
from this spec. **The gates and the level curve should be confirmed first.**

Per the standing rule that either side of a test may be wrong, this spec does
not assume the tests are correct about the gate thresholds. It records what they
expect and flags the decision.

## 6. Non-goals

- Deciding the level curve. Needs an owner.
- `f7_level_gate_admin` and `f7_level_gate_curator` until the curve is settled.
- The `admin_api` suite, which fails separately.

## 7. Acceptance

1. `f7_level_exp_from_topic_and_post` passes, asserting the last item's landing
   value rather than a count.
2. Setting `FORUM_EXP_TOPIC_CREATE=0` in the test environment makes the same
   test observe 0 exp - proving the value is read from config, not hardcoded.
3. `f7_level_level_up_notification` passes.
4. `f7_level_mod_received_exp_capped` passes or is reclassified with evidence.
5. 935 unit tests pass; all suites compile.
