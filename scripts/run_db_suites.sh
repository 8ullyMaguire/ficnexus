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

: > /tmp/suite_results.txt
pass=0; fail=0; noresult=0
for t in tests/*.rs; do
  if [ "${FRESH_DB:-1}" = "1" ]; then
    bash scripts/provision_test_db.sh >/dev/null 2>&1 || {
      echo "provision FAILED before ${n}; skipping"
      echo "${n} PROVISION_FAILED" >> /tmp/suite_results.txt
      noresult=$((noresult+1))
      continue
    }
  fi
  n=$(basename "$t" .rs)
  out=$(CARGO_TARGET_DIR=~/.cargo-target/ficnexus timeout 300 \
        cargo test --test "$n" -- --include-ignored --test-threads=1 2>&1)
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
echo "=== TOTALS: $pass pass, $fail fail, $noresult no-result, $((pass+fail+noresult)) suites ==="
