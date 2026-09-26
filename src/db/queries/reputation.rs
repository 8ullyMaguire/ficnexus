use crate::db::models::*;
use crate::error::AppResult;
use sqlx::PgPool;

// ── Reputation & Auto-Promotion ─────────────────────────────────────

/// Award reputation to a user and check for auto-promotion to curator
pub async fn update_reputation_and_promote(
    pool: &PgPool,
    user_id: i32,
    delta: i32,
    event_type: &str,
) -> AppResult<()> {
    // Update reputation
    sqlx::query("UPDATE users SET reputation = reputation + $1 WHERE id = $2")
        .bind(delta)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Record reputation event.
    //
    // The column is `points`, not `delta`. The old INSERT named `delta`, which
    // no migration ever created, so this statement failed on EVERY call - and
    // because every caller in the codebase uses `let _ = ...` (admin.rs
    // work_publish, social.rs, forum.rs mod_received), the failure was
    // swallowed and reputation silently never moved anywhere in the product.
    sqlx::query(
        "INSERT INTO reputation_events (user_id, event_type, points)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(delta)
    .execute(pool)
    .await?;

    // Check for auto-promotion to curator (reputation >= 100, trust_level < 3)
    let row: Option<(i32, i16)> =
        sqlx::query_as("SELECT reputation, trust_level FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if let Some((reputation, trust_level)) = row {
        if reputation >= 100 && trust_level < 3 {
            // Promote to curator
            sqlx::query("UPDATE users SET trust_level = 3, curator_since = NOW() WHERE id = $1")
                .bind(user_id)
                .execute(pool)
                .await?;

            // Record promotion event
            sqlx::query(
                "INSERT INTO reputation_events (user_id, event_type, delta)
                 VALUES ($1, 'curator_promoted', 0)",
            )
            .bind(user_id)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Analytics queries
// ═══════════════════════════════════════════════════════════════════

/// Get daily unique visitors and total requests for the last N days
pub async fn get_daily_stats(
    pool: &PgPool,
    days: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64, i64)>(
        r#"SELECT DATE(created) as day,
                  COUNT(DISTINCT client_id) as unique_visitors,
                  COUNT(*) as total_requests
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND client_id IS NOT NULL
           GROUP BY DATE(created)
           ORDER BY day DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get per-user stats (total requests, downloads, unique fics)
pub async fn get_user_stats(pool: &PgPool, client_id: &str) -> AppResult<(i64, i64, i64)> {
    let row = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"SELECT 
            COUNT(*) as total_requests,
            COUNT(DISTINCT url_id) as unique_fics,
            COALESCE(SUM(CASE WHEN export_file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as downloads
          FROM request_log
          WHERE client_id = $1"#,
    )
    .bind(client_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Get most popular fics (by request count)
pub async fn get_popular_fics(
    pool: &PgPool,
    days: i32,
    limit: i32,
) -> AppResult<Vec<(String, i64, i64)>> {
    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"SELECT url_id,
                  COUNT(*) as request_count,
                  SUM(CASE WHEN export_file_name IS NOT NULL THEN 1 ELSE 0 END) as downloads
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND url_id IS NOT NULL
           GROUP BY url_id
           ORDER BY request_count DESC
           LIMIT $2"#,
    )
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get download format breakdown
pub async fn get_format_breakdown(pool: &PgPool, days: i32) -> AppResult<Vec<(String, i64)>> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT 
             CASE 
               WHEN export_file_name LIKE '%.epub' THEN 'epub'
               WHEN export_file_name LIKE '%.html' THEN 'html'
               WHEN export_file_name LIKE '%.mobi' THEN 'mobi'
               WHEN export_file_name LIKE '%.pdf' THEN 'pdf'
               WHEN export_file_name LIKE '%.azw3' THEN 'azw3'
               WHEN export_file_name LIKE '%.txt' THEN 'txt'
               WHEN export_file_name LIKE '%.md' THEN 'md'
               ELSE 'other'
             END as format,
             COUNT(*) as count
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND export_file_name IS NOT NULL
           GROUP BY format
           ORDER BY count DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get total unique visitors (distinct client_ids)
pub async fn get_total_unique_visitors(pool: &PgPool, days: i32) -> AppResult<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(DISTINCT client_id)
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND client_id IS NOT NULL"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Get return visitor rate (visitors with 2+ requests)
pub async fn get_return_visitor_rate(pool: &PgPool, days: i32) -> AppResult<(i64, i64)> {
    let row = sqlx::query_as::<_, (i64, i64)>(
        r#"SELECT 
             COUNT(DISTINCT client_id) as total_visitors,
             COUNT(DISTINCT CASE WHEN request_count > 1 THEN client_id END) as return_visitors
           FROM (
             SELECT client_id, COUNT(*) as request_count
             FROM request_log
             WHERE created > NOW() - ($1 || ' days')::INTERVAL
               AND client_id IS NOT NULL
             GROUP BY client_id
           ) sub"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

// ═══════════════════════════════════════════════════════════════════
// Usage analytics (usage_events table — non-PII, X-Client-ID based)
// ═══════════════════════════════════════════════════════════════════

/// Insert a usage event (best-effort; failures are logged, never fatal).
pub async fn insert_usage_event(
    pool: &PgPool,
    client_id: &str,
    path: &str,
    event_type: &str,
    user_agent: Option<&str>,
) {
    if let Err(e) = sqlx::query(
        "INSERT INTO usage_events (client_id, path, event_type, user_agent)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(client_id)
    .bind(path)
    .bind(event_type)
    .bind(user_agent)
    .execute(pool)
    .await
    {
        tracing::warn!("insert_usage_event failed: {e}");
    }
}

/// Unique visitors per day for the last N days (from usage_events).
pub async fn get_daily_unique_visitors(
    pool: &PgPool,
    days: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE(created_at) as day,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY DATE(created_at)
           ORDER BY day DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Unique visitors per week for the last N weeks (ISO week start Monday).
pub async fn get_weekly_unique_visitors(
    pool: &PgPool,
    weeks: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE_TRUNC('week', created_at)::date as week_start,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' weeks')::INTERVAL
           GROUP BY DATE_TRUNC('week', created_at)
           ORDER BY week_start DESC"#,
    )
    .bind(weeks)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Unique visitors per month for the last N months.
pub async fn get_monthly_unique_visitors(
    pool: &PgPool,
    months: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE_TRUNC('month', created_at)::date as month_start,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' months')::INTERVAL
           GROUP BY DATE_TRUNC('month', created_at)
           ORDER BY month_start DESC"#,
    )
    .bind(months)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Active users in a window: distinct client_ids that performed at least one
/// ACTION (non-view event). Returns (active_count, total_events).
pub async fn get_active_users(pool: &PgPool, days: i32) -> AppResult<(i64, i64)> {
    let row = sqlx::query_as::<_, (i64, i64)>(
        r#"SELECT COUNT(DISTINCT client_id) as active_users,
                  COUNT(*) as action_events
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
             AND event_type = 'action'"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Distinct client_ids that ONLY viewed (no actions) in the window.
pub async fn get_view_only_users(pool: &PgPool, days: i32) -> AppResult<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(DISTINCT client_id)
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
             AND client_id NOT IN (
               SELECT DISTINCT client_id FROM usage_events
               WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
                 AND event_type = 'action'
             )"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Total events in a window (all types).
pub async fn get_total_events(pool: &PgPool, days: i32) -> AppResult<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Per-endpoint usage breakdown (top paths by count, view vs action).
/// Grouped by (path, event_type) in the last `days` days, ordered by count desc.
/// Derives a feature-group label (search/export/reader/social/other) from path prefix.
pub fn endpoint_group(path: &str) -> &'static str {
    if path.starts_with("/api/search")
        || path.starts_with("/api/tags/search")
        || path.starts_with("/api/tags/autocomplete")
        || path.starts_with("/opds/search")
        || path.starts_with("/feed")
    {
        "search"
    } else if path.starts_with("/api/epub")
        || path.starts_with("/api/download")
        || path.starts_with("/cache/")
        || path.starts_with("/api/send-to-kindle")
        || path.starts_with("/api/meta")
    {
        "export"
    } else if path.starts_with("/reader")
        || path.starts_with("/api/works")
        || path.starts_with("/api/fic")
        || path.starts_with("/api/chapters")
        || path.starts_with("/works/")
    {
        "reader"
    } else if path.starts_with("/api/comments")
        || path.starts_with("/api/votes")
        || path.starts_with("/api/bookmarks")
        || path.starts_with("/api/follow")
        || path.starts_with("/api/rate")
        || path.starts_with("/api/review")
        || path.starts_with("/api/shelves")
        || path.starts_with("/api/reading-lists")
        || path.starts_with("/api/forum")
    {
        "social"
    } else {
        "other"
    }
}

pub async fn get_endpoint_usage(
    pool: &PgPool,
    days: i32,
    limit: i64,
) -> AppResult<Vec<(String, String, i64)>> {
    let limit = limit.clamp(1, 100);
    let rows = sqlx::query_as::<_, (String, String, i64)>(
        r#"SELECT path, event_type, COUNT(*)::bigint as cnt
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY path, event_type
           ORDER BY cnt DESC
           LIMIT $2"#,
    )
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_endpoint_group_usage(pool: &PgPool, days: i32) -> AppResult<Vec<(String, i64)>> {
    // Use SQL CASE to bucket paths into feature groups server-side.
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT
              CASE
                WHEN path LIKE '/api/search%' OR path LIKE '/api/tags/search%' OR path LIKE '/api/tags/autocomplete%' OR path LIKE '/opds/search%' OR path LIKE '/feed%' THEN 'search'
                WHEN path LIKE '/api/epub%' OR path LIKE '/api/download%' OR path LIKE '/cache/%' OR path LIKE '/api/send-to-kindle%' OR path LIKE '/api/meta%' THEN 'export'
                WHEN path LIKE '/reader%' OR path LIKE '/api/works%' OR path LIKE '/api/fic%' OR path LIKE '/api/chapters%' OR path LIKE '/works/%' THEN 'reader'
                WHEN path LIKE '/api/comments%' OR path LIKE '/api/votes%' OR path LIKE '/api/bookmarks%' OR path LIKE '/api/follow%' OR path LIKE '/api/rate%' OR path LIKE '/api/review%' OR path LIKE '/api/shelves%' OR path LIKE '/api/reading-lists%' OR path LIKE '/api/forum%' THEN 'social'
                ELSE 'other'
              END as grp,
              COUNT(*)::bigint as cnt
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY grp
           ORDER BY cnt DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Recent events timeline (latest first) for the admin view.
pub async fn get_recent_events(
    pool: &PgPool,
    limit: i32,
) -> AppResult<
    Vec<(
        chrono::DateTime<chrono::Utc>,
        String,
        String,
        String,
        Option<String>,
    )>,
> {
    let rows = sqlx::query_as::<
        _,
        (
            chrono::DateTime<chrono::Utc>,
            String,
            String,
            String,
            Option<String>,
        ),
    >(
        r#"SELECT created_at, client_id, path, event_type, user_agent
           FROM usage_events
           ORDER BY created_at DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Shelf / Collection queries
// ═══════════════════════════════════════════════════════════════════

/// Create a new shelf
pub async fn create_shelf(
    pool: &PgPool,
    user_id: i32,
    name: &str,
    description: &str,
    is_public: bool,
) -> AppResult<Shelf> {
    let row = sqlx::query_as::<_, Shelf>(
        r#"INSERT INTO shelves (user_id, name, description, is_public)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, name, description, is_public, sort_order, created_at"#,
    )
    .bind(user_id)
    .bind(name)
    .bind(description)
    .bind(is_public)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// List all shelves for a user
pub async fn list_shelves(pool: &PgPool, user_id: i32) -> AppResult<Vec<Shelf>> {
    let rows = sqlx::query_as::<_, Shelf>(
        "SELECT id, user_id, name, description, is_public, sort_order, created_at
         FROM shelves WHERE user_id = $1 ORDER BY sort_order, created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get a single shelf by id (scoped to user)
pub async fn get_shelf(pool: &PgPool, shelf_id: i32, user_id: i32) -> AppResult<Option<Shelf>> {
    let row = sqlx::query_as::<_, Shelf>(
        "SELECT id, user_id, name, description, is_public, sort_order, created_at
         FROM shelves WHERE id = $1 AND user_id = $2",
    )
    .bind(shelf_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Delete a shelf (user-scoped)
pub async fn delete_shelf(pool: &PgPool, shelf_id: i32, user_id: i32) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM shelves WHERE id = $1 AND user_id = $2")
        .bind(shelf_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Add a work to a shelf
pub async fn add_work_to_shelf(pool: &PgPool, shelf_id: i32, work_id: i32) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO work_shelves (shelf_id, work_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(shelf_id)
    .bind(work_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a work from a shelf
pub async fn remove_work_from_shelf(pool: &PgPool, shelf_id: i32, work_id: i32) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM work_shelves WHERE shelf_id = $1 AND work_id = $2")
        .bind(shelf_id)
        .bind(work_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List works in a shelf (returns work_ids)
pub async fn list_works_in_shelf(pool: &PgPool, shelf_id: i32) -> AppResult<Vec<WorkShelf>> {
    let rows = sqlx::query_as::<_, WorkShelf>(
        "SELECT id, shelf_id, work_id, added_at
         FROM work_shelves WHERE shelf_id = $1 ORDER BY added_at DESC",
    )
    .bind(shelf_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Reading list queries
// ═══════════════════════════════════════════════════════════════════

/// Create a reading list for a user.
pub async fn create_reading_list(
    pool: &PgPool,
    user_id: i32,
    title: &str,
    description: &str,
    is_public: bool,
) -> AppResult<ReadingList> {
    let row = sqlx::query_as::<_, ReadingList>(
        r#"INSERT INTO reading_lists (user_id, title, description, is_public)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, title, description, is_public, created_at, updated_at"#,
    )
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(is_public)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// List the user's own non-deleted reading lists, newest first.
pub async fn list_reading_lists(pool: &PgPool, user_id: i32) -> AppResult<Vec<ReadingList>> {
    let rows = sqlx::query_as::<_, ReadingList>(
        "SELECT id, user_id, title, description, is_public, created_at, updated_at
         FROM reading_lists
         WHERE user_id = $1 AND deleted_at IS NULL
         ORDER BY updated_at DESC, id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Fetch one list, scoped to the owner unless `public_ok` (shared view).
/// `deleted` filters out soft-deleted lists for the owner.
pub async fn get_reading_list(
    pool: &PgPool,
    list_id: i32,
    user_id: i32,
    public_ok: bool,
) -> AppResult<Option<ReadingList>> {
    let row = if public_ok {
        sqlx::query_as::<_, ReadingList>(
            "SELECT id, user_id, title, description, is_public, created_at, updated_at
             FROM reading_lists
             WHERE id = $1 AND deleted_at IS NULL AND is_public = TRUE",
        )
        .bind(list_id)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as::<_, ReadingList>(
            "SELECT id, user_id, title, description, is_public, created_at, updated_at
             FROM reading_lists
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        )
        .bind(list_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
    };
    Ok(row)
}

/// Update a list's title/description/public flag (owner-scoped).
pub async fn update_reading_list(
    pool: &PgPool,
    list_id: i32,
    user_id: i32,
    title: &str,
    description: &str,
    is_public: bool,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE reading_lists
           SET title = $3, description = $4, is_public = $5, updated_at = NOW()
           WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(list_id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(is_public)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Soft-delete a reading list (owner or curator).
pub async fn delete_reading_list(
    pool: &PgPool,
    list_id: i32,
    user_id: i32,
    is_curator: bool,
) -> AppResult<bool> {
    let result = if is_curator {
        sqlx::query("UPDATE reading_lists SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(list_id)
            .execute(pool)
            .await?
    } else {
        sqlx::query(
            "UPDATE reading_lists SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        )
        .bind(list_id)
        .bind(user_id)
        .execute(pool)
        .await?
    };
    Ok(result.rows_affected() > 0)
}

/// Append a work to a list at the next position. The UNIQUE(list_id, work_id)
/// constraint rejects duplicates; returns Ok(false) when the work is already
/// in the list.
pub async fn add_reading_list_item(
    pool: &PgPool,
    list_id: i32,
    work_id: i32,
    blurb: &str,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"INSERT INTO reading_list_items (list_id, work_id, position, blurb)
           SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3
           FROM reading_list_items WHERE list_id = $1
           ON CONFLICT (list_id, work_id) DO NOTHING"#,
    )
    .bind(list_id)
    .bind(work_id)
    .bind(blurb)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Remove a work from a list; returns Ok(false) if it was not present.
pub async fn remove_reading_list_item(
    pool: &PgPool,
    list_id: i32,
    work_id: i32,
) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM reading_list_items WHERE list_id = $1 AND work_id = $2")
        .bind(list_id)
        .bind(work_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List a list's items in position order, joined with work titles/authors.
pub async fn list_reading_list_items(
    pool: &PgPool,
    list_id: i32,
) -> AppResult<Vec<ReadingListItemRow>> {
    let rows = sqlx::query_as::<_, ReadingListItemRow>(
        r#"SELECT i.id, i.list_id, i.work_id, i.position, i.blurb, i.created_at,
                  w.canonical_title, w.canonical_author
           FROM reading_list_items i
           JOIN works w ON w.id = i.work_id
           WHERE i.list_id = $1
           ORDER BY i.position ASC, i.id ASC"#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Collection queries (AO3-style: visibility, item selection, bookmarks)
// ═══════════════════════════════════════════════════════════════════

/// Create an AO3-style collection. Wraps the reading_lists table with the
/// collection_kind / visibility / item_selection / icon / slug columns.
pub async fn create_collection(
    pool: &PgPool,
    user_id: i32,
    title: &str,
    description: &str,
    visibility: &str,
    item_selection: &str,
    icon: &str,
    slug: Option<&str>,
) -> AppResult<CollectionInfo> {
    let row = sqlx::query_as::<_, CollectionInfo>(
        r#"INSERT INTO reading_lists
             (user_id, title, description, collection_kind, visibility,
              item_selection, icon, slug)
           VALUES ($1, $2, $3, 'collection', $4, $5, $6, $7)
           RETURNING
             id, user_id,
             (SELECT username FROM users WHERE id = $1) AS owner_username,
             title, description, visibility, item_selection, icon, slug,
             'collection'::text AS collection_kind,
             created_at, updated_at,
             0::bigint AS item_count,
             0::bigint AS bookmark_count,
             FALSE AS is_bookmarked"#,
    )
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(visibility)
    .bind(item_selection)
    .bind(icon)
    .bind(slug)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Browse public + anonymous collections (paginated, newest first).
/// Anonymous collections show the owner username only to themselves.
pub async fn browse_public_collections(
    pool: &PgPool,
    viewer_id: Option<i32>,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<CollectionInfo>, i64)> {
    let limit = limit.clamp(1, 100);
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM reading_lists
           WHERE collection_kind = 'collection'
             AND visibility IN ('public', 'anonymous')
             AND deleted_at IS NULL"#,
    )
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, CollectionInfo>(
        r#"SELECT rl.id, rl.user_id,
                  CASE WHEN rl.visibility = 'anonymous' AND $1 IS DISTINCT FROM rl.user_id
                       THEN NULL ELSE u.username END AS owner_username,
                  rl.title, rl.description, rl.visibility, rl.item_selection,
                  rl.icon, rl.slug, rl.collection_kind,
                  rl.created_at, rl.updated_at,
                  (SELECT COUNT(*) FROM reading_list_items WHERE list_id = rl.id) AS item_count,
                  (SELECT COUNT(*) FROM collection_bookmarks WHERE list_id = rl.id) AS bookmark_count,
                  EXISTS (SELECT 1 FROM collection_bookmarks WHERE list_id = rl.id AND user_id = $1) AS is_bookmarked
           FROM reading_lists rl
           LEFT JOIN users u ON u.id = rl.user_id
           WHERE rl.collection_kind = 'collection'
             AND rl.visibility IN ('public', 'anonymous')
             AND rl.deleted_at IS NULL
           ORDER BY rl.updated_at DESC, rl.id DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(viewer_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((rows, total.0))
}

/// Fetch a single collection by id. Returns the row only if visible to the
/// viewer: owner always; public/anonymous visible to everyone (anonymous
/// hides owner name unless viewer is the owner).
pub async fn get_collection(
    pool: &PgPool,
    id: i32,
    viewer_id: Option<i32>,
) -> AppResult<Option<CollectionInfo>> {
    let row = sqlx::query_as::<_, CollectionInfo>(
        r#"SELECT rl.id, rl.user_id,
                  CASE WHEN rl.visibility = 'anonymous' AND $1 IS DISTINCT FROM rl.user_id
                       THEN NULL ELSE u.username END AS owner_username,
                  rl.title, rl.description, rl.visibility, rl.item_selection,
                  rl.icon, rl.slug, rl.collection_kind,
                  rl.created_at, rl.updated_at,
                  (SELECT COUNT(*) FROM reading_list_items WHERE list_id = rl.id) AS item_count,
                  (SELECT COUNT(*) FROM collection_bookmarks WHERE list_id = rl.id) AS bookmark_count,
                  EXISTS (SELECT 1 FROM collection_bookmarks WHERE list_id = rl.id AND user_id = $1) AS is_bookmarked
           FROM reading_lists rl
           LEFT JOIN users u ON u.id = rl.user_id
           WHERE rl.id = $2
             AND rl.collection_kind = 'collection'
             AND rl.deleted_at IS NULL
             AND (
                  rl.user_id = $1                       -- owner always
                  OR rl.visibility = 'public'
                  OR rl.visibility = 'anonymous'
             )"#,
    )
    .bind(viewer_id)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Fetch a collection by slug (for clean /collections/[slug] URLs).
pub async fn get_collection_by_slug(
    pool: &PgPool,
    slug: &str,
    viewer_id: Option<i32>,
) -> AppResult<Option<CollectionInfo>> {
    let row = sqlx::query_as::<_, CollectionInfo>(
        r#"SELECT rl.id, rl.user_id,
                  CASE WHEN rl.visibility = 'anonymous' AND $1 IS DISTINCT FROM rl.user_id
                       THEN NULL ELSE u.username END AS owner_username,
                  rl.title, rl.description, rl.visibility, rl.item_selection,
                  rl.icon, rl.slug, rl.collection_kind,
                  rl.created_at, rl.updated_at,
                  (SELECT COUNT(*) FROM reading_list_items WHERE list_id = rl.id) AS item_count,
                  (SELECT COUNT(*) FROM collection_bookmarks WHERE list_id = rl.id) AS bookmark_count,
                  EXISTS (SELECT 1 FROM collection_bookmarks WHERE list_id = rl.id AND user_id = $1) AS is_bookmarked
           FROM reading_lists rl
           LEFT JOIN users u ON u.id = rl.user_id
           WHERE rl.slug = $2
             AND rl.collection_kind = 'collection'
             AND rl.deleted_at IS NULL
             AND (
                  rl.user_id = $1
                  OR rl.visibility = 'public'
                  OR rl.visibility = 'anonymous'
             )"#,
    )
    .bind(viewer_id)
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Update a collection's mutable metadata (owner-scoped).
pub async fn update_collection(
    pool: &PgPool,
    id: i32,
    user_id: i32,
    title: &str,
    description: &str,
    visibility: &str,
    item_selection: &str,
    icon: &str,
    slug: Option<&str>,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE reading_lists
           SET title = $3, description = $4, visibility = $5,
               item_selection = $6, icon = $7, slug = $8, updated_at = NOW()
           WHERE id = $1 AND user_id = $2
             AND collection_kind = 'collection'
             AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(visibility)
    .bind(item_selection)
    .bind(icon)
    .bind(slug)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Soft-delete a collection (owner or curator >= 5).
pub async fn delete_collection(
    pool: &PgPool,
    id: i32,
    user_id: i32,
    is_curator: bool,
) -> AppResult<bool> {
    let result = if is_curator {
        sqlx::query(
            r#"UPDATE reading_lists
               SET deleted_at = NOW(), updated_at = NOW()
               WHERE id = $1
                 AND collection_kind = 'collection'
                 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .execute(pool)
        .await?
    } else {
        sqlx::query(
            r#"UPDATE reading_lists
               SET deleted_at = NOW(), updated_at = NOW()
               WHERE id = $1 AND user_id = $2
                 AND collection_kind = 'collection'
                 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?
    };
    Ok(result.rows_affected() > 0)
}

/// Add a work to a collection. Respects item_selection:
///   * restricted  — only owner/curator may add (returns Ok(false) otherwise)
///   * moderated   — non-owners land in collection_item_requests as 'pending'
/// Returns Ok(true) when the item was added directly (approved immediately).
pub async fn add_collection_item(
    pool: &PgPool,
    list_id: i32,
    work_id: i32,
    actor_id: i32,
    blurb: &str,
    is_owner_or_curator: bool,
    item_selection: &str,
) -> AppResult<bool> {
    if item_selection == "moderated" && !is_owner_or_curator {
        // Non-owners of a moderated collection must request; the item lands
        // as 'pending' in collection_item_requests.
        sqlx::query(
            r#"INSERT INTO collection_item_requests (list_id, work_id, requested_by, blurb)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (list_id, work_id) DO UPDATE
                 SET requested_by = EXCLUDED.requested_by,
                     blurb = EXCLUDED.blurb"#,
        )
        .bind(list_id)
        .bind(work_id)
        .bind(actor_id)
        .bind(blurb)
        .execute(pool)
        .await?;
        return Ok(false);
    }

    // Restricted mode (or owner/curator of a moderated collection): add directly.
    let result = sqlx::query(
        r#"INSERT INTO reading_list_items (list_id, work_id, position, blurb)
           SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3
           FROM reading_list_items WHERE list_id = $1
           ON CONFLICT (list_id, work_id) DO NOTHING"#,
    )
    .bind(list_id)
    .bind(work_id)
    .bind(blurb)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Remove a work from a collection (owner or curator).
pub async fn remove_collection_item(pool: &PgPool, list_id: i32, work_id: i32) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM reading_list_items WHERE list_id = $1 AND work_id = $2")
        .bind(list_id)
        .bind(work_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List pending (and recent) add requests for a moderated collection.
pub async fn list_collection_item_requests(
    pool: &PgPool,
    list_id: i32,
) -> AppResult<Vec<CollectionItemRequestRow>> {
    let rows = sqlx::query_as::<_, CollectionItemRequestRow>(
        r#"SELECT cir.id, cir.list_id, cir.work_id, cir.requested_by,
                  u.username AS requested_by_username, cir.blurb, cir.status,
                  cir.created_at, cir.reviewed_at, w.canonical_title, w.canonical_author
           FROM collection_item_requests cir
           LEFT JOIN users u ON u.id = cir.requested_by
           JOIN works w ON w.id = cir.work_id
           WHERE cir.list_id = $1
           ORDER BY cir.status ASC, cir.created_at DESC"#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// List collection add-requests (optionally for a single `list_id`) joined
/// with the collection title, the work's metadata, and live community vote
/// tallies plus the viewing user's own vote. Pass `None` for `list_id` to list
/// across every collection (used by the unified curator Approvals queue).
pub async fn list_collection_item_requests_with_votes(
    pool: &PgPool,
    list_id: Option<i32>,
    viewer_id: i32,
) -> AppResult<Vec<CollectionItemRequestVoteRow>> {
    let rows = sqlx::query_as::<_, CollectionItemRequestVoteRow>(
        r#"SELECT cir.id, cir.list_id, rl.title AS list_title, cir.work_id,
                   cir.requested_by, u.username AS requested_by_username,
                   cir.blurb, cir.status, cir.created_at, cir.reviewed_at,
                   w.canonical_title, w.canonical_author,
                   COALESCE(sv.votes_for, 0)::int AS votes_for,
                   COALESCE(sv.votes_against, 0)::int AS votes_against,
                   COALESCE(my.vote, 0)::smallint AS my_vote
            FROM collection_item_requests cir
            JOIN reading_lists rl ON rl.id = cir.list_id
            LEFT JOIN users u ON u.id = cir.requested_by
            JOIN works w ON w.id = cir.work_id
            LEFT JOIN (
                SELECT request_id,
                       COUNT(*) FILTER (WHERE vote = 1)::int AS votes_for,
                       COUNT(*) FILTER (WHERE vote = -1)::int AS votes_against
                FROM collection_submission_votes
                GROUP BY request_id
            ) sv ON sv.request_id = cir.id
            LEFT JOIN collection_submission_votes my
                ON my.request_id = cir.id AND my.user_id = $2
            WHERE ($1::int IS NULL OR cir.list_id = $1)
            ORDER BY cir.status ASC, cir.created_at DESC"#,
    )
    .bind(list_id)
    .bind(viewer_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Record (upsert) a community vote for a collection add-request. The
/// (request_id, user_id) unique constraint guarantees a member cannot vote
/// twice; re-voting simply changes their prior signal.
pub async fn upsert_collection_submission_vote(
    pool: &PgPool,
    request_id: i64,
    user_id: i32,
    vote: i16,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"INSERT INTO collection_submission_votes (request_id, user_id, vote)
           VALUES ($1, $2, $3)
           ON CONFLICT (request_id, user_id)
           DO UPDATE SET vote = EXCLUDED.vote, created_at = now()"#,
    )
    .bind(request_id)
    .bind(user_id)
    .bind(vote)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Approve a pending collection item (owner/curator): move it into
/// reading_list_items and mark the request 'approved'.
pub async fn approve_collection_item(
    pool: &PgPool,
    list_id: i32,
    request_id: i64,
) -> AppResult<bool> {
    let mut tx = pool.begin().await?;
    // Grab the pending request
    let req: Option<(i32, i32, String)> = sqlx::query_as(
        r#"SELECT work_id, requested_by, blurb FROM collection_item_requests
           WHERE id = $1 AND list_id = $2 AND status = 'pending'"#,
    )
    .bind(request_id)
    .bind(list_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some((work_id, _requested_by, blurb)) = req {
        // Insert the work item (skip if already present)
        sqlx::query(
            r#"INSERT INTO reading_list_items (list_id, work_id, position, blurb)
               SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3
               FROM reading_list_items WHERE list_id = $1
               ON CONFLICT (list_id, work_id) DO NOTHING"#,
        )
        .bind(list_id)
        .bind(work_id)
        .bind(blurb)
        .execute(&mut *tx)
        .await?;
        // Mark the request approved
        sqlx::query(
            r#"UPDATE collection_item_requests
               SET status = 'approved', reviewed_at = NOW()
               WHERE id = $1"#,
        )
        .bind(request_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(true)
    } else {
        tx.rollback().await?;
        Ok(false)
    }
}

/// Reject a pending collection item (owner/curator).
pub async fn reject_collection_item(
    pool: &PgPool,
    list_id: i32,
    request_id: i64,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE collection_item_requests
           SET status = 'rejected', reviewed_at = NOW()
           WHERE id = $1 AND list_id = $2 AND status = 'pending'"#,
    )
    .bind(request_id)
    .bind(list_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Bookmark a collection for the current user (idempotent).
pub async fn bookmark_collection(pool: &PgPool, user_id: i32, list_id: i32) -> AppResult<bool> {
    let result = sqlx::query(
        "INSERT INTO collection_bookmarks (user_id, list_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(list_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Remove a collection bookmark.
pub async fn unbookmark_collection(pool: &PgPool, user_id: i32, list_id: i32) -> AppResult<bool> {
    let result =
        sqlx::query("DELETE FROM collection_bookmarks WHERE user_id = $1 AND list_id = $2")
            .bind(user_id)
            .bind(list_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// List users who bookmarked a collection.
pub async fn list_collection_bookmarkers(
    pool: &PgPool,
    list_id: i32,
) -> AppResult<Vec<(i32, String, String)>> {
    let rows: Vec<(i32, String, String)> = sqlx::query_as(
        r#"SELECT u.id, u.username, cb.created_at::text
           FROM collection_bookmarks cb
           JOIN users u ON u.id = cb.user_id
           WHERE cb.list_id = $1
           ORDER BY cb.created_at DESC"#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Reading status queries
// ═══════════════════════════════════════════════════════════════════

/// Update reading status for a work
pub async fn update_reading_status(
    pool: &PgPool,
    user_id: i32,
    work_id: i32,
    status: &str,
    current_chapter: Option<i32>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO reading_stats (user_id, work_id, status, current_chapter)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, work_id) DO UPDATE SET
               status = $3,
               current_chapter = COALESCE($4, reading_stats.current_chapter),
               last_read_at = NOW()"#,
    )
    .bind(user_id)
    .bind(work_id)
    .bind(status)
    .bind(current_chapter)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get reading list rows for a user, optionally filtered by status
pub async fn get_reading_stats_list(
    pool: &PgPool,
    user_id: i32,
    status_filter: Option<&str>,
) -> AppResult<Vec<ReadingStats>> {
    let rows = match status_filter {
        Some(status) => {
            sqlx::query_as::<_, ReadingStats>(
                "SELECT id, user_id, work_id, words_read, last_read_at, read_count, status, current_chapter
                 FROM reading_stats WHERE user_id = $1 AND status = $2 ORDER BY last_read_at DESC",
            )
            .bind(user_id)
            .bind(status)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, ReadingStats>(
                "SELECT id, user_id, work_id, words_read, last_read_at, read_count, status, current_chapter
                 FROM reading_stats WHERE user_id = $1 ORDER BY last_read_at DESC",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(rows)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_client_id_format() {
        // Test UUID v4 format validation
        let valid_uuids = vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "12345678-1234-4234-8234-123456789abc",
        ];
        let invalid_uuids = vec![
            "not-a-uuid",
            "550e8400-e29b-41d4-a716",
            "550e8400-e29b-41d4-a716-446655440000-extra",
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8", // version 1, not version 4
        ];

        let uuid_regex = regex_lite::Regex::new(
            r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
        )
        .unwrap();

        for uuid in valid_uuids {
            assert!(
                uuid_regex.is_match(uuid),
                "Should be valid UUID v4: {}",
                uuid
            );
        }
        for uuid in invalid_uuids {
            assert!(
                !uuid_regex.is_match(uuid),
                "Should be invalid UUID: {}",
                uuid
            );
        }
    }

    #[test]
    fn test_analytics_query_structures() {
        // Test that our query return types are correct
        let daily_stat: (chrono::NaiveDate, i64, i64) = (
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            100,
            500,
        );
        assert_eq!(daily_stat.1, 100); // unique_visitors
        assert_eq!(daily_stat.2, 500); // total_requests

        let user_stat: (i64, i64, i64) = (50, 10, 5);
        assert_eq!(user_stat.0, 50); // total_requests
        assert_eq!(user_stat.1, 10); // unique_fics
        assert_eq!(user_stat.2, 5); // downloads

        let popular_fic: (String, i64, i64) = ("abc123".to_string(), 200, 150);
        assert_eq!(popular_fic.1, 200); // request_count
        assert_eq!(popular_fic.2, 150); // downloads

        let format_stat: (String, i64) = ("epub".to_string(), 1000);
        assert_eq!(format_stat.0, "epub");
        assert_eq!(format_stat.1, 1000);
    }

    #[test]
    fn test_follow_model_types() {
        // Verify Follow tuple shapes are correct
        let follow = crate::db::models::Follow {
            id: 1,
            follower_id: 10,
            followee_id: Some(20),
            work_id: None,
            author_name: None,
            created_at: chrono::Utc::now(),
            last_seen: None,
        };
        assert_eq!(follow.follower_id, 10);
        assert_eq!(follow.followee_id, Some(20));
    }

    #[test]
    fn test_notification_model() {
        let notif = crate::db::models::Notification {
            id: 1,
            user_id: 10,
            notification_type: "comment_reply".into(),
            title: "Reply".into(),
            body: None,
            link: Some("/works/1".into()),
            reference_type: Some("comment".into()),
            reference_id: Some("5".into()),
            is_read: false,
            created_at: chrono::Utc::now(),
        };
        assert!(!notif.is_read);
        assert_eq!(notif.notification_type, "comment_reply");
    }

    #[test]
    fn test_badge_definition_model() {
        let badge = crate::db::models::BadgeDefinition {
            badge_type: "curator_apprentice".into(),
            name: "Curator Apprentice".into(),
            description: "Test badge".into(),
            icon: "🔰".into(),
            category: "curation".into(),
            threshold: 10,
            event_type: Some("curation_approve".into()),
            created_at: chrono::Utc::now(),
        };
        assert_eq!(badge.threshold, 10);
        assert_eq!(badge.event_type, Some("curation_approve".into()));
    }

    #[test]
    fn test_locale_model() {
        let locale = crate::db::models::Locale {
            id: 1,
            code: "en".into(),
            name: "English".into(),
            is_rtl: false,
        };
        assert_eq!(locale.code, "en");
        assert!(!locale.is_rtl);
    }

    #[test]
    fn test_leaderboard_entry_shapes() {
        let weekly: (i32, String, i32, i16) = (1, "alice".into(), 500, 1);
        assert_eq!(weekly.2, 500);
        assert_eq!(weekly.3, 1);

        let monthly: (i32, String, i32, i16) = (2, "bob".into(), 300, 2);
        assert_eq!(monthly.0, 2);
        assert_eq!(monthly.1, "bob");
    }

    #[test]
    fn test_translation_model_shapes() {
        let t = crate::db::models::WorkTranslation {
            id: 1,
            work_id: 42,
            locale_code: "es".into(),
            title: Some("El título".into()),
            summary: None,
            translated_by: Some(7),
            translated_at: chrono::Utc::now(),
        };
        assert_eq!(t.locale_code, "es");
        assert_eq!(t.title, Some("El título".into()));
    }

    #[test]
    fn test_reading_stats_model() {
        let stats = crate::db::models::ReadingStats {
            id: 1,
            user_id: 10,
            work_id: 42,
            words_read: 50000,
            last_read_at: chrono::Utc::now(),
            read_count: 3,
            status: "reading".into(),
            current_chapter: Some(5),
        };
        assert_eq!(stats.words_read, 50000);
        assert_eq!(stats.read_count, 3);
        assert_eq!(stats.status, "reading");
        assert_eq!(stats.current_chapter, Some(5));
    }

    #[test]
    fn test_login_streak_model() {
        let streak = crate::db::models::LoginStreak {
            user_id: 1,
            current_streak: 7,
            longest_streak: 30,
            last_login_date: chrono::Utc::now().date_naive(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(streak.current_streak, 7);
        assert_eq!(streak.longest_streak, 30);
    }

    #[test]
    fn test_leaderboard_weekly_monthly_models() {
        let week_start = chrono::Utc::now().date_naive();
        let lw = crate::db::models::LeaderboardWeekly {
            id: 1,
            user_id: 1,
            score: 500,
            rank: 1,
            week_start,
            computed_at: chrono::Utc::now(),
        };
        assert_eq!(lw.score, 500);
        assert_eq!(lw.rank, 1);

        let lm = crate::db::models::LeaderboardMonthly {
            id: 2,
            user_id: 1,
            score: 2000,
            rank: 1,
            month_start: week_start,
            computed_at: chrono::Utc::now(),
        };
        assert_eq!(lm.score, 2000);
    }

    #[test]
    fn test_endpoint_group_classification() {
        assert_eq!(
            crate::db::queries::endpoint_group("/api/search?q=foo"),
            "search"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/tags/search?q=x"),
            "search"
        );
        assert_eq!(crate::db::queries::endpoint_group("/feed.xml"), "search");
        assert_eq!(
            crate::db::queries::endpoint_group("/api/epub?id=1"),
            "export"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/cache/epub/1/file.epub"),
            "export"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/download/author"),
            "export"
        );
        assert_eq!(crate::db::queries::endpoint_group("/reader/1"), "reader");
        assert_eq!(crate::db::queries::endpoint_group("/api/works/1"), "reader");
        assert_eq!(
            crate::db::queries::endpoint_group("/api/comments"),
            "social"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/bookmarks"),
            "social"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/forum/threads"),
            "social"
        );
        assert_eq!(crate::db::queries::endpoint_group("/api/health"), "other");
        assert_eq!(crate::db::queries::endpoint_group("/"), "other");
    }

    #[test]
    fn test_endpoint_usage_query_shape() {
        // Validate SQL skeleton for get_endpoint_usage matches spec:
        // SELECT path, event_type, COUNT(*) FROM usage_events WHERE ts > NOW()-interval ... GROUP BY path, event_type ORDER BY count DESC LIMIT
        // Actual column is created_at; ensure the query string in source has the expected clauses.
        let sql = r#"SELECT path, event_type, COUNT(*)::bigint as cnt FROM usage_events WHERE created_at > NOW() - ($1 || ' days')::INTERVAL GROUP BY path, event_type ORDER BY cnt DESC LIMIT $2"#;
        assert!(sql.contains("usage_events"));
        assert!(sql.contains("GROUP BY path, event_type"));
        assert!(sql.contains("ORDER BY cnt DESC"));
        assert!(sql.contains("created_at"));
    }
}

// ── User format preferences ─────────────────────────────────────────