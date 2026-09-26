# FicNexus — specification: the F7 permission gates need an owner decision

Status: **blocked on product decision.** Written 2026-09-26.
Companion to `docs/specs/f7-leveling.md` (which is implemented).

## 1. What is left

Two tests. `f7_level_gate_admin` and `f7_level_gate_curator`. Both assert 403
and get 200 and 409 respectively. Unlike the rest of F7, **the config does not
say what these should be** — the exp curve is fully specified, the gates are
not.

## 2. What the tests expect

### 2.1 `f7_level_gate_admin` — category creation needs level >= 100

The test is explicit and unambiguous about the threshold:

```rust
// Level 99 (not 100): cannot create category (token must carry level 99).
UPDATE users SET level = 99, exp = 9900 WHERE id = $1
let token = auth_header_level(uid, u, 10, 99);
POST /api/forum/categories  ->  assert_eq!(s, StatusCode::FORBIDDEN)

// Level 100: can.
UPDATE users SET level = 100, exp = 10000 WHERE id = $1
```

The token carries `trust_level 10` and `level 99`. So the test is saying: trust
level 10 is **not** sufficient, and level 100 is the threshold.

**What the code does** — `src/routes/forum.rs:209-215`:

```rust
fn require_admin(auth: &AuthUser) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(uid)
}
```

It gates on `trust_level < 5` and **never reads `auth.level`**. With trust 10
the check passes, hence the 200.

So the gap is real and precisely located: the handler authorises on one axis
(trust) where the product intends two (trust *and* level).

### 2.2 `f7_level_gate_curator` — moderation needs level >= 50 AND account age

```rust
// Curator at level 50 (14+ days old): can moderate.
UPDATE users SET level = 50, exp = 5000,
                created_at = NOW() - INTERVAL '30 days' WHERE id = $1
POST /api/forum/posts/{id}/moderate  ->  assert_eq!(s, StatusCode::OK)
```

The comment says "14+ days old" and the fixture uses 30. **Account age is a
fourth gate dimension that appears nowhere in the config** — not in the 25 forum
fields, not in `AuthUser`. The test currently gets 409, which is a conflict
response rather than a permission one, so the test may be reaching a different
failure entirely (another post moderated, or a daily-cap collision) rather than
failing the gate.

## 3. Why this is not being decided here

`require_admin` has **four call sites** — `forum.rs:1297`, `:1325`, `:2078`,
`:2113`. Changing it from a trust check to a level check changes who can do all
four things. Concretely:

- Every existing moderator with `trust_level >= 5` and `level < 100` loses
  category and admin-forum access on deploy.
- `level` is earned through forum participation (`forum_exp_per_level` = 100).
  A trusted moderator who has not posted 100 exp gets locked out.
- Conversely, a user who posts enough to reach level 100 but has never been
  granted trust would gain admin access if the check became level-only.

Neither of those is obviously right, and the correct answer depends on whether
level and trust are meant to be **independent gates (both required)** or a
**single progression (level implies trust)**. That is a product decision with
real user-facing consequences, and it is not derivable from the code.

The safe reading — and the one the tests encode — is that they are independent
and both required. But "safe" here means "breaks existing moderators on
deploy", which is exactly the kind of call an owner should make consciously.

For 2.2 the account-age rule is worse: there is no field for it, so implementing
it means inventing both the threshold's home (config? hardcoded?) and the
mechanism.

## 4. What is needed to unblock

1. **Are level and trust independent gates, or does level imply trust?**
2. **What is the level threshold for each of the four `require_admin` call
   sites?** The test says 100 for category creation. The others are unspecified.
3. **Should `require_admin` be one function or four?** Today one function
   governs all four, so a single threshold applies to all of them. If the
   intended thresholds differ per route, this needs splitting.
4. **For the curator gate: is account age real?** If so, where does the 14-day
   threshold live, and should it be config?

## 5. Recommendation, for whoever decides

Add the level check **in addition to** the existing trust check, not instead of
it, and gate the rollout:

```rust
fn require_admin(auth: &AuthUser) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(uid)
}
```

becomes a `require_admin_at_level(auth, min_level)` with the level threshold
read from `Config`, defaulting to today's behaviour (0) so nothing changes until
the value is set deliberately. That way the decision is a config change on
deploy rather than a code change, and it is reversible.

**This is a recommendation, not a decision, and nothing was changed.**

## 6. State

These two tests stay red. `forum_api` will not be fully green, and
`f7_level_gate_curator` in particular may not even be failing for the reason its
name suggests — the 409 suggests it is not reaching the gate. That is recorded
rather than hidden.
