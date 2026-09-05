use axum::response::Redirect;
use axum::{
    Router,
    body::Body,
    http::{Response, header},
    routing::{delete, get, post},
};
use std::sync::Arc;
use tower::Layer;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[cfg(test)]
use axum::http::StatusCode;

use crate::cache::CacheSemaphores;
use crate::config::Config;
use crate::db;
use crate::limiter;
use crate::recommender::{engine::RecommendationEngine, worker::CollectionWorker};
use crate::routes;
use crate::scrape::registry::ScraperRegistry;
use crate::visitor::middleware::visitor_middleware;

/// Shared application state accessible by all handlers
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    /// Dedicated Redis connection for `/api/health` PING checks. Kept
    /// separate from `redis` because the bookmark-import worker blocks on a
    /// shared multiplexed connection with an unbounded BRPOP; a PING queued
    /// behind that BRPOP would time out and make health report `redis:false`
    /// even though Redis is fine. Health also wraps its PING in a short
    /// timeout so a stuck Redis can never hang the endpoint.
    pub health_redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::TieredRateLimiter>,
    pub recommender_engine: RecommendationEngine,
    /// Pluggable recommendation platform: config-driven strategy registry
    /// (REC_ENGINE_MODE=pluggable). In legacy mode the handlers route around
    /// it — the registry is still built so `/api/recommendations/strategies`
    /// works and the pipeline worker has something to run.
    pub strategy_registry: crate::recommender::registry::StrategyRegistry,
    pub collection_worker: CollectionWorker,
    /// Self-healing telemetry + agent-run persistence.
    pub heal: crate::heal::HealService,
    /// Wayback Machine fallback (P8#5) — CDX limiter + config. When
    /// enabled, the export path falls back to IA snapshots on
    /// Blocked/NotFound/transient scrape failures.
    pub wayback: crate::scrape::wayback::WaybackService,
    /// 300s cache for the non-personalized popular suggestions
    /// (`GET /api/search/suggest`). Keyed by nothing: the popular list is
    /// global and only reflects `q`/`tag_type_id`-independent data once the
    /// query params become part of the SQL. Personalized requests bypass it.
    pub suggest_cache:
        Arc<tokio::sync::Mutex<Option<(std::time::Instant, Vec<serde_json::Value>)>>>,
    /// Ollama embeddings client for the Roadmap Consensus Engine.
    pub ollama: crate::services::ollama::OllamaClient,
    /// Email transport for Send-to-Kindle. `SmtpMailer` in production,
    /// `MockMailer` in tests (recorded sends, no relay needed).
    pub mailer: Box<dyn crate::services::mailer::Mailer>,
}

/// Build and run the HTTP server
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // ── Recommendation convergence: default the curator prior to the
    // site admin when REC_CURATOR_PRIOR is not explicitly set. This makes
    // cold users converge toward the admin's taste by default (opt-out via
    // the recs.personalized preference). The admin is the highest-trust,
    // highest-role user.
    let mut config = config;
    if config.rec_curator_prior.is_none() {
        let admin_id: Option<i32> = sqlx::query_scalar(
            "SELECT id FROM users
             ORDER BY trust_level DESC, role DESC, id ASC
             LIMIT 1",
        )
        .fetch_optional(&db_pool)
        .await
        .ok()
        .flatten();
        if let Some(admin_id) = admin_id {
            tracing::info!(
                "REC_CURATOR_PRIOR unset — converging recs toward admin user {admin_id}"
            );
            config.rec_curator_prior = Some(admin_id);
        }
    }

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str()).expect("Invalid Redis URL");
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    // Dedicated connection for the health endpoint. Health must never share
    // the main multiplexed connection: the bookmark-import worker parks an
    // unbounded BRPOP on it, and a PING sent behind that block would time out
    // (health would report redis:false while Redis is perfectly healthy).
    let health_redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis for health checks");

    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::with_config(
        redis_conn.clone(),
        config.dynamic_rate_limit,
        Some(&config),
    )
    .await
    .expect("Failed to initialize rate limiter");

    // Load datacenter IPs if configured
    if !config.ip_tag_sources.is_empty() {
        rate_limiter
            .load_datacenter_ips(&config.ip_tag_sources)
            .await;
    }

    // Create shared state
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    // Pluggable recommendation platform: config-driven strategy registry.
    // In legacy mode the registry exists but the handlers route around it
    // (REC_ENGINE_MODE=legacy → byte-identical behavior).
    let strategy_registry = crate::recommender::registry::StrategyRegistry::new(
        crate::recommender::available_strategies(&config),
        &config.rec_strategies,
    );
    tracing::info!(
        "rec engine mode: {:?}, strategies: {}",
        config.rec_engine_mode,
        strategy_registry.enabled_names().join(",")
    );
    let collection_worker = CollectionWorker::new(
        db_pool.clone(),
        redis_conn.clone(),
        http_client.clone(),
        config.clone(),
        scraper_registry.clone(),
    );
    let ollama_client = crate::services::ollama::OllamaClient::new(
        config.ollama_url.clone(),
        // Embedding model for the Roadmap Consensus Engine. The chat model
        // (Ask the Archive / comment triage) is passed per-request in the
        // Ollama API body by `generate_json`/`generate`; the client's
        // `model` field is only used by `embed()`.
        config.ollama_embed_model.clone(),
        http_client.clone(),
    );
    let heal_service = crate::heal::HealService::new(db_pool.clone(), config.clone());
    let wayback_service = crate::scrape::wayback::WaybackService::from_config(&config);
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        health_redis,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        strategy_registry,
        collection_worker,
        heal: heal_service,
        wayback: wayback_service,
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        ollama: ollama_client,
        mailer: Box::new(crate::services::mailer::SmtpMailer {
            cfg: crate::services::mailer::SmtpConfig {
                host: config.smtp_host.clone(),
                port: config.smtp_port,
                user: config.smtp_user.clone(),
                pass: config.smtp_pass.clone(),
                from: config.smtp_from.clone(),
            },
        }),
    });

    // Spawn the batch training pipeline for the pluggable recommendation
    // platform: every `REC_TRAIN_EVERY_H` hours, refresh signals and run
    // each enabled strategy's train step (records rec_training_runs).
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                if !state.strategy_registry.specs().is_empty() {
                    let results = crate::recommender::worker::run_training_pipeline(
                        &state.db,
                        &state.config,
                        &state.http_client,
                        &state.ollama,
                        &state.strategy_registry,
                    )
                    .await;
                    tracing::info!(
                        "rec training pipeline finished: {}",
                        serde_json::to_string(&results).unwrap_or_default()
                    );
                }
                let interval_h = state.config.rec_train_every_h.max(1) as u64;
                tokio::time::sleep(std::time::Duration::from_secs(interval_h * 3600)).await;
            }
        });
    }

    // Nightly content scan: verify cached bodies (story prose vs noise) and
    // detect archive warnings the author didn't tag. Runs on a 24h interval;
    // each pass scans the whole body cache (the model is local + cheap).
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let (scanned, failed) = crate::services::content_scan::scan_cache(
                    &state.db,
                    &state.ollama,
                    &state.config,
                    None,
                )
                .await;
                tracing::info!("nightly content scan finished: {scanned} scanned, {failed} failed");
                tokio::time::sleep(std::time::Duration::from_secs(24 * 3600)).await;
            }
        });
    }

    // Auto-delete cron: check for expired pending noise entries every hour.
    // Fics classified as noise with high confidence get a 72h grace period
    // for curator review. After expiry, the body cache is deleted.
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                let deleted = crate::services::content_scan::process_expired_deletions(
                    &state.db,
                    &state.config,
                )
                .await;
                if deleted > 0 {
                    tracing::info!("auto-delete cron: {deleted} expired noise fics removed");
                }
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await; // every hour
            }
        });
    }

    // Trust promotion pass: earn TL0–TL4 from engagement signals, revoke at
    // TL3+ on confirmed spam, and collect TL5 candidates. Runs on a 6h
    // interval — matches Discourse's "trust is computed, not requested" model.
    // The weekly moderation digest (weekly-digest bin / GET /api/admin/digest)
    // reuses the same promotion pass on demand.
    {
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                match crate::services::trust::run_trust_promotion(&state.db, &state.config).await {
                    Ok((promotions, candidates)) => {
                        if !promotions.is_empty() || !candidates.is_empty() {
                            tracing::info!(
                                "trust promotion pass: {} promotion(s), {} TL5 candidate(s)",
                                promotions.len(),
                                candidates.len()
                            );
                        }
                    }
                    Err(e) => tracing::warn!("trust promotion pass failed: {e}"),
                }
                if let Err(e) = crate::services::trust::run_trust_revocation(&state.db).await {
                    tracing::warn!("trust revocation pass failed: {e}");
                }
                tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
            }
        });
    }

    // Build router
    let app = build_router(state.clone()).await;

    // Backfill forum topic slugs for legacy rows (pre-migration 047) so the
    // by-slug board routes work for existing topics. Best-effort; runs once
    // at startup.
    crate::routes::forum::backfill_topic_slugs(&state.db).await;

    // Spawn background workers. The bookmark-import worker drains the
    // Redis `bookmark_import_queue` and creates a notification when each
    // import job finishes.
    let import_worker = crate::services::bookmark_import::BookmarkImportWorker::new(
        state.db.clone(),
        state.redis.clone(),
    );
    tokio::spawn(async move {
        import_worker.run().await;
    });

    // Translation worker (M1 of translation-everything.md): drains the Redis
    // `translate_queue`, calls Ollama, writes machine rows. Gated on both the
    // master switch and the LLM switch so UI-only deployments skip it.
    if config.translation_enabled && config.translate_llm_enabled {
        let tw_pool = state.db.clone();
        let tw_conn = state.redis.clone();
        let tw_config = config.clone();
        let tw_url = config.ollama_url.clone();
        let tw_model = config.translate_model.clone();
        tokio::spawn(async move {
            crate::services::translation::run_worker(tw_pool, tw_conn, tw_config, tw_url, tw_model)
                .await;
        });
    }

    // Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server error");
}

/// Build the Axum router with all routes
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    // Redirect handler functions (avoid closures with async blocks)
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    Router::new()
        // API routes
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/epub", get(routes::export::epub_handler))
        .route(
            "/api/epub/convert",
            get(routes::export::convert_format_handler),
        )
        .route("/api/meta", get(routes::meta::meta_handler))
        .route("/api/remote", get(remote_handler))
        .route("/api/health", get(routes::health::health_handler))
        // Cache download routes
        .route(
            "/cache/{etype}/{url_id}/{fname}",
            get(routes::cache_download::download_with_hash),
        )
        .route(
            "/cache/{etype}/{url_id}",
            get(routes::cache_download::download_or_export),
        )
        // Sitemaps (SEO)
        .route(
            "/sitemaps/index.xml",
            get(routes::sitemap::sitemap_index_handler),
        )
        .route(
            "/sitemaps/works-{shard}.xml",
            get(routes::sitemap::sitemap_works_handler),
        )
        // Anonymous device library (Tier-2)
        .route(
            "/api/v1/device/library",
            get(routes::device_library::get_device_library),
        )
        .route(
            "/api/v1/device/bookmark",
            post(routes::device_library::add_device_bookmark),
        )
        .route(
            "/api/v1/device/bookmark/{work_id}",
            delete(routes::device_library::remove_device_bookmark),
        )
        .route(
            "/api/v1/device/follow",
            post(routes::device_library::add_device_follow),
        )
        .route(
            "/api/v1/device/follow",
            delete(routes::device_library::remove_device_follow),
        )
        .route(
            "/api/v1/device/merge",
            post(routes::device_library::merge_device_library),
        )
        // Proof-of-work challenge for shadowbanned clients
        .route("/api/pow/challenge", get(routes::pow::challenge_handler))
        .route(
            "/api/pow/solve",
            axum::routing::post(routes::pow::solve_handler),
        )
        // Batch download routes
        .route(
            "/api/download/author",
            get(routes::download::download_author_handler),
        )
        .route(
            "/api/download/series",
            get(routes::download::download_series_handler),
        )
        // Recommender routes
        .route(
            "/api/recommendations",
            get(crate::recommender::routes::recommendations_handler),
        )
        .route(
            "/api/recommendations/personal",
            get(crate::recommender::routes::personal_recommendations_handler),
        )
        .route(
            "/api/recommendations/embeddings",
            get(crate::recommender::embedding_recs::embedding_recommendations_handler),
        )
        .route(
            "/api/recommendations/strategies",
            get(crate::recommender::routes::strategies_handler),
        )
        .route(
            "/api/recommendations/suggest",
            axum::routing::post(crate::recommender::routes::suggest_handler),
        )
        .route(
            "/api/recommendations/vote",
            axum::routing::post(crate::recommender::routes::vote_handler),
        )
        .route(
            "/api/recommendations/votes",
            get(crate::recommender::routes::votes_handler),
        )
        // Rec training pipeline admin trigger
        .route(
            "/api/recommendations/train",
            axum::routing::post(crate::recommender::routes::train_handler),
        )
        // ── Recipe Builder (P1 extension platform) ──────────────────────
        .route("/api/recipes", get(crate::routes::recipes::list_recipes))
        .route(
            "/api/recipes",
            axum::routing::post(crate::routes::recipes::create_recipe),
        )
        .route(
            "/api/recipes/active",
            get(crate::routes::recipes::get_active_recipe),
        )
        .route(
            "/api/recipes/gallery",
            get(crate::routes::recipes::browse_gallery),
        )
        .route(
            "/api/recipes/{id}",
            axum::routing::put(crate::routes::recipes::update_recipe),
        )
        .route(
            "/api/recipes/{id}",
            axum::routing::delete(crate::routes::recipes::delete_recipe),
        )
        .route(
            "/api/recipes/{id}/activate",
            axum::routing::post(crate::routes::recipes::activate_recipe),
        )
        .route(
            "/api/recipes/{id}/install",
            axum::routing::post(crate::routes::recipes::install_recipe),
        )
        .route(
            "/api/recipes/{id}/publish",
            axum::routing::post(crate::routes::recipes::publish_recipe),
        )
        // Per-fic community suggestions (auth-based, URL-scrapeable, votable)
        .route(
            "/api/fic-suggestions",
            get(crate::fic_suggestions::list_suggestions),
        )
        .route(
            "/api/fic-suggestions",
            axum::routing::post(crate::fic_suggestions::create_suggestion),
        )
        .route(
            "/api/fic-suggestions/{id}/vote",
            axum::routing::post(crate::fic_suggestions::vote_suggestion),
        )
        .route(
            "/api/fic-suggestions/{id}/remove",
            axum::routing::post(crate::fic_suggestions::remove_suggestion),
        )
        // Tag routes (v3)
        .route(
            "/api/tags/submit",
            axum::routing::post(crate::tags::routes::submit_tag),
        )
        .route(
            "/api/tags/vote",
            axum::routing::post(crate::tags::routes::vote_tag),
        )
        .route(
            "/api/tags/flag",
            axum::routing::post(crate::tags::routes::flag_tag),
        )
        .route("/api/tags", get(crate::tags::routes::get_tags))
        // Tag health & status routes
        .route(
            "/api/tags/resolve",
            get(crate::tags::routes::resolve_tag_handler),
        )
        .route("/api/tags/search", get(crate::tags::routes::search_tags))
        .route(
            "/api/tags/autocomplete",
            get(crate::tags::routes::tag_autocomplete),
        )
        .route("/api/tags/{id}", get(crate::tags::routes::get_tag_detail))
        // Curator routes (v3)
        .route(
            "/api/curator/alias",
            axum::routing::post(crate::tags::curator::create_alias),
        )
        .route(
            "/api/curator/aliases",
            get(crate::tags::curator::list_aliases),
        )
        .route(
            "/api/curator/aliases/{alias_name}",
            axum::routing::delete(crate::tags::curator::delete_alias),
        )
        .route(
            "/api/curator/merge",
            axum::routing::post(crate::tags::curator::merge_tags),
        )
        .route(
            "/api/curator/tags/{id}",
            axum::routing::delete(crate::tags::curator::delete_tag),
        )
        .route(
            "/api/curator/tags/{id}",
            axum::routing::put(crate::tags::curator::update_tag),
        )
        .route("/api/curator/flags", get(crate::tags::curator::list_flags))
        .route(
            "/api/curator/flags/{id}/resolve",
            axum::routing::post(crate::tags::curator::resolve_flag),
        )
        // Curator content-fix endpoints (body blob override for wrong scrapes,
        // with peer voting: fixes apply only after other curators approve)
        .route(
            "/api/curator/content/{url_id}/propose",
            axum::routing::post(crate::routes::curator_content::propose_fix),
        )
        .route(
            "/api/curator/consensus",
            get(crate::routes::consensus::consensus_feed),
        )
        .route(
            "/api/curator/marginalia",
            get(crate::routes::marginalia_admin::list),
        )
        .route(
            "/api/curator/content/proposals",
            axum::routing::get(crate::routes::curator_content::list_proposals),
        )
        .route(
            "/api/curator/content/proposals/{id}/vote",
            axum::routing::post(crate::routes::curator_content::vote_fix),
        )
        .route(
            "/api/curator/metadata/propose",
            axum::routing::post(crate::routes::curator_content::propose_metadata_fix),
        )
        .route(
            "/api/curator/metadata/proposals",
            get(crate::routes::curator_content::list_metadata_proposals),
        )
        .route(
            "/api/curator/metadata/proposals/{id}/vote",
            axum::routing::post(crate::routes::curator_content::vote_metadata_fix),
        )
        .route(
            "/api/curator/approvals",
            get(crate::routes::collections::curator_approvals_handler),
        )
        .route(
            "/api/curator/content/{url_id}",
            axum::routing::get(crate::routes::curator_content::get_body),
        )
        .route(
            "/api/curator/content/{url_id}",
            axum::routing::delete(crate::routes::curator_content::delete_body),
        )
        // Work deletion: branch on role; curator review queue for the rest
        .route(
            "/api/works/{url_id}/delete",
            axum::routing::post(crate::routes::work_delete::post_delete_request),
        )
        .route(
            "/api/curator/work-deletions",
            axum::routing::get(crate::routes::work_delete::list_delete_requests),
        )
        .route(
            "/api/curator/work-deletions/{id}/resolve",
            axum::routing::post(crate::routes::work_delete::resolve_delete_request),
        )
        // Series routes
        .route("/api/series/{id}", get(crate::routes::series::get_series))
        .route(
            "/api/authors/search",
            get(crate::routes::authors::search_authors),
        )
        // Author bibliography page: keyed by canonical author name
        .route(
            "/api/authors/by-name/{name}",
            get(crate::routes::series::get_author),
        )
        .route(
            "/api/authors/{id}",
            get(crate::routes::authors::get_author_profile),
        )
        .route(
            "/api/authors/{id}",
            axum::routing::put(crate::routes::authors::update_author_profile),
        )
        .route(
            "/api/authors/{id}/socials",
            axum::routing::post(crate::routes::authors::add_social),
        )
        .route(
            "/api/authors/{id}/socials/{social_id}",
            axum::routing::delete(crate::routes::authors::remove_social),
        )
        .route(
            "/api/curator/authors/merge",
            axum::routing::post(crate::routes::authors::propose_merge),
        )
        .route(
            "/api/curator/authors/pending",
            get(crate::routes::authors::pending_merges),
        )
        .route(
            "/api/curator/authors/approve/{proposal_id}",
            axum::routing::post(crate::routes::authors::approve_merge),
        )
        .route(
            "/api/curator/authors/reject/{proposal_id}",
            axum::routing::post(crate::routes::authors::reject_merge),
        )
        // Advanced search route
        .route("/api/search", get(crate::search::routes::search_handler))
        // Ask the Archive — natural-language search via Ollama
        .route(
            "/api/search/ask",
            axum::routing::post(crate::search::ask::ask_handler),
        )
        // Full-text search over fic bodies ("fics where X says Y")
        .route(
            "/api/search/body",
            axum::routing::get(crate::search::body::body_search_handler),
        )
        // Blind Date with a Fic — random discovery with title/fandom hidden
        .route(
            "/api/blind-date",
            get(crate::routes::blind::blind_date_handler),
        )
        .route(
            "/api/blind-date/reveal",
            get(crate::routes::blind::blind_date_reveal_handler),
        )
        // Search filter suggestions (popular + personalized)
        .route(
            "/api/search/suggest",
            get(crate::search::suggest::search_suggest_handler),
        )
        // Per-user search history chips
        .route(
            "/api/search/history/chips",
            get(crate::search::history_chips::search_history_chips_handler),
        )
        // Saved searches + daily alerts
        .route(
            "/api/search/saved",
            axum::routing::post(crate::routes::saved_search::create_saved_search),
        )
        .route(
            "/api/search/saved",
            get(crate::routes::saved_search::list_saved_searches),
        )
        .route(
            "/api/search/saved/{id}",
            axum::routing::delete(crate::routes::saved_search::delete_saved_search),
        )
        .route(
            "/api/search/saved/{id}/alert",
            axum::routing::put(crate::routes::saved_search::update_saved_search_alert),
        )
        .route(
            "/api/search/saved/{id}/run",
            axum::routing::post(crate::routes::saved_search::run_saved_search),
        )
        // Auth routes
        .route(
            "/api/auth/register",
            axum::routing::post(crate::routes::social::register_handler),
        )
        .route(
            "/api/auth/login",
            axum::routing::post(crate::routes::social::login_handler),
        )
        .route(
            "/api/auth/refresh",
            axum::routing::post(crate::routes::social::refresh_handler),
        )
        .route("/api/auth/me", get(crate::routes::social::me_handler))
        .route("/api/users/me/level", get(crate::routes::forum::my_level))
        // Bounty routes (reputation spend for demand-side requests; XP untouched)
        .route("/api/bounties", get(crate::routes::bounties::list_bounties))
        .route(
            "/api/bounties",
            axum::routing::post(crate::routes::bounties::create_bounty_handler),
        )
        .route(
            "/api/bounties/{id}/claim",
            axum::routing::post(crate::routes::bounties::claim_bounty_handler),
        )
        .route(
            "/api/bounties/{id}/resolve",
            axum::routing::post(crate::routes::bounties::resolve_bounty_handler),
        )
        .route(
            "/api/xp/idle-tick",
            post(crate::routes::bounties::idle_tick_handler),
        )
        // F7 subsystems: site info, invites, registration applications, blocks
        .route("/api/site", get(crate::routes::subsystems::site_info))
        .route(
            "/api/admin/invites",
            axum::routing::post(crate::routes::subsystems::create_invite),
        )
        .route(
            "/api/admin/invites",
            get(crate::routes::subsystems::list_invites),
        )
        .route(
            "/api/registration-applications",
            axum::routing::post(crate::routes::subsystems::apply_registration),
        )
        .route(
            "/api/admin/registration-applications",
            get(crate::routes::subsystems::list_registration_applications),
        )
        .route(
            "/api/admin/registration-applications/{id}/review",
            axum::routing::post(crate::routes::subsystems::review_registration_application),
        )
        .route(
            "/api/blocks",
            axum::routing::post(crate::routes::subsystems::block_user),
        )
        .route("/api/blocks", get(crate::routes::subsystems::list_blocks))
        .route(
            "/api/blocks/{user_id}",
            axum::routing::delete(crate::routes::subsystems::unblock_user),
        )
        // Similar / vector search routes
        .route(
            "/api/search/similar/{work_id}",
            get(crate::routes::search::similar_by_vector),
        )
        .route(
            "/api/works/{id}/similar",
            get(crate::routes::search::similar_by_bookmarks),
        )
        // "Surprise me" — a random fic with full metadata (Blind Date UI)
        .route("/api/works/random", get(crate::routes::search::random_work))
        // Per-fic "readers also bookmarked" rec anchors (co-occurrence table)
        .route(
            "/api/v1/works/{url_id}/also-bookmarked",
            get(crate::routes::search::also_bookmarked),
        )
        // Social routes
        .route(
            "/api/bookmarks",
            axum::routing::post(crate::routes::social::add_bookmark_handler),
        )
        .route(
            "/api/bookmarks",
            get(crate::routes::social::list_bookmarks_handler),
        )
        .route(
            "/api/bookmarks/{work_id}",
            axum::routing::delete(crate::routes::social::remove_bookmark_handler),
        )
        .route(
            "/api/bookmarks/export",
            get(crate::routes::social::export_bookmarks_csv),
        )
        .route(
            "/api/bookmarks/export.csv",
            get(crate::routes::social::export_bookmarks_csv),
        )
        .route(
            "/api/bookmarks/import",
            axum::routing::post(crate::routes::social::import_bookmarks_csv),
        )
        .route(
            "/api/ratings",
            axum::routing::post(crate::routes::social::rate_work_handler),
        )
        .route(
            "/api/ratings/{work_id}",
            get(crate::routes::social::get_ratings_handler),
        )
        // Kudos — one-click appreciation, separate from 5-star ratings.
        // GET is public; POST/DELETE require auth (see social.rs).
        .route(
            "/api/kudos/{work_id}",
            get(crate::routes::social::get_kudos_handler),
        )
        .route(
            "/api/kudos/{work_id}",
            axum::routing::post(crate::routes::social::give_kudos_handler),
        )
        .route(
            "/api/kudos/{work_id}",
            axum::routing::delete(crate::routes::social::remove_kudos_handler),
        )
        .route(
            "/api/reviews",
            axum::routing::post(crate::routes::reviews::upsert_review_handler),
        )
        .route(
            "/api/reviews/{id}",
            axum::routing::delete(crate::routes::reviews::delete_review_handler),
        )
        .route(
            "/api/works/{id}/reviews",
            get(crate::routes::reviews::list_reviews_handler),
        )
        .route(
            "/api/comments",
            axum::routing::post(crate::routes::social::add_comment_handler),
        )
        .route(
            "/api/comments/{work_id}",
            get(crate::routes::social::list_comments_handler),
        )
        .route(
            "/api/works/{id}",
            get(crate::routes::social::get_work_handler),
        )
        .route(
            "/api/works/{id}/stats",
            get(crate::routes::social::work_stats_handler),
        )
        .route(
            "/api/my-works",
            get(crate::routes::social::list_my_works_handler),
        )
        .route(
            "/api/reactions/{target_type}/{target_id}/react",
            post(crate::routes::reactions::react),
        )
        .route(
            "/api/reactions/{target_type}/{target_id}",
            get(crate::routes::reactions::get_reactions),
        )
        .route(
            "/api/leaderboard/curators",
            get(crate::routes::social::leaderboard_curators_handler),
        )
        .route(
            "/api/leaderboard",
            get(crate::routes::leaderboard::leaderboard_handler),
        )
        .route(
            "/api/leaderboard/categories",
            get(crate::routes::leaderboard::leaderboard_categories_handler),
        )
        .route(
            "/api/users/{id}",
            get(crate::routes::social::user_profile_handler),
        )
        // One-click full user data export (ZIP of all personal data)
        .route(
            "/api/user/export",
            get(crate::routes::user_export::user_export_handler),
        )
        .route(
            "/api/user/account",
            axum::routing::delete(crate::routes::user_export::delete_account_handler),
        )
        .route(
            "/api/user/consent",
            axum::routing::post(crate::routes::user_export::record_consent_handler),
        )
        .route(
            "/api/user/preferences/formats",
            get(crate::routes::user_preferences::get_format_preferences),
        )
        .route(
            "/api/user/preferences/formats",
            axum::routing::put(crate::routes::user_preferences::update_format_preferences),
        )
        // DMCA / copyright takedowns
        .route(
            "/api/copyright/notice",
            axum::routing::post(crate::routes::copyright::submit_notice),
        )
        .route(
            "/api/admin/copyright/notices",
            get(crate::routes::copyright::list_notices),
        )
        .route(
            "/api/admin/copyright/notices/{id}/action",
            axum::routing::post(crate::routes::copyright::action_notice),
        )
        .route(
            "/api/admin/copyright/notices/{id}/reject",
            axum::routing::post(crate::routes::copyright::reject_notice),
        )
        // User-supplied site credentials (opt-in, 30-day expiry, AES-GCM encrypted)
        .route(
            "/api/user/site-credentials",
            axum::routing::put(crate::routes::user_credentials::set_site_credentials_handler),
        )
        .route(
            "/api/user/site-credentials",
            get(crate::routes::user_credentials::list_site_credentials_handler),
        )
        .route(
            "/api/user/site-credentials/{domain}",
            axum::routing::delete(crate::routes::user_credentials::delete_site_credentials_handler),
        )
        // Notification routes
        .route(
            "/api/notifications",
            get(crate::routes::notifications::list_notifications_handler),
        )
        .route(
            "/api/notifications/unread-count",
            get(crate::routes::notifications::unread_count_handler),
        )
        .route(
            "/api/notifications/read-all",
            axum::routing::post(crate::routes::notifications::mark_all_read_handler),
        )
        .route(
            "/api/notifications/{id}/read",
            axum::routing::post(crate::routes::notifications::mark_notification_read_handler),
        )
        .route(
            "/api/notifications/preferences",
            get(crate::routes::notifications::get_preferences_handler),
        )
        .route(
            "/api/notifications/preferences",
            axum::routing::put(crate::routes::notifications::update_preferences_handler),
        )
        // Follow routes
        .route("/api/feed", get(crate::routes::feed::feed_handler))
        .route(
            "/api/follows",
            axum::routing::post(crate::routes::follows::follow_handler),
        )
        .route(
            "/api/follows",
            get(crate::routes::follows::list_follows_handler),
        )
        .route(
            "/api/follows/{id}",
            axum::routing::delete(crate::routes::follows::unfollow_handler),
        )
        .route(
            "/api/follows/{follow_id}/exclusions",
            axum::routing::post(crate::routes::follows::add_follow_exclusion_handler),
        )
        .route(
            "/api/follows/{follow_id}/exclusions",
            get(crate::routes::follows::list_follow_exclusions_handler),
        )
        .route(
            "/api/follows/{follow_id}/exclusions/{exclusion_id}",
            axum::routing::delete(crate::routes::follows::delete_follow_exclusion_handler),
        )
        .route(
            "/api/follows/check/{target_type}/{target_id}",
            get(crate::routes::follows::check_follow_handler),
        )
        // ── Forum (F2: categories + topic list, read side) ────────────
        .route(
            "/api/forum/categories",
            get(crate::routes::forum::list_categories),
        )
        .route(
            "/api/forum/categories",
            axum::routing::post(crate::routes::forum::create_category),
        )
        .route(
            "/api/forum/categories/{id}",
            axum::routing::patch(crate::routes::forum::update_category),
        )
        .route("/api/forum/topics", get(crate::routes::forum::list_topics))
        .route(
            "/api/forum/topics",
            axum::routing::post(crate::routes::forum::create_topic),
        )
        // Literal `by-slug` segment must be registered BEFORE `{topicId}` so
        // the static path wins over the numeric param.
        .route(
            "/api/forum/topics/by-slug/{topicSlug}",
            get(crate::routes::forum::topic_detail_by_slug),
        )
        .route(
            "/api/forum/topics/{topicId}",
            get(crate::routes::forum::topic_detail),
        )
        .route(
            "/api/forum/topics/{topicId}",
            axum::routing::patch(crate::routes::forum::update_topic),
        )
        .route(
            "/api/forum/topics/{topicId}",
            axum::routing::delete(crate::routes::forum::delete_topic),
        )
        .route(
            "/api/forum/topics/{topicId}/posts",
            axum::routing::post(crate::routes::forum::create_post),
        )
        .route(
            "/api/forum/topics/{topicId}/follow",
            axum::routing::get(crate::routes::forum::get_follow_state),
        )
        .route(
            "/api/forum/topics/{topicId}/follow",
            axum::routing::post(crate::routes::forum::toggle_follow),
        )
        .route(
            "/api/forum/topics/{topicId}/read",
            axum::routing::post(crate::routes::forum::mark_topic_read),
        )
        .route(
            "/api/forum/edits/{target_type}/{target_id}",
            get(crate::routes::forum::forum_edit_history),
        )
        .route(
            "/api/forum/edits/queue",
            get(crate::routes::forum::forum_edit_queue),
        )
        .route(
            "/api/forum/edits/{proposalId}/review",
            axum::routing::post(crate::routes::forum::review_forum_edit),
        )
        .route("/api/forum/search", get(crate::routes::forum::search_forum))
        .route(
            "/api/forum/posts/{postId}",
            axum::routing::patch(crate::routes::forum::update_post),
        )
        .route(
            "/api/forum/posts/{postId}",
            axum::routing::delete(crate::routes::forum::delete_post),
        )
        .route(
            "/api/forum/moderation/status",
            get(crate::routes::forum::moderation_status),
        )
        .route(
            "/api/forum/moderation/queue",
            get(crate::routes::forum::moderation_queue),
        )
        .route(
            "/api/forum/posts/{postId}/moderate",
            axum::routing::post(crate::routes::forum::moderate_post),
        )
        .route(
            "/api/forum/posts/{postId}/moderations",
            get(crate::routes::forum::post_moderations),
        )
        .route(
            "/api/forum/posts/{postId}/react",
            axum::routing::post(crate::routes::forum::react_to_post),
        )
        .route(
            "/api/forum/posts/{postId}/reactions",
            get(crate::routes::forum::get_post_reactions)
                .post(crate::routes::forum::react_to_post)
                .delete(crate::routes::forum::remove_post_reaction),
        )
        .route(
            "/api/forum/metamod/queue",
            get(crate::routes::forum::metamod_queue),
        )
        .route(
            "/api/forum/metamod/{actionId}/vote",
            axum::routing::post(crate::routes::forum::metamod_vote),
        )
        .route(
            "/api/forum/metamod/grants/{id}",
            get(crate::routes::forum::metamod_grant_detail),
        )
        .route(
            "/api/forum/metamod/grants/{id}/verdict",
            axum::routing::post(crate::routes::forum::metamod_grant_verdict),
        )
        .route(
            "/api/admin/forum/hide/{postId}",
            axum::routing::post(crate::routes::forum::admin_hide_post),
        )
        .route(
            "/api/admin/forum/topics/{topicId}/lock",
            axum::routing::post(crate::routes::forum::admin_lock_topic),
        )
        .route(
            "/api/admin/forum/topics/{topicId}/pin",
            axum::routing::post(crate::routes::forum::admin_pin_topic),
        )
        .route(
            "/api/admin/forum/bans",
            axum::routing::post(crate::routes::forum::admin_create_ban),
        )
        .route(
            "/api/admin/forum/bans/{id}",
            axum::routing::delete(crate::routes::forum::admin_delete_ban),
        )
        .route(
            "/api/admin/forum/bans",
            get(crate::routes::forum::admin_list_bans),
        )
        .route(
            "/api/forum/moderation/user/{userId}/grants",
            get(crate::routes::forum::moderation_user_grants),
        )
        .route(
            "/api/forum/unread",
            get(crate::routes::forum::unread_topics),
        )
        .route(
            "/api/forum/recent",
            get(crate::routes::forum::recent_topics),
        )
        .route(
            "/api/forum/popular",
            get(crate::routes::forum::popular_topics),
        )
        .route("/api/forum/rss", get(crate::routes::forum::forum_rss))
        .route(
            "/api/forum/preferences",
            get(crate::routes::forum::get_forum_prefs)
                .put(crate::routes::forum::set_forum_prefs)
                .post(crate::routes::forum::set_forum_prefs),
        )
        .route(
            "/api/forum/topics/{topicId}/tags",
            get(crate::routes::forum::get_topic_tags)
                .post(crate::routes::forum::set_topic_tags)
                .put(crate::routes::forum::set_topic_tags),
        )
        // ── ActivityPub federation (NodeBB parity, opt-in via ACTIVITYPUB_ENABLED) ─
        .route(
            "/.well-known/webfinger",
            get(crate::activitypub::webfinger::webfinger),
        )
        .route("/actor", get(crate::activitypub::actors::instance_actor))
        .route("/actor/{id}", get(crate::activitypub::actors::legacy_actor))
        .route("/uid/{id}", get(crate::activitypub::actors::user_actor))
        .route(
            "/category/{id}",
            get(crate::activitypub::actors::category_actor),
        )
        .route(
            "/inbox",
            axum::routing::post(crate::activitypub::inbox::inbox)
                .get(crate::activitypub::outbox::outbox),
        )
        .route(
            "/actor/inbox",
            axum::routing::post(crate::activitypub::inbox::actor_inbox),
        )
        .route("/outbox", get(crate::activitypub::outbox::outbox))
        .route(
            "/api/activitypub/status",
            get(crate::activitypub::ap_status),
        )
        // ── Fic Requests (prompt board) ──────────────────────────────
        .route(
            "/api/requests",
            axum::routing::post(crate::routes::requests::create_request),
        )
        .route("/api/requests", get(crate::routes::requests::list_requests))
        .route(
            "/api/requests/{id}",
            get(crate::routes::requests::get_request),
        )
        .route(
            "/api/requests/{id}",
            axum::routing::delete(crate::routes::requests::delete_request),
        )
        .route(
            "/api/requests/{id}/answers",
            axum::routing::post(crate::routes::requests::add_answer),
        )
        .route(
            "/api/requests/{id}/answers/{aid}",
            axum::routing::delete(crate::routes::requests::delete_answer),
        )
        .route(
            "/api/requests/{id}/answers/{aid}/vote",
            axum::routing::post(crate::routes::requests::vote_answer),
        )
        .route(
            "/api/requests/{id}/upvote",
            axum::routing::post(crate::routes::requests::upvote_request),
        )
        .route(
            "/api/requests/{id}/accept/{aid}",
            axum::routing::post(crate::routes::requests::accept_answer),
        )
        .route(
            "/api/requests/{id}/candidates",
            get(crate::routes::requests::candidates),
        )
        .route("/api/docs/ask", get(crate::routes::docs::ask_docs))
        .route(
            "/api/admin/docs/ingest",
            axum::routing::post(crate::routes::docs::ingest_docs),
        )
        // ── Translate-everything (docs/plans/translation-everything.md) ──
        // M1: machine/human translation API + M2: universal curator proposals
        .route(
            "/api/translate/batch",
            axum::routing::post(crate::routes::translate::batch),
        )
        .route(
            "/api/translate/enqueue",
            axum::routing::post(crate::routes::translate::enqueue),
        )
        .route(
            "/api/translate/status",
            get(crate::routes::translate::status),
        )
        .route(
            "/api/translate/coverage",
            get(crate::routes::translate::coverage),
        )
        .route(
            "/api/translate/flag",
            axum::routing::post(crate::routes::proposals::flag_translation),
        )
        .route(
            "/api/proposals",
            axum::routing::post(crate::routes::proposals::create),
        )
        .route(
            "/api/proposals/{id}",
            get(crate::routes::proposals::get_one),
        )
        .route(
            "/api/proposals/{id}/vote",
            axum::routing::post(crate::routes::proposals::vote),
        )
        .route(
            "/api/proposals/{id}/decide",
            axum::routing::post(crate::routes::proposals::decide),
        )
        .route(
            "/api/curator/proposals",
            get(crate::routes::proposals::queue),
        )
        .route(
            "/api/follows/followers/{user_id}",
            get(crate::routes::follows::get_followers_handler),
        )
        // Follow updates feed + refresh-fic (program items 6 & 7)
        .route(
            "/api/v1/updates",
            get(crate::routes::updates::updates_handler),
        )
        .route(
            "/api/v1/follows/{id}/seen",
            axum::routing::post(crate::routes::updates::mark_seen_handler),
        )
        .route(
            "/api/v1/works/{url_id}/refresh",
            axum::routing::post(crate::routes::updates::refresh_fic_handler),
        )
        // Badge & Leaderboard routes
        .route(
            "/api/badges",
            get(crate::routes::badges::list_badge_definitions_handler),
        )
        .route(
            "/api/users/{id}/badges",
            get(crate::routes::badges::get_user_badges_handler),
        )
        .route(
            "/api/leaderboard/curators/weekly",
            get(crate::routes::badges::leaderboard_weekly_handler),
        )
        .route(
            "/api/leaderboard/curators/monthly",
            get(crate::routes::badges::leaderboard_monthly_handler),
        )
        // Shelf / Collection routes
        .route(
            "/api/shelves",
            axum::routing::post(crate::routes::shelves::create_shelf_handler),
        )
        .route(
            "/api/shelves",
            get(crate::routes::shelves::list_shelves_handler),
        )
        .route(
            "/api/shelves/{id}",
            axum::routing::delete(crate::routes::shelves::delete_shelf_handler),
        )
        .route(
            "/api/shelves/add",
            axum::routing::post(crate::routes::shelves::add_work_to_shelf_handler),
        )
        .route(
            "/api/shelves/{shelf_id}/works/{work_id}",
            axum::routing::delete(crate::routes::shelves::remove_work_from_shelf_handler),
        )
        .route(
            "/api/shelves/{shelf_id}/works",
            get(crate::routes::shelves::list_works_in_shelf_handler),
        )
        // Reading list (bundles) routes
        .route(
            "/api/lists",
            axum::routing::post(crate::routes::lists::create_list_handler),
        )
        .route("/api/lists", get(crate::routes::lists::list_lists_handler))
        .route(
            "/api/lists/{id}",
            get(crate::routes::lists::get_list_handler),
        )
        .route(
            "/api/lists/{id}",
            axum::routing::patch(crate::routes::lists::update_list_handler),
        )
        .route(
            "/api/lists/{id}",
            axum::routing::delete(crate::routes::lists::delete_list_handler),
        )
        .route(
            "/api/lists/{id}/items",
            axum::routing::post(crate::routes::lists::add_item_handler),
        )
        .route(
            "/api/lists/{id}/items/{work_id}",
            axum::routing::delete(crate::routes::lists::remove_item_handler),
        )
        // Collection (AO3-style) routes
        .route(
            "/api/collections",
            get(crate::routes::collections::browse_collections_handler),
        )
        .route(
            "/api/collections",
            axum::routing::post(crate::routes::collections::create_collection_handler),
        )
        .route(
            "/api/collections/{id}",
            get(crate::routes::collections::get_collection_handler),
        )
        .route(
            "/api/collections/by-slug/{slug}",
            get(crate::routes::collections::get_collection_by_slug_handler),
        )
        .route(
            "/api/collections/{id}",
            axum::routing::patch(crate::routes::collections::update_collection_handler),
        )
        .route(
            "/api/collections/{id}",
            axum::routing::delete(crate::routes::collections::delete_collection_handler),
        )
        .route(
            "/api/collections/{id}/items",
            axum::routing::post(crate::routes::collections::add_collection_item_handler),
        )
        .route(
            "/api/collections/{id}/items/{work_id}",
            axum::routing::delete(crate::routes::collections::remove_collection_item_handler),
        )
        .route(
            "/api/collections/{id}/bookmark",
            axum::routing::post(crate::routes::collections::bookmark_collection_handler),
        )
        .route(
            "/api/collections/{id}/bookmark",
            axum::routing::delete(crate::routes::collections::unbookmark_collection_handler),
        )
        .route(
            "/api/collections/{id}/bookmarkers",
            get(crate::routes::collections::list_collection_bookmarkers_handler),
        )
        .route(
            "/api/collections/{id}/requests",
            get(crate::routes::collections::list_collection_item_requests_handler),
        )
        .route(
            "/api/collections/{id}/requests/{req_id}/approve",
            axum::routing::post(crate::routes::collections::approve_collection_item_handler),
        )
        .route(
            "/api/collections/{id}/requests/{req_id}/reject",
            axum::routing::post(crate::routes::collections::reject_collection_item_handler),
        )
        .route(
            "/api/collections/{id}/requests/{req_id}/vote",
            axum::routing::post(crate::routes::collections::vote_collection_item_request_handler),
        )
        .route(
            "/api/collections/view/{id}",
            get(crate::routes::collections::view_collection_handler),
        )
        .route(
            "/api/collections/{id}/challenge",
            get(crate::routes::collections::challenge_info_handler),
        )
        .route(
            "/api/collections/{id}/challenge/signup",
            axum::routing::post(crate::routes::collections::challenge_signup_handler),
        )
        .route(
            "/api/collections/{id}/challenge/assign",
            axum::routing::post(crate::routes::collections::challenge_assign_handler),
        )
        .route(
            "/api/collections/{cid}/challenge/{aid}/claim",
            axum::routing::post(crate::routes::collections::challenge_claim_handler),
        )
        .route("/api/pseuds", get(crate::routes::pseuds::list_pseuds))
        .route(
            "/api/pseuds",
            axum::routing::post(crate::routes::pseuds::create_pseud),
        )
        .route(
            "/api/pseuds/{id}",
            axum::routing::patch(crate::routes::pseuds::update_pseud),
        )
        .route(
            "/api/pseuds/{id}",
            axum::routing::delete(crate::routes::pseuds::delete_pseud),
        )
        .route(
            "/api/pseuds/creatorships",
            get(crate::routes::pseuds::list_creatorships),
        )
        .route(
            "/api/pseuds/invite",
            axum::routing::post(crate::routes::pseuds::invite_creatorship),
        )
        .route(
            "/api/pseuds/creatorships/{id}/approve",
            axum::routing::post(crate::routes::pseuds::approve_creatorship),
        )
        .route(
            "/api/pseuds/creatorships/{id}/reject",
            axum::routing::post(crate::routes::pseuds::reject_creatorship),
        )
        .route("/api/skins", get(crate::routes::skins::list_skins))
        .route(
            "/api/skins",
            axum::routing::post(crate::routes::skins::create_skin),
        )
        .route("/api/skins/{id}", get(crate::routes::skins::get_skin))
        .route(
            "/api/skins/{id}",
            axum::routing::patch(crate::routes::skins::update_skin),
        )
        .route(
            "/api/skins/{id}",
            axum::routing::delete(crate::routes::skins::delete_skin),
        )
        .route(
            "/api/skins/{id}/apply",
            axum::routing::post(crate::routes::skins::set_user_skin),
        )
        .route("/api/me/skin", get(crate::routes::skins::get_user_skin))
        .route(
            "/api/works/{work_id}/skins/{skin_id}",
            axum::routing::post(crate::routes::skins::assign_work_skin),
        )
        // Reading status routes
        .route(
            "/api/reading/status",
            axum::routing::post(crate::routes::quests::update_reading_status_handler),
        )
        .route(
            "/api/reading/list",
            get(crate::routes::quests::get_reading_list_handler),
        )
        // Quest routes
        .route(
            "/api/users/{id}/reading-stats",
            get(crate::routes::quests::get_reading_stats_handler),
        )
        .route(
            "/api/reading/record",
            axum::routing::post(crate::routes::quests::record_read_handler),
        )
        .route(
            "/api/users/{id}/streak",
            get(crate::routes::quests::get_streak_handler),
        )
        // Reading history routes (AO3-style per-visit log)
        .route(
            "/api/reading/history",
            get(crate::routes::quests::get_reading_history_handler),
        )
        .route(
            "/api/reading/history",
            axum::routing::post(crate::routes::quests::record_read_history_handler),
        )
        .route(
            "/api/reading/history/{id}",
            delete(crate::routes::quests::delete_read_history_handler),
        )
        .route(
            "/api/reading/history/clear",
            axum::routing::post(crate::routes::quests::clear_read_history_handler),
        )
        // Entity recommendations: similar tags/fandoms/authors/collections/users.
        .route(
            "/api/v0/recommendations/entities",
            get(crate::recommender::routes::entity_recs_handler),
        )
        // Public fic-content APIs — reader HTML, export URLs, meta, similar.
        // Served fresh when online, stale (cached) copy when offline so
        // previously-opened fics keep reading without a connection.
        .route("/sw.js", get(serve_sw))
        .route(
            "/api/reader/{url_id}",
            get(crate::routes::reader::reader_handler),
        )
        .route(
            "/api/reader/{url_id}/sequel",
            get(crate::routes::reader::reader_sequel_handler),
        )
        .route(
            "/api/reader/{url_id}/related",
            get(crate::routes::reader::reader_related_handler),
        )
        .route(
            "/api/reader/{url_id}/sequential",
            get(crate::routes::reader::reader_sequential_handler),
        )
        .route(
            "/api/reader/{url_id}/next-up",
            get(crate::routes::reader::reader_next_up_handler),
        )
        .route(
            "/api/reader/{url_id}/marginalia",
            get(crate::routes::reader::reader_marginalia_handler),
        )
        // Locale & Translation routes
        .route(
            "/api/locales",
            get(crate::routes::locales::list_locales_handler),
        )
        .route(
            "/api/translations/{locale_code}",
            get(crate::routes::locales::get_ui_translations_handler),
        )
        .route(
            "/api/works/{id}/translations",
            axum::routing::post(crate::routes::locales::upsert_work_translation_handler),
        )
        .route(
            "/api/works/{id}/translations/{locale_code}",
            get(crate::routes::locales::get_work_translation_handler),
        )
        .route(
            "/api/works/{id}/chapter-translations/{locale}",
            get(crate::routes::locales::get_chapter_translations),
        )
        .route(
            "/api/works/{id}/chapter-translations/{locale}",
            axum::routing::put(crate::routes::locales::upsert_chapter_translations),
        )
        // Trending routes
        .route(
            "/api/trending",
            get(crate::routes::trending::trending_general_handler),
        )
        .route(
            "/api/trending/tags",
            get(crate::routes::trending::trending_tags_handler),
        )
        .route(
            "/api/trending/tag/{tag_type_id}/{tag_name}",
            get(crate::routes::trending::trending_by_tag_handler),
        )
        // Fandom routes
        .route("/api/fandoms", get(crate::routes::fandom::list_fandoms))
        .route(
            "/api/fandoms/{slug}",
            get(crate::routes::fandom::get_fandom),
        )
        // Work proposal routes
        .route(
            "/api/work-proposals",
            axum::routing::post(crate::routes::work_proposals::create_proposal_handler),
        )
        .route(
            "/api/work-proposals",
            get(crate::routes::work_proposals::list_proposals_handler),
        )
        .route(
            "/api/work-proposals/{id}",
            get(crate::routes::work_proposals::get_proposal_handler),
        )
        .route(
            "/api/work-proposals/{id}/vote",
            axum::routing::post(crate::routes::work_proposals::vote_proposal_handler),
        )
        // Threaded comment routes
        .route(
            "/api/works/{url_id}/comments",
            get(crate::routes::comments::get_comments_handler),
        )
        .route(
            "/api/works/{url_id}/comments",
            axum::routing::post(crate::routes::comments::post_comment_handler),
        )
        .route(
            "/api/comment/{id}",
            axum::routing::delete(crate::routes::comments::delete_comment_handler),
        )
        .route(
            "/api/comment/{id}/hide",
            axum::routing::patch(crate::routes::comments::hide_comment_handler),
        )
        .route(
            "/api/comment/{id}",
            axum::routing::patch(crate::routes::comments::edit_comment_handler),
        )
        // Manual upload routes
        .route(
            "/api/upload",
            axum::routing::post(crate::routes::upload::handle_manual_upload),
        )
        .route(
            "/api/upload/{url_id}/update",
            axum::routing::post(crate::routes::upload::handle_fic_update),
        )
        .route(
            "/api/upload/{url_id}",
            axum::routing::delete(crate::routes::upload::handle_fic_delete),
        )
        // Send-to-Kindle route
        .route(
            "/api/send-to-kindle",
            axum::routing::post(crate::routes::kindle::send_to_kindle_handler),
        )
        // OPDS catalog routes
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route(
            "/opds/popular",
            get(crate::routes::opds::feeds::popular_feed),
        )
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route(
            "/opds/tags/{type_id}",
            get(crate::routes::opds::tags::tags_by_type),
        )
        .route(
            "/opds/tags/{type_id}/{tag_name}",
            get(crate::routes::opds::tags::fics_by_tag),
        )
        .route(
            "/opds/authors",
            get(crate::routes::opds::authors::author_list),
        )
        .route(
            "/opds/recommendations/popular",
            get(crate::routes::opds::recommendations::popular_recommendations),
        )
        .route(
            "/opds/recommendations",
            get(crate::routes::opds::recommendations::fic_recommendations),
        )
        .route(
            "/opds/search",
            get(crate::routes::opds::search::search_feed),
        )
        .route(
            "/opds/shelves",
            get(crate::routes::opds::shelves::shelf_list),
        )
        .route(
            "/opds/shelf/{shelf_id}",
            get(crate::routes::opds::shelves::shelf_contents),
        )
        .route(
            "/opds/manifest",
            get(crate::routes::opds::manifest::manifest),
        )
        // RSS/Atom feeds. The per-fic route uses a bare {url_id} segment
        // (axum 0.8 forbids mixed literal+param segments like {url_id}.xml);
        // the handler strips a trailing ".xml" so /feed/works/<id>.xml works.
        .route("/feed.xml", get(crate::routes::rss::new_arrivals_feed))
        .route("/feed/follows.xml", get(crate::routes::rss::follows_feed))
        .route("/feed/works/{url_id}", get(crate::routes::rss::work_feed))
        // Public per-saved-search feed (user_id + search_id path params).
        .route(
            "/feed/saved/{user_id}/{search_id}",
            get(crate::routes::saved_search::saved_search_feed),
        )
        // Analytics routes
        .route(
            "/api/analytics",
            get(crate::routes::analytics::analytics_handler),
        )
        .route(
            "/api/analytics/user/{client_id}",
            get(crate::routes::analytics::user_stats_handler),
        )
        .route(
            "/api/reading/analytics",
            get(crate::routes::analytics::personal_reading_handler),
        )
        .route(
            "/api/authors/{id}/analytics",
            get(crate::routes::analytics::author_analytics_handler),
        )
        .route(
            "/api/admin/analytics",
            get(crate::routes::analytics::admin_analytics_handler),
        )
        .route(
            "/api/admin/endpoint-usage",
            get(crate::routes::analytics::endpoint_usage_handler),
        )
        // Modlog (transparent moderation log — any logged-in user)
        .route("/api/modlog", get(crate::routes::modlog::modlog_handler))
        .route("/api/me/trust", get(crate::routes::trust::my_trust))
        .route("/api/admin/trust", get(crate::routes::trust::admin_trust))
        .route(
            "/api/admin/trust/{id}",
            axum::routing::put(crate::routes::trust::admin_set_trust),
        )
        .route("/api/admin/digest", get(crate::routes::trust::admin_digest))
        // Extension marketplace (trust-gated publishing, open gallery).
        .nest(
            "/api/extensions",
            crate::routes::extensions::router(state.clone()),
        )
        // ── Admin routes ────────────────────────────────────────────
        .route(
            "/api/admin/heal",
            axum::routing::post(crate::routes::heal::heal_handler),
        )
        .route(
            "/api/admin/heal/extractions",
            get(crate::routes::heal::list_extractions_handler),
        )
        .route(
            "/api/admin/heal/extractions/{id}/trust",
            axum::routing::post(crate::routes::heal::trust_extraction_handler),
        )
        .route(
            "/api/admin/heal/replay-pending",
            axum::routing::post(crate::routes::heal::replay_pending_handler),
        )
        .route(
            "/api/admin/moderation/queue",
            get(crate::routes::admin::mod_queue),
        )
        .route(
            "/api/admin/content-scan",
            get(crate::routes::admin::list_content_scan),
        )
        .route(
            "/api/admin/content-scan/run",
            axum::routing::post(crate::routes::admin::run_content_scan),
        )
        .route(
            "/api/admin/content-scan/{url_id}/review",
            axum::routing::post(crate::routes::admin::review_content_scan),
        )
        .route(
            "/api/admin/search-mining",
            get(crate::routes::admin::search_mining),
        )
        .route(
            "/api/admin/dedupe/embeddings",
            axum::routing::post(crate::routes::admin::run_embedding_dedupe),
        )
        .route(
            "/api/admin/backfill/tags",
            axum::routing::post(crate::routes::admin::admin_backfill_tags),
        )
        .route(
            "/api/admin/backfill/bodies",
            axum::routing::post(crate::routes::admin::admin_backfill_bodies),
        )
        .route(
            "/api/admin/moderation/comments",
            get(crate::routes::admin::moderation_comments),
        )
        .route(
            "/api/admin/moderation/comments/{id}/hide",
            axum::routing::post(crate::routes::admin::admin_hide_comment),
        )
        .route(
            "/api/admin/moderation/comments/{id}/delete",
            axum::routing::post(crate::routes::admin::admin_delete_comment),
        )
        .route(
            "/api/admin/blacklist",
            get(crate::routes::admin::list_blacklist),
        )
        .route(
            "/api/admin/blacklist/fic",
            axum::routing::post(crate::routes::admin::blacklist_fic),
        )
        .route(
            "/api/admin/blacklist/author",
            axum::routing::post(crate::routes::admin::blacklist_author),
        )
        .route(
            "/api/admin/moderation/approve/{work_id}",
            axum::routing::post(crate::routes::admin::approve_upload),
        )
        .route(
            "/api/admin/moderation/reject/{work_id}",
            axum::routing::post(crate::routes::admin::reject_upload),
        )
        .route(
            "/api/admin/scraper-health",
            get(crate::routes::admin::scraper_health),
        )
        .route("/api/admin/users", get(crate::routes::admin::admin_users))
        .route(
            "/api/admin/users/{id}/role",
            axum::routing::put(crate::routes::admin::set_user_role),
        )
        .route(
            "/api/admin/reputation/award",
            axum::routing::post(crate::routes::admin::rep_award_handler),
        )
        .route(
            "/api/admin/users/{id}/ban",
            axum::routing::put(crate::routes::admin::toggle_ban),
        )
        .route("/api/admin/stats", get(crate::routes::admin::admin_stats))
        .route("/api/admin/bots", get(crate::routes::admin::admin_bots))
        .route(
            "/api/admin/bots/{client_id}/shadowban",
            axum::routing::post(crate::routes::admin::admin_bot_shadowban),
        )
        .route(
            "/api/admin/bots/{client_id}/unshadowban",
            axum::routing::post(crate::routes::admin::admin_bot_unshadowban),
        )
        .route(
            "/api/admin/realtime",
            get(crate::routes::admin::admin_realtime),
        )
        .route(
            "/api/admin/search-analytics",
            get(crate::routes::admin::admin_search_analytics),
        )
        .route(
            "/api/admin/roadmap-consensus",
            get(crate::routes::admin::admin_roadmap_consensus),
        )
        // Translation review workflow (draft → post-edit → approve/reject)
        .route(
            "/api/admin/translations",
            get(crate::routes::admin::admin_list_translations),
        )
        .route(
            "/api/admin/translations/{id}/approve",
            axum::routing::post(crate::routes::admin::admin_approve_translation),
        )
        .route(
            "/api/admin/translations/{id}/reject",
            axum::routing::post(crate::routes::admin::admin_reject_translation),
        )
        .route(
            "/api/admin/translations/{id}/edit",
            axum::routing::post(crate::routes::admin::admin_edit_translation),
        )
        .route(
            "/api/admin/chapter-translation-versions/{id}/approve",
            axum::routing::post(crate::routes::admin::admin_approve_chapter_translation_version),
        )
        .route(
            "/api/admin/chapter-translation-versions/{id}/reject",
            axum::routing::post(crate::routes::admin::admin_reject_chapter_translation_version),
        )
        // Rating / warning verification queue
        .route(
            "/api/admin/rating-checks",
            get(crate::routes::admin::admin_rating_checks),
        )
        .route(
            "/api/admin/rating-checks/{work_id}/verify",
            axum::routing::post(crate::routes::admin::admin_verify_rating),
        )
        // Character / relationship score fixing
        .route(
            "/api/admin/characters/{id}/score",
            axum::routing::put(crate::routes::admin::admin_fix_tag_score),
        )
        // Auto-tagger: zero-shot tag classification + review queue
        .route(
            "/api/admin/auto-tag",
            axum::routing::post(crate::routes::auto_tag::auto_tag_fic),
        )
        .route(
            "/api/admin/auto-tag/backfill",
            axum::routing::post(crate::routes::auto_tag::auto_tag_backfill),
        )
        .route(
            "/api/admin/auto-tag/queue",
            get(crate::routes::auto_tag::auto_tag_queue),
        )
        .route(
            "/api/admin/auto-tag/approve/{url_id}/{tag_id}",
            axum::routing::post(crate::routes::auto_tag::auto_tag_approve),
        )
        .route(
            "/api/admin/auto-tag/dismiss/{url_id}/{tag_id}",
            axum::routing::post(crate::routes::auto_tag::auto_tag_dismiss),
        )
        // ── Bulk admin actions ────────────────────────────────────
        .route(
            "/api/admin/bulk/refresh",
            axum::routing::post(crate::routes::bulk::bulk_refresh),
        )
        .route(
            "/api/admin/bulk/auto-tag",
            axum::routing::post(crate::routes::bulk::bulk_auto_tag),
        )
        .route(
            "/api/admin/fics/search",
            get(crate::routes::bulk::search_fics_for_admin),
        )
        // ── Admin metadata correction ──────────────────────────────
        .route(
            "/api/admin/works/{id}/metadata",
            axum::routing::put(crate::routes::admin::update_work_metadata),
        )
        // ── User reports ───────────────────────────────────────────
        .route(
            "/api/reports",
            axum::routing::post(crate::routes::reports::create_report),
        )
        .route(
            "/api/admin/reports",
            get(crate::routes::reports::list_reports),
        )
        .route(
            "/api/admin/reports/{id}/resolve",
            axum::routing::post(crate::routes::reports::resolve_report),
        )
        // Roadmap Consensus Engine
        .route(
            "/api/roadmap/suggest",
            axum::routing::post(crate::routes::roadmap::suggest_handler),
        )
        .route(
            "/api/roadmap/arena",
            get(crate::routes::roadmap::arena_handler),
        )
        .route(
            "/api/roadmap/vote",
            axum::routing::post(crate::routes::roadmap::vote_handler),
        )
        .route(
            "/api/roadmap/consensus",
            get(crate::routes::roadmap::consensus_handler),
        )
        // Roadmap Kanban + Changelog
        .route(
            "/api/roadmap/features",
            get(crate::routes::roadmap::features_list_handler),
        )
        .route(
            "/api/roadmap/features/{id}",
            axum::routing::patch(crate::routes::roadmap::feature_move_handler),
        )
        .route(
            "/api/roadmap/changelog",
            get(crate::routes::roadmap::changelog_list_handler),
        )
        .route(
            "/api/roadmap/changelog",
            axum::routing::post(crate::routes::roadmap::changelog_create_handler),
        )
        // ── Feature flags & progression (v3) ─────────────────────
        .route("/api/features", get(routes::features::list_features))
        .route(
            "/api/features/available",
            get(routes::features::available_features),
        )
        .route(
            "/api/features/{slug}/enable",
            post(routes::features::enable_feature),
        )
        .route(
            "/api/features/{slug}/disable",
            post(routes::features::disable_feature),
        )
        .route(
            "/api/me/progression",
            get(routes::progression::get_progression),
        )
        .route(
            "/api/me/prefs",
            get(routes::progression::get_user_prefs).put(routes::progression::set_user_prefs),
        )
        .route(
            "/api/me/layout/{page}",
            get(routes::progression::get_user_layout).put(routes::progression::set_user_layout),
        )
        .route(
            "/api/me/views",
            get(routes::progression::get_user_views).post(routes::progression::create_user_view),
        )
        .route(
            "/api/me/views/{id}",
            axum::routing::delete(routes::features::delete_user_view),
        )
        .route(
            "/api/me/views/{id}/toggle-pin",
            axum::routing::put(routes::features::toggle_view_pin),
        )
        .route("/api/admin/features", get(routes::features::admin_features))
        .route(
            "/api/admin/features/{slug}",
            axum::routing::put(routes::features::update_feature),
        )
        .route(
            "/api/admin/features/stats",
            get(routes::features::feature_stats),
        )
        // ── Customization (theme, nav, widgets) ───────────────────
        .route(
            "/api/me/theme",
            get(routes::customization::get_theme).put(routes::customization::set_theme),
        )
        .route("/api/nav", get(routes::customization::get_nav))
        .route("/api/widgets", get(routes::customization::get_widgets))
        // Legacy redirect routes
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        // Static frontend
        .fallback_service(
            crate::frontend::cache_headers::CacheHeadersLayer.layer(
                ServeDir::new(&frontend_dir)
                    .append_index_html_on_directories(true)
                    .fallback(ServeFile::new(frontend_dir.join("index.html"))),
            ),
        )
        // Middleware
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::routes::analytics::track_usage,
        ))
        // Visitor funnel — mint a `vh_vis` cookie on first request and
        // expose the parsed VisitorId to downstream handlers via request
        // extensions so the register/login flows can merge anonymous state.
        .layer(axum::middleware::from_fn(visitor_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        // Shared state
        .with_state(state)
}

/// Remote info handler: GET /api/remote
async fn remote_handler(
    axum::extract::ConnectInfo(remote_addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
        "is_automated": false,
    }))
}

/// GET /sw.js — the PWA service worker. Served by ServeDir with a `no-cache`
/// Cache-Control, but tower-http's mime_guess map lacks a type for the `.js`
/// extension's modern `text/javascript` expectation, so we serve it explicitly
/// to guarantee the correct Content-Type (service workers are rejected by
/// browsers when served with an incompatible MIME type).
async fn serve_sw(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Response<Body>, crate::error::AppError> {
    serve_sw_from(&state.config.frontend_dir).await
}

async fn serve_sw_from(
    frontend_dir: &std::path::Path,
) -> Result<Response<Body>, crate::error::AppError> {
    let content = tokio::fs::read_to_string(frontend_dir.join("sw.js"))
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to read sw.js: {}", e)))?;
    Ok(Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "text/javascript")
        // Service workers MUST never be cached by intermediate caches: a
        // stale sw.js means the browser keeps running an old offline-reading
        // strategy (and an old precache list) for months. Explicit no-store
        // also stops Cloudflare edge caches from serving a stale worker.
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(content))
        .map_err(|e| {
            crate::error::AppError::Internal(format!("Failed to build sw.js response: {}", e))
        })?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sw_served_with_javascript_content_type() {
        // The handler reads from the configured frontend dir. In tests we pass
        // the repo-relative path; if the frontend build is absent (cargo test
        // without `npm run build`) this is a static-serving contract check,
        // not a build requirement — nothing to assert against.
        let content = tokio::fs::read_to_string("./frontend/build/sw.js").await;
        if content.is_err() {
            return;
        }
        let resp = serve_sw_from(std::path::Path::new("./frontend/build"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/javascript"
        );
    }
}
