# FicNexus — specification: admin tier (TL10) is unreachable by any real login

Status: **blocked on an owner decision.** Found 2026-09-26 while fixing
`reports_api`. Nothing changed; this document records the contradiction and the
options.

## 1. The contradiction

Three facts, each verified against the code and the live test database:

1. `users.trust_level` is `CHECK (trust_level BETWEEN 0 AND 6)` —
   `migrations/013_trust_levels.sql:25`, confirmed live as
   `CHECK (((trust_level >= 0) AND (trust_level <= 6)))`.

2. JWT issuance mints the `trust_level` claim **from that column**:
   `src/routes/auth.rs:145-168` selects `trust_level` in the login query and
   assigns `trust_level: row.2` to the `AuthUser` that `create_token` signs.

3. Administrator-tier routes require `Config::admin_min_trust`, default **10**
   (`src/config.rs:563`, env `ADMIN_MIN_TRUST`), enforced by
   `trust::require_admin_tier` against the claim.

Therefore `2 -> 1 -> 3` gives: **no account that logs in can hold a token with
`trust_level >= 10`.** The maximum reachable claim is 6. Every admin-tier route
is unreachable in production, and the only tokens that pass are the ones the
tests mint by hand.

The highest `trust_level` present in the live test database is 5, so the gap is
not theoretical there either.

## 2. Why the tests did not catch it

`admin_api` and `reports_api` mint tokens directly:

```rust
fn auth_header(user_id: i32, trust_level: i16, username: &str) -> String
```

`auth_header(..., 10, ...)` builds a token the product **cannot issue**. The tests
therefore validate that the routes read the claim correctly, which is real and
worth validating, and they simultaneously prove the claim is unattainable.

This is the same shape as the `users.role` / `trust_level` overload across three
suites: the test helper constructs the state the code under test expects, and
nothing in the test suite establishes that the product can produce that state.

There is no test that logs in as a real user and then calls an admin endpoint.
That absence is the actual gap.

## 3. Options

**Option A — raise the column ceiling to 10 (or 12).**
Requires a migration on an APPLIED file's table. `013_trust_levels.sql` is the
origin of the constraint but must not be edited; a new numbered migration
altering the constraint is the mechanism. Then decide who gets promoted, and how
— by hand, by `set_user_role`, or by a trust ladder.

*Consequence:* TL10 becomes reachable, and the tier separation this cycle just
built becomes real rather than aspirational. Also makes `users.role` more clearly
redundant for authorization, which is a second decision.

**Option B — lower `admin_min_trust` to 6.**
One-line config default, no migration.

*Consequence:* the top of the trust ladder and the admin tier collapse into the
same value. TL5 moderators and TL6 users both sit one step below admin, and
there is no headroom left in the ladder — the next rung has nowhere to go. This
also contradicts what the tests assert (TL5 must be blocked, TL10 allowed): at 6
the tests mint an unattainable 10 and the boundary becomes untestable.

**Option C — introduce a separate admin flag.**
`users.is_admin BOOLEAN` (or keep `users.role`, which already exists and already
has a consumer). Admin routes check the flag; `trust_level` stays the 0-6
community ladder. The 11 routes moved this cycle revert to reading a flag.

*Consequence:* models "administrator" as a distinct capability rather than as
the top of a trust ladder — which is what the existing schema already implies,
since it has a separate `role` column. Costs a migration and a change to the 11
gates. Most correct, most work.

## 4. Recommendation

**Option C**, on the evidence in the schema rather than on taste: `013_trust_levels.sql`
defines a 0-6 *community* ladder, and `users.role` exists as a separate column
precisely because being an administrator was not on that ladder. A single 0-10
scale would have no reason to carry two columns.

If Option A is chosen instead, the same migration must also state how a user is
promoted, because a ceiling with no promotion path is an unreachable tier in a
different way.

## 5. What is NOT blocked

`reports_api` and the trust-gate fixture fixes from this cycle stand on their own
and are committed. The report-filing gate needs `trust_level >= 1`, which is
within the ladder. Only the admin tier is affected, and nothing in this cycle made
that worse — the routes previously checked `< 5`, which *was* reachable, so the
change from 5 to 10 did close a working path.

That is stated plainly because it is a real consequence of the previous commit
and the owner should hear it from the log rather than discover it:

- Before: moderators (TL5) could reach admin endpoints, and a real login could
  reach them too.
- After: no real login can reach admin endpoints at all.

Neither state is correct. The gap is now closed *and* the door is locked, and
Options A/B/C are about which side to open.

## 6. Acceptance for whichever option is chosen

1. A test that logs in through the real path (`POST /api/auth/login`), then calls
   an admin-tier endpoint with the returned token, and expects 200. This is the
   test whose absence let the contradiction survive. It must fail before the fix.
2. The same test with a TL5 token expects 403 — already covered, and must stay.
3. `SHOW CREATE TABLE users` confirms the constraint matches whatever the option
   chose.
