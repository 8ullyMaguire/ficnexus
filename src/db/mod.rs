pub mod models;
pub mod queries;
pub mod reviews;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::Path;
use std::time::Duration;

/// Initialize the database connection pool and run migrations
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    // Run migrations from the migrations directory relative to the binary,
    // UNLESS FICHUB_SKIP_MIGRATIONS is set (deploy.sh runs `migrate` as an
    // explicit deploy step before restarting the service, so boot stays
    // boring — no migration surprises mid-restart).
    if std::env::var("FICHUB_SKIP_MIGRATIONS").is_ok() || std::env::var("FICNEXUS_SKIP_MIGRATIONS").is_ok() {
        tracing::info!("FICHUB_SKIP_MIGRATIONS set — skipping migrations on boot");
        return Ok(pool);
    }

    let manifest_migrations = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
        exe_path.parent()
            .map(|d| d.join("migrations"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| manifest_migrations.clone())
    } else {
        manifest_migrations.clone()
    };

    if migrations_path.exists() {
        // set_ignore_missing: the migrations directory was consolidated (72
        // files → single 001_initial.sql baseline + renumbered feature
        // migrations); pre-consolidation databases record historical versions
        // that no longer exist on disk. Ignore those orphans — checksum
        // validation still catches modified migrations we do ship.
        let mut migrator = sqlx::migrate::Migrator::new(migrations_path).await?;
        migrator.set_ignore_missing(true);
        migrator.run(&pool).await?;
        tracing::info!("Database migrations applied");
    } else {
        tracing::warn!("Migrations directory not found at {:?}", migrations_path);
    }

    Ok(pool)
}
