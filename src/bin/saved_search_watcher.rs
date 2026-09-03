//! Nightly saved-search alert watcher.
//!
//! Re-runs every saved search with `alert_mode <> 'none'` (i.e. `rss`), diffs
//! the current match set against `saved_search_matches`, and inserts
//! newly-seen works (`ON CONFLICT DO NOTHING`). Updates each search's
//! `last_run_at` / `last_match_count`. New matches surface in the per-search
//! public Atom feed at `/feed/saved/{user_id}/{search_id}`.
//!
//! Run nightly (e.g. cron / systemd timer):
//!     DATABASE_URL=... ./saved_search_watcher

use fichub::db;
use fichub::search::routes::run_search_work_ids;
use sqlx::PgPool;
use std::env;

/// How many results to fetch per saved search run (capped at the DB query
/// scope). Alerts only care about distinct new work ids, not the full result
/// set, so a generous cap is fine.
const ALERT_PER_PAGE: usize = 500;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let tag_hidden_threshold = env::var("TAG_HIDDEN_THRESHOLD")
        .unwrap_or_else(|_| "-3".to_string())
        .parse()
        .unwrap_or(-3);

    let pool: PgPool = db::init_pool(&database_url).await.expect("Failed to connect to database");

    tracing::info!("Running saved-search alert watcher...");

    // Load all alerting saved searches (alert_mode <> 'none').
    let searches: Vec<(i64,)> = sqlx::query_as("SELECT id FROM saved_searches WHERE alert_mode <> 'none' AND query_text IS NOT NULL AND btrim(query_text) <> ''")
        .fetch_all(&pool)
        .await
        .expect("failed to load alerting saved searches");

    let mut total_new: usize = 0;
    let mut processed: usize = 0;

    for (search_id,) in searches {
        // Load the query text for this search.
        let query_text: Option<String> = sqlx::query_scalar(
            "SELECT query_text FROM saved_searches WHERE id = $1",
        )
        .bind(search_id)
        .fetch_optional(&pool)
        .await
        .expect("failed to load saved search query text");

        let Some(query_text) = query_text else {
            tracing::warn!(search_id, "saved search missing query text; skipping");
            continue;
        };

        // Re-run via the shared builder pipeline to get the current match set.
        let work_ids = match run_search_work_ids(&pool, &query_text, ALERT_PER_PAGE, tag_hidden_threshold).await {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!(search_id, %e, "saved search re-run failed; skipping");
                sqlx::query(
                    "UPDATE saved_searches SET last_run_at = now() WHERE id = $1",
                )
                .bind(search_id)
                .execute(&pool)
                .await
                .ok();
                continue;
            }
        };

        // Record this run's timestamps/count before diffing.
        sqlx::query(
            "UPDATE saved_searches SET last_run_at = now(), last_match_count = $1 WHERE id = $2",
        )
        .bind(work_ids.len() as i32)
        .bind(search_id)
        .execute(&pool)
        .await
        .expect("failed to update saved search run stats");

        // Insert newly-seen works. ON CONFLICT DO NOTHING keeps only genuine
        // new matches (the diff is handled by the PK search_id+work_id).
        if !work_ids.is_empty() {
            let new = sqlx::query(
                r#"INSERT INTO saved_search_matches (search_id, work_id)
                   SELECT $1, w FROM unnest($2::int[]) AS w
                   ON CONFLICT (search_id, work_id) DO NOTHING"#,
            )
            .bind(search_id)
            .bind(&work_ids)
            .execute(&pool)
            .await
            .map(|r| r.rows_affected() as usize)
            .unwrap_or(0);
            if new > 0 {
                tracing::info!(search_id, new, "recorded new matches");
            }
            total_new += new;
        }

        processed += 1;
    }

    tracing::info!(processed, total_new, "saved-search watcher complete");
    println!("OK: processed {} saved searches, {} new matches", processed, total_new);
}
