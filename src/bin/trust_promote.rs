//! `trust-promote` — automatic trust promotion pass (nightly via cron).
//!
//! Usage: `cargo run --bin trust-promote` (with `DATABASE_URL` set).
//! Promotes TL0→TL4 from reading/community signals, runs spam-based
//! revocation at TL3+, and prints TL5 candidates for the weekly digest.
//! Idempotent — safe to run repeatedly.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("error: DATABASE_URL must be set (run with .env loaded)");
            return ExitCode::FAILURE;
        }
    };

    let pool = match sqlx::PgPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("error: could not connect to database: {err}");
            return ExitCode::FAILURE;
        }
    };

    let config = fichub::config::Config::from_env();
    match fichub::services::trust::run_trust_promotion(&pool, &config).await {
        Ok((promotions, candidates)) => {
            println!(
                "trust-promote: {} automatic promotion(s) applied",
                promotions.len()
            );
            for (uid, from, to, reason) in &promotions {
                println!("  user #{uid}: TL{from} -> TL{to} ({reason})");
            }
            println!(
                "trust-promote: {} TL5 candidates surfaced for admin confirmation",
                candidates.len()
            );
            for (uid, m) in &candidates {
                println!(
                    "  user #{uid}: {} forum posts, {:.0}% days active (30d), {} reports filed",
                    m.forum_posts,
                    m.days_active_30 as f64 / 30.0 * 100.0,
                    m.reports_filed,
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: promotion pass failed: {e}");
            ExitCode::FAILURE
        }
    }
}
