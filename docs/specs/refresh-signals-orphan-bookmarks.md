# FicNexus — specification: `refresh_signals` fails on any orphaned bookmark

Status: active. Written 2026-09-26, before any code change.
Found while executing step 3 of `docs/PLAN-failing-suite-classification.md`,
which classified this as a test problem. **That classification was wrong**, and
this spec records why.

## 1. The defect

`src/recommender/signals.rs:63-76` builds the recommender's unified signal view:

```sql
INSERT INTO rec_user_signals (user_id, work_id, signal_type, signal_weight, occurred_at)
SELECT b.user_id, b.url_id, 'bookmark',
       $2 * (CASE WHEN b.user_id = $1 THEN $3 ELSE 1.0 END),
       COALESCE(b.created_at, NOW())
FROM bookmarks b
WHERE b.url_id IS NOT NULL
```

`rec_user_signals.work_id` has a foreign key:

```
FOREIGN KEY (work_id) REFERENCES fic_info(id) ON DELETE CASCADE
```

`bookmarks` has **no** foreign key on `url_id`. Its constraints are:

```
PRIMARY KEY (id)
UNIQUE (user_id, url_id)
NOT NULL url_id
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
FOREIGN KEY (work_id) REFERENCES works(id)
```

So `bookmarks.url_id` is an unconstrained text column, and it is populated with
`fic_info` slugs elsewhere. Nothing prevents it from referencing a row that does
not exist. The moment it does, this insert violates the `rec_user_signals` FK and
`refresh_signals` returns an error.

The same pattern repeats for `work_ratings` at lines 83-88, which inserts
`r.url_id` into the same FK-constrained column.

## 2. How it presents

As a test failure, which is why it was misclassified. On the current test
database:

```
 user_id |        username        |       url_id
---------+------------------------+-------------------
     301 | userexportit_full_user | userexportit_fic_a
      24 | bkmkcsv_export_user    | bkmkcsv_export_url1
     301 | userexportit_full_user | userexportit_fic_b
```

`user_export_api` and `bookmark_csv_api` leave bookmarks whose `url_id` has no
`fic_info` row. `rec_curator` then fails:

```
refresh signals: Database("insert or update on table \"rec_user_signals\"
violates foreign key constraint \"rec_user_signals_work_id_fkey\"")
```

`rec_curator` did nothing wrong. It seeded its five `fic_info` rows correctly
and its bookmarks all resolve. It is the victim of two other suites' leftovers,
and of production code that cannot tolerate the state its own schema permits.

This is the same class of finding as `embedding_dedupe` and the roadmap
consensus panel: **a constraint somewhere that the code never accounted for,
which only surfaces once something actually runs.**

## 3. Why this is production, not test

Three reasons, and the third is decisive.

1. `bookmarks.url_id` has no FK, so orphans are a legal database state, not a
   corruption. They arise whenever a `fic_info` row is removed without
   cascading to `bookmarks` - which the schema permits and which is exactly what
   the two test suites did.
2. The recommender calls `refresh_signals` on the live path. Any user whose
   bookmark set contains one orphan takes down the signal refresh for
   **every** user, because the function starts with `DELETE FROM
   rec_user_signals` and then rebuilds the whole table in one statement.
3. A defensive `EXISTS` in the query costs a semi-join on an indexed column. A
   dropped recommender costs the product its recommendations.

The alternative - adding an FK from `bookmarks.url_id` to `fic_info(id)` - is
rejected deliberately. That changes the schema, and existing databases already
contain orphans, so the migration would fail without a repair step. It also
conflates two different identities: `bookmarks.work_id` points at `works(id)`
while `bookmarks.url_id` points at a `fic_info` slug, and a bookmark legitimately
has the former without the latter.

## 4. Requirements

**R1.** `refresh_signals` must not fail on data its own schema permits. The
bookmark insert joins `fic_info`; the rating insert does the same.

**R2.** Filtering must be by existence, not by a pattern or a test-only prefix.
A join to the referenced table expresses the actual requirement.

**R3.** The unit and schema invariants must still hold: the number of
`rec_user_signals` rows equals the number of qualifying signals that *resolve*,
which is the semantics the recommender already assumes.

**R4.** Add a regression test that creates an orphaned bookmark and asserts
`refresh_signals` succeeds. Without it, this regresses the next time someone
tidies the query.

**R5.** Do not change the schema in this cycle, and do not change
`refresh_signals`' signature or return type.

## 5. Non-goals

- Making `user_export_api` and `bookmark_csv_api` clean up after themselves.
  They arguably should, but that is separate, and this fix must not depend on
  them.
- Adding the missing foreign key. See §3.
- Deciding whether an orphaned bookmark should be deleted rather than ignored.
  Ignoring it is correct for now; deleting user data is a product decision.

## 6. Acceptance

1. With the three orphaned bookmarks still in the database,
   `rec_curator` passes.
2. `refresh_signals` called against a database containing an orphaned bookmark
   returns `Ok` — demonstrated, not assumed.
3. A non-orphaned bookmark still produces a `rec_user_signals` row, so the fix
   did not silently disable the signal source. This is the assertion that would
   catch a wrong fix.
4. 935 unit tests pass; all 56 suites compile.
