-- Search v2: relationship polarity.
--
-- AO3 relationship tags encode polarity in the name separator:
--   `A/B`  -> romantic  (literal slash)
--   `A & B`-> platonic  (literal ampersand)
--
-- This generated column classifies every tag once, so the search builder can
-- filter relationships by polarity with a plain index seek instead of a
-- per-row LIKE scan:
--   1 = romantic (name contains '/'), 2 = platonic (name contains '&'),
--   0 = a relationship tag with neither literal (e.g. "Gen", "Other"),
--   NULL = not a relationship tag (tag_type_id != 3).
ALTER TABLE tags
    ADD COLUMN rel_polarity SMALLINT GENERATED ALWAYS AS (
        CASE
            WHEN tag_type_id = 3 AND name LIKE '%/%' THEN 1
            WHEN tag_type_id = 3 AND name LIKE '%&%' THEN 2
            WHEN tag_type_id = 3 THEN 0
            ELSE NULL
        END
    ) STORED;

CREATE INDEX idx_tags_rel_polarity
    ON tags (tag_type_id, rel_polarity)
    WHERE tag_type_id = 3;
