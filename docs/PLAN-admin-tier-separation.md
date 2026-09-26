# FicNexus — implementation plan: staff/admin tier separation

Companion to `docs/specs/admin-tier-separation.md`. Executable from this text
alone.

## Step 1 — add the config field (R4)

**File:** `src/config.rs`.

Add next to the other trust thresholds (`forum_mod_min_trust` at :330,
`forum_resolve_min_trust` at :333):

```rust
pub admin_min_trust: i16,
```

Three places, matching how every other field is wired:

1. struct-literal default near :551 — `admin_min_trust: 10,`
2. `from_env` near :1177:
   ```rust
   let admin_min_trust = std::env::var("ADMIN_MIN_TRUST")
       .ok().and_then(|s| s.parse().ok()).unwrap_or(10);
   ```
3. the struct construction near :1401 — `admin_min_trust,`

**Default 10, not 5.** The whole point is that the current behaviour is wrong,
and a default that preserves it would mean the fix does nothing until someone
opts in. Note this differs from the struct-literal convention used by
`forum_*` fields (which default to 0) — deliberately, and the reason is above.

Add a unit assertion beside the existing ones at :1730:

```rust
assert_eq!(config.admin_min_trust, 10);
```

## Step 2 — centralise the guard (R3)

**File:** `src/routes/admin.rs`.

**77 handlers across 17 files** contain a copy-pasted
`if user.trust_level < 5 { return ... }` — 37 in `admin.rs` and 5 more in
`auto_tag.rs`, which is a separate file and easy to miss. Verified:

    grep -rc 'trust_level < 5' src/routes/*.rs | grep -v ':0'

There is currently **no** `fn require_` helper anywhere in `admin.rs`
(`grep -n 'fn require_' src/routes/admin.rs` returns nothing), so there is no
existing style to match and one is being introduced. Put it where every route in
the module can reach it — `src/routes/admin.rs` if the routes being moved are
all there, otherwise a shared module, because `auto_tag.rs` needs it too and
duplicating the helper would recreate exactly the problem R3 exists to fix.

```rust
/// Gate an administrator-tier route.
///
/// Reads the threshold from `Config` (ADMIN_MIN_TRUST, default 10) rather than
/// hardcoding it: 37 inline `trust_level < 5` checks is why the staff/admin
/// boundary drifted in the first place (docs/specs/admin-tier-separation.md).
///
/// The claim is `AuthUser.trust_level`, which JWT issuance reads from
/// `users.trust_level` (src/routes/auth.rs:145-162). It is NOT `users.role`,
/// which is a separate legacy column that no admin route consults.
fn require_admin_tier(
    auth: &AuthUser,
    state: &AppState,
) -> Result<i32, AppError> {
    let uid = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("login required".to_string()))?;
    if auth.trust_level < state.config.admin_min_trust {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(uid)
}
```

**Verify the exact `AppError` variants and the `AuthUser` import path used in
this file before writing it** — `admin.rs` already has a `require_admin`-shaped
helper or inline checks; match what is there rather than inventing a second
style. `grep -n 'fn require_' src/routes/admin.rs`.

## Step 3 — move only the six families (R1, R5)

Replace the inline `trust_level < 5` check with `require_admin_tier` in exactly
these handlers, and nothing else:

| handler | route |
|---|---|
| `admin_bots` | `GET /api/admin/bots` |
| `admin_bot_shadowban` | `POST /api/admin/bots/{id}/shadowban` |
| `moderation_comments` | `GET /api/admin/moderation/comments` |
| `admin_hide_comment` | `POST /api/admin/moderation/comments/{id}/hide` |
| `admin_stats` | `GET /api/admin/stats` |
| `list_blacklist`, `blacklist_fic`, `blacklist_author` | `/api/admin/blacklist*` |
| the auto-tag queue handler | `GET /api/admin/auto-tag/queue` |

**The auto-tag routes are NOT in `admin.rs`.** They live in
`src/routes/auto_tag.rs`, which has 5 inline `trust_level < 5` checks of its
own. Locate the handler by path:

```bash
grep -n 'auto-tag/queue' src/routes/auto_tag.rs
```

A plan that only touched `admin.rs` would have fixed six of the seven tests and
missed this one entirely.

**R5 check, and this is the one that matters:** count before and after.

```bash
grep -rc 'trust_level < 5' src/routes/*.rs | grep -v ':0'   # 77 total, 17 files
```

After the change the **total across all files** must be 77 minus the number of
handlers moved, and `auto_tag.rs` must have dropped too. If the total dropped
by more, a route was moved that no test asked about, and that locks out working
moderators on deploy for no stated reason.

## Step 4 — verification (R1 and R2 together)

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='fichub-test-secret'
cargo build                                          # clean
cargo test --lib                                     # 935 passed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'   # 0
cargo test --test admin_api -- --include-ignored --test-threads=1
```

Expected: the seven tier tests pass.

**R2 is asserted by the same tests.** Each one sends a trust-5 token expecting
403 *and* a trust-10 token expecting 200. A gate that returned 403 to everyone
would pass the first half and fail the second — so the trust-10 half is the
assertion that catches an over-correction. Do not skip it by running only the
403 assertions.

Confirm the config is really driving it:

```bash
ADMIN_MIN_TRUST=5 cargo test --test admin_api admin_bots_requires_role_10 \
  -- --include-ignored --test-threads=1
```

It must now FAIL, because at threshold 5 a trust-5 user is allowed. If it
passes, the value is hardcoded and step 1 did nothing.

## Step 5 — fix the `seed_admin_user` naming (R6)

**File:** `tests/admin_api.rs:184`.

The helper writes `users.role` while its parameter is called `role` and callers
pass what they mean as a trust level. Rename the parameter and leave the
column alone — the column is what it has always written, and R6 is about the
naming being misleading, not about behaviour.

```rust
async fn seed_admin_user(db: &sqlx::PgPool, username: &str, legacy_role: i16) -> i32 {
```

Add a comment stating which column it writes and which one the routes read:

```rust
/// Seeds the legacy `users.role` column.
///
/// NOT the trust level. Admin routes authorize on `AuthUser.trust_level`,
/// which JWT issuance reads from `users.trust_level` (src/routes/auth.rs:145).
/// Callers pass the trust level they want to the JWT via `auth_header`;
/// this value is essentially unused by the routes under test.
```

## Step 6 — commit

- `fix(auth): require admin tier (TL10) for admin-only endpoints` — steps 1-3
- `test(admin): correct seed_admin_user naming — it writes users.role, not trust_level` — step 5

Tag `admin-tier-separation-2026-09-26`. Sync to `~/code/rust/ficnexus` and commit
the session note to `~/secondbrain`.

## Out of scope, recorded not hidden

- **The other 31 admin routes** stay at `trust_level < 5`. Section 6 of the
  spec lists the ones that look wrong by name, including `admin_users` and
  `set_user_role`, where a moderator being able to promote users is a
  privilege-escalation path. No test states an intent, so nothing is changed.
  Step 2's helper makes it a one-line change each once an owner decides.
- **`admin_users_search_role_ban`** fails for a separate reason: the test sends
  `{"role": 5}` and asserts `users.role` persisted, but the handler requires
  `trust_level` and writes `users.trust_level`. The handler is correct —
  `trust_level` is the claim every route authorizes on. The test is stale. Not
  fixed here because it is a different defect with a different owner question
  (should the endpoint still be called `role`?).
