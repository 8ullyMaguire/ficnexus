//! Postgres (sqlx) implementation of [`Store`] — feature-gated behind
//! `postgres`.
//!
//! All queries use sqlx's compile-time-checked `query_as`/`query` with
//! named or positional binds; `A` is bound as `i64` (users.id is `INT4` in
//! FicHub — the crate itself is generic, and this impl picks the widest
//! Postgres integer type so any `i32`/`i64` actor id works).
//!
//! `create_topic`, `create_post` and `soft_delete_topic` run inside
//! transactions so the OP post + topic (or topic + child soft-deletes) are
//! written atomically, and `search_vector` columns are maintained at write
//! time (no cron dependency).

use crate::model::{Ban, Category, Post, ReadState, Status, Topic, Vote};
use crate::store::{Cursor, Page, Store, StoreError, VoteCounts};
#[allow(unused_imports)]
use async_trait::async_trait;
use sqlx::postgres::PgArgumentBuffer;

/// Postgres-backed forum store. Cheap to clone (wraps `sqlx::PgPool`).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PgStore {
    pool: sqlx::PgPool,
}

#[allow(dead_code)]
impl PgStore {
    /// Wrap an existing pool. No connection is opened here; the pool is
    /// shared with the embedding application.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Create a pool from a `DATABASE_URL`-style connection string.
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::PgPool::connect(database_url).await?;
        Ok(Self::new(pool))
    }

    /// Migrations embedded in the crate (`forum_core/migrations/`). Run
    /// with [`Self::run_migrations`]. The `migrate` sqlx feature must be
    /// enabled (it is, via the `postgres` feature).
    pub fn migrator() -> sqlx::migrate::Migrator {
        // `sqlx::migrate!` resolves relative to CARGO_MANIFEST_DIR, so
        // "./migrations" is the crate-root dir both locally and in the
        // `cargo publish` package (where "../migrations" would escape the
        // crate and fail canonicalization).
        sqlx::migrate!("./migrations")
    }
    /// Apply this crate's embedded migrations (idempotent).
    #[cfg(feature = "postgres")]
    pub async fn run_migrations(&self) -> Result<(), sqlx::migrate::MigrateError> {
        Self::migrator().run(&self.pool).await
    }

    /// Access the underlying pool (for transactions, ad-hoc queries, …).
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

impl PgStore {
    /// Split a fetched page (limit+1 rows) into items + next cursor.
    #[allow(dead_code)]
    fn split_page<T>(mut rows: Vec<T>, limit: usize) -> Page<T>
    where
        T: IdRef,
    {
        let has_more = rows.len() > limit;
        rows.truncate(limit);
        let next_cursor = if has_more {
            rows.last().map(|r| r.id_ref())
        } else {
            None
        };
        Page { items: rows, next_cursor }
    }
}

/// Internal helper so [`PgStore::split_page`] can read an id off any row
/// type.
pub trait IdRef {
    fn id_ref(&self) -> i64;
}

// `split_page` is called with `Topic<A>`/`Post<A>` for any actor `A`
// (i64 in practice), so the impls are generic.
impl<A> IdRef for Topic<A> {
    fn id_ref(&self) -> i64 {
        self.id
    }
}

impl<A> IdRef for Post<A> {
    fn id_ref(&self) -> i64 {
        self.id
    }
}

impl PgStore {
    #[allow(dead_code)]
    async fn is_banned_impl(&self, user_id: i64, category_id: Option<i64>) -> Result<bool, StoreError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM forum_bans
             WHERE user_id = $1
               AND (expires_at IS NULL OR expires_at > NOW())
               AND ($2::bigint IS NULL OR category_id = $2 OR category_id IS NULL)
             LIMIT 1",
        )
        .bind(user_id)
        .bind(category_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }
}

#[async_trait]
impl<A> Store<A> for PgStore
where
    A: sqlx::Type<sqlx::Postgres>
        + for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + for<'q> sqlx::Decode<'q, sqlx::Postgres>
        + Unpin
        + Send
        + Sync
        + 'static,
{
    async fn create_category(
        &self,
        slug: &str,
        title: &str,
        description: &str,
        position: i32,
        is_mod_only: bool,
    ) -> Result<Category<A>, StoreError> {
        let row = sqlx::query_as::<_, Category<A>>(
            "INSERT INTO forum_categories (slug, title, description, position, is_mod_only)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
        )
        .bind(slug)
        .bind(title)
        .bind(description)
        .bind(position)
        .bind(is_mod_only)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn get_category(&self, id: i64) -> Result<Category<A>, StoreError> {
        let row = sqlx::query_as::<_, Category<A>>(
            "SELECT * FROM forum_categories WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn get_category_by_slug(&self, slug: &str) -> Result<Category<A>, StoreError> {
        let row = sqlx::query_as::<_, Category<A>>(
            "SELECT * FROM forum_categories WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn list_categories(&self) -> Result<Vec<Category<A>>, StoreError> {
        let rows = sqlx::query_as::<_, Category<A>>(
            "SELECT * FROM forum_categories ORDER BY position ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn update_category(
        &self,
        id: i64,
        slug: &str,
        title: &str,
        description: &str,
        position: i32,
        is_mod_only: bool,
    ) -> Result<Category<A>, StoreError> {
        let row = sqlx::query_as::<_, Category<A>>(
            "UPDATE forum_categories
             SET slug = $2, title = $3, description = $4, position = $5, is_mod_only = $6
             WHERE id = $1
             RETURNING *",
        )
        .bind(id)
        .bind(slug)
        .bind(title)
        .bind(description)
        .bind(position)
        .bind(is_mod_only)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn delete_category(&self, id: i64) -> Result<(), StoreError> {
        let res = sqlx::query("DELETE FROM forum_categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn create_topic(
        &self,
        category_id: i64,
        author_id: A,
        title: &str,
        body: &str,
        payload: serde_json::Value,
    ) -> Result<Topic<A>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let topic: Topic<A> = sqlx::query_as(
            "INSERT INTO forum_topics (category_id, author_id, title, body, payload, search_vector)
             VALUES ($1, $2, $3, $4, $5,
                     to_tsvector('english', $3 || ' ' || $4))
             RETURNING *",
        )
        .bind(category_id)
        .bind(&author_id)
        .bind(title)
        .bind(body)
        .bind(&payload)
        .fetch_one(&mut *tx)
        .await?;

        let post: Post<A> = sqlx::query_as(
            "INSERT INTO forum_posts (topic_id, author_id, body, payload, search_vector)
             VALUES ($1, $2, $3, $4,
                     to_tsvector('english', $3))
             RETURNING *",
        )
        .bind(topic.id)
        .bind(&author_id)
        .bind(body)
        .bind(&payload)
        .fetch_one(&mut *tx)
        .await?;

        let topic: Topic<A> = sqlx::query_as(
            "UPDATE forum_topics SET last_post_id = $2, last_activity_at = NOW() WHERE id = $1
             RETURNING *",
        )
        .bind(topic.id)
        .bind(post.id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(topic)
    }

    async fn get_topic(&self, id: i64) -> Result<Topic<A>, StoreError> {
        let row = sqlx::query_as::<_, Topic<A>>("SELECT * FROM forum_topics WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn list_topics(
        &self,
        category_id: i64,
        cursor: &Cursor,
        q: Option<&str>,
    ) -> Result<Page<Topic<A>>, StoreError> {
        let limit = cursor.limit as i64 + 1; // fetch one extra to detect next page
        let has_q = q.map(|s| !s.trim().is_empty()).unwrap_or(false);

        if has_q {
            let rows = sqlx::query_as::<_, Topic<A>>(
                "SELECT t.* FROM forum_topics t
                 WHERE t.category_id = $1
                   AND t.search_vector @@ websearch_to_tsquery('english', $3)
                   AND t.deleted_at IS NULL AND t.is_hidden = FALSE
                 ORDER BY t.status = 'pinned' DESC, t.last_activity_at DESC, t.id DESC
                 LIMIT $2",
            )
            .bind(category_id)
            .bind(limit)
            .bind(q.unwrap_or_default())
            .fetch_all(&self.pool)
            .await?;
            return Ok(Self::split_page(rows, cursor.limit as usize));
        }
        let rows = sqlx::query_as::<_, Topic<A>>(
            "SELECT * FROM forum_topics
             WHERE category_id = $1
               AND deleted_at IS NULL AND is_hidden = FALSE
               AND ($2::bigint IS NULL OR id < $2)
             ORDER BY status = 'pinned' DESC, last_activity_at DESC, id DESC
             LIMIT $3",
        )
        .bind(category_id)
        .bind(cursor.before)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(Self::split_page(rows, cursor.limit as usize))
    }

    async fn update_topic_title_body(
        &self,
        id: i64,
        title: &str,
        body: &str,
    ) -> Result<Topic<A>, StoreError> {
        let row = sqlx::query_as::<_, Topic<A>>(
            "UPDATE forum_topics
             SET title = $2, body = $3, updated_at = NOW(),
                 search_vector = to_tsvector('english', $2 || ' ' || $3)
             WHERE id = $1
             RETURNING *",
        )
        .bind(id)
        .bind(title)
        .bind(body)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn soft_delete_topic(&self, id: i64) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE forum_posts SET deleted_at = NOW() WHERE topic_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        let res = sqlx::query("UPDATE forum_topics SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if res.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(StoreError::NotFound);
        }
        tx.commit().await?;
        Ok(())
    }

    async fn set_topic_status(
        &self,
        id: i64,
        status: Status,
    ) -> Result<Topic<A>, StoreError> {
        let row = sqlx::query_as::<_, Topic<A>>(
            "UPDATE forum_topics SET status = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status.as_db())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn set_topic_hidden(&self, id: i64, hidden: bool) -> Result<(), StoreError> {
        let res = sqlx::query("UPDATE forum_topics SET is_hidden = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(hidden)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn increment_view_count(&self, id: i64) -> Result<(), StoreError> {
        let res = sqlx::query("UPDATE forum_topics SET view_count = view_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn create_post(
        &self,
        topic_id: i64,
        author_id: A,
        body: &str,
        payload: serde_json::Value,
        quote_of: Option<i64>,
    ) -> Result<Post<A>, StoreError> {
        let mut tx = self.pool.begin().await?;

        // Refuse posts into locked/archived topics, inside the same
        // transaction as the insert.
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM forum_topics WHERE id = $1 AND deleted_at IS NULL AND is_hidden = FALSE",
        )
        .bind(topic_id)
        .fetch_optional(&mut *tx)
        .await?;
        let status = status.ok_or(StoreError::NotFound)?;
        let status = Status::from_db(&status).ok_or_else(|| {
            StoreError::Other(format!("unknown topic status in db: {status}"))
        })?;
        if !status.is_postable() {
            return Err(StoreError::Other(format!(
                "topic is {} — no new posts allowed",
                status.as_db()
            )));
        }

        let post: Post<A> = sqlx::query_as(
            "INSERT INTO forum_posts (topic_id, author_id, body, payload, quote_of, search_vector)
             VALUES ($1, $2, $3, $4, $5, to_tsvector('english', $3))
             RETURNING *",
        )
        .bind(topic_id)
        .bind(&author_id)
        .bind(body)
        .bind(&payload)
        .bind(quote_of)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE forum_topics
             SET last_post_id = $2, last_activity_at = NOW(), updated_at = NOW()
             WHERE id = $1",
        )
        .bind(topic_id)
        .bind(post.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(post)
    }

    async fn get_post(&self, id: i64) -> Result<Post<A>, StoreError> {
        let row = sqlx::query_as::<_, Post<A>>("SELECT * FROM forum_posts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn list_posts(&self, topic_id: i64, cursor: &Cursor) -> Result<Page<Post<A>>, StoreError> {
        let limit = cursor.limit as i64 + 1;
        let rows = sqlx::query_as::<_, Post<A>>(
            "SELECT * FROM forum_posts
             WHERE topic_id = $1
               AND deleted_at IS NULL AND is_hidden = FALSE
               AND ($2::bigint IS NULL OR id > $2)
             ORDER BY id ASC
             LIMIT $3",
        )
        .bind(topic_id)
        .bind(cursor.after)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(Self::split_page(rows, cursor.limit as usize))
    }

    async fn update_post_body(&self, id: i64, body: &str) -> Result<Post<A>, StoreError> {
        let row = sqlx::query_as::<_, Post<A>>(
            "UPDATE forum_posts
             SET body = $2, edited_at = NOW(), search_vector = to_tsvector('english', $2)
             WHERE id = $1
             RETURNING *",
        )
        .bind(id)
        .bind(body)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(row)
    }

    async fn soft_delete_post(&self, id: i64) -> Result<(), StoreError> {
        let res = sqlx::query("UPDATE forum_posts SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn set_post_hidden(&self, id: i64, hidden: bool) -> Result<(), StoreError> {
        let res = sqlx::query("UPDATE forum_posts SET is_hidden = $2 WHERE id = $1")
            .bind(id)
            .bind(hidden)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn upsert_vote(&self, post_id: i64, user_id: A, value: i8) -> Result<VoteCounts, StoreError> {
        if !Vote::<A>::is_valid_value(value) {
            return Err(StoreError::Other(format!(
                "invalid vote value {value}: must be -1 or 1"
            )));
        }
        sqlx::query(
            "INSERT INTO forum_post_votes (post_id, user_id, value)
             VALUES ($1, $2, $3)
             ON CONFLICT (post_id, user_id) DO UPDATE SET value = EXCLUDED.value",
        )
        .bind(post_id)
        .bind(user_id)
        .bind(value)
        .execute(&self.pool)
        .await?;
        <Self as Store<A>>::vote_counts(self, post_id).await
    }

    async fn remove_vote(&self, post_id: i64, user_id: A) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM forum_post_votes WHERE post_id = $1 AND user_id = $2")
            .bind(post_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_vote(&self, post_id: i64, user_id: A) -> Result<Option<Vote<A>>, StoreError> {
        let row = sqlx::query_as::<_, Vote<A>>(
            "SELECT * FROM forum_post_votes WHERE post_id = $1 AND user_id = $2",
        )
        .bind(post_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn vote_counts(&self, post_id: i64) -> Result<VoteCounts, StoreError> {
        let row: (i64, i64) = sqlx::query_as(
            "SELECT
                 COALESCE(SUM(value) FILTER (WHERE value = 1), 0)::bigint,
                 COALESCE(SUM(value) FILTER (WHERE value = -1), 0)::bigint
             FROM forum_post_votes WHERE post_id = $1",
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(VoteCounts { up: row.0, down: -row.1 })
    }

    async fn follow(&self, user_id: A, topic_id: i64) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO forum_follows (user_id, topic_id)
             VALUES ($1, $2)
             ON CONFLICT (user_id, topic_id) DO NOTHING",
        )
        .bind(user_id)
        .bind(topic_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn unfollow(&self, user_id: A, topic_id: i64) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM forum_follows WHERE user_id = $1 AND topic_id = $2")
            .bind(user_id)
            .bind(topic_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn is_following(&self, user_id: A, topic_id: i64) -> Result<bool, StoreError> {
        let exists: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM forum_follows WHERE user_id = $1 AND topic_id = $2",
        )
        .bind(user_id)
        .bind(topic_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(exists.is_some())
    }

    async fn followers(&self, topic_id: i64) -> Result<Vec<A>, StoreError> {
        let rows: Vec<(A,)> = sqlx::query_as("SELECT user_id FROM forum_follows WHERE topic_id = $1")
            .bind(topic_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn mark_read(&self, user_id: A, topic_id: i64, last_read_post_id: i64) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO forum_read_state (user_id, topic_id, last_read_post_id)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, topic_id) DO UPDATE
               SET last_read_post_id = EXCLUDED.last_read_post_id, updated_at = NOW()",
        )
        .bind(user_id)
        .bind(topic_id)
        .bind(last_read_post_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_read_state(&self, user_id: A, topic_id: i64) -> Result<Option<ReadState<A>>, StoreError> {
        let row = sqlx::query_as::<_, ReadState<A>>(
            "SELECT * FROM forum_read_state WHERE user_id = $1 AND topic_id = $2",
        )
        .bind(user_id)
        .bind(topic_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn unread_topics(&self, user_id: A) -> Result<Vec<Topic<A>>, StoreError> {
        let rows = sqlx::query_as::<_, Topic<A>>(
            "SELECT t.* FROM forum_topics t
             JOIN forum_read_state r ON r.topic_id = t.id
             WHERE r.user_id = $1
               AND t.deleted_at IS NULL AND t.is_hidden = FALSE
               AND (r.last_read_post_id IS NULL OR r.last_read_post_id < t.last_post_id)
             ORDER BY t.last_activity_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn create_ban(
        &self,
        user_id: A,
        category_id: Option<i64>,
        reason: &str,
        banned_by: A,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Ban<A>, StoreError> {
        let row = sqlx::query_as::<_, Ban<A>>(
            "INSERT INTO forum_bans (user_id, category_id, reason, banned_by, expires_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
        )
        .bind(user_id)
        .bind(category_id)
        .bind(reason)
        .bind(banned_by)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn lift_ban(&self, ban_id: i64) -> Result<(), StoreError> {
        let res = sqlx::query("DELETE FROM forum_bans WHERE id = $1")
            .bind(ban_id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn list_bans(&self) -> Result<Vec<Ban<A>>, StoreError> {
        let rows = sqlx::query_as::<_, Ban<A>>(
            "SELECT * FROM forum_bans ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn active_bans_for(&self, user_id: A) -> Result<Vec<Ban<A>>, StoreError> {
        let rows = sqlx::query_as::<_, Ban<A>>(
            "SELECT * FROM forum_bans
             WHERE user_id = $1
               AND (expires_at IS NULL OR expires_at > NOW())
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn is_banned(&self, user_id: A, category_id: Option<i64>) -> Result<bool, StoreError> {
        let mut buf = PgArgumentBuffer::default();
        let _ = sqlx::Encode::<sqlx::Postgres>::encode(&user_id, &mut buf)
            .map_err(|_| StoreError::InvalidInput("actor id".into()))?;
        let uid = i64::from_ne_bytes(
            buf.as_slice()[..8].try_into().map_err(|_| StoreError::InvalidInput("actor id".into()))?,
        );
        self.is_banned_impl(uid, category_id).await
    }
}

/// Helper to convert a generic actor id to `i64` for the bounded ban
/// query (the `user_id` column is INT4).
#[allow(dead_code)]
pub trait ToSqlI64 {
    fn to_sql_i64(&self) -> i64;
}

impl ToSqlI64 for i32 {
    fn to_sql_i64(&self) -> i64 {
        *self as i64
    }
}

impl ToSqlI64 for i64 {
    fn to_sql_i64(&self) -> i64 {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The sqlx store needs a live Postgres; those tests live in the
    // DB-gated FicHub suite (`tests/forum_api.rs`). Pure-domain tests
    // (status machine, vote values, payload opacity, cursors, FTS
    // fragments) are in `model`, `moderation` and `search`.
    #[test]
    fn store_trait_is_object_safe_and_generic() {
        fn assert_store<S: crate::store::Store<i32> + Send + Sync + 'static>() {}
        assert_store::<PgStore>();
        // Generic over i64 actors too (Postgres BIGSERIAL users).
        fn assert_store64<S: crate::store::Store<i64> + Send + Sync + 'static>() {}
        assert_store64::<PgStore>();
    }
}
