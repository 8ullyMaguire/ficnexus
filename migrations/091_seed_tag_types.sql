-- 091_seed_tag_types.sql
--
-- tag_types is declared in 001_initial.sql as
--   id smallint NOT NULL, name text NOT NULL
-- with a hand-assigned id and no sequence, and no migration ever inserted a
-- row. The table is therefore empty on any database built from migrations/,
-- and tags.tag_type_id is a NOT NULL foreign key onto it, so no tag can be
-- created.
--
-- The contents are defined by the repository's own specification,
-- docs/design/SPECIFICATION.md:428:
--   1 fandom, 2 character, 3 relationship, 4 freeform,
--   5 warning, 6 category, 7 rating
--
-- This is a shipped-feature gap, not a test-fixture problem:
-- src/routes/opds/tags.rs:41 serves this table as an OPDS resource and :104
-- resolves tag type names by join.
--
-- Measured on 2026-09-26: with the table empty, 25 of 56 DB-gated integration
-- suites pass. Seeding exactly these 7 rows, changing nothing else, takes it to
-- 32 of 56.
--
-- Idempotent: keyed on the primary key, so re-running is a no-op.

INSERT INTO tag_types (id, name) VALUES
  (1, 'fandom'),
  (2, 'character'),
  (3, 'relationship'),
  (4, 'freeform'),
  (5, 'warning'),
  (6, 'category'),
  (7, 'rating')
ON CONFLICT (id) DO NOTHING;
