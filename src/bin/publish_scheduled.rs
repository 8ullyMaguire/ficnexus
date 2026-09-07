//! `publish-scheduled` — publish due scheduled forum topics (cron, every minute).
//!
//! Usage: `cargo run --bin publish-scheduled` (with `DATABASE_URL` set).
//! Flips `forum_topics` rows with `scheduled_at <= NOW()` to published
//! (`scheduled_at = NULL`) and notifies each author via the site
//! notification producer. Idempotent — safe to run repeatedly.

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

    // Claim due rows first so concurrent runs can't double-notify.
    let due: Vec<(i64, i32, String, Option<String>)> = match sqlx::query_as(
        "UPDATE forum_topics SET scheduled_at = NULL
         WHERE scheduled_at IS NOT NULL AND scheduled_at <= NOW()
           AND deleted_at IS NULL
         RETURNING id, author_id, title, topic_slug",
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("error: publish query failed: {err}");
            return ExitCode::FAILURE;
        }
    };

    let mut notified = 0;
    for (topic_id, author_id, title, slug) in &due {
        let link = format!("/forum/topic/{topic_id}");
        let display = match slug {
            Some(s) => format!("/forum/topic/{s}"),
            None => link.clone(),
        };
        let body = format!("Your scheduled topic \"{title}\" is now published.");
        if fichub::db::queries::social::create_notification(
            &pool,
            *author_id,
            "topic_published",
            "Scheduled topic published",
            Some(&body),
            Some(&display),
            Some("forum_topic"),
            Some(&topic_id.to_string()),
        )
        .await
        .is_ok()
        {
            notified += 1;
        }
    }

    println!(
        "publish-scheduled: {} topic(s) published, {} author(s) notified",
        due.len(),
        notified
    );
    ExitCode::SUCCESS
}
