-- Saved searches + daily alerts.
--
-- A user saves a search (a name + the raw query text + the JSON AST of the
-- parsed boolean query). They can rerun it on demand (the /api/search/saved
-- run endpoint) or opt it into nightly alerting (alert_mode = 'rss'), which
-- a watcher bin (saved_search_watcher) re-runs each night, diffs the current
-- match set against what has already been seen, and records newly-seen works
-- in saved_search_matches so they can be surfaced in a public per-search Atom
-- feed at /feed/saved/{user_id}/{search_id}.

CREATE TABLE IF NOT EXISTS public.saved_searches (
    id BIGSERIAL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    name text NOT NULL,
    query_text text NOT NULL,
    query_json jsonb NOT NULL,
    -- 'none' = saved but not alerted; 'rss' = watcher tracks new matches.
    alert_mode text NOT NULL DEFAULT 'none'
        CHECK (alert_mode IN ('none', 'rss')),
    last_run_at timestamptz,
    last_match_count integer,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT saved_searches_user_name_key UNIQUE (user_id, name)
);

-- Newly-seen works for an alerting saved search. First-seen tracks when the
-- watcher first noticed the work; notified_at is reserved for future
-- notification delivery (currently the Atom feed reads first_seen).
CREATE TABLE IF NOT EXISTS public.saved_search_matches (
    search_id bigint NOT NULL REFERENCES public.saved_searches(id) ON DELETE CASCADE,
    work_id integer NOT NULL REFERENCES public.works(id) ON DELETE CASCADE,
    first_seen timestamptz NOT NULL DEFAULT now(),
    notified_at timestamptz,
    PRIMARY KEY (search_id, work_id)
);

-- Fast per-user listing.
CREATE INDEX IF NOT EXISTS idx_saved_searches_user
    ON public.saved_searches (user_id);

-- Watcher queries only alerting searches.
CREATE INDEX IF NOT EXISTS idx_saved_searches_alert_mode
    ON public.saved_searches (alert_mode)
    WHERE alert_mode <> 'none';
