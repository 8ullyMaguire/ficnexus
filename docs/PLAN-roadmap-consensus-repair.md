# FicNexus — implementation plan: roadmap consensus admin panel and
# `roadmap_api` isolation

Companion to `docs/specs/roadmap-consensus-repair.md`. An LLM should be able to
execute this from the text alone. Every step names the file, the edit, and the
command that proves it.

## Step 0 — Fix the production query (R1)

**File:** `src/routes/admin.rs`, inside `admin_roadmap_consensus`, the
controversy query at ~line 1300.

**Change:** `WHERE status = 'open'` → `WHERE status != 'rejected'`

**Why this exact replacement, and not a guess.** The sibling handler
`consensus_handler` in `src/routes/roadmap.rs:425-434` is a near-identical copy
of this query, and it already uses `status != 'rejected'`. The two files are
duplicated logic that drifted. The public Roadmap page therefore shows a
working controversy scatter; the admin copy has been filtering a value its own
CHECK forbids and returning nothing.

Copying the sibling is the defensible choice: it is the in-repo idiom, it is
already exercised by the public endpoint, and it needs no new judgement about
which statuses are "still decidable". Anything narrower (for example
`status IN ('idea','up_next','in_progress')`) would be inventing policy.

**Verify immediately — do not defer to the end:**

```bash
psql "$DATABASE_URL" -c "select status, count(*) from feature_clusters group by 1"
# must show only valid statuses, none of them 'open'
psql "$DATABASE_URL" -c "select count(*) from feature_clusters where status != 'rejected'"
# must be > 0 on a seeded database
```

## Step 1 — Add a deterministic tiebreaker (R2)

**File:** `src/routes/admin.rs`, the leaderboard `ORDER BY` at ~line 1294.
**File:** `src/routes/roadmap.rs`, the same `ORDER BY` at ~line 418.

**Change:** `ORDER BY c.elo_rating DESC` → `ORDER BY c.elo_rating DESC, c.id ASC`

**Why both files.** They are duplicates; fixing one leaves the same
non-determinism in the other, and `consensus.rs:214` has a third variant. A
partial fix here is how the drift started.

**Why `c.id` last.** Elo must remain the primary key, or ranking breaks. `id`
is a stable integer surrogate, so it is total and repeatable.

**Verify:**

```bash
# Two clusters, identical Elo, three calls: the order must not vary.
psql "$DATABASE_URL" -c "select id, representative_text from feature_clusters order by elo_rating desc, id asc limit 5"
```

## Step 2 — Isolate `roadmap_api` (R3)

**File:** `tests/roadmap_api.rs`, the `cleanup` helper at ~line 181.

**Problem:** the leaderboard query is global and unscoped, so the test's
`lb[0]` assertion depends on whatever rows other suites left in
`feature_clusters`. It passes 13/13 on a cleared table and fails 1/13 on a
dirty one.

**Change:** make `cleanup` remove every `feature_clusters` row the suite's
prefixes can create, not only the ids tracked in the current run. Concretely,
add a prefix sweep alongside the id loop:

```sql
DELETE FROM feature_clusters
WHERE representative_text LIKE 'rmd\_%' ESCAPE '\'
   OR representative_text LIKE 'consensus\_%' ESCAPE '\'
```

Keep the existing id-based deletes: they are needed for rows whose text was
generated rather than prefixed.

**Do not** change the assertion. `lb[0]["text"] == "rmd_pubcons_top_feature"`
is correct and should hold once the query has a tiebreaker and the table is
clean. Loosening it to "some row has this text" would pass forever while the
determinism bug remained.

## Step 3 — Gate (R4, acceptance 5, 6)

```bash
cd /home/alvaro/code-local/rust/ficnexus
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
cargo build                                                    # must be clean
cargo test --lib 2>&1 | grep '^test result'                   # 935 passed; 0 failed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'  # 0
```

## Step 4 — Prove isolation both ways (acceptance 4)

This is the step that matters and the one most likely to be skipped. Run
`roadmap_api` in both states and record both numbers:

```bash
export DATABASE_URL='postgres://fichub:***@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='fichub-test-secret'

# (a) clean table
psql "$DATABASE_URL" -c "delete from feature_clusters"
cargo test --test roadmap_api -- --include-ignored --test-threads=1

# (b) polluted table - seed admin_api's rows first, then roadmap_api's own
psql "$DATABASE_URL" -c "delete from feature_clusters"
cargo test --test admin_api -- --include-ignored --test-threads=1 || true
cargo test --test roadmap_api -- --include-ignored --test-threads=1
```

**(b) must also be 13/13.** If (b) fails, the isolation fix is incomplete —
do not proceed to the full run, and do not report the cycle as done.

## Step 5 — Full re-measure

```bash
bash scripts/provision_test_db.sh
bash scripts/run_db_suites.sh
```

Record the totals. Do not compare against a previous run unless the database
was recreated the same way — the pass count depends on database history, which
is the finding this cycle exists to fix.

## Step 6 — Commit and tag

Two commits, production and test separated, because §3 of the spec keeps them
separate deliberately:

1. `fix(roadmap): the admin consensus panel filtered a status the schema forbids`
   — `src/routes/admin.rs`, `src/routes/roadmap.rs`
2. `test(roadmap): isolate roadmap_api from other suites' feature_clusters rows`
   — `tests/roadmap_api.rs`

Tag: `roadmap-consensus-repair-2026-09-26`

Then sync to `/home/alvaro/code/rust/ficnexus` and commit the session note to
`~/secondbrain` (`docs/sessions/` is gitignored in this repository).

## Rollback

Every step is either a one-line SQL predicate or an added DELETE. Reverting the
two commits restores the previous behaviour exactly; no migration is involved
and no schema changes.
