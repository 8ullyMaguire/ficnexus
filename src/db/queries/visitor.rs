use super::VisitorStateRow;
use crate::error::AppResult;
use sqlx::{PgPool, Row};

// ── Visitor state (anonymous funnel) ────────────────────────────────

/// Load visitor state by visitor ID. Returns None if not found.
pub async fn get_visitor_state(
    pool: &PgPool,
    visitor_id: uuid::Uuid,
) -> AppResult<Option<VisitorStateRow>> {
    let row = sqlx::query_as::<_, VisitorStateRow>(
        "SELECT visitor_id, ratings, saved_searches, bookmarks, created_at, updated_at
         FROM visitor_state WHERE visitor_id = $1",
    )
    .bind(visitor_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Upsert visitor state (insert or update on conflict).
pub async fn upsert_visitor_state(
    pool: &PgPool,
    visitor_id: uuid::Uuid,
    ratings: &serde_json::Value,
    saved_searches: &serde_json::Value,
    bookmarks: &serde_json::Value,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO visitor_state (visitor_id, ratings, saved_searches, bookmarks, updated_at)
         VALUES ($1, $2, $3, $4, now())
         ON CONFLICT (visitor_id) DO UPDATE SET
           ratings = EXCLUDED.ratings,
           saved_searches = EXCLUDED.saved_searches,
           bookmarks = EXCLUDED.bookmarks,
           updated_at = now()",
    )
    .bind(visitor_id)
    .bind(ratings)
    .bind(saved_searches)
    .bind(bookmarks)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete a visitor state row (after merge on registration).
pub async fn delete_visitor_state(pool: &PgPool, visitor_id: uuid::Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM visitor_state WHERE visitor_id = $1")
        .bind(visitor_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete visitor state rows older than 30 days (cron cleanup).
pub async fn cleanup_stale_visitor_state(pool: &PgPool) -> AppResult<u64> {
    let result =
        sqlx::query("DELETE FROM visitor_state WHERE updated_at < now() - interval '30 days'")
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// Merge anonymous visitor state into a newly-registered user account.
///
/// Called from the register handler after a user is successfully created.
/// Ratings, saved searches, and bookmarks from `visitor_state` are copied
/// into their user-specific tables, then the visitor row is deleted.
///
/// Returns a tuple of (merged_ratings, merged_searches, merged_bookmarks).
/// Errors are logged and swallowed so a merge failure never blocks registration.
pub async fn merge_visitor_state_into_user(
    pool: &PgPool,
    visitor_id: uuid::Uuid,
    user_id: i32,
) -> AppResult<(u64, u64, u64)> {
    let Some(row) = get_visitor_state(pool, visitor_id).await? else {
        return Ok((0, 0, 0));
    };

    let mut merged_ratings: u64 = 0;
    let mut merged_searches: u64 = 0;
    let mut merged_bookmarks: u64 = 0;

    // Merge ratings: visitor_state.ratings is a JSONB object {url_id: rating, ...}
    if let serde_json::Value::Object(ratings_map) = &row.ratings {
        for (url_id, rating_val) in ratings_map {
            let rating_i64 = rating_val.as_i64().unwrap_or(0);
            if !(1..=5).contains(&rating_i64) {
                continue;
            }
            let rating = rating_i64 as i16;

            // Resolve work_id from url_id via fic_info → works join.
            let work_id: Option<i32> =
                sqlx::query_scalar("SELECT w.id FROM works w JOIN fic_info fi ON fi.work_id = w.id WHERE fi.id = $1 LIMIT 1")
                    .bind(url_id)
                    .fetch_optional(pool)
                    .await?;

            if let Some(wid) = work_id {
                sqlx::query(
                    "INSERT INTO work_ratings (user_id, work_id, url_id, rating)
                     VALUES ($1, $2, $3, $4)
                     ON CONFLICT (user_id, url_id) DO UPDATE SET rating = $4, work_id = $2",
                )
                .bind(user_id)
                .bind(wid)
                .bind(url_id)
                .bind(rating)
                .execute(pool)
                .await?;
                merged_ratings += 1;
            }
        }
    }

    // Merge saved searches: visitor_state.saved_searches is a JSONB array
    // of objects with at least {name, query_text, query_json}.
    if let serde_json::Value::Array(searches) = &row.saved_searches {
        for item in searches {
            let obj = match item {
                serde_json::Value::Object(o) => o,
                _ => continue,
            };
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Imported search");
            let query_text = obj.get("query_text").and_then(|v| v.as_str()).unwrap_or("");
            let query_json = obj
                .get("query_json")
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            sqlx::query(
                "INSERT INTO saved_searches (user_id, name, query_text, query_json, alert_mode)
                 VALUES ($1, $2, $3, $4, 'none')
                 ON CONFLICT (user_id, name) DO NOTHING",
            )
            .bind(user_id)
            .bind(name)
            .bind(query_text)
            .bind(&query_json)
            .execute(pool)
            .await?;
            merged_searches += 1;
        }
    }

    // Merge bookmarks: visitor_state.bookmarks is a JSONB array of url_ids.
    if let serde_json::Value::Array(bookmarks) = &row.bookmarks {
        for item in bookmarks {
            let url_id = match item {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => continue,
            };

            // Resolve work_id from url_id.
            let work_id: Option<i32> =
                sqlx::query_scalar("SELECT w.id FROM works w JOIN fic_info fi ON fi.work_id = w.id WHERE fi.id = $1 LIMIT 1")
                    .bind(&url_id)
                    .fetch_optional(pool)
                    .await?;

            sqlx::query(
                "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
                 VALUES ($1, $2, $3, '', false)
                 ON CONFLICT (user_id, url_id) DO NOTHING",
            )
            .bind(user_id)
            .bind(&url_id)
            .bind(work_id)
            .execute(pool)
            .await?;
            merged_bookmarks += 1;
        }
    }

    // Clean up the visitor state row.
    delete_visitor_state(pool, visitor_id).await?;

    tracing::info!(
        visitor_id = %visitor_id,
        user_id,
        merged_ratings,
        merged_searches,
        merged_bookmarks,
        "visitor state merged into user account"
    );

    Ok((merged_ratings, merged_searches, merged_bookmarks))
}
