use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Simple stdout logging for cron output
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: PgPool = PgPool::connect(&database_url).await?;

    let today = chrono::Utc::now().date_naive();

    sqlx::query(
        r#"INSERT INTO admin_daily_stats (date, total_users, new_users, total_works, new_works, manual_uploads, epubs_downloaded, words_read)
           VALUES ($1,
               (SELECT COUNT(*) FROM users),
               (SELECT COUNT(*) FROM users WHERE created_at::date = $1),
               (SELECT COUNT(*) FROM works),
               (SELECT COUNT(*) FROM works WHERE created_at::date = $1),
               (SELECT COUNT(*) FROM works w
                 LEFT JOIN fic_info fi ON fi.work_id = w.id
                 WHERE fi.source_type IN ('manual_epub','manual_text','import')
                   AND w.created_at::date = $1),
               (SELECT COALESCE(SUM(CASE WHEN etype = 'epub' THEN 1 ELSE 0 END), 0) FROM export_log WHERE created::date = $1),
               (SELECT COALESCE(SUM(words_read), 0) FROM reading_stats WHERE last_read_at::date = $1)
           )
           ON CONFLICT (date) DO UPDATE SET
               total_users = EXCLUDED.total_users,
               new_users = EXCLUDED.new_users,
               total_works = EXCLUDED.total_works,
               new_works = EXCLUDED.new_works,
               manual_uploads = EXCLUDED.manual_uploads,
               epubs_downloaded = EXCLUDED.epubs_downloaded,
               words_read = EXCLUDED.words_read"#
    )
    .bind(today)
    .execute(&pool)
    .await?;

    println!("Admin stats computed for {}", today);
    Ok(())
}
