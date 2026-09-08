mod schema;
mod import;

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::Parser;
use sqlx::PgPool;

#[derive(Parser)]
#[command(
    name = "forum-import",
    about = "NodeBB JSON export → FicNexus forum_* tables importer",
    version
)]
struct Cli {
    /// PostgreSQL connection URL.
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// Path to NodeBB JSON export file.
    #[arg(short, long)]
    input: PathBuf,

    /// Dry run — validate without writing.
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Read input file.
    let content = std::fs::read_to_string(&cli.input)
        .context(format!("reading input file: {}", cli.input.display()))?;
    let data: schema::ImportFile = serde_json::from_str(&content)
        .context("parsing JSON input")?;

    tracing::info!(
        "Loaded import file: {} users, {} categories, {} topics, {} posts",
        data.users.len(),
        data.categories.len(),
        data.topics.len(),
        data.posts.len()
    );

    if cli.dry_run {
        tracing::info!("DRY RUN — no data will be written");
    }

    // Connect to database.
    let pool = PgPool::connect(&cli.database_url)
        .await
        .context("connecting to database")?;

    tracing::info!("Connected to database");

    // Run import.
    let stats = import::run_import(&pool, &data, cli.dry_run).await?;

    tracing::info!(
        "Import complete: {} users inserted ({} skipped), {} categories ({} skipped), {} topics ({} skipped), {} posts ({} skipped), {} notifications",
        stats.users_inserted, stats.users_skipped,
        stats.categories_inserted, stats.categories_skipped,
        stats.topics_inserted, stats.topics_skipped,
        stats.posts_inserted, stats.posts_skipped,
        stats.notifications_created
    );

    if cli.dry_run {
        tracing::info!("DRY RUN — no data was written");
    }

    Ok(())
}
