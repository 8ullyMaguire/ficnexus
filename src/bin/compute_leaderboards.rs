use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: PgPool = PgPool::connect(&database_url).await?;

    // Weekly leaderboard — based on reputation
    sqlx::query(
        r#"INSERT INTO leaderboard_weekly (user_id, score, rank, week_start)
           SELECT u.id, u.reputation, ROW_NUMBER() OVER (ORDER BY u.reputation DESC),
                  date_trunc('week', NOW())::date
           FROM users u WHERE u.reputation > 0
           ON CONFLICT (user_id, week_start) DO UPDATE SET
               score = EXCLUDED.score,
               rank = EXCLUDED.rank,
               computed_at = NOW()"#
    )
    .execute(&pool)
    .await?;

    // Monthly leaderboard
    sqlx::query(
        r#"INSERT INTO leaderboard_monthly (user_id, score, rank, month_start)
           SELECT u.id, u.reputation, ROW_NUMBER() OVER (ORDER BY u.reputation DESC),
                  date_trunc('month', NOW())::date
           FROM users u WHERE u.reputation > 0
           ON CONFLICT (user_id, month_start) DO UPDATE SET
               score = EXCLUDED.score,
               rank = EXCLUDED.rank,
               computed_at = NOW()"#
    )
    .execute(&pool)
    .await?;

    println!("Leaderboards recomputed");
    Ok(())
}
