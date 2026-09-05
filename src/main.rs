pub mod activitypub;
pub mod body_cache;
pub mod cache;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod export;
pub mod fic_suggestions;
pub mod frontend;
pub mod heal;
pub mod ingest;
pub mod leaderboard;
pub mod limiter;
pub mod modlog;
pub mod progression;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod services;
pub mod tags;
pub mod trending;
pub mod user_formats;
pub mod visitor;
pub mod works;

/// Matches `@username` tokens (FicNexus usernames: 2-32 alphanumeric/underscore)
/// used by forum mention notifications. Compile once at startup.
/// NOTE: no look-around — `regex_lite` doesn't support it. The word-boundary
/// check for `foo@bar` is done in `extract_mentions` (forum.rs).
pub static MENTION_RE: std::sync::LazyLock<regex_lite::Regex> = std::sync::LazyLock::new(|| {
    regex_lite::Regex::new(r"@([A-Za-z0-9_]{2,32})\b").expect("valid mention regex")
});

#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting ficnexus server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
