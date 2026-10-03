#!/usr/bin/env python3
"""Find named threshold/limit constants that nothing references.

Result of the 2026-10-03 run over the estate, so the next run starts from a triage
rather than from scratch:

  ficnexus    ADMIN_MIN_TRUST  refs=1  DEAD since the curator fix -- admin is the
                                     is_admin claim, so no trust literal is the
                                     right admin gate. Kept as documentation of the
                                     old ladder; delete or repoint deliberately.
  ficnexus    META_FAIR/_UNFAIR/_UNSURE, MOD_ROLE  relics of the 2026-09 metamod
                                     retirement. The table and its CHECK constraint
                                     survive in migration 001; no code writes it.
                                     Delete when migration 001 is squashed.
  lorehaven / tessera / streakforge   left ALONE -- another agent owns those repos.
                                     Re-run once that work lands.

A constant whose only occurrence is its own definition means the design decided
something the code never applied. Found on ficnexus: `CURATOR_MIN_TRUST` (3) was
declared in src/services/trust.rs and referenced nowhere, while every curator gate
hardcoded `5` -- so trust levels 3 and 4 were refused tools they were entitled to, and
935 lib tests passed throughout.

The audit is a grep. That is the point: it costs nothing and finds the class of bug
where a stated boundary and the implemented boundary disagree.
"""
import os
import re
import subprocess
import sys
from collections import defaultdict

ROOTS = [os.path.expanduser(p) for p in
         ("~/code-local/rust", "~/code-local/go", "~/code-local/python",
          "~/code/rust", "~/code/go", "~/code/python")]

EXCLUDE = ("target/", "/.git/", "node_modules/", "__pycache__/",
           "hermes-", "upstream/")

# Constants whose whole job is to be a named number. Anything matching is a candidate.
DECL = re.compile(
    r"\b(?:pub(?:\([^)]*\))?\s+)?(?:const|static(?:\s+mut)?)\s+"
    r"(?P<name>[A-Z][A-Z0-9_]{3,})\s*:\s*(?:i\d+|u\d+|f\d+|usize|f64)\s*=\s*"
    r"(?P<val>[-\w.]+)"
    r"\s*;",
)

# Names that are almost certainly configuration, not a stated boundary.
SKIP = re.compile(r"^(VERSION|SCHEMA|MAGIC|_|MAX_|MIN_|DEFAULT_|PORT|HOST|"
                  r".*_ENV|.*_PATH|.*_URL|.*_NAME|.*_FILE|.*_DIR|.*_SIZE|"
                  r".*_LIMIT_TOTAL|.*_COUNT_LIMIT|.*_BYTES|.*_LEN|.*_SIZE_)")


def rust_repos():
    seen = {}
    for root in ROOTS:
        if not os.path.isdir(root):
            continue
        for d in sorted(os.listdir(root)):
            p = os.path.join(root, d)
            if os.path.isdir(os.path.join(p, ".git")):
                real = os.path.realpath(p)
                seen.setdefault(d, real)
    return seen


def main():
    repos = rust_repos()
    if not repos:
        sys.exit("no rust repos found")

    findings = []
    for name, path in sorted(repos.items()):
        try:
            out = subprocess.run(
                ["git", "-C", path, "grep", "-n", "-E",
                 r"\b[A-Z][A-Z0-9_]{3,}\s*:\s*(i|u)(8|16|32|64|size)\s*=",
                 "--", "*.rs", "*.go"],
                capture_output=True, text=True, timeout=60).stdout
        except subprocess.TimeoutExpired:
            continue

        # group by constant name
        per = defaultdict(list)
        for line in out.splitlines():
            m = re.search(r"\b([A-Z][A-Z0-9_]{3,})\s*:\s*(?:i|u)(?:8|16|32|64|size)\s*=\s*([-\w.]+)", line)
            if not m:
                continue
            cname = m.group(1)
            if SKIP.match(cname):
                continue
            per[cname].append(line.split(":", 2)[0] + ":" + line.split(":", 2)[1])

        for cname, locs in per.items():
            # Count references to the name anywhere in tracked source.
            try:
                refs = subprocess.run(
                    ["git", "-C", path, "grep", "-c", "-w", cname,
                     "--", "*.rs", "*.go"],
                    capture_output=True, text=True, timeout=60).stdout
            except subprocess.TimeoutExpired:
                continue
            total = sum(int(l.rsplit(":", 1)[1]) for l in refs.splitlines()
                        if ":" in l)
            # One occurrence == the declaration itself == referenced nowhere.
            if total <= 1:
                findings.append((name, cname, total, locs[0] if locs else "?"))

    if not findings:
        print("no unreferenced numeric constants found")
        return
    print(f"{len(findings)} constants declared and never referenced:\n")
    for repo, cname, total, loc in sorted(findings):
        print(f"  {repo:16} {cname:34} refs={total}  {loc}")


if __name__ == "__main__":
    main()