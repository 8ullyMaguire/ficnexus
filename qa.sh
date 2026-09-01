#!/usr/bin/env bash
# fichub QA — local-first bug discovery + safe local autofix. Zero cloud tokens.
# v2: flock, doctor gate, strict lifecycle, repro/verify gates, exit codes.
# Usage:
#   ./qa.sh                          one discovery pass
#   ./qa.sh --doctor                 environment check (exit 3 if broken)
#   ./qa.sh --loop [--autofix]       converge until 0 open P0/P1 or no progress
#   ./qa.sh --repro BUG-0001         run repro command for one bug
#   ./qa.sh --verify BUG-0001        run all gates for one bug
#   ./qa.sh mark-fixed BUG-0001 --test path
#   ./qa.sh mark-needs-human BUG-0001 --reason "..."
#   ./qa.sh --workpackets            generate per-bug work packets
#   ./qa.sh --metrics                convergence metrics
# Exit: 0 clean · 1 bugs remain · 2 no progress · 3 env failure · 4 flaky · 5 autofix rejected
set -euo pipefail
cd "$(dirname "$0")"

# ── flock: prevent cron/manual overlap ──────────────────────────────────────
LOCK_FILE="${XDG_CACHE_HOME:-$HOME/.cache}/fichub-qa.lock"
mkdir -p "$(dirname "$LOCK_FILE")"
exec 200>"$LOCK_FILE"
if ! flock -n 200; then
  echo "QA already running; exiting."
  exit 0
fi

SKIP_BROWSER=0; RUN_TRIAGE=0; GEN_ISSUES=0; AUTOFIX=0; LOOP=0; DOCTOR=0; REPRO=""; VERIFY=""; WORKPACKETS=0; METRICS=0; MODE=""
for arg in "$@"; do
  case "$arg" in
    --skip-browser) SKIP_BROWSER=1 ;;
    --triage) RUN_TRIAGE=1 ;;
    --issues) GEN_ISSUES=1 ;;
    --autofix) AUTOFIX=1 ;;
    --loop) LOOP=1 ;;
    --doctor) DOCTOR=1 ;;
    --repro) REPRO="${2:-}"; shift ;;
    --verify) VERIFY="${2:-}"; shift ;;
    --workpackets) WORKPACKETS=1 ;;
    --metrics) METRICS=1 ;;
    mark-fixed|mark-needs-human|mark-flaky) MODE="$arg" ;;
  esac
done

if [ ! -d qa/node_modules ]; then
  echo "installing qa deps..."
  (cd qa && npm install --no-audit --no-fund playwright-core@1.62.1 better-sqlite3 >/dev/null 2>&1)
fi

# ── doctor gate ─────────────────────────────────────────────────────────────
if [ "$DOCTOR" = "1" ] || [ "$REPRO" != "" ] || [ "$VERIFY" != "" ]; then
  node qa/doctor.js || { echo "env broken; not filing app bugs"; exit 3; }
fi
if [ "$REPRO" != "" ]; then
  node qa/verify.js "$REPRO"; exit $?
fi
if [ "$VERIFY" != "" ]; then
  node qa/verify.js "$VERIFY"; exit $?
fi
if [ "$MODE" = "mark-fixed" ] || [ "$MODE" = "mark-needs-human" ] || [ "$MODE" = "mark-flaky" ]; then
  node qa/mark.js "$@"; exit $?
fi
if [ "$WORKPACKETS" = "1" ]; then
  node qa/gen-workpackets.js "${@:2}"; exit $?
fi
if [ "$METRICS" = "1" ]; then
  node qa/report.js --metrics; exit $?
fi

open_bugs() {
  node -e "
    const Database = require('./qa/node_modules/better-sqlite3');
    const db = new Database('./qa/bugs.db');
    const r = db.prepare(\"SELECT COUNT(*) n FROM bugs WHERE status IN ('new','triaged','needs-repro','reproducible','autofix-proposed','autofix-verified','ready-for-cloud','cloud-in-progress','fixed-pending-verify','flaky','regression')\").get();
    console.log(r.n);
  " 2>/dev/null || echo 0
}

run_once() {
  echo "== FicHub QA (local, deterministic) =="
  QA_SKIP_BROWSER=$SKIP_BROWSER node qa/run.js || true
  if [ "$RUN_TRIAGE" = "1" ]; then echo; echo "== local LLM triage =="; node qa/triage.js || true; fi
  if [ "$AUTOFIX" = "1" ]; then echo; echo "== local autofix (proposal-only, branch-based) =="; node qa/autofix.js || true; fi
  if [ "$GEN_ISSUES" = "1" ]; then echo; echo "== ISSUES.md + work packets =="; node qa/gen-issues.js || true; node qa/gen-workpackets.js >/dev/null 2>&1 || true; fi
}

if [ "$LOOP" = "1" ]; then
  MAX_PASSES=10
  pass=0; prev_open=-1; no_progress=0
  echo "== loop mode: discover → triage → autofix → rediscover until 0 open P0/P1 =="
  while [ "$pass" -lt "$MAX_PASSES" ]; do
    pass=$((pass + 1))
    echo; echo "── pass $pass ──"
    run_once
    OPEN=$(open_bugs)
    echo; echo "pass $pass: $OPEN open bugs"
    if [ "$OPEN" = "0" ]; then echo "== no open bugs. convergence reached. =="; exit 0; fi
    if [ "$prev_open" -ge 0 ] && [ "$OPEN" -ge "$prev_open" ]; then
      no_progress=$((no_progress + 1))
      if [ "$no_progress" -ge 2 ]; then echo "== no progress for 2 passes; needs human/cloud. =="; exit 2; fi
    else no_progress=0; fi
    prev_open=$OPEN
  done
  echo "== max passes reached. =="; exit 2
else
  run_once
fi

echo
echo "done. Queue: qa/bugs.db · ISSUES.md: qa/reports/ISSUES.md · Work packets: qa/reports/workpackets/"
