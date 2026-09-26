-- 092_seed_locales.sql
--
-- locales is declared in 001_initial.sql and is referenced by four tables via
-- FOREIGN KEY (locale_code) REFERENCES locales(code):
--   work_translations, chapter_translations, chapter_translation_versions
--   (and at least one further referencing table).
--
-- No migration ever inserted a row, so on any database built purely from
-- migrations/ the table is empty and every one of those inserts fails with a
-- foreign key violation. src/db/queries/social.rs:656 reads
--   SELECT id, code, name, is_rtl FROM locales ORDER BY id
-- to populate a locale picker, and always gets nothing.
--
-- The set below is the authoritative one, taken from the executable frontend
-- rather than from prose: frontend/src/lib/i18n/dictionaries/index.ts registers
-- exactly six dictionaries - en, de, es, fr, pt-BR, zh - and
-- frontend/src/lib/i18n/index.svelte.ts treats those as LOCALES with 'en' as
-- DEFAULT_LOCALE.
--
-- Note that docs/design/frontend-design.md:291 claims "15 languages". That
-- does not match the code: 6 dictionary modules exist. The code is treated as
-- authoritative, and the doc is stale. Recorded rather than silently ignored.
--
-- All six are left-to-right, so is_rtl is false for every row. The column drives
-- text direction in the UI, so it is set explicitly rather than defaulted.
--
-- 'en' is additionally required by users.locale, which is NOT NULL DEFAULT 'en'.
-- 'es' is required by tests/translation_review_api.rs.
--
-- Idempotent: keyed on the unique code constraint, so re-running is a no-op.

INSERT INTO locales (code, name, is_rtl) VALUES
  ('en',    'English',   false),
  ('de',    'Deutsch',   false),
  ('es',    'Espanol',   false),
  ('fr',    'Francais',  false),
  ('pt-BR', 'Portugues (Brasil)', false),
  ('zh',    'Chinese',   false)
ON CONFLICT (code) DO NOTHING;
