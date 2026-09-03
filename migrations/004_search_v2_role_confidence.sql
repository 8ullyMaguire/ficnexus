-- Search v2: per-fic tag role confidence.
--
-- `score` is the legacy priority/primary weight on a fic's tag
-- (10 = main/primary). To let the builder express "main character" /
-- "@fandom" (primary) filters, we materialise a floating role confidence that
-- downstream agents can fill in later too. Backfill maps the legacy scores:
--   score >= 10          -> 1.0  (main/primary role)
--   score > 0            -> 0.5  (secondary / supporting)
--   score else (0 or < 0)-> 0.0
ALTER TABLE fic_tags
    ADD COLUMN role_confidence REAL NOT NULL DEFAULT 1.0;

UPDATE fic_tags SET role_confidence = CASE
    WHEN score >= 10 THEN 1.0
    WHEN score > 0 THEN 0.5
    ELSE 0.0
END;
