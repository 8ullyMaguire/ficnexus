# Admin authorization: an invariant test, not middleware

Date: 2026-09-27 · Status: implemented (`tests/admin_route_invariants.rs`)
Context: `docs/PLAN-admin-gates-nonsimple.md`, `docs/specs/admin-tier-separation.md`

## The decision

I recommended middleware on the `/api/admin` router last turn and then **did not
build it**, because measuring the change first showed it is a restructuring of
77 route registrations across 17 chunks, not a middleware insertion. `src/server.rs`
builds the router as:

    let merged = chunk_core_api()
        .merge(chunk_device_download())
        … 17 chunks …
        .merge(chunk_realtime());

The admin routes are **spread across those chunks**, not collected in one, so
there is no `/api/admin` sub-router to attach a layer to. Making one means
relocating 77 `.route()` calls out of unrelated chunks into a new
`chunk_admin()`, touching every line number other code and tests reference.

That is a real refactor with real regression risk, and it fixes a problem that is
**currently at zero**: I checked, and **all 77 admin routes are gated**.

The live risk is not "a gate is wrong" — that is what the last two commits fixed.
The live risk is **the next person adds an admin route and writes no gate**. That
is a recurrence risk, and it is closed by a test rather than a refactor.

So: invariant test now, middleware as a separate deliberate piece of work.

## What was actually found while checking

All 77 `/api/admin` routes are gated — but **not by one mechanism**. There are
**ten** distinct gate helpers plus two inline forms:

| shape | enforces | example |
|---|---|---|
| `require_admin_tier` | `is_admin` flag | `admin.rs` (38 sites) — canonical |
| `require_admin` | `auth.level < 10` | `backfill.rs` |
| `require_admin` | `trust_level >= 5` | `forum_privileges.rs` |
| `require_mod` | trust >= 3, moderator | `subsystems.rs`, `forum.rs` |
| `require_curator` | trust >= 5 | `curator_content.rs`, `tags/curator.rs` |
| `require_forum_category_admin` | → `require_admin_at_level` | `forum.rs` |
| `require_forum_moderator` | → `require_admin_at_level` | `forum.rs` |
| `require_trust_resolve` | `assert_staff_or_min_trust` | `forum.rs` |
| `require_trust_queue` | `assert_staff_or_min_trust` | `forum.rs` |
| inline `COALESCE(trust, 0)` SQL | `users.trust` >= 4 | `features.rs` |

Two things worth recording from that table:

**`require_admin` means two different things.** In `backfill.rs` it checks
`auth.level < 10`; in `forum_privileges.rs` it checks `trust_level >= 5` — and
that one is written `if auth.trust_level >= 5 || auth.trust_level >= 5`, a
duplicated operand that is dead weight. Same name, same file tree, different
axis. Not touched here; it is a rename-and-clarify, not a defect.

**`features.rs` gates on a fourth trust column.** `SELECT COALESCE(trust, 0) FROM
users … if trust < 4` — it is the only reader of `users.trust` anywhere in
`src/`. So the table now demonstrably carries `role`, `trust_level`, `trust` and
`is_admin`, and admin authorization reads three of them. That is the same
ambiguity `docs/specs/admin-tier-separation.md` started from, still standing.

## The test

`tests/admin_route_invariants.rs`, two tests, no new dependencies:

1. **`every_admin_route_is_authorization_gated`** — parses `src/server.rs` for
   `/api/admin` route registrations, resolves each handler symbol to its file,
   extracts the body by brace matching, and requires a gate.
2. **`admin_gate_helpers_are_known`** — asserts the parse found a plausible
   number of routes and that `require_admin_tier` still exists in
   `src/services/trust.rs`, so a rename surfaces here.

**Both are mutation-checked**, which is the part that matters — a structural test
that cannot fail is worse than none, because it looks like coverage:

| mutation | result |
|---|---|
| baseline | 2 passed |
| gate deleted from `mod_queue` | FAILED, naming `/api/admin/moderation/queue` |
| new ungated handler + route added | FAILED, naming `/api/admin/totally-new-ungated` |
| restore | 2 passed |

The second mutation is the actual recurrence scenario, and the failure message
names the route and the fix.

### Guarding against a vacuous pass

A parser bug that matches nothing would make this test pass while checking
zero routes. Three explicit guards against that:

- assert the route count is `>= 70` (it is 77) with a message saying a silently
  empty parse would make the test vacuously pass
- fail on **unresolvable** symbols rather than skipping them, so a handler that
  moved is a test failure, not silent coverage loss
- `admin_gate_helpers_are_known` independently re-parses and cross-checks

## Three parser bugs found by running it, not by reading it

Each was invisible in review and obvious in execution:

1. **`?` in a function returning `Vec`** — compile error.
2. **`find("fn name(")` matched a call site** — returned a body ending at byte
   1845 for a signature at byte 6233, and panicked on the slice. Now anchors on
   start-of-line whitespace.
3. **Off-by-one on the path literal** — `"/api/admin` starts *at* the opening
   quote, so `find('"')` returns 0. Now searches from index 1.

Plus one design error: the first gate matcher accepted only
`require_*|is_admin|trust_level` and reported the three `features.rs` routes as
**ungated**. They are gated — with a SQL read of `users.trust`. The matcher now
accepts that too, and the test comment says why.

The Python prototype I used to explore the problem matched all 77 routes and all
74 resolvable handlers on the first run. The Rust version took three fixes to
reach parity. Same logic, worse language for text munging.

## What this does not do

- It proves a gate is **present**, not **correct**. A handler calling
  `require_mod` where it should call `require_admin_tier` passes. `admin_api` is
  the behavioural test for correctness.
- It does not cover non-`/api/admin` privileged surfaces: `/api/curator/*`,
  `/api/moderation/*`, or the owner-or-admin bypasses in
  `docs/PLAN-admin-gates-nonsimple.md` groups C–E, which are not admin routes.
- `run_db_suites.sh` enumerates `tests/*.rs` dynamically, so this is picked up as
  suite 58. It needs no database, but the runner provisions one per suite, so it
  costs a provision cycle. Worth splitting the runner into DB-gated and
  non-DB-gated suites at some point — not done here.

## The middleware question, for later

It remains the right end state: one place, applied by construction, no
opportunity to forget. It is a real refactor (77 route moves across 17 chunks)
and should be its own commit with its own review, not a follow-on to a test.
