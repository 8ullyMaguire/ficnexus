-- 069 — Wayback fallback telemetry
--
-- Track when a scrape was served from the Internet Archive Wayback Machine
-- so curators/readers know the source may be slightly stale.

ALTER TABLE scrape_failures
    ADD COLUMN IF NOT EXISTS fallback_source TEXT;
