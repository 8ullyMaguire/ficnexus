use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::cache::CacheSemaphores;
use crate::config::Config;
use crate::db;
use crate::limiter;
use crate::routes;
use crate::scrape::registry::ScraperRegistry;

/// Shared application state accessible by all handlers.
///
/// This is the exporter's whole dependency surface: Postgres (fic/export
/// cache), Redis (rate-limit buckets + proof-of-work), the scraper registry,
/// the body/export cache, self-heal telemetry and the Wayback fallback.
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    /// Dedicated Redis connection for `/api/health` PING checks. Kept
    /// separate from `redis` because a blocking read parked on a shared
    /// multiplexed connection would make health report `redis:false` while
    /// Redis is fine.
    pub health_redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::TieredRateLimiter>,
    /// Self-healing telemetry for the scrape side of the export path.
    pub heal: crate::heal::HealService,
    /// Wayback Machine fallback — when enabled, the export path falls back to
    /// IA snapshots on Blocked/NotFound/transient scrape failures.
    pub wayback: crate::scrape::wayback::WaybackService,
    /// Ollama client, used by `/api/meta` to resolve author links.
    pub ollama: crate::services::ollama::OllamaClient,
}

/// Build and run the HTTP server
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str()).expect("Invalid Redis URL");
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    let health_redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis for health checks");

    // Build HTTP client (no cookie store — batch downloads use isolated
    // clients with private jars via isolated_scrape_client()).
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

    let ollama_client = crate::services::ollama::OllamaClient::new(
        config.ollama_url.clone(),
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
        heal: heal_service,
        wayback: wayback_service,
        ollama: ollama_client,
    });

    // Build router
    let app = build_router(state.clone()).await;

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
        // The entire UI: a URL box and a format picker.
        .route("/", get(routes::landing::landing))
}

/// Build the exporter router.
///
/// Everything else ficnexus used to serve (accounts, bookmarks, ratings,
/// reviews, comments, requests, forum, progression, recipes, marketplace,
/// follows, lists, RSS, reader, roadmap, leaderboards, ActivityPub) is
/// darkhold's job now; see `docs/EXPORTER-SCOPE.md`.
pub async fn build_router(state: Arc<AppState>) -> Router {
    chunk_export()
        .into()
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
