# search_api: two bugs. One unimplemented feature, one self-contradictory test

Date: 2026-09-27 · Status: diagnosed, fixing
Suite: `tests/search_api.rs` — 37 passed, 4 failed

## The four failures are two problems, not four

### Group 1 — `main_char_attr` is not a parameter this API has (3 tests)

The tests send:

    /api/search?main_char_attr=SearchTest%20Harry%20Potter|SearchTest%20Dark%20Harry%20Potter

Two filters joined by `|`: a main character, **and** an attribute (freeform
tag). The wire struct has no such field:

    // src/search/routes.rs:24
    pub struct SearchQueryParams {
        pub q: Option<String>,
        …
        /// Main/primary character filter (Guide tier) — maps to a character tag
        /// with role_confidence >= 1.0 (`@char:Name`).
        pub main_char: Option<String>,
        …
    }

`SearchQueryParams` derives `Deserialize` **without** `deny_unknown_fields`, so
`main_char_attr` is dropped silently and the search runs **unfiltered**. Every
fic in the database comes back, which is why all three tests fail on their
*negative* assertion — "fic where Harry is secondary must NOT match" — while the
positive one passes.

The pipeline filter that the tests want is expressible today as
`main_char=<name>&include_tags=7:<attribute>`, but that is two parameters, not
one `|`-joined value.

**Decision: implement the parameter.** The tests describe a coherent feature and
`main_char_attr` is named in three places in `src/` as if it exists
(`src/bin/backfill_scores.rs`, `src/tags/backfill.rs`, `src/roadmap_seed.rs`) —
it was designed, documented, and never built. Deleting the tests would discard
the specification.

The `role_confidence` half of this turned out to be a **second, independent bug**
— see below — so the feature was doubly non-functional.

### Group 2 — `search_rating_filter` contradicts itself (1 test)

    seed_fic(&db, "searchit_rt_a", "Rating Alpha", "Shared topic rating.", …)
    seed_fic(&db, "searchit_rt_b", "Rating Beta",   "Shared topic rating.", …)
    let body = get_search("/api/search?q=rating&rating=SearchTest%20General%20Audiences").await;
    assert!(!ids.contains(&"searchit_rt_b"), "fic B has no rating tag: {body}");

`q=rating` matches **both** fics — "Rating" is in both titles and both bodies
contain the word "rating". The `rating=` filter is working correctly; the text
query is what puts fic B in the result set. The test's own docstring says
"`rating=General Audiences` returns only the rated fic", which is not what a
combined `q=` + `rating=` request means.

The `rating` path itself is correct and deliberate: it resolves the name to a
`tag_type_id = 7` tag and appends it to `include_tags` (routes.rs:559-586), and
the next assertion in the same test documents that an unresolvable rating is
silently ignored by design.

**Decision: fix the test** — drop the `q=rating` term so the request is
`rating=` alone, which is what the docstring describes. The product is right.

## A second real bug, found while tracing Group 1: `role_confidence` was never maintained

`main_char` filters on `fic_tags.role_confidence >= 1.0`
(`src/search/builder.rs`, `ROLE_MAIN_CONFIDENCE`). That column was materialised
from `score` **once**, by migration 004:

    ALTER TABLE fic_tags ADD COLUMN role_confidence REAL NOT NULL DEFAULT 1.0;
    UPDATE fic_tags SET role_confidence = CASE
        WHEN score >= 10 THEN 1.0 WHEN score > 0 THEN 0.5 ELSE 0.0 END;

`score` is written continuously by scrape-time scoring, and **nothing ever
wrote `role_confidence` again**:

    $ grep -rn 'role_confidence' src/ | grep -Ei 'insert|update'
    (no matches)

The only DML naming the column in the entire tree is that migration. And the
column default is `1.0`, so **every newly inserted row is born "main"** — which
means the `main_char` filter matched every fic containing the character in any
role. The filter was a no-op.

**Fixed by making the column generated**, so `score` is the single source of
truth and no code path can set one without the other:

    ALTER TABLE fic_tags DROP COLUMN role_confidence;
    ALTER TABLE fic_tags ADD COLUMN role_confidence REAL
        GENERATED ALWAYS AS (
            CASE WHEN score >= 10 THEN 1.0 WHEN score > 0 THEN 0.5 ELSE 0.0 END
        ) STORED;

Migration `095_role_confidence_generated.sql`. A stored generated column cannot
be converted in place, hence drop + re-add; there is no index on the column, so
nothing needs rebuilding. Verified the construct works on this server
(PostgreSQL 18.6) before writing it.

Two comments that would have sent the next reader to the wrong file are also
corrected: `builder.rs` pointed at "migration 003" for the backfill (it is 004),
and `src/tags/backfill.rs` still described `main_char_attr` as reading `score`
directly.

## A third bug, found by removing the confound: `rating=` was a no-op

Once the `q=rating` term was dropped, fic B **still** came back — with **zero**
rating tags. The diagnostic is what settled it:

    DIAG ids=["searchit_rt_a","searchit_rt_b"]
         rating_tags=[("searchit_rt_a", 1), ("searchit_rt_b", 0)]  total=2

The rating handler resolved the name and pushed it onto `include_tags`:

    search_params.include_tags.push(TagFilter { tag_type_id: 7, … });

…but the builder is constructed from **`capped_params`**, a clone of
`search_params` taken *earlier* in the same function:

    let mut capped_params = SearchParams { per_page: …, ..search_params.clone() };

so the pushed filter never reached the query. `rating=` was silently a no-op,
and the original test only ever passed its *positive* assertion because
`q=rating` had put fic A in the results for an unrelated reason.

Fixed by moving the resolution above the clone and mutating `capped_params`, so
it is also in place before the `expanded_include_tag_ids` resolution.

**Worth naming as a pattern:** the test was not weak, it was *confounded*. It
asserted the right thing in a request whose text query masked the broken
filter. Dropping the confound is what exposed the bug — and would not have been
done had the test passed by accident.

## Order of work

`role_confidence` first — it is a migration and it had to be right before
`main_char_attr` could be tested. Then the parameter. Then the rating test.

Note the three `main_char_attr` tests are not expected to pass on the strength of
the migration alone: with the parameter still unrecognised, they are unfiltered
and still fail. Both changes are needed.
