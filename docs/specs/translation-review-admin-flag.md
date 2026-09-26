# translation_review_api: 7/7 from 0/7 — the admin flag, not the role

Date: 2026-09-27 · Status: implemented and green
Suite: `tests/translation_review_api.rs`

## The failure

Every one of the seven tests failed, six with `403` and one with `401`. Uniform
auth failures, so the cause was in the token, not the endpoints. It was the most
completely broken suite in the repo.

## Two independent defects

### 1. The token hardcoded `is_admin: false` (six tests)

    fn auth_header(user_id: i32, username: &str, trust_level: i16) -> String {
        ...
        is_admin: false,
    }

Every route under test gates on `require_admin_tier`, which reads `auth.is_admin`
**from the token**:

    pub fn require_admin_tier(auth: &AuthUser, _state: &AppState) -> Result<i32, AppError> {
        let uid = auth.user_id.ok_or_else(|| AppError::Unauthorized(...))?;
        if !auth.is_admin {
            return Err(AppError::Forbidden("Admin access required".to_string()));
        }
        Ok(uid)
    }

The tests seeded `users.role = 10` and trusted that to mean administrator. It
does not. `is_admin` is a separate boolean column, and `users.trust_level` is
CHECK-constrained to 0-6, so **no trust value can express "administrator"** —
which is the entire reason the flag exists (`docs/specs/admin-flag.md`). The old
`trust_level < 5` gates this work replaced were reachable at all only because
that constraint had been loosened somewhere; the flag is the supported path.

Fix: `auth_header` takes an `is_admin` parameter, and a new `set_admin` helper
writes the column. Both are required, because the claim is what the gate reads
and the column is the source of truth — a test that sets only one disagrees
with production.

This is the same shape as `tests/admin_api.rs::auth_header_for` / `set_admin`,
which already did it correctly. This suite predated that work and kept the old
shape. The fix is a port, not a new idea.

### 2. Anonymous asserted as 403 (two tests)

    // Role gating: anonymous → 400 {err:401}, sub-admin → HTTP 403.
    assert_eq!(s, StatusCode::FORBIDDEN);

A 403 tells a client to retry with different credentials. An anonymous caller
sent none, so the correct answer is 401. The file header had this recorded as
house style ("anonymous callers get HTTP 400 {err:401}") — with a `400` that
contradicts the `403` asserted three lines below it. Split into 401 anonymous /
403 authenticated-non-admin, in both `translation_review_role_gate` and
`char_score_fix_validation`.

### 3. The documented run command was dead

    set -a; . /personal/documents/code/rust/fichub/.env; set +a;

A path from a different machine — the working copy is `fichub`, this repo is
`ficnexus`. The header also did not mention `--include-ignored`, without which
every one of these tests reports `ignored` and **executes nothing**. Replaced
with the flags `scripts/run_db_suites.sh` actually uses.

Worth flagging: this dead instruction is what made me nearly file a wrong
finding. I ran the suite through a quick helper without
`--include-ignored`, got `0 passed; 0 failed; 7 ignored`, and generalised it
into a spec claiming **46% of the whole test suite never runs**. The runner
passes `--include-ignored`, so the real result was `0 passed; 7 failed`. The
spec was retracted — see `docs/specs/ignored-tests.md`.

## Scope check

Scanned all 58 suites for the same hardcoded-`is_admin` pattern against
`require_admin_tier`-gated handlers. **This was the only one.** The `roadmap_api`
anonymous-403 tests are left alone deliberately: they gate on trust, not
`require_admin_tier`, and 403 is defensible for a trust gate on a route that is
not admin-scoped. Not investigated further here.

## Result

    translation_review_api: test result: ok. 7 passed; 0 failed; 0 ignored
