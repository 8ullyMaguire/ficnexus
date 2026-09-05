//! `fichub migrate` — run database migrations as an explicit deploy step.
//!
//! Deploy.sh runs this BEFORE restarting the service (the service boots with
//! FICHUB_SKIP_MIGRATIONS=1 so boot stays boring). Running migrations in a
//! separate step means a failed/mismatched migration can never take the
//! service down mid-restart — you see the error here, in the deploy log.
//!
//! Usage:
//!   fichub migrate                      # apply pending migrations
//!   fichub migrate --dry-run            # print pending without applying
use sqlx::postgres::PgPoolOptions;
use std::path::Path;
use std::time::Duration;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let dry_run = std::env::args().any(|a| a == "--dry-run");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let manifest_migrations = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
        exe_path
            .parent()
            .map(|d| d.join("migrations"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| manifest_migrations.clone())
    } else {
        manifest_migrations.clone()
    };

    // The migrations directory was consolidated (72 files → single
    // 001_initial.sql baseline + a few renumbered feature migrations), so
    // databases migrated before the consolidation still record versions that
    // no longer exist in the directory. Ignore those historical orphans —
    // validation still catches *modified* migrations we do ship.
    let mut migrator = sqlx::migrate::Migrator::new(migrations_path.as_path())
        .await
        .expect("failed to load migrations");
    migrator.set_ignore_missing(true);

    if dry_run {
        // Migrator doesn't expose a public applied-migrations list in this
        // sqlx version; a dry-run is informational only. Print what exists.
        println!(
            "DRY-RUN: {} migration files loaded from {:?}",
            migrator.iter().count(),
            migrations_path
        );
        return;
    }

    migrator.run(&pool).await.expect("migration failed");
    println!(
        "Migrations applied successfully ({} migration files).",
        migrator.iter().count()
    );
}
