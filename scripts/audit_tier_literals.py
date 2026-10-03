#!/usr/bin/env python3
"""Does any gate's literal agree with the constant that states it?

The ficnexus curator bug was two steps worse than a dead constant:

    pub const CURATOR_MIN_TRUST: i16 = 3;   // referenced nowhere
    if user.trust_level < 5 { ... }        // hardcoded everywhere

So two checks, and the second is the one that catches the case where someone *did*
wire up the constant:

  1. a declared threshold constant with zero references   (dead)
  2. an inline `trust_level < N` whose N differs from the constant whose name
     matches the gate's own error message                   (disagreeing)

Check 2 is a heuristic on purpose. It only fires when a gate's message names a tier
("curator", "moderator", "admin") and a literal disagrees with the constant for that
tier. That keeps it quiet on gates whose number is a genuine per-call decision --
owner-or-staff checks, scope-restricted actions -- which are correct as written.

Run:  python3 scripts/audit_tier_literals.py [repo_root]
"""
import os
import re
import subprocess
import sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))

# The constants that state a tier boundary, and the word that names it.
TIERS = {
    "CURATOR_MIN_TRUST": "curator",
    "ADMIN_MIN_TRUST": "admin",
    "RESOLVE_MIN_TRUST": "resolv",
}


def git(*args):
    return subprocess.run(["git", "-C", ROOT, *args],
                          capture_output=True, text=True).stdout


def declared_constants():
    # git grep -E is POSIX ERE and has no \b, so match the identifier shape
    # directly: a leading non-identifier char (or start), then the constant.
    out = git("grep", "-n", "-E",
              r"(^|[^A-Za-z0-9_])[A-Z][A-Z0-9_]*(MIN_TRUST|MAX_TRUST)"
              r"[[:space:]]*:[[:space:]]*i[0-9]+[[:space:]]*=",
              "--", "*.rs")
    found = {}
    for line in out.splitlines():
        m = re.search(r"\b([A-Z][A-Z0-9_]*(?:MIN_TRUST|MAX_TRUST))\s*:\s*i\d+\s*=\s*(-?\d+)",
                      line)
        if m:
            found[m.group(1)] = int(m.group(2))
    return found


def gates():
    """Inline `trust_level < N` sites, with the message they emit.

    git grep -A output has three line forms, and getting this wrong silently yields
    ZERO parsed sites -- which looks exactly like "no problems found". That is the
    failure mode this parser exists to avoid, so each form is matched explicitly:

        path:lineno:text      the match itself
        path-lineno-text      a context line   <- different separator, easy to miss
        --                   the group divider

    An earlier version only accepted `[:-]` after the line number, matched 0 of 167
    lines, and reported a clean bill of health for a codebase that still had the bug.
    """
    out = git("grep", "-n", "-A2", r"trust_level < [0-9]", "--", "*.rs")
    match_re = re.compile(r"^(?P<path>[^:]+):(?P<line>[0-9]+):(?P<text>.*)$")
    ctx_re = re.compile(r"^(?P<path>.+)-(?P<line>[0-9]+)-(?P<text>.*)$")

    sites, cur = [], None
    for raw in out.splitlines():
        m = match_re.match(raw)
        if m:
            text = m.group("text").strip()
            g = re.search(r"trust_level < ([0-9]+)", text)
            if g:
                cur = {"loc": f"{m.group('path')}:{m.group('line')}",
                       "n": int(g.group(1)), "cond": text, "msg": ""}
                sites.append(cur)
                continue
            if text.startswith("//"):
                cur = None                    # the comment describing a past bug
                continue
        else:
            c = ctx_re.match(raw)
            if not c:
                cur = None
                continue
            text = c.group("text")

        if cur and not cur["msg"]:
            msg = re.search(r'Forbidden\(\s*(?:\\?")([^"\\]+)', text)
            if msg:
                cur["msg"] = msg.group(1)

    return sites


def main():
    consts = declared_constants()
    if not consts:
        sys.exit(f"no tier constants found in {ROOT}")

    print(f"tier constants in {os.path.basename(ROOT)}:")
    for k, v in sorted(consts.items()):
        refs = git("grep", "-c", "-w", k, "--", "*.rs")
        total = sum(int(l.rsplit(":", 1)[1]) for l in refs.splitlines() if ":" in l)
        flag = "  <-- DEAD, referenced nowhere" if total <= 1 else ""
        print(f"  {k:22} = {v}   refs={total}{flag}")

    print()
    bad = []
    for s in gates():
        msg = s["msg"].lower()
        for name, word in TIERS.items():
            if name not in consts or word not in msg:
                continue
            want = consts[name]
            if s["n"] != want:
                bad.append((s, name, want))
            break

    if not bad:
        print("no gate disagrees with the constant its own message names")
        return

    print(f"{len(bad)} gate(s) contradict the constant their message names:\n")
    for s, name, want in bad:
        print(f"  {s['loc']}")
        print(f"    message   \"{s['msg']}\"  -> names the {name} tier")
        print(f"    gate      trust_level < {s['n']}")
        print(f"    constant  {name} = {want}")
    print(f"\nfix: route these through a helper reading {sorted({b[1] for b in bad})}")


if __name__ == "__main__":
    main()