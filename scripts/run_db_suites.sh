#!/usr/bin/env bash
# Run every DB-gated integration suite serially and record the real result.
# --test-threads=1 is required: the suites serialise on their own Mutex, delete
# their seed rows by unique prefix, and assume they are alone in the database.
set -uo pipefail
cd "$(dirname "$0")/.."
# Forum flood control has a real 10s window (FORUM_POST_DELAY_SECS, default 10
# in Config::from_env). Suites share users, so two posts inside 10s trip it and
# the failure lands on whichever test posted second, not on the one that
# deserves it. check_flood returns early when delay <= 0 (src/routes/forum.rs
# :470-477), so the window is disabled for the test run.
#
# Tradeoff: forum_api no longer asserts that flood control works. That is
# deliberate and recorded rather than accidental - the two tests that tripped it
# were never testing the limiter, they were testing replies and search.
export FORUM_POST_DELAY_SECS="${FORUM_POST_DELAY_SECS:-0}"

# Config::from_env panics on a missing REDIS_URL (src/config.rs:592), and
# docs/specs/db-gated-suites.md:161 already uses the local instance. Default it
# so the runner stands alone, but never override a value an operator has set --
# a second Redis is a legitimate test target. ${VAR:-...} and not ${VAR-...} so
# a set-but-empty REDIS_URL also gets the default.
#
# This default deliberately does NOT go into config.rs. Defaulting there would
# turn a correct panic into a silent misconfiguration, which is exactly how a
# 0-of-57 sweep read as 57 real product defects.
export REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"

# A fresh database per suite, not once for the run. Suites share users and
# tables, so a suite's result depends on what ran before it; the whole point of
# this script is a comparable number per suite. provision_test_db.sh now fails
# loudly instead of leaving a stale schema behind (see the EXPECTED_TABLES
# assertion and the DROP note in that script).
#
# Override with FRESH_DB=0 to reproduce the shared-database behaviour.
if [ "${FRESH_DB:-1}" = "1" ]; then
  echo "Re-provisioning the test database before each suite..."
fi

# flock: only one sweep at a time. Two concurrent runs -- or a sweep plus a
# one-off `provision_test_db.sh` in another shell -- will otherwise drop the
# database out from under each other, and every suite after that point fails
# with "relation \"users\" does not exist". That happened here and produced a
# sweep whose failures were all provisioning artifacts. Take the lock on the
# same descriptor the loop uses so the guard covers the whole run.
exec 9>/tmp/ficnexus_test_db.lock
if ! flock -n 9; then
  echo "Another ficnexus test run holds /tmp/ficnexus_test_db.lock." >&2
  echo "Wait for it, or kill it. Running two at once invalidates both." >&2
  exit 1
fi

# FRESH_DB=0 means nothing provisions and the per-suite capture never runs, so
# a run with no DATABASE_URL would produce 57 identical config panics -- a tally
# indistinguishable from 57 real defects. FRESH_DB is constant for the whole run,
# so this can be decided up front. The normal FRESH_DB=1 path is unaffected: it
# sets DATABASE_URL inside the loop, and deliberately does not inherit one.
if [ "${FRESH_DB:-1}" = "0" ] && [ -z "${DATABASE_URL:-}" ]; then
  echo "DATABASE_URL is not set and FRESH_DB=0, so no database was provisioned." >&2
  echo "Either let the runner provision one (drop FRESH_DB=0) or export" >&2
  echo "DATABASE_URL yourself." >&2
  exit 1
fi

: > /tmp/suite_results.txt
pass=0; fail=0; noresult=0; noconfig=0
for t in tests/*.rs; do
  n=$(basename "$t" .rs)
  if [ "${FRESH_DB:-1}" = "1" ]; then
    # Capture, don't discard. The provisioner prints DATABASE_URL as its last
    # line and Config::from_env requires it (src/config.rs:590). This runner
    # used to send that to /dev/null, so every suite panicked in from_env and
    # the sweep still reported a confident-looking 0 of 57.
    if ! provision_out=$(bash scripts/provision_test_db.sh 2>/dev/null); then
      echo "provision FAILED before ${n}; skipping"
      echo "${n} PROVISION_FAILED" >> /tmp/suite_results.txt
      noresult=$((noresult+1))
      continue
    fi
    # grep rather than tail -1: immune to a trailing blank line or warning.
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
  out=$(CARGO_TARGET_DIR=~/.cargo-target/ficnexus timeout 300 \
        cargo test --test "$n" -- --include-ignored --test-threads=1 2>&1)
  # A config.rs panic means the suite never reached a single assertion. It is
  # not a product defect and must not be counted as one -- that confusion is
  # what let a 0-of-57 startup failure sit in the docs looking like 57 broken
  # features. Classified separately so the total cannot be misread.
  # Matched on the message, not on a source location. Each suite has its own
  # `std::env::var("DATABASE_URL").expect(...)` in its own `pool()` helper --
  # tests/tags_api.rs:33, tests/admin_api.rs:37 and others -- and the wording
  # differs per suite ("load .env" vs "run with .env loaded"). Keying on
  # src/config.rs missed all of them, and a real Config::from_env panic
  # (config.rs:590/592) is the same condition: the required env was absent
  # before the suite ran a single assertion.
  if printf '%s' "$out" | grep -qE 'must be set: NotPresent'; then
    reason=$(printf '%s' "$out" | grep -oE '[A-Z_]+ must be set' | head -1)
    noconfig=$((noconfig+1))
    echo "${n} CONFIG_MISSING (${reason:-unknown})" >> /tmp/suite_results.txt
    printf '%-34s CONFIG_MISSING %s\n' "$n" "${reason:-unknown}"
    continue
  fi
  line=$(printf '%s' "$out" | grep -E '^test result:' | tail -1)
  if [ -z "$line" ]; then
    echo "$n: NO RESULT (compile error or timeout)" >> /tmp/suite_results.txt
    noresult=$((noresult+1))
    printf '%-34s NO RESULT\n' "$n"
  else
    echo "$n: $line" >> /tmp/suite_results.txt
    if printf '%s' "$line" | grep -q ' 0 failed'; then
      pass=$((pass+1)); printf '%-34s PASS  %s\n' "$n" "$(printf '%s' "$line" | sed 's/test result: //')"
    else
      fail=$((fail+1)); printf '%-34s FAIL  %s\n' "$n" "$(printf '%s' "$line" | sed 's/test result: //')"
    fi
  fi
done
echo
# A non-zero config-missing count means the runner is misconfigured again, and
# it is deliberately a separate number so it cannot hide inside "fail".
echo "=== TOTALS: $pass pass, $fail fail, $noresult no-result, $noconfig config-missing, $((pass+fail+noresult+noconfig)) suites ==="
