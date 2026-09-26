# The DB suite runner cannot run the DB suites

Date: 2026-09-26 · Status: spec written, not yet implemented
Target: `scripts/run_db_suites.sh`

## 1. The problem, measured

`bash scripts/run_db_suites.sh` reports:

    === TOTALS: 0 pass, 57 fail, 0 no-result, 57 suites ===

Zero of 57, and every suite finishes in 0.01s. Not slow failures — instant
ones, which means the process never got as far as touching the database.

`docs/sessions/2026-09-26-db-gated-suites.md` records a baseline of 39 of 56
suites passing. The same script now reports 0 of 57 passing, with one more
suite present. Those two numbers cannot both describe the same code, and the
session note is the more recent of the two claims, so something in between
broke the runner.

The cause, from a single suite run:

    thread 'submit_vote_flag_list_flow' panicked at src/config.rs:592:52:
    REDIS_URL must be set: NotPresent

`Config::from_env` requires exactly two variables, and both are read with
`std::env::var(...).expect(...)`:

- `src/config.rs:590` — `DATABASE_URL must be set`
- `src/config.rs:592` — `REDIS_URL must be set`

Everything else in `Config::from_env` uses `unwrap_or_else` and has a default.

## 2. Why it is a bug and not a missing step in my invocation

The runner is the documented entry point. It exists to produce "a comparable
number per suite" — that is its entire purpose, stated in its own header
comment. It provisions a fresh database before each suite. A script that
cannot be run without the operator first reading a spec file and hand-assembling
three environment variables is not a runner.

Two specifics, both verified by reading the file:

1. **`provision_test_db.sh` prints `DATABASE_URL` as its last line. The runner
   captures that stdout into `/dev/null` and discards it.** So the one piece
   of configuration the runner obtains for itself is thrown away one line later.
   The provisioner is not merely failing to be reused; its output is actively
   destroyed.

2. **The runner exports exactly one variable**, `FORUM_POST_DELAY_SECS`. The
   two required ones are never assigned anywhere in the file.

`docs/specs/db-gated-suites.md:161` does show the correct manual invocation:

    export DATABASE_URL=... REDIS_URL=redis://127.0.0.1:6379 JWT_SECRET=test-secret

So the knowledge exists and is written down. It was never moved into the script
that is supposed to need it, and the spec's own §7 verification block is a
manual loop for the same work the runner already automates.

## 3. Why nobody noticed

The failure mode is maximally quiet. The runner does not error out; it produces
a clean tally. `0 pass, 57 fail` is a plausible-looking number, and a reader
has no way to tell it apart from 57 genuine product defects. Every suite
"failing" is a coherent result to skim past.

The suite-level messages compound this. `src/config.rs` panics before any
assertion runs, so there are no assertion failures to read — the output says
`3 failed` with no test ever having run a query.

`tags_api` is the clearest illustration, because I ran it by hand with only
`DATABASE_URL` set:

    0 passed; 3 failed        (0.11s)   with DATABASE_URL set, REDIS_URL not
    REDIS_URL must be set: NotPresent

Adding the first variable moved the failure to the second. Both were missing
from the runner all along, and the script reported neither.

## 4. Scope

**In:** `scripts/run_db_suites.sh` — set up the required environment so the
script runs standalone.

**Out:** any change to `Config::from_env` (it should keep panicking on a
missing required var; that is correct behaviour and several suites depend on
the config being built), any change to the 57 suites, any production code.

Explicitly not doing the tempting fix: defaulting `REDIS_URL` inside
`config.rs`, or making the suites tolerate an absent database. Both convert a
loud, correct panic into a quiet, wrong result, which is how this got missed
in the first place.

## 5. What the fix has to satisfy

- Running `bash scripts/run_db_suites.sh` with no prior environment produces a
  real tally.
- The `DATABASE_URL` used is the one the provisioner just printed, not a
  separately configured constant that can drift from it.
- `REDIS_URL` defaults to `redis://127.0.0.1:6379`, and remains overridable from
  the environment, because the spec already treats that as the local value.
- A pre-existing `DATABASE_URL`/`REDIS_URL` in the environment is respected, so
  the script does not override an operator who has set up a different instance.
- With `FRESH_DB=0`, no provisioning happens, so a pre-existing
  `DATABASE_URL` must be used or the run must fail with a clear message rather
  than 57 identical panics.

## 6. Verification

Before: 0 of 57, every suite at 0.01s.

After, from a shell with `DATABASE_URL` and `REDIS_URL` explicitly unset:

```bash
cd ~/code-local/rust/ficnexus
env -u DATABASE_URL -u REDIS_URL bash scripts/run_db_suites.sh | tail -3
```

Must not report 0 pass. Must not report 0.01s per suite. A suite that genuinely
fails must do so on an assertion, with the panic gone from its output.

Cross-check against the recorded baseline, which is the real assertion of
correctness here — a number much lower than 39 would mean the runner is still
misconfigured in a new way:

```bash
cd ~/code-local/rust/ficnexus
env -u DATABASE_URL -u REDIS_URL bash scripts/run_db_suites.sh | grep TOTALS
```

Expect a pass count at or above 39 of 57. If it lands below, the fix did not
restore the baseline and the assumption that the runner was merely unconfigured
is wrong — which is itself worth knowing, and would mean re-reading the
39/56 measurement to find out what changed underneath it.

And the regression test for the quiet-failure mode: the output must be able to
distinguish "the suite failed" from "the suite never ran". If a sweep can still
report a confident-looking tally for a run in which `Config::from_env` panicked,
this bug can recur in the same silent way.

## 7. Addendum: the 56-vs-57 denominator

The spec text above calls the recorded baseline "39 of 56". The suite count has
been **57** for the entire recent history of this repository:

    04e3a50 test: real DB-suite baseline - 57 suites, 38 pass
    96debec test(forum): fix the two F7 gate tests      57
    a5530a1 docs: baseline now 39/18 after the F7 gate 57
    d52d5d5 fix(scripts): flush Redis                    57

`git ls-tree -r <commit> tests/ | grep -c '\.rs$'` returns 57 at every one of
those commits, and 57 files exist now. No suite has been added or removed
recently.

So the **39 figure is current and the 56 denominator is a typo** that has been
copied forward through the session notes. Read the baseline as **39 of 57**,
which is also what `04e3a50` says outright ("57 suites, 38 pass") before the
F7 gate work moved it to 39.

Consequence for verification: a correct sweep must report **at or above 39 of
57**, and reading it as "39 of 56" invites reading 39/57 as a regression. That
is the opposite of what the number means, and it is the same failure shape as
the original bug — a real result that reads as a problem.
