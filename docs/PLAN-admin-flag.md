# FicNexus — implementation plan: a separate administrator flag

Companion to `docs/specs/admin-flag.md`. Executable from this text alone.
Owner-approved 2026-09-26.

## Step 1 — migration

**New file:** `migrations/093_admin_flag.sql`. Confirm 093 is free first
(`ls migrations/ | tail -5`); `093_report_category.sql` may already exist, in
which case use the next free number.

```sql
-- Administrator is a distinct capability, not a rung on the trust ladder.
-- 013_trust_levels.sql defines trust_level as CHECK (0..6) and JWT issuance
-- mints the claim from that column, so no login can hold a token with
-- trust_level >= 10. Admin routes authorize on this flag instead.
-- See docs/specs/admin-flag.md.
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT false;

-- Partial index: the admin list reads this, and it is a tiny fraction of users.
CREATE INDEX IF NOT EXISTS users_is_admin_idx ON users (id) WHERE is_admin;

COMMENT ON COLUMN users.is_admin IS
  'Administrator. Distinct from trust_level (0-6 community ladder) and from the legacy users.role.';
```

`users.role` is not touched. Do not add a CHECK or FK to it.

**Verify:**

```bash
psql "$DATABASE_URL" -c "\d users" | grep is_admin
psql "$DATABASE_URL" -tAc "SELECT count(*) FROM users WHERE is_admin"   # 0
```

## Step 2 — carry the flag into the token

**File:** `src/routes/auth.rs`.

1. `User` struct — add `pub is_admin: bool`.
2. Login `SELECT` at `auth.rs:145` — add `is_admin` to the column list, and add
   the field to the `AuthUser` construction at `auth.rs:158-167`.
3. `AuthUser` (line 212) — add `pub is_admin: bool`, with a comment saying it is
   the administrator flag and distinct from `trust_level`. Update `Default` to
   `false`.
4. `Claims` + `create_token` (line 75) — add `is_admin: bool`, and set it from
   `user.is_admin`.
5. Wherever `Claims` is decoded into `AuthUser` (the `from` / extractor path),
   copy `is_admin` across.

Every SELECT that builds a `User` for token creation must now supply the column,
or it will not compile — that is deliberate, and the compiler is the checklist
for step 2.

**Verify:** `cargo build` and grep for every `User {` construction:

```bash
grep -rn 'User {' src/ --include=*.rs
```

## Step 3 — the gate reads the flag

**File:** `src/services/trust.rs`, `require_admin_tier` (line 634).

Replace the `auth.trust_level` comparison with `auth.is_admin`. Keep the
`admin_min_trust` lookup out of it — see step 4. The function name stays: it is
still the administrator tier. Add a doc comment recording *why* it reads a flag
and not a ladder, with a pointer to the spec, because the next person will look
at a 0-6 ladder and a name like "admin tier" and try to reconcile them.

## Step 4 — remove the dead config

**File:** `src/config.rs`.

`admin_min_trust` was added earlier today for the trust-threshold approach that
option C replaces. Remove the struct field, the `from_env` read (line 1199), the
default (line 563), and the unit assertion added for it. Remove `ADMIN_MIN_TRUST`
from `.env.example` and any docs that mention it. R6 — a config field nothing
reads is worse than no field, because it looks configurable.

## Step 5 — rewrite the test helpers

**File:** `tests/admin_api.rs` and every other suite calling `auth_header`.

`auth_header(user_id, 10, name)` mints a token the product cannot issue. Change
the helper to take the flag:

```rust
/// Mints a token for a user row.
///
/// `is_admin` is a real column and the admin gates read it from the JWT
/// (`docs/specs/admin-flag.md`). Trust level is separate and does not grant
/// admin. Tests that want an admin pass `true`; a TL5 moderator passes
/// `(5, false)`.
fn auth_header_for(user_id: i32, trust_level: i16, is_admin: bool, username: &str) -> String
```

Then, at every call site:

- admin token → `auth_header_for(id, <any>, true, name)` — trust becomes
  irrelevant for admin routes, which is the point (R2)
- moderator token → `auth_header_for(id, 5, false, name)` — unchanged behaviour
- ordinary user → `auth_header_for(id, 0, false, name)`

`seed_admin_user` keeps writing `users.role`; it never mattered. Better: delete
it and seed `is_admin` directly, since its only purpose was to look like it
configured authorization. Prefer deleting — one fewer misleading helper.

Check every suite, not just `admin_api`:

```bash
grep -rln 'auth_header' tests/
```

## Step 6 — the acceptance test (R4)

This is the test whose absence let the TL10 contradiction survive. It must fail
before the fix and pass after.

**New file** `tests/admin_auth_real_login.rs`, or a test in `admin_api.rs`:

```rust
/// Log in through the real path and reach an admin endpoint with the token the
/// server issued. Hand-minted tokens hid the fact that no login could ever
/// produce trust_level >= 10; see docs/specs/admin-flag.md R4.
#[tokio::test]
#[ignore]
async fn real_login_reaches_admin_route() {
    let _g = db_guard();
    let db = pool().await;
    // A plain user, trust 0, not an admin.
    let id = /* insert user, is_admin = false */;
    let pw = /* set a known password hash */;
    // Non-admin: 403.
    let (s, _) = post_json(&app, "/api/auth/login",
        None, json!({ "username": NAME, "password": PW })).await;
    let token = /* from the login response */;
    let (s, _) = get_json(&app, "/api/admin/stats", Some(&token)).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "non-admin must be 403");

    // Promote, then log in again: 200. Uses a fresh token, because the claim is
    // baked in at issue - an existing token will not change.
    sqlx::query("UPDATE users SET is_admin = true WHERE id = $1")...;
    let (s, _) = post_json(&app, "/api/auth/login", ...) // re-login
    let (s, b) = get_json(&app, "/api/admin/stats", Some(&new_token)).await;
    assert_eq!(s, StatusCode::OK, "admin must reach admin route: {b}");
}
```

Use the same helpers the rest of `admin_api` uses for login, so the test is not
a parallel implementation of auth. Find the existing login helper first
(`grep -n 'async fn login\|/api/auth/login' tests/admin_api.rs`).

## Step 7 — gate

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='fichub-test-secret'
cargo build
cargo test --lib                                    # 935
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'   # 0
cargo test --test admin_api -- --include-ignored --test-threads=1
```

Expected: `admin_api` 12 passed, 1 failed — the remaining one is the stale
`admin_users_search_role_ban` fixture, documented in
`admin-tier-separation.md` §3 and out of scope here.

## Step 8 — promotion path (R7)

Hand-editing the column is not a promotion path — that was the objection to
option A. The `set_user_role` route (`src/routes/admin.rs`) already exists and
already writes an authorization-relevant column; it should set `is_admin` too.

Decide explicitly, and write it in the commit message:

- `set_user_role` gains an `is_admin` field, **or**
- a separate `POST /api/admin/users/{id}/is_admin` endpoint

Prefer the second: `set_user_role` is named for a column that is not the one
authorization reads, and the name is the problem this whole cycle has been
about. Whatever is chosen must be behind `require_admin_tier`, so it cannot be
the first privilege-escalation path in the product.

## Step 9 — commit and sync

One commit for steps 1-7, a second for step 8 if separate. Tag
`admin-flag-2026-09-26`. Update `docs/specs/admin-flag.md` with the outcome, sync
to `~/code/rust/ficnexus`, and commit the session note to `~/secondbrain`.
