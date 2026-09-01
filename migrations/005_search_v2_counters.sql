-- Search v2: precomputed works counters + metadata columns.
--
-- Previously the builder counted kudos/comments/bookmarks via live JOIN
-- subqueries over social tables and mapped language/beta nowhere useful.
-- These denormalised columns let search filter and sort on engagement
-- directly, and give language / beta / publish-date fields a real target.
ALTER TABLE works
    ADD COLUMN kudos_count     INT4 NOT NULL DEFAULT 0,
    ADD COLUMN comments_count  INT4 NOT NULL DEFAULT 0,
    ADD COLUMN bookmarks_count INT4 NOT NULL DEFAULT 0,
    ADD COLUMN hit_count       INT4 NOT NULL DEFAULT 0,
    ADD COLUMN language_code   TEXT NOT NULL DEFAULT 'en',
    ADD COLUMN beta_status     TEXT NOT NULL DEFAULT 'unknown',
    ADD COLUMN first_published TIMESTAMPTZ,
    ADD COLUMN last_updated    TIMESTAMPTZ;
