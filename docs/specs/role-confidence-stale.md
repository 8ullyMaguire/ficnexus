# role_confidence is a stale copy of score: main-character search never worked

Date: 2026-09-27 · Status: diagnosed, fixing
Suite: `tests/search_api.rs` — 37 passed, 4 failed (3 of them one bug)

## The symptom

Three tests fail on the *negative* half of a main-character filter:

    // Fic B: Harry is SECONDARY (score 1, Ron is main at 10) -> must NOT match
    seed_fic_tag(&db, "searchit_mca_side", ron, 10).await;
    seed_fic_tag(&db, "searchit_mca_side", harry, 1).await;
    ...
    assert!(!ids.contains(&"searchit_mca_side"),
            "fic where Harry is secondary must NOT match: {body}");

A fic where the character is *secondary* is returned as a main-character match.
The feature appears to ignore the distinction it exists to express.

## Root cause: two columns, one source of truth, and nothing keeps them in sync

Search filters on `fic_tags.role_confidence`:

    // src/search/builder.rs
    const ROLE_MAIN_CONFIDENCE: f32 = 1.0;
    …
    qb.push(" AND ft.role_confidence >= ");
    qb.push_bind(ROLE_MAIN_CONFIDENCE);

`role_confidence` was **materialised from `score` once**, in migration 004:

    ALTER TABLE fic_tags ADD COLUMN role_confidence REAL NOT NULL DEFAULT 1.0;
    UPDATE fic_tags SET role_confidence = CASE
        WHEN score >= 10 THEN 1.0
        WHEN score > 0 THEN 0.5
        ELSE 0.0
    END;

That is a one-time backfill of a value that is supposed to track `score`. Then
`score` kept being written — by scrape-time scoring (main character 10,
secondary 1, primary ship 5, other ships 1, per `src/tags/backfill.rs`) — and
`role_confidence` never followed. Its column default is `1.0`, so **every new
row is born "main"**.

Verified rather than inferred:

    $ grep -rn 'role_confidence' src/ | grep -Ei 'insert|update'
    (no matches)

Zero writes in the entire codebase outside the migration. The only other
mention is `src/tags/backfill.rs`'s module doc, which still describes
`main_char_attr` as working off **`score`** — it was written before the switch to
`role_confidence` and was never updated.

The builder's own comment is also wrong:

    /// `fic_tags.role_confidence` threshold … The backfill in migration 003
    /// maps score>=10 → 1.0

Migration 003 is `search_v2_polarity` (relationship polarity). The role
confidence backfill is migration **004**. Small, but it is the kind of wrong
pointer that sends the next reader to the wrong file.

## Why this is a product bug and the tests are right

The three tests encode exactly the behaviour the feature promises: a character
who is secondary in a fic is not that fic's main character. They seed `score`,
which is the documented convention in `src/tags/backfill.rs`. The tests are
correct; the column they depend on is not maintained.

**Consequence in production:** any `main_char_attr=` or `@char:` filter returns
every fic containing the character in any role. The filter is a no-op.

## The fix: make the column generated, so it cannot drift

Rather than adding a trigger or remembering to update two columns everywhere:

    ALTER TABLE fic_tags
        DROP COLUMN role_confidence,
        ADD COLUMN role_confidence REAL
            GENERATED ALWAYS AS (
                CASE WHEN score >= 10 THEN 1.0
                     WHEN score > 0  THEN 0.5
                     ELSE 0.0 END
            ) STORED;

`score` becomes the single source of truth, the mapping lives in the schema
next to the data, and there is no code path that can set one without the other.

Verified the construct works on this server before writing the migration:

    $ psql -c "CREATE TEMP TABLE t(score smallint);
               ALTER TABLE t ADD COLUMN gen real GENERATED ALWAYS AS
                 (CASE WHEN score>=10 THEN 1.0 WHEN score>0 THEN 0.5 ELSE 0.0 END) STORED;"
    $ SELECT score, gen FROM t ORDER BY score DESC;
     10|1
      1|0.5
      0|0
    PostgreSQL 18.6

Note this requires dropping and re-adding the column, because a stored generated
column cannot be converted in place. The search builder's index on
`role_confidence` would need rebuilding too — checked, and there is none, so the
query is currently a sequential scan inside the `EXISTS` subquery either way.

## Also correcting

The builder's doc comment pointing at migration 003, and
`src/tags/backfill.rs`'s module doc which describes `main_char_attr` as reading
`score`. Both will be right after this change; both are wrong now, and both are
how a future reader concludes the search path works off `score` and "fixes"
something that is not broken.
