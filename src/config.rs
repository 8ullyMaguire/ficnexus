use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    /// Where scraped fic bodies are cached as JSON blobs (the site is a
    /// cache of all gathered fanfiction). Defaults to the attach drive:
    /// /public/literature/fichub/bodies.
    pub body_cache_dir: PathBuf,
    /// Directory for the metadata-pass provenance store (META_DIR). Each
    /// scrape/fallback pass writes a versioned JSON file so future scrapers
    /// can re-merge previously scraped works without re-fetching bodies.
    /// Defaults to `<body_cache_dir>/../meta`.
    pub meta_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    /// Path to a MaxMind GeoLite2-ASN `.mmdb` file for datacenter-IP blocking.
    /// Empty = datacenter blocking disabled (fail-open).
    pub maxmind_db: Option<String>,
    // Recommender settings
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    // ── Pluggable recommendation platform (Stage 0+) ───────────────────
    /// `legacy` (default) preserves today's behavior exactly; `pluggable`
    /// routes through the strategy registry + RRF ranker.
    pub rec_engine_mode: RecEngineMode,
    /// `REC_STRATEGIES` — comma list `name:weight` (default `cooccur`).
    pub rec_strategies: String,
    /// Half-life (days) for the time-decayed co-occurrence strategy.
    pub rec_decay_halflife_days: f64,
    /// Embedding model for rec_embeddings (Ollama).
    pub rec_embed_model: String,
    /// Embedding dimension (nomic-embed-text = 384).
    pub rec_embed_dim: usize,
    /// Implicit-MF latent factor count (REC_MF_FACTORS).
    pub rec_mf_factors: usize,
    /// Implicit-MF ALS iterations (REC_MF_ITERS).
    pub rec_mf_iters: usize,
    /// Minimum unified signals before MF training runs.
    pub rec_mf_train_min_signals: usize,
    /// Bandit exploration slots per rec list (REC_BANDIT_SLOTS).
    pub rec_bandit_slots: usize,
    /// Batch pipeline cadence (hours) — embeddings/MF/clusters/transitions.
    pub rec_train_every_h: u32,
    /// Shadow mode: compute pluggable recs but return legacy output while
    /// logging impressions — the safe way to eval a new strategy.
    pub rec_shadow_mode: bool,
    /// Curator user id whose profile shapes low-signal users (REC_CURATOR_PRIOR).
    pub rec_curator_prior: Option<i32>,
    /// Curator-prior floor α (default 0.2): even a brand-new user keeps 20%
    /// of their own (empty) profile weight.
    pub rec_prior_floor: f64,
    /// Curator-prior τ: signals at which α reaches ~63% of the way to 1.0.
    pub rec_curator_tau: f64,
    /// Per-strategy HTTP timeout (Ollama/sidecar calls), seconds.
    pub rec_strategy_timeout_secs: u64,
    /// Optional external sidecar base URL (REC_EXTERNAL_URL) — the
    /// `external` strategy proxies scoring to a Python sidecar.
    pub rec_external_url: Option<String>,
    // Tagging v3 settings
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    /// Maximum manual-upload payload size in bytes (MAX_UPLOAD_BYTES,
    /// default 20 MiB). Reject larger uploads with HTTP 413.
    pub max_upload_bytes: usize,
    /// Wayback Machine fallback: when a native scrape fails with
    /// Blocked/NotFound/transient, try the Internet Archive snapshot.
    pub wayback_fallback_enabled: bool,
    /// Max age of a Wayback snapshot to accept (days). Prevents pulling
    /// a 10-year-old draft when a newer snapshot exists.
    pub wayback_max_snapshot_age_days: i32,
    /// CDX API requests per second (rate limit; default 1).
    pub wayback_cdx_rate_limit_per_sec: u64,
    /// FicHub.net metadata fallback kill-switch (default on).
    pub fichub_fallback_enabled: bool,
    /// Opportunistic tag top-up from Wayback snapshots (default off).
    pub tag_topup_enabled: bool,
    /// Use Redis for stateful rate limiting (default off = in-memory).
    pub rs_use_redis: bool,
    /// Master switch for the Redis rate limiter (default off).
    pub rs_rate_limiter_enabled: bool,
    /// Redis connection pool size (0 = use default of 10).
    pub rs_redis_pool_size: u32,
    /// Redis timeout in milliseconds.
    pub rs_redis_timeout_ms: u64,
    /// Reconnect delay in milliseconds.
    pub rs_redis_reconnect_ms: u64,
    /// Maximum retry attempts.
    pub rs_redis_max_retries: u32,
    /// Base retry delay in milliseconds.
    pub rs_redis_retry_base_ms: u64,
    /// Maximum retry delay cap in milliseconds.
    pub rs_redis_retry_cap_ms: u64,
    /// Redis key prefix (e.g. "fichub:").
    pub rs_redis_key_prefix: String,
    /// Redis hash tag pattern (e.g. "{rl}").
    pub rs_redis_hash_tag: String,
    /// Enable Redis compression (default off).
    pub rs_redis_compression: bool,
    /// Compression threshold in bytes.
    pub rs_redis_compression_threshold: usize,
    /// Compression level (1-9 for gzip).
    pub rs_redis_compression_level: i32,
    /// Compression algorithm (e.g. "gzip").
    pub rs_redis_compression_algo: String,
    /// Minimum size in bytes for compression to apply.
    pub rs_redis_compression_min_size: usize,
    /// Maximum size in bytes for compression.
    pub rs_redis_compression_max_size: usize,
    /// Compression window size.
    pub rs_redis_compression_window: usize,
    /// Compression memory level (1-9).
    pub rs_redis_compression_mem_level: i32,
    /// Compression strategy flags.
    pub rs_redis_compression_strategy: i32,
    /// Compression chunk size.
    pub rs_redis_compression_chunk_size: usize,
    /// Compression destination size hint.
    pub rs_redis_compression_dst_size: usize,
    /// OPDS shelf shared token for authenticated access
    pub opds_shelf_token: String,
    /// Base URL for OPDS feeds (used for xml:base on relative URLs).
    /// E.g. "https://fichub.example.com". If empty, no xml:base is set.
    pub opds_base_url: Option<String>,
    /// Public origin of this deployment (e.g. "https://fichub.example.com").
    /// Used to build absolute URLs in sitemaps. Falls back to
    /// `opds_base_url`, then "http://localhost:8000".
    pub public_origin: String,

    // ── Tiered rate limiting (anti-bot) ──────────────────────────────
    /// Capacity (burst tokens) of the download-tier bucket per IP.
    pub rl_download_capacity: f64,
    /// Refill rate of the download-tier bucket per IP, tokens/second.
    pub rl_download_flow: f64,
    /// Capacity (burst tokens) of the auth-tier bucket per IP.
    pub rl_auth_capacity: f64,
    /// Refill rate of the auth-tier bucket per IP, tokens/second.
    pub rl_auth_flow: f64,
    /// Capacity (burst tokens) of the loose/search-tier bucket per IP.
    pub rl_search_capacity: f64,
    /// Refill rate of the loose/search-tier bucket per IP, tokens/second.
    pub rl_search_flow: f64,
    /// Extra capacity granted to the per-`(ip, client_id)` bucket on top of
    /// the per-IP bucket. `0` disables the client keying entirely.
    pub rl_client_bonus_capacity: f64,
    /// Extra refill rate granted to the per-`(ip, client_id)` bucket.
    pub rl_client_bonus_flow: f64,
    /// Multiplier applied to the per-IP bucket capacity/flow when the request
    /// carries a client_id. Identified clients get their own strict bucket, so
    /// the per-IP bucket becomes a NAT-scaled anti-abuse ceiling instead of
    /// punishing shared NAT.
    pub rl_nat_multiplier: f64,
    /// Capacity of the shadowbanned-download bucket (much stricter than the
    /// normal download tier).
    pub rl_shadowban_capacity: f64,
    /// Refill rate of the shadowbanned-download bucket, tokens/second.
    pub rl_shadowban_flow: f64,
    /// TTL in seconds of a shadowban set entry (admin-unshadowbar after).
    pub rl_shadowban_ttl: u64,
    /// Master switch; when false the tiered limiter only applies the legacy
    /// static-delay behavior (test/dev mode, same as `dynamic_rate_limit`).
    pub rl_tiered_enabled: bool,
    /// Capacity of the static-delays tier (applies to all IPs, no per-IP bucket).
    pub rl_static_capacity: f64,
    /// Refill rate of the static-delays tier, tokens/second.
    pub rl_static_flow: f64,
    /// Extra capacity on top of static for identified clients.
    pub rl_static_client_capacity: f64,
    /// Extra refill rate for identified clients on the static tier.
    pub rl_static_client_flow: f64,
    /// ── Proof-of-work (PoW) challenge for shadowbanned clients ──────
    /// Leading zero BITS the SHA-256(challenge || nonce) hex must start with.
    /// 16 bits ≈ 65k hashes ≈ 0.1–1s on a laptop; 4 bits for cheap tests.
    pub pow_difficulty: u32,
    /// TTL in seconds of a stored solve (`fichub:pow:solved:<challenge>`)
    /// and of a challenge. 600 = 10 minutes.
    pub pow_ttl_secs: u64,
    /// Ollama API base URL for embeddings (Roadmap Consensus Engine).
    pub ollama_url: String,
    /// Ollama embedding model.
    pub ollama_embed_model: String,
    /// Ollama chat/generation model for tiny LLM apps (comment triage).
    pub ollama_chat_model: String,
    // ── Send-to-Kindle (SMTP) ────────────────────────────────────────
    /// SMTP relay host for sending Kindle emails (e.g. "smtp.gmail.com").
    pub smtp_host: String,
    /// SMTP relay port (usually 587 for STARTTLS).
    pub smtp_port: u16,
    /// SMTP username (empty if the relay needs no auth).
    pub smtp_user: String,
    /// SMTP password (empty if the relay needs no auth).
    pub smtp_pass: String,
    /// SMTP from-address used on outgoing emails.
    pub smtp_from: String,
    // ── Self-healing agent (docs/AGENTS.md) ─────────────────────────
    /// Master switch for the diagnose/fix agent loop (default false).
    pub agent_enabled: bool,
    /// Model name for agent calls (default `deepseek/deepseek-v4-flash`).
    pub agent_model: String,
    /// API key for the remote agent endpoint (COMMANDCODE_API_KEY or
    /// AGENT_API_KEY). `None` when neither is set.
    pub agent_api_key: Option<String>,
    /// Base URL of the OpenAI-compatible agent endpoint.
    pub agent_base_url: String,
    /// Local Ollama URL for agent on-the-fly extraction calls.
    pub agent_ollama_url: String,
    /// Translation config — §6 of docs/plans/translation-everything.md
    /// Master switch: translation endpoints + worker enabled
    pub translation_enabled: bool,
    /// LLM-backed machine translations enabled
    pub translate_llm_enabled: bool,
    /// Ollama model for translation (default: ollama_chat_model)
    pub translate_model: String,
    /// Max characters per translation chunk
    pub translate_chunk_chars: u32,
    /// Global budget: max machine translations per hour
    pub translate_global_budget_per_hour: u64,
    /// Per-user budget: max machine translations per hour
    pub translate_user_budget_per_hour: u64,
    /// Award points for approved machine translations (default false)
    pub translate_badge_machine: bool,
    /// Auto-approve high-confidence machine translations (default false)
    pub translate_auto_approve_machine: bool,
    /// Supported target locales for translation UI
    pub translate_locales: Vec<String>,
    /// Prompt once to set reading locale on first visit (default true)
    pub translate_reading_locale_ask: bool,
    /// SPA dictionary override from DB (default true)
    pub translate_ui_strings: bool,
    /// Keep machine rows visible after dismissal (default false)
    pub translate_keep_machine_on_dismiss: bool,
    /// Points awarded when a human-translation proposal is approved (M6)
    pub translate_points_approved: i64,
    /// Points awarded when an approved translation is improved then re-approved
    pub translate_points_improved: i64,
    /// Points for curators reviewing a translation proposal (M6)
    pub translate_points_reviewed: i64,
    /// Points for flagging a bad machine translation (M6)
    pub translate_points_flag: i64,
    /// Daily cap on translation-related points per user (M6)
    pub translate_points_daily_cap: i64,
    // ── Universal curator proposals (translation-everything.md §3) ──────
    /// Pending votes required before a translate/ui_string proposal resolves.
    pub proposal_quorum_translate: i64,
    /// Quorum for content/metadata fix proposals.
    pub proposal_quorum_content_fix: i64,
    /// Quorum for forum post edits (1 = single curator, forum product rule).
    pub proposal_quorum_post_edit: i64,
    /// Quorum for doc/request edits (1 = single curator).
    pub proposal_quorum_doc_edit: i64,
    /// Quorum for work deletion requests.
    pub proposal_quorum_work_deletion: i64,
    /// Max proposals a user can open per day (anti-spam; 0 = unlimited).
    pub proposals_per_user_per_day: i32,
    pub agent_max_runs_per_day: u32,
    /// Min seconds between agent runs for the same domain (cooldown).
    pub agent_cooldown_domain_secs: u64,
    /// Use-on-the-fly-with-validation: when a structural scrape failure
    /// occurs AND the agent is enabled, extract metadata from the HTML
    /// snapshot on the fly and complete the pending export (default false).
    pub agent_use_on_fly: bool,
    /// Max snapshot HTML chars sent to the extraction agent (guard against
    /// huge pages blowing the context window).
    pub agent_extract_max_snapshot_chars: usize,
    // ── Forum moderation points (SPEC-COMMUNITY-PLATFORM §8, F5) ─────
    /// Points granted per 72h window to an eligible moderator.
    pub forum_points_per_window: i32,
    /// Length of a moderation point window, hours (72h default).
    pub forum_window_hours: i32,
    /// Minimum role for forum moderation eligibility (5 = curator).
    pub forum_mod_min_level: i16,
    /// Minimum reputation/exp for eligibility (users.reputation proxy until F7).
    pub forum_mod_min_exp: i32,
    /// Minimum account age (days) for eligibility.
    pub forum_mod_min_age_days: i32,
    /// Minimum pool size (eligible moderators) before point mechanics activate.
    pub forum_mod_pool_min: i32,
    // ── Forum metamoderation (SPEC-COMMUNITY-PLATFORM §2/§3/§4, F6) ────
    /// Minimum role for metamod eligibility (1 = any logged-in user).
    pub forum_meta_min_level: i16,
    /// Minimum reputation/exp for metamod eligibility.
    pub forum_meta_min_exp: i32,
    /// Minimum account age (days) for metamod eligibility.
    pub forum_meta_min_age_days: i32,
    /// Minimum number of forum posts authored for metamod eligibility.
    pub forum_meta_min_posts: i32,
    /// Ratings sampled per moderator action before it leaves the queue.
    pub forum_meta_ratings: i32,
    /// Minimum eligible-metamoderator pool before metamod activates
    /// (pool too small → dormant, public modlog is the audit).
    pub forum_meta_pool_min: i32,
    /// Rolling audit window (days) for the unfair-rate computation.
    pub forum_meta_audit_window: i32,
    /// Minimum rated actions in the window before a cooldown can trigger.
    pub forum_meta_min_rated: i32,
    /// Unfair-rate threshold (0.30 = 30%); crossing it triggers a cooldown.
    pub forum_meta_unfair_rate: f64,
    /// Length of a metamod cooldown (days), applied to forum_mod_grants.cooldown_until.
    pub forum_meta_cooldown_days: i32,
    // ── Forum trust moderation (replaces points/metamod, 2026-09) ──────
    /// Trust tier that may access the moderation queue (default 4 = Elder).
    /// (env: FORUM_MOD_MIN_TRUST)
    pub forum_mod_min_trust: i16,
    /// Trust tier that may resolve reports / fast-hide / lock / pin
    /// (default 5 = Community Moderator). (env: FORUM_RESOLVE_MIN_TRUST)
    pub forum_resolve_min_trust: i16,
    /// Max moderation actions per user per UTC day (anti-abuse cap).
    /// (env: FORUM_MOD_ACTIONS_PER_DAY, default 50)
    pub forum_mod_actions_per_day: i32,
    // ── F7 site-wide leveling (SPEC-COMMUNITY-PLATFORM §10) ──────────────
    /// Level required to act as curator/mod (level >= this = mod).
    pub forum_curator_level: i16,
    /// Level required to act as admin.
    pub forum_admin_level: i16,
    /// Exp required per level (level = exp / this, capped at 100).
    pub forum_exp_per_level: i32,
    /// Exp awarded for creating a forum topic.
    pub forum_exp_topic_create: i32,
    /// Exp awarded for creating a forum post (reply).
    pub forum_exp_post_create: i32,
    /// Exp awarded to a post author when a mod gives a positive rating.
    pub forum_exp_mod_received: i32,
    /// Daily cap on mod-received exp per user.
    pub forum_exp_mod_daily_cap: i32,
    // ── Forum visibility ─────────────────────────────────────────────
    /// Allow anonymous visitors to read public forum content (categories,
    /// topic lists, topic detail, search). Set `FORUM_PUBLIC_READ=false` to
    /// make the whole forum login-only (private-instance hosting).
    pub forum_public_read: bool,
    // ── ActivityPub federation ───────────────────────────────────────
    /// Master switch — `ACTIVITYPUB_ENABLED=true` exposes /actor, webfinger,
    /// inbox/outbox. Default false (opt-in).
    pub activitypub_enabled: bool,
    /// Canonical domain for ActivityPub actor IDs (e.g. `fichub.example.com`).
    /// Defaults to the request Host header when absent.
    pub activitypub_domain: Option<String>,
    /// When true, federation accepts loopback/private hosts (docker tests).
    pub activitypub_allow_loopback: bool,
    /// preferredUsername for the instance actor (the bot's Fediverse handle).
    /// Falls back to node_name when unset. (env: ACTIVITYPUB_BOT_USERNAME)
    pub activitypub_bot_username: Option<String>,
    // ── Trust system (TL0-TL6) ───────────────────────────────────────
    /// Enable preference-similarity trust boost: users who bookmark/rate
    /// works whose tags align with the author's style gain small trust
    /// boosts. (default true, env: TRUST_PREFERENCE_SIMILARITY).
    pub trust_preference_similarity_enabled: bool,
    /// Base trust boost (in thousandths) for a preference-similar
    /// interaction. 10 = 0.01 trust. (env: TRUST_SIMILARITY_BOOST).
    pub trust_similarity_boost: i64,
    /// Decay multiplier for trust recovery after each loss event.
    /// Each time trust is lost, the next recovery is multiplied by this
    /// factor (0.0-1.0). E.g. 0.7 means 30% slower each time.
    /// (env: TRUST_RECOVERY_DECAY).
    pub trust_recovery_decay: f64,
}

impl Config {
    /// Build a Config with inert defaults for tests. Only `meta_dir` (and the
    /// cache/body dirs) matter to the metadata store; everything else is a
    /// placeholder so the struct can be constructed without env vars.
    #[cfg(test)]
    pub fn for_test(meta_dir: PathBuf) -> Self {
        Self {
            database_url: String::new(),
            redis_url: String::new(),
            cache_dir: meta_dir.clone(),
            secondary_cache_dir: None,
            export_version: 0,
            dynamic_rate_limit: false,
            node_name: String::new(),
            calibre_container: String::new(),
            tmp_dir: meta_dir.clone(),
            body_cache_dir: meta_dir.clone(),
            frontend_dir: meta_dir.clone(),
            app_port: 0,
            meta_dir: meta_dir,
            trusted_proxies: Vec::new(),
            ip_tag_sources: Vec::new(),
            maxmind_db: None,
            rec_default_delay_secs: 0,
            rec_site_rate_limits: std::collections::HashMap::new(),
            rec_max_favourite_pages: 0,
            rec_max_user_favourite_pages: 0,
            rec_max_recommendations: 0,
            rec_min_favouriters_for_collab: 0,
            rec_voting_boost_gamma: 0.0,
            rec_cache_ttl_hours: 0,
            rec_suggest_limit_per_hour: 0,
            rec_vote_limit_per_hour: 0,
            rec_precompute_enabled: false,
            rec_precompute_interval_hours: 0,
            rec_enable_cross_site: false,
            rec_engine_mode: RecEngineMode::Legacy,
            rec_strategies: String::new(),
            rec_decay_halflife_days: 0.0,
            rec_embed_model: String::new(),
            rec_embed_dim: 0,
            rec_mf_factors: 0,
            rec_mf_iters: 0,
            rec_mf_train_min_signals: 0,
            rec_bandit_slots: 0,
            rec_train_every_h: 0,
            rec_shadow_mode: false,
            rec_curator_prior: None,
            rec_prior_floor: 0.0,
            rec_curator_tau: 0.0,
            rec_strategy_timeout_secs: 0,
            rec_external_url: None,
            curator_token: None,
            tag_hidden_threshold: 0,
            tag_auto_delete_threshold: None,
            tag_submit_limit_per_hour: 0,
            tag_vote_limit_per_hour: 0,
            search_max_per_page: 0,
            max_upload_bytes: 0,
            wayback_fallback_enabled: false,
            wayback_max_snapshot_age_days: 0,
            wayback_cdx_rate_limit_per_sec: 0,
            fichub_fallback_enabled: false,
            tag_topup_enabled: false,
            rs_use_redis: false,
            rs_rate_limiter_enabled: false,
            rs_redis_pool_size: 0,
            rs_redis_timeout_ms: 0,
            rs_redis_reconnect_ms: 0,
            rs_redis_max_retries: 0,
            rs_redis_retry_base_ms: 0,
            rs_redis_retry_cap_ms: 0,
            rs_redis_key_prefix: String::new(),
            rs_redis_hash_tag: String::new(),
            rs_redis_compression: false,
            rs_redis_compression_threshold: 0,
            rs_redis_compression_level: 0,
            rs_redis_compression_algo: String::new(),
            rs_redis_compression_min_size: 0,
            rs_redis_compression_max_size: 0,
            rs_redis_compression_window: 0,
            rs_redis_compression_mem_level: 0,
            rs_redis_compression_strategy: 0,
            rs_redis_compression_chunk_size: 0,
            rs_redis_compression_dst_size: 0,
            opds_shelf_token: String::new(),
            opds_base_url: None,
            public_origin: String::new(),
            rl_download_capacity: 0.0,
            rl_download_flow: 0.0,
            rl_auth_capacity: 0.0,
            rl_auth_flow: 0.0,
            rl_search_capacity: 0.0,
            rl_search_flow: 0.0,
            rl_client_bonus_capacity: 0.0,
            rl_client_bonus_flow: 0.0,
            rl_nat_multiplier: 0.0,
            rl_shadowban_capacity: 0.0,
            rl_shadowban_flow: 0.0,
            rl_shadowban_ttl: 0,
            rl_tiered_enabled: false,
            rl_static_capacity: 0.0,
            rl_static_flow: 0.0,
            rl_static_client_capacity: 0.0,
            rl_static_client_flow: 0.0,
            pow_difficulty: 0,
            pow_ttl_secs: 0,
            ollama_url: String::new(),
            ollama_embed_model: String::new(),
            ollama_chat_model: String::new(),
            smtp_host: String::new(),
            smtp_port: 0,
            smtp_user: String::new(),
            smtp_pass: String::new(),
            smtp_from: String::new(),
            agent_enabled: false,
            agent_model: String::new(),
            agent_api_key: None,
            agent_base_url: String::new(),
            agent_ollama_url: String::new(),
            translation_enabled: false,
            translate_llm_enabled: false,
            translate_model: String::new(),
            translate_chunk_chars: 0,
            translate_global_budget_per_hour: 0,
            translate_user_budget_per_hour: 0,
            translate_badge_machine: false,
            translate_auto_approve_machine: false,
            translate_locales: Vec::new(),
            translate_reading_locale_ask: false,
            translate_ui_strings: false,
            translate_keep_machine_on_dismiss: false,
            translate_points_approved: 0,
            translate_points_improved: 0,
            translate_points_reviewed: 0,
            translate_points_flag: 0,
            translate_points_daily_cap: 0,
            proposal_quorum_translate: 0,
            proposal_quorum_content_fix: 0,
            proposal_quorum_post_edit: 0,
            proposal_quorum_doc_edit: 0,
            proposal_quorum_work_deletion: 0,
            proposals_per_user_per_day: 0,
            agent_max_runs_per_day: 0,
            agent_cooldown_domain_secs: 0,
            agent_use_on_fly: false,
            agent_extract_max_snapshot_chars: 0,
            forum_points_per_window: 0,
            forum_window_hours: 0,
            forum_mod_min_level: 0,
            forum_mod_min_exp: 0,
            forum_mod_min_age_days: 0,
            forum_mod_pool_min: 0,
            forum_meta_min_level: 0,
            forum_meta_min_exp: 0,
            forum_meta_min_age_days: 0,
            forum_meta_min_posts: 0,
            forum_meta_ratings: 0,
            forum_meta_pool_min: 0,
            forum_meta_audit_window: 0,
            forum_meta_min_rated: 0,
            forum_meta_unfair_rate: 0.0,
            forum_meta_cooldown_days: 0,
            forum_mod_min_trust: 0,
            forum_resolve_min_trust: 0,
            forum_mod_actions_per_day: 0,
            forum_curator_level: 0,
            forum_admin_level: 0,
            forum_exp_per_level: 0,
            forum_exp_topic_create: 0,
            forum_exp_post_create: 0,
            forum_exp_mod_received: 0,
            forum_exp_mod_daily_cap: 0,
            forum_public_read: false,
            activitypub_enabled: false,
            activitypub_domain: None,
            activitypub_allow_loopback: false,
            activitypub_bot_username: None,
            trust_preference_similarity_enabled: false,
            trust_similarity_boost: 0,
            trust_recovery_decay: 0.0,
        }
    }

    /// Load configuration from environment variables.
    /// Required vars: DATABASE_URL, REDIS_URL, CACHE_DIR
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

        let cache_dir = std::env::var("CACHE_DIR").unwrap_or_else(|_| "./cache".to_string());

        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);

        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let node_name = std::env::var("NODE_NAME").unwrap_or_else(|_| "orion".to_string());

        let calibre_container = std::env::var("CALIBRE_CONTAINER").unwrap_or_default();

        let tmp_dir = std::env::var("TMP_DIR").unwrap_or_else(|_| "./tmp".to_string());

        let body_cache_dir = std::env::var("BODY_CACHE_DIR")
            .unwrap_or_else(|_| "/public/literature/fichub/bodies".to_string());

        // META_DIR defaults to a sibling of the body cache so both stores
        // live on the same attached drive (NOT the database).
        let meta_dir = std::env::var("META_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::path::PathBuf::from(&body_cache_dir)
                    .parent()
                    .map(|p| p.join("meta"))
                    .unwrap_or_else(|| std::path::PathBuf::from("/public/literature/fichub/meta"))
            });

        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let frontend_dir =
            std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "./frontend/build".to_string());

        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // IP tag sources: format "path,type,tag" per line in IP_TAG_SOURCES
        let ip_tag_sources = std::env::var("IP_TAG_SOURCES")
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, ',').collect();
                if parts.len() == 3 {
                    Some((
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string(),
                        parts[2].trim().to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // MaxMind GeoLite2-ASN DB path for datacenter-IP blocking (optional).
        let maxmind_db = std::env::var("MAXMIND_DB").ok().filter(|s| !s.is_empty());

        // Recommender config
        let rec_default_delay_secs = std::env::var("REC_DEFAULT_DELAY_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let rec_site_rate_limits_str =
            std::env::var("REC_SITE_RATE_LIMITS").unwrap_or_else(|_| "{}".to_string());
        let rec_site_rate_limits: HashMap<String, u64> =
            serde_json::from_str(&rec_site_rate_limits_str).unwrap_or_default();

        let rec_max_favourite_pages = std::env::var("REC_MAX_FAVOURITE_PAGES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let rec_max_user_favourite_pages = std::env::var("REC_MAX_USER_FAVOURITE_PAGES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let rec_max_recommendations = std::env::var("REC_MAX_RECOMMENDATIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        let rec_min_favouriters_for_collab = std::env::var("REC_MIN_FAVOURITERS_FOR_COLLAB")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let rec_voting_boost_gamma = std::env::var("REC_VOTING_BOOST_GAMMA")
            .unwrap_or_else(|_| "0.2".to_string())
            .parse()
            .unwrap_or(0.2);

        let rec_cache_ttl_hours = std::env::var("REC_CACHE_TTL_HOURS")
            .unwrap_or_else(|_| "12".to_string())
            .parse()
            .unwrap_or(12);

        let rec_suggest_limit_per_hour = std::env::var("REC_SUGGEST_LIMIT_PER_HOUR")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let rec_vote_limit_per_hour = std::env::var("REC_VOTE_LIMIT_PER_HOUR")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let rec_precompute_enabled = std::env::var("REC_PRECOMPUTE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let rec_precompute_interval_hours = std::env::var("REC_PRECOMPUTE_INTERVAL_HOURS")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .unwrap_or(6);

        let rec_enable_cross_site = std::env::var("REC_ENABLE_CROSS_SITE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        // ── Pluggable recommendation platform (Stage 0+) ────────────────
        let rec_engine_mode = RecEngineMode::from_env();
        let rec_strategies =
            std::env::var("REC_STRATEGIES").unwrap_or_else(|_| "cooccur".to_string());
        let rec_decay_halflife_days = std::env::var("REC_DECAY_HALFLIFE_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30.0);
        let rec_embed_model =
            std::env::var("REC_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
        let rec_embed_dim = std::env::var("REC_EMBED_DIM")
            .unwrap_or_else(|_| "384".to_string())
            .parse()
            .unwrap_or(384);
        let rec_mf_factors = std::env::var("REC_MF_FACTORS")
            .unwrap_or_else(|_| "16".to_string())
            .parse()
            .unwrap_or(16);
        let rec_mf_iters = std::env::var("REC_MF_ITERS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let rec_mf_train_min_signals = std::env::var("REC_MF_TRAIN_MIN_SIGNALS")
            .unwrap_or_else(|_| "500".to_string())
            .parse()
            .unwrap_or(500);
        let rec_bandit_slots = std::env::var("REC_BANDIT_SLOTS")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        let rec_train_every_h = std::env::var("REC_TRAIN_EVERY_H")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .unwrap_or(6);
        let rec_shadow_mode = std::env::var("REC_SHADOW_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let rec_curator_prior = std::env::var("REC_CURATOR_PRIOR")
            .ok()
            .and_then(|s| s.parse::<i32>().ok());
        let rec_prior_floor = std::env::var("REC_PRIOR_FLOOR")
            .unwrap_or_else(|_| "0.2".to_string())
            .parse()
            .unwrap_or(0.2);
        let rec_curator_tau = std::env::var("REC_CURATOR_TAU")
            .unwrap_or_else(|_| "25".to_string())
            .parse()
            .unwrap_or(25.0);
        let rec_strategy_timeout_secs = std::env::var("REC_STRATEGY_TIMEOUT_SECS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let rec_external_url = std::env::var("REC_EXTERNAL_URL")
            .ok()
            .filter(|s| !s.is_empty());

        // Tagging v3 config
        let curator_token = std::env::var("CURATOR_TOKEN").ok();
        let tag_hidden_threshold = std::env::var("TAG_HIDDEN_THRESHOLD")
            .unwrap_or_else(|_| "-3".to_string())
            .parse()
            .unwrap_or(-3);
        let tag_auto_delete_threshold = std::env::var("TAG_AUTO_DELETE_THRESHOLD")
            .ok()
            .and_then(|s| s.parse().ok());
        let tag_submit_limit_per_hour = std::env::var("TAG_SUBMIT_LIMIT_PER_HOUR")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let tag_vote_limit_per_hour = std::env::var("TAG_VOTE_LIMIT_PER_HOUR")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);
        let search_max_per_page = std::env::var("SEARCH_MAX_PER_PAGE")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .unwrap_or(50);
        let max_upload_bytes = std::env::var("MAX_UPLOAD_BYTES")
            .unwrap_or_else(|_| (20 * 1024 * 1024).to_string())
            .parse()
            .unwrap_or(20 * 1024 * 1024);

        // ── Wayback Machine fallback (P8#5) ────────────────────────────
        let wayback_fallback_enabled = std::env::var("WAYBACK_FALLBACK_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        let wayback_max_snapshot_age_days = std::env::var("WAYBACK_MAX_SNAPSHOT_AGE_DAYS")
            .unwrap_or_else(|_| "365".to_string())
            .parse()
            .unwrap_or(365);
        let wayback_cdx_rate_limit_per_sec = std::env::var("WAYBACK_CDX_RATE_LIMIT_PER_SEC")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);

        // ── FicHub.net metadata fallback ───────────────────────────────
        // FicHub mirrors many sites and can still report real words/chapters
        // when a native scraper is blocked. Independent kill-switch.
        let fichub_fallback_enabled = std::env::var("FICHUB_FALLBACK_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        // ── Opportunistic tag top-up ───────────────────────────────────
        // When a native scrape succeeds but yields 0 tags and the site is
        // Wayback-eligible, fetch one snapshot and merge its tags. Opt-in
        // (CDX rate limiter throttles it); never adds latency when the
        // native scrape already produced tags.
        let tag_topup_enabled = std::env::var("TAG_TOPUP_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        // Redis stateful rate limiter
        let rs_use_redis = std::env::var("RS_USE_REDIS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let rs_rate_limiter_enabled = std::env::var("RS_RATE_LIMITER_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let rs_redis_pool_size = std::env::var("RS_REDIS_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let rs_redis_timeout_ms = std::env::var("RS_REDIS_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5000);
        let rs_redis_reconnect_ms = std::env::var("RS_REDIS_RECONNECT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        let rs_redis_max_retries = std::env::var("RS_REDIS_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
        let rs_redis_retry_base_ms = std::env::var("RS_REDIS_RETRY_BASE_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);
        let rs_redis_retry_cap_ms = std::env::var("RS_REDIS_RETRY_CAP_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5000);
        let rs_redis_key_prefix = std::env::var("RS_REDIS_KEY_PREFIX").unwrap_or_else(|_| "fichub:".into());
        let rs_redis_hash_tag = std::env::var("RS_REDIS_HASH_TAG").unwrap_or_else(|_| "{rl}".into());
        let rs_redis_compression = std::env::var("RS_REDIS_COMPRESSION")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let rs_redis_compression_threshold = std::env::var("RS_REDIS_COMPRESSION_THRESHOLD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024);
        let rs_redis_compression_level = std::env::var("RS_REDIS_COMPRESSION_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(6);
        let rs_redis_compression_algo = std::env::var("RS_REDIS_COMPRESSION_ALGO")
            .unwrap_or_else(|_| "gzip".into());
        let rs_redis_compression_min_size = std::env::var("RS_REDIS_COMPRESSION_MIN_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        let rs_redis_compression_max_size = std::env::var("RS_REDIS_COMPRESSION_MAX_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024 * 1024);
        let rs_redis_compression_window = std::env::var("RS_REDIS_COMPRESSION_WINDOW")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);
        let rs_redis_compression_mem_level = std::env::var("RS_REDIS_COMPRESSION_MEM_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8);
        let rs_redis_compression_strategy = std::env::var("RS_REDIS_COMPRESSION_STRATEGY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let rs_redis_compression_chunk_size = std::env::var("RS_REDIS_COMPRESSION_CHUNK_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8192);
        let rs_redis_compression_dst_size = std::env::var("RS_REDIS_COMPRESSION_DST_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(16384);

        let opds_shelf_token =
            std::env::var("OPDS_SHELF_TOKEN").unwrap_or_else(|_| "fichub".to_string());

        let opds_base_url = std::env::var("OPDS_BASE_URL")
            .ok()
            .filter(|s| !s.is_empty());

        // ── Public origin (sitemaps etc.) ───────────────────────────
        // PUBLIC_BASE_URL wins; falls back to OPDS_BASE_URL (deployments
        // usually set it to the same origin), then localhost.
        let public_origin = std::env::var("PUBLIC_BASE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| opds_base_url.clone())
            .unwrap_or_else(|| "http://localhost:8000".to_string());

        // ── Tiered rate limiting (anti-bot) ─────────────────────────
        // Tiers are (capacity, flow-tokens/sec). Defaults implement:
        //   download: 10 burst, refills 60/hr  = 60 downloads/hour, burst 10
        //   auth:     10 burst, refills 10/min = 10 auth attempts/minute
        //   search:   1000 burst, refills 1000/min (very high — bots don't hurt)
        //   shadowban download: 5 burst, refills 5/hr (friction, not a hard block)
        let rl_download_capacity = env_f64("RL_DOWNLOAD_CAPACITY", 10.0);
        let rl_download_flow = env_f64("RL_DOWNLOAD_FLOW", 60.0 / 3600.0);
        let rl_auth_capacity = env_f64("RL_AUTH_CAPACITY", 10.0);
        let rl_auth_flow = env_f64("RL_AUTH_FLOW", 10.0 / 60.0);
        let rl_search_capacity = env_f64("RL_SEARCH_CAPACITY", 1000.0);
        let rl_search_flow = env_f64("RL_SEARCH_FLOW", 1000.0 / 60.0);
        let rl_client_bonus_capacity = env_f64("RL_CLIENT_BONUS_CAPACITY", 5.0);
        let rl_client_bonus_flow = env_f64("RL_CLIENT_BONUS_FLOW", 30.0 / 3600.0);
        let rl_nat_multiplier = env_f64("RL_NAT_MULTIPLIER", 4.0);
        let rl_shadowban_capacity = env_f64("RL_SHADOWBAN_CAPACITY", 5.0);
        let rl_shadowban_flow = env_f64("RL_SHADOWBAN_FLOW", 5.0 / 3600.0);
        let rl_shadowban_ttl = std::env::var("RL_SHADOWBAN_TTL")
            .unwrap_or_else(|_| "86400".to_string()) // 24h
            .parse::<u64>()
            .unwrap_or(86400);
        let rl_tiered_enabled = std::env::var("RL_TIERED_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        let rl_static_capacity = env_f64("RL_STATIC_CAPACITY", 10.0);
        let rl_static_flow = env_f64("RL_STATIC_FLOW", 10.0 / 60.0);
        let rl_static_client_capacity = env_f64("RL_STATIC_CLIENT_CAPACITY", 5.0);
        let rl_static_client_flow = env_f64("RL_STATIC_CLIENT_FLOW", 5.0 / 60.0);

        // Proof-of-work challenge for shadowbanned clients
        let pow_difficulty = std::env::var("POW_DIFFICULTY")
            .unwrap_or_else(|_| "16".to_string())
            .parse::<u32>()
            .unwrap_or(16);
        let pow_ttl_secs = std::env::var("POW_TTL_SECS")
            .unwrap_or_else(|_| "600".to_string())
            .parse::<u64>()
            .unwrap_or(600);

        // Ollama embeddings (Roadmap Consensus Engine)
        let ollama_url =
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        let ollama_embed_model =
            std::env::var("OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
        // Ollama chat model for tiny LLM apps (comment moderation triage).
        let ollama_chat_model =
            std::env::var("OLLAMA_CHAT_MODEL").unwrap_or_else(|_| "lfm2.5:8b".to_string());

        // Send-to-Kindle (SMTP) — optional; if SMTP_HOST is unset the
        // /api/send-to-kindle endpoint reports SMTP not configured.
        let smtp_host = std::env::var("SMTP_HOST").unwrap_or_default();
        let smtp_port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(587);
        let smtp_user = std::env::var("SMTP_USER").unwrap_or_default();
        let smtp_pass = std::env::var("SMTP_PASS").unwrap_or_default();
        let smtp_from = std::env::var("SMTP_FROM").unwrap_or_default();

        // ── Self-healing agent (docs/AGENTS.md) ─────────────────────
        let agent_enabled = std::env::var("AGENT_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let agent_model = std::env::var("AGENT_MODEL")
            .unwrap_or_else(|_| "deepseek/deepseek-v4-flash".to_string());
        let agent_api_key = std::env::var("COMMANDCODE_API_KEY")
            .ok()
            .or_else(|| std::env::var("AGENT_API_KEY").ok())
            .filter(|s| !s.is_empty());
        let agent_base_url = std::env::var("AGENT_API_URL")
            .unwrap_or_else(|_| "https://api.commandcode.ai/provider/v1".to_string());
        let agent_ollama_url = std::env::var("AGENT_OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let agent_max_runs_per_day = std::env::var("AGENT_MAX_RUNS_PER_DAY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(6);
        let agent_cooldown_domain_secs = std::env::var("AGENT_COOLDOWN_DOMAIN_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600);
        let agent_use_on_fly = std::env::var("AGENT_USE_ON_FLY")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let agent_extract_max_snapshot_chars = std::env::var("AGENT_EXTRACT_MAX_SNAPSHOT_CHARS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(120000);

        // ── Translation config (§6 of translation-everything.md) ─────────────
        let translation_enabled = std::env::var("TRANSLATION_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        let translate_llm_enabled = std::env::var("TRANSLATE_LLM_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        let translate_model =
            std::env::var("TRANSLATE_MODEL").unwrap_or_else(|_| ollama_chat_model.clone());
        let translate_chunk_chars = std::env::var("TRANSLATE_CHUNK_CHARS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4000);
        let translate_global_budget_per_hour = std::env::var("TRANSLATE_GLOBAL_BUDGET_PER_HOUR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);
        let translate_user_budget_per_hour = std::env::var("TRANSLATE_USER_BUDGET_PER_HOUR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);
        let translate_badge_machine = std::env::var("TRANSLATE_BADGE_MACHINE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let translate_auto_approve_machine = std::env::var("TRANSLATE_AUTO_APPROVE_MACHINE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let translate_locales = std::env::var("TRANSLATE_LOCALES")
            .unwrap_or_else(|_| "de,es,fr,pt-BR,zh".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        let translate_reading_locale_ask = std::env::var("TRANSLATE_READING_LOCALE_ASK")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        let translate_ui_strings = std::env::var("TRANSLATE_UI_STRINGS")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        let translate_keep_machine_on_dismiss = std::env::var("TRANSLATE_KEEP_MACHINE_ON_DISMISS")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let translate_points_approved = std::env::var("TRANSLATE_POINTS_APPROVED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);
        let translate_points_improved = std::env::var("TRANSLATE_POINTS_IMPROVED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(25);
        let translate_points_reviewed = std::env::var("TRANSLATE_POINTS_REVIEWED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let translate_points_flag = std::env::var("TRANSLATE_POINTS_FLAG")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let translate_points_daily_cap = std::env::var("TRANSLATE_POINTS_DAILY_CAP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        // ── Universal curator proposals (§3 of translation-everything.md) ──
        let proposal_quorum_translate = std::env::var("PROPOSAL_QUORUM_TRANSLATE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let proposal_quorum_content_fix = std::env::var("PROPOSAL_QUORUM_CONTENT_FIX")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let proposal_quorum_post_edit = std::env::var("PROPOSAL_QUORUM_POST_EDIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let proposal_quorum_doc_edit = std::env::var("PROPOSAL_QUORUM_DOC_EDIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let proposal_quorum_work_deletion = std::env::var("PROPOSAL_QUORUM_WORK_DELETION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let proposals_per_user_per_day = std::env::var("PROPOSALS_PER_USER_PER_DAY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        // ── Forum moderation points (SPEC-COMMUNITY-PLATFORM §8, F5) ──
        let forum_points_per_window = std::env::var("FORUM_POINTS_PER_WINDOW")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);
        let forum_window_hours = std::env::var("FORUM_WINDOW_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(72);
        let forum_mod_min_level = std::env::var("FORUM_MOD_MIN_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50); // F7: level threshold for curators (was role 5)
        let forum_mod_min_exp = std::env::var("FORUM_MOD_MIN_EXP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(20);
        let forum_mod_min_age_days = std::env::var("FORUM_MOD_MIN_AGE_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(14);
        let forum_mod_pool_min = std::env::var("FORUM_MOD_POOL_MIN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8);

        // ── Forum metamoderation (SPEC-COMMUNITY-PLATFORM §2/§3/§4, F6) ──
        let forum_meta_min_level = std::env::var("FORUM_META_MIN_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let forum_meta_min_exp = std::env::var("FORUM_META_MIN_EXP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);
        let forum_meta_min_age_days = std::env::var("FORUM_META_MIN_AGE_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(90);
        let forum_meta_min_posts = std::env::var("FORUM_META_MIN_POSTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let forum_meta_ratings = std::env::var("FORUM_META_RATINGS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
        let forum_meta_pool_min = std::env::var("FORUM_META_POOL_MIN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8);
        let forum_meta_audit_window = std::env::var("FORUM_META_AUDIT_WINDOW")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        let forum_meta_min_rated = std::env::var("FORUM_META_MIN_RATED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let forum_meta_unfair_rate = std::env::var("FORUM_META_UNFAIR_RATE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.30);
        let forum_meta_cooldown_days = std::env::var("FORUM_META_COOLDOWN_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        // ── Forum trust moderation (replaces points/metamod, 2026-09) ──
        let forum_mod_min_trust = std::env::var("FORUM_MOD_MIN_TRUST")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4);
        let forum_resolve_min_trust = std::env::var("FORUM_RESOLVE_MIN_TRUST")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);
        let forum_mod_actions_per_day = std::env::var("FORUM_MOD_ACTIONS_PER_DAY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);

        // ── F7 site-wide leveling (SPEC-COMMUNITY-PLATFORM §10) ─────────
        let forum_curator_level = std::env::var("FORUM_CURATOR_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);
        let forum_admin_level = std::env::var("FORUM_ADMIN_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        let forum_exp_per_level = std::env::var("FORUM_EXP_PER_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        let forum_exp_topic_create = std::env::var("FORUM_EXP_TOPIC_CREATE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let forum_exp_post_create = std::env::var("FORUM_EXP_POST_CREATE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        let forum_exp_mod_received = std::env::var("FORUM_EXP_MOD_RECEIVED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let forum_exp_mod_daily_cap = std::env::var("FORUM_EXP_MOD_DAILY_CAP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        // ── Forum visibility ───────────────────────────────────────────
        // Public reads are the default (matches the open-community model);
        // set FORUM_PUBLIC_READ=false to require login for all forum reads.
        let forum_public_read = std::env::var("FORUM_PUBLIC_READ")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        // ── ActivityPub federation ───────────────────────────────────
        let activitypub_enabled = std::env::var("ACTIVITYPUB_ENABLED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);
        let activitypub_domain = std::env::var("ACTIVITYPUB_DOMAIN")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let activitypub_allow_loopback = std::env::var("ACTIVITYPUB_ALLOW_LOOPBACK")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);
        let activitypub_bot_username = std::env::var("ACTIVITYPUB_BOT_USERNAME")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            body_cache_dir: PathBuf::from(body_cache_dir),
            meta_dir,
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            ip_tag_sources,
            maxmind_db,
            rec_default_delay_secs,
            rec_site_rate_limits,
            rec_max_favourite_pages,
            rec_max_user_favourite_pages,
            rec_max_recommendations,
            rec_min_favouriters_for_collab,
            rec_voting_boost_gamma,
            rec_cache_ttl_hours,
            rec_suggest_limit_per_hour,
            rec_vote_limit_per_hour,
            rec_precompute_enabled,
            rec_precompute_interval_hours,
            rec_enable_cross_site,
            rec_engine_mode,
            rec_strategies,
            rec_decay_halflife_days,
            rec_embed_model,
            rec_embed_dim,
            rec_mf_factors,
            rec_mf_iters,
            rec_mf_train_min_signals,
            rec_bandit_slots,
            rec_train_every_h,
            rec_shadow_mode,
            rec_curator_prior,
            rec_prior_floor,
            rec_curator_tau,
            rec_strategy_timeout_secs,
            rec_external_url,
            curator_token,
            tag_hidden_threshold,
            tag_auto_delete_threshold,
            tag_submit_limit_per_hour,
            tag_vote_limit_per_hour,
            search_max_per_page,
            max_upload_bytes,
            wayback_fallback_enabled,
            wayback_max_snapshot_age_days,
            wayback_cdx_rate_limit_per_sec,
            fichub_fallback_enabled,
            tag_topup_enabled,
            rs_use_redis,
            rs_rate_limiter_enabled,
            rs_redis_pool_size,
            rs_redis_timeout_ms,
            rs_redis_reconnect_ms,
            rs_redis_max_retries,
            rs_redis_retry_base_ms,
            rs_redis_retry_cap_ms,
            rs_redis_key_prefix,
            rs_redis_hash_tag,
            rs_redis_compression,
            rs_redis_compression_threshold,
            rs_redis_compression_level,
            rs_redis_compression_algo,
            rs_redis_compression_min_size,
            rs_redis_compression_max_size,
            rs_redis_compression_window,
            rs_redis_compression_mem_level,
            rs_redis_compression_strategy,
            rs_redis_compression_chunk_size,
            rs_redis_compression_dst_size,
            opds_shelf_token,
            opds_base_url,
            public_origin,
            rl_download_capacity,
            rl_download_flow,
            rl_auth_capacity,
            rl_auth_flow,
            rl_search_capacity,
            rl_search_flow,
            rl_client_bonus_capacity,
            rl_client_bonus_flow,
            rl_nat_multiplier,
            rl_shadowban_capacity,
            rl_shadowban_flow,
            rl_shadowban_ttl,
            rl_tiered_enabled,
            rl_static_capacity,
            rl_static_flow,
            rl_static_client_capacity,
            rl_static_client_flow,
            pow_difficulty,
            pow_ttl_secs,
            ollama_url,
            ollama_embed_model,
            ollama_chat_model,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            smtp_from,
            agent_enabled,
            agent_model,
            agent_api_key,
            agent_base_url,
            agent_ollama_url,
            agent_max_runs_per_day,
            agent_cooldown_domain_secs,
            agent_use_on_fly,
            agent_extract_max_snapshot_chars,
            translation_enabled,
            translate_llm_enabled,
            translate_model,
            translate_chunk_chars,
            translate_global_budget_per_hour,
            translate_user_budget_per_hour,
            translate_badge_machine,
            translate_auto_approve_machine,
            translate_locales,
            translate_reading_locale_ask,
            translate_ui_strings,
            translate_keep_machine_on_dismiss,
            translate_points_approved,
            translate_points_improved,
            translate_points_reviewed,
            translate_points_flag,
            translate_points_daily_cap,
            proposal_quorum_translate,
            proposal_quorum_content_fix,
            proposal_quorum_post_edit,
            proposal_quorum_doc_edit,
            proposal_quorum_work_deletion,
            proposals_per_user_per_day,
            forum_points_per_window,
            forum_window_hours,
            forum_mod_min_level,
            forum_mod_min_exp,
            forum_mod_min_age_days,
            forum_mod_pool_min,
            forum_meta_min_level,
            forum_meta_min_exp,
            forum_meta_min_age_days,
            forum_meta_min_posts,
            forum_meta_ratings,
            forum_meta_pool_min,
            forum_meta_audit_window,
            forum_meta_min_rated,
            forum_meta_unfair_rate,
            forum_meta_cooldown_days,
            forum_mod_min_trust,
            forum_resolve_min_trust,
            forum_mod_actions_per_day,
            forum_curator_level,
            forum_admin_level,
            forum_exp_per_level,
            forum_exp_topic_create,
            forum_exp_post_create,
            forum_exp_mod_received,
            forum_exp_mod_daily_cap,
            forum_public_read,
            activitypub_enabled,
            activitypub_domain,
            activitypub_allow_loopback,
            activitypub_bot_username,
            // ── Trust system ─────────────────────────────────────────
            trust_preference_similarity_enabled: std::env::var("TRUST_PREFERENCE_SIMILARITY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            trust_similarity_boost: std::env::var("TRUST_SIMILARITY_BOOST")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            trust_recovery_decay: std::env::var("TRUST_RECOVERY_DECAY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
        }
    }
}

/// Parse a `f64` env var with a fallback (used by the tiered rate-limit config).
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Recommendation engine mode. `Legacy` (default) preserves today's
/// behavior exactly; `Pluggable` routes through the strategy registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecEngineMode {
    Legacy,
    Pluggable,
}

impl RecEngineMode {
    pub fn from_env() -> Self {
        match std::env::var("REC_ENGINE_MODE").as_deref() {
            Ok("pluggable") => RecEngineMode::Pluggable,
            _ => RecEngineMode::Legacy,
        }
    }

    pub fn is_pluggable(&self) -> bool {
        matches!(self, RecEngineMode::Pluggable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Serializes all env-var-manipulating tests to prevent cross-pollution.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper to set env vars and restore on drop.
    struct EnvGuard {
        keys: Vec<String>,
    }

    impl EnvGuard {
        fn new() -> Self {
            EnvGuard { keys: Vec::new() }
        }

        fn set(&mut self, key: &str, val: &str) {
            self.keys.push(key.to_string());
            // SAFETY: Test-only env var manipulation under ENV_LOCK, single-threaded.
            unsafe {
                std::env::set_var(key, val);
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for key in &self.keys {
                // SAFETY: Test-only env var cleanup under ENV_LOCK, single-threaded.
                unsafe {
                    std::env::remove_var(key);
                }
            }
        }
    }

    /// Clear all known config env vars (before each test).
    fn clear_config_env() {
        let keys = [
            "DATABASE_URL",
            "REDIS_URL",
            "CACHE_DIR",
            "SECONDARY_CACHE_DIR",
            "EXPORT_VERSION",
            "DYNAMIC_RATE_LIMIT",
            "NODE_NAME",
            "CALIBRE_CONTAINER",
            "TMP_DIR",
            "PORT",
            "FRONTEND_DIR",
            "TRUSTED_PROXIES",
            "IP_TAG_SOURCES",
            "MAXMIND_DB",
            "REC_DEFAULT_DELAY_SECS",
            "REC_SITE_RATE_LIMITS",
            "REC_MAX_FAVOURITE_PAGES",
            "REC_MAX_USER_FAVOURITE_PAGES",
            "REC_MAX_RECOMMENDATIONS",
            "REC_MIN_FAVOURITERS_FOR_COLLAB",
            "REC_VOTING_BOOST_GAMMA",
            "REC_CACHE_TTL_HOURS",
            "REC_SUGGEST_LIMIT_PER_HOUR",
            "REC_VOTE_LIMIT_PER_HOUR",
            "REC_PRECOMPUTE_ENABLED",
            "REC_PRECOMPUTE_INTERVAL_HOURS",
            "REC_ENABLE_CROSS_SITE",
            "REC_ENGINE_MODE",
            "REC_STRATEGIES",
            "REC_DECAY_HALFLIFE_DAYS",
            "REC_EMBED_MODEL",
            "REC_EMBED_DIM",
            "REC_MF_FACTORS",
            "REC_MF_ITERS",
            "REC_MF_TRAIN_MIN_SIGNALS",
            "REC_BANDIT_SLOTS",
            "REC_TRAIN_EVERY_H",
            "REC_SHADOW_MODE",
            "REC_CURATOR_PRIOR",
            "REC_PRIOR_FLOOR",
            "REC_CURATOR_TAU",
            "REC_STRATEGY_TIMEOUT_SECS",
            "REC_EXTERNAL_URL",
            "RL_DOWNLOAD_CAPACITY",
            "RL_DOWNLOAD_FLOW",
            "RL_AUTH_CAPACITY",
            "RL_AUTH_FLOW",
            "RL_SEARCH_CAPACITY",
            "RL_SEARCH_FLOW",
            "RL_CLIENT_BONUS_CAPACITY",
            "RL_CLIENT_BONUS_FLOW",
            "RL_NAT_MULTIPLIER",
            "RL_SHADOWBAN_CAPACITY",
            "RL_SHADOWBAN_FLOW",
            "RL_SHADOWBAN_TTL",
            "RL_TIERED_ENABLED",
            "POW_DIFFICULTY",
            "POW_TTL_SECS",
            "SMTP_HOST",
            "SMTP_PORT",
            "SMTP_USER",
            "SMTP_PASS",
            "SMTP_FROM",
            "FORUM_POINTS_PER_WINDOW",
            "FORUM_WINDOW_HOURS",
            "FORUM_MOD_MIN_LEVEL",
            "FORUM_MOD_MIN_EXP",
            "FORUM_MOD_MIN_AGE_DAYS",
            "FORUM_MOD_POOL_MIN",
            "FORUM_META_MIN_LEVEL",
            "FORUM_META_MIN_EXP",
            "FORUM_META_MIN_AGE_DAYS",
            "FORUM_META_MIN_POSTS",
            "FORUM_META_RATINGS",
            "FORUM_META_POOL_MIN",
            "FORUM_META_AUDIT_WINDOW",
            "FORUM_META_MIN_RATED",
            "FORUM_META_UNFAIR_RATE",
            "FORUM_META_COOLDOWN_DAYS",
            "FORUM_CURATOR_LEVEL",
            "FORUM_ADMIN_LEVEL",
            "FORUM_EXP_PER_LEVEL",
            "FORUM_EXP_TOPIC_CREATE",
            "FORUM_EXP_POST_CREATE",
            "FORUM_EXP_MOD_RECEIVED",
            "FORUM_EXP_MOD_DAILY_CAP",
            "AGENT_EXTRACT_MAX_SNAPSHOT_CHARS",
            "FORUM_PUBLIC_READ",
            "TRANSLATE_READING_LOCALE_ASK",
            "TRANSLATE_UI_STRINGS",
            "TRANSLATE_KEEP_MACHINE_ON_DISMISS",
            "TRANSLATE_POINTS_APPROVED",
            "TRANSLATE_POINTS_IMPROVED",
            "TRANSLATE_POINTS_REVIEWED",
            "TRANSLATE_POINTS_FLAG",
            "TRANSLATE_POINTS_DAILY_CAP",
        ];
        for key in &keys {
            // SAFETY: Test-only env var cleanup under ENV_LOCK, single-threaded.
            unsafe {
                std::env::remove_var(key);
            }
        }
    }

    #[test]
    fn test_from_env_defaults() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test_db");
        guard.set("REDIS_URL", "redis://localhost/0");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgres://localhost/test_db");
        assert_eq!(config.redis_url, "redis://localhost/0");
        assert_eq!(config.cache_dir, PathBuf::from("./cache"));
        assert!(config.secondary_cache_dir.is_none());
        assert_eq!(config.export_version, 1);
        assert!(config.dynamic_rate_limit);
        assert_eq!(config.node_name, "orion");
        assert_eq!(config.calibre_container, "");
        assert_eq!(config.tmp_dir, PathBuf::from("./tmp"));
        assert_eq!(config.app_port, 3000);
        assert_eq!(config.frontend_dir, PathBuf::from("./frontend/build"));
        assert!(config.trusted_proxies.is_empty());
        assert!(config.ip_tag_sources.is_empty());
        assert!(config.rec_site_rate_limits.is_empty());
        assert_eq!(config.rec_default_delay_secs, 5);
        assert_eq!(config.rec_max_favourite_pages, 3);
        assert_eq!(config.rec_max_user_favourite_pages, 3);
        assert_eq!(config.rec_max_recommendations, 20);
        assert_eq!(config.rec_min_favouriters_for_collab, 5);
        assert!((config.rec_voting_boost_gamma - 0.2).abs() < f64::EPSILON);
        assert_eq!(config.rec_cache_ttl_hours, 12);
        assert_eq!(config.rec_suggest_limit_per_hour, 5);
        assert_eq!(config.rec_vote_limit_per_hour, 10);
        assert!(config.rec_precompute_enabled);
        assert_eq!(config.rec_precompute_interval_hours, 6);
        assert!(config.rec_enable_cross_site);
        // Tiered rate-limit defaults
        assert!((config.rl_download_capacity - 10.0).abs() < f64::EPSILON);
        assert!((config.rl_download_flow - 60.0 / 3600.0).abs() < f64::EPSILON);
        assert!((config.rl_auth_capacity - 10.0).abs() < f64::EPSILON);
        assert!((config.rl_auth_flow - 10.0 / 60.0).abs() < f64::EPSILON);
        assert!((config.rl_search_capacity - 1000.0).abs() < f64::EPSILON);
        assert!((config.rl_search_flow - 1000.0 / 60.0).abs() < f64::EPSILON);
        assert!((config.rl_nat_multiplier - 4.0).abs() < f64::EPSILON);
        assert!((config.rl_shadowban_capacity - 5.0).abs() < f64::EPSILON);
        assert!((config.rl_shadowban_flow - 5.0 / 3600.0).abs() < f64::EPSILON);
        assert_eq!(config.rl_shadowban_ttl, 86400);
        assert!(config.rl_tiered_enabled);
        assert!((config.rl_static_capacity - 10.0).abs() < f64::EPSILON);
        assert!((config.rl_static_flow - 10.0 / 60.0).abs() < f64::EPSILON);
        assert!((config.rl_static_client_capacity - 5.0).abs() < f64::EPSILON);
        assert!((config.rl_static_client_flow - 5.0 / 60.0).abs() < f64::EPSILON);
        // Redis stateful rate limiter defaults
        assert!(!config.rs_use_redis);
        assert!(!config.rs_rate_limiter_enabled);
        assert_eq!(config.rs_redis_pool_size, 10);
        assert_eq!(config.rs_redis_timeout_ms, 5000);
        assert_eq!(config.rs_redis_reconnect_ms, 100);
        assert_eq!(config.rs_redis_max_retries, 3);
        assert_eq!(config.rs_redis_retry_base_ms, 50);
        assert_eq!(config.rs_redis_retry_cap_ms, 5000);
        assert_eq!(config.rs_redis_key_prefix, "fichub:");
        assert_eq!(config.rs_redis_hash_tag, "{rl}");
        assert!(!config.rs_redis_compression);
        assert_eq!(config.rs_redis_compression_threshold, 1024);
        assert_eq!(config.rs_redis_compression_level, 6);
        assert_eq!(config.rs_redis_compression_algo, "gzip");
        assert_eq!(config.rs_redis_compression_min_size, 100);
        assert_eq!(config.rs_redis_compression_max_size, 1024 * 1024);
        assert_eq!(config.rs_redis_compression_window, 4096);
        assert_eq!(config.rs_redis_compression_mem_level, 8);
        assert_eq!(config.rs_redis_compression_strategy, 0);
        assert_eq!(config.rs_redis_compression_chunk_size, 8192);
        assert_eq!(config.rs_redis_compression_dst_size, 16384);
        // PoW defaults
        assert_eq!(config.pow_difficulty, 16);
        assert_eq!(config.pow_ttl_secs, 600);
        // Self-healing agent defaults
        assert!(!config.agent_enabled, "AGENT_ENABLED defaults to false");
        assert_eq!(config.agent_model, "deepseek/deepseek-v4-flash");
        assert_eq!(
            config.agent_base_url,
            "https://api.commandcode.ai/provider/v1"
        );
        assert_eq!(config.agent_max_runs_per_day, 6);
        assert_eq!(config.agent_cooldown_domain_secs, 3600);
        assert!(
            !config.agent_use_on_fly,
            "AGENT_USE_ON_FLY defaults to false"
        );
        assert_eq!(
            config.agent_extract_max_snapshot_chars, 120000,
            "AGENT_EXTRACT_MAX_SNAPSHOT_CHARS defaults to 120000"
        );
        assert_eq!(config.agent_ollama_url, "http://localhost:11434");
        // Forum moderation point defaults (SPEC §8)
        assert_eq!(config.forum_points_per_window, 5);
        assert_eq!(config.forum_window_hours, 72);
        // F7: forum_mod_min_level is now a LEVEL threshold (curator = 50),
        // not the legacy role value (5).
        assert_eq!(config.forum_mod_min_level, 50);
        assert_eq!(config.forum_mod_min_exp, 20);
        assert_eq!(config.forum_mod_min_age_days, 14);
        assert_eq!(config.forum_mod_pool_min, 8);
        // Forum metamoderation defaults (SPEC §2/§3/§4)
        assert_eq!(config.forum_meta_min_level, 1);
        assert_eq!(config.forum_meta_min_exp, 50);
        assert_eq!(config.forum_meta_min_age_days, 90);
        assert_eq!(config.forum_meta_min_posts, 10);
        assert_eq!(config.forum_meta_ratings, 3);
        assert_eq!(config.forum_meta_pool_min, 8);
        assert_eq!(config.forum_meta_audit_window, 30);
        assert_eq!(config.forum_meta_min_rated, 10);
        assert!((config.forum_meta_unfair_rate - 0.30).abs() < f64::EPSILON);
        assert_eq!(config.forum_meta_cooldown_days, 30);
        // Forum trust moderation defaults (replaces points/metamod, 2026-09)
        assert_eq!(config.forum_mod_min_trust, 4);
        assert_eq!(config.forum_resolve_min_trust, 5);
        assert_eq!(config.forum_mod_actions_per_day, 50);
        // F7 site-wide leveling defaults (SPEC §10)
        assert_eq!(config.forum_curator_level, 50);
        assert_eq!(config.forum_admin_level, 100);
        assert_eq!(config.forum_exp_per_level, 100);
        assert_eq!(config.forum_exp_topic_create, 2);
        assert_eq!(config.forum_exp_post_create, 2);
        assert_eq!(config.forum_exp_mod_received, 1);
        assert_eq!(config.forum_exp_mod_daily_cap, 3);
        // Translation points defaults (M6)
        assert!(config.translate_reading_locale_ask);
        assert!(config.translate_ui_strings);
        assert!(!config.translate_keep_machine_on_dismiss);
        assert_eq!(config.translate_points_approved, 15);
        assert_eq!(config.translate_points_improved, 25);
        assert_eq!(config.translate_points_reviewed, 2);
        assert_eq!(config.translate_points_flag, 1);
        assert_eq!(config.translate_points_daily_cap, 100);
    }

    #[test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    fn test_panics_without_database_url() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("REDIS_URL", "redis://localhost");
        let _ = Config::from_env();
    }

    #[test]
    #[should_panic(expected = "REDIS_URL must be set")]
    fn test_panics_without_redis_url() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test");
        let _ = Config::from_env();
    }

    #[test]
    fn test_rec_site_rate_limits_valid_json() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test");
        guard.set("REDIS_URL", "redis://localhost");
        guard.set("REC_SITE_RATE_LIMITS", r#"{"ao3": 10, "ffn": 5}"#);

        let config = Config::from_env();
        let mut expected = HashMap::new();
        expected.insert("ao3".to_string(), 10);
        expected.insert("ffn".to_string(), 5);
        assert_eq!(config.rec_site_rate_limits, expected);
    }

    #[test]
    fn test_rec_site_rate_limits_invalid_json() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test");
        guard.set("REDIS_URL", "redis://localhost");
        guard.set("REC_SITE_RATE_LIMITS", "not valid json");

        let config = Config::from_env();
        assert!(config.rec_site_rate_limits.is_empty());
    }

    #[test]
    fn test_from_env_custom_values() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://custom/db");
        guard.set("REDIS_URL", "redis://custom");
        guard.set("CACHE_DIR", "/alt/cache");
        guard.set("SECONDARY_CACHE_DIR", "/sec/cache");
        guard.set("EXPORT_VERSION", "2");
        guard.set("DYNAMIC_RATE_LIMIT", "false");
        guard.set("NODE_NAME", "custom-node");
        guard.set("CALIBRE_CONTAINER", "calibre-srv");
        guard.set("TMP_DIR", "/alt/tmp");
        guard.set("PORT", "9090");
        guard.set("FRONTEND_DIR", "/alt/frontend");
        guard.set("TRUSTED_PROXIES", "10.0.0.1, 10.0.0.2");
        guard.set("IP_TAG_SOURCES", "/p1,type1,tag1\n/p2,type2,tag2");
        guard.set("MAXMIND_DB", "/opt/fichub/GeoLite2-ASN.mmdb");
        guard.set("REC_DEFAULT_DELAY_SECS", "15");
        guard.set("REC_MAX_FAVOURITE_PAGES", "7");
        guard.set("REC_MAX_USER_FAVOURITE_PAGES", "5");
        guard.set("REC_MAX_RECOMMENDATIONS", "100");
        guard.set("REC_MIN_FAVOURITERS_FOR_COLLAB", "2");
        guard.set("REC_VOTING_BOOST_GAMMA", "0.8");
        guard.set("REC_CACHE_TTL_HOURS", "48");
        guard.set("REC_SUGGEST_LIMIT_PER_HOUR", "2");
        guard.set("REC_VOTE_LIMIT_PER_HOUR", "30");
        guard.set("REC_PRECOMPUTE_ENABLED", "false");
        guard.set("REC_PRECOMPUTE_INTERVAL_HOURS", "24");
        guard.set("REC_ENABLE_CROSS_SITE", "false");
        guard.set("POW_DIFFICULTY", "8");
        guard.set("POW_TTL_SECS", "120");
        // Forum metamoderation custom values (were previously leaked-in from
        // other tests because clear_config_env didn't clear FORUM_* — now
        // they are set explicitly).
        guard.set("FORUM_META_MIN_LEVEL", "2");
        guard.set("FORUM_META_MIN_EXP", "80");
        guard.set("FORUM_META_MIN_AGE_DAYS", "120");
        guard.set("FORUM_META_MIN_POSTS", "15");
        guard.set("FORUM_META_RATINGS", "5");
        guard.set("FORUM_META_POOL_MIN", "12");
        guard.set("FORUM_META_AUDIT_WINDOW", "45");
        guard.set("FORUM_META_MIN_RATED", "20");
        guard.set("FORUM_META_UNFAIR_RATE", "0.5");
        guard.set("FORUM_META_COOLDOWN_DAYS", "60");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgres://custom/db");
        assert_eq!(config.redis_url, "redis://custom");
        assert_eq!(config.cache_dir, PathBuf::from("/alt/cache"));
        assert_eq!(
            config.secondary_cache_dir,
            Some(PathBuf::from("/sec/cache"))
        );
        assert_eq!(config.export_version, 2);
        assert!(!config.dynamic_rate_limit);
        assert_eq!(config.node_name, "custom-node");
        assert_eq!(config.calibre_container, "calibre-srv");
        assert_eq!(config.tmp_dir, PathBuf::from("/alt/tmp"));
        assert_eq!(config.app_port, 9090);
        assert_eq!(config.frontend_dir, PathBuf::from("/alt/frontend"));
        assert_eq!(
            config.trusted_proxies,
            vec!["10.0.0.1".to_string(), "10.0.0.2".to_string()]
        );
        assert_eq!(config.ip_tag_sources.len(), 2);
        assert_eq!(
            config.ip_tag_sources[0],
            ("/p1".to_string(), "type1".to_string(), "tag1".to_string())
        );
        assert_eq!(
            config.ip_tag_sources[1],
            ("/p2".to_string(), "type2".to_string(), "tag2".to_string())
        );
        assert_eq!(
            config.maxmind_db,
            Some("/opt/fichub/GeoLite2-ASN.mmdb".to_string())
        );
        assert_eq!(config.rec_default_delay_secs, 15);
        assert_eq!(config.rec_max_favourite_pages, 7);
        assert_eq!(config.rec_max_user_favourite_pages, 5);
        assert_eq!(config.rec_max_recommendations, 100);
        assert_eq!(config.rec_min_favouriters_for_collab, 2);
        assert!((config.rec_voting_boost_gamma - 0.8).abs() < f64::EPSILON);
        assert_eq!(config.rec_cache_ttl_hours, 48);
        assert_eq!(config.rec_suggest_limit_per_hour, 2);
        assert_eq!(config.rec_vote_limit_per_hour, 30);
        assert!(!config.rec_precompute_enabled);
        assert_eq!(config.rec_precompute_interval_hours, 24);
        assert!(!config.rec_enable_cross_site);
        // PoW custom values
        assert_eq!(config.pow_difficulty, 8);
        assert_eq!(config.pow_ttl_secs, 120);
        // Forum metamoderation custom values
        assert_eq!(config.forum_meta_min_level, 2);
        assert_eq!(config.forum_meta_min_exp, 80);
        assert_eq!(config.forum_meta_min_age_days, 120);
        assert_eq!(config.forum_meta_min_posts, 15);
        assert_eq!(config.forum_meta_ratings, 5);
        assert_eq!(config.forum_meta_pool_min, 12);
        assert_eq!(config.forum_meta_audit_window, 45);
        assert_eq!(config.forum_meta_min_rated, 20);
        assert!((config.forum_meta_unfair_rate - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.forum_meta_cooldown_days, 60);
    }
}
