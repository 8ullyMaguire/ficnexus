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
    /// Realtime connection manager for WebSocket / SSE pubsub.
    pub rt_manager: crate::realtime::ConnectionManager,
    /// JWT secret for token verification. Centralized here so all handlers
    /// read from one source (previously duplicated across 8 files).
    pub jwt_secret: String,
    /// Optional Redis client for cross-instance realtime pubsub.
    pub redis_client: Option<redis::Client>,
}

/// Redirect to root — used by legacy redirect routes.
async fn redirect_to_root() -> Redirect {
    Redirect::to("/")
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
             ORDER BY trust_level DESC, id ASC
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

    // Build HTTP client (no cookie store — batch downloads use isolated
    // clients with private jars via isolated_scrape_client()). Remaining
    // callers (oembed, health, etc.) make single stateless requests.
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
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
        rt_manager: crate::realtime::ConnectionManager::new(),
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into()),
        redis_client: Some(redis_client),
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

    // Start Redis pubsub listener for cross-instance fanout (best-effort).
    {
        let rt_manager = state.rt_manager.clone();
        let redis_client = redis::Client::open(&*config.redis_url)
            .expect("Failed to create Redis client for pubsub");
        tokio::spawn(async move {
            if let Err(e) =
                crate::realtime::pubsub::start_redis_pubsub(redis_client, rt_manager).await
            {
                tracing::warn!("Redis pubsub listener failed (single-instance mode): {}", e);
            }
        });
    }

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

/// Route chunk: the exporter surface. Give it a story URL, get a file.
fn chunk_export() -> impl Into<Router<Arc<AppState>>> {
    Router::new()
        // What this service is, as JSON.
        .route("/api/", get(routes::api_docs::api_docs_handler))
        // The export endpoints (EPUB by default, `format=` for the rest).
        .route("/api/epub", get(routes::export::epub_handler))
        .route(
            "/api/epub/convert",
            get(routes::export::convert_format_handler),
        )
        // Metadata preview for a URL, before downloading.
        .route("/api/meta", get(routes::meta::meta_handler))
        // Health check (DB + Redis ping).
        .route("/api/health", get(routes::health::health_handler))
        // Supported output formats.
        .route("/api/formats", get(crate::user_formats::list_formats))
        // Serve an already-built file from the cache directory.
        .route(
            "/cache/{etype}/{url_id}/{fname}",
            get(routes::cache_download::download_with_hash),
        )
        .route(
            "/cache/{etype}/{url_id}",
            get(routes::cache_download::download_or_export),
        )
        // Proof-of-work challenge for shadowbanned clients (anti-bot).
        .route("/api/pow/challenge", get(routes::pow::challenge_handler))
        .route("/api/pow/solve", post(routes::pow::solve_handler))
}

/// Build the exporter router.
///
/// Everything else ficnexus used to serve (accounts, bookmarks, ratings,
/// reviews, comments, requests, forum, progression, recipes, marketplace,
/// follows, lists, RSS, reader, roadmap, leaderboards, ActivityPub) lives in
/// darkhold now; see `docs/EXPORTER-SCOPE.md`.
async fn build_router(state: Arc<AppState>) -> Router {
    chunk_export()
        .into()
        .with_state(state.clone())
        .fallback_service(
            crate::frontend::cache_headers::CacheHeadersLayer.layer(
                ServeDir::new(&state.config.frontend_dir)
                    .append_index_html_on_directories(true)
                    .fallback(ServeFile::new(state.config.frontend_dir.join("index.html"))),
            ),
        )
        // Middleware
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::routes::analytics::track_usage,
        ))
        // Visitor funnel — mint a `fh_vis` cookie on first request and
        // expose the parsed VisitorId to downstream handlers via request
        // extensions so the register/login flows can merge anonymous state.
        .layer(axum::middleware::from_fn(visitor_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
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
