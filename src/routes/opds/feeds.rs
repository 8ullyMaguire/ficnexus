use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::server::AppState;

use super::{build_feed, fic_entry, iso_now, opds_response, FeedKind, PageParams};

/// Build a navigation entry for the root catalog.
fn nav_entry(title: &str, href: &str, id: &str, summary: &str, now: &str, base_url: Option<&str>) -> String {
    let abs = super::abs_url(href, base_url);
    format!(
        r#"  <entry>
    <title>{title}</title>
    <link href="{href}" type="application/atom+xml; profile=opds-catalog; kind=navigation" rel="subsection"/>
    <updated>{now}</updated>
    <id>{id}</id>
    <content type="text">{summary}</content>
  </entry>
"#,
        title = super::html_escape(title),
        href = super::html_escape(&abs),
        id = super::html_escape(id),
        summary = super::html_escape(summary),
        now = now,
    )
}

/// GET /opds — Root catalog with links to all sub-feeds
pub async fn root_catalog(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let now = iso_now();
    let base_url = state.config.opds_base_url.as_deref();
    let entries = format!(
        "{}{}{}{}{}{}",
        nav_entry(
            "Recent Fics",
            "/opds/new",
            "urn:ficnexus:catalog:new",
            "Recently added and updated fanfiction",
            &now,
            base_url,
        ),
        nav_entry(
            "Popular Fics",
            "/opds/popular",
            "urn:ficnexus:catalog:popular",
            "Popular fanfiction sorted by word count",
            &now,
            base_url,
        ),
        nav_entry(
            "Tags",
            "/opds/tags",
            "urn:ficnexus:catalog:tags",
            "Browse fanfiction by tags",
            &now,
            base_url,
        ),
        nav_entry(
            "Authors",
            "/opds/authors",
            "urn:ficnexus:catalog:authors",
            "Browse fanfiction by author",
            &now,
            base_url,
        ),
        nav_entry(
            "Recommendations",
            "/opds/recommendations/popular",
            "urn:ficnexus:catalog:recommendations",
            "Popular and personalised recommendations",
            &now,
            base_url,
        ),
        nav_entry(
            "Search",
            "/opds/search?q=",
            "urn:ficnexus:catalog:search",
            "Search fanfiction",
            &now,
            base_url,
        ),
    );

    let header = super::atom_xml_header(base_url);
    let self_link = super::abs_url("/opds", base_url);
    let search_link = super::abs_url("/opds/search", base_url);
    let body = format!(
        r#"{header}<id>urn:ficnexus:catalog</id>
  <title>FicNexus</title>
  <updated>{now}</updated>
  <author><name>FicNexus</name></author>
  <link href="{self_link}" rel="self" type="application/atom+xml; profile=opds-catalog; kind=navigation"/>
  <link href="{search_link}" rel="search" type="application/opensearchdescription+xml"/>
  <category term="fiction" label="Fanfiction"/>
{entries}</feed>"#,
        header = header,
        now = now,
        self_link = super::html_escape(&self_link),
        search_link = super::html_escape(&search_link),
        entries = entries,
    );
    opds_response(body, FeedKind::Navigation)
}

/// Fetch recent fics with pagination.
/// Uses a static SQL string to avoid dynamic SQL issues.
async fn fetch_recent(
    state: &Arc<AppState>,
    page: usize,
    per_page: usize,
) -> AppResult<(Vec<(String, String, String, String, String, i64, i32, String)>, i64)> {
    let offset = (page.saturating_sub(1)) * per_page;

    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, i32, String)>(
        r#"SELECT id, title, author, description,
                  COALESCE(to_char(fic_updated AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'), '1970-01-01T00:00:00Z'),
                  words, chapters, status
           FROM fic_info
           ORDER BY fic_updated DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_info")
        .fetch_one(&state.db)
        .await?;

    Ok((rows, total.0))
}

/// Fetch popular fics with pagination.
async fn fetch_popular(
    state: &Arc<AppState>,
    page: usize,
    per_page: usize,
) -> AppResult<(Vec<(String, String, String, String, String, i64, i32, String)>, i64)> {
    let offset = (page.saturating_sub(1)) * per_page;

    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, i32, String)>(
        r#"SELECT id, title, author, description,
                  COALESCE(to_char(fic_updated AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'), '1970-01-01T00:00:00Z'),
                  words, chapters, status
           FROM fic_info
           ORDER BY words DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_info")
        .fetch_one(&state.db)
        .await?;

    Ok((rows, total.0))
}

/// GET /opds/new — Recent fics with pagination
pub async fn recent_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PageParams>,
) -> Result<impl IntoResponse, AppError> {
    let (rows, total) = fetch_recent(&state, params.page(), params.per_page()).await?;

    let base_url = state.config.opds_base_url.as_deref();
    let ids: Vec<String> = rows.iter().map(|(id, ..)| id.clone()).collect();
    let acq_map = super::acquisition_links_many(&state.db, base_url, &ids).await;

    let entries: String = rows
        .iter()
        .map(|(id, t, a, d, u, w, c, s)| {
            let acq = acq_map.get(id).map(|v| v.as_slice()).unwrap_or(&[]);
            fic_entry(id, t, a, d, u, *w, *c, s, acq)
        })
        .collect();

    let now = iso_now();
    let pagi = super::PaginationInfo {
        base_path: "/opds/new".into(),
        page: params.page(),
        per_page: params.per_page(),
        total,
    };

    let self_link = format!(
        "/opds/new?page={}&per_page={}",
        params.page(),
        params.per_page()
    );

    let body = build_feed(
        "FicNexus — Recent Fics",
        "urn:ficnexus:feed:new",
        &entries,
        &now,
        Some(&self_link),
        FeedKind::Acquisition,
        Some(&pagi),
        base_url,
    );

    Ok(opds_response(body, FeedKind::Acquisition).into_response())
}

/// GET /opds/popular — Popular fics (by word count) with pagination
pub async fn popular_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PageParams>,
) -> Result<axum::response::Response, AppError> {
    let (rows, total) = fetch_popular(&state, params.page(), params.per_page()).await?;

    let base_url = state.config.opds_base_url.as_deref();
    let ids: Vec<String> = rows.iter().map(|(id, ..)| id.clone()).collect();
    let acq_map = super::acquisition_links_many(&state.db, base_url, &ids).await;

    let entries: String = rows
        .iter()
        .map(|(id, t, a, d, u, w, c, s)| {
            let acq = acq_map.get(id).map(|v| v.as_slice()).unwrap_or(&[]);
            fic_entry(id, t, a, d, u, *w, *c, s, acq)
        })
        .collect();

    let now = iso_now();
    let pagi = super::PaginationInfo {
        base_path: "/opds/popular".into(),
        page: params.page(),
        per_page: params.per_page(),
        total,
    };

    let self_link = format!(
        "/opds/popular?page={}&per_page={}",
        params.page(),
        params.per_page()
    );

    let body = build_feed(
        "FicNexus — Popular Fics",
        "urn:ficnexus:feed:popular",
        &entries,
        &now,
        Some(&self_link),
        FeedKind::Acquisition,
        Some(&pagi),
        base_url,
    );

    Ok(opds_response(body, FeedKind::Acquisition).into_response())
}
