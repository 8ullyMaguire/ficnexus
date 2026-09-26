# FicNexus — the F7 level gate: spec

Status: **approved by owner 2026-09-26.** Supersedes the "needs an owner
decision" framing in `docs/specs/f7-gate-assignments.md`; that document is kept
because its analysis of the gap is still correct, but two of its conclusions
about *which* gate is broken are corrected here.

Owner decision: **level and trust are independent gates, both required; the
thresholds come from Config.**

## 1. What the previous spec got wrong

`f7-gate-assignments.md` said both red tests were caused by `require_admin`
gating on `trust_level < 5` and never reading `auth.level`. That is true of
`f7_level_gate_admin` and **false of `f7_level_gate_curator`.**

The two tests exercise different functions:

| test | route | gate it actually hits |
|---|---|---|
| `f7_level_gate_admin` | `POST /api/forum/categories` | `require_admin` (forum.rs:209) |
| `f7_level_gate_curator` | `POST /api/forum/posts/{id}/moderate` | `require_trust_queue` (forum.rs:398) |

`moderate_post` never calls `require_admin`. Its gate is
`assert_staff_or_min_trust(.., queue_min_trust(state), ..)` — a trust check with
a configurable floor. So the curator test is not testing the admin gate at all,
and editing `require_admin` cannot turn it green.

## 2. Why the curator test returns 409, not 403

The previous spec noticed the 409 and speculated it was "a different failure
entirely". Confirmed — it is a **state leak inside the test**, not the gate.

`moderate_post` (forum.rs:673) runs these checks in order:

```rust
require_trust_queue(&state, &auth).await?;      // the gate
check_daily_cap(&state, user_id).await?;
let (author_id, _score, mod_count) = load_moddable_post(...)?;
if author_id == user_id { Forbidden("Cannot moderate your own post") }
if mod_count >= 5        { BadRequest("post has reached the moderation limit") }
if already_modded(..)    { Conflict("You already moderated this post") }   // <- 409
```

Only one 409 exists in `forum.rs` and it is `already_modded`. The test moderates
`post_id` **twice** — once at level 50 (expects OK) and once at level 49 (expects
FORBIDDEN) — against the same post and the same curator. The second call is a
duplicate moderation, so it gets 409 before the gate can reject it.

**The second assertion never tested the gate, and never did.** The gate would
have to be infinitely strict for the test to pass: the request is rejected as a
duplicate regardless of level.

Note the ordering consequence: `already_modded` is checked *after* the gate, so
if the gate worked the level-49 call would be 403 and the test would be honest.
The 409 is therefore also evidence that **the gate is not rejecting level 49**,
which is the real bug.

## 3. Two defects in `f7_level_gate_curator`

Both are test defects, and both have to be fixed for the test to be able to pass:

**D1 — the forbidden case does not carry the level it claims to test.** At
level 49 the test calls `auth_header(cur_id, u_cur, 5)`. That helper maps a role
argument to a level (`src` test helper, `10→100, 5→50, 1→1, else 0`), so the
"level 49" request actually carries **level 50** in its token. The forbidden
assertion is asserting on the wrong level. It must use
`auth_header_level(cur_id, u_cur, 5, 49)`.

**D2 — the duplicate moderation (above).** The level-49 case must moderate a
*different* post, or the fixture must be re-seeded between the two cases.

`f7_level_gate_admin` has no such defects: its level-99 request uses
`auth_header_level(.., 99)` and its level-100 request uses
`auth_header(.., 10)`, which maps 10→100. Both tokens carry the levels they
claim. The `auth_header` use in that test is correct, not an oversight.

## 4. The design

### 4.1 Split the one gate into two

`require_admin` governs four call sites with two different intents:

| site | route | intent | level needed |
|---|---|---|---|
| 1297 | `DELETE /api/admin/forum/bans/{id}` | lift a ban | moderator (50) |
| 1325 | `GET /api/admin/forum/bans` | list bans | moderator (50) |
| 2078 | `POST /api/forum/categories` | create category | 100 |
| 2113 | `PATCH /api/forum/categories/{id}` | edit/reorder category | 100 |

Today one function applies one threshold to all four, which cannot express that
distinction. Replace it with a threshold-taking helper plus two named wrappers:

```rust
fn require_admin_at_level(auth: &AuthUser, min_level: i16) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.trust_level < FORUM_ADMIN_MIN_TRUST {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    if i16::from(auth.level) < min_level {
        return Err(AppError::Forbidden("Forum level too low".into()));
    }
    Ok(uid)
}

/// Moderator: trust 5+, `forum_mod_min_level` (default 0 = today's behaviour).
fn require_forum_moderator(auth: &AuthUser, cfg: &Config) -> Result<i32, AppError> {
    require_admin_at_level(auth, cfg.forum_mod_min_level)
}

/// Category management: trust 5+, `forum_category_min_level` (default 0).
fn require_forum_category_admin(auth: &AuthUser, cfg: &Config) -> Result<i32, AppError> {
    require_admin_at_level(auth, cfg.forum_category_min_level)
}
```

**Trust and level are both required** (the owner's decision). `is_admin` is
deliberately *not* an alternative path here: `require_admin` in `forum.rs` is the
pre-existing forum gate and stays trust+level. The `is_admin` flag governs
`/api/admin/*` in `admin.rs` via `require_admin_tier`, which is a separate
surface with its own 77 call sites. Mixing them would make "who can create a
category" depend on a flag that has nothing to do with forum participation.

### 4.2 The curator gate

`require_trust_queue` is used by more routes than `/moderate`, and it is the
queue-access gate. Adding a level requirement to it would lock out every
moderator whose level is below the threshold — a much wider blast radius than
the two tests imply.

So: **do not change `require_trust_queue`.** Add a level check to
`moderate_post` only, reading a new `forum_curator_min_level` (default 0), and
place it **before** `check_daily_cap` so a level rejection is not masked by a cap
exhaustion:

```rust
require_trust_queue(&state, &auth).await?;
if i16::from(auth.level) < state.config.forum_curator_min_level {
    return Err(AppError::Forbidden("Forum level too low to moderate".into()));
}
check_daily_cap(&state, user_id).await?;
```

### 4.3 Config

**One of these already existed.** `forum_mod_min_level` was declared in
`config.rs:298` with the env var `FORUM_MOD_MIN_LEVEL`, a default of **50**, and
the comment *"F7: level threshold for curators (was role 5)"*. It was plumbed
through `from_env`, through the `Default` impl, and through the struct
construction — and **read by nothing**. A grep for readers outside `config.rs`
returns zero hits.

So the plumbing for exactly this feature was already half-built and never
connected. The plan originally specified three new fields; the correct answer is
two, and the existing one is reused rather than shadowed under a new name.

| field | env var | default | governs | status |
|---|---|---:|---|---|
| `forum_mod_min_level` | `FORUM_MOD_MIN_LEVEL` | 50 | `/moderate` | **existed, unwired** — now read |
| `forum_category_min_level` | `FORUM_CATEGORY_MIN_LEVEL` | 0 | category create / edit | new |
| `forum_ban_min_level` | `FORUM_BAN_MIN_LEVEL` | 0 | ban list / ban lift | new |

**Naming note.** The two new fields are deliberately *not* called
`forum_mod_*`. The existing `forum_mod_min_level` is the curator threshold and
its `Default` impl is 0 while `from_env` is 50 — the two constructors disagree,
which is pre-existing and left alone here, but it is the kind of thing that
misleads the next reader. The ban gate is named `forum_ban_min_level` for the
action it guards rather than for the tier it sits in.

**The two new fields default to 0**, which means the level axis is inert for
them: deploying this changes nobody's ability to list bans or edit categories.
The curator gate is the exception — `FORUM_MOD_MIN_LEVEL` already defaulted to
**50**, so wiring it up *does* change behaviour, and that is the one rollout
risk in this work (see §7).

Three separate thresholds exist because one number cannot serve a ban list, a
category, and a moderation action.

## 5. Test changes

**Fix, do not delete.** `f7_level_gate_curator` is a real requirement with two
real defects (D1, D2). Delete it and the gate has no test at all.

- D1: use `auth_header_level(cur_id, u_cur, 5, 49)` for the forbidden case.
- D2: seed a second post for the level-49 case so it is not a duplicate
  moderation.
- Assert the config thresholds explicitly in the tests rather than relying on
  the default of 0 — otherwise the tests pass vacuously, exactly the defect in
  `authors.rs::test_auto_approve_only_admin` (a test comparing a literal to
  itself) which the admin-flag work removed.
- Add a case proving `is_admin` alone does **not** grant category creation, so
  the "independent gates" decision is pinned.

## 6. Acceptance

1. `f7_level_gate_admin` green: 403 at level 99, 200 at level 100.
2. `f7_level_gate_curator` green: 200 at level 50, 403 at level 49 — and the 403
   is a level rejection, not a 409 duplicate.
3. `forum_api` failure count does not increase against the recorded baseline
   (`docs/BASELINE-db-suites.md`, 8 under `run_db_suites.sh`).
4. `cargo test --lib` stays at 935 passed.
5. With all three config values at their defaults, a moderator with trust 5 and
   level 0 can still list and lift bans — i.e. the change is inert by default.

## 7. Risk

Two distinct risks, one of them real *on deploy* rather than on configuration.

**Real now:** `FORUM_MOD_MIN_LEVEL` defaults to 50, so wiring it into
`moderate_post` immediately requires level 50 to moderate. Anyone currently
moderating with trust 5 and level below 50 loses the ability at the moment this
ships. That is presumably the intent — the field was named and defaulted for
exactly this — but it is a behaviour change, not a no-op, and it needs saying
plainly rather than hiding behind "config-driven".

Mitigation before deploy:

```sql
SELECT count(*) FROM users
WHERE trust_level >= 4 AND level < 50 AND banned_until IS NULL;
```

If that returns anyone, either lower `FORUM_MOD_MIN_LEVEL`, or raise those
users' level, before shipping. The test suite cannot tell you this — it seeds
its own users.

**On configuration:** the other two default to 0, so an operator setting
`FORUM_CATEGORY_MIN_LEVEL=100` locks out every existing category manager (trust
5, level 0). Recorded rather than defended in code, because the whole point of
config is that the operator decides.
