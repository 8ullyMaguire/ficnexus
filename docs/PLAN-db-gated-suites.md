# FicNexus — implementation plan: DB-gated integration suites

Companion to `docs/specs/db-gated-suites.md`. That document says what is true;
this one says what to do, in order, with the verification for each step.
Written 2026-09-26, before any code change.

## 0. Ground rules

- `CARGO_TARGET_DIR=~/.cargo-target/ficnexus` on every cargo command. The repo
  is on an sshfs mount; a local target dir is much faster.
- **`--test-threads=1` on every suite run.** The suites serialise on their own
  `Mutex`, self-delete seed rows by unique prefix, and assume exclusivity in the
  database. This is required for correctness, not politeness.
- **Tests only.** If a fix needs a production line, stop and decide. See §6.
- `touch tests/*.rs` before a compile check if you have run `cargo test` already
  in the same tree — cargo will otherwise report cached results and the failure
  count will look wrong. This cost real confusion once already.

## 1. Provision (already done, verify it)

```bash
cd ~/code-local/rust/ficnexus
bash scripts/provision_test_db.sh
```

Expected last two lines:

```
SCHEMA OK - 180 tables
postgres://fichub:***@127.0.0.1/ficnexus_test
```

If it does not print 180 tables, stop and fix provisioning before touching any
test. Everything downstream depends on a real schema. The four host
prerequisites (pgvector installed, superuser for `CREATE EXTENSION`, the
`fichub` role, explicit URLs because `psql -d <name>` ignores them) are
documented in the spec §4 and encoded in the script.

Redis must be running: `redis-cli ping` → `PONG`.

## 2. Record the baseline

```bash
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379'
export JWT_SECRET='test-secret'
cd ~/code-local/rust/ficnexus
touch tests/*.rs
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --no-run 2>&1 | tee /tmp/compile.log
grep -cE 'could not compile `fichub` \(test ' /tmp/compile.log     # expect 10
```

Keep `/tmp/compile.log`. Every later step diffs against it.

## 3. Fix `E0063` — the 10 `AppState` literals

The failing suites: `ask_archive_api`, `curator_api`, `feedback_api`, `heal_api`,
`honeypot_api`, `recommender_personal`, `tags_api`, `translation_review_api`,
`uploads_api`, `user_export_api`.

**Do not hand-write this.** `tests/forum_api.rs:68` already builds `AppState`
correctly with all 19 fields, and 45 other suites do the same. Read it, then
copy its `AppState { ... }` block verbatim into each failing suite, adjusting
only the variable names that suite already uses.

The three fields the failures name:

| field | value to use |
|---|---|
| `jwt_secret` | `"test-secret".to_string()` — match what the suite's auth helpers expect |
| `redis_client` | `Some(redis_client.clone())` if a `redis::Client` is in scope, else `None` |
| `rt_manager` | whatever the 46 working suites pass — **read `forum_api.rs` for this, do not guess the constructor** |

Work one suite at a time and compile it immediately:

```bash
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --test curator_api --no-run
```

**Verify after each:** the error count for that suite drops. Do not batch all
ten — a shared mistake copied ten times is ten mistakes.

## 4. Fix `E0560` — the 12 `role` fixtures

Same refactor as the unit suite, same trap: `role` was overloaded, so the field
is chosen by the test's intent, not by the number.

For each site, read the surrounding test and apply:

- The test creates a user only to own a post/comment/report → `trust_level: 0`.
  The privilege is incidental.
- The test asserts a curator action is allowed or refused → `trust_level`, with
  a value the production gate actually compares against
  (`src/routes/authors.rs:551` uses `>= 5`; readers are `< 3`).
- The test exercises a forum mod/admin gate → `level` (0-100 axis).

Never carry a number across without checking it is in range for the new field.

**Verify:** `cargo test --test <name> --no-run` for each of the seven suites
with `E0560` sites.

## 5. Resolve the removed handler `forum::my_level`

`tests/forum_api.rs:233` references `fichub::routes::forum::my_level`, which
does not exist.

```bash
grep -rn 'my_level' ~/code-local/rust/ficnexus/src/ | head
grep -rn 'users/me/level\|me_level\|/level' ~/code-local/rust/ficnexus/src/server.rs | head
```

Three outcomes:

1. **It moved.** Point the test at the new module path. Nothing else changes.
2. **It was renamed.** Use the new name.
3. **It was genuinely removed.** Delete the route registration from the test
   router, and delete any test whose only purpose was to exercise it. Record
   the removal in the commit message.

**Do not re-create the handler to make a test compile.** That would be a
production change to satisfy a test, which is backwards, and the removal was
most likely deliberate.

## 6. Gate: everything compiles

```bash
cd ~/code-local/rust/ficnexus
touch tests/*.rs
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --no-run 2>&1 | tee /tmp/compile2.log
grep -cE 'could not compile `fichub` \(test ' /tmp/compile2.log    # expect 0
```

If non-zero, there are more sites than the baseline showed — cargo stops after
some suites fail, so later errors surface only as earlier ones are fixed. Expect
to iterate this step. That is normal, not a sign the plan is wrong.

## 7. Run all 56 and record real results

```bash
cd ~/code-local/rust/ficnexus
: > /tmp/suite_results.txt
for t in tests/*.rs; do
  n=$(basename "$t" .rs)
  out=$(CARGO_TARGET_DIR=~/.cargo-target/ficnexus timeout 300         cargo test --test "$n" -- --include-ignored --test-threads=1 2>&1)
  line=$(printf '%s' "$out" | grep -E '^test result:' | tail -1)
  if [ -z "$line" ]; then
    echo "$n: NO RESULT (compile error or timeout)" | tee -a /tmp/suite_results.txt
  else
    echo "$n: $line" | tee -a /tmp/suite_results.txt
  fi
done
```

Then produce the count the spec §5 asks for:

```bash
grep -c 'ok\.' /tmp/suite_results.txt        # suites passing
grep -c 'FAILED' /tmp/suite_results.txt     # suites failing
grep -c 'NO RESULT' /tmp/suite_results.txt  # suites that could not run
```

## 8. Handle failures as findings, not as obstacles

For each failing suite, in this order:

1. Read the assertion and the production code it exercises.
2. Decide: **is the test stale, or is the code wrong?**
   - The product changed deliberately (this is common here — the suites were
     written across many months of change) → the test is stale. Update it, and
     say in the commit which behaviour changed and why you believe so.
   - The code contradicts its own spec or another test → that is a regression.
     **Stop and report it.** Do not fix production code inside a test-repair
     commit; record it for its own spec-and-plan cycle.
3. Never edit an assertion purely to make it green. The question is always
   "which of these two is wrong", and the answer has to be justified.

Suites that need a service absent from this host (a live Ollama, an SMTP
server) may not be runnable. `OllamaClient` is constructed with a URL in tests
but not called, so most should run — verify rather than assume. Record anything
that genuinely cannot run, and what it would take.

## 9. Commit

Two commits, so the repair and the measurement are separable:

- **Commit 1** — the compile repairs (steps 3-6). Message lists the 10 suites,
  the 22 sites, and the three error classes, and states that no production code
  changed.
- **Commit 2** — any test updates from step 8, plus
  `docs/sessions/2026-09-26-db-gated-suites.md` recording the real counts and
  every finding. Do not fold a regression into commit 1; if step 8 turns up a
  code bug, leave it as a documented finding.

Tag: `db-gated-suites-2026-09-26`.

The session note is a durable record, so put the counts in the **vault**
(`~/secondbrain/90-Meta/`) as well — this repo gitignores `docs/sessions/`
("local only"), so a note written there is not committed by default.

## 10. Definition of done

- [ ] `scripts/provision_test_db.sh` prints `SCHEMA OK - 180 tables`
- [ ] `cargo test --no-run` reports 0 compile errors across all 56 suites
- [ ] every suite has a recorded result: pass, fail, or cannot-run with a reason
- [ ] `git diff --stat` shows changes only in `tests/` and `docs/`
- [ ] `cargo build` still succeeds, unchanged
- [ ] any regression found is documented, not silently fixed
- [ ] committed in two parts, tagged
- [ ] vault log has the counts
