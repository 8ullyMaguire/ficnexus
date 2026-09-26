# FicNexus — implementation plan

Companion to `docs/specs/test-build-repair.md`. That document says what is
true; this one says what to do. Written 2026-09-26, before any code change.

## 0. Conventions for this repository

- Build target dir is **not** the default here. Always pass
  `CARGO_TARGET_DIR=~/.cargo-target/ficnexus`; the repo is on an sshfs mount
  where a local target build is far faster.
- `cargo fmt` and `cargo clippy` are the style gate. Clippy currently reports
  warnings on the existing tree; do not fix unrelated ones in this change, and
  do not introduce new ones.
- Integration suites in `tests/` are `#[ignore]`d and DB-gated. They are out of
  scope for steps 1-3. See step 7.

## 1. Fix the test build

### 1.1 Inventory before touching anything

```bash
cd ~/code-local/rust/ficnexus
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib --no-run 2>&1 \
  | grep -E '^error' | sort | uniq -c | sort -rn
```

Expected: 14 errors — 9 `AuthUser has no field named role`, 1 each for
`User`, `Claims` (x2: literal + field access), `Config.forum_curator_level`,
`Config.forum_admin_level`.

```bash
grep -rn 'role: [0-9]\|forum_curator_level\|forum_admin_level' src/
```

Expected sites: `src/modlog.rs:141,152`; `src/routes/authors.rs:735,743,751,763,771`;
`src/routes/social.rs:1087,1107`; `src/routes/download.rs:888`;
`src/config.rs:556,557`.

### 1.2 Fix `Config` first — it is unambiguous

`src/config.rs:556-557`, inside a test-only `Config` literal, set two fields that
no longer exist. Both gates moved to the auth struct, so they are simply
deleted:

```rust
// delete both lines
forum_curator_level: 0,
forum_admin_level: 0,
```

No replacement. If the test needed a specific level, it should set
`trust_level`/`level` on the `AuthUser` it builds, not on `Config`.

**Verify:** `cargo test --lib --no-run 2>&1 | grep -c '^error'` drops by 2.

### 1.3 Fix `AuthUser` fixtures by intent, not by number

This is the substantive step. For **each** of the 9 sites, read the assertions
in the surrounding test and decide which of the two fields it means.

Decision rule:

- The test exercises a **curator** action, or asserts a user is refused/allowed
  curator access → `trust_level`. A reader is 0, a curator is 5
  (`src/routes/authors.rs:551` uses `trust_level >= 5` for auto-approve).
- The test exercises a **forum mod/admin** gate, or anything ranked on the
  0-100 progression axis → `level`.
- The test does not care about privilege at all → `trust_level: 0` is the
  neutral choice; the point is that it compiles and the test still means what
  it meant.

Old values `0`, `5`, `10` do **not** map across. Old `role: 5` meant "trusted
moderator" in a world where role was one axis; splitting it into
`trust_level: 5` is only right if the test is about trust. Read the assertions.

Worked example — `src/routes/authors.rs:735-771` is a curator-check test whose
assertions are written in terms of `trust_level` already
(`assert!(user.trust_level < 3, "Reader should not pass curator check")` at
`:738`, `assert!(curator.trust_level >= 5, ...)` at `:746`). Its fixtures
(`role: 0`, `role: 5`, `role: 10`) therefore become `trust_level:` with the same
numbers, and the `role: 10` values become `trust_level: 10` — which is out of
the 0-6 range, so those two sites need the value the assertions actually want
(5) rather than the number that was typed.

That last point is the trap: **when a fixture's old number is out of range for
its new field, the assertions win.** Fix the fixture to satisfy the assertion,
not to preserve the literal.

### 1.4 Fix `User` and `Claims`

- `User.role` → `User.trust_level`, same intent rule as `AuthUser`.
- `Claims.role` → `Claims.trust_level` in both the struct literal and the field
  access site. `Claims` is the JWT payload, so the value must match what
  `auth.rs` puts in the token — read the encoder before choosing the number.

**Verify:** the errors are gone.

```bash
cd ~/code-local/rust/ficnexus
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib --no-run 2>&1 \
  | grep -E '^error' | head
```

Expected: no output.

## 2. Run the suite

```bash
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib 2>&1 | tail -30
```

Expected: every test passes. If a test fails on an **assertion** rather than a
compile error, that is a second finding: it means the fixture change altered
meaning. Go back to §1.3 and re-read the intent. Do not "fix" the assertion to
match a fixture you guessed at.

## 3. Prove production behaviour is unchanged

```bash
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo build
git diff --stat
```

Every changed line must be inside a `#[cfg(test)]` module or a test file. Read
the diff and confirm this line by line — it is the definition of done in the
spec, and a production line changed here means the refactor was partly reverted
by accident.

## 4. Lint gate

```bash
cd ~/code-local/rust/ficnexus
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo fmt --all -- --check
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo clippy --lib --all-targets 2>&1 \
  | grep -E '^(warning|error)' | wc -l
```

Compare the clippy count against the pre-change baseline. It must not increase.
Record both numbers in the commit message.

## 5. Commit and tag

```bash
cd ~/code-local/rust/ficnexus
git add -A
git commit -F /tmp/msg     # see the message template below
git tag test-build-repair-2026-09-26
```

Commit message must state: the 14 errors, that production code is unchanged, the
before/after clippy counts, and the test count that now runs.

## 6. Document

Add `docs/specs/test-build-repair.md` (already written) and a short
`docs/sessions/2026-09-26-test-build-repair.md` recording what was broken, why
`cargo build` did not catch it, and the numbers after. Then note in
`docs/README.md` that the unit suite is runnable, so the next person does not
have to rediscover this.

## 7. What comes after — do not skip to this

Step 1-6 is a prerequisite. The repository is 235k lines with 56 DB-gated
integration suites that have not been run against a live PostgreSQL in this
audit. Once the unit suite is green:

1. Provision a scratch database, apply all 38 migrations.
2. Run the DB-gated suites with the documented invocation:
   `cargo test --test <name> -- --include-ignored --test-threads=1`
3. Record real results. Do not assume they pass because the code builds.

Treat this as its own spec-and-plan cycle. It is a large, genuinely unknown
piece of work, and the point of step 1 is that "it compiles" turned out not to
mean "it works".

**Do not use `docs/sessions/SESSION-2026-09-09.md` as a backlog.** Its
"optional remaining work" list points at `docs/missing.md` and
`docs/xp-removal-todo.md`, neither of which exists on disk, and its item
"remove `level` from `AuthUser`" is now wrong — `level` is the live F7 forum
gate. That summary is a record of a past session, not a statement of intent.

## 8. Definition of done

- [ ] `cargo build` succeeds
- [ ] `cargo test --lib --no-run` compiles, zero errors
- [ ] `cargo test --lib` passes
- [ ] `git diff --stat` shows test-only changes
- [ ] `cargo fmt --check` clean
- [ ] clippy count not increased
- [ ] committed, tagged, session note written
- [ ] spec and plan committed alongside
