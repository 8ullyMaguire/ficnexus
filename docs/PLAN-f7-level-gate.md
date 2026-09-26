# Plan — implement the F7 level gate

Implements `docs/specs/f7-level-gate.md`. Every step has an exact command and
its expected output. Run from `~/code-local/rust/ficnexus`.

## 0. Environment for every step

```bash
cd ~/code-local/rust/ficnexus
export CARGO_TARGET_DIR=~/.cargo-target/ficnexus
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379'
export JWT_SECRET='fichub-test-secret'
```

**Never run anything else against `ficnexus_test` while a test run is in
flight.** `run_db_suites.sh` takes an flock on `/tmp/ficnexus_test_db.lock`;
a standalone `provision_test_db.sh` in another shell will drop the database out
from under a running suite. This happened once already.

## 1. Config: three new thresholds

**File:** `src/config.rs`

### 1.1 Struct fields

Find the block containing `forum_mod_min_trust` (around line 330) and add
immediately after `forum_resolve_min_trust`:

```rust
    /// Minimum forum level to manage bans. 0 = level axis inert (default).
    pub forum_mod_min_level: i16,
    /// Minimum forum level to create or edit categories. 0 = inert (default).
    pub forum_category_min_level: i16,
    /// Minimum forum level to apply a moderation action. 0 = inert (default).
    pub forum_curator_min_level: i16,
```

### 1.2 Defaults

Find the `Default` impl block containing `forum_mod_min_trust: 0` (around line
555) and add after `forum_resolve_min_trust: 0`:

```rust
            forum_mod_min_level: 0,
            forum_category_min_level: 0,
            forum_curator_min_level: 0,
```

### 1.3 Environment parsing

Find the `from_env` block containing `forum_mod_min_trust` (around line 1181).
Mirror its exact style — read the existing lines first and match them, including
the fallback value and any `.parse().unwrap_or(...)`:

```rust
        let forum_mod_min_level = std::env::var("FORUM_MOD_MIN_LEVEL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let forum_category_min_level = std::env::var("FORUM_CATEGORY_MIN_LEVEL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let forum_curator_min_level = std::env::var("FORUM_CURATOR_MIN_LEVEL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
```

### 1.4 Struct construction

Find the `Self { .. }` construction (around line 1405, beside
`forum_mod_min_trust`) and add the three names.

### 1.5 Unit test

Find the existing assertion near line 1729:

```rust
        assert_eq!(config.forum_mod_min_trust, 4);
        assert_eq!(config.forum_resolve_min_trust, 5);
```

Add, proving the default is inert:

```rust
        assert_eq!(config.forum_mod_min_level, 0, "level axis must be inert by default");
        assert_eq!(config.forum_category_min_level, 0);
        assert_eq!(config.forum_curator_min_level, 0);
```

**Verify step 1:**

```bash
cargo build 2>&1 | grep -E '^(error|warning: unused)' -A5 | head -20
cargo test --lib config 2>&1 | grep -E '^test result'
```

Expected: no errors; the config tests pass.

**Commit:**

```bash
git add src/config.rs
git -c user.name='Hermes Agent' -c user.email='hermes@local' commit -m \
  'feat(config): three forum level thresholds, default 0 so the level axis is inert'
```

## 2. The admin gate, split by intent

**File:** `src/routes/forum.rs`

### 2.1 Replace `require_admin`

Current (line 209):

```rust
fn require_admin(auth: &AuthUser) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(uid)
}
```

Replace with:

```rust
/// Trust is the base requirement for every forum admin action. Level is an
/// additional, independent axis: both must pass. The owner decided level and
/// trust are independent gates rather than one progression, so neither implies
/// the other and neither is sufficient alone.
fn require_admin_at_level(auth: &AuthUser, min_level: i16) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    if auth.level < min_level {
        return Err(AppError::Forbidden(
            "Forum level too low for this action".to_string(),
        ));
    }
    Ok(uid)
}

/// Ban management. `is_admin` is deliberately not an alternative path here:
/// this is the forum gate, governed by forum participation, not the
/// `/api/admin/*` tier that `require_admin_tier` guards.
fn require_forum_moderator(auth: &AuthUser, state: &AppState) -> Result<i32, AppError> {
    require_admin_at_level(auth, state.config.forum_mod_min_level)
}

/// Category create/edit — the strictest forum action.
fn require_forum_category_admin(auth: &AuthUser, state: &AppState) -> Result<i32, AppError> {
    require_admin_at_level(auth, state.config.forum_category_min_level)
}
```

Confirm `AuthUser.level` is `i16` and `trust_level` is `i16` by reading the
struct in `src/routes/auth.rs` before writing the comparisons. If `level` is
`i32`, drop the casts.

### 2.2 Repoint the four call sites

| line | route | new call |
|---|---|---|
| 1297 | `DELETE /api/admin/forum/bans/{id}` | `require_forum_moderator(&auth, &state)?` |
| 1325 | `GET /api/admin/forum/bans` | `require_forum_moderator(&auth, &state)?` |
| 2078 | `POST /api/forum/categories` | `require_forum_category_admin(&auth, &state)?` |
| 2113 | `PATCH /api/forum/categories/{id}` | `require_forum_category_admin(&auth, &state)?` |

Site 1297 is currently `let actor_id = require_admin(&auth)?;` — keep the
binding, change only the call.

**Verify step 2:**

```bash
cargo build 2>&1 | grep -E '^error' -A6 | head -20
```

Expected: clean. If `state` is not in scope at a call site, the handler's
signature is missing `State(state): State<Arc<AppState>>`; add it.

**Commit:**

```bash
git add src/routes/forum.rs
git -c user.name='Hermes Agent' -c user.email='hermes@local' commit -m \
  'feat(forum): split require_admin into moderator and category-admin gates, level as an independent axis'
```

## 3. The curator gate

**File:** `src/routes/forum.rs`, `moderate_post` (line 673)

Leave `require_trust_queue` (line 398) alone — it guards several routes and
widening it would lock out moderators the two tests never mention.

Insert **after** `require_trust_queue` and **before** `check_daily_cap`, so a
level rejection is not masked by a cap exhaustion:

```rust
    require_trust_queue(&state, &auth).await?;
    if auth.level < state.config.forum_curator_min_level {
        return Err(AppError::Forbidden(
            "Forum level too low to moderate".to_string(),
        ));
    }
    check_daily_cap(&state, user_id).await?;
```

**Verify step 3:**

```bash
cargo build 2>&1 | grep -E '^error' -A6 | head -20
```

**Commit:**

```bash
git add src/routes/forum.rs
git -c user.name='Hermes Agent' -c user.email='hermes@local' commit -m \
  'feat(forum): level gate on /moderate, checked before the daily cap'
```

## 4. Test fixes

**File:** `tests/forum_api.rs`

### 4.1 `f7_level_gate_curator` — defect D1, wrong level in the token

Line ~3560 reads `let token2 = auth_header(cur_id, u_cur, 5);`. That helper maps
its third argument through `10→100, 5→50, 1→1`, so the "level 49" case carries
level 50. Change to:

```rust
    let token2 = auth_header_level(cur_id, u_cur, 5, 49);
```

### 4.2 `f7_level_gate_curator` — defect D2, duplicate moderation

The test moderates one post twice. The second call returns 409
("You already moderated this post") before the gate can reject it, which is why
the test has been red for the wrong reason. Seed a second post after the first
moderation and use it for the level-49 case:

```rust
    // A distinct post: the level-49 case must be rejected by the gate, not by
    // already_modded(). Reusing post_id here returns 409, which is how this
    // test stayed red while asserting a level gate that was never reached.
    let post2_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1 OFFSET 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("second op post id");
```

`seed_topic` must create at least two posts for this `OFFSET 1` to resolve. If it
creates one, insert a second explicitly before this query:

```rust
    sqlx::query("INSERT INTO forum_posts (topic_id, author_id, body, score, created_at)
                 VALUES ($1, $2, 'fr7 gate second', 0, NOW())")
        .bind(tid)
        .bind(author_id)
        .execute(&db)
        .await
        .ok();
```

Then point the level-49 request at `post2_id`.

### 4.3 Pin the thresholds so the tests cannot pass vacuously

Both tests must set the config values they assert, or they pass because the
threshold is 0 and the level axis is inert. `app()` in `forum_api.rs` builds the
test state; find where it constructs `AppState` and set the three fields in a
test-only override. If `app()` takes no config parameter, add one with a
default so no other test changes:

```rust
async fn app_with(cfg: Config) -> Router { /* as app(), but state.config = cfg */ }
async fn app() -> Router { app_with(Config::default()).await }
```

Then in the two tests use `app_with(...)` with the threshold set, e.g. for the
curator test:

```rust
    let mut cfg = Config::default();
    cfg.forum_curator_min_level = 50;
    let app = app_with(cfg).await;
```

and for the admin test `cfg.forum_category_min_level = 100;`.

A test whose asserted value is the default proves nothing — that is precisely
the `authors.rs::test_auto_approve_only_admin` defect removed during the
admin-flag work.

### 4.4 `f7_level_gate_admin` — pin that `is_admin` is not a path

Add after the level-100 success case, so the "independent gates" decision is
pinned rather than merely implied:

```rust
    // An /api/admin/* administrator is not automatically a forum category
    // admin: the forum gate is trust + level, not the is_admin flag.
    sqlx::query("UPDATE users SET level = 0, exp = 0, is_admin = true WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .ok();
    let token3 = auth_header_level(uid, u, 10, 0);
    let (s, b) = post_json(
        &app,
        "/api/forum/categories",
        Some(&token3),
        json!({ "slug": "fr7-adm2", "title": "F7 Adm2" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "is_admin must not bypass the forum level gate: {b}");
```

**Verify step 4:**

```bash
bash scripts/provision_test_db.sh
cargo test --test forum_api f7_level_gate -- --include-ignored --test-threads=1 2>&1 | tail -20
```

Expected: `2 passed; 0 failed`.

**If `f7_level_gate_curator` still fails with 409**, the second post is not
distinct — print `post2_id` and `post_id` and confirm they differ. If it fails
with 500, `state.config` is not reachable in the handler; check the signature.

**Commit:**

```bash
git add tests/forum_api.rs
git -c user.name='Hermes Agent' -c user.email='hermes@local' commit -m \
  'test(forum): fix f7_level_gate_curator - it asserted 409 duplicate-moderation, and its level-49 token carried level 50'
```

## 5. Prove the gates have teeth

A test that passes with the gate removed is not a test. Mutate, run, restore:

```bash
# break the category gate
cp src/routes/forum.rs /tmp/forum.rs.bak
python3 - <<'EOF'
p='src/routes/forum.rs'; t=open(p).read()
t=t.replace("    if auth.level < min_level {\n        return Err(AppError::Forbidden(\n            \"Forum level too low for this action\".to_string(),\n        ));\n    }\n", "")
open(p,'w').write(t)
EOF
bash scripts/provision_test_db.sh
cargo test --test forum_api f7_level_gate -- --include-ignored --test-threads=1 2>&1 | grep -E '^test result'
cp /tmp/forum.rs.bak src/routes/forum.rs
bash scripts/provision_test_db.sh
cargo test --test forum_api f7_level_gate -- --include-ignored --test-threads=1 2>&1 | grep -E '^test result'
```

Expected: `0 passed; 2 failed` with the gate removed, `2 passed; 0 failed`
restored. **Both runs must be recorded in the commit message.** If the first
passes, the tests are vacuous — go back to step 4.3.

## 6. Regression sweep

```bash
cargo test --lib 2>&1 | grep -E '^test result'
```

Expected: `935 passed; 0 failed` — unchanged. The config additions must not
move this number; if it does, a default changed something.

```bash
bash scripts/run_db_suites.sh
```

Expected: `38 pass, 19 fail, 0 no-result, 57 suites` — the baseline in
`docs/BASELINE-db-suites.md`, with `forum_api` at 8 failures (down from 8, or
better if the two F7 tests go green). **Do not touch the database while this
runs.**

If `forum_api` is 6, both F7 tests are fixed. If it is 9, a gate is too strict —
compare per-test failures against the baseline table, not the count alone.

**Commit:**

```bash
git add -A
git -c user.name='Hermes Agent' -c user.email='hermes@local' commit -m \
  'test: full sweep after the F7 level gate'
git tag -a f7-level-gate-2026-09-26 -m 'trust and level are independent gates; thresholds in Config, default 0'
```

## 7. Docs and sync

Update `docs/specs/f7-gate-assignments.md` with a header pointing at the new
spec, so the old document does not stand as the current answer:

```markdown
> **Superseded 2026-09-26** by `docs/specs/f7-level-gate.md`. The gap analysis
> here is correct, but §2.2 and §3 are not: `f7_level_gate_curator` exercises
> `require_trust_queue`, not `require_admin`, and its 409 is a duplicate
> moderation inside the test rather than a gate failure.
```

Append the F7 gate to the repo README's feature list if one exists.

```bash
rsync -a --delete --exclude 'target' --exclude 'node_modules' --exclude '.env' \
  ~/code-local/rust/ficnexus/ ~/code/rust/ficnexus/
cd ~/code/rust/ficnexus && git log --oneline | head -1
```

Then log the cycle in `~/secondbrain/90-Meta/`.
