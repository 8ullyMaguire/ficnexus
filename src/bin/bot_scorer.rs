use sqlx::PgPool;
use std::env;

/// Hourly behavioral aggregator: rolls request_log up into bot_scores so the
/// admin dashboard can show per-IP/per-client behavior (requests, downloads,
/// failed auths, export:request ratio) without scanning the raw log on every
/// page load.
///
/// Zero-PII: bot_scores only stores anonymized aggregate signals keyed by IP
/// (needed for abuse detection) and client_id. The admin API exposes ONLY the
/// client_id + signals, never raw IPs.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: PgPool = PgPool::connect(&database_url).await?;

    // Aggregate the last 24h of request_log into hourly bot_scores windows.
    // etype values: 'epub' = download/export, 'search', 'auth', etc.
    sqlx::query(
        r#"
        INSERT INTO bot_scores (ip, client_id, window_start, requests, downloads, failed_auths, export_ratio)
        SELECT
            COALESCE(ip, '0.0.0.0'::inet) AS ip,
            COALESCE(client_id, 'anon') AS client_id,
            date_trunc('hour', created) AS window_start,
            COUNT(*) AS requests,
            COUNT(*) FILTER (WHERE etype = 'epub') AS downloads,
            COUNT(*) FILTER (WHERE etype = 'auth_failed') AS failed_auths,
            CASE WHEN COUNT(*) = 0 THEN 0
                 ELSE (COUNT(*) FILTER (WHERE etype = 'epub')::float / COUNT(*)::float)
            END AS export_ratio
        FROM request_log
        WHERE created >= now() - interval '24 hours'
        GROUP BY COALESCE(ip, '0.0.0.0'::inet), COALESCE(client_id, 'anon'), date_trunc('hour', created)
        ON CONFLICT (ip, client_id, window_start) DO UPDATE SET
            requests = EXCLUDED.requests,
            downloads = EXCLUDED.downloads,
            failed_auths = EXCLUDED.failed_auths,
            export_ratio = EXCLUDED.export_ratio
        "#,
    )
    .execute(&pool)
    .await?;

    // Retention: keep only the last 14 days of aggregates.
    sqlx::query("DELETE FROM bot_scores WHERE window_start < now() - interval '14 days'")
        .execute(&pool)
        .await?;

    // Request-log retention (NEXT.md item 9): raw request_log is only useful
    // for recent behavioral windows (bot-scorer consumes 24h). Prune > 30d to
    // keep the table bounded; historical trends come from bot_scores/daily
    // rollups instead.
    let pruned = sqlx::query("DELETE FROM request_log WHERE created < now() - interval '30 days'")
        .execute(&pool)
        .await?
        .rows_affected();

    println!("bot_scores aggregated for last 24h (hourly windows); pruned {pruned} request_log rows > 30d");
    Ok(())
}
