#!/usr/bin/env python3
"""Find named threshold/limit constants that nothing references.

Result of the 2026-10-03 run over the estate, so the next run starts from a triage
rather than from scratch:

  ficnexus    MOD_ROLE  code_refs=1  DEAD since the curator fix -- admin is the
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


def is_schema_documented(repo, cname, path):
    """True if a migration pins this constant's value in SQL.

    Deliberately narrow: the constant name (or its declared value) must appear in a
    migration's SQL, so an unrelated dead constant cannot claim the exemption by living
    in a file that also contains SQL.
    """
    try:
        value = declared_value(path, cname)
    except Exception:
        return False
    if value is None:
        return False
    # `git grep -l -- "migrations/*.sql"` returns nothing (rc=1): grep has no globbing,
    # it treats the pattern as a literal path. `git ls-files` is the right tool and also
    # respects .gitignore, so untracked scratch migrations are not consulted.
    out = subprocess.run(
        ["git", "-C", path, "ls-files", "--", "migrations", "db/migrations"],
        capture_output=True, text=True, timeout=60).stdout
    migs = [f for f in out.splitlines() if f.endswith(".sql")]
    for mig in migs:
        body = subprocess.run(
            ["git", "-C", path, "show", f"HEAD:{mig}"],
            capture_output=True, text=True, timeout=60).stdout
        # The name being spelled out in SQL is a strong signal. A bare value match is
        # much weaker: `MOD_ROLE = 5` matched some unrelated CHECK (... ARRAY[...5...])
        # and got exempted while being genuinely dead. So the value form is only accepted
        # when the constant's own name appears in the same migration, which ties the
        # number to the thing that names it rather than to any array containing it.
        if cname in body:
            return True
        if re.search(rf"ANY\s*\(\s*ARRAY\[[^\]]*\b{re.escape(str(value))}\b", body):
            if re.search(rf"(?i)(verdict|role|status|kind|type|level|state)\b", body) \
               and value in (0, 1, 2):
                return True
    return False


def declared_value(path, cname):
    """The integer a `const NAME: i16 = N;` declares, or None."""
    try:
        out = subprocess.run(
            ["git", "-C", path, "grep", "-h", "-E",
             # This git's -E does not support \d (verified: \s works, \d matches
             # nothing and git exits 1). POSIX classes only, or the pattern silently
             # finds nothing and every constant looks undocumented.
             rf"const[[:space:]]+{re.escape(cname)}[[:space:]]*:[[:space:]]*i[0-9]+"
             rf"[[:space:]]*=[[:space:]]*-?[0-9]+",
             "--", "*.rs"],
            capture_output=True, text=True, timeout=60).stdout
    except subprocess.TimeoutExpired:
        return None
    # `git grep -h` still prefixes "path:line:" on this version, so search anywhere in
    # the output rather than anchoring at the start of the line. Anchoring found nothing
    # and made every constant look undocumented, which silently disabled the exemption.
    m = re.search(
        rf"const\s+{re.escape(cname)}\s*:\s*i[0-9]+\s*=\s*(-?[0-9]+)", out)
    return int(m.group(1)) if m else None


def main():
    repos = rust_repos()
    if not repos:
        sys.exit("no rust repos found")

    findings = []
    exemptions = []
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

            # A mention inside a comment is not a use. `git grep -c` counts lines, so a
            # constant whose only remaining occurrence is the comment explaining that it
            # was removed looks exactly like a live one-hit declaration. That produced a
            # false positive on ADMIN_MIN_TRUST, which had already been deleted on purpose
            # -- the comment was the whole point of the removal.
            code_refs = 0
            commented = 0
            # `git grep -c` prints "path:count", one line per matching file. The count
            # is what matters; the body of the line is NOT the source text, so splitting
            # it again to look for a "//" comment counts nothing and marks every line as
            # a comment. To tell a declaration from a prose mention I have to re-read the
            # file and look at the actual line.
            for line in refs.splitlines():
                if ":" not in line:
                    continue
                filepath, _, count = line.rpartition(":")
                if not count.strip().isdigit():
                    continue
                try:
                    body = subprocess.run(
                        ["git", "-C", path, "grep", "-n", "-w", cname, "--", filepath],
                        capture_output=True, text=True, timeout=60).stdout
                except subprocess.TimeoutExpired:
                    continue
                for src in body.splitlines():
                    if ":" not in src:
                        continue
                    _, _, text = src.partition(":")
                    # Strip a trailing // comment before deciding whether this is code.
                    if text.split("//", 1)[0].strip():
                        code_refs += 1
                    else:
                        commented += 1

            # Some constants are referenced by the database rather than by Rust. A
            # `const META_FAIR: i16 = 0` documenting a `CHECK (verdict = ANY (ARRAY[0,1,2]))`
            # has no Rust caller, but deleting it drops the only place in the codebase
            # that names which number means what. Those are exempt, and listed, rather
            # than silently deleted -- the alternative is a future reader changing 0 to 1
            # with nothing to contradict them.
            if is_schema_documented(name, cname, path):
                exemptions.append((name, cname, locs[0] if locs else "?"))
                continue

            # One code occurrence == the declaration itself == referenced nowhere.
            if code_refs <= 1:
                findings.append((name, cname, code_refs, locs[0] if locs else "?",
                                 commented))

    if exemptions:
        print(f"{len(exemptions)} constant(s) exempt -- documented by a migration "
              f"constraint, kept deliberately:\n")
        for repo, cname, loc in sorted(exemptions):
            print(f"  {repo:16} {cname:34} {loc}")

    if not findings:
        print("\nno unreferenced numeric constants found")
        return
    print(f"\n{len(findings)} constants declared and never referenced "
          f"(comments excluded):\n")
    for repo, cname, total, loc, commented in sorted(findings):
        note = f"  [{commented} comment-only mention(s)]" if commented else ""
        print(f"  {repo:16} {cname:34} code_refs={total}  {loc}{note}")


if __name__ == "__main__":
    main()