# FicNexus — implementation plan: F7 gates read level alongside trust

Companion to `docs/specs/f7-authorization.md`. Owner-approved 2026-09-26.

## Step 0 — thresholds, measured not guessed

Already measured in the spec; recorded here so the plan is executable alone.

| gate | test | trust | level | expected |
|---|---|---|---|---|
| curator `POST /posts/{id}/moderate` | `f7_level_gate_curator` | 5 | 50 | 200 |
| curator, same route | same | 5 | 49 | 403 |
| curator, young account | **new case, step 4** | 5 | 50, age 0 | 403 |
| admin `POST /forum/categories` | `f7_level_gate_admin` | 10 | 99 | 403 |
| admin, same route | same | 10 | 100 | 200 |

**Two thresholds: curator 50, admin 100.** A single field cannot express both —
50 lets a curator create categories, 100 stops moderators moderating.

## Step 1 — config fields

**File:** `src/config.rs`. Three fields, four sites each, following
`forum_exp_per_level` exactly (doc comment carrying the default, struct-literal
default, `from_env` read, unit assertion):

```rust
/// Forum level required to moderate. Checked alongside trust_level >= 5.
/// Default 50. Env: FORUM_CURATOR_MIN_LEVEL.
pub forum_curator_min_level: i16,

/// Forum level required for admin forum actions, alongside trust_level >= 5.
/// Default 100. Env: FORUM_ADMIN_MIN_LEVEL.
pub forum_admin_min_level: i16,

/// Account age in days required to moderate, alongside level and trust.
/// The test's own comment says "14+ days old"; the suite does not constrain
/// this value. Default 14. Env: FORUM_CURATOR_MIN_ACCOUNT_DAYS.
pub forum_curator_min_account_days: i16,
```

**R2, R2a, R3.** Unit-assert all three defaults. `forum_admin_min_level`
already exists from the earlier admin-tier work and is being repurposed — its
semantics change from "trust level required for admin routes" to "forum level
required for admin forum actions", so update its doc comment and check every
other reader of it. `grep -rn 'admin_min_trust' src/` must come back empty of
gate call sites.

## Step 2 — the curator gate

**File:** `src/routes/forum.rs`, `require_admin` is at line 209 — but the
moderation path is a *different* gate. Find it:

```bash
grep -rn 'fn .*curator\|fn .*moderate' src/routes/forum.rs
```

Three conditions, all required, each with its own message (**R5**):

```rust
// trust (existing)
if auth.trust_level < 5 { return Err(Forbidden("moderation requires trust 5+")) }
// level (new)
if auth.level < state.config.forum_curator_min_level {
    return Err(Forbidden("moderation requires forum level {min}"));
}
// account age (new)
```

The age term needs the user's `created_at`. If the gate does not already hold a
pool, add the lookup; the test seeds `created_at`, so a column read is the
mechanism. Keep it to one query.

## Step 3 — the admin gate

**File:** the same module or wherever `f7_level_gate_admin` lands
(`POST /forum/categories`). Trust 10 **and** level 100, two separate checks with
separate messages.

Note this is the *forum* admin gate. It is a different surface from the 11
admin-tier routes now on the `is_admin` flag (`admin-flag.md`) — do not conflate
them, and do not make the category route require `is_admin`, or moderators who
can moderate today would lose category creation on deploy.

## Step 4 — add the missing age test (R2a)

The term is new, so its test is new. In `f7_level_gate_curator`, after the
level-49 case:

```rust
// Level 50 but a NEW account: still forbidden. The account-age term is
// separate from the level term, and this is the only assertion that
// distinguishes them - without it the age term is unconstrained by the suite.
// See docs/specs/f7-authorization.md section 3a.
sqlx::query("UPDATE users SET level = 50, exp = 5000, created_at = NOW() WHERE id = $1")
    .bind(cur_id).execute(&db).await.ok();
let token3 = auth_header(cur_id, u_cur, 5);
let (s, b) = post_json(&app, &format!("/api/forum/posts/{post_id}/moderate"),
    Some(&token3), json!({ "reason": "Insightful" })).await;
assert_eq!(s, StatusCode::FORBIDDEN, "young level-50 curator forbidden: {b}");
```

Must fail before step 2 and pass after.

## Step 5 — unblock the two tests

Remove the "blocked" markers from `f7_level_gate_curator` (line 3514) and
`f7_level_gate_admin` (line 3677). **R4.**

**R6.** `auth_header_level(user_id, username, trust_level, level)` at line 299 and
`auth_header` at 294 both must still compile and mint the right claims. They are
the only path to a level claim; if a change to them silently drops the level, both
F7 tests fail at once with a confusing 403 rather than a compile error.

## Step 6 — gate

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='fichub-test-secret'
cargo build
cargo test --lib                                    # 935
cargo test --test forum_api -- --include-ignored --test-threads=1
```

`forum_api` has 4 pre-existing failures unrelated to F7. The two F7 tests must
leave the blocked list; the other four are not this change's business.

**Mutation check** before committing: neuter the level comparison
(`auth.level < X` → `false`) and confirm the suite goes red. A gate that passes
with the check removed is not a gate.

## Step 7 — commit

`feat(forum): F7 permission gates check level, and account age, alongside trust`

State the three defaults, that they were read off the tests rather than chosen,
that the age term is unproven by the pre-existing suite and got a new test, and
that `level` is unbounded while `trust_level` is 0-6 — the reason this gate is
reachable and the admin-tier gate was not.

Tag `f7-gates-2026-09-26`. Sync to `~/code/rust/ficnexus`, log to `~/secondbrain`.
