# FicNexus — specification: a separate administrator flag (option C)

Status: **approved by the owner 2026-09-26.** Replaces the analysis in
`docs/specs/admin-tier-unreachable.md`, which is retained as the record of why.
Plan: `docs/PLAN-admin-flag.md`.

## 1. Decision

Administrator status becomes a **distinct capability**, not the top of the trust
ladder. `users.trust_level` stays a 0-6 community ladder exactly as
`013_trust_levels.sql` defines it. Admin-tier routes authorize on a boolean.

The owner rejected:

- **A (raise the ceiling)** — a ceiling with no promotion path is an unreachable
  tier in a different way.
- **B (lower `admin_min_trust` to 6)** — collapses the top of the ladder into
  admin, leaves no headroom for the next rung, and makes the tests' TL10
  unattainable, so the boundary becomes untestable.

## 2. The column: a new `users.is_admin`, not `users.role`

Measured before deciding, because the owner's "or reuse `users.role`" left it
open.

**`users.role` in the live database:**

| value | users |
|---|---|
| 0 | 50 |
| 1 | 9 |
| 5 | 6 |
| 10 | 22 |

It is not a boolean and not a flag. It is **a second, legacy trust ladder** with
the same 5 and 10 rungs — which is precisely why `seed_admin_user(db, name, 10)`
in `admin_api` read as though it were configuring authorization. It wasn't. That
ambiguity is what cost hours this cycle, in three suites.

**Writers:** exactly one — `subsystems.rs:526`, `UPDATE users SET role = 1 WHERE
id = $1 AND role = 0`, on subsystem approval. It means "this account was
activated", and only fires `WHERE role = 0`.

**Authorization readers: none.** Two places read it, neither for access control:

- `db/queries/proposals.rs:93` — `SELECT role FROM users WHERE id = $1`
- `routes/social.rs:913` — selects it as profile data to display

`AuthUser` already carries the comment `/// F7 trust level (0-6). Replaces legacy
role.` (`routes/auth.rs:215`) — the cutover was started and not finished.

**Decision: a new `users.is_admin BOOLEAN NOT NULL DEFAULT false`.**

Reviving `role` would mean untangling "activated" from "administrator" in a
column that has no referential integrity, no CHECK, and two display readers —
and the `WHERE role = 0` writer would have to keep working while an admin flag
lives in the same field. A separate boolean has one meaning, one writer, and no
display coupling.

`users.role` is left completely alone. Deprecating it is separate work and is
explicitly not in scope.

## 3. What changes

**Config.** `admin_min_trust` (added earlier today) is removed or repurposed.
The threshold is no longer a trust level, so keeping the name would be a lie in
the code. The gate reads a flag.

**The 11 gates.** `trust::require_admin_tier` stops reading
`auth.trust_level` and reads the administrator flag instead. Its name stays — it
is the admin tier — but the field it consults changes.

**JWT.** `Claims` (`auth.rs:77-84`) carries `sub`, `username`,
`trust_level`, `level`, `exp`, `iat` — no admin claim. The login query
(`auth.rs:145-168`) selects `trust_level` into `AuthUser`; adding `is_admin`
means selecting it and adding both a `User` field, an `AuthUser` field, and a
`Claims` field.

Reading the column per request instead would mean a query on every admin call
and no revocation: a demoted admin's existing 30-day token would keep working
either way, since the claim is baked in at issue. The claim is chosen for
consistency with how `trust_level` already works, not because it is more secure
— and the 30-day non-revocable window is a pre-existing property of the token
scheme, not something this change introduces.

**Tests.** `auth_header(user_id, 10, ...)` currently mints a token the product
cannot issue. Those call sites must be rewritten to set the flag, and the
acceptance test from the previous spec — log in through
`POST /api/auth/login`, then call an admin endpoint with the returned token —
becomes the primary test, because its absence is what let the contradiction
survive.

## 4. Requirements

**R1.** An administrator is a user with the flag set, and no trust level grants
admin. A TL6 user with the flag off cannot reach an admin-tier route.

**R1a.** The column is `users.is_admin BOOLEAN NOT NULL DEFAULT false`, added by
a new numbered migration. `users.role` is untouched.

**R2.** A user with `trust_level = 0` and the flag on **can** reach admin-tier
routes. This is the case the current code cannot express, and the one that
proves admin is not a ladder position.

**R3.** `013_trust_levels.sql` is not edited. If a column is added, it is a new
numbered migration, after 092.

**R4.** The acceptance test logs in via `POST /api/auth/login` and reaches an
admin endpoint with the returned token, expecting 200. It fails before the fix.

**R5.** The same test with the flag off expects 403. Already covered by
`admin_api` and must stay.

**R6.** `admin_min_trust` is removed from `Config`, or the gate stops using it.
No dead config field.

**R7.** The flag is reachable by a real administrative action. Hand-editing the
column is not a promotion path, for the reason given in option A.

## 5. Non-goals

- The other 28 admin routes still at TL5 (from `admin-tier-separation.md` §6).
  This change does not decide them.
- Whether `users.role` keeps its other consumer in `subsystems.rs:526`.
- The F7 level gate, which the owner decided separately and which is
  orthogonal: that is a *level* check alongside trust, not a replacement for the
  admin flag.

## 6. Acceptance

1. R1-R4 proven by named tests, red before the fix.
2. `admin_api` stays 12/13 — the remaining failure is the stale
   `admin_users_search_role_ban` fixture, documented in
   `admin-tier-separation.md` §3.
3. 935 unit tests pass; all 57 suites compile.
