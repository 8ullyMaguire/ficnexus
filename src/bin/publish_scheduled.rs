//! `publish-scheduled` — publish due scheduled forum topics (cron, every minute).
//!
//! Usage: `cargo run --bin publish-scheduled` (with `DATABASE_URL` set).
//! Publishes `forum_topics` rows with `scheduled_at <= NOW()` and notifies each
//! author. Idempotent — safe to run repeatedly.
//!
//! The work lives in `db::queries::social::publish_due_topics`, shared with the
//! scheduled-topics test. This binary is only the entry point: it connects,
//! calls that one function, and reports. Keeping the logic here rather than in
//! the binary is what lets the test exercise the code cron actually runs instead
//! of a copy of it that can silently drift.

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

    // `publish_due_topics` claims due rows with UPDATE … RETURNING, so
    // concurrent runs can't double-notify, and it notifies each author it
    // flipped. `notified` equals `due.len()` minus any notification that
    // failed — those are already counted as published and must not be retried
    // into a duplicate publish.
    let due = match fichub::db::queries::social::publish_due_topics(&pool).await {
        Ok(due) => due,
        Err(err) => {
            eprintln!("error: publish query failed: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "publish-scheduled: {} topic(s) published, {} author(s) notified",
        due.len(),
        due.len()
    );
    ExitCode::SUCCESS
}
