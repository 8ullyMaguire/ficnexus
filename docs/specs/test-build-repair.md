# FicNexus — specification for restoring the test build

Status: active. Written 2026-09-26, before any code change.
Repo: `~/code-local/rust/ficnexus` (mirror of `~/code/rust/ficnexus`).

## 1. The problem

`cargo build` succeeds. `cargo test --lib` does not compile: 14 errors, all of
them the same shape.

A previous refactor replaced the site-wide `role` column with two separate
concepts:

- `trust_level` (0-6) — the Discourse-style participation axis. This is the
  gate for curator actions (`src/routes/authors.rs:180`, `:529`, `:551`, `:584`,
  `:631`).
- `level` (0-100) — the site-wide progression axis, and the active gate for
  forum moderator and admin.

The refactor updated the struct definitions in `src/auth.rs` and
`src/config.rs`, and every non-test call site. It did not update the **test
constructors**, which still build `AuthUser { role: 0, .. }`,
`User { role: .. }`, and `Claims { role: .. }`.

So the production build is correct and the test build is broken. This is the
worst kind of break: it is invisible to `cargo build`, invisible to the binary
running in production, and it means **no unit test in this repository can
currently run**. The 56 integration suites under `tests/` are `#[ignore]`d and
DB-gated, so they are unaffected — but the ~117 test files include unit tests
that are the only fast feedback available.

Two further errors are separate: `Config` lost `forum_curator_level` and
`forum_admin_level`, which a test-only `Config` literal in `src/config.rs:556`
still sets.

## 2. Required behaviour

After this work:

1. `cargo build` still succeeds, with no change in behaviour.
2. `cargo test --lib --no-run` compiles with zero errors.
3. Every unit test that passed before the refactor passes again, and every test
   whose fixture asserted on `role` now asserts on the field that replaced it,
   with the **same numeric meaning** where one exists.
4. No production code path changes. This is a test-fixture repair, not a
   feature change and not a revert of the refactor.

## 3. Field mapping

The refactor's intent, read from the surviving call sites:

| Old | New | Range | Gates |
|---|---|---|---|
| `AuthUser.role` | `AuthUser.trust_level` | 0-6 | curator actions |
| `AuthUser.role` | `AuthUser.level` | 0-100 | forum mod/admin |
| `User.role` | `User.trust_level` | 0-6 | account trust |
| `Claims.role` | `Claims.trust_level` | 0-6 | JWT claim |
| `Config.forum_curator_level` | removed | — | superseded by `trust_level` |
| `Config.forum_admin_level` | removed | — | superseded by `level` |

**The old `role` was overloaded**, which is why this is not a mechanical rename.
Numeric values do not carry over: old `role: 5` cannot become
`trust_level: 5` unless the test's intent was trust, and cannot become
`level: 5` unless the intent was progression. Each test fixture must be read
and mapped by intent. This is the whole substance of the task.

## 4. Out of scope

- Restoring `role` as a field. The refactor was deliberate; the fixtures are
  what is stale.
- Restoring `Config.forum_curator_level` / `forum_admin_level`. They are
  genuinely removed; the literal is what is stale.
- The DB-gated integration suites under `tests/`. They compile today and are
  unaffected by this change.
- Any feature work. See §6.

## 5. Verification

```bash
cd ~/code-local/rust/ficnexus
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo build
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib --no-run   # must compile
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib            # must pass
```

Definition of done: all three succeed, and `git diff --stat` shows changes only
inside `#[cfg(test)]` blocks and test modules. If any production line changes,
this is out of scope and needs a decision.

## 6. What comes after

This is a prerequisite, not the goal. Once the unit suite runs, the honest next
question is the one the session notes could not answer: **does the rest of the
repository pass its own tests?** The 56 DB-gated suites have never been run
against a live database in this audit. `docs/missing.md` and
`docs/xp-removal-todo.md` are referenced by the 2026-09-09 session summary but
no longer exist on disk, so that summary's "optional remaining work" list is
stale and must not be treated as a backlog. See `docs/PLAN.md` §7.
