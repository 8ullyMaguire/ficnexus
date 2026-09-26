# FicNexus — specification: F7 authorization reads level alongside trust

Status: **approved by the owner 2026-09-26.** Companion analysis in
`docs/specs/f7-gate-assignments.md` is retained.
Plan: `docs/PLAN-f7-gates.md`.

## 1. Decision

F7 authorization checks **`auth.level` alongside `auth.trust_level`**. Both must
pass. Neither replaces the other.

Rejected:

- **Trust only** — leaves the F7 level system with no authorization meaning at
  all, which makes the whole leveling feature decorative.
- **Level replaces trust** — a level-100 user would become an admin regardless of
  trust, inverting the two systems the owner wants kept distinct (and colliding
  with `admin-flag.md`, which puts administrator on a flag entirely).

## 2. The threshold comes from Config, defaulting to today's behaviour

`Config` already carries the other F7 fields and the tests were written against
those defaults. The default must be the value that makes the existing tests
green, so deploying changes nobody's access.

**Read off the tests, not off the prose.** Measured:

| gate | test | trust | level | expectation |
|---|---|---|---|---|
| curator — `POST /posts/{id}/moderate` | `f7_level_gate_curator` | 5 | 50 | **200** |
| curator — same route | `f7_level_gate_curator` | 5 | 49 | **403** |
| admin — `POST /forum/categories` | `f7_level_gate_admin` | 10 | 99 | **403** |
| admin — same route | `f7_level_gate_admin` | — | 100 | **200** |

**This is two thresholds, not one.** The first draft of this spec proposed a
single `forum_admin_min_level`; the tests do not support that. Curator admits at
**50** and refuses at **49**; admin refuses at **99** and admits at **100**. One
field cannot express both — set it to 50 and the admin gate admits a level-50
curator; set it to 100 and moderators lose the ability to moderate, which is a
regression on a capability the product has today.

So there are **two** config fields:

- `forum_curator_min_level`, default **50**
- `forum_admin_min_level`, default **100**

The curator test also seeds `created_at = NOW() - INTERVAL '30 days'`, so it
implies an **account-age** component as well. The comment "14+ days old" says the
intent. That is a third field (`forum_curator_min_account_days`) and is **not**
proven by the test as written: the account is 30 days old in the passing case, but
no test asserts that a *young* level-50 curator is refused, so the age term's
threshold is unconstrained by the suite. Defaulting it to 14 preserves the
stated intent, and the missing test is recorded below rather than invented.

**The defaults must be the values that make the existing tests green.** A
threshold chosen to satisfy a test is a hardcoded literal with extra steps; these
are the values the tests already encode, and they are stated here so a later
edit that changes who is an admin is a visible diff.

## 3a. The account-age term is unproven — recorded, not invented

`f7_level_gate_curator` seeds `created_at = NOW() - INTERVAL '30 days'` and its
comment says "14+ days old", but **no assertion distinguishes a young
level-50 curator from an old one.** A test that would: seed level 50 with
`created_at = NOW()`, expect 403. It does not exist, so the age threshold is
chosen from the comment alone.

Adding that test is in scope — it is the test for a term this change introduces,
and introducing a term without it is how the term rots. It goes in as a new case
in `f7_level_gate_curator`.

## 3. Scope: two gates, not four call sites

Three separate `require_admin` functions exist, in three files, and the F7 tests
do not all target the same one:

| file | line | gates |
|---|---|---|
| `src/routes/forum.rs` | 209 | forum actions |
| `src/routes/forum_privileges.rs` | 66 | forum privileges |
| `src/routes/backfill.rs` | 31 | backfill (not forum; **out of scope**) |

Within `forum.rs` there are two *distinct* surfaces: the curator/moderation path
(`POST /posts/{id}/moderate`, threshold 50) and the admin path
(`POST /forum/categories`, threshold 100). Adding one check to `require_admin`
would not satisfy either test.

Each gates on `trust_level` and never reads `auth.level` or account age.

**R1.** The level check is added *alongside* the trust check, not in place of it.
A user failing either is refused.

**R2.** Two thresholds: `Config::forum_curator_min_level` (default 50) and
`Config::forum_admin_min_level` (default 100), each read from
`FORUM_CURATOR_MIN_LEVEL` / `FORUM_ADMIN_MIN_LEVEL`, each documented with its
default in the doc comment, matching the convention the other F7 fields use.

**R2a.** `Config::forum_curator_min_account_days`, default 14, per the test's own
comment ("14+ days old"). Recorded as unproven by the suite in §3a.

**R3.** A unit test asserts the default value, so a later edit to the default
that silently changes who is an admin is caught.

**R4.** The two F7 gate tests go green. They are currently red on purpose and
were marked blocked; this change unblocks them.

**R5.** The curator gate is a separate check from the admin gate, with its own
error message. A 403 must say which threshold refused, because the two will be
tuned independently and a combined condition makes that unanswerable.

**R6.** `auth_header_level` at `tests/forum_api.rs:299` is the only helper that
can mint a level claim; `auth_header` (line 294) wraps it with a level argument
too, so both must keep working after the change.

## 4. Non-goals

- The other 28 admin routes at TL5, and the 11 now on the admin flag
  (`admin-flag.md`). F7 gates are a separate surface from admin-tier routes.
- Whether `auth.level` is reachable on the JWT for a real login — the same
  ceiling problem as `trust_level` applies, since `level` is also minted from a
  database column. That is **not** resolved here and is recorded as a known
  open question below.

## 4a. Deliberately not the admin-tier routes

The 11 routes moved to `Config::admin_min_trust` and then to the `is_admin` flag
(`admin-flag.md`) are a **different surface**. The forum category route in scope
here is gated on forum level and trust; it is not an admin-tier route and must not
become one, or moderators who can moderate today would lose category creation on
deploy.

`Config::admin_min_trust` is being repurposed by `admin-flag.md` — the same field
name will briefly mean "forum level for admin forum actions" while its old
meaning is being removed. Land them in that order and check every reader of the
field in between.

## 5. Verified: unlike the trust level, this one is reachable

The same question was asked of `level` as of `trust_level`, before implementing,
because the TL10 contradiction came from exactly this shape. Checked against the
live database:

```
level       smallint  NOT NULL   (no CHECK constraint)
exp         bigint    NOT NULL
trust_level smallint  NOT NULL   CHECK (trust_level >= 0 AND trust_level <= 6)
```

`users` has exactly two CHECK constraints — `curator_status` and
`users_trust_level_check`. **There is no bound on `level`.**

And a real account already sits at the top of it:

```
SELECT max(level) FROM users   ->   100
```

So a level-100 claim is reachable by a real login, which is the single thing
`trust_level` could not offer. This gate is implementable as specified; the
trust-level gate was not, and that is why `admin-flag.md` exists.

Recording it here because the next person will otherwise assume the two are
symmetrical. They are not: `trust_level` is a bounded 0-6 ladder, `level` is an
unbounded progression.
