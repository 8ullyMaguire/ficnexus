# FicNexus — specification for the DB-gated integration suites

Status: active. Written 2026-09-26, before any code change.
Companion to `docs/specs/test-build-repair.md`, which fixed the unit suite.
Repo: `~/code-local/rust/ficnexus`.

## 1. The problem, and why it is the same problem

The unit suite was repaired on 2026-09-26 (935 pass, 0 fail). The 56
integration suites under `tests/` were not part of that work, because they are
`#[ignore]`d and DB-gated and therefore did not compile in `--lib`.

They do not compile either.

```
cargo test --no-run
  10 of 56 suites fail to compile
  22 errors, in 3 classes
```

This is the same `trust_level` refactor, one layer out. That refactor changed
`AuthUser`, `User`, `Claims` and `Config`, and grew `AppState`. It updated the
unit tests inside `src/`. It did not update the integration suites in `tests/`,
which construct the same types directly.

The failure mode is identical and worth naming precisely, because it is the
lesson of both this spec and its predecessor:

- `cargo build` does not compile `#[cfg(test)]` **or** `tests/`, so both the
  binary and the unit suite looked fine.
- The suites are `#[ignore]`d, so nothing ran them by default.
- Therefore a 235k-line repository shipped with **zero runnable tests of any
  kind** — 935 unit tests that could not compile, and 56 suites that were never
  reachable.

## 2. The three error classes, exactly

### 2.1 `E0063` — missing `AppState` fields (10 sites)

```
error[E0063]: missing fields `jwt_secret`, `redis_client` and `rt_manager`
   --> tests/ask_archive_api.rs:87:26
```

Sites: `ask_archive_api.rs:87`, `curator_api.rs:73`, `feedback_api.rs:65`,
`heal_api.rs:75`, `honeypot_api.rs:61`, `recommender_personal.rs:86`,
`tags_api.rs:79`, `translation_review_api.rs:66`, `uploads_api.rs:104`,
`user_export_api.rs:88`.

`AppState` (`src/server.rs:29`) has 19 fields. These 10 suites construct it
literally and predate the three newest. **46 other suites already do this
correctly** — `tests/forum_api.rs:68` is the reference shape, and it also shows
what the three new fields need:

- `jwt_secret: String` — any test value; the suites that exercise auth use a
  fixed literal, so use the same one across suites for consistency.
- `redis_client: Option<redis::Client>` — `Some(client.clone())` where a client
  is already in scope, else `None`.
- `rt_manager: ConnectionManager::new()` — the realtime hub. Check the
  constructor's signature in `src/realtime/mod.rs` before calling it; if it is
  not `new()`, use whatever the working suites use.

### 2.2 `E0560` — `User`/`AuthUser` `role` field (12 sites)

```
error[E0560]: struct `User` has no field named `role`
   --> tests/curator_api.rs:164:9
```

Sites: `curator_api.rs:164,183`, `feedback_api.rs:147`, `heal_api.rs:141`,
`honeypot_api.rs:362,452,545`, `recommender_personal.rs:146`, `tags_api.rs:161`,
`translation_review_api.rs:158`, `uploads_api.rs:160`, `user_export_api.rs:148`.

Identical to the unit-suite repair, and identical in the trap: `role` was
overloaded, so `role: N` maps to `trust_level` or `level` **by the intent of
the surrounding test**, not by the number. Most of these suites create a user
purely to own a post or a comment, in which case the privilege is incidental and
`trust_level: 0` is correct.

Where a test genuinely needs elevated privilege, read the production gate it is
exercising before choosing a value. A user with `trust_level: 5` passes curator
checks; `level >= 50` is the forum mod/admin axis.

### 2.3 `E0425` — removed handler `fichub::routes::forum::my_level`

`tests/forum_api.rs:233` routes `/api/users/me/level` to
`fichub::routes::forum::my_level`, which no longer exists. This did not appear
in the 10-suite count because `forum_api.rs` stops at the earlier errors.

**Resolve by finding out what replaced it, not by guessing.** Search
`src/routes/forum.rs` and `src/server.rs` for the current route. If the endpoint
was genuinely removed, delete the route registration and any test that exercises
it, and say so in the commit. If it moved, point the test at the new location.
Do not reintroduce a deleted endpoint to make a test compile.

## 3. Required outcome

1. All 56 suites compile.
2. Every suite that can run against the provisioned database does, and its real
   result is recorded — pass or fail. **A failing assertion is a finding to
   report, not something to paper over.** See §5.
3. No production code changes. This is test-code repair.
4. The result is reproducible from a clean machine by one script.

## 4. Environment prerequisites (discovered, non-obvious)

These are host facts, verified on 2026-09-26, and each one blocked the run:

- **pgvector is required and was not installed.** `001_initial.sql` does
  `CREATE EXTENSION vector`. Installed from `cachyos-extra` (`pgvector 0.8.6`).
- **`CREATE EXTENSION` needs superuser.** Running the migrations as the app role
  fails with `permission denied to create extension vector`. Extensions must be
  created first, as an admin, and the migration then skips them.
- **The `fichub` role must exist before any migration runs.** Migrations
  072-089 do `ALTER TABLE ... OWNER TO fichub`. This is the deployment shape
  from `.env.example`, not a test convenience.
- **`psql -d <dbname>` ignores a URL** and connects over the unix socket as the
  OS user. Every invocation must use an explicit `postgres://` URL.
- **Redis must be up.** Every suite builds a real `AppState` with a live
  multiplexed connection.

`scripts/provision_test_db.sh` (written with this spec) encodes all of it. It is
idempotent, and prints the `DATABASE_URL` to use.

Verified on 2026-09-26: all 38 migrations apply, **180 tables** created.

## 5. What "passing" means here, honestly

The suites were written against a schema and a set of handlers as they were when
each suite was added. Some may now be testing behaviour that has since changed
deliberately — exactly the situation that produced
`docs/sessions/SESSION-2026-09-09.md`'s stale backlog.

So the deliverable of this work is **a real measurement**, not a green run:

- A suite that passes: fixed, and now providing coverage.
- A suite that fails on an assertion: the failure is a finding. Report it
  precisely — suite, test, expected, actual — and determine whether the test is
  stale (product changed on purpose) or the code is wrong (regression). Do not
  edit the assertion to match current behaviour without deciding which it is.
- A suite that cannot be made to run (needs a service that does not exist here,
  e.g. Ollama or SMTP): say so, and record what it would take. `OllamaClient` is
  constructed with a URL in tests but is not called, so most suites should not
  need a live Ollama. Verify rather than assume.

The output of this work is a number: **N of 56 suites pass, M fail, K cannot
run**, with reasons.

## 6. Out of scope

- Any production code change, including re-adding a deleted handler.
- The `cargo fmt` backlog (547 pre-existing diffs).
- The `cargo clippy` backlog (599 pre-existing findings).
- Feature work of any kind.

## 7. Verification

```bash
cd ~/code-local/rust/ficnexus
bash scripts/provision_test_db.sh            # prints DATABASE_URL
export DATABASE_URL=... REDIS_URL=redis://127.0.0.1:6379 JWT_SECRET=test-secret
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --no-run   # 0 errors
for t in tests/*.rs; do
  n=$(basename "$t" .rs)
  CARGO_TARGET_DIR=~/.cargo-target/ficnexus     cargo test --test "$n" -- --include-ignored --test-threads=1 2>&1 | tail -3
done
```

`--test-threads=1` is mandatory, not a precaution: the suites serialise on their
own `Mutex`, delete their own seed rows by unique prefix, and assume they are
alone in the database. Running them in parallel corrupts each other's fixtures.

Definition of done: 0 compile errors, and a recorded pass/fail/unrun count for
all 56, with reasons for anything that is not a pass.
