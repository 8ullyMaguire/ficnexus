# FicNexus — specification: `reports_api` — a leaked-rows assertion and a trust-gate fixture

Status: active. Written 2026-09-26, before any code change.
Plan: `docs/PLAN-reports-suite.md`.

## 1. Problem

`tests/reports_api.rs` has 3 failures, from two unrelated causes. Neither is a
production defect; both are test defects, and the first is a lesson about count
assertions.

## 2. Cause 1 — a count assertion that can only pass on an empty database

`admin_lists_and_resolves_reports` (`tests/reports_api.rs:433`):

```rust
let open_id      = seed_report(&db, user_id, "work", work_id, "Wrong author", "open").await;
let dismissed_id = seed_report(&db, user_id, "work", work_id, "Spam", "dismissed").await;

let (status, body) = get_json(&app, "/api/admin/reports", Some(&admin_token)).await;
let items = body["items"].as_array().expect("items array");
assert_eq!(items.len(), 1, "only open reports, got: {body}");
```

The endpoint is **correct**. `list_reports` filters on the requested status
(`src/routes/reports.rs:319`, `status == "all"` selects everything, otherwise it
filters), and an administrator listing every open report in the system is the
intended behaviour. The test is asserting that the database contains exactly one
open report, which is a property of an empty database, not of the endpoint.

Actual observed count: **16**, with `reporter_id: null` on most of them. Those
are rows leaked by `forum_api`, whose `f5_report_forum_target` files reports and
whose cleanup deletes `forum_categories` and `users` but never the
`user_reports` rows it created.

Two independent problems, and fixing only the assertion would hide the first:

### 2.1 The assertion is a count, not an identity

`assert_eq!(items.len(), 1)` is satisfied by *any* single open report. Fixing it
to "find my report in the list" makes the test correct regardless of what else
is in the table, and it is the assertion that actually expresses the intent:
"the default status filter returns my open report and not my dismissed one".

A count assertion of this shape cannot distinguish "the filter works" from "the
table happens to hold one row".

### 2.2 `forum_api` leaks `user_reports` rows

`f5_report_forum_target` creates reports and its cleanup removes the category
and the users, leaving orphaned `user_reports` rows. Those rows are what
`reports_api` is seeing. They also accumulate on every run — the count grows,
which is why this failure is not deterministic across a fresh provision and a
used database.

**Both are fixed.** The assertion stops depending on a global count, and the
leak is cleaned at source, so the table does not grow without bound.

### 2.3 The missing FK is not fixed here

`user_reports.reporter_id` is nullable, the leaked rows show `reporter_id:
null`, and `information_schema` reports **no foreign key on the column at all**
(checked `pg_constraint` directly, not just the column list). Deleting the user
sets it NULL rather than cascading.

So cleanup cannot be written as "delete the reporter's reports" — the predicate
matches nothing once the user is gone. The working cleanup keys on `target_id`,
which the test still has in scope, and must run **before** the users are deleted.

Whether `reporter_id` should be `NOT NULL` is a product question — reports about
deleted users may be a legitimate state — and is out of scope. What matters is
that the rows are not left behind by a test.

## 3. Cause 2 — the trust-gate fixture, for the third time

`logged_in_user_files_work_report` and `report_creation_validates_input` both get
**403** where they expect 200:

```
left: 403
right: 200
```

`create_report` gates at `src/routes/reports.rs:96`:

```rust
trust::assert_staff_or_min_trust(&state.db, Some(reporter_id), auth.trust_level, 1, "Filing reports").await?
```

`assert_min_trust` reads `users.trust_level` **from the database**
(`src/services/trust.rs:102-108`), so the JWT's trust level does not move it.

The test's seeding helper writes a different column:

```rust
async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    "INSERT INTO users (username, password_hash, role) VALUES ($1, 'test-hash', $2) ..."
}
```

and the token is minted with trust 0 (`auth_header(user_id, 0, USERNAME)`).

Confirmed against the live database:

```
 username            | role | trust_level
 reports_test_user   |   0  |     0
```

`role` is 0 and `trust_level` is 0, and the gate needs `trust_level >= 1`. The
gate is correct — a brand-new account should not be able to file reports, since
flags carry trust-weighted value and that is the whole point of the sandbox. The
fixture is wrong.

**This is the same defect as `uploads_api` (class A) and
`f5_report_forum_target` in `forum_api`.** Three suites, one cause: a test seeds
`users.role` or relies on a hand-prepared database, while the gate reads
`users.trust_level`.

`role` is a real, separate legacy column with a real consumer
(`src/routes/subsystems.rs:526` sets it to 1 on subsystem approval). It is not
dead and is not what any of these gates read.

## 4. Requirements

**R1.** `admin_lists_and_resolves_reports` must locate **its own** report in the
response by id, not assert a global count. The dismissed report must still be
absent, and that absence is asserted against the same identified report rather
than against a length.

**R2.** The test must pass whether the table holds 1 row or 16. Proven by running
it against a database that already has leaked rows.

**R3.** `forum_api` cleans up the `user_reports` rows it creates, keyed on
`target_id` and run before the users are deleted, so repeated runs do not grow
the table without bound. Proven by counting rows before and after two
consecutive runs, not by reading the SQL.

**R4.** The report-filing tests seed `users.trust_level` at 1 or above. The
database row is what the gate reads.

**R5.** `seed_user`'s parameter is renamed to say which column it writes, for the
same reason as `admin_api`'s `seed_admin_user`. This is the third suite with the
same misleading helper shape.

**R6.** No gate is lowered. `FORUM_MIN_TRUST`-style configuration does not apply
here and the minimum stays 1.

## 5. Non-goals

- Making `user_reports.reporter_id` `NOT NULL`. Whether a report about a deleted
  user is legitimate is a product question (2.3).
- Adding a `DELETE ... ON DELETE CASCADE` from `users` to `user_reports` — that
  would delete moderation history when a user is deleted, which is almost
  certainly wrong for an audit trail.
- The admin list endpoint. It is correct.

**R7.** The new identity assertions are proven to have teeth by mutating the
production code they cover, not by rewriting them. Neutering
`resolve_report`'s `UPDATE ... SET status = $1` to `SET status = status` — still
returning 200 and the requested status — must make the suite fail.

## 6. Acceptance

1. All four `reports_api` tests pass against the current, already-polluted
   database — not only against a freshly provisioned one. That is the difference
   between R2 being satisfied and merely appearing to be.
2. `SELECT count(*) FROM user_reports` stops growing across two consecutive runs
   of `forum_api`.
3. The resolve assertion fails when resolve is neutered.
4. 935 unit tests pass; all 57 suites compile.

## 7. Outcome

All met, with one addition to R2's scope. There were **three** count
assertions, not one: `len() == 1` for the open list, `len() == 2` for
`status=all`, and `len() == 0` after the resolve. Each is satisfied by any table
state with the right number of rows. The resolve one is the dangerous case — "no
open left" passes with resolve completely broken, as long as some other report is
there instead.

The leak fix took three attempts, all of which looked correct:

| attempt | why it deleted nothing |
|---|---|
| `WHERE reporter_id = $1` | column is nullable with no FK; deleting the user nulls it, so the predicate can never match afterwards |
| `WHERE target_id = ANY($1)` as `Vec<String>` | `target_id` is `integer` |
| `WHERE target_id = ANY($1)` as `Vec<i32>` | works — count holds at 23 |

`.ok()` made "matched zero rows" silent in all three. Verified by row count
before and after, twice, rather than by reading the SQL.

The mutation pass has one trap worth recording: the first attempt replaced the
assertion with `filter(..).count() == 0`, which is **semantically identical**,
and came back green under mutation. It proved nothing because it mutated no
behaviour. Mutating the production `UPDATE` is what produced the real red.
