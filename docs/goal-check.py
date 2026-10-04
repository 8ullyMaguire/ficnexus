#!/usr/bin/env python3
"""An executable definition of done for ficnexus.

Why this exists
---------------
ficnexus was the one audited repo without a predicate. That makes "is ficnexus done?"
unanswerable: there is nothing to ask. The other five repos (threadlight, lorebot,
kindred, polaris, zeph) each have one, and this is that pattern applied here.

What makes it worth having rather than ceremonial
------------------------------------------------
Every clause is built so that **the empty case cannot be mistaken for the good case**.
That is not a stylistic preference. In ficnexus specifically, `audit_tier_literals.py`
reported "no gate contradicts" because a checker parsed `git grep -A2` with the wrong
separator, matched 0 of 167 lines, and printed a confident pass. `tier_constants_agree.rs`
was written afterwards to catch exactly that class of failure.

So each clause below also reports *how much it looked at*, and this file refuses to
report COMPLETE if any clause examined nothing.

Usage
-----
    python3 docs/goal-check.py                     # the predicate
    python3 docs/goal-check.py --update-baseline   # re-measure floors after growth

Exit status is 0 only when every clause passes.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MIG_DIR = ROOT / "migrations"
TESTS = ROOT / "tests"
SRC = ROOT / "src"

# Floors, not targets. They exist so a silent drop -- a filter that matches nothing, a
# test binary that stops running -- is a FAIL rather than a quieter pass. Raise them
# with --update-baseline when the suite grows on purpose.
BASELINE_LIB_TESTS = 935

results: list[tuple[str, str, str]] = []


def add(clause: str, status: str, detail: str = "") -> None:
    results.append((clause, status, detail))


def sh(cmd: str, timeout: int = 900, env: dict | None = None) -> tuple[int, str, str]:
    e = dict(os.environ)
    if env:
        e.update(env)
    try:
        p = subprocess.run(
            cmd, shell=True, capture_output=True, text=True, timeout=timeout, cwd=ROOT, env=e
        )
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return 124, "", "timed out"


# --------------------------------------------------------------------------- build


def clause_build() -> None:
    rc, out, err = sh("cargo build 2>&1", timeout=1800)
    if rc == 124:
        add("builds", "UNKNOWN", "cargo build timed out")
        return
    if rc != 0:
        errors = [l for l in (out + err).splitlines() if l.startswith("error")]
        add("builds", "FAIL", f"{len(errors)} error line(s); first: {errors[0][:140] if errors else '?'}")
        return
    add("builds", "PASS", "cargo build clean")


def clause_fmt() -> None:
    rc, out, _ = sh("cargo fmt --all --check 2>&1", timeout=300)
    if rc == 0:
        add("fmt clean", "PASS", "no diff")
        return
    files = re.findall(r"^Diff in (\S+)", out, re.M)
    add("fmt clean", "FAIL", f"{len(files)} file(s) need formatting; first: {files[0] if files else out[:120]}")


def clause_clippy() -> None:
    """Clippy must stay at or below the recorded finding count.

    ficnexus does NOT deny warnings -- no [workspace.lints], no clippy.toml -- and carries
    roughly 100 pedantic findings at HEAD (doc indentation, `Iterator::last` on a
    DoubleEndedIterator, `Default::default()` field assignment, redundant closures). Those
    are real but pre-existing and mechanical; declaring them a failure on day one would be
    this clause inventing a bar the repo never set.

    So the clause is a ratchet, not a gate: findings may not increase. Clearing them is
    welcome and lowers the number; adding one fails. Dead code is excluded because
    "delete the handler" is not a legitimate response to "this handler has no route yet" --
    that gap belongs to the routing clause, which has a budget.
    """
    rc, out, err = sh(
        "cargo clippy --all-targets --message-format short 2>&1", timeout=1800
    )
    if rc == 124:
        add("clippy clean", "UNKNOWN", "clippy timed out")
        return

    lines = [l for l in (out + err).splitlines() if ":" in l]
    findings = [
        l for l in lines
        if re.search(r":\s*(error|warning):\s", l)
        and "never used" not in l and "never read" not in l
        and "generated " not in l
    ]
    n = len(findings)

    budget_file = ROOT / "docs" / "clippy-budget.txt"
    if not budget_file.exists():
        add("clippy clean", "FAIL",
            f"docs/clippy-budget.txt is missing, so there is no recorded budget for "
            f"{n} finding(s). Create it with the current count; this clause cannot decide "
            "on its own whether that number is acceptable.")
        return
    m = re.search(r"^max_findings=(\d+)\s*$", budget_file.read_text(), re.M)
    if not m:
        add("clippy clean", "FAIL", f"docs/clippy-budget.txt has no `max_findings=N` line")
        return
    allowed = int(m.group(1))

    if n > allowed:
        first = findings[0].split(" ", 1)[-1][:120]
        add("clippy clean", "FAIL",
            f"{n} findings, above the recorded budget of {allowed}. Clearing them is "
            f"welcome -- lower max_findings when you do. First: {first}")
        return
    add("clippy clean", "PASS",
        f"{n} findings, within the recorded budget of {allowed}. A ratchet: this may fall, "
        "never rise. Dead code is excluded and reported by the routing clause.")


# ----------------------------------------------------------------------- structure


def clause_migrations_ordered() -> None:
    """Migration timestamps must be unique and ordered.

    A duplicate timestamp means one of the two files is silently never applied, and that
    stays invisible until a column is missing at runtime.
    """
    files = sorted(MIG_DIR.glob("*.sql"))
    if not files:
        add("migrations ordered", "FAIL",
            f"no .sql in {MIG_DIR.name}/. If migrations moved, this clause is stale and would "
            "otherwise pass on an empty directory -- the exact failure mode it is about.")
        return
    stamps = []
    for f in files:
        m = re.match(r"(\d+)", f.name)
        if not m:
            add("migrations ordered", "FAIL", f"unnumbered migration: {f.name}")
            return
        stamps.append(m.group(1))
    dupes = sorted({s for s in stamps if stamps.count(s) > 1})
    if dupes:
        add("migrations ordered", "FAIL", f"duplicate timestamps {dupes}; one of each pair is never applied")
        return
    add("migrations ordered", "PASS", f"{len(files)} migrations, {stamps[0]}..{stamps[-1]}")


def clause_db_reachable() -> str | None:
    dsn = os.environ.get("DATABASE_URL", "").strip()
    if not dsn:
        add("database reachable", "UNKNOWN",
            "DATABASE_URL is not set, so the DB-backed suites cannot run. Export it, e.g.\n"
            "         export DATABASE_URL=postgres://polaris:***@127.0.0.1:55433/"
            "ficnexus_test?sslmode=disable")
        return None
    rc, out, _ = sh(f'psql "{dsn}" -tAc "SELECT 1" 2>&1', timeout=60)
    if rc == 0 and "1" in out:
        add("database reachable", "PASS", f"{dsn.split('@')[-1]} answers SELECT 1")
        return dsn

    err = out.strip()
    # Distinguish the two failures, because the fixes are unrelated. "does not exist"
    # means provision it; "extension is not available" means this Postgres cannot run
    # ficnexus at all, and no amount of retrying will change that.
    if "does not exist" in err:
        add("database reachable", "FAIL",
            f"{dsn.split('@')[-1]} does not exist. Provision it with:\n"
            f"         PGHOST=... PGPORT=... scripts/provision_test_db.sh\n"
            "         (the script honours PGHOST/PGPORT/PGUSER/PGPASSWORD -- it no longer\n"
            "          hardcodes 127.0.0.1:5432)")
        return None
    if "extension" in err and "not available" in err:
        add("database reachable", "FAIL",
            f"{dsn.split('@')[-1]} is unreachable: {err[:100]}\n"
            "         This Postgres has no pgvector. ficnexus migrations 001 and 088 use\n"
            "         vector(N) as a COLUMN type, so this instance cannot run this repo at\n"
            "         all -- a stock postgres:15-alpine image has cube, ltree and pg_trgm\n"
            "         but not vector. Point DATABASE_URL at an instance built from\n"
            "         pgvector/pgvector, or start one.\n"
            "         This is NOT fixable by re-running the provision script.")
        return None
    add("database reachable", "FAIL", f"cannot reach {dsn.split('@')[-1]}: {err[:140]}")
    return None


# ------------------------------------------------------------------------- tests


def clause_lib_tests() -> None:
    rc, out, err = sh("cargo test --lib 2>&1", timeout=2400, env={"TMPDIR": "/tmp"})
    if rc == 124:
        add("unit tests green", "UNKNOWN", "timed out")
        return
    results_line = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed", out)
    if not results_line:
        # The vacuous-pass guard: zero matching output is not a pass.
        add("unit tests green", "FAIL",
            "cargo test --lib produced no 'test result' line, so I cannot tell a clean run "
            "from one that ran nothing.")
        return
    passed = sum(int(r[1]) for r in results_line)
    failed = sum(int(r[2]) for r in results_line)
    if failed:
        names = [l for l in out.splitlines() if l.startswith("--- FAIL")][:5]
        add("unit tests green", "FAIL", f"{failed} failed; " + "; ".join(n[:60] for n in names))
        return
    if passed < BASELINE_LIB_TESTS:
        add("unit tests green", "FAIL",
            f"{passed} passed, below the floor of {BASELINE_LIB_TESTS}. A suite that shrank "
            "usually means a filter stopped matching.")
        return
    add("unit tests green", "PASS", f"{passed} passed (floor {BASELINE_LIB_TESTS})")


# ----------------------------------------------------------------- the repo's own audits


def run_audit(script: str, clause: str, what: str, clean_sentence: str) -> None:
    """Run one of ficnexus's audit scripts and require a stated count.

    The point of requiring the count is that these scripts print a summary. A script that
    silently examined nothing would otherwise look identical to a clean one.
    """
    path = ROOT / "scripts" / script
    if not path.exists():
        add(clause, "FAIL", f"scripts/{script} is missing; a clause guarding a gone script is not a check")
        return
    rc, out, err = sh(f"python3 scripts/{script} 2>&1", timeout=1800)
    if rc == 124:
        add(clause, "UNKNOWN", f"{script} timed out")
        return
    if rc == 2 and "STALE EXEMPTION" in (out + err):
        add(clause, "FAIL", f"{script} found an exemption for a column that is now written: "
                            f"{(out + err).strip().splitlines()[-1][:140]}")
        return
    if rc != 0:
        add(clause, "FAIL", f"{script} exited {rc}: {(err or out).strip().splitlines()[-1][:140] if (err or out).strip() else 'no output'}")
        return
    # Two acceptable shapes, both of which state something countable:
    #   "... 3 gate(s) contradict ..."   -> 3 findings
    #   "no gate disagrees ..."          -> 0 findings
    # Anything else means the script changed its output, and a clause that cannot read it
    # is worse than no clause: it would either fail forever or, worse, be "fixed" by
    # deleting the assertion.
    m = re.search(r"(\d+)\s+" + what, out)
    if m:
        n = int(m.group(1))
        if n:
            add(clause, "FAIL", f"{n} {what}; see the script's own output for the list")
            return
        add(clause, "PASS", "0 " + what)
        return
    if clean_sentence and clean_sentence in out:
        add(clause, "PASS", f"clean: {clean_sentence!r}")
        return
    add(clause, "FAIL",
        f"{script} exited 0 but stated neither a count of {what!r} nor the clean sentence "
        f"{clean_sentence!r}, so a run that examined nothing is indistinguishable from a "
        f"clean one. Update this clause and the script together. Output was: {out[:200]!r}")


def clause_dead_constants() -> None:
    """Dead constants in THIS repo.

    `audit_dead_constants.py` sweeps every repo under ~/code, not just ficnexus, which is
    what makes it useful as a cross-repo sweep and wrong as a ficnexus clause: all six
    remaining findings belong to lorehaven (owner-excluded), streakforge and tessera.
    Filtering here rather than in the script, because the sweep is the script's job and
    the scoping is this clause's.

    A finding in another repo is reported as its own informational clause, so it stays
    visible without making ficnexus's predicate hostage to repos nobody is working on.
    """
    path = ROOT / "scripts" / "audit_dead_constants.py"
    if not path.exists():
        add("dead constants", "FAIL", "scripts/audit_dead_constants.py is missing")
        return
    rc, out, err = sh("python3 scripts/audit_dead_constants.py 2>&1", timeout=1800)
    if rc == 124:
        add("dead constants", "UNKNOWN", "audit timed out")
        return
    text = out + err
    if "STALE EXEMPTION" in text:
        add("dead constants", "FAIL",
            f"exemption list names a column that is now written: {text.strip().splitlines()[-1][:140]}")
        return
    if rc != 0:
        add("dead constants", "FAIL", f"audit exited {rc}: {text.strip().splitlines()[-1][:140] if text.strip() else 'no output'}")
        return

    rows = re.findall(r"^\s{2}(\S+)\s+(\S+)\s+code_refs=(\d+)", text, re.M)
    mine = [(r, c, n) for r, c, n in rows if r == "ficnexus"]
    theirs = [(r, c, n) for r, c, n in rows if r != "ficnexus"]

    if not rows and "no unreferenced" not in text:
        add("dead constants", "FAIL",
            f"audit exited 0 but printed neither findings nor the clean sentence, so a run "
            f"that examined nothing looks clean: {text[:180]!r}")
        return

    if mine:
        detail = ", ".join(f"{c}" for _, c, _ in mine)
        add("dead constants", "FAIL",
            f"{len(mine)} constant(s) declared and never referenced: {detail}")
    else:
        add("dead constants", "PASS", "no dead constants in ficnexus")

    if theirs:
        names = ", ".join(sorted({r for r, _, _ in theirs}))
        add("dead constants (other repos)", "PASS",
            f"{len(theirs)} in {names}; informational, not a ficnexus blocker")


def clause_tier_literals() -> None:
    run_audit("audit_tier_literals.py", "tier gates", r"gate\(s\) contradict",
              "no gate disagrees with the constant its own message names")


def clause_tier_audit_tests() -> None:
    """The tests that keep the checker honest must themselves run.

    `tier_constants_agree.rs` contains `the_checker_actually_finds_gates`, which is the
    test that fails if the checker goes vacuous. It is easy to leave a test file in place
    that no command runs; this clause makes its execution part of the predicate.
    """
    rc, out, _ = sh("cargo test --test tier_constants_agree --test audit_tooling 2>&1", timeout=2400)
    if rc == 124:
        add("audit tests run", "UNKNOWN", "timed out")
        return
    found = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed", out)
    if not found:
        add("audit tests run", "FAIL",
            "neither tier_constants_agree nor audit_tooling produced a test result line")
        return
    passed = sum(int(r[1]) for r in found)
    failed = sum(int(r[2]) for r in found)
    if failed:
        add("audit tests run", "FAIL", f"{failed} failed")
        return
    if passed < 7:
        add("audit tests run", "FAIL",
            f"{passed} passed, expected at least 7 (3 tier + 4 audit tooling). One of the "
            "checker-guard tests is not running.")
        return
    add("audit tests run", "PASS", f"{passed} checker-guard tests across 2 binaries")


# --------------------------------------------------------------------- routing


def clause_handler_reachability() -> None:
    """Public handlers must be reachable from the router, within a recorded budget.

    ficnexus mirrors a Go original with a larger route table, so unrouted handlers are
    expected. What this clause prevents is the count drifting upward unnoticed, which is
    what a bare count would do -- so the gap is pinned in docs/routing-budget.txt and
    raising it has to be a deliberate edit.

    Measured from real source, not from a name list: a handler counts as routed when the
    router names it.
    """
    server = SRC / "server.rs"
    if not server.exists():
        add("handlers routed", "FAIL", "src/server.rs is missing; this clause is stale")
        return

    router_src = server.read_text(errors="replace")

    # Public async handlers defined anywhere under src/.
    defined: dict[str, str] = {}
    for f in sorted(SRC.rglob("*.rs")):
        for m in re.finditer(r"pub\s+async\s+fn\s+([a-z_][a-z0-9_]*)", f.read_text(errors="replace")):
            defined.setdefault(m.group(1), str(f.relative_to(ROOT)))

    if not defined:
        add("handlers routed", "FAIL",
            "no `pub async fn` found under src/; either the codebase moved or this clause "
            "is stale. A zero here would otherwise read as 'nothing unrouted'.")
        return

    # A handler is routed when the router names it. Word-boundary match, because
    # `list` must not be satisfied by `list_all`.
    unrouted = sorted(
        (name, loc) for name, loc in defined.items()
        if not re.search(rf"\b{re.escape(name)}\b", router_src)
    )

    budget_file = ROOT / "docs" / "routing-budget.txt"
    if not budget_file.exists():
        add("handlers routed", "FAIL",
            f"docs/routing-budget.txt is missing, so there is no recorded budget for "
            f"{len(unrouted)} unrouted handler(s) of {len(defined)}. Without it this clause "
            "has nothing to compare against and would either always fail or always pass.")
        return

    raw = budget_file.read_text()
    m = re.search(r"^unrouted=(\d+)\s*$", raw, re.M)
    if not m:
        add("handlers routed", "FAIL",
            f"docs/routing-budget.txt has no `unrouted=N` line: {raw[:100]!r}")
        return
    allowed = int(m.group(1))
    actual = len(unrouted)

    if actual > allowed:
        sample = ", ".join(n for n, _ in unrouted[:6])
        add("handlers routed", "FAIL",
            f"{actual} unrouted, above the recorded budget of {allowed}. If that is "
            f"deliberate, raise it in docs/routing-budget.txt with a reason -- do not let "
            f"it drift. First few: {sample}")
        return
    add("handlers routed", "PASS",
        f"{len(defined) - actual}/{len(defined)} handlers reachable from server.rs "
        f"({100*(len(defined)-actual)//max(len(defined),1)}%), "
        f"{actual} known-unrouted within budget={allowed}")


# ------------------------------------------------------------------------ driver


def update_baseline() -> None:
    rc, out, _ = sh("cargo test --lib 2>&1", timeout=2400, env={"TMPDIR": "/tmp"})
    if rc != 0:
        print("refusing to write a baseline from a red run", file=sys.stderr)
        sys.exit(1)
    found = re.findall(r"test result: \w+\. (\d+) passed", out)
    if not found:
        print("no test results found; refusing to write a baseline", file=sys.stderr)
        sys.exit(1)
    n = sum(int(x) for x in found)
    print(f"measured {n} lib tests; set BASELINE_LIB_TESTS = {n}")


def main() -> None:
    if "--update-baseline" in sys.argv:
        update_baseline()
        return

    clause_build()
    clause_fmt()
    clause_migrations_ordered()
    clause_db_reachable()
    clause_lib_tests()
    clause_clippy()
    clause_dead_constants()
    clause_tier_literals()
    clause_tier_audit_tests()
    clause_handler_reachability()

    print()
    width = max(len(c) for c, _, _ in results)
    for clause, status, detail in results:
        mark = {"PASS": "ok  ", "FAIL": "FAIL", "UNKNOWN": "????"}[status]
        print(f"[{mark}] {clause:<{width}}")
        if detail:
            for line in detail.splitlines():
                print(f"       {line}")

    print()
    failed = [r for r in results if r[1] == "FAIL"]
    unknown = [r for r in results if r[1] == "UNKNOWN"]
    if failed:
        print(f"INCOMPLETE -- {len(failed)} clause(s) failed:")
        for clause, _, detail in failed:
            print(f"  - {clause}: {detail}")
        sys.exit(1)
    if unknown:
        print(f"INCOMPLETE -- {len(unknown)} clause(s) could not be evaluated:")
        for clause, _, detail in unknown:
            print(f"  - {clause}: {detail}")
        sys.exit(1)
    print(f"COMPLETE -- all {len(results)} clauses pass.")


if __name__ == "__main__":
    main()