-- 001_initial: consolidated schema (generated from live prod dump)
-- sqlx: no-transaction

--
--


SET idle_in_transaction_session_timeout = 0;
SET standard_conforming_strings = on;

--
-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;


--
-- Name: vector; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;


--
-- Name: update_fic_tag_score(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_fic_tag_score() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fic_tags SET score = score + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' AND NEW.value <> OLD.value THEN
        UPDATE fic_tags SET score = score - OLD.value + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fic_tags SET score = score - OLD.value
        WHERE url_id = OLD.url_id AND tag_id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--


--
-- Name: admin_daily_stats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.admin_daily_stats (
    date date NOT NULL,
    total_users integer DEFAULT 0 NOT NULL,
    new_users integer DEFAULT 0 NOT NULL,
    total_works integer DEFAULT 0 NOT NULL,
    new_works integer DEFAULT 0 NOT NULL,
    manual_uploads integer DEFAULT 0 NOT NULL,
    epubs_downloaded bigint DEFAULT 0 NOT NULL,
    words_read bigint DEFAULT 0 NOT NULL
);


--
-- Name: agent_runs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.agent_runs (
    id bigint NOT NULL,
    trigger_type text NOT NULL,
    trigger_ref bigint,
    class text NOT NULL,
    model text,
    iterations integer DEFAULT 0 NOT NULL,
    tokens integer DEFAULT 0 NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    diff_summary text,
    tests_passed boolean,
    merged boolean DEFAULT false,
    deployed boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    finished_at timestamp with time zone,
    CONSTRAINT agent_runs_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'diagnosed'::text, 'proposed'::text, 'merged'::text, 'deployed'::text, 'failed'::text, 'paused'::text])))
);


--
-- Name: agent_runs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.agent_runs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: agent_runs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.agent_runs_id_seq OWNED BY public.agent_runs.id;


--
-- Name: arena_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.arena_votes (
    id integer NOT NULL,
    user_id integer,
    client_id character varying(36),
    cluster_ids integer[] NOT NULL,
    best_cluster_id integer,
    worst_cluster_id integer,
    created_at timestamp with time zone DEFAULT now()
);


--
-- Name: arena_votes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.arena_votes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: arena_votes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.arena_votes_id_seq OWNED BY public.arena_votes.id;


--
-- Name: ask_translation_cache; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ask_translation_cache (
    nl_query text NOT NULL,
    params jsonb NOT NULL,
    model text NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: author_blacklist; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.author_blacklist (
    source_id bigint NOT NULL,
    author_id bigint NOT NULL,
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    reason integer DEFAULT 1 NOT NULL
);


--
-- Name: author_merge_proposals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.author_merge_proposals (
    id integer NOT NULL,
    source_author text NOT NULL,
    source_url text NOT NULL,
    target_profile_id integer NOT NULL,
    proposed_by integer,
    approved_by integer,
    status text DEFAULT 'pending'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    resolved_at timestamp with time zone,
    CONSTRAINT author_merge_proposals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text])))
);


--
-- Name: author_merge_proposals_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.author_merge_proposals_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: author_merge_proposals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.author_merge_proposals_id_seq OWNED BY public.author_merge_proposals.id;


--
-- Name: author_profile_links; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.author_profile_links (
    id integer NOT NULL,
    profile_id integer NOT NULL,
    source_author text NOT NULL,
    source_url text NOT NULL,
    source_id integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: author_profile_links_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.author_profile_links_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: author_profile_links_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.author_profile_links_id_seq OWNED BY public.author_profile_links.id;


--
-- Name: author_profiles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.author_profiles (
    id integer NOT NULL,
    canonical_name text NOT NULL,
    bio text DEFAULT ''::text,
    avatar_url text DEFAULT ''::text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: author_profiles_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.author_profiles_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: author_profiles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.author_profiles_id_seq OWNED BY public.author_profiles.id;


--
-- Name: author_socials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.author_socials (
    id integer NOT NULL,
    profile_id integer NOT NULL,
    platform text NOT NULL,
    url text NOT NULL,
    label text DEFAULT ''::text,
    is_visible boolean DEFAULT true NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL
);


--
-- Name: author_socials_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.author_socials_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: author_socials_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.author_socials_id_seq OWNED BY public.author_socials.id;


--
-- Name: auto_merge_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.auto_merge_log (
    id integer NOT NULL,
    source_url text NOT NULL,
    matched_work_id integer NOT NULL,
    confidence double precision NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);


--
-- Name: auto_merge_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.auto_merge_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: auto_merge_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.auto_merge_log_id_seq OWNED BY public.auto_merge_log.id;


--
-- Name: badge_definitions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.badge_definitions (
    badge_type text NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    icon text DEFAULT '🏆'::text,
    category text DEFAULT 'general'::text NOT NULL,
    threshold integer NOT NULL,
    event_type text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: blocked_users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.blocked_users (
    user_id integer NOT NULL,
    blocked_user_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: bookmarks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.bookmarks (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    url_id character varying(128) NOT NULL,
    work_id integer,
    notes text DEFAULT ''::text,
    is_private boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: bookmarks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.bookmarks_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: bookmarks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.bookmarks_id_seq OWNED BY public.bookmarks.id;


--
-- Name: bot_scores; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.bot_scores (
    ip inet NOT NULL,
    client_id character varying(36) NOT NULL,
    window_start timestamp with time zone NOT NULL,
    requests integer DEFAULT 0 NOT NULL,
    downloads integer DEFAULT 0 NOT NULL,
    failed_auths integer DEFAULT 0 NOT NULL,
    export_ratio double precision DEFAULT 0 NOT NULL
);


--
-- Name: challenge_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.challenge_assignments (
    id integer NOT NULL,
    challenge_id integer NOT NULL,
    giver_id integer NOT NULL,
    recipient_id integer NOT NULL,
    work_id integer,
    claimed boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: challenge_assignments_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.challenge_assignments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: challenge_assignments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.challenge_assignments_id_seq OWNED BY public.challenge_assignments.id;


--
-- Name: challenge_signups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.challenge_signups (
    id integer NOT NULL,
    challenge_id integer NOT NULL,
    user_id integer NOT NULL,
    offer_tags text DEFAULT ''::text NOT NULL,
    request_tags text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: challenge_signups_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.challenge_signups_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: challenge_signups_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.challenge_signups_id_seq OWNED BY public.challenge_signups.id;


--
-- Name: challenges; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.challenges (
    id integer NOT NULL,
    collection_id integer NOT NULL,
    signup_open boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: challenges_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.challenges_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: challenges_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.challenges_id_seq OWNED BY public.challenges.id;


--
-- Name: chapter_translation_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.chapter_translation_versions (
    id bigint NOT NULL,
    work_id integer NOT NULL,
    locale_code text NOT NULL,
    version integer NOT NULL,
    chapters jsonb DEFAULT '[]'::jsonb NOT NULL,
    translated_by integer,
    status text DEFAULT 'draft'::text NOT NULL,
    reviewed_by integer,
    reviewed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chapter_translation_versions_status_check CHECK ((status = ANY (ARRAY['draft'::text, 'approved'::text, 'rejected'::text])))
);


--
-- Name: chapter_translation_versions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.chapter_translation_versions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: chapter_translation_versions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.chapter_translation_versions_id_seq OWNED BY public.chapter_translation_versions.id;


--
-- Name: chapter_translations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.chapter_translations (
    id bigint NOT NULL,
    work_id integer NOT NULL,
    locale_code text NOT NULL,
    chapters jsonb DEFAULT '[]'::jsonb NOT NULL,
    translated_by integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    reviewed_by integer,
    reviewed_at timestamp with time zone,
    CONSTRAINT chapter_translations_status_check CHECK ((status = ANY (ARRAY['draft'::text, 'approved'::text, 'rejected'::text])))
);


--
-- Name: chapter_translations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.chapter_translations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: chapter_translations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.chapter_translations_id_seq OWNED BY public.chapter_translations.id;


--
-- Name: collection_bookmarks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.collection_bookmarks (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    list_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: collection_bookmarks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.collection_bookmarks_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: collection_bookmarks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.collection_bookmarks_id_seq OWNED BY public.collection_bookmarks.id;


--
-- Name: collection_item_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.collection_item_requests (
    id bigint NOT NULL,
    list_id integer NOT NULL,
    work_id integer NOT NULL,
    requested_by integer NOT NULL,
    blurb text DEFAULT ''::text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    reviewed_at timestamp with time zone,
    CONSTRAINT collection_item_requests_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text])))
);


--
-- Name: collection_item_requests_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.collection_item_requests_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: collection_item_requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.collection_item_requests_id_seq OWNED BY public.collection_item_requests.id;


--
-- Name: comment_triage; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comment_triage (
    id bigint NOT NULL,
    comment_id integer,
    category text NOT NULL,
    reason text DEFAULT ''::text NOT NULL,
    confidence real DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: comment_triage_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.comment_triage_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: comment_triage_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.comment_triage_id_seq OWNED BY public.comment_triage.id;


--
-- Name: comments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comments (
    id bigint NOT NULL,
    url_id character varying(128) NOT NULL,
    user_id integer,
    parent_id bigint,
    body text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    is_hidden boolean DEFAULT false NOT NULL,
    work_id integer,
    constructive boolean DEFAULT true NOT NULL
);


--
-- Name: comments_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.comments_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: comments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.comments_id_seq OWNED BY public.comments.id;


--
-- Name: content_scan; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.content_scan (
    id bigint NOT NULL,
    url_id text NOT NULL,
    scanned_at timestamp with time zone DEFAULT now() NOT NULL,
    classification text DEFAULT 'unclear'::text NOT NULL,
    confidence real DEFAULT 0 NOT NULL,
    detected_warnings text DEFAULT ''::text NOT NULL,
    reason text DEFAULT ''::text NOT NULL,
    review_status text DEFAULT 'pending'::text NOT NULL,
    reviewed_by integer,
    reviewed_at timestamp with time zone,
    CONSTRAINT content_scan_review_status_check CHECK ((review_status = ANY (ARRAY['pending'::text, 'confirmed'::text, 'dismissed'::text])))
);


--
-- Name: content_scan_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.content_scan_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: content_scan_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.content_scan_id_seq OWNED BY public.content_scan.id;


--
-- Name: creatorships; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.creatorships (
    id integer NOT NULL,
    pseud_id integer NOT NULL,
    work_id integer,
    series_id integer,
    approved boolean DEFAULT false NOT NULL,
    invited_at timestamp with time zone DEFAULT now() NOT NULL,
    approved_at timestamp with time zone,
    CONSTRAINT creatorships_check CHECK (((work_id IS NOT NULL) OR (series_id IS NOT NULL)))
);


--
-- Name: creatorships_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.creatorships_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: creatorships_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.creatorships_id_seq OWNED BY public.creatorships.id;


--
-- Name: curator_content_overrides; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.curator_content_overrides (
    url_id text NOT NULL,
    body_html text NOT NULL,
    chapters integer DEFAULT 1 NOT NULL,
    title text,
    description text,
    notes text,
    created_by integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: curator_fix_proposals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.curator_fix_proposals (
    id bigint NOT NULL,
    url_id text NOT NULL,
    body_html text NOT NULL,
    reason text DEFAULT ''::text NOT NULL,
    proposed_by integer NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    upvotes integer DEFAULT 0 NOT NULL,
    downvotes integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    decided_at timestamp with time zone,
    decided_by integer,
    CONSTRAINT curator_fix_proposals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text, 'applied'::text])))
);


--
-- Name: curator_fix_proposals_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.curator_fix_proposals_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: curator_fix_proposals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.curator_fix_proposals_id_seq OWNED BY public.curator_fix_proposals.id;


--
-- Name: curator_fix_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.curator_fix_votes (
    proposal_id bigint NOT NULL,
    user_id integer NOT NULL,
    vote smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT curator_fix_votes_vote_check CHECK ((vote = ANY (ARRAY[1, '-1'::integer])))
);


--
-- Name: curator_metadata_proposals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.curator_metadata_proposals (
    id bigint NOT NULL,
    work_id integer NOT NULL,
    url_id text DEFAULT ''::text NOT NULL,
    field text NOT NULL,
    old_value text DEFAULT ''::text NOT NULL,
    new_value text DEFAULT ''::text NOT NULL,
    reason text DEFAULT ''::text NOT NULL,
    proposed_by integer NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    upvotes integer DEFAULT 0 NOT NULL,
    downvotes integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    decided_at timestamp with time zone,
    decided_by integer,
    CONSTRAINT curator_metadata_proposals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text, 'applied'::text])))
);


--
-- Name: curator_metadata_proposals_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.curator_metadata_proposals_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: curator_metadata_proposals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.curator_metadata_proposals_id_seq OWNED BY public.curator_metadata_proposals.id;


--
-- Name: curator_metadata_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.curator_metadata_votes (
    proposal_id bigint NOT NULL,
    user_id integer NOT NULL,
    vote smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT curator_metadata_votes_vote_check CHECK ((vote = ANY (ARRAY[1, '-1'::integer])))
);


--
-- Name: daily_quests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.daily_quests (
    id integer NOT NULL,
    quest_type text NOT NULL,
    title text NOT NULL,
    description text NOT NULL,
    target_count integer NOT NULL,
    reward_xp integer NOT NULL,
    badge_type text,
    active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: daily_quests_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.daily_quests_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: daily_quests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.daily_quests_id_seq OWNED BY public.daily_quests.id;


--
-- Name: doc_sections; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.doc_sections (
    id bigint NOT NULL,
    slug text NOT NULL,
    page text NOT NULL,
    anchor text,
    title text NOT NULL,
    body text NOT NULL,
    keywords text[] DEFAULT '{}'::text[] NOT NULL,
    feature text DEFAULT 'home'::text NOT NULL,
    embedding public.vector(768)
);


--
-- Name: doc_sections_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.doc_sections_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: doc_sections_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.doc_sections_id_seq OWNED BY public.doc_sections.id;


--
-- Name: exp_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.exp_events (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    amount integer NOT NULL,
    event_type text NOT NULL,
    reference_type text,
    reference_id bigint,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: exp_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.exp_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: exp_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.exp_events_id_seq OWNED BY public.exp_events.id;


--
-- Name: export_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.export_log (
    url_id character varying(128),
    version integer NOT NULL,
    etype text NOT NULL,
    input_hash text NOT NULL,
    export_hash text NOT NULL,
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: extensions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.extensions (
    id integer NOT NULL,
    kind text NOT NULL,
    slug text NOT NULL,
    name text NOT NULL,
    description text,
    author_id integer,
    version integer DEFAULT 1 NOT NULL,
    tier text DEFAULT 'config'::text NOT NULL,
    gate_level integer DEFAULT 0 NOT NULL,
    payload jsonb DEFAULT '{}'::jsonb NOT NULL,
    stats jsonb DEFAULT '{"rating": 0, "installs": 0}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT extensions_kind_check CHECK ((kind = ANY (ARRAY['recipe'::text, 'theme'::text, 'layout'::text, 'view'::text]))),
    CONSTRAINT extensions_tier_check CHECK ((tier = ANY (ARRAY['config'::text, 'recipe'::text, 'service'::text, 'wasm'::text])))
);


--
-- Name: extensions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.extensions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: extensions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.extensions_id_seq OWNED BY public.extensions.id;


--
-- Name: feature_clusters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.feature_clusters (
    id integer NOT NULL,
    representative_text text NOT NULL,
    embedding public.vector(768) NOT NULL,
    elo_rating real DEFAULT 1500.0,
    matches_played integer DEFAULT 0,
    times_picked_best integer DEFAULT 0,
    times_picked_worst integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT now(),
    status text DEFAULT 'open'::text,
    CONSTRAINT feature_clusters_status_check CHECK ((status = ANY (ARRAY['open'::text, 'shipped'::text, 'rejected'::text, 'deferred'::text])))
);


--
-- Name: feature_clusters_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.feature_clusters_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: feature_clusters_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.feature_clusters_id_seq OWNED BY public.feature_clusters.id;


--
-- Name: feature_suggestions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.feature_suggestions (
    id integer NOT NULL,
    user_id integer,
    client_id character varying(36),
    raw_text text NOT NULL,
    cluster_id integer,
    created_at timestamp with time zone DEFAULT now()
);


--
-- Name: feature_suggestions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.feature_suggestions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: feature_suggestions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.feature_suggestions_id_seq OWNED BY public.feature_suggestions.id;


--
-- Name: features; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.features (
    id integer NOT NULL,
    slug text NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    long_help text DEFAULT ''::text NOT NULL,
    icon text,
    category text NOT NULL,
    gate_type text DEFAULT 'rank'::text NOT NULL,
    gate_value integer DEFAULT 0 NOT NULL,
    requires_feature text,
    is_default boolean DEFAULT false NOT NULL,
    is_revocable boolean DEFAULT true NOT NULL,
    admin_only boolean DEFAULT false NOT NULL,
    widget_component text,
    nav_target text,
    sort_hint integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: features_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.features_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: features_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.features_id_seq OWNED BY public.features.id;


--
-- Name: fic_blacklist; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_blacklist (
    url_id character varying(128),
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    reason integer DEFAULT 1 NOT NULL
);


--
-- Name: fic_bookmark_cooccur; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_bookmark_cooccur (
    work_a character varying(128) NOT NULL,
    work_b character varying(128) NOT NULL,
    site_domain character varying(255) NOT NULL,
    cooccur_count integer DEFAULT 1 NOT NULL,
    last_updated timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fic_bookmark_cooccur_check CHECK (((work_a)::text < (work_b)::text))
);


--
-- Name: fic_bookmarks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_bookmarks (
    user_hash character varying(64) NOT NULL,
    url_id character varying(128) NOT NULL,
    site_domain character varying(255) NOT NULL,
    first_seen timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: fic_info; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_info (
    id character varying(128) NOT NULL,
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    title text NOT NULL,
    author text NOT NULL,
    author_url text,
    author_local_id text,
    chapters integer NOT NULL,
    words bigint NOT NULL,
    description text NOT NULL,
    fic_created timestamp with time zone NOT NULL,
    fic_updated timestamp with time zone NOT NULL,
    status text NOT NULL,
    source text NOT NULL,
    extra_meta text,
    raw_extended_meta text,
    source_id bigint,
    author_id bigint,
    content_hash character varying(256),
    raw_json jsonb,
    work_id integer,
    text_search tsvector GENERATED ALWAYS AS ((setweight(to_tsvector('english'::regconfig, COALESCE(title, ''::text)), 'A'::"char") || setweight(to_tsvector('english'::regconfig, COALESCE(description, ''::text)), 'B'::"char"))) STORED,
    source_type text DEFAULT 'scraper'::text NOT NULL,
    body_text_search tsvector
);


--
-- Name: fic_request_answer_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_request_answer_votes (
    answer_id integer NOT NULL,
    user_id integer NOT NULL,
    vote smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT fic_request_answer_votes_vote_check CHECK ((vote = ANY (ARRAY['-1'::integer, 1])))
);


--
-- Name: fic_request_answers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_request_answers (
    id integer NOT NULL,
    request_id integer NOT NULL,
    user_id integer NOT NULL,
    work_id integer,
    pitch text DEFAULT ''::text NOT NULL,
    source text DEFAULT 'user'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    answer_kind text DEFAULT 'work'::text NOT NULL,
    search_query text
);


--
-- Name: fic_request_answers_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.fic_request_answers_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: fic_request_answers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.fic_request_answers_id_seq OWNED BY public.fic_request_answers.id;


--
-- Name: fic_request_upvotes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_request_upvotes (
    request_id integer NOT NULL,
    user_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: fic_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_requests (
    id integer NOT NULL,
    user_id integer NOT NULL,
    title text NOT NULL,
    body text DEFAULT ''::text NOT NULL,
    seed_work_id integer,
    status text DEFAULT 'open'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    closed_at timestamp with time zone,
    deleted_at timestamp with time zone,
    accepted_answer_id integer,
    CONSTRAINT fic_requests_status_check CHECK ((status = ANY (ARRAY['open'::text, 'answered'::text, 'closed'::text])))
);


--
-- Name: fic_requests_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.fic_requests_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: fic_requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.fic_requests_id_seq OWNED BY public.fic_requests.id;


--
-- Name: fic_tag_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_tag_votes (
    url_id character varying(128) NOT NULL,
    tag_id integer NOT NULL,
    voter_ip inet NOT NULL,
    value smallint NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fic_tag_votes_value_check CHECK ((value = ANY (ARRAY['-1'::integer, 1])))
);


--
-- Name: fic_tags; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_tags (
    url_id character varying(128) NOT NULL,
    tag_id integer NOT NULL,
    added_by_ip inet DEFAULT '0.0.0.0'::inet NOT NULL,
    score smallint DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    is_machine_suggested boolean DEFAULT false NOT NULL,
    reviewed_at timestamp with time zone
);


--
-- Name: fic_version_bump; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_version_bump (
    id character varying(128) NOT NULL,
    value integer
);


--
-- Name: fic_works; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fic_works (
    url_id character varying(128) NOT NULL,
    site_domain character varying(255) NOT NULL,
    site_work_id character varying(255) NOT NULL,
    favouriter_count integer DEFAULT 0 NOT NULL,
    first_favourite_scraped timestamp with time zone,
    last_favourite_scraped timestamp with time zone,
    last_cooccur_update timestamp with time zone
);


--
-- Name: follows; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.follows (
    id bigint NOT NULL,
    follower_id integer NOT NULL,
    followee_id integer,
    work_id integer,
    author_name text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_seen timestamp with time zone,
    CONSTRAINT follows_target CHECK ((num_nonnulls(followee_id, work_id, author_name) = 1))
);


--
-- Name: follows_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.follows_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: follows_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.follows_id_seq OWNED BY public.follows.id;


--
-- Name: forum_bans; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_bans (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    category_id bigint,
    reason text DEFAULT ''::text NOT NULL,
    banned_by integer NOT NULL,
    expires_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: forum_bans_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_bans_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_bans_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_bans_id_seq OWNED BY public.forum_bans.id;


--
-- Name: forum_categories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_categories (
    id bigint NOT NULL,
    slug text NOT NULL,
    title text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    "position" integer DEFAULT 0 NOT NULL,
    is_mod_only boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: forum_categories_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_categories_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_categories_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_categories_id_seq OWNED BY public.forum_categories.id;


--
-- Name: forum_edit_proposals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_edit_proposals (
    id bigint NOT NULL,
    target_type text NOT NULL,
    target_id bigint NOT NULL,
    author_id integer NOT NULL,
    snapshot jsonb NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    reviewed_by integer,
    reviewed_at timestamp with time zone,
    review_note text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT forum_edit_proposals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text]))),
    CONSTRAINT forum_edit_proposals_target_type_check CHECK ((target_type = ANY (ARRAY['topic'::text, 'post'::text])))
);


--
-- Name: forum_edit_proposals_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_edit_proposals_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_edit_proposals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_edit_proposals_id_seq OWNED BY public.forum_edit_proposals.id;


--
-- Name: forum_follows; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_follows (
    user_id integer NOT NULL,
    topic_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: forum_metamod_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_metamod_votes (
    mod_action_id bigint NOT NULL,
    voter_id integer NOT NULL,
    verdict smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT forum_metamod_votes_verdict_check CHECK ((verdict = ANY (ARRAY[0, 1, 2])))
);


--
-- Name: forum_mod_actions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_mod_actions (
    id bigint NOT NULL,
    post_id bigint NOT NULL,
    moderator_id integer NOT NULL,
    reason text NOT NULL,
    delta smallint NOT NULL,
    score_after integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: forum_mod_actions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_mod_actions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_mod_actions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_mod_actions_id_seq OWNED BY public.forum_mod_actions.id;


--
-- Name: forum_mod_grants; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_mod_grants (
    user_id integer NOT NULL,
    points_left smallint DEFAULT 0 NOT NULL,
    granted_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    cooldown_until timestamp with time zone
);


--
-- Name: forum_post_reactions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_post_reactions (
    post_id bigint NOT NULL,
    user_id integer NOT NULL,
    emoji character varying(8) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: forum_posts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_posts (
    id bigint NOT NULL,
    topic_id bigint NOT NULL,
    author_id integer NOT NULL,
    body text NOT NULL,
    payload jsonb DEFAULT '{}'::jsonb NOT NULL,
    quote_of bigint,
    edited_at timestamp with time zone,
    deleted_at timestamp with time zone,
    is_hidden boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    search_vector tsvector,
    score integer DEFAULT 0 NOT NULL,
    mod_count integer DEFAULT 0 NOT NULL,
    hidden_until timestamp with time zone
);


--
-- Name: forum_posts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_posts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_posts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_posts_id_seq OWNED BY public.forum_posts.id;


--
-- Name: forum_read_state; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_read_state (
    user_id integer NOT NULL,
    topic_id bigint NOT NULL,
    last_read_post_id bigint,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: forum_topic_views; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_topic_views (
    id bigint NOT NULL,
    topic_id bigint NOT NULL,
    user_id integer,
    viewed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: forum_topic_views_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_topic_views_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_topic_views_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_topic_views_id_seq OWNED BY public.forum_topic_views.id;


--
-- Name: forum_topics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_topics (
    id bigint NOT NULL,
    category_id bigint NOT NULL,
    author_id integer NOT NULL,
    title text NOT NULL,
    body text NOT NULL,
    payload jsonb DEFAULT '{}'::jsonb NOT NULL,
    status text DEFAULT 'open'::text NOT NULL,
    view_count bigint DEFAULT 0 NOT NULL,
    last_post_id bigint,
    last_activity_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    is_hidden boolean DEFAULT false NOT NULL,
    search_vector tsvector,
    topic_slug text,
    CONSTRAINT forum_topics_status_check CHECK ((status = ANY (ARRAY['open'::text, 'locked'::text, 'pinned'::text, 'archived'::text])))
);


--
-- Name: forum_topics_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_topics_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_topics_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_topics_id_seq OWNED BY public.forum_topics.id;


--
-- Name: heal_extractions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.heal_extractions (
    id bigint NOT NULL,
    failure_id bigint,
    agent_run_id bigint,
    url text NOT NULL,
    url_id text,
    title text,
    author text,
    chapters integer,
    words bigint,
    description text,
    status text,
    validated boolean DEFAULT false NOT NULL,
    trusted boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: heal_extractions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.heal_extractions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: heal_extractions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.heal_extractions_id_seq OWNED BY public.heal_extractions.id;


--
-- Name: kudos; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.kudos (
    id bigint NOT NULL,
    work_id integer NOT NULL,
    user_id integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: kudos_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.kudos_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: kudos_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.kudos_id_seq OWNED BY public.kudos.id;


--
-- Name: leaderboard_monthly; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leaderboard_monthly (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    score integer DEFAULT 0 NOT NULL,
    rank smallint NOT NULL,
    month_start date NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: leaderboard_monthly_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.leaderboard_monthly_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: leaderboard_monthly_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.leaderboard_monthly_id_seq OWNED BY public.leaderboard_monthly.id;


--
-- Name: leaderboard_weekly; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leaderboard_weekly (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    score integer DEFAULT 0 NOT NULL,
    rank smallint NOT NULL,
    week_start date NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: leaderboard_weekly_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.leaderboard_weekly_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: leaderboard_weekly_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.leaderboard_weekly_id_seq OWNED BY public.leaderboard_weekly.id;


--
-- Name: locales; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.locales (
    id integer NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    is_rtl boolean DEFAULT false NOT NULL
);


--
-- Name: locales_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.locales_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: locales_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.locales_id_seq OWNED BY public.locales.id;


--
-- Name: login_streaks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.login_streaks (
    user_id integer NOT NULL,
    current_streak integer DEFAULT 0 NOT NULL,
    longest_streak integer DEFAULT 0 NOT NULL,
    last_login_date date DEFAULT CURRENT_DATE NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: marginalia; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.marginalia (
    id bigint NOT NULL,
    work_id integer NOT NULL,
    chapter_index integer NOT NULL,
    passage_hash text NOT NULL,
    topic_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    passage_text text
);


--
-- Name: marginalia_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.marginalia_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: marginalia_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.marginalia_id_seq OWNED BY public.marginalia.id;


--
-- Name: modlog; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.modlog (
    id bigint NOT NULL,
    actor_id integer,
    actor_username text,
    action text NOT NULL,
    target_type text DEFAULT ''::text NOT NULL,
    target_id text DEFAULT ''::text NOT NULL,
    details jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: modlog_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.modlog_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: modlog_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.modlog_id_seq OWNED BY public.modlog.id;


--
-- Name: notification_preferences; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notification_preferences (
    user_id integer NOT NULL,
    comment_reply boolean DEFAULT true NOT NULL,
    follow_update boolean DEFAULT true NOT NULL,
    work_update boolean DEFAULT true NOT NULL,
    badge_earned boolean DEFAULT true NOT NULL,
    curator_promotion boolean DEFAULT true NOT NULL,
    recommendation boolean DEFAULT false NOT NULL,
    email_digest text DEFAULT 'never'::text NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT notification_preferences_email_digest_check CHECK ((email_digest = ANY (ARRAY['instant'::text, 'daily'::text, 'weekly'::text, 'never'::text])))
);


--
-- Name: notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notifications (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    notification_type text NOT NULL,
    title text NOT NULL,
    body text,
    link text,
    reference_type text,
    reference_id text,
    is_read boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: notifications_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.notifications_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: notifications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.notifications_id_seq OWNED BY public.notifications.id;


--
-- Name: opds_shelf_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.opds_shelf_items (
    shelf_id integer NOT NULL,
    url_id character varying(128) NOT NULL,
    added_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: opds_shelves; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.opds_shelves (
    id integer NOT NULL,
    name text NOT NULL,
    description text,
    token text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: opds_shelves_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.opds_shelves_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: opds_shelves_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.opds_shelves_id_seq OWNED BY public.opds_shelves.id;


--
-- Name: pending_exports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pending_exports (
    id bigint NOT NULL,
    url text NOT NULL,
    format text DEFAULT 'epub'::text NOT NULL,
    client_ip inet,
    client_id text,
    status text DEFAULT 'pending'::text NOT NULL,
    attempts integer DEFAULT 0 NOT NULL,
    error_kind text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone,
    CONSTRAINT pending_exports_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'completed'::text, 'failed'::text, 'cancelled'::text])))
);


--
-- Name: pending_exports_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.pending_exports_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pending_exports_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.pending_exports_id_seq OWNED BY public.pending_exports.id;


--
-- Name: precomputed_recommendations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.precomputed_recommendations (
    url_id character varying(128) NOT NULL,
    recommended_url_id character varying(128) NOT NULL,
    score real NOT NULL,
    rank smallint NOT NULL,
    computed_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: pseuds; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pseuds (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    is_default boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: pseuds_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.pseuds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: pseuds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.pseuds_id_seq OWNED BY public.pseuds.id;


--
-- Name: reactions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reactions (
    target_type text NOT NULL,
    target_id bigint NOT NULL,
    user_id integer NOT NULL,
    emoji character varying(8) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT reactions_target_type_check CHECK ((target_type = ANY (ARRAY['work'::text, 'comment'::text, 'chapter'::text])))
);


--
-- Name: reading_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reading_history (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    work_id integer NOT NULL,
    visited_at timestamp with time zone DEFAULT now() NOT NULL,
    chapter_num integer DEFAULT 1
);


--
-- Name: reading_history_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.reading_history_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: reading_history_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.reading_history_id_seq OWNED BY public.reading_history.id;


--
-- Name: reading_list_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reading_list_items (
    id integer NOT NULL,
    list_id integer NOT NULL,
    work_id integer NOT NULL,
    "position" integer DEFAULT 0 NOT NULL,
    blurb text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: reading_list_items_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.reading_list_items_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: reading_list_items_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.reading_list_items_id_seq OWNED BY public.reading_list_items.id;


--
-- Name: reading_lists; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reading_lists (
    id integer NOT NULL,
    user_id integer NOT NULL,
    title text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    is_public boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    collection_kind text DEFAULT 'list'::text NOT NULL,
    visibility text DEFAULT 'private'::text NOT NULL,
    item_selection text DEFAULT 'restricted'::text NOT NULL,
    icon text,
    slug text,
    CONSTRAINT reading_lists_collection_kind_check CHECK ((collection_kind = ANY (ARRAY['list'::text, 'collection'::text]))),
    CONSTRAINT reading_lists_item_selection_check CHECK ((item_selection = ANY (ARRAY['moderated'::text, 'restricted'::text]))),
    CONSTRAINT reading_lists_visibility_check CHECK ((visibility = ANY (ARRAY['public'::text, 'private'::text, 'anonymous'::text])))
);


--
-- Name: reading_lists_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.reading_lists_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: reading_lists_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.reading_lists_id_seq OWNED BY public.reading_lists.id;


--
-- Name: reading_stats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reading_stats (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    work_id integer NOT NULL,
    words_read bigint DEFAULT 0 NOT NULL,
    last_read_at timestamp with time zone DEFAULT now() NOT NULL,
    read_count integer DEFAULT 1 NOT NULL,
    status text DEFAULT 'reading'::text NOT NULL,
    current_chapter integer,
    CONSTRAINT reading_stats_status_check CHECK ((status = ANY (ARRAY['want_to_read'::text, 'reading'::text, 'completed'::text, 'dropped'::text])))
);


--
-- Name: reading_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.reading_stats_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: reading_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.reading_stats_id_seq OWNED BY public.reading_stats.id;


--
-- Name: rec_author_graph; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_author_graph (
    author_a text NOT NULL,
    author_b text NOT NULL,
    cooccur_count integer DEFAULT 0 NOT NULL,
    tag_jaccard double precision DEFAULT 0 NOT NULL,
    CONSTRAINT rec_author_graph_check CHECK ((author_a < author_b))
);


--
-- Name: rec_bandit_arms; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_bandit_arms (
    work_id character varying(128) NOT NULL,
    strategy text DEFAULT 'bandit'::text NOT NULL,
    alpha double precision DEFAULT 1.0 NOT NULL,
    beta double precision DEFAULT 1.0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: rec_embeddings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_embeddings (
    work_id character varying(128) NOT NULL,
    model text NOT NULL,
    embedding public.vector(384) NOT NULL,
    content_hash text NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: rec_impressions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_impressions (
    id bigint NOT NULL,
    user_id integer,
    work_id character varying(128) NOT NULL,
    strategy text NOT NULL,
    shown_at timestamp with time zone DEFAULT now() NOT NULL,
    engaged_at timestamp with time zone
);


--
-- Name: rec_impressions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rec_impressions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rec_impressions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rec_impressions_id_seq OWNED BY public.rec_impressions.id;


--
-- Name: rec_models; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_models (
    id bigint NOT NULL,
    name text NOT NULL,
    version text DEFAULT '1'::text NOT NULL,
    params_path text,
    trained_at timestamp with time zone DEFAULT now() NOT NULL,
    metrics jsonb DEFAULT '{}'::jsonb NOT NULL
);


--
-- Name: rec_models_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rec_models_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rec_models_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rec_models_id_seq OWNED BY public.rec_models.id;


--
-- Name: rec_training_runs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_training_runs (
    id bigint NOT NULL,
    strategy text NOT NULL,
    started_at timestamp with time zone DEFAULT now() NOT NULL,
    duration_ms bigint DEFAULT 0 NOT NULL,
    metrics jsonb DEFAULT '{}'::jsonb NOT NULL,
    ok boolean DEFAULT true NOT NULL
);


--
-- Name: rec_training_runs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.rec_training_runs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: rec_training_runs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.rec_training_runs_id_seq OWNED BY public.rec_training_runs.id;


--
-- Name: rec_transitions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_transitions (
    from_work character varying(128) NOT NULL,
    to_work character varying(128) NOT NULL,
    weight double precision DEFAULT 1.0 NOT NULL,
    last_seen timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: rec_user_clusters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_user_clusters (
    user_id integer NOT NULL,
    cluster_id integer NOT NULL,
    affinity jsonb DEFAULT '{}'::jsonb NOT NULL,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: rec_user_curator_align; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_user_curator_align (
    user_id integer NOT NULL,
    alignment double precision NOT NULL,
    n_signals integer DEFAULT 0 NOT NULL,
    alpha double precision DEFAULT 1.0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: rec_user_signals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.rec_user_signals (
    user_id integer NOT NULL,
    work_id character varying(128) NOT NULL,
    signal_type text NOT NULL,
    signal_weight double precision NOT NULL,
    occurred_at timestamp with time zone NOT NULL
);


--
-- Name: recommendation_suggestions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.recommendation_suggestions (
    id bigint NOT NULL,
    url_id character varying(128) NOT NULL,
    suggested_url_id character varying(128) NOT NULL,
    submitted_by_ip inet,
    comment text,
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    user_id integer,
    scraped_url text,
    status text DEFAULT 'active'::text NOT NULL,
    created_by text DEFAULT 'legacy'::text NOT NULL,
    CONSTRAINT recommendation_suggestions_created_by_check CHECK ((created_by = ANY (ARRAY['user'::text, 'legacy'::text]))),
    CONSTRAINT recommendation_suggestions_status_check CHECK ((status = ANY (ARRAY['active'::text, 'removed'::text])))
);


--
-- Name: recommendation_suggestions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.recommendation_suggestions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: recommendation_suggestions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.recommendation_suggestions_id_seq OWNED BY public.recommendation_suggestions.id;


--
-- Name: recommendation_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.recommendation_votes (
    suggestion_id bigint NOT NULL,
    voter_ip inet,
    vote smallint NOT NULL,
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    user_id integer,
    CONSTRAINT recommendation_votes_actor_check CHECK (((((user_id IS NOT NULL))::integer + ((voter_ip IS NOT NULL))::integer) = 1)),
    CONSTRAINT recommendation_votes_vote_check CHECK ((vote = ANY (ARRAY['-1'::integer, 1])))
);


--
-- Name: registration_applications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.registration_applications (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    reason text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    reviewed_by integer,
    reviewed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT registration_applications_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text])))
);


--
-- Name: registration_applications_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.registration_applications_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: registration_applications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.registration_applications_id_seq OWNED BY public.registration_applications.id;


--
-- Name: reputation_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reputation_events (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    event_type text NOT NULL,
    points integer NOT NULL,
    reference_type text,
    reference_id text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: reputation_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.reputation_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: reputation_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.reputation_events_id_seq OWNED BY public.reputation_events.id;


--
-- Name: request_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.request_log (
    id bigint NOT NULL,
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    source_id bigint,
    etype text NOT NULL,
    query text NOT NULL,
    info_request_ms integer NOT NULL,
    url_id text,
    fic_info text,
    export_ms integer,
    export_file_name text,
    export_file_hash text,
    url text,
    client_id character varying(36),
    user_agent text,
    source_url text,
    status text DEFAULT 'success'::text NOT NULL,
    error_message text,
    ip inet,
    CONSTRAINT request_log_status_check CHECK ((status = ANY (ARRAY['success'::text, 'error'::text])))
);


--
-- Name: request_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.request_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: request_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.request_log_id_seq OWNED BY public.request_log.id;


--
-- Name: request_source; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.request_source (
    id bigint NOT NULL,
    created timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    is_automated boolean DEFAULT false,
    route text,
    description text
);


--
-- Name: request_source_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.request_source_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: request_source_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.request_source_id_seq OWNED BY public.request_source.id;


--
-- Name: reviews; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reviews (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    work_id integer NOT NULL,
    url_id character varying(128),
    rating smallint NOT NULL,
    title character varying(200),
    body text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    constructive boolean DEFAULT true NOT NULL,
    CONSTRAINT reviews_rating_check CHECK ((rating = ANY (ARRAY[1, 2, 3, 4, 5])))
);


--
-- Name: reviews_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.reviews_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: reviews_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.reviews_id_seq OWNED BY public.reviews.id;


--
-- Name: schema_migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.schema_migrations (
    version bigint NOT NULL,
    dirty boolean NOT NULL
);


--
-- Name: scrape_failures; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.scrape_failures (
    id bigint NOT NULL,
    url text NOT NULL,
    url_id text,
    domain text NOT NULL,
    error_kind text NOT NULL,
    message text,
    html_snapshot_path text,
    fingerprint text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    resolved_at timestamp with time zone,
    resolution text,
    CONSTRAINT scrape_failures_error_kind_check CHECK ((error_kind = ANY (ARRAY['blocked'::text, 'timeout'::text, 'parse'::text, 'not_found'::text, 'export'::text, 'unknown'::text])))
);


--
-- Name: scrape_failures_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.scrape_failures_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: scrape_failures_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.scrape_failures_id_seq OWNED BY public.scrape_failures.id;


--
-- Name: search_queries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.search_queries (
    id bigint NOT NULL,
    query text NOT NULL,
    ts timestamp with time zone DEFAULT now() NOT NULL,
    total_results integer DEFAULT 0 NOT NULL,
    main_char_attr text,
    client_id character varying(36),
    user_id integer
);


--
-- Name: search_queries_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.search_queries_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: search_queries_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.search_queries_id_seq OWNED BY public.search_queries.id;


--
-- Name: series; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.series (
    id integer NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    user_id integer,
    pseud_id integer
);


--
-- Name: series_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.series_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: series_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.series_id_seq OWNED BY public.series.id;


--
-- Name: series_works; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.series_works (
    series_id integer NOT NULL,
    work_id integer NOT NULL,
    "position" integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: shelves; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.shelves (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text,
    is_public boolean DEFAULT false NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: shelves_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.shelves_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: shelves_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.shelves_id_seq OWNED BY public.shelves.id;


--
-- Name: site_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.site_settings (
    key text NOT NULL,
    value text NOT NULL
);


--
-- Name: skins; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.skins (
    id integer NOT NULL,
    user_id integer NOT NULL,
    title text NOT NULL,
    css text DEFAULT ''::text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    is_work_skin boolean DEFAULT false NOT NULL,
    is_public boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: skins_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.skins_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: skins_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.skins_id_seq OWNED BY public.skins.id;


--
-- Name: tag_aliases; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag_aliases (
    alias_name text NOT NULL COLLATE pg_catalog."C",
    canonical_tag_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: tag_embeddings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag_embeddings (
    tag_id integer NOT NULL,
    embedding public.vector(768) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: tag_flags; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag_flags (
    id bigint NOT NULL,
    url_id character varying(128) NOT NULL,
    tag_id integer NOT NULL,
    flagged_by_ip inet NOT NULL,
    reason text,
    resolved boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: tag_flags_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tag_flags_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tag_flags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tag_flags_id_seq OWNED BY public.tag_flags.id;


--
-- Name: tag_score_fixes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag_score_fixes (
    id bigint NOT NULL,
    url_id character varying(128) NOT NULL,
    tag_id integer NOT NULL,
    old_score smallint NOT NULL,
    new_score smallint NOT NULL,
    fixed_by integer,
    fixed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tag_score_fixes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tag_score_fixes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tag_score_fixes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tag_score_fixes_id_seq OWNED BY public.tag_score_fixes.id;


--
-- Name: tag_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag_types (
    id smallint NOT NULL,
    name text NOT NULL
);


--
-- Name: tags; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tags (
    id integer NOT NULL,
    name text NOT NULL COLLATE pg_catalog."C",
    tag_type_id smallint NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: tags_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tags_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tags_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tags_id_seq OWNED BY public.tags.id;


--
-- Name: translations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.translations (
    id bigint NOT NULL,
    locale_code text NOT NULL,
    namespace text NOT NULL,
    key text NOT NULL,
    value text NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: translations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.translations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: translations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.translations_id_seq OWNED BY public.translations.id;


--
-- Name: usage_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.usage_events (
    id bigint NOT NULL,
    client_id text NOT NULL,
    path text NOT NULL,
    event_type text DEFAULT 'view'::text NOT NULL,
    user_agent text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT usage_events_event_type_check CHECK ((event_type = ANY (ARRAY['view'::text, 'action'::text])))
);


--
-- Name: usage_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.usage_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: usage_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.usage_events_id_seq OWNED BY public.usage_events.id;


--
-- Name: user_badges; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_badges (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    badge_type text NOT NULL,
    earned_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_badges_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_badges_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_badges_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_badges_id_seq OWNED BY public.user_badges.id;


--
-- Name: user_daily_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_daily_progress (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    quest_id integer NOT NULL,
    progress integer DEFAULT 0 NOT NULL,
    completed boolean DEFAULT false NOT NULL,
    completed_at timestamp with time zone,
    quest_date date DEFAULT CURRENT_DATE NOT NULL
);


--
-- Name: user_daily_progress_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_daily_progress_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_daily_progress_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_daily_progress_id_seq OWNED BY public.user_daily_progress.id;


--
-- Name: user_features; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_features (
    user_id integer NOT NULL,
    feature_id integer NOT NULL,
    unlocked_at timestamp with time zone,
    enabled boolean DEFAULT false NOT NULL,
    enabled_at timestamp with time zone,
    pinned boolean DEFAULT false NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL
);


--
-- Name: user_invites; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_invites (
    id bigint NOT NULL,
    code text NOT NULL,
    created_by integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone,
    used_by integer,
    used_at timestamp with time zone
);


--
-- Name: user_invites_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_invites_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_invites_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_invites_id_seq OWNED BY public.user_invites.id;


--
-- Name: user_layouts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_layouts (
    user_id integer NOT NULL,
    page text NOT NULL,
    layout jsonb NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_prefs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_prefs (
    user_id integer NOT NULL,
    key text NOT NULL,
    value jsonb NOT NULL
);


--
-- Name: user_recipes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_recipes (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    description text,
    blend jsonb DEFAULT '{}'::jsonb NOT NULL,
    filters jsonb DEFAULT '{}'::jsonb NOT NULL,
    boost jsonb DEFAULT '{}'::jsonb NOT NULL,
    curator_prior real DEFAULT 0.1 NOT NULL,
    is_active boolean DEFAULT false NOT NULL,
    is_public boolean DEFAULT false NOT NULL,
    installs integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_recipes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_recipes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_recipes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_recipes_id_seq OWNED BY public.user_recipes.id;


--
-- Name: user_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_reports (
    id bigint NOT NULL,
    reporter_id integer,
    target_type text NOT NULL,
    target_id integer NOT NULL,
    reason text NOT NULL,
    details jsonb,
    status text DEFAULT 'open'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT user_reports_status_check CHECK ((status = ANY (ARRAY['open'::text, 'resolved'::text, 'dismissed'::text]))),
    CONSTRAINT user_reports_target_type_check CHECK ((target_type = ANY (ARRAY['comment'::text, 'work'::text, 'user'::text, 'forum_topic'::text, 'forum_post'::text])))
);


--
-- Name: user_reports_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_reports_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_reports_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_reports_id_seq OWNED BY public.user_reports.id;


--
-- Name: user_site_credentials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_site_credentials (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    domain text NOT NULL,
    username text NOT NULL,
    password_enc text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL
);


--
-- Name: user_site_credentials_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_site_credentials_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_site_credentials_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_site_credentials_id_seq OWNED BY public.user_site_credentials.id;


--
-- Name: user_skins; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_skins (
    user_id integer NOT NULL,
    skin_id integer,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_views; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_views (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    query jsonb NOT NULL,
    pinned boolean DEFAULT false NOT NULL
);


--
-- Name: user_views_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_views_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_views_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_views_id_seq OWNED BY public.user_views.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id integer NOT NULL,
    username text NOT NULL,
    password_hash text NOT NULL,
    email text,
    role smallint DEFAULT 0 NOT NULL,
    reputation integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    curator_since timestamp with time zone,
    curator_status text DEFAULT 'active'::text,
    total_words_read bigint DEFAULT 0 NOT NULL,
    total_works_read integer DEFAULT 0 NOT NULL,
    last_active_at timestamp with time zone,
    locale text DEFAULT 'en'::text NOT NULL,
    bio text DEFAULT ''::text,
    is_banned boolean DEFAULT false NOT NULL,
    kindle_email text,
    level smallint DEFAULT 0 NOT NULL,
    exp bigint DEFAULT 0 NOT NULL,
    xp integer DEFAULT 0 NOT NULL,
    rank integer DEFAULT 1 NOT NULL,
    trust integer DEFAULT 0 NOT NULL,
    show_marginalia boolean DEFAULT true,
    CONSTRAINT users_curator_status_check CHECK ((curator_status = ANY (ARRAY['active'::text, 'suspended'::text])))
);


--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: work_proposal_votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_proposal_votes (
    proposal_id integer NOT NULL,
    user_id integer NOT NULL,
    vote smallint NOT NULL,
    voted_at timestamp with time zone DEFAULT now(),
    CONSTRAINT work_proposal_votes_vote_check CHECK ((vote = ANY (ARRAY['-1'::integer, 0, 1])))
);


--
-- Name: work_proposals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_proposals (
    id integer NOT NULL,
    proposer_id integer NOT NULL,
    action_type text NOT NULL,
    source_work_id integer,
    target_work_id integer,
    work_id integer,
    details jsonb,
    status text DEFAULT 'pending'::text,
    created_at timestamp with time zone DEFAULT now(),
    closed_at timestamp with time zone,
    CONSTRAINT work_proposals_action_type_check CHECK ((action_type = ANY (ARRAY['merge'::text, 'split'::text]))),
    CONSTRAINT work_proposals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'accepted'::text, 'rejected'::text])))
);


--
-- Name: work_proposals_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.work_proposals_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: work_proposals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.work_proposals_id_seq OWNED BY public.work_proposals.id;


--
-- Name: work_rating_verifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_rating_verifications (
    id bigint NOT NULL,
    work_id integer NOT NULL,
    rating text,
    warnings jsonb DEFAULT '[]'::jsonb NOT NULL,
    verified_by integer,
    verified_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: work_rating_verifications_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.work_rating_verifications_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: work_rating_verifications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.work_rating_verifications_id_seq OWNED BY public.work_rating_verifications.id;


--
-- Name: work_ratings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_ratings (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    url_id character varying(128) NOT NULL,
    work_id integer,
    rating smallint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT work_ratings_rating_check CHECK (((rating = ANY (ARRAY[1, 2, 3, 4, 5])) OR (rating = '-1'::integer)))
);


--
-- Name: work_ratings_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.work_ratings_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: work_ratings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.work_ratings_id_seq OWNED BY public.work_ratings.id;


--
-- Name: work_shelves; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_shelves (
    id bigint NOT NULL,
    shelf_id integer NOT NULL,
    work_id integer NOT NULL,
    added_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: work_shelves_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.work_shelves_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: work_shelves_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.work_shelves_id_seq OWNED BY public.work_shelves.id;


--
-- Name: work_skins; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_skins (
    work_id integer NOT NULL,
    skin_id integer NOT NULL
);


--
-- Name: work_translations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_translations (
    id bigint NOT NULL,
    work_id integer NOT NULL,
    locale_code text NOT NULL,
    title text,
    summary text,
    translated_by integer,
    translated_at timestamp with time zone DEFAULT now() NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    reviewed_by integer,
    reviewed_at timestamp with time zone,
    CONSTRAINT work_translations_status_check CHECK ((status = ANY (ARRAY['draft'::text, 'approved'::text, 'rejected'::text])))
);


--
-- Name: work_translations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.work_translations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: work_translations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.work_translations_id_seq OWNED BY public.work_translations.id;


--
-- Name: works; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.works (
    id integer NOT NULL,
    canonical_title text DEFAULT ''::text NOT NULL,
    canonical_author text DEFAULT ''::text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    default_source_id text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    embedding public.vector(384),
    embedding_updated_at timestamp with time zone,
    uploader_id integer,
    is_visible boolean DEFAULT true NOT NULL,
    rating_verified_at timestamp with time zone
);


--
-- Name: works_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.works_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: works_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.works_id_seq OWNED BY public.works.id;


--
-- Name: xp_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.xp_events (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    event_type text NOT NULL,
    xp integer NOT NULL,
    source_ref text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: xp_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.xp_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: xp_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.xp_events_id_seq OWNED BY public.xp_events.id;


--
-- Name: xp_source_defs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.xp_source_defs (
    event_type text NOT NULL,
    xp_amount integer NOT NULL,
    daily_cap integer,
    description text
);


--
-- Name: agent_runs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.agent_runs ALTER COLUMN id SET DEFAULT nextval('public.agent_runs_id_seq'::regclass);


--
-- Name: arena_votes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.arena_votes ALTER COLUMN id SET DEFAULT nextval('public.arena_votes_id_seq'::regclass);


--
-- Name: author_merge_proposals id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_merge_proposals ALTER COLUMN id SET DEFAULT nextval('public.author_merge_proposals_id_seq'::regclass);


--
-- Name: author_profile_links id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_profile_links ALTER COLUMN id SET DEFAULT nextval('public.author_profile_links_id_seq'::regclass);


--
-- Name: author_profiles id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_profiles ALTER COLUMN id SET DEFAULT nextval('public.author_profiles_id_seq'::regclass);


--
-- Name: author_socials id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_socials ALTER COLUMN id SET DEFAULT nextval('public.author_socials_id_seq'::regclass);


--
-- Name: auto_merge_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.auto_merge_log ALTER COLUMN id SET DEFAULT nextval('public.auto_merge_log_id_seq'::regclass);


--
-- Name: bookmarks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bookmarks ALTER COLUMN id SET DEFAULT nextval('public.bookmarks_id_seq'::regclass);


--
-- Name: challenge_assignments id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_assignments ALTER COLUMN id SET DEFAULT nextval('public.challenge_assignments_id_seq'::regclass);


--
-- Name: challenge_signups id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_signups ALTER COLUMN id SET DEFAULT nextval('public.challenge_signups_id_seq'::regclass);


--
-- Name: challenges id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenges ALTER COLUMN id SET DEFAULT nextval('public.challenges_id_seq'::regclass);


--
-- Name: chapter_translation_versions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translation_versions ALTER COLUMN id SET DEFAULT nextval('public.chapter_translation_versions_id_seq'::regclass);


--
-- Name: chapter_translations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translations ALTER COLUMN id SET DEFAULT nextval('public.chapter_translations_id_seq'::regclass);


--
-- Name: collection_bookmarks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_bookmarks ALTER COLUMN id SET DEFAULT nextval('public.collection_bookmarks_id_seq'::regclass);


--
-- Name: collection_item_requests id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_requests ALTER COLUMN id SET DEFAULT nextval('public.collection_item_requests_id_seq'::regclass);


--
-- Name: comment_triage id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_triage ALTER COLUMN id SET DEFAULT nextval('public.comment_triage_id_seq'::regclass);


--
-- Name: comments id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments ALTER COLUMN id SET DEFAULT nextval('public.comments_id_seq'::regclass);


--
-- Name: content_scan id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_scan ALTER COLUMN id SET DEFAULT nextval('public.content_scan_id_seq'::regclass);


--
-- Name: creatorships id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.creatorships ALTER COLUMN id SET DEFAULT nextval('public.creatorships_id_seq'::regclass);


--
-- Name: curator_fix_proposals id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_fix_proposals ALTER COLUMN id SET DEFAULT nextval('public.curator_fix_proposals_id_seq'::regclass);


--
-- Name: curator_metadata_proposals id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_proposals ALTER COLUMN id SET DEFAULT nextval('public.curator_metadata_proposals_id_seq'::regclass);


--
-- Name: daily_quests id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_quests ALTER COLUMN id SET DEFAULT nextval('public.daily_quests_id_seq'::regclass);


--
-- Name: doc_sections id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.doc_sections ALTER COLUMN id SET DEFAULT nextval('public.doc_sections_id_seq'::regclass);


--
-- Name: exp_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.exp_events ALTER COLUMN id SET DEFAULT nextval('public.exp_events_id_seq'::regclass);


--
-- Name: extensions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.extensions ALTER COLUMN id SET DEFAULT nextval('public.extensions_id_seq'::regclass);


--
-- Name: feature_clusters id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_clusters ALTER COLUMN id SET DEFAULT nextval('public.feature_clusters_id_seq'::regclass);


--
-- Name: feature_suggestions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_suggestions ALTER COLUMN id SET DEFAULT nextval('public.feature_suggestions_id_seq'::regclass);


--
-- Name: features id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.features ALTER COLUMN id SET DEFAULT nextval('public.features_id_seq'::regclass);


--
-- Name: fic_request_answers id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answers ALTER COLUMN id SET DEFAULT nextval('public.fic_request_answers_id_seq'::regclass);


--
-- Name: fic_requests id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_requests ALTER COLUMN id SET DEFAULT nextval('public.fic_requests_id_seq'::regclass);


--
-- Name: follows id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows ALTER COLUMN id SET DEFAULT nextval('public.follows_id_seq'::regclass);


--
-- Name: forum_bans id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_bans ALTER COLUMN id SET DEFAULT nextval('public.forum_bans_id_seq'::regclass);


--
-- Name: forum_categories id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_categories ALTER COLUMN id SET DEFAULT nextval('public.forum_categories_id_seq'::regclass);


--
-- Name: forum_edit_proposals id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_edit_proposals ALTER COLUMN id SET DEFAULT nextval('public.forum_edit_proposals_id_seq'::regclass);


--
-- Name: forum_mod_actions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_mod_actions ALTER COLUMN id SET DEFAULT nextval('public.forum_mod_actions_id_seq'::regclass);


--
-- Name: forum_posts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_posts ALTER COLUMN id SET DEFAULT nextval('public.forum_posts_id_seq'::regclass);


--
-- Name: forum_topic_views id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_topic_views ALTER COLUMN id SET DEFAULT nextval('public.forum_topic_views_id_seq'::regclass);


--
-- Name: forum_topics id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_topics ALTER COLUMN id SET DEFAULT nextval('public.forum_topics_id_seq'::regclass);


--
-- Name: heal_extractions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.heal_extractions ALTER COLUMN id SET DEFAULT nextval('public.heal_extractions_id_seq'::regclass);


--
-- Name: kudos id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kudos ALTER COLUMN id SET DEFAULT nextval('public.kudos_id_seq'::regclass);


--
-- Name: leaderboard_monthly id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_monthly ALTER COLUMN id SET DEFAULT nextval('public.leaderboard_monthly_id_seq'::regclass);


--
-- Name: leaderboard_weekly id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_weekly ALTER COLUMN id SET DEFAULT nextval('public.leaderboard_weekly_id_seq'::regclass);


--
-- Name: locales id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.locales ALTER COLUMN id SET DEFAULT nextval('public.locales_id_seq'::regclass);


--
-- Name: marginalia id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.marginalia ALTER COLUMN id SET DEFAULT nextval('public.marginalia_id_seq'::regclass);


--
-- Name: modlog id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.modlog ALTER COLUMN id SET DEFAULT nextval('public.modlog_id_seq'::regclass);


--
-- Name: notifications id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications ALTER COLUMN id SET DEFAULT nextval('public.notifications_id_seq'::regclass);


--
-- Name: opds_shelves id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.opds_shelves ALTER COLUMN id SET DEFAULT nextval('public.opds_shelves_id_seq'::regclass);


--
-- Name: pending_exports id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_exports ALTER COLUMN id SET DEFAULT nextval('public.pending_exports_id_seq'::regclass);


--
-- Name: pseuds id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pseuds ALTER COLUMN id SET DEFAULT nextval('public.pseuds_id_seq'::regclass);


--
-- Name: reading_history id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_history ALTER COLUMN id SET DEFAULT nextval('public.reading_history_id_seq'::regclass);


--
-- Name: reading_list_items id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_list_items ALTER COLUMN id SET DEFAULT nextval('public.reading_list_items_id_seq'::regclass);


--
-- Name: reading_lists id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_lists ALTER COLUMN id SET DEFAULT nextval('public.reading_lists_id_seq'::regclass);


--
-- Name: reading_stats id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_stats ALTER COLUMN id SET DEFAULT nextval('public.reading_stats_id_seq'::regclass);


--
-- Name: rec_impressions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_impressions ALTER COLUMN id SET DEFAULT nextval('public.rec_impressions_id_seq'::regclass);


--
-- Name: rec_models id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_models ALTER COLUMN id SET DEFAULT nextval('public.rec_models_id_seq'::regclass);


--
-- Name: rec_training_runs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_training_runs ALTER COLUMN id SET DEFAULT nextval('public.rec_training_runs_id_seq'::regclass);


--
-- Name: recommendation_suggestions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_suggestions ALTER COLUMN id SET DEFAULT nextval('public.recommendation_suggestions_id_seq'::regclass);


--
-- Name: registration_applications id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.registration_applications ALTER COLUMN id SET DEFAULT nextval('public.registration_applications_id_seq'::regclass);


--
-- Name: reputation_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reputation_events ALTER COLUMN id SET DEFAULT nextval('public.reputation_events_id_seq'::regclass);


--
-- Name: request_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.request_log ALTER COLUMN id SET DEFAULT nextval('public.request_log_id_seq'::regclass);


--
-- Name: request_source id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.request_source ALTER COLUMN id SET DEFAULT nextval('public.request_source_id_seq'::regclass);


--
-- Name: reviews id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviews ALTER COLUMN id SET DEFAULT nextval('public.reviews_id_seq'::regclass);


--
-- Name: scrape_failures id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scrape_failures ALTER COLUMN id SET DEFAULT nextval('public.scrape_failures_id_seq'::regclass);


--
-- Name: search_queries id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.search_queries ALTER COLUMN id SET DEFAULT nextval('public.search_queries_id_seq'::regclass);


--
-- Name: series id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series ALTER COLUMN id SET DEFAULT nextval('public.series_id_seq'::regclass);


--
-- Name: shelves id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.shelves ALTER COLUMN id SET DEFAULT nextval('public.shelves_id_seq'::regclass);


--
-- Name: skins id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.skins ALTER COLUMN id SET DEFAULT nextval('public.skins_id_seq'::regclass);


--
-- Name: tag_flags id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_flags ALTER COLUMN id SET DEFAULT nextval('public.tag_flags_id_seq'::regclass);


--
-- Name: tag_score_fixes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_score_fixes ALTER COLUMN id SET DEFAULT nextval('public.tag_score_fixes_id_seq'::regclass);


--
-- Name: tags id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tags ALTER COLUMN id SET DEFAULT nextval('public.tags_id_seq'::regclass);


--
-- Name: translations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.translations ALTER COLUMN id SET DEFAULT nextval('public.translations_id_seq'::regclass);


--
-- Name: usage_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.usage_events ALTER COLUMN id SET DEFAULT nextval('public.usage_events_id_seq'::regclass);


--
-- Name: user_badges id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_badges ALTER COLUMN id SET DEFAULT nextval('public.user_badges_id_seq'::regclass);


--
-- Name: user_daily_progress id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_daily_progress ALTER COLUMN id SET DEFAULT nextval('public.user_daily_progress_id_seq'::regclass);


--
-- Name: user_invites id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_invites ALTER COLUMN id SET DEFAULT nextval('public.user_invites_id_seq'::regclass);


--
-- Name: user_recipes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_recipes ALTER COLUMN id SET DEFAULT nextval('public.user_recipes_id_seq'::regclass);


--
-- Name: user_reports id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_reports ALTER COLUMN id SET DEFAULT nextval('public.user_reports_id_seq'::regclass);


--
-- Name: user_site_credentials id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_site_credentials ALTER COLUMN id SET DEFAULT nextval('public.user_site_credentials_id_seq'::regclass);


--
-- Name: user_views id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_views ALTER COLUMN id SET DEFAULT nextval('public.user_views_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: work_proposals id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposals ALTER COLUMN id SET DEFAULT nextval('public.work_proposals_id_seq'::regclass);


--
-- Name: work_rating_verifications id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_rating_verifications ALTER COLUMN id SET DEFAULT nextval('public.work_rating_verifications_id_seq'::regclass);


--
-- Name: work_ratings id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_ratings ALTER COLUMN id SET DEFAULT nextval('public.work_ratings_id_seq'::regclass);


--
-- Name: work_shelves id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_shelves ALTER COLUMN id SET DEFAULT nextval('public.work_shelves_id_seq'::regclass);


--
-- Name: work_translations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_translations ALTER COLUMN id SET DEFAULT nextval('public.work_translations_id_seq'::regclass);


--
-- Name: works id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.works ALTER COLUMN id SET DEFAULT nextval('public.works_id_seq'::regclass);


--
-- Name: xp_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.xp_events ALTER COLUMN id SET DEFAULT nextval('public.xp_events_id_seq'::regclass);


--



--
-- Name: admin_daily_stats admin_daily_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_daily_stats
    ADD CONSTRAINT admin_daily_stats_pkey PRIMARY KEY (date);


--
-- Name: agent_runs agent_runs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.agent_runs
    ADD CONSTRAINT agent_runs_pkey PRIMARY KEY (id);


--
-- Name: arena_votes arena_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.arena_votes
    ADD CONSTRAINT arena_votes_pkey PRIMARY KEY (id);


--
-- Name: arena_votes arena_votes_user_id_cluster_ids_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.arena_votes
    ADD CONSTRAINT arena_votes_user_id_cluster_ids_key UNIQUE (user_id, cluster_ids);


--
-- Name: ask_translation_cache ask_translation_cache_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ask_translation_cache
    ADD CONSTRAINT ask_translation_cache_pkey PRIMARY KEY (nl_query);


--
-- Name: author_blacklist author_blacklist_source_id_author_id_reason_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_blacklist
    ADD CONSTRAINT author_blacklist_source_id_author_id_reason_key UNIQUE (source_id, author_id, reason);


--
-- Name: author_merge_proposals author_merge_proposals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_merge_proposals
    ADD CONSTRAINT author_merge_proposals_pkey PRIMARY KEY (id);


--
-- Name: author_profile_links author_profile_links_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_profile_links
    ADD CONSTRAINT author_profile_links_pkey PRIMARY KEY (id);


--
-- Name: author_profile_links author_profile_links_source_url_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_profile_links
    ADD CONSTRAINT author_profile_links_source_url_key UNIQUE (source_url);


--
-- Name: author_profiles author_profiles_canonical_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_profiles
    ADD CONSTRAINT author_profiles_canonical_name_key UNIQUE (canonical_name);


--
-- Name: author_profiles author_profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_profiles
    ADD CONSTRAINT author_profiles_pkey PRIMARY KEY (id);


--
-- Name: author_socials author_socials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_socials
    ADD CONSTRAINT author_socials_pkey PRIMARY KEY (id);


--
-- Name: author_socials author_socials_profile_id_platform_url_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_socials
    ADD CONSTRAINT author_socials_profile_id_platform_url_key UNIQUE (profile_id, platform, url);


--
-- Name: auto_merge_log auto_merge_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.auto_merge_log
    ADD CONSTRAINT auto_merge_log_pkey PRIMARY KEY (id);


--
-- Name: badge_definitions badge_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.badge_definitions
    ADD CONSTRAINT badge_definitions_pkey PRIMARY KEY (badge_type);


--
-- Name: blocked_users blocked_users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blocked_users
    ADD CONSTRAINT blocked_users_pkey PRIMARY KEY (user_id, blocked_user_id);


--
-- Name: bookmarks bookmarks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bookmarks
    ADD CONSTRAINT bookmarks_pkey PRIMARY KEY (id);


--
-- Name: bookmarks bookmarks_user_id_url_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bookmarks
    ADD CONSTRAINT bookmarks_user_id_url_id_key UNIQUE (user_id, url_id);


--
-- Name: bot_scores bot_scores_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bot_scores
    ADD CONSTRAINT bot_scores_pkey PRIMARY KEY (ip, client_id, window_start);


--
-- Name: challenge_assignments challenge_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_assignments
    ADD CONSTRAINT challenge_assignments_pkey PRIMARY KEY (id);


--
-- Name: challenge_signups challenge_signups_challenge_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_signups
    ADD CONSTRAINT challenge_signups_challenge_id_user_id_key UNIQUE (challenge_id, user_id);


--
-- Name: challenge_signups challenge_signups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_signups
    ADD CONSTRAINT challenge_signups_pkey PRIMARY KEY (id);


--
-- Name: challenges challenges_collection_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenges
    ADD CONSTRAINT challenges_collection_id_key UNIQUE (collection_id);


--
-- Name: challenges challenges_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenges
    ADD CONSTRAINT challenges_pkey PRIMARY KEY (id);


--
-- Name: chapter_translation_versions chapter_translation_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translation_versions
    ADD CONSTRAINT chapter_translation_versions_pkey PRIMARY KEY (id);


--
-- Name: chapter_translation_versions chapter_translation_versions_work_id_locale_code_version_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translation_versions
    ADD CONSTRAINT chapter_translation_versions_work_id_locale_code_version_key UNIQUE (work_id, locale_code, version);


--
-- Name: chapter_translations chapter_translations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translations
    ADD CONSTRAINT chapter_translations_pkey PRIMARY KEY (id);


--
-- Name: chapter_translations chapter_translations_work_id_locale_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translations
    ADD CONSTRAINT chapter_translations_work_id_locale_code_key UNIQUE (work_id, locale_code);


--
-- Name: collection_bookmarks collection_bookmarks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_bookmarks
    ADD CONSTRAINT collection_bookmarks_pkey PRIMARY KEY (id);


--
-- Name: collection_bookmarks collection_bookmarks_user_id_list_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_bookmarks
    ADD CONSTRAINT collection_bookmarks_user_id_list_id_key UNIQUE (user_id, list_id);


--
-- Name: collection_item_requests collection_item_requests_list_id_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_requests
    ADD CONSTRAINT collection_item_requests_list_id_work_id_key UNIQUE (list_id, work_id);


--
-- Name: collection_item_requests collection_item_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_requests
    ADD CONSTRAINT collection_item_requests_pkey PRIMARY KEY (id);


--
-- Name: comment_triage comment_triage_comment_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_triage
    ADD CONSTRAINT comment_triage_comment_id_key UNIQUE (comment_id);


--
-- Name: comment_triage comment_triage_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_triage
    ADD CONSTRAINT comment_triage_pkey PRIMARY KEY (id);


--
-- Name: comments comments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_pkey PRIMARY KEY (id);


--
-- Name: content_scan content_scan_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_scan
    ADD CONSTRAINT content_scan_pkey PRIMARY KEY (id);


--
-- Name: content_scan content_scan_url_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_scan
    ADD CONSTRAINT content_scan_url_id_key UNIQUE (url_id);


--
-- Name: creatorships creatorships_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_pkey PRIMARY KEY (id);


--
-- Name: creatorships creatorships_pseud_id_series_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_pseud_id_series_id_key UNIQUE (pseud_id, series_id);


--
-- Name: creatorships creatorships_pseud_id_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_pseud_id_work_id_key UNIQUE (pseud_id, work_id);


--
-- Name: curator_content_overrides curator_content_overrides_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_content_overrides
    ADD CONSTRAINT curator_content_overrides_pkey PRIMARY KEY (url_id);


--
-- Name: curator_fix_proposals curator_fix_proposals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_fix_proposals
    ADD CONSTRAINT curator_fix_proposals_pkey PRIMARY KEY (id);


--
-- Name: curator_fix_votes curator_fix_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_fix_votes
    ADD CONSTRAINT curator_fix_votes_pkey PRIMARY KEY (proposal_id, user_id);


--
-- Name: curator_metadata_proposals curator_metadata_proposals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_proposals
    ADD CONSTRAINT curator_metadata_proposals_pkey PRIMARY KEY (id);


--
-- Name: curator_metadata_votes curator_metadata_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_votes
    ADD CONSTRAINT curator_metadata_votes_pkey PRIMARY KEY (proposal_id, user_id);


--
-- Name: daily_quests daily_quests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_quests
    ADD CONSTRAINT daily_quests_pkey PRIMARY KEY (id);


--
-- Name: daily_quests daily_quests_quest_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_quests
    ADD CONSTRAINT daily_quests_quest_type_key UNIQUE (quest_type);


--
-- Name: doc_sections doc_sections_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.doc_sections
    ADD CONSTRAINT doc_sections_pkey PRIMARY KEY (id);


--
-- Name: doc_sections doc_sections_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.doc_sections
    ADD CONSTRAINT doc_sections_slug_key UNIQUE (slug);


--
-- Name: exp_events exp_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.exp_events
    ADD CONSTRAINT exp_events_pkey PRIMARY KEY (id);


--
-- Name: export_log export_log_url_id_version_etype_input_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_log
    ADD CONSTRAINT export_log_url_id_version_etype_input_hash_key UNIQUE (url_id, version, etype, input_hash);


--
-- Name: extensions extensions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.extensions
    ADD CONSTRAINT extensions_pkey PRIMARY KEY (id);


--
-- Name: extensions extensions_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.extensions
    ADD CONSTRAINT extensions_slug_key UNIQUE (slug);


--
-- Name: feature_clusters feature_clusters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_clusters
    ADD CONSTRAINT feature_clusters_pkey PRIMARY KEY (id);


--
-- Name: feature_suggestions feature_suggestions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_suggestions
    ADD CONSTRAINT feature_suggestions_pkey PRIMARY KEY (id);


--
-- Name: features features_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.features
    ADD CONSTRAINT features_pkey PRIMARY KEY (id);


--
-- Name: features features_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.features
    ADD CONSTRAINT features_slug_key UNIQUE (slug);


--
-- Name: fic_blacklist fic_blacklist_url_id_reason_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_blacklist
    ADD CONSTRAINT fic_blacklist_url_id_reason_key UNIQUE (url_id, reason);


--
-- Name: fic_bookmark_cooccur fic_bookmark_cooccur_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_bookmark_cooccur
    ADD CONSTRAINT fic_bookmark_cooccur_pkey PRIMARY KEY (work_a, work_b);


--
-- Name: fic_bookmarks fic_bookmarks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_bookmarks
    ADD CONSTRAINT fic_bookmarks_pkey PRIMARY KEY (user_hash, url_id);


--
-- Name: fic_info fic_info_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_info
    ADD CONSTRAINT fic_info_pkey PRIMARY KEY (id);


--
-- Name: fic_request_answer_votes fic_request_answer_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answer_votes
    ADD CONSTRAINT fic_request_answer_votes_pkey PRIMARY KEY (answer_id, user_id);


--
-- Name: fic_request_answers fic_request_answers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answers
    ADD CONSTRAINT fic_request_answers_pkey PRIMARY KEY (id);


--
-- Name: fic_request_answers fic_request_answers_request_id_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answers
    ADD CONSTRAINT fic_request_answers_request_id_work_id_key UNIQUE (request_id, work_id);


--
-- Name: fic_request_upvotes fic_request_upvotes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_upvotes
    ADD CONSTRAINT fic_request_upvotes_pkey PRIMARY KEY (request_id, user_id);


--
-- Name: fic_requests fic_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_requests
    ADD CONSTRAINT fic_requests_pkey PRIMARY KEY (id);


--
-- Name: fic_tag_votes fic_tag_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_tag_votes
    ADD CONSTRAINT fic_tag_votes_pkey PRIMARY KEY (url_id, tag_id, voter_ip);


--
-- Name: fic_tags fic_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_tags
    ADD CONSTRAINT fic_tags_pkey PRIMARY KEY (url_id, tag_id);


--
-- Name: fic_version_bump fic_version_bump_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_version_bump
    ADD CONSTRAINT fic_version_bump_pkey PRIMARY KEY (id);


--
-- Name: fic_works fic_works_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_works
    ADD CONSTRAINT fic_works_pkey PRIMARY KEY (url_id);


--
-- Name: fic_works fic_works_site_domain_site_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_works
    ADD CONSTRAINT fic_works_site_domain_site_work_id_key UNIQUE (site_domain, site_work_id);


--
-- Name: follows follows_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows
    ADD CONSTRAINT follows_pkey PRIMARY KEY (id);


--
-- Name: forum_bans forum_bans_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_bans
    ADD CONSTRAINT forum_bans_pkey PRIMARY KEY (id);


--
-- Name: forum_categories forum_categories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_categories
    ADD CONSTRAINT forum_categories_pkey PRIMARY KEY (id);


--
-- Name: forum_categories forum_categories_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_categories
    ADD CONSTRAINT forum_categories_slug_key UNIQUE (slug);


--
-- Name: forum_edit_proposals forum_edit_proposals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_edit_proposals
    ADD CONSTRAINT forum_edit_proposals_pkey PRIMARY KEY (id);


--
-- Name: forum_follows forum_follows_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_follows
    ADD CONSTRAINT forum_follows_pkey PRIMARY KEY (user_id, topic_id);


--
-- Name: forum_metamod_votes forum_metamod_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_metamod_votes
    ADD CONSTRAINT forum_metamod_votes_pkey PRIMARY KEY (mod_action_id, voter_id);


--
-- Name: forum_mod_actions forum_mod_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_mod_actions
    ADD CONSTRAINT forum_mod_actions_pkey PRIMARY KEY (id);


--
-- Name: forum_mod_grants forum_mod_grants_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_mod_grants
    ADD CONSTRAINT forum_mod_grants_pkey PRIMARY KEY (user_id);


--
-- Name: forum_post_reactions forum_post_reactions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_post_reactions
    ADD CONSTRAINT forum_post_reactions_pkey PRIMARY KEY (post_id, user_id, emoji);


--
-- Name: forum_posts forum_posts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_pkey PRIMARY KEY (id);


--
-- Name: forum_read_state forum_read_state_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_read_state
    ADD CONSTRAINT forum_read_state_pkey PRIMARY KEY (user_id, topic_id);


--
-- Name: forum_topic_views forum_topic_views_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_topic_views
    ADD CONSTRAINT forum_topic_views_pkey PRIMARY KEY (id);


--
-- Name: forum_topics forum_topics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_topics
    ADD CONSTRAINT forum_topics_pkey PRIMARY KEY (id);


--
-- Name: heal_extractions heal_extractions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.heal_extractions
    ADD CONSTRAINT heal_extractions_pkey PRIMARY KEY (id);


--
-- Name: kudos kudos_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kudos
    ADD CONSTRAINT kudos_pkey PRIMARY KEY (id);


--
-- Name: kudos kudos_work_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kudos
    ADD CONSTRAINT kudos_work_id_user_id_key UNIQUE (work_id, user_id);


--
-- Name: leaderboard_monthly leaderboard_monthly_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_monthly
    ADD CONSTRAINT leaderboard_monthly_pkey PRIMARY KEY (id);


--
-- Name: leaderboard_monthly leaderboard_monthly_user_id_month_start_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_monthly
    ADD CONSTRAINT leaderboard_monthly_user_id_month_start_key UNIQUE (user_id, month_start);


--
-- Name: leaderboard_weekly leaderboard_weekly_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_weekly
    ADD CONSTRAINT leaderboard_weekly_pkey PRIMARY KEY (id);


--
-- Name: leaderboard_weekly leaderboard_weekly_user_id_week_start_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_weekly
    ADD CONSTRAINT leaderboard_weekly_user_id_week_start_key UNIQUE (user_id, week_start);


--
-- Name: locales locales_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.locales
    ADD CONSTRAINT locales_code_key UNIQUE (code);


--
-- Name: locales locales_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.locales
    ADD CONSTRAINT locales_pkey PRIMARY KEY (id);


--
-- Name: login_streaks login_streaks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.login_streaks
    ADD CONSTRAINT login_streaks_pkey PRIMARY KEY (user_id);


--
-- Name: marginalia marginalia_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.marginalia
    ADD CONSTRAINT marginalia_pkey PRIMARY KEY (id);


--
-- Name: marginalia marginalia_work_id_chapter_index_passage_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.marginalia
    ADD CONSTRAINT marginalia_work_id_chapter_index_passage_hash_key UNIQUE (work_id, chapter_index, passage_hash);


--
-- Name: modlog modlog_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.modlog
    ADD CONSTRAINT modlog_pkey PRIMARY KEY (id);


--
-- Name: notification_preferences notification_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_preferences
    ADD CONSTRAINT notification_preferences_pkey PRIMARY KEY (user_id);


--
-- Name: notifications notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);


--
-- Name: opds_shelf_items opds_shelf_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.opds_shelf_items
    ADD CONSTRAINT opds_shelf_items_pkey PRIMARY KEY (shelf_id, url_id);


--
-- Name: opds_shelves opds_shelves_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.opds_shelves
    ADD CONSTRAINT opds_shelves_pkey PRIMARY KEY (id);


--
-- Name: pending_exports pending_exports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pending_exports
    ADD CONSTRAINT pending_exports_pkey PRIMARY KEY (id);


--
-- Name: precomputed_recommendations precomputed_recommendations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.precomputed_recommendations
    ADD CONSTRAINT precomputed_recommendations_pkey PRIMARY KEY (url_id, recommended_url_id);


--
-- Name: pseuds pseuds_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pseuds
    ADD CONSTRAINT pseuds_pkey PRIMARY KEY (id);


--
-- Name: pseuds pseuds_user_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pseuds
    ADD CONSTRAINT pseuds_user_id_name_key UNIQUE (user_id, name);


--
-- Name: reactions reactions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reactions
    ADD CONSTRAINT reactions_pkey PRIMARY KEY (target_type, target_id, user_id, emoji);


--
-- Name: reading_history reading_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_history
    ADD CONSTRAINT reading_history_pkey PRIMARY KEY (id);


--
-- Name: reading_list_items reading_list_items_list_id_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_list_items
    ADD CONSTRAINT reading_list_items_list_id_work_id_key UNIQUE (list_id, work_id);


--
-- Name: reading_list_items reading_list_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_list_items
    ADD CONSTRAINT reading_list_items_pkey PRIMARY KEY (id);


--
-- Name: reading_lists reading_lists_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_lists
    ADD CONSTRAINT reading_lists_pkey PRIMARY KEY (id);


--
-- Name: reading_stats reading_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_stats
    ADD CONSTRAINT reading_stats_pkey PRIMARY KEY (id);


--
-- Name: reading_stats reading_stats_user_id_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_stats
    ADD CONSTRAINT reading_stats_user_id_work_id_key UNIQUE (user_id, work_id);


--
-- Name: rec_author_graph rec_author_graph_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_author_graph
    ADD CONSTRAINT rec_author_graph_pkey PRIMARY KEY (author_a, author_b);


--
-- Name: rec_bandit_arms rec_bandit_arms_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_bandit_arms
    ADD CONSTRAINT rec_bandit_arms_pkey PRIMARY KEY (work_id, strategy);


--
-- Name: rec_embeddings rec_embeddings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_embeddings
    ADD CONSTRAINT rec_embeddings_pkey PRIMARY KEY (work_id, model);


--
-- Name: rec_impressions rec_impressions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_impressions
    ADD CONSTRAINT rec_impressions_pkey PRIMARY KEY (id);


--
-- Name: rec_models rec_models_name_version_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_models
    ADD CONSTRAINT rec_models_name_version_key UNIQUE (name, version);


--
-- Name: rec_models rec_models_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_models
    ADD CONSTRAINT rec_models_pkey PRIMARY KEY (id);


--
-- Name: rec_training_runs rec_training_runs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_training_runs
    ADD CONSTRAINT rec_training_runs_pkey PRIMARY KEY (id);


--
-- Name: rec_transitions rec_transitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_transitions
    ADD CONSTRAINT rec_transitions_pkey PRIMARY KEY (from_work, to_work);


--
-- Name: rec_user_clusters rec_user_clusters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_user_clusters
    ADD CONSTRAINT rec_user_clusters_pkey PRIMARY KEY (user_id, cluster_id);


--
-- Name: rec_user_curator_align rec_user_curator_align_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_user_curator_align
    ADD CONSTRAINT rec_user_curator_align_pkey PRIMARY KEY (user_id);


--
-- Name: rec_user_signals rec_user_signals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_user_signals
    ADD CONSTRAINT rec_user_signals_pkey PRIMARY KEY (user_id, work_id, signal_type);


--
-- Name: recommendation_suggestions recommendation_suggestions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_suggestions
    ADD CONSTRAINT recommendation_suggestions_pkey PRIMARY KEY (id);


--
-- Name: recommendation_suggestions recommendation_suggestions_url_id_suggested_url_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_suggestions
    ADD CONSTRAINT recommendation_suggestions_url_id_suggested_url_id_key UNIQUE (url_id, suggested_url_id);


--
-- Name: recommendation_suggestions recommendation_suggestions_url_id_suggested_url_id_submitte_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_suggestions
    ADD CONSTRAINT recommendation_suggestions_url_id_suggested_url_id_submitte_key UNIQUE (url_id, suggested_url_id, submitted_by_ip);


--
-- Name: registration_applications registration_applications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.registration_applications
    ADD CONSTRAINT registration_applications_pkey PRIMARY KEY (id);


--
-- Name: reputation_events reputation_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reputation_events
    ADD CONSTRAINT reputation_events_pkey PRIMARY KEY (id);


--
-- Name: request_log request_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.request_log
    ADD CONSTRAINT request_log_pkey PRIMARY KEY (id);


--
-- Name: request_source request_source_is_automated_route_description_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.request_source
    ADD CONSTRAINT request_source_is_automated_route_description_key UNIQUE (is_automated, route, description);


--
-- Name: request_source request_source_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.request_source
    ADD CONSTRAINT request_source_pkey PRIMARY KEY (id);


--
-- Name: reviews reviews_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviews
    ADD CONSTRAINT reviews_pkey PRIMARY KEY (id);


--
-- Name: reviews reviews_user_work_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviews
    ADD CONSTRAINT reviews_user_work_unique UNIQUE (user_id, work_id);


--
-- Name: schema_migrations schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.schema_migrations
    ADD CONSTRAINT schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: scrape_failures scrape_failures_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scrape_failures
    ADD CONSTRAINT scrape_failures_pkey PRIMARY KEY (id);


--
-- Name: search_queries search_queries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.search_queries
    ADD CONSTRAINT search_queries_pkey PRIMARY KEY (id);


--
-- Name: series series_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series
    ADD CONSTRAINT series_pkey PRIMARY KEY (id);


--
-- Name: series_works series_works_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series_works
    ADD CONSTRAINT series_works_pkey PRIMARY KEY (series_id, work_id);


--
-- Name: shelves shelves_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.shelves
    ADD CONSTRAINT shelves_pkey PRIMARY KEY (id);


--
-- Name: shelves shelves_user_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.shelves
    ADD CONSTRAINT shelves_user_id_name_key UNIQUE (user_id, name);


--
-- Name: site_settings site_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.site_settings
    ADD CONSTRAINT site_settings_pkey PRIMARY KEY (key);


--
-- Name: skins skins_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.skins
    ADD CONSTRAINT skins_pkey PRIMARY KEY (id);


--
-- Name: tag_aliases tag_aliases_alias_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT tag_aliases_alias_name_key UNIQUE (alias_name);


--
-- Name: tag_embeddings tag_embeddings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_embeddings
    ADD CONSTRAINT tag_embeddings_pkey PRIMARY KEY (tag_id);


--
-- Name: tag_flags tag_flags_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_flags
    ADD CONSTRAINT tag_flags_pkey PRIMARY KEY (id);


--
-- Name: tag_flags tag_flags_url_id_tag_id_flagged_by_ip_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_flags
    ADD CONSTRAINT tag_flags_url_id_tag_id_flagged_by_ip_key UNIQUE (url_id, tag_id, flagged_by_ip);


--
-- Name: tag_score_fixes tag_score_fixes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_score_fixes
    ADD CONSTRAINT tag_score_fixes_pkey PRIMARY KEY (id);


--
-- Name: tag_types tag_types_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_types
    ADD CONSTRAINT tag_types_name_key UNIQUE (name);


--
-- Name: tag_types tag_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_types
    ADD CONSTRAINT tag_types_pkey PRIMARY KEY (id);


--
-- Name: tags tags_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_name_key UNIQUE (name);


--
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: translations translations_locale_code_namespace_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.translations
    ADD CONSTRAINT translations_locale_code_namespace_key_key UNIQUE (locale_code, namespace, key);


--
-- Name: translations translations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.translations
    ADD CONSTRAINT translations_pkey PRIMARY KEY (id);


--
-- Name: usage_events usage_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.usage_events
    ADD CONSTRAINT usage_events_pkey PRIMARY KEY (id);


--
-- Name: user_badges user_badges_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_badges
    ADD CONSTRAINT user_badges_pkey PRIMARY KEY (id);


--
-- Name: user_badges user_badges_user_id_badge_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_badges
    ADD CONSTRAINT user_badges_user_id_badge_type_key UNIQUE (user_id, badge_type);


--
-- Name: user_daily_progress user_daily_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_daily_progress
    ADD CONSTRAINT user_daily_progress_pkey PRIMARY KEY (id);


--
-- Name: user_daily_progress user_daily_progress_user_id_quest_id_quest_date_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_daily_progress
    ADD CONSTRAINT user_daily_progress_user_id_quest_id_quest_date_key UNIQUE (user_id, quest_id, quest_date);


--
-- Name: user_features user_features_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_features
    ADD CONSTRAINT user_features_pkey PRIMARY KEY (user_id, feature_id);


--
-- Name: user_invites user_invites_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_invites
    ADD CONSTRAINT user_invites_code_key UNIQUE (code);


--
-- Name: user_invites user_invites_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_invites
    ADD CONSTRAINT user_invites_pkey PRIMARY KEY (id);


--
-- Name: user_layouts user_layouts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_layouts
    ADD CONSTRAINT user_layouts_pkey PRIMARY KEY (user_id, page);


--
-- Name: user_prefs user_prefs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_prefs
    ADD CONSTRAINT user_prefs_pkey PRIMARY KEY (user_id, key);


--
-- Name: user_recipes user_recipes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_recipes
    ADD CONSTRAINT user_recipes_pkey PRIMARY KEY (id);


--
-- Name: user_reports user_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_reports
    ADD CONSTRAINT user_reports_pkey PRIMARY KEY (id);


--
-- Name: user_site_credentials user_site_credentials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_site_credentials
    ADD CONSTRAINT user_site_credentials_pkey PRIMARY KEY (id);


--
-- Name: user_site_credentials user_site_credentials_user_id_domain_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_site_credentials
    ADD CONSTRAINT user_site_credentials_user_id_domain_key UNIQUE (user_id, domain);


--
-- Name: user_skins user_skins_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_skins
    ADD CONSTRAINT user_skins_pkey PRIMARY KEY (user_id);


--
-- Name: user_views user_views_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_views
    ADD CONSTRAINT user_views_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: work_proposal_votes work_proposal_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposal_votes
    ADD CONSTRAINT work_proposal_votes_pkey PRIMARY KEY (proposal_id, user_id);


--
-- Name: work_proposals work_proposals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposals
    ADD CONSTRAINT work_proposals_pkey PRIMARY KEY (id);


--
-- Name: work_rating_verifications work_rating_verifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_rating_verifications
    ADD CONSTRAINT work_rating_verifications_pkey PRIMARY KEY (id);


--
-- Name: work_rating_verifications work_rating_verifications_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_rating_verifications
    ADD CONSTRAINT work_rating_verifications_work_id_key UNIQUE (work_id);


--
-- Name: work_ratings work_ratings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_ratings
    ADD CONSTRAINT work_ratings_pkey PRIMARY KEY (id);


--
-- Name: work_ratings work_ratings_user_id_url_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_ratings
    ADD CONSTRAINT work_ratings_user_id_url_id_key UNIQUE (user_id, url_id);


--
-- Name: work_shelves work_shelves_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_shelves
    ADD CONSTRAINT work_shelves_pkey PRIMARY KEY (id);


--
-- Name: work_shelves work_shelves_shelf_id_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_shelves
    ADD CONSTRAINT work_shelves_shelf_id_work_id_key UNIQUE (shelf_id, work_id);


--
-- Name: work_skins work_skins_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_skins
    ADD CONSTRAINT work_skins_pkey PRIMARY KEY (work_id, skin_id);


--
-- Name: work_translations work_translations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_translations
    ADD CONSTRAINT work_translations_pkey PRIMARY KEY (id);


--
-- Name: work_translations work_translations_work_id_locale_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_translations
    ADD CONSTRAINT work_translations_work_id_locale_code_key UNIQUE (work_id, locale_code);


--
-- Name: works works_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.works
    ADD CONSTRAINT works_pkey PRIMARY KEY (id);


--
-- Name: xp_events xp_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.xp_events
    ADD CONSTRAINT xp_events_pkey PRIMARY KEY (id);


--
-- Name: xp_source_defs xp_source_defs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.xp_source_defs
    ADD CONSTRAINT xp_source_defs_pkey PRIMARY KEY (event_type);


--
-- Name: idx_agent_runs_status_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_runs_status_created ON public.agent_runs USING btree (status, created_at);


--
-- Name: idx_author_profile_links_author; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_author_profile_links_author ON public.author_profile_links USING btree (source_author);


--
-- Name: idx_author_profile_links_profile; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_author_profile_links_profile ON public.author_profile_links USING btree (profile_id);


--
-- Name: idx_author_socials_profile; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_author_socials_profile ON public.author_socials USING btree (profile_id);


--
-- Name: idx_auto_merge_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auto_merge_work ON public.auto_merge_log USING btree (matched_work_id);


--
-- Name: idx_bookmarks_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bookmarks_user ON public.bookmarks USING btree (user_id, created_at DESC);


--
-- Name: idx_bookmarks_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bookmarks_work ON public.bookmarks USING btree (url_id);


--
-- Name: idx_bot_scores_window; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bot_scores_window ON public.bot_scores USING btree (window_start DESC);


--
-- Name: idx_challenge_assignments_challenge; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_challenge_assignments_challenge ON public.challenge_assignments USING btree (challenge_id);


--
-- Name: idx_challenge_signups_challenge; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_challenge_signups_challenge ON public.challenge_signups USING btree (challenge_id);


--
-- Name: idx_chapter_translation_versions_review; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_chapter_translation_versions_review ON public.chapter_translation_versions USING btree (status, created_at DESC);


--
-- Name: idx_chapter_translations_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_chapter_translations_status ON public.chapter_translations USING btree (status);


--
-- Name: idx_chapter_translations_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_chapter_translations_work ON public.chapter_translations USING btree (work_id, locale_code);


--
-- Name: idx_collection_bookmarks_collection; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_bookmarks_collection ON public.collection_bookmarks USING btree (list_id);


--
-- Name: idx_collection_bookmarks_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_bookmarks_user ON public.collection_bookmarks USING btree (user_id, created_at DESC);


--
-- Name: idx_collection_item_requests_list; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collection_item_requests_list ON public.collection_item_requests USING btree (list_id, status);


--
-- Name: idx_collections_owner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collections_owner ON public.reading_lists USING btree (user_id, collection_kind, deleted_at) WHERE ((collection_kind = 'collection'::text) AND (deleted_at IS NULL));


--
-- Name: idx_collections_public; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_collections_public ON public.reading_lists USING btree (collection_kind, visibility, deleted_at, id DESC) WHERE ((collection_kind = 'collection'::text) AND (visibility = ANY (ARRAY['public'::text, 'anonymous'::text])) AND (deleted_at IS NULL));


--
-- Name: idx_comment_triage_comment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comment_triage_comment_id ON public.comment_triage USING btree (comment_id);


--
-- Name: idx_comments_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_parent ON public.comments USING btree (parent_id, created_at) WHERE ((deleted_at IS NULL) AND (is_hidden = false));


--
-- Name: idx_comments_url_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_url_id ON public.comments USING btree (url_id);


--
-- Name: idx_comments_url_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_url_parent ON public.comments USING btree (url_id, parent_id);


--
-- Name: idx_comments_work_constructive; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_work_constructive ON public.comments USING btree (work_id) WHERE ((deleted_at IS NULL) AND (constructive = true));


--
-- Name: idx_comments_work_top_newest; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comments_work_top_newest ON public.comments USING btree (url_id, created_at DESC) WHERE ((parent_id IS NULL) AND (deleted_at IS NULL) AND (is_hidden = false));


--
-- Name: idx_content_scan_classification; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_scan_classification ON public.content_scan USING btree (classification);


--
-- Name: idx_content_scan_review_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_scan_review_status ON public.content_scan USING btree (review_status);


--
-- Name: idx_content_scan_warnings; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_scan_warnings ON public.content_scan USING btree (detected_warnings);


--
-- Name: idx_cooccur_a; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cooccur_a ON public.fic_bookmark_cooccur USING btree (work_a);


--
-- Name: idx_cooccur_b; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cooccur_b ON public.fic_bookmark_cooccur USING btree (work_b);


--
-- Name: idx_cooccur_domain; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cooccur_domain ON public.fic_bookmark_cooccur USING btree (site_domain);


--
-- Name: idx_creatorships_pseud; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_creatorships_pseud ON public.creatorships USING btree (pseud_id);


--
-- Name: idx_creatorships_series; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_creatorships_series ON public.creatorships USING btree (series_id) WHERE (series_id IS NOT NULL);


--
-- Name: idx_creatorships_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_creatorships_work ON public.creatorships USING btree (work_id) WHERE (work_id IS NOT NULL);


--
-- Name: idx_curator_fix_proposals_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_curator_fix_proposals_status ON public.curator_fix_proposals USING btree (status, created_at);


--
-- Name: idx_curator_metadata_proposals_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_curator_metadata_proposals_status ON public.curator_metadata_proposals USING btree (status, created_at);


--
-- Name: idx_curator_metadata_proposals_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_curator_metadata_proposals_work ON public.curator_metadata_proposals USING btree (work_id, status);


--
-- Name: idx_doc_sections_embedding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_doc_sections_embedding ON public.doc_sections USING hnsw (embedding public.vector_cosine_ops);


--
-- Name: idx_doc_sections_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_doc_sections_slug ON public.doc_sections USING btree (slug);


--
-- Name: idx_exp_events_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_exp_events_user_created ON public.exp_events USING btree (user_id, created_at DESC);


--
-- Name: idx_extensions_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_extensions_kind ON public.extensions USING btree (kind);


--
-- Name: idx_extensions_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_extensions_slug ON public.extensions USING btree (slug);


--
-- Name: idx_feature_clusters_embedding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_feature_clusters_embedding ON public.feature_clusters USING ivfflat (embedding public.vector_cosine_ops) WITH (lists='10');


--
-- Name: idx_fic_bookmarks_domain; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_bookmarks_domain ON public.fic_bookmarks USING btree (site_domain);


--
-- Name: idx_fic_bookmarks_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_bookmarks_url ON public.fic_bookmarks USING btree (url_id);


--
-- Name: idx_fic_bookmarks_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_bookmarks_user ON public.fic_bookmarks USING btree (user_hash);


--
-- Name: idx_fic_info_author_trgm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_info_author_trgm ON public.fic_info USING gin (author public.gin_trgm_ops);


--
-- Name: idx_fic_info_body_text_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_info_body_text_search ON public.fic_info USING gin (body_text_search);


--
-- Name: idx_fic_info_text_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_info_text_search ON public.fic_info USING gin (text_search);


--
-- Name: idx_fic_info_title_trgm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_info_title_trgm ON public.fic_info USING gin (title public.gin_trgm_ops);


--
-- Name: idx_fic_info_work_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_info_work_id ON public.fic_info USING btree (work_id);


--
-- Name: idx_fic_req_ans_votes_ans; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_req_ans_votes_ans ON public.fic_request_answer_votes USING btree (answer_id);


--
-- Name: idx_fic_req_answers_req; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_req_answers_req ON public.fic_request_answers USING btree (request_id);


--
-- Name: idx_fic_req_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_req_status ON public.fic_requests USING btree (status, created_at DESC);


--
-- Name: idx_fic_request_upvotes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_request_upvotes_user ON public.fic_request_upvotes USING btree (user_id);


--
-- Name: idx_fic_tags_machine_queue; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_tags_machine_queue ON public.fic_tags USING btree (reviewed_at) WHERE (is_machine_suggested = true);


--
-- Name: idx_fic_tags_tag; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_tags_tag ON public.fic_tags USING btree (tag_id);


--
-- Name: idx_fic_tags_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_tags_url ON public.fic_tags USING btree (url_id);


--
-- Name: idx_fic_works_favouriter_count; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_works_favouriter_count ON public.fic_works USING btree (favouriter_count);


--
-- Name: idx_fic_works_site_domain; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fic_works_site_domain ON public.fic_works USING btree (site_domain);


--
-- Name: idx_follows_author; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_follows_author ON public.follows USING btree (author_name, created_at DESC);


--
-- Name: idx_follows_followee; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_follows_followee ON public.follows USING btree (followee_id, created_at DESC);


--
-- Name: idx_follows_follower; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_follows_follower ON public.follows USING btree (follower_id, created_at DESC);


--
-- Name: idx_follows_last_seen; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_follows_last_seen ON public.follows USING btree (follower_id, last_seen DESC) WHERE (work_id IS NOT NULL);


--
-- Name: idx_follows_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_follows_unique ON public.follows USING btree (follower_id, COALESCE(followee_id, 0), COALESCE(work_id, 0), COALESCE(author_name, ''::text));


--
-- Name: idx_follows_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_follows_work ON public.follows USING btree (work_id, created_at DESC);


--
-- Name: idx_forum_bans_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_bans_user ON public.forum_bans USING btree (user_id);


--
-- Name: idx_forum_edit_proposals_queue; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_edit_proposals_queue ON public.forum_edit_proposals USING btree (status, created_at);


--
-- Name: idx_forum_edit_proposals_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_edit_proposals_target ON public.forum_edit_proposals USING btree (target_type, target_id, created_at DESC);


--
-- Name: idx_forum_metamod_votes_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_metamod_votes_action ON public.forum_metamod_votes USING btree (mod_action_id, created_at);


--
-- Name: idx_forum_metamod_votes_voter; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_metamod_votes_voter ON public.forum_metamod_votes USING btree (voter_id, created_at);


--
-- Name: idx_forum_mod_actions_post; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_mod_actions_post ON public.forum_mod_actions USING btree (post_id, created_at DESC);


--
-- Name: idx_forum_post_reactions_post; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_post_reactions_post ON public.forum_post_reactions USING btree (post_id);


--
-- Name: idx_forum_post_reactions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_post_reactions_user ON public.forum_post_reactions USING btree (user_id);


--
-- Name: idx_forum_post_reactions_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_post_reactions_user_created ON public.forum_post_reactions USING btree (user_id, created_at DESC);


--
-- Name: idx_forum_posts_mod_queue; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_posts_mod_queue ON public.forum_posts USING btree (score, created_at, id) WHERE ((deleted_at IS NULL) AND (is_hidden = false) AND (hidden_until IS NULL));


--
-- Name: idx_forum_posts_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_posts_search ON public.forum_posts USING gin (search_vector);


--
-- Name: idx_forum_posts_topic; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_posts_topic ON public.forum_posts USING btree (topic_id, created_at, id) WHERE ((deleted_at IS NULL) AND (is_hidden = false));


--
-- Name: idx_forum_topics_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_topics_category ON public.forum_topics USING btree (category_id, last_activity_at DESC) WHERE ((deleted_at IS NULL) AND (is_hidden = false));


--
-- Name: idx_forum_topics_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_topics_search ON public.forum_topics USING gin (search_vector);


--
-- Name: idx_forum_topics_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_forum_topics_slug ON public.forum_topics USING btree (topic_slug) WHERE (topic_slug IS NOT NULL);


--
-- Name: idx_forum_topics_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_forum_topics_status ON public.forum_topics USING btree (status);


--
-- Name: idx_heal_extractions_failure; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_heal_extractions_failure ON public.heal_extractions USING btree (failure_id);


--
-- Name: idx_heal_extractions_trusted_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_heal_extractions_trusted_created ON public.heal_extractions USING btree (trusted, created_at);


--
-- Name: idx_kudos_guest_one_per_work; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_kudos_guest_one_per_work ON public.kudos USING btree (work_id) WHERE (user_id IS NULL);


--
-- Name: idx_kudos_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_kudos_work ON public.kudos USING btree (work_id);


--
-- Name: idx_lb_monthly_rank; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lb_monthly_rank ON public.leaderboard_monthly USING btree (month_start, rank);


--
-- Name: idx_lb_weekly_rank; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lb_weekly_rank ON public.leaderboard_weekly USING btree (week_start, rank);


--
-- Name: idx_marginalia_chapter; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_marginalia_chapter ON public.marginalia USING btree (work_id, chapter_index);


--
-- Name: idx_merge_proposals_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_merge_proposals_status ON public.author_merge_proposals USING btree (status);


--
-- Name: idx_merge_proposals_unique_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_merge_proposals_unique_pending ON public.author_merge_proposals USING btree (source_url, target_profile_id) WHERE (status = 'pending'::text);


--
-- Name: idx_modlog_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_modlog_action ON public.modlog USING btree (action);


--
-- Name: idx_modlog_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_modlog_actor ON public.modlog USING btree (actor_id);


--
-- Name: idx_modlog_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_modlog_created ON public.modlog USING btree (created_at DESC);


--
-- Name: idx_notifications_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_user ON public.notifications USING btree (user_id, created_at DESC) WHERE (is_read = false);


--
-- Name: idx_notifications_user_all; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_user_all ON public.notifications USING btree (user_id, created_at DESC);


--
-- Name: idx_pending_exports_status_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pending_exports_status_created ON public.pending_exports USING btree (status, created_at);


--
-- Name: idx_pending_exports_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pending_exports_url ON public.pending_exports USING btree (url);


--
-- Name: idx_precomputed_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_precomputed_url ON public.precomputed_recommendations USING btree (url_id, rank);


--
-- Name: idx_pseuds_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pseuds_name ON public.pseuds USING btree (lower(name));


--
-- Name: idx_pseuds_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pseuds_user ON public.pseuds USING btree (user_id);


--
-- Name: idx_ratings_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ratings_work ON public.work_ratings USING btree (url_id, rating);


--
-- Name: idx_reactions_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reactions_target ON public.reactions USING btree (target_type, target_id);


--
-- Name: idx_reactions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reactions_user ON public.reactions USING btree (user_id);


--
-- Name: idx_reading_history_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reading_history_user ON public.reading_history USING btree (user_id, visited_at DESC);


--
-- Name: idx_reading_history_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reading_history_work ON public.reading_history USING btree (work_id);


--
-- Name: idx_reading_list_items_list; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reading_list_items_list ON public.reading_list_items USING btree (list_id, "position");


--
-- Name: idx_reading_lists_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reading_lists_user ON public.reading_lists USING btree (user_id, deleted_at);


--
-- Name: idx_reading_stats_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reading_stats_status ON public.reading_stats USING btree (user_id, status);


--
-- Name: idx_reading_stats_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reading_stats_user ON public.reading_stats USING btree (user_id, last_read_at DESC);


--
-- Name: idx_rec_embeddings_hnsw; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rec_embeddings_hnsw ON public.rec_embeddings USING hnsw (embedding public.vector_cosine_ops);


--
-- Name: idx_rec_impressions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rec_impressions_user ON public.rec_impressions USING btree (user_id, shown_at DESC);


--
-- Name: idx_rec_impressions_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rec_impressions_work ON public.rec_impressions USING btree (work_id);


--
-- Name: idx_rec_training_runs_strategy; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rec_training_runs_strategy ON public.rec_training_runs USING btree (strategy, started_at DESC);


--
-- Name: idx_recommendation_suggestions_url_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_recommendation_suggestions_url_status ON public.recommendation_suggestions USING btree (url_id, status);


--
-- Name: idx_reg_apps_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reg_apps_status ON public.registration_applications USING btree (status, created_at DESC);


--
-- Name: idx_rep_events_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_rep_events_user ON public.reputation_events USING btree (user_id, created_at DESC);


--
-- Name: idx_request_log_client_actions; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_client_actions ON public.request_log USING btree (client_id, created DESC) WHERE (client_id IS NOT NULL);


--
-- Name: idx_request_log_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_client_id ON public.request_log USING btree (client_id) WHERE (client_id IS NOT NULL);


--
-- Name: idx_request_log_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_created ON public.request_log USING btree (created DESC);


--
-- Name: idx_request_log_date_export; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_date_export ON public.request_log USING btree (created) WHERE ((export_file_name IS NOT NULL) AND (etype = 'epub'::text));


--
-- Name: idx_request_log_errors; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_errors ON public.request_log USING btree (created) WHERE (status = 'error'::text);


--
-- Name: idx_request_log_ip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_ip ON public.request_log USING btree (ip);


--
-- Name: idx_request_log_scraper_health; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_scraper_health ON public.request_log USING btree (source_url, created) WHERE (source_url IS NOT NULL);


--
-- Name: idx_request_log_url_id_etype_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_request_log_url_id_etype_created ON public.request_log USING btree (url_id, etype, created);


--
-- Name: idx_reviews_work_rating; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reviews_work_rating ON public.reviews USING btree (work_id, rating);


--
-- Name: idx_reviews_work_visible; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reviews_work_visible ON public.reviews USING btree (work_id, created_at DESC) WHERE (deleted_at IS NULL);


--
-- Name: idx_scrape_failures_domain_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scrape_failures_domain_created ON public.scrape_failures USING btree (domain, created_at);


--
-- Name: idx_scrape_failures_fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scrape_failures_fingerprint ON public.scrape_failures USING btree (fingerprint);


--
-- Name: idx_search_queries_query; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_search_queries_query ON public.search_queries USING btree (query);


--
-- Name: idx_search_queries_ts; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_search_queries_ts ON public.search_queries USING btree (ts DESC);


--
-- Name: idx_search_queries_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_search_queries_user ON public.search_queries USING btree (user_id, ts DESC);


--
-- Name: idx_series_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_name ON public.series USING btree (lower(name));


--
-- Name: idx_series_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_user ON public.series USING btree (user_id) WHERE (user_id IS NOT NULL);


--
-- Name: idx_series_works_position; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_works_position ON public.series_works USING btree (series_id, "position");


--
-- Name: idx_series_works_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_works_work ON public.series_works USING btree (work_id);


--
-- Name: idx_skins_public; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_skins_public ON public.skins USING btree (is_public) WHERE (is_public = true);


--
-- Name: idx_skins_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_skins_user ON public.skins USING btree (user_id);


--
-- Name: idx_tag_embeddings_embedding; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tag_embeddings_embedding ON public.tag_embeddings USING ivfflat (embedding public.vector_cosine_ops) WITH (lists='10');


--
-- Name: idx_tag_flags_unresolved; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tag_flags_unresolved ON public.tag_flags USING btree (created_at) WHERE (resolved = false);


--
-- Name: idx_tag_score_fixes_fixed_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tag_score_fixes_fixed_at ON public.tag_score_fixes USING btree (fixed_at);


--
-- Name: idx_tag_score_fixes_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tag_score_fixes_url ON public.tag_score_fixes USING btree (url_id);


--
-- Name: idx_tags_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tags_name ON public.tags USING btree (name);


--
-- Name: idx_tags_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tags_type ON public.tags USING btree (tag_type_id);


--
-- Name: idx_topic_views_topic_user; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_topic_views_topic_user ON public.forum_topic_views USING btree (topic_id, user_id);


--
-- Name: idx_topic_views_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_topic_views_user ON public.forum_topic_views USING btree (user_id, topic_id);


--
-- Name: idx_udp_user_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_udp_user_date ON public.user_daily_progress USING btree (user_id, quest_date);


--
-- Name: idx_usage_events_client_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_events_client_created ON public.usage_events USING btree (client_id, created_at);


--
-- Name: idx_usage_events_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_events_created ON public.usage_events USING btree (created_at);


--
-- Name: idx_usage_events_type_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_events_type_created ON public.usage_events USING btree (event_type, created_at);


--
-- Name: idx_user_invites_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_invites_created_by ON public.user_invites USING btree (created_by, created_at DESC);


--
-- Name: idx_user_recipes_public; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_recipes_public ON public.user_recipes USING btree (is_public) WHERE (is_public = true);


--
-- Name: idx_user_recipes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_recipes_user ON public.user_recipes USING btree (user_id);


--
-- Name: idx_user_reports_reporter; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_reports_reporter ON public.user_reports USING btree (reporter_id);


--
-- Name: idx_user_reports_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_reports_status ON public.user_reports USING btree (status, created_at DESC);


--
-- Name: idx_user_reports_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_reports_target ON public.user_reports USING btree (target_type, target_id);


--
-- Name: idx_user_site_credentials_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_site_credentials_user ON public.user_site_credentials USING btree (user_id);


--
-- Name: idx_users_reputation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_reputation ON public.users USING btree (reputation DESC);


--
-- Name: idx_work_rating_verifications_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_rating_verifications_work ON public.work_rating_verifications USING btree (work_id);


--
-- Name: idx_work_ratings_work_rating; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_ratings_work_rating ON public.work_ratings USING btree (work_id, rating);


--
-- Name: idx_work_shelves_shelf; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_shelves_shelf ON public.work_shelves USING btree (shelf_id);


--
-- Name: idx_work_shelves_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_shelves_work ON public.work_shelves USING btree (work_id);


--
-- Name: idx_work_translations_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_translations_status ON public.work_translations USING btree (status);


--
-- Name: idx_work_translations_work; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_translations_work ON public.work_translations USING btree (work_id);


--
-- Name: idx_works_uploader; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_works_uploader ON public.works USING btree (uploader_id);


--
-- Name: idx_xp_events_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_xp_events_user_created ON public.xp_events USING btree (user_id, created_at);


--
-- Name: recommendation_votes_suggestion_ip_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX recommendation_votes_suggestion_ip_key ON public.recommendation_votes USING btree (suggestion_id, voter_ip);


--
-- Name: recommendation_votes_suggestion_user_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX recommendation_votes_suggestion_user_key ON public.recommendation_votes USING btree (suggestion_id, user_id) WHERE (user_id IS NOT NULL);


--
-- Name: users_email_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX users_email_key ON public.users USING btree (email) WHERE (email <> ''::text);


--
-- Name: fic_tag_votes trg_fic_tag_vote_delete; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_fic_tag_vote_delete AFTER DELETE ON public.fic_tag_votes FOR EACH ROW EXECUTE FUNCTION public.update_fic_tag_score();


--
-- Name: fic_tag_votes trg_fic_tag_vote_insert; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_fic_tag_vote_insert AFTER INSERT ON public.fic_tag_votes FOR EACH ROW EXECUTE FUNCTION public.update_fic_tag_score();


--
-- Name: fic_tag_votes trg_fic_tag_vote_update; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_fic_tag_vote_update AFTER UPDATE ON public.fic_tag_votes FOR EACH ROW EXECUTE FUNCTION public.update_fic_tag_score();


--
-- Name: arena_votes arena_votes_best_cluster_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.arena_votes
    ADD CONSTRAINT arena_votes_best_cluster_id_fkey FOREIGN KEY (best_cluster_id) REFERENCES public.feature_clusters(id);


--
-- Name: arena_votes arena_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.arena_votes
    ADD CONSTRAINT arena_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: arena_votes arena_votes_worst_cluster_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.arena_votes
    ADD CONSTRAINT arena_votes_worst_cluster_id_fkey FOREIGN KEY (worst_cluster_id) REFERENCES public.feature_clusters(id);


--
-- Name: author_merge_proposals author_merge_proposals_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_merge_proposals
    ADD CONSTRAINT author_merge_proposals_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: author_merge_proposals author_merge_proposals_proposed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_merge_proposals
    ADD CONSTRAINT author_merge_proposals_proposed_by_fkey FOREIGN KEY (proposed_by) REFERENCES public.users(id);


--
-- Name: author_merge_proposals author_merge_proposals_target_profile_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_merge_proposals
    ADD CONSTRAINT author_merge_proposals_target_profile_id_fkey FOREIGN KEY (target_profile_id) REFERENCES public.author_profiles(id) ON DELETE CASCADE;


--
-- Name: author_profile_links author_profile_links_profile_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_profile_links
    ADD CONSTRAINT author_profile_links_profile_id_fkey FOREIGN KEY (profile_id) REFERENCES public.author_profiles(id) ON DELETE CASCADE;


--
-- Name: author_socials author_socials_profile_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.author_socials
    ADD CONSTRAINT author_socials_profile_id_fkey FOREIGN KEY (profile_id) REFERENCES public.author_profiles(id) ON DELETE CASCADE;


--
-- Name: auto_merge_log auto_merge_log_matched_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.auto_merge_log
    ADD CONSTRAINT auto_merge_log_matched_work_id_fkey FOREIGN KEY (matched_work_id) REFERENCES public.works(id);


--
-- Name: blocked_users blocked_users_blocked_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blocked_users
    ADD CONSTRAINT blocked_users_blocked_user_id_fkey FOREIGN KEY (blocked_user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: blocked_users blocked_users_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blocked_users
    ADD CONSTRAINT blocked_users_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: bookmarks bookmarks_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bookmarks
    ADD CONSTRAINT bookmarks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: bookmarks bookmarks_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bookmarks
    ADD CONSTRAINT bookmarks_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id);


--
-- Name: challenge_assignments challenge_assignments_challenge_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_assignments
    ADD CONSTRAINT challenge_assignments_challenge_id_fkey FOREIGN KEY (challenge_id) REFERENCES public.challenges(id) ON DELETE CASCADE;


--
-- Name: challenge_assignments challenge_assignments_giver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_assignments
    ADD CONSTRAINT challenge_assignments_giver_id_fkey FOREIGN KEY (giver_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: challenge_assignments challenge_assignments_recipient_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_assignments
    ADD CONSTRAINT challenge_assignments_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: challenge_assignments challenge_assignments_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_assignments
    ADD CONSTRAINT challenge_assignments_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE SET NULL;


--
-- Name: challenge_signups challenge_signups_challenge_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_signups
    ADD CONSTRAINT challenge_signups_challenge_id_fkey FOREIGN KEY (challenge_id) REFERENCES public.challenges(id) ON DELETE CASCADE;


--
-- Name: challenge_signups challenge_signups_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenge_signups
    ADD CONSTRAINT challenge_signups_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: challenges challenges_collection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.challenges
    ADD CONSTRAINT challenges_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES public.reading_lists(id) ON DELETE CASCADE;


--
-- Name: chapter_translation_versions chapter_translation_versions_locale_code_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translation_versions
    ADD CONSTRAINT chapter_translation_versions_locale_code_fkey FOREIGN KEY (locale_code) REFERENCES public.locales(code);


--
-- Name: chapter_translation_versions chapter_translation_versions_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translation_versions
    ADD CONSTRAINT chapter_translation_versions_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: chapter_translation_versions chapter_translation_versions_translated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translation_versions
    ADD CONSTRAINT chapter_translation_versions_translated_by_fkey FOREIGN KEY (translated_by) REFERENCES public.users(id);


--
-- Name: chapter_translation_versions chapter_translation_versions_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translation_versions
    ADD CONSTRAINT chapter_translation_versions_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: chapter_translations chapter_translations_locale_code_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translations
    ADD CONSTRAINT chapter_translations_locale_code_fkey FOREIGN KEY (locale_code) REFERENCES public.locales(code);


--
-- Name: chapter_translations chapter_translations_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translations
    ADD CONSTRAINT chapter_translations_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: chapter_translations chapter_translations_translated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translations
    ADD CONSTRAINT chapter_translations_translated_by_fkey FOREIGN KEY (translated_by) REFERENCES public.users(id);


--
-- Name: chapter_translations chapter_translations_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chapter_translations
    ADD CONSTRAINT chapter_translations_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: collection_bookmarks collection_bookmarks_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_bookmarks
    ADD CONSTRAINT collection_bookmarks_list_id_fkey FOREIGN KEY (list_id) REFERENCES public.reading_lists(id) ON DELETE CASCADE;


--
-- Name: collection_bookmarks collection_bookmarks_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_bookmarks
    ADD CONSTRAINT collection_bookmarks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: collection_item_requests collection_item_requests_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_requests
    ADD CONSTRAINT collection_item_requests_list_id_fkey FOREIGN KEY (list_id) REFERENCES public.reading_lists(id) ON DELETE CASCADE;


--
-- Name: collection_item_requests collection_item_requests_requested_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_requests
    ADD CONSTRAINT collection_item_requests_requested_by_fkey FOREIGN KEY (requested_by) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: collection_item_requests collection_item_requests_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.collection_item_requests
    ADD CONSTRAINT collection_item_requests_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: comment_triage comment_triage_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comment_triage
    ADD CONSTRAINT comment_triage_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.comments(id) ON DELETE CASCADE;


--
-- Name: comments comments_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.comments(id) ON DELETE CASCADE;


--
-- Name: comments comments_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: comments comments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: comments comments_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id);


--
-- Name: content_scan content_scan_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_scan
    ADD CONSTRAINT content_scan_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: creatorships creatorships_pseud_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_pseud_id_fkey FOREIGN KEY (pseud_id) REFERENCES public.pseuds(id) ON DELETE CASCADE;


--
-- Name: creatorships creatorships_series_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_series_id_fkey FOREIGN KEY (series_id) REFERENCES public.series(id) ON DELETE CASCADE;


--
-- Name: creatorships creatorships_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: curator_content_overrides curator_content_overrides_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_content_overrides
    ADD CONSTRAINT curator_content_overrides_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: curator_fix_proposals curator_fix_proposals_decided_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_fix_proposals
    ADD CONSTRAINT curator_fix_proposals_decided_by_fkey FOREIGN KEY (decided_by) REFERENCES public.users(id);


--
-- Name: curator_fix_proposals curator_fix_proposals_proposed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_fix_proposals
    ADD CONSTRAINT curator_fix_proposals_proposed_by_fkey FOREIGN KEY (proposed_by) REFERENCES public.users(id);


--
-- Name: curator_fix_votes curator_fix_votes_proposal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_fix_votes
    ADD CONSTRAINT curator_fix_votes_proposal_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.curator_fix_proposals(id) ON DELETE CASCADE;


--
-- Name: curator_fix_votes curator_fix_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_fix_votes
    ADD CONSTRAINT curator_fix_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: curator_metadata_proposals curator_metadata_proposals_decided_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_proposals
    ADD CONSTRAINT curator_metadata_proposals_decided_by_fkey FOREIGN KEY (decided_by) REFERENCES public.users(id);


--
-- Name: curator_metadata_proposals curator_metadata_proposals_proposed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_proposals
    ADD CONSTRAINT curator_metadata_proposals_proposed_by_fkey FOREIGN KEY (proposed_by) REFERENCES public.users(id);


--
-- Name: curator_metadata_proposals curator_metadata_proposals_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_proposals
    ADD CONSTRAINT curator_metadata_proposals_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: curator_metadata_votes curator_metadata_votes_proposal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_votes
    ADD CONSTRAINT curator_metadata_votes_proposal_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.curator_metadata_proposals(id) ON DELETE CASCADE;


--
-- Name: curator_metadata_votes curator_metadata_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.curator_metadata_votes
    ADD CONSTRAINT curator_metadata_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: exp_events exp_events_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.exp_events
    ADD CONSTRAINT exp_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: export_log export_log_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_log
    ADD CONSTRAINT export_log_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id);


--
-- Name: extensions extensions_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.extensions
    ADD CONSTRAINT extensions_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.users(id);


--
-- Name: feature_suggestions feature_suggestions_cluster_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_suggestions
    ADD CONSTRAINT feature_suggestions_cluster_id_fkey FOREIGN KEY (cluster_id) REFERENCES public.feature_clusters(id) ON DELETE SET NULL;


--
-- Name: feature_suggestions feature_suggestions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_suggestions
    ADD CONSTRAINT feature_suggestions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: features features_requires_feature_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.features
    ADD CONSTRAINT features_requires_feature_fkey FOREIGN KEY (requires_feature) REFERENCES public.features(slug);


--
-- Name: fic_blacklist fic_blacklist_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_blacklist
    ADD CONSTRAINT fic_blacklist_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id);


--
-- Name: fic_bookmark_cooccur fic_bookmark_cooccur_work_a_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_bookmark_cooccur
    ADD CONSTRAINT fic_bookmark_cooccur_work_a_fkey FOREIGN KEY (work_a) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: fic_bookmark_cooccur fic_bookmark_cooccur_work_b_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_bookmark_cooccur
    ADD CONSTRAINT fic_bookmark_cooccur_work_b_fkey FOREIGN KEY (work_b) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: fic_bookmarks fic_bookmarks_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_bookmarks
    ADD CONSTRAINT fic_bookmarks_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: fic_request_answer_votes fic_request_answer_votes_answer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answer_votes
    ADD CONSTRAINT fic_request_answer_votes_answer_id_fkey FOREIGN KEY (answer_id) REFERENCES public.fic_request_answers(id);


--
-- Name: fic_request_answer_votes fic_request_answer_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answer_votes
    ADD CONSTRAINT fic_request_answer_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: fic_request_answers fic_request_answers_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answers
    ADD CONSTRAINT fic_request_answers_request_id_fkey FOREIGN KEY (request_id) REFERENCES public.fic_requests(id);


--
-- Name: fic_request_answers fic_request_answers_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answers
    ADD CONSTRAINT fic_request_answers_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: fic_request_answers fic_request_answers_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_answers
    ADD CONSTRAINT fic_request_answers_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id);


--
-- Name: fic_request_upvotes fic_request_upvotes_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_upvotes
    ADD CONSTRAINT fic_request_upvotes_request_id_fkey FOREIGN KEY (request_id) REFERENCES public.fic_requests(id) ON DELETE CASCADE;


--
-- Name: fic_request_upvotes fic_request_upvotes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_request_upvotes
    ADD CONSTRAINT fic_request_upvotes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: fic_requests fic_requests_accepted_answer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_requests
    ADD CONSTRAINT fic_requests_accepted_answer_id_fkey FOREIGN KEY (accepted_answer_id) REFERENCES public.fic_request_answers(id);


--
-- Name: fic_requests fic_requests_seed_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_requests
    ADD CONSTRAINT fic_requests_seed_work_id_fkey FOREIGN KEY (seed_work_id) REFERENCES public.works(id);


--
-- Name: fic_requests fic_requests_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_requests
    ADD CONSTRAINT fic_requests_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: fic_tag_votes fic_tag_votes_url_id_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_tag_votes
    ADD CONSTRAINT fic_tag_votes_url_id_tag_id_fkey FOREIGN KEY (url_id, tag_id) REFERENCES public.fic_tags(url_id, tag_id) ON DELETE CASCADE;


--
-- Name: fic_tags fic_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_tags
    ADD CONSTRAINT fic_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: fic_tags fic_tags_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_tags
    ADD CONSTRAINT fic_tags_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: fic_works fic_works_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_works
    ADD CONSTRAINT fic_works_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: fic_info fk_fic_info_work_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fic_info
    ADD CONSTRAINT fk_fic_info_work_id FOREIGN KEY (work_id) REFERENCES public.works(id);


--
-- Name: follows follows_followee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows
    ADD CONSTRAINT follows_followee_id_fkey FOREIGN KEY (followee_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: follows follows_follower_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows
    ADD CONSTRAINT follows_follower_id_fkey FOREIGN KEY (follower_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: follows follows_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.follows
    ADD CONSTRAINT follows_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: forum_bans forum_bans_banned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_bans
    ADD CONSTRAINT forum_bans_banned_by_fkey FOREIGN KEY (banned_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: forum_bans forum_bans_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_bans
    ADD CONSTRAINT forum_bans_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.forum_categories(id) ON DELETE CASCADE;


--
-- Name: forum_bans forum_bans_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_bans
    ADD CONSTRAINT forum_bans_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_edit_proposals forum_edit_proposals_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_edit_proposals
    ADD CONSTRAINT forum_edit_proposals_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_edit_proposals forum_edit_proposals_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_edit_proposals
    ADD CONSTRAINT forum_edit_proposals_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: forum_follows forum_follows_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_follows
    ADD CONSTRAINT forum_follows_topic_id_fkey FOREIGN KEY (topic_id) REFERENCES public.forum_topics(id) ON DELETE CASCADE;


--
-- Name: forum_follows forum_follows_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_follows
    ADD CONSTRAINT forum_follows_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_metamod_votes forum_metamod_votes_mod_action_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_metamod_votes
    ADD CONSTRAINT forum_metamod_votes_mod_action_id_fkey FOREIGN KEY (mod_action_id) REFERENCES public.forum_mod_actions(id) ON DELETE CASCADE;


--
-- Name: forum_metamod_votes forum_metamod_votes_voter_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_metamod_votes
    ADD CONSTRAINT forum_metamod_votes_voter_id_fkey FOREIGN KEY (voter_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_mod_actions forum_mod_actions_moderator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_mod_actions
    ADD CONSTRAINT forum_mod_actions_moderator_id_fkey FOREIGN KEY (moderator_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_mod_actions forum_mod_actions_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_mod_actions
    ADD CONSTRAINT forum_mod_actions_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.forum_posts(id) ON DELETE CASCADE;


--
-- Name: forum_mod_grants forum_mod_grants_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_mod_grants
    ADD CONSTRAINT forum_mod_grants_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_post_reactions forum_post_reactions_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_post_reactions
    ADD CONSTRAINT forum_post_reactions_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.forum_posts(id) ON DELETE CASCADE;


--
-- Name: forum_post_reactions forum_post_reactions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_post_reactions
    ADD CONSTRAINT forum_post_reactions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_posts forum_posts_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_posts forum_posts_quote_of_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_quote_of_fkey FOREIGN KEY (quote_of) REFERENCES public.forum_posts(id) ON DELETE SET NULL;


--
-- Name: forum_posts forum_posts_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_topic_id_fkey FOREIGN KEY (topic_id) REFERENCES public.forum_topics(id) ON DELETE CASCADE;


--
-- Name: forum_read_state forum_read_state_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_read_state
    ADD CONSTRAINT forum_read_state_topic_id_fkey FOREIGN KEY (topic_id) REFERENCES public.forum_topics(id) ON DELETE CASCADE;


--
-- Name: forum_read_state forum_read_state_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_read_state
    ADD CONSTRAINT forum_read_state_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_topic_views forum_topic_views_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_topic_views
    ADD CONSTRAINT forum_topic_views_topic_id_fkey FOREIGN KEY (topic_id) REFERENCES public.forum_topics(id) ON DELETE CASCADE;


--
-- Name: forum_topics forum_topics_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_topics
    ADD CONSTRAINT forum_topics_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_topics forum_topics_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_topics
    ADD CONSTRAINT forum_topics_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.forum_categories(id) ON DELETE CASCADE;


--
-- Name: heal_extractions heal_extractions_agent_run_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.heal_extractions
    ADD CONSTRAINT heal_extractions_agent_run_id_fkey FOREIGN KEY (agent_run_id) REFERENCES public.agent_runs(id) ON DELETE SET NULL;


--
-- Name: heal_extractions heal_extractions_failure_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.heal_extractions
    ADD CONSTRAINT heal_extractions_failure_id_fkey FOREIGN KEY (failure_id) REFERENCES public.scrape_failures(id) ON DELETE CASCADE;


--
-- Name: kudos kudos_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kudos
    ADD CONSTRAINT kudos_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: kudos kudos_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.kudos
    ADD CONSTRAINT kudos_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: leaderboard_monthly leaderboard_monthly_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_monthly
    ADD CONSTRAINT leaderboard_monthly_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: leaderboard_weekly leaderboard_weekly_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_weekly
    ADD CONSTRAINT leaderboard_weekly_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: login_streaks login_streaks_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.login_streaks
    ADD CONSTRAINT login_streaks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: marginalia marginalia_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.marginalia
    ADD CONSTRAINT marginalia_topic_id_fkey FOREIGN KEY (topic_id) REFERENCES public.forum_topics(id) ON DELETE CASCADE;


--
-- Name: marginalia marginalia_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.marginalia
    ADD CONSTRAINT marginalia_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: notification_preferences notification_preferences_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_preferences
    ADD CONSTRAINT notification_preferences_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: notifications notifications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: opds_shelf_items opds_shelf_items_shelf_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.opds_shelf_items
    ADD CONSTRAINT opds_shelf_items_shelf_id_fkey FOREIGN KEY (shelf_id) REFERENCES public.opds_shelves(id) ON DELETE CASCADE;


--
-- Name: opds_shelf_items opds_shelf_items_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.opds_shelf_items
    ADD CONSTRAINT opds_shelf_items_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: precomputed_recommendations precomputed_recommendations_recommended_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.precomputed_recommendations
    ADD CONSTRAINT precomputed_recommendations_recommended_url_id_fkey FOREIGN KEY (recommended_url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: precomputed_recommendations precomputed_recommendations_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.precomputed_recommendations
    ADD CONSTRAINT precomputed_recommendations_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: pseuds pseuds_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pseuds
    ADD CONSTRAINT pseuds_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reactions reactions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reactions
    ADD CONSTRAINT reactions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reading_history reading_history_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_history
    ADD CONSTRAINT reading_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reading_history reading_history_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_history
    ADD CONSTRAINT reading_history_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: reading_list_items reading_list_items_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_list_items
    ADD CONSTRAINT reading_list_items_list_id_fkey FOREIGN KEY (list_id) REFERENCES public.reading_lists(id) ON DELETE CASCADE;


--
-- Name: reading_list_items reading_list_items_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_list_items
    ADD CONSTRAINT reading_list_items_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: reading_lists reading_lists_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_lists
    ADD CONSTRAINT reading_lists_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reading_stats reading_stats_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_stats
    ADD CONSTRAINT reading_stats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reading_stats reading_stats_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reading_stats
    ADD CONSTRAINT reading_stats_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: rec_bandit_arms rec_bandit_arms_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_bandit_arms
    ADD CONSTRAINT rec_bandit_arms_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: rec_embeddings rec_embeddings_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_embeddings
    ADD CONSTRAINT rec_embeddings_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: rec_impressions rec_impressions_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_impressions
    ADD CONSTRAINT rec_impressions_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: rec_transitions rec_transitions_from_work_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_transitions
    ADD CONSTRAINT rec_transitions_from_work_fkey FOREIGN KEY (from_work) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: rec_transitions rec_transitions_to_work_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_transitions
    ADD CONSTRAINT rec_transitions_to_work_fkey FOREIGN KEY (to_work) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: rec_user_clusters rec_user_clusters_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_user_clusters
    ADD CONSTRAINT rec_user_clusters_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: rec_user_curator_align rec_user_curator_align_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_user_curator_align
    ADD CONSTRAINT rec_user_curator_align_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: rec_user_signals rec_user_signals_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.rec_user_signals
    ADD CONSTRAINT rec_user_signals_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: recommendation_suggestions recommendation_suggestions_suggested_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_suggestions
    ADD CONSTRAINT recommendation_suggestions_suggested_url_id_fkey FOREIGN KEY (suggested_url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: recommendation_suggestions recommendation_suggestions_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_suggestions
    ADD CONSTRAINT recommendation_suggestions_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: recommendation_suggestions recommendation_suggestions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_suggestions
    ADD CONSTRAINT recommendation_suggestions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: recommendation_votes recommendation_votes_suggestion_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_votes
    ADD CONSTRAINT recommendation_votes_suggestion_id_fkey FOREIGN KEY (suggestion_id) REFERENCES public.recommendation_suggestions(id) ON DELETE CASCADE;


--
-- Name: recommendation_votes recommendation_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recommendation_votes
    ADD CONSTRAINT recommendation_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: registration_applications registration_applications_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.registration_applications
    ADD CONSTRAINT registration_applications_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: registration_applications registration_applications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.registration_applications
    ADD CONSTRAINT registration_applications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reputation_events reputation_events_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reputation_events
    ADD CONSTRAINT reputation_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: request_log request_log_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.request_log
    ADD CONSTRAINT request_log_source_id_fkey FOREIGN KEY (source_id) REFERENCES public.request_source(id);


--
-- Name: reviews reviews_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviews
    ADD CONSTRAINT reviews_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reviews reviews_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviews
    ADD CONSTRAINT reviews_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: search_queries search_queries_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.search_queries
    ADD CONSTRAINT search_queries_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: series series_pseud_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series
    ADD CONSTRAINT series_pseud_id_fkey FOREIGN KEY (pseud_id) REFERENCES public.pseuds(id) ON DELETE SET NULL;


--
-- Name: series series_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series
    ADD CONSTRAINT series_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: series_works series_works_series_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series_works
    ADD CONSTRAINT series_works_series_id_fkey FOREIGN KEY (series_id) REFERENCES public.series(id) ON DELETE CASCADE;


--
-- Name: series_works series_works_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series_works
    ADD CONSTRAINT series_works_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: shelves shelves_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.shelves
    ADD CONSTRAINT shelves_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: skins skins_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.skins
    ADD CONSTRAINT skins_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: tag_aliases tag_aliases_canonical_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT tag_aliases_canonical_tag_id_fkey FOREIGN KEY (canonical_tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_embeddings tag_embeddings_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_embeddings
    ADD CONSTRAINT tag_embeddings_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_flags tag_flags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_flags
    ADD CONSTRAINT tag_flags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_flags tag_flags_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_flags
    ADD CONSTRAINT tag_flags_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: tag_flags tag_flags_url_id_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_flags
    ADD CONSTRAINT tag_flags_url_id_tag_id_fkey FOREIGN KEY (url_id, tag_id) REFERENCES public.fic_tags(url_id, tag_id) ON DELETE CASCADE;


--
-- Name: tag_score_fixes tag_score_fixes_fixed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_score_fixes
    ADD CONSTRAINT tag_score_fixes_fixed_by_fkey FOREIGN KEY (fixed_by) REFERENCES public.users(id);


--
-- Name: tag_score_fixes tag_score_fixes_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_score_fixes
    ADD CONSTRAINT tag_score_fixes_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;


--
-- Name: tag_score_fixes tag_score_fixes_url_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_score_fixes
    ADD CONSTRAINT tag_score_fixes_url_id_fkey FOREIGN KEY (url_id) REFERENCES public.fic_info(id) ON DELETE CASCADE;


--
-- Name: tags tags_tag_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_tag_type_id_fkey FOREIGN KEY (tag_type_id) REFERENCES public.tag_types(id);


--
-- Name: translations translations_locale_code_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.translations
    ADD CONSTRAINT translations_locale_code_fkey FOREIGN KEY (locale_code) REFERENCES public.locales(code);


--
-- Name: user_badges user_badges_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_badges
    ADD CONSTRAINT user_badges_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_daily_progress user_daily_progress_quest_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_daily_progress
    ADD CONSTRAINT user_daily_progress_quest_id_fkey FOREIGN KEY (quest_id) REFERENCES public.daily_quests(id) ON DELETE CASCADE;


--
-- Name: user_daily_progress user_daily_progress_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_daily_progress
    ADD CONSTRAINT user_daily_progress_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_features user_features_feature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_features
    ADD CONSTRAINT user_features_feature_id_fkey FOREIGN KEY (feature_id) REFERENCES public.features(id) ON DELETE CASCADE;


--
-- Name: user_features user_features_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_features
    ADD CONSTRAINT user_features_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_invites user_invites_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_invites
    ADD CONSTRAINT user_invites_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_invites user_invites_used_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_invites
    ADD CONSTRAINT user_invites_used_by_fkey FOREIGN KEY (used_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: user_layouts user_layouts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_layouts
    ADD CONSTRAINT user_layouts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_prefs user_prefs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_prefs
    ADD CONSTRAINT user_prefs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_recipes user_recipes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_recipes
    ADD CONSTRAINT user_recipes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_reports user_reports_reporter_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_reports
    ADD CONSTRAINT user_reports_reporter_id_fkey FOREIGN KEY (reporter_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: user_site_credentials user_site_credentials_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_site_credentials
    ADD CONSTRAINT user_site_credentials_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_skins user_skins_skin_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_skins
    ADD CONSTRAINT user_skins_skin_id_fkey FOREIGN KEY (skin_id) REFERENCES public.skins(id) ON DELETE SET NULL;


--
-- Name: user_skins user_skins_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_skins
    ADD CONSTRAINT user_skins_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_views user_views_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_views
    ADD CONSTRAINT user_views_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: work_proposal_votes work_proposal_votes_proposal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposal_votes
    ADD CONSTRAINT work_proposal_votes_proposal_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.work_proposals(id) ON DELETE CASCADE;


--
-- Name: work_proposal_votes work_proposal_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposal_votes
    ADD CONSTRAINT work_proposal_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: work_proposals work_proposals_proposer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposals
    ADD CONSTRAINT work_proposals_proposer_id_fkey FOREIGN KEY (proposer_id) REFERENCES public.users(id);


--
-- Name: work_proposals work_proposals_source_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposals
    ADD CONSTRAINT work_proposals_source_work_id_fkey FOREIGN KEY (source_work_id) REFERENCES public.works(id);


--
-- Name: work_proposals work_proposals_target_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposals
    ADD CONSTRAINT work_proposals_target_work_id_fkey FOREIGN KEY (target_work_id) REFERENCES public.works(id);


--
-- Name: work_proposals work_proposals_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_proposals
    ADD CONSTRAINT work_proposals_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id);


--
-- Name: work_rating_verifications work_rating_verifications_verified_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_rating_verifications
    ADD CONSTRAINT work_rating_verifications_verified_by_fkey FOREIGN KEY (verified_by) REFERENCES public.users(id);


--
-- Name: work_rating_verifications work_rating_verifications_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_rating_verifications
    ADD CONSTRAINT work_rating_verifications_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: work_ratings work_ratings_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_ratings
    ADD CONSTRAINT work_ratings_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: work_ratings work_ratings_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_ratings
    ADD CONSTRAINT work_ratings_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id);


--
-- Name: work_shelves work_shelves_shelf_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_shelves
    ADD CONSTRAINT work_shelves_shelf_id_fkey FOREIGN KEY (shelf_id) REFERENCES public.shelves(id) ON DELETE CASCADE;


--
-- Name: work_shelves work_shelves_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_shelves
    ADD CONSTRAINT work_shelves_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: work_skins work_skins_skin_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_skins
    ADD CONSTRAINT work_skins_skin_id_fkey FOREIGN KEY (skin_id) REFERENCES public.skins(id) ON DELETE CASCADE;


--
-- Name: work_skins work_skins_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_skins
    ADD CONSTRAINT work_skins_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: work_translations work_translations_locale_code_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_translations
    ADD CONSTRAINT work_translations_locale_code_fkey FOREIGN KEY (locale_code) REFERENCES public.locales(code);


--
-- Name: work_translations work_translations_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_translations
    ADD CONSTRAINT work_translations_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: work_translations work_translations_translated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_translations
    ADD CONSTRAINT work_translations_translated_by_fkey FOREIGN KEY (translated_by) REFERENCES public.users(id);


--
-- Name: work_translations work_translations_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_translations
    ADD CONSTRAINT work_translations_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.works(id) ON DELETE CASCADE;


--
-- Name: works works_uploader_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.works
    ADD CONSTRAINT works_uploader_id_fkey FOREIGN KEY (uploader_id) REFERENCES public.users(id);


--
-- Name: xp_events xp_events_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.xp_events
    ADD CONSTRAINT xp_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
--


