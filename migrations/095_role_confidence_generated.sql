-- role_confidence becomes a generated column, so it cannot drift from score.
--
-- 004_search_v2_role_confidence.sql materialised `role_confidence` from `score`
-- with a one-time UPDATE, intending it to track the column:
--
--   score >= 10 -> 1.0 (main/primary)   score > 0 -> 0.5   else 0.0
--
-- It never tracked it. `score` is written continuously by scrape-time scoring
-- (main character 10, secondary 1, primary ship 5, other ships 1 - see
-- src/tags/backfill.rs), and nothing ever wrote role_confidence again: the only
-- DML naming the column in the whole tree is the UPDATE above. Worse, the
-- column default is 1.0, so every newly inserted row is born "main".
--
-- Search filters on exactly that column (src/search/builder.rs,
-- ROLE_MAIN_CONFIDENCE = 1.0), so `main_char_attr=` and `@char:` filters
-- matched every fic containing the character in *any* role. The filter was a
-- no-op. Three tests in search_api caught it by asserting that a fic where the
-- character is secondary must not match.
--
-- Fix: make score the single source of truth and express the mapping in the
-- schema, so no code path can set one without the other. A stored generated
-- column cannot be converted in place, hence drop + re-add.
--
-- There is no index on role_confidence, so nothing needs rebuilding here; the
-- EXISTS subquery is a scan either way.
--
-- Analysis: docs/specs/role-confidence-stale.md

ALTER TABLE fic_tags
    DROP COLUMN role_confidence;

ALTER TABLE fic_tags
    ADD COLUMN role_confidence REAL
        GENERATED ALWAYS AS (
            CASE
                WHEN score >= 10 THEN 1.0
                WHEN score > 0 THEN 0.5
                ELSE 0.0
            END
        ) STORED;
