use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

use super::{
    build_feed, fic_entry, html_escape, iso_now, opds_response, FeedKind,
};

/// Query params for tag feeds
#[derive(Debug, Deserialize)]
pub struct TagQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl TagQuery {
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

/// GET /opds/tags — Navigation feed listing all tag types
pub async fn tag_types(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    #[derive(sqlx::FromRow)]
    struct TagTypeRow {
        id: i16,
        name: String,
    }

    let rows: Vec<TagTypeRow> = sqlx::query_as::<_, TagTypeRow>(
        "SELECT id, name FROM tag_types ORDER BY id",
    )
    .fetch_all(&state.db)
    .await?;

    let now = iso_now();
    let mut entries = String::new();

    for row in &rows {
        entries.push_str(&format!(
            r#"  <entry>
    <title>{name}</title>
    <link href="/opds/tags/{id}" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
    <updated>{now}</updated>
    <id>urn:ficnexus:tag-type:{id}</id>
    <summary>Browse tags of type: {name}</summary>
  </entry>
"#,
            name = html_escape(&row.name),
            id = row.id,
            now = now,
        ));
    }

    let body = build_feed(
        "FicNexus — Tags by Type",
        "urn:ficnexus:tag-types",
        &entries,
        &now,
        Some("/opds/tags"),
        FeedKind::Navigation,
        None,
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Navigation))
}

/// GET /opds/tags/{type_id} — Navigation feed listing canonical tags of that type
pub async fn tags_by_type(
    State(state): State<Arc<AppState>>,
    Path(type_id): Path<i16>,
) -> Result<impl IntoResponse, AppError> {
    #[derive(sqlx::FromRow)]
    struct TagRow {
        id: i32,
        name: String,
        cnt: Option<i64>,
    }

    let rows: Vec<TagRow> = sqlx::query_as::<_, TagRow>(
        r#"SELECT t.id, t.name, COUNT(*) as cnt
           FROM tags t
           JOIN fic_tags ft ON ft.tag_id = t.id
           WHERE t.tag_type_id = $1
           GROUP BY t.id, t.name
           ORDER BY cnt DESC
           LIMIT 100"#,
    )
    .bind(type_id)
    .fetch_all(&state.db)
    .await?;

    let now = iso_now();
    let type_name = sqlx::query_scalar::<_, String>("SELECT name FROM tag_types WHERE id = $1")
        .bind(type_id)
        .fetch_optional(&state.db)
        .await?
        .unwrap_or_else(|| format!("type-{}", type_id));

    let mut entries = String::new();
    for row in &rows {
        entries.push_str(&format!(
            r#"  <entry>
    <title>{name}{cnt}</title>
    <link href="/opds/tags/{type_id}/{encoded_name}" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <updated>{now}</updated>
    <id>urn:ficnexus:tag:{id}</id>
    <summary>{cnt} fics tagged with {name}</summary>
  </entry>
"#,
            name = html_escape(&row.name),
            cnt = if let Some(c) = row.cnt {
                format!(" ({})", c)
            } else {
                String::new()
            },
            type_id = type_id,
            encoded_name = html_escape(&row.name),
            now = now,
            id = row.id,
        ));
    }

    let body = build_feed(
        &format!("FicNexus — Tags: {}", type_name),
        &format!("urn:ficnexus:tag-type:{}", type_id),
        &entries,
        &now,
        Some(&format!("/opds/tags/{}", type_id)),
        FeedKind::Navigation,
        None,
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Navigation))
}

/// GET /opds/tags/{type_id}/{tag_name} — Acquisition feed of fics with that tag
pub async fn fics_by_tag(
    State(state): State<Arc<AppState>>,
    Path((type_id, tag_name)): Path<(i16, String)>,
    Query(params): Query<TagQuery>,
) -> Result<impl IntoResponse, AppError> {
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
        r#"SELECT fi.id, fi.title, fi.author, fi.description,
                  fi.fic_updated, fi.words, fi.chapters, fi.status
           FROM fic_info fi
           JOIN fic_tags ft ON ft.url_id = fi.id
           JOIN tags t ON t.id = ft.tag_id
           WHERE t.name = $1 AND t.tag_type_id = $2
           ORDER BY fi.fic_updated DESC
           LIMIT $3 OFFSET $4"#,
    )
    .bind(&tag_name)
    .bind(type_id)
    .bind(limit)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*)
           FROM fic_info fi
           JOIN fic_tags ft ON ft.url_id = fi.id
           JOIN tags t ON t.id = ft.tag_id
           WHERE t.name = $1 AND t.tag_type_id = $2"#,
    )
    .bind(&tag_name)
    .bind(type_id)
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
        base_path: format!("/opds/tags/{}/{}", type_id, urlencoding(&tag_name)),
        page: params.page(),
        per_page: params.per_page(),
        total: total.0,
    };

    let self_link = format!(
        "/opds/tags/{}/{}?page={}&per_page={}",
        type_id,
        urlencoding(&tag_name),
        params.page(),
        params.per_page()
    );

    let body = build_feed(
        &format!("FicNexus — Tag: {}", tag_name),
        &format!("urn:ficnexus:tag:{}-{}", type_id, tag_name),
        &entries,
        &now,
        Some(&self_link),
        FeedKind::Acquisition,
        Some(&pagi),
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Acquisition))
}

/// Minimal URL-encoding for tag names in paths (replaces spaces and special chars)
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
    fn test_tag_query_defaults() {
        let q = TagQuery {
            page: None,
            per_page: None,
        };
        assert_eq!(q.page(), 1);
        assert_eq!(q.per_page(), 50);
        assert_eq!(q.offset(), 0);
    }

    #[test]
    fn test_tag_query_custom() {
        let q = TagQuery {
            page: Some(2),
            per_page: Some(10),
        };
        assert_eq!(q.page(), 2);
        assert_eq!(q.per_page(), 10);
        assert_eq!(q.offset(), 10);
    }

    #[test]
    fn test_tag_query_clamp_low() {
        let q = TagQuery {
            page: Some(0),
            per_page: Some(0),
        };
        assert_eq!(q.page(), 1);
        assert_eq!(q.per_page(), 1);
    }

    #[test]
    fn test_tag_query_clamp_high() {
        let q = TagQuery {
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
        assert_eq!(urlencoding("Harry Potter"), "Harry%20Potter");
    }

    #[test]
    fn test_urlencoding_special_chars() {
        assert_eq!(urlencoding("a#b?c/d&e"), "a%23b%3Fc%2Fd%26e");
    }

    #[test]
    fn test_urlencoding_empty() {
        assert_eq!(urlencoding(""), "");
    }
}
