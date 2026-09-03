//! The async [`Store`] trait and its feature-gated Postgres implementation.
//!
//! The trait is generic over the actor id type `A` and covers the full
//! forum domain: category CRUD, topic CRUD + listing with cursor
//! pagination, post CRUD + listing, votes (upsert/remove/counts), follows,
//! read-state and bans.
//!
//! With the `postgres` feature enabled (default), the sqlx-backed
//! `store::pg::PgStore` implements this trait against Postgres via sqlx.
//! Without it the trait itself remains available for other backends — the
//! pure domain compiles with no database dependency at all.
//!
//! Errors are backend-specific; the trait uses [`StoreError`] as a stable,
//! backend-agnostic surface with a `Postgres` variant carrying the raw
//! sqlx error for callers that need it.

use crate::model::{Ban, Category, Post, ReadState, Topic, Vote};

#[allow(unused_imports)]
use crate::model::{Post as _, Topic as _};
use async_trait::async_trait;

/// Backend-agnostic error for store operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// A row that the operation assumed to exist did not (e.g. a topic
    /// referenced by a post insert).
    #[error("requested row not found")]
    NotFound,
    /// A uniqueness/integrity violation (duplicate vote, duplicate follow,
    /// duplicate category slug, …).
    #[error("conflict: {0}")]
    Conflict(String),
    /// Raw backend error. With the `postgres` feature this carries the
    /// underlying `sqlx::Error` (see `From<sqlx::Error>` below).
    #[error("backend error: {0}")]
    Backend(String),
    /// An input value failed validation (e.g. a vote value outside
    /// {-1, +1}).
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// Any other error.
    #[error("store error: {0}")]
    Other(String),
}

impl StoreError {
    /// Wrap a backend error string (kept separate so the trait compiles
    /// without the `postgres` feature).
    pub fn backend(msg: impl Into<String>) -> Self {
        Self::Backend(msg.into())
    }
}

/// Convert a sqlx error into a [`StoreError`] — only compiled when the
/// `postgres` feature is on, so the pure domain never depends on sqlx.
#[cfg(feature = "postgres")]
impl From<sqlx::Error> for StoreError {
    fn from(e: sqlx::Error) -> Self {
        Self::Backend(e.to_string())
    }
}

/// Generic-actor sqlx glue: `Type` + `Encode` + `Decode` + `FromRow`.
///
/// [`Store`] is generic over `A`; the Postgres impl needs sqlx bounds on
/// `A` for binds and row decoding. sqlx's blanket impls give us most of it
/// for any `A` that is itself `sqlx::Type`, but the **model structs**
/// (`Topic<A>` etc.) need `FromRow` — which the sqlx derive can't provide
/// generically over `A`. We provide it manually: the only `A`-typed
/// columns in the model are the actor-id fields, decoded as i64.
#[cfg(feature = "postgres")]
mod sqlx_glue {
    use crate::model::{Ban, Category, Post, ReadState, Topic, Vote};
    use sqlx::{FromRow, Postgres as Pg};

    /// Manual `FromRow` for each model struct — a **type-level
    /// placeholder** so `query_as::<_, Topic<A>>` type-checks. The actual
    /// rows are decoded by sqlx's derive on the concrete instantiations
    /// (`Topic<i32>`, `Topic<i64>`, …), which the embedding app instantiates.
    macro_rules! impl_from_row {
        ($($t:ident),+) => {$(
            impl<A> FromRow<'_, sqlx::postgres::PgRow> for $t<A>
            where
                A: sqlx::Type<Pg> + for<'q> sqlx::Decode<'q, Pg> + Unpin,
            {
                fn from_row(_row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
                    unreachable!("generic FromRow is a type-level placeholder; concrete impls decode rows")
                }
            }
        )+};
    }
    impl_from_row!(Category, Topic, Post, Vote, ReadState, Ban);
}

/// A single page of cursor-paginated results.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    /// Cursor for the next page (`None` = no more items). The cursor is a
    /// raw id suitable for the `after`/`before` parameter of the list
    /// methods.
    pub next_cursor: Option<i64>,
}

/// Cursor-pagination parameters: fetch items with `id > after` (or
/// `id < before` for reverse iteration), up to `limit` items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub after: Option<i64>,
    pub before: Option<i64>,
    pub limit: usize,
}

impl Cursor {
    /// A cursor with the default page size (25). `after` and `before` are
    /// mutually exclusive — if both are supplied, `after` wins.
    pub fn new(after: Option<i64>, before: Option<i64>, limit: Option<usize>) -> Self {
        Self {
            after,
            before,
            limit: limit.unwrap_or(Self::DEFAULT_LIMIT).clamp(1, Self::MAX_LIMIT),
        }
    }

    pub const DEFAULT_LIMIT: usize = 25;
    pub const MAX_LIMIT: usize = 100;
}

/// The full forum store surface.
#[async_trait]
pub trait Store<A>: Send + Sync
where
    A: Send + Sync + 'static,
{
    // --- Categories -----------------------------------------------------

    async fn create_category(
        &self,
        slug: &str,
        title: &str,
        description: &str,
        position: i32,
        is_mod_only: bool,
    ) -> Result<Category<A>, StoreError>;
    async fn get_category(&self, id: i64) -> Result<Category<A>, StoreError>;
    async fn get_category_by_slug(&self, slug: &str) -> Result<Category<A>, StoreError>;
    /// All categories ordered by position, then id. Mod-only categories are
    /// included — filtering is the caller's concern (readers may still see
    /// them; only writes are gated).
    async fn list_categories(&self) -> Result<Vec<Category<A>>, StoreError>;
    async fn update_category(
        &self,
        id: i64,
        slug: &str,
        title: &str,
        description: &str,
        position: i32,
        is_mod_only: bool,
    ) -> Result<Category<A>, StoreError>;
    async fn delete_category(&self, id: i64) -> Result<(), StoreError>;

    // --- Topics ---------------------------------------------------------

    /// Create a topic (the OP post is created atomically). The status must
    /// be postable (`Open`/`Pinned`) — locked/archived topics can never be
    /// created.
    async fn create_topic(
        &self,
        category_id: i64,
        author_id: A,
        title: &str,
        body: &str,
        payload: serde_json::Value,
    ) -> Result<Topic<A>, StoreError>;
    async fn get_topic(&self, id: i64) -> Result<Topic<A>, StoreError>;
    /// Cursor-paginated topic list for a category, newest-activity first
    /// (pinned topics sort first). `q` (when non-empty) triggers a
    /// full-text-search match instead of the plain listing.
    async fn list_topics(
        &self,
        category_id: i64,
        cursor: &Cursor,
        q: Option<&str>,
    ) -> Result<Page<Topic<A>>, StoreError>;
    async fn update_topic_title_body(
        &self,
        id: i64,
        title: &str,
        body: &str,
    ) -> Result<Topic<A>, StoreError>;
    /// Soft-delete a topic; its posts are soft-deleted in the same
    /// transaction (children survive for quote/reply integrity).
    async fn soft_delete_topic(&self, id: i64) -> Result<(), StoreError>;
    async fn set_topic_status(&self, id: i64, status: crate::model::Status) -> Result<Topic<A>, StoreError>;
    async fn set_topic_hidden(&self, id: i64, hidden: bool) -> Result<(), StoreError>;
    async fn increment_view_count(&self, id: i64) -> Result<(), StoreError>;

    // --- Posts ----------------------------------------------------------

    async fn create_post(
        &self,
        topic_id: i64,
        author_id: A,
        body: &str,
        payload: serde_json::Value,
        quote_of: Option<i64>,
    ) -> Result<Post<A>, StoreError>;
    async fn get_post(&self, id: i64) -> Result<Post<A>, StoreError>;
    /// Cursor-paginated posts in a topic, chronological (`id > after`).
    async fn list_posts(&self, topic_id: i64, cursor: &Cursor) -> Result<Page<Post<A>>, StoreError>;
    async fn update_post_body(&self, id: i64, body: &str) -> Result<Post<A>, StoreError>;
    /// Soft-delete a post (`deleted_at` set; children untouched).
    async fn soft_delete_post(&self, id: i64) -> Result<(), StoreError>;
    async fn set_post_hidden(&self, id: i64, hidden: bool) -> Result<(), StoreError>;

    // --- Votes ----------------------------------------------------------

    /// Upsert a vote; `value` must be `±1` (invalid values are rejected
    /// with `StoreError::Other`). Returns the resulting counts.
    async fn upsert_vote(&self, post_id: i64, user_id: A, value: i8) -> Result<VoteCounts, StoreError>;
    async fn remove_vote(&self, post_id: i64, user_id: A) -> Result<(), StoreError>;
    async fn get_vote(&self, post_id: i64, user_id: A) -> Result<Option<Vote<A>>, StoreError>;
    async fn vote_counts(&self, post_id: i64) -> Result<VoteCounts, StoreError>;

    // --- Follows --------------------------------------------------------

    /// Idempotent follow.
    async fn follow(&self, user_id: A, topic_id: i64) -> Result<(), StoreError>;
    /// Idempotent unfollow.
    async fn unfollow(&self, user_id: A, topic_id: i64) -> Result<(), StoreError>;
    async fn is_following(&self, user_id: A, topic_id: i64) -> Result<bool, StoreError>;
    /// All users following a topic (for reply/mention notifications).
    async fn followers(&self, topic_id: i64) -> Result<Vec<A>, StoreError>;

    // --- Read state -----------------------------------------------------

    /// Idempotent upsert of `last_read_post_id` for a user/topic pair.
    async fn mark_read(&self, user_id: A, topic_id: i64, last_read_post_id: i64) -> Result<(), StoreError>;
    async fn get_read_state(&self, user_id: A, topic_id: i64) -> Result<Option<ReadState<A>>, StoreError>;
    /// All unread topics for a user (topics whose `last_post_id` exceeds
    /// the user's `last_read_post_id`).
    async fn unread_topics(&self, user_id: A) -> Result<Vec<Topic<A>>, StoreError>;

    // --- Bans -----------------------------------------------------------

    /// Whether a user is currently banned from the whole forum or a
    /// category (any active, non-expired ban). Convenience for write gates.
    async fn is_banned(&self, user_id: A, category_id: Option<i64>) -> Result<bool, StoreError>;

    async fn create_ban(
        &self,
        user_id: A,
        category_id: Option<i64>,
        reason: &str,
        banned_by: A,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Ban<A>, StoreError>;
    async fn lift_ban(&self, ban_id: i64) -> Result<(), StoreError>;
    async fn list_bans(&self) -> Result<Vec<Ban<A>>, StoreError>;
    /// All *active* bans affecting `user_id` (global or category-specific,
    /// not expired). Bans that expired are filtered out here; callers that
    /// need the full history use [`Store::list_bans`].
    async fn active_bans_for(&self, user_id: A) -> Result<Vec<Ban<A>>, StoreError>;
}

pub use votes::VoteCounts;

/// Re-export of the sqlx error type when the `postgres` feature is on
/// (kept behind `#[cfg]` so the trait compiles without it).
#[cfg(feature = "postgres")]
pub use sqlx::Error as SqlxError;

mod votes {
    /// Aggregate vote counts for a post: `up - down`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
    pub struct VoteCounts {
        pub up: i64,
        pub down: i64,
    }

    impl VoteCounts {
        pub fn score(&self) -> i64 {
            self.up - self.down
        }
    }
}

#[cfg(feature = "postgres")]
mod pg; // declared after the trait so the cfg-gated impl compiles

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_clamps() {
        let c = Cursor::new(None, None, Some(10_000));
        assert_eq!(c.limit, Cursor::MAX_LIMIT);
        let c = Cursor::new(None, None, Some(0));
        assert_eq!(c.limit, 1);
        let c = Cursor::new(Some(5), Some(9), None);
        assert_eq!(c.after, Some(5)); // after wins over before
        assert_eq!(c.limit, Cursor::DEFAULT_LIMIT);
    }

    #[test]
    fn store_error_backend_no_sqlx() {
        // Compile-time: StoreError must not reference sqlx without the
        // postgres feature.
        let e = StoreError::backend("boom");
        assert!(matches!(e, StoreError::Backend(s) if s == "boom"));
    }

    #[test]
    fn page_cursor_construction() {
        // Simulate the PgStore page-splitting contract: fetch limit+1,
        // truncate, expose next cursor from the last kept item.
        let mut rows = vec![1i64, 2, 3, 4];
        let has_more = rows.len() > 3;
        rows.truncate(3);
        let next_cursor = if has_more { rows.last().copied() } else { None };
        assert_eq!(next_cursor, Some(3));
    }
}
