//! ficnexus binary: the exporter HTTP server.
//!
//! All logic lives in the library crate; the binary only sets up `.env`,
//! tracing and the config, then hands off to `server::run`.

use fichub::config::Config;
use fichub::server;

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
    let config = Config::from_env();

    tracing::info!("Starting ficnexus exporter on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
