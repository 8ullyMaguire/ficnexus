# FicNexus — implementation plan: `reports_api` suite

Companion to `docs/specs/reports-suite.md`. Executable from this text alone.

Test-file changes only. No production code, no migration.

## Step 1 — replace the count assertion with an identity assertion (R1)

**File:** `tests/reports_api.rs`, around line 433.

Current:

```rust
let items = body["items"].as_array().expect("items array");
assert_eq!(items.len(), 1, "only open reports, got: {body}");
let item = &items[0];
assert_eq!(item["id"].as_i64(), Some(open_id));
```

Replace with a search for this test's own report:

```rust
// Find OUR report by id. A count assertion here would be asserting that the
// table holds exactly one open row, which is a property of an empty database,
// not of the endpoint - the admin list is supposed to return every open report
// in the system. See docs/specs/reports-suite.md section 2.
let items = body["items"].as_array().expect("items array");
let item = items
    .iter()
    .find(|i| i["id"].as_i64() == Some(open_id))
    .unwrap_or_else(|| panic!("open report {open_id} missing from list: {body}"));
```

Keep every field assertion that follows — they are the real content of the test
and they are correct. Keep the `dismissed_id` absence check, but assert it the
same way: find `dismissed_id` in the items and assert it is **not** there, rather
than relying on `items.len()`:

```rust
assert!(
    !items.iter().any(|i| i["id"].as_i64() == Some(dismissed_id)),
    "dismissed report must not appear under status=open: {body}"
);
```

**R2 proof.** Run this test twice, and also run it after `forum_api` has leaked
rows. It must pass in all three cases. A test that only passes on a fresh
database has not been fixed.

## Step 2 — stop `forum_api` leaking `user_reports` rows (R3)

**File:** `tests/forum_api.rs`, in `f5_report_forum_target`'s cleanup block.

That test files real reports and its cleanup deletes the category and the users
but not the reports, so every run leaves rows behind. Add to the existing
cleanup, before the category delete:

```rust
// The reports this test filed reference the post/topic it is deleting, and the
// reporter user. Without this they accumulate on every run, and
// reports_api's admin-list test sees them. See docs/specs/reports-suite.md 2.2.
sqlx::query("DELETE FROM user_reports WHERE reporter_id = $1")
    .bind(reporter_id)
    .execute(&db)
    .await
    .ok();
```

`reporter_id` is the seeded reporter's id. Confirm the variable name in the test
before using it — it is seeded as `reporter_id` in the version this plan was
written against.

**Verify the leak is actually stopped**, do not assume it:

```bash
psql "$DATABASE_URL" -tAc "SELECT count(*) FROM user_reports"   # before
cargo test --test forum_api f5_report_forum_target -- --include-ignored --test-threads=1
cargo test --test forum_api f5_report_forum_target -- --include-ignored --test-threads=1
psql "$DATABASE_URL" -tAc "SELECT count(*) FROM user_reports"   # after: must be unchanged
```

## Step 3 — seed the trust level the gate reads (R4, R5)

**File:** `tests/reports_api.rs`.

`create_report` gates on `users.trust_level` from the **database**
(`src/services/trust.rs:102-108`), so the JWT value does not matter. Rename the
helper and seed the right column:

```rust
/// Seeds a user at an explicit database `trust_level`.
///
/// `create_report` calls `assert_staff_or_min_trust(..., 1, "Filing reports")`,
/// which reads `users.trust_level` **from the database** — the JWT's
/// trust_level does not move it. The old helper wrote the separate legacy
/// `users.role` column, which no trust gate reads, so every report attempt
/// 403'd. Same defect as uploads_api and admin_api; see
/// `docs/specs/reports-suite.md` section 3.
async fn seed_user_at_trust(pool: &sqlx::PgPool, username: &str, trust_level: i16) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash, trust_level) VALUES ($1, 'test-hash', $2)
         ON CONFLICT (username) DO UPDATE SET trust_level = EXCLUDED.trust_level
         RETURNING id",
    )
    .bind(username)
    .bind(trust_level)
    .fetch_one(pool)
    .await
    .expect("seed_user_at_trust failed")
    .get(0)
}
```

Then replace the call sites that file reports:

- `logged_in_user_files_work_report` — seed at **2**
- `report_creation_validates_input` — seed at **2**

Leave the admin user seeding as-is if it only needs to read the list; the admin
list path checks `auth.trust_level >= 5` from the JWT
(`src/routes/reports.rs:303`), which the test already mints at 10.

**Do not lower the gate.** The minimum stays 1: a brand-new account must not be
able to file reports, because flags carry trust-weighted value. R6.

## Step 4 — gate

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='fichub-test-secret'
cargo build                                              # clean
cargo test --lib                                         # 935 passed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'   # 0
cargo test --test reports_api -- --include-ignored --test-threads=1
```

Expected: 3 passed, 0 failed.

## Step 5 — commit

One commit, since all three changes are the same class of fix:

    test(reports): find our own report instead of counting; seed trust_level

Tag `reports-suite-2026-09-26`, sync to `~/code/rust/ficnexus`, and commit the
session note to `~/secondbrain`.

## Rollback

Test files only. No production behaviour changes.
