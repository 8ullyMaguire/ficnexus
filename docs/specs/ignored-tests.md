# The `#[ignore]` attributes: a false alarm I raised and then disproved

Date: 2026-09-27 · Status: **retracted**, kept for the record
Repo: `~/code-local/rust/ficnexus`

## What I claimed

I ran `cargo test --test translation_review_api` directly and got
`0 passed; 0 failed; 7 ignored`. I generalised that into a spec titled
"314 ignored tests: the suite that reports green while most of it never runs",
with a table of per-suite ignore counts and the claim that **46% of the test
suite has never been executed**.

## Why it was wrong

`scripts/run_db_suites.sh` runs every suite as:

    cargo test --test "$n" -- --include-ignored --test-threads=1

`--include-ignored`. The attributes are overridden on every sweep, and the
results file agrees — `translation_review_api` there reads:

    translation_review_api: test result: FAILED. 0 passed; 7 failed; 0 ignored

**Zero ignored, seven failed.** The tests run. They are genuinely broken. So is
`forum_api`: `44 passed; 5 failed; 0 ignored`, not 49 suppressed.

My direct invocation was missing the flag the runner always passes. I read
"ignored" as a skip and built a document on it. The counts in that document —
314 attributes, 50 of 58 suites, `forum_api` 49/68 — are real counts of
attributes; the **inference** drawn from them was wrong.

## How it happened, precisely

I had a one-off helper (`r1.sh`) to run a single suite quickly, written for
debugging an individual failure. It called `cargo test --test "$suite"` with no
`--include-ignored`. I then used its output as the basis for a claim about the
repo, without checking the flag against the canonical runner.

The check I skipped was one line: `grep include-ignored scripts/run_db_suites.sh`,
which I had in fact read earlier in the same session — the output was in front of
me in the exploration above and I did not use it.

This is the same failure mode as the 401/403 work in miniature: **a test result
that reflects the harness I happened to invoke, read as a fact about the
product.** A debug helper is not the system under test. When the two disagree,
the canonical runner wins, and the difference must be explained before the
lower-trust source is quoted.

## What survives, and what does not

Does not survive:

- "46% of the test suite has never been executed" — **false**
- "the 45/58 pass count is measuring a fraction of the real suite" — **false**;
  it measures the whole suite, because ignored tests are included
- "fixing the red without touching the ignored is polishing a dashboard wired to
  the wrong panel" — **false**; the red is the panel
- every recommendation derived from those claims

Survives, and is worth keeping:

- The file-header run instructions are still **wrong**: every one of these
  headers says to source `/personal/documents/code/rust/fichub/.env`, a path
  from a different machine that does not exist here. The *current* runner does
  the right thing, but the documented procedure does not, and it is the
  documented one a human reads.
- `#[ignore]` still carries no reason, so a reader cannot tell why. Rust
  supports `#[ignore = "reason"]` and none of the 314 use it. This is
  cosmetic-today and a real problem the first time someone is tempted to use the
  attribute to hide a failure.
- The genuine, correctly-scoped problem: **35 tests fail across 13 suites.**
  That is the real state of this repo, and it is what the next work is.

## The lesson, recorded because it recurs here

I have now twice this session taken a single test invocation and generalised
from it. The first time (401 vs 403) the error was mine and tests were
patched to match reality. The second time I nearly wrote a whole spec and a
plan on top of a missing command-line flag.

Rule: **before quoting a test result as a fact about the repo, confirm the
invocation matches the canonical runner.** A helper script written for
debugging is evidence about the helper.
