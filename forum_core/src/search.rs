//! Postgres full-text-search helpers — pure string/struct construction,
//! no DB access.
//!
//! The forum schema maintains `tsvector` columns (`search_vector`) on
//! `forum_topics` and `forum_posts` at write time. This module builds the
//! SQL fragments used to query them (`@@` predicates, `ts_rank` ordering,
//! `ts_headline` snippets) with a default English config.

use serde::{Deserialize, Serialize};

/// Default text search configuration used by the fragment builders.
pub const DEFAULT_CONFIG: &str = "english";

/// A parsed, validated search query ready to be interpolated into a SQL
/// predicate. Construct via [`FtsQuery::parse`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FtsQuery {
    /// The `websearch_to_tsquery` expression — always `$1`-parameterizable.
    pub tsquery: String,
    /// Optional category filter (`category_id = $N`).
    pub category_id: Option<i64>,
    /// `LIMIT` value.
    pub limit: i64,
    /// `OFFSET` value.
    pub offset: i64,
}

impl FtsQuery {
    /// Parse raw user input into a parameterized search query.
    ///
    /// `limit` is clamped to `[1, MAX_LIMIT]` (default 50), `offset` to
    /// `[0, MAX_OFFSET]` (default 5000). The raw query is **never**
    /// interpolated into SQL — the SQL builder always binds it via the
    /// `websearch_to_tsquery` placeholder.
    pub fn parse(
        raw: &str,
        category_id: Option<i64>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Self {
        Self {
            tsquery: raw.trim().to_owned(),
            category_id,
            limit: limit.map(|l| l.clamp(1, Self::MAX_LIMIT)).unwrap_or(50),
            offset: offset.map(|o| o.clamp(0, Self::MAX_OFFSET)).unwrap_or(0),
        }
    }

    /// Cap on results per page.
    pub const MAX_LIMIT: i64 = 100;
    /// Hard cap on the cursor offset (prevents deep-pagination scans).
    pub const MAX_OFFSET: i64 = 5000;

    /// Whether this query has any non-empty search terms.
    pub fn is_empty(&self) -> bool {
        self.tsquery.trim().is_empty()
    }
}

/// Build a Postgres `ts_rank`-ordered search fragment (topics or posts).
///
/// The returned SQL is a complete, parameterized `SELECT`:
///
/// ```sql
/// SELECT id, ts_rank(search_vector, websearch_to_tsquery('english', $1)) AS rank
/// FROM forum_topics
/// WHERE search_vector @@ websearch_to_tsquery('english', $1)
/// ORDER BY rank DESC, last_activity_at DESC
/// LIMIT $2 OFFSET $3
/// ```
///
/// Bind order: `$1` = raw query, `$2` = limit, `$3` = offset.
pub fn topics_search_sql(query: &FtsQuery) -> String {
    let cfg = DEFAULT_CONFIG;
    let category_clause = match query.category_id {
        Some(id) => format!(" AND category_id = {id}"),
        None => String::new(),
    };
    format!(
        "SELECT id, ts_rank(search_vector, websearch_to_tsquery('{cfg}', $1)) AS rank \
         FROM forum_topics \
         WHERE search_vector @@ websearch_to_tsquery('{cfg}', $1) \
         AND deleted_at IS NULL AND is_hidden = FALSE{category_clause} \
         ORDER BY rank DESC, last_activity_at DESC \
         LIMIT $2 OFFSET $3"
    )
}

/// Build a Postgres search fragment over `forum_posts` with a snippet
/// (headline) of the surrounding text.
///
/// Bind order: `$1` = raw query, `$2` = limit, `$3` = offset.
pub fn posts_search_sql(query: &FtsQuery) -> String {
    let cfg = DEFAULT_CONFIG;
    let category_clause = match query.category_id {
        Some(id) => format!(" AND category_id = {id}"),
        None => String::new(),
    };
    format!(
        "SELECT id, topic_id, \
         ts_headline('{cfg}', body, websearch_to_tsquery('{cfg}', $1), 'StartSel=<mark>, StopSel=</mark>') AS snippet \
         FROM forum_posts \
         WHERE search_vector @@ websearch_to_tsquery('{cfg}', $1) \
         AND deleted_at IS NULL AND is_hidden = FALSE{category_clause} \
         ORDER BY ts_rank(search_vector, websearch_to_tsquery('{cfg}', $1)) DESC, created_at DESC \
         LIMIT $2 OFFSET $3"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clamps_and_defaults() {
        let q = FtsQuery::parse("  hello world  ", None, None, None);
        assert_eq!(q.tsquery, "hello world");
        assert_eq!(q.limit, 50);
        assert_eq!(q.offset, 0);
        assert!(!q.is_empty());

        let q = FtsQuery::parse("x", None, Some(10_000), Some(-5));
        assert_eq!(q.limit, FtsQuery::MAX_LIMIT);
        assert_eq!(q.offset, 0);

        let q = FtsQuery::parse("x", Some(7), Some(0), Some(1_000_000));
        assert_eq!(q.limit, 1);
        assert_eq!(q.offset, FtsQuery::MAX_OFFSET);
        assert_eq!(q.category_id, Some(7));
    }

    #[test]
    fn topics_sql_shape() {
        let q = FtsQuery::parse("hurt comfort", None, Some(20), Some(40));
        let sql = topics_search_sql(&q);
        assert!(sql.starts_with("SELECT id, ts_rank(search_vector, websearch_to_tsquery('english', $1)) AS rank"));
        assert!(sql.contains("WHERE search_vector @@ websearch_to_tsquery('english', $1)"));
        assert!(sql.contains("deleted_at IS NULL AND is_hidden = FALSE"));
        assert!(sql.contains("ORDER BY rank DESC, last_activity_at DESC"));
        assert!(sql.ends_with("LIMIT $2 OFFSET $3"));
        assert!(!sql.contains("category_id"));
    }

    #[test]
    fn topics_sql_with_category_filter() {
        let q = FtsQuery::parse("x", Some(3), None, None);
        let sql = topics_search_sql(&q);
        assert!(sql.contains("AND category_id = 3"));
    }

    #[test]
    fn posts_sql_shape_with_snippet() {
        let q = FtsQuery::parse("canon divergence", None, None, None);
        let sql = posts_search_sql(&q);
        assert!(sql.starts_with("SELECT id, topic_id,"));
        assert!(sql.contains("ts_headline('english', body, websearch_to_tsquery('english', $1), 'StartSel=<mark>, StopSel=</mark>') AS snippet"));
        assert!(sql.contains("ORDER BY ts_rank(search_vector, websearch_to_tsquery('english', $1)) DESC, created_at DESC"));
        assert!(sql.ends_with("LIMIT $2 OFFSET $3"));
        assert!(!sql.contains("category_id"));
    }

    #[test]
    fn raw_query_never_interpolated() {
        // User input must never appear verbatim in the SQL — only in the
        // `$1` placeholder position inside websearch_to_tsquery.
        let evil = "x'; DROP TABLE forum_topics; --";
        let q = FtsQuery::parse(evil, None, None, None);
        let sql = topics_search_sql(&q);
        assert!(!sql.contains("DROP"));
        assert!(!sql.contains(";"));
        assert_eq!(sql.matches("$1").count(), 2);
        assert_eq!(sql.matches("$2").count(), 1);
        assert_eq!(sql.matches("$3").count(), 1);
    }
}
