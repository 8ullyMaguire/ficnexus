use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::server::AppState;

use super::{
    build_feed, fic_entry, html_escape, iso_now, opds_response, FeedKind,
};

/// Query params for author feeds
#[derive(Debug, Deserialize)]
pub struct AuthorQuery {
    pub author: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl AuthorQuery {
    pub fn page(&self) -> usize {
        self.page.unwrap_or(1).max(1)
    }
    pub fn per_page(&self) -> usize {
        self.per_page.unwrap_or(50).max(1).min(100)
    }
    pub fn offset(&self) -> usize {
        (self.page() - 1) * self.per_page()
    }
}

/// GET /opds/authors — Navigation feed listing authors with fic counts,
/// or if `?author=Name` is given, acquisition feed of that author's fics.
pub async fn author_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuthorQuery>,
) -> Result<impl IntoResponse, AppError> {
    // If an author name is provided, show their fics
    if let Some(ref name) = params.author {
        if !name.is_empty() {
            let body = build_author_fic_feed(&state, &params, name).await?;
            return Ok(opds_response(body, FeedKind::Acquisition));
        }
    }

    // Otherwise show the author index
    let body = build_author_index_feed(&state).await?;
    Ok(opds_response(body, FeedKind::Navigation))
}

/// Build the navigation feed body for all authors
async fn build_author_index_feed(state: &Arc<AppState>) -> AppResult<String> {
    #[derive(sqlx::FromRow)]
    struct AuthorRow {
        author: String,
        cnt: Option<i64>,
    }

    let rows: Vec<AuthorRow> = sqlx::query_as::<_, AuthorRow>(
        r#"SELECT author, COUNT(*) as cnt
           FROM fic_info
           GROUP BY author
           ORDER BY cnt DESC
           LIMIT 100"#,
    )
    .fetch_all(&state.db)
    .await?;

    let now = iso_now();
    let mut entries = String::new();

    for row in &rows {
        let cnt = row.cnt.unwrap_or(0);
        entries.push_str(&format!(
            r#"  <entry>
    <title>{author} ({cnt})</title>
    <link href="/opds/authors?author={encoded}" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <updated>{now}</updated>
    <id>urn:ficnexus:author:{id}</id>
    <summary>{cnt} fics by {author}</summary>
  </entry>
"#,
            author = html_escape(&row.author),
            encoded = html_escape(&row.author),
            cnt = cnt,
            now = now,
            id = html_escape(&row.author),
        ));
    }

    let base_url = state.config.opds_base_url.as_deref();

    Ok(build_feed(
        "FicNexus — Authors",
        "urn:ficnexus:authors",
        &entries,
        &now,
        Some("/opds/authors"),
        FeedKind::Navigation,
        None,
        base_url,
    ))
}

/// Build the acquisition feed body for a specific author's fics
async fn build_author_fic_feed(
    state: &Arc<AppState>,
    params: &AuthorQuery,
    author_name: &str,
) -> AppResult<String> {
    let offset = params.offset();
    let limit = params.per_page() as i64;

    #[derive(sqlx::FromRow)]
    struct FicRow {
        id: String,
        title: String,
        author: String,
        description: String,
        fic_updated: chrono::DateTime<chrono::Utc>,
        words: i64,
        chapters: i32,
        status: String,
    }

    let rows: Vec<FicRow> = sqlx::query_as::<_, FicRow>(
        r#"SELECT id, title, author, description,
                  fic_updated, words, chapters, status
           FROM fic_info
           WHERE author = $1
           ORDER BY fic_updated DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(author_name)
    .bind(limit)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_info WHERE author = $1",
    )
    .bind(author_name)
    .fetch_one(&state.db)
    .await?;

    let now = iso_now();
    let base_url = state.config.opds_base_url.as_deref();
    let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    let acq_map = super::acquisition_links_many(&state.db, base_url, &ids).await;
    let mut entries = String::new();

    for row in &rows {
        let updated = row
            .fic_updated
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let acq = acq_map.get(&row.id).map(|v| v.as_slice()).unwrap_or(&[]);
        entries.push_str(&fic_entry(
            &row.id,
            &row.title,
            &row.author,
            &row.description,
            &updated,
            row.words,
            row.chapters,
            &row.status,
            acq,
        ));
    }

    let pagi = super::PaginationInfo {
        base_path: format!("/opds/authors?author={}", urlencoding(author_name)),
        page: params.page(),
        per_page: params.per_page(),
        total: total.0,
    };

    let self_link = format!(
        "/opds/authors?author={}&page={}&per_page={}",
        urlencoding(author_name),
        params.page(),
        params.per_page()
    );

    Ok(build_feed(
        &format!("FicNexus — Author: {}", author_name),
        &format!("urn:ficnexus:author:{}", html_escape(author_name)),
        &entries,
        &now,
        Some(&self_link),
        FeedKind::Acquisition,
        Some(&pagi),
        base_url,
    ))
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F")
        .replace('/', "%2F")
        .replace('&', "%26")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_author_query_defaults() {
        let q = AuthorQuery {
            author: None,
            page: None,
            per_page: None,
        };
        assert_eq!(q.page(), 1);
        assert_eq!(q.per_page(), 50);
        assert_eq!(q.offset(), 0);
    }

    #[test]
    fn test_author_query_custom() {
        let q = AuthorQuery {
            author: Some("Test Author".into()),
            page: Some(2),
            per_page: Some(10),
        };
        assert_eq!(q.page(), 2);
        assert_eq!(q.per_page(), 10);
        assert_eq!(q.offset(), 10);
    }

    #[test]
    fn test_author_query_clamp_low() {
        let q = AuthorQuery {
            author: None,
            page: Some(0),
            per_page: Some(0),
        };
        assert_eq!(q.page(), 1);
        assert_eq!(q.per_page(), 1);
    }

    #[test]
    fn test_author_query_clamp_high() {
        let q = AuthorQuery {
            author: None,
            page: Some(1),
            per_page: Some(200),
        };
        assert_eq!(q.per_page(), 100);
    }

    #[test]
    fn test_urlencoding_basic() {
        assert_eq!(urlencoding("hello"), "hello");
    }

    #[test]
    fn test_urlencoding_spaces() {
        assert_eq!(urlencoding("Test Author"), "Test%20Author");
    }

    #[test]
    fn test_urlencoding_special_chars() {
        assert_eq!(urlencoding("a#b?c/d&e"), "a%23b%3Fc%2Fd%26e");
    }
}
