# Plan: the DB suite runner runs the DB suites

Spec: `docs/specs/db-suite-runner-environment.md`

Goal: `bash scripts/run_db_suites.sh` works from a bare shell and produces a
tally at or above the recorded 39/57 baseline, instead of 0/57 at 0.01s each.

Only `scripts/run_db_suites.sh` changes. No Rust, no suite edits, no
`config.rs`. The suites and the config are correct as they are.

---

## Step 1 — capture the provisioned DATABASE_URL instead of discarding it

`Config::from_env` requires `DATABASE_URL` (`src/config.rs:590`) and
`REDIS_URL` (`:592`), both via `.expect()`. The runner currently exports only
`FORUM_POST_DELAY_SECS` and sends the provisioner's stdout to `/dev/null`.

The provisioner's **last line** is the URL — verified: it ends with
`postgres://fichub:...@127.0.0.1/ficnexus_test` after the `SCHEMA OK` line. Its
earlier lines are per-migration `OK` output, so a bare `tail -1` is right, but
`tail -1` alone is brittle: if the provisioner ever prints a trailing blank
line or a warning after the URL, the capture silently becomes empty and every
suite panics again — the exact failure this plan exists to remove.

**Do this.** In the `FRESH_DB` block, capture stdout into a variable and pull
the URL from it with a pattern that tolerates a trailing newline:

```bash
  if [ "${FRESH_DB:-1}" = "1" ]; then
    # Capture, don't discard. The provisioner prints DATABASE_URL as its last
    # line and Config::from_env requires it (src/config.rs:590). This runner
    # used to send this to /dev/null, so every suite panicked on startup and
    # the sweep reported a confident-looking 0 of 57.
    if ! provision_out=$(bash scripts/provision_test_db.sh 2>/dev/null); then
      echo "provision FAILED before ${n}; skipping"
      echo "${n} PROVISION_FAILED" >> /tmp/suite_results.txt
      noresult=$((noresult+1))
      continue
    fi
    # grep, not tail -1: immune to a trailing blank line or trailing warning.
    db_url=$(printf '%s\n' "$provision_out" | grep -Eo 'postgres(ql)?://[^[:space:]]+' | tail -1)
    if [ -z "$db_url" ]; then
      echo "provision produced no DATABASE_URL before ${n}; skipping"
      echo "${n} PROVISION_FAILED" >> /tmp/suite_results.txt
      noresult=$((noresult+1))
      continue
    fi
    DATABASE_URL="$db_url"
    export DATABASE_URL
  fi
```

Note the existing code references `${n}` before `n` is assigned — `n` is set
after the provision call. That is a latent bug: on a provisioning failure the
error message names the *previous* suite. Set `n` before the block while you
are in there.

`grep -Eo` rather than `tail -1` on the last line, for the reason above.

**Verify:**

```bash
cd ~/code-local/rust/ficnexus
env -u DATABASE_URL -u REDIS_URL bash -c '
  out=$(bash scripts/provision_test_db.sh 2>/dev/null) || { echo PROVISION_FAILED; exit 1; }
  db_url=$(printf "%s\n" "$out" | grep -Eo "postgres(ql)?://[^[:space:]]+" | tail -1)
  echo "captured: ${db_url:-<EMPTY>}"
'
```

Expect `captured: postgres://fichub:...@127.0.0.1/ficnexus_test`, never
`<EMPTY>`. Then confirm the guard rejects a provisioner that prints no URL:

```bash
env -u DATABASE_URL bash -c '
  out="SCHEMA OK - 180 tables"
  db_url=$(printf "%s\n" "$out" | grep -Eo "postgres(ql)?://[^[:space:]]+" | tail -1)
  echo "guard: ${db_url:-<EMPTY - correct>}"
'
```

Expect `<EMPTY - correct>`. Both together prove the capture works and the
guard fires, which is what stops a future refactor from reintroducing the
silent version of this bug.

---

## Step 2 — supply REDIS_URL, and respect anything already set

Redis is confirmed listening on `127.0.0.1:6379` (`redis-cli ping` → `PONG`),
and `docs/specs/db-gated-suites.md:161` already treats that as the local value.

Add near the top, with the other `export`, so it applies to both the
`FRESH_DB=1` and `FRESH_DB=0` paths:

```bash
# Config::from_env panics on a missing REDIS_URL (src/config.rs:592), and the
# specs already use the local instance. Default it, but never override a value
# an operator has set - a second Redis is a legitimate test target.
export REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"
```

`${REDIS_URL:-...}` not `${REDIS_URL-...}`: the `:-` form also substitutes when
`REDIS_URL` is set but empty, which is a realistic way to get here.

**Do not** add a default for `REDIS_URL` inside `config.rs`. That would turn
a correct panic into a silent misconfiguration, which is how this bug survived
being reported as "0 of 57 failing" without anyone looking for it.

**Verify:**

```bash
cd ~/code-local/rust/ficnexus
bash -c 'source scripts/run_db_suites.sh 2>/dev/null & sleep 2; kill %1 2>/dev/null' 2>/dev/null
# Simpler, direct check of the export form:
bash -c 'unset REDIS_URL; export REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"; echo "$REDIS_URL"'
```

Expect `redis://127.0.0.1:6379`. Then prove it does not override:

```bash
bash -c 'export REDIS_URL=redis://example:9999; export REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"; echo "$REDIS_URL"'
```

Expect `redis://example:9999`. An override that gets clobbered is the same
class of quiet failure as the original bug.

---

## Step 3 — fail loudly when FRESH_DB=0 has no usable DATABASE_URL

With `FRESH_DB=0` nothing provisions, so Step 1's capture never runs. If
`DATABASE_URL` is unset the sweep would again produce 57 identical config
panics — the same confident-looking tally, the exact failure mode the spec
requires be distinguishable.

After the `FRESH_DB` block, before the suite loop:

```bash
if [ -z "${DATABASE_URL:-}" ]; then
  echo "DATABASE_URL is not set and FRESH_DB=0, so no database was provisioned." >&2
  echo "Either let the runner provision one (drop FRESH_DB=0) or export" >&2
  echo "DATABASE_URL yourself." >&2
  exit 1
fi
```

A hard exit rather than a tally, because a tally here is indistinguishable
from real results.

**Verify:**

```bash
cd ~/code-local/rust/ficnexus
env -u DATABASE_URL FRESH_DB=0 bash scripts/run_db_suites.sh; echo "exit=$?"
```

Expect the three-line message and `exit=1` — and it must return in about a
second, not start 57 suites. Then confirm the normal path is unaffected:

```bash
env -u DATABASE_URL bash -c 'timeout 60 bash scripts/run_db_suites.sh 2>&1 | head -4'
```

Expect it to reach `Re-provisioning...` and start running suites, not exit 1.

---

## Step 4 — make "never ran" distinguishable from "failed"

The spec's last requirement: a sweep must not be able to report a
plausible-looking tally for a run in which `Config::from_env` panicked.

Today `run_db_suites.sh` greps for `^test result:` and reads the numbers out of
it. A config panic produces no `test result:` line at all, so the suite lands
in the `NO RESULT` bucket — which the totals *do* already distinguish. The gap
is narrower and worse than that: the summary prints `fail` for these, and a
reader scanning the tail sees `57 fail` with no hint that 57 is a startup
failure.

Classify a suite whose output contains a `config.rs` panic as its own outcome,
so the total can never read as product failures:

```bash
  if printf '%s' "$out" | grep -qE 'panicked at src/config\.rs:[0-9]+:[0-9]+:'; then
    reason=$(printf '%s' "$out" | grep -oE '[A-Z_]+ must be set: NotPresent' | head -1)
    echo "${n} CONFIG_MISSING (${reason:-unknown})" >> /tmp/suite_results.txt
    noconfig=$((noconfig+1))
    printf '%-34s CONFIG_MISSING %s\n' "$n" "${reason:-unknown}"
    continue
  fi
```

Add `noconfig=0` to the initialiser next to `pass=0; fail=0; noresult=0`, and
print it in the totals line:

```bash
echo "=== TOTALS: $pass pass, $fail fail, $noresult no-result, $noconfig config-missing, $total suites ==="
```

A `config-missing` count above zero after this fix is a regression in the
runner itself, and is now visible in one number rather than hidden inside
`fail`.

**Verify — the mutation check, which is the whole point.** Reintroduce the
original bug in a scratch copy and confirm the tally changes shape:

```bash
cd ~/code-local/rust/ficnexus
cp scripts/run_db_suites.sh /tmp/rs_fixed.sh
# Remove the DATABASE_URL export to simulate the pre-fix state.
sed '/^    export DATABASE_URL$/d' /tmp/rs_fixed.sh > /tmp/rs_broken.sh
chmod +x /tmp/rs_broken.sh
```

Rather than run a full 57-suite sweep twice (each pass re-provisions a
database per suite), assert the classifier directly on a captured failure. Feed
it the real output of a suite run with the env removed:

```bash
cd ~/code-local/rust/ficnexus
out=$(CARGO_TARGET_DIR=~/.cargo-target/ficnexus env -u DATABASE_URL -u REDIS_URL \
      timeout 200 cargo test --test tags_api -- --include-ignored --test-threads=1 2>&1)
printf '%s' "$out" | grep -qE 'panicked at src/config\.rs:[0-9]+:[0-9]+:' \
  && echo "CLASSIFIED as config-missing (correct)" || echo "MISSED (regression)"
```

Expect `CLASSIFIED as config-missing (correct)`. With the env supplied, the
same suite must NOT be classified:

```bash
out=$(DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test' \
      REDIS_URL=redis://127.0.0.1:6379 \
      CARGO_TARGET_DIR=~/.cargo-target/ficnexus \
      timeout 200 cargo test --test tags_api -- --include-ignored --test-threads=1 2>&1)
printf '%s' "$out" | grep -qE 'panicked at src/config\.rs' \
  && echo "CLASSIFIED (wrong - should have run)" || echo "not classified (correct - suite ran)"
```

Expect `not classified (correct - suite ran)`. The first must not fire when
the second does; if it does, the classifier is matching on something other
than the config panic.

---

## Step 5 — full sweep, and the real assertion

```bash
cd ~/code-local/rust/ficnexus
env -u DATABASE_URL -u REDIS_URL bash scripts/run_db_suites.sh 2>&1 | tail -5
```

The pass count must be **at or above 39** (the baseline in
`docs/sessions/2026-09-26-db-gated-suites.md`, out of 57 suites). `0` means
the fix did not take. Below 39 means it took but something else regressed, and
the assumption that the runner was merely unconfigured is wrong — re-read that
session note and find what changed underneath it.

`config-missing` must be **0**. Any non-zero value means a suite still panics
in `Config::from_env`, which after this fix can only be a suite needing
another required var.

Record the number and the per-suite list. Do not tidy the failures in this
change: each one is a real defect that needs the individual judgement the
classification spec already calls for.

---

## Step 6 — document, so the next person does not rediscover this

- `docs/PLAN-db-suite-runner-environment.md` — this file, committed alongside.
- Append a section to `docs/sessions/2026-09-26-db-gated-suites.md` recording
  the measured before/after, since that note is where the 39/56 baseline lives
  and it is the natural place someone looks.
- The 39-vs-56 denominator discrepancy: the baseline is out of 56, the sweep
  now finds 57 suites. Establish where the 57th came from and state it, or a
  future reader will compare 39/56 against 39/57 and read it as a regression.

Definition of done: Steps 1–4 each verified as written, Step 5 at or above 39
pass with 0 config-missing, Step 6 committed, `bash -n` clean on the script,
and the diff confined to `scripts/run_db_suites.sh` plus docs.

---

## Addendum: three steps in this plan were wrong, and running them is what found it

Kept because the plan is the artifact a future session reads, and a plan that
lists only its successes will have its errors re-derived from scratch.

**Step 4's classifier matched the wrong file.** It keyed on
`panicked at src/config.rs:` — the real location of the `Config::from_env`
panic. But the suites never reach `Config::from_env` first: each has its own
`std::env::var("DATABASE_URL").expect(...)` inside its own `pool()` helper, at
`tests/tags_api.rs:33`, `tests/admin_api.rs:37`, `tests/auto_tag.rs:25` and
others, and each carries a *different* hint string. Keyed on the file, the
classifier matched none of them and the plan's own mutation check reported
`MISSED (regression)`. It matches the message now.

**The regex could not span the hint.** The real text is

    DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a): NotPresent

The first attempt required `must be set` adjacent to `: NotPresent`, which
nothing in the corpus contains. A parenthesised hint sits between the halves.
Confirmed the working pattern in isolation before wiring it back into the
script, after the second attempt's failure turned out to be a quoting bug in my
test harness rather than in the pattern.

**Step 3's guard broke the default path.** Written before the loop, it probed
`${DATABASE_URL:-}` — but the loop sets that *per suite* from the provisioner.
So on the default `FRESH_DB=1` run the guard fired immediately and exited 1
having printed nothing useful. The loop is where `DATABASE_URL` becomes set, so
a guard placed before it can only ever see an operator-supplied value. The
guard now tests `FRESH_DB` itself.

The general lesson, and the reason each of these survived plan review: the plan
was checked for internal consistency and never executed. Every one of the three
was caught within a minute of running the verification step that already
existed in the plan. A verification step nobody runs is a comment.

## Measured result

    === TOTALS: 39 pass, 18 fail, 0 no-result, 0 config-missing, 57 suites ===

The baseline, exactly. The runner was the whole problem; the code under it is
unchanged and as healthy as it was recorded.

