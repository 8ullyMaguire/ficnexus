# FicNexus — specification: roadmap consensus admin panel, and a suite that
# only passes on a clean database

Status: active. Written 2026-09-26, before any code change.
Records: `docs/PLAN-roadmap-consensus-repair.md`.
Evidence: `docs/sessions/2026-09-26-roadmap-consensus-repair.md` (repo-local).

## 1. Problem

Two defects in one place, both in roadmap consensus. Neither is a role/level
gate assertion; they are distinct and were found by a pass-count that changed
for the wrong reason.

### 1.1 A production query filters a value the schema forbids

`admin_roadmap_consensus` (`src/routes/admin.rs:1274-1310`) returns a
leaderboard plus a controversy panel. The controversy query is:

```sql
SELECT id, representative_text, matches_played, times_picked_best,
       times_picked_worst,
       (times_picked_best + times_picked_worst) AS controversy,
       elo_rating::float8
FROM feature_clusters
WHERE status = 'open'
ORDER BY matches_played DESC
LIMIT 100
```

`migrations/064_roadmap_kanban.sql:12-13` replaced the CHECK with:

```sql
CHECK (status IN ('idea','long_term','medium_term','up_next','in_progress',
                   'finished','shipped','rejected'))
```

`'open'` is not in that set, and it cannot be: 064 mapped the existing
`'open'` rows to new values and removed it deliberately. Proven against the
live database:

```
insert into feature_clusters (representative_text, embedding, status)
values ('tieproof','[0.1,...]','open');
ERROR:  new row for relation "feature_clusters" violates check constraint
        "feature_clusters_status_check"
```

**The controversy panel has therefore always been empty.** It is the
"volume vs controversy" scatter the admin UI is meant to draw; it has never had
a single point on it. This is the same migration 064 defect class as the invalid
`DEFAULT 'open'` repaired in `docs/specs/missing-reference-data.md` — 064
changed the allowed set and did not update the code that still filtered on a
removed value. That repair fixed the column default; this is the query.

**Scope check.** A sweep for `status = 'open'` across `src/` returns six hits.
Only `admin.rs:1307` is against `feature_clusters`. The others are
`reports.rs` (moderation reports), `bounties.rs`, `trust.rs` and
`weekly_digest.rs`, which are different tables with their own valid `'open'`
value. They must not be touched. Verified by table, not by string match.

### 1.2 The leaderboard has no tiebreaker, so its order is not defined

```sql
ORDER BY c.elo_rating DESC
LIMIT 100
```

SQL does not guarantee a stable order among equal keys. `tests/admin_api.rs:411`
inserts a cluster named `consensus_top_feature` at `elo_rating = 1700`, and
`tests/roadmap_api.rs` inserts `rmd_pubcons_top_feature` at the same 1700. When
both exist, `lb[0]` is whichever row the server returns first, which varies.

The test asserts `lb[0]["text"] == "rmd_pubcons_top_feature"`. It is asserting
a tie-break that the query does not specify.

## 2. Why `roadmap_api` fails only sometimes

- `admin_api` and `roadmap_api` both seed `feature_clusters`.
- `run_db_suites.sh` runs suites alphabetically, so `admin_api` runs first.
- `admin_api` cleans up its own two rows by id (`tests/admin_api.rs:444`).
- Therefore ordering alone should not leak rows — but the two suites can
  interleave leftovers from *other* suites' `rmd_*` rows, and
  `roadmap_api`'s `cleanup` only removes ids it tracked in that run.

Observed, verified rather than assumed:

| database state | `roadmap_api` result |
|---|---|
| leftover `rmd_*` / `consensus_*` rows deleted | **13 passed, 0 failed** |
| immediately afterwards, without recreating the DB | **12 passed, 1 failed** |

So the suite's result depends on rows other suites left behind. **A pass count
in this repository is only meaningful against a known database state**, and a
clean `cargo test --test roadmap_api` is the only reliable way to attribute a
failure.

## 3. Requirements

**R1.** `admin_roadmap_consensus`'s controversy query must return rows.
- It must not filter on a value the CHECK forbids.
- Decide the replacement set deliberately and comment it. The panels that
  exist are those still in progress or decided, not yet delivered. The
  defensible reading of "controversy" is matches played vs disagreement, which
  is most meaningful for items still being decided.

**R2.** The leaderboard order must be total and deterministic.
- Add a unique tiebreaker to `ORDER BY` so equal Elo has a defined order.
- `c.id` is a sufficient and stable choice; it must be the *last* key so Elo
  still dominates.
- This is a production change, so it needs justification in the commit: the
  admin leaderboard is consumed by a UI that will otherwise jitter between
  equally-ranked items.

**R3.** `roadmap_api` must be isolated from other suites' rows.
- Its assertions on `lb[0]` are only meaningful if the leaderboard contains
  only what the test put in it. Give it a deterministic database state of its
  own rather than inheriting whatever ran before.
- Prefer a real fix in the test over a loosened assertion. Asserting
  "my row is somewhere in the list with the right Elo" would pass forever while
  the bug remained.

**R4.** Do not change the pass count by weakening assertions.
- Every assertion that stays must still be able to fail.

## 4. Non-goals

- The 22 other failing suites. They are role/level gate assertions and are a
  separate cycle.
- `admin_api`'s own 9 failures. Same reason.
- `ask_archive_api`'s analytics-marker failure. Unrelated, pre-existing.
- Any change to the allowed `feature_clusters` status set. 064 is applied and
  immutable; widening the CHECK to re-admit `'open'` would undo a deliberate
  migration.

## 5. Acceptance

Each must be demonstrated, not asserted:

1. `status = 'open'` in `admin.rs:1307` is gone, replaced by a valid set.
2. A controversy query against a database holding a decidable cluster returns a
   row — the real check that R1 is met, since a passing test cannot distinguish
   "fixed" from "still empty" without seeded data.
3. Two clusters at equal Elo return in a stable, repeatable order across
   repeated calls.
4. `roadmap_api` passes 13/13 on a database that has just had
   `feature_clusters` cleared, **and** on one that has not.
5. All 56 suites still compile; 935 unit tests still pass.
6. `git status --short` shows only intended files.
