use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

use super::{FeedKind, build_feed, fic_entry, html_escape, iso_now, opds_response};

/// Query params for shelf authentication
#[derive(Debug, Deserialize)]
pub struct ShelfToken {
    pub token: Option<String>,
}

/// Query params for shelf listing
#[derive(Debug, Deserialize)]
pub struct ShelfQuery {
    pub token: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl ShelfQuery {
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

/// Verify the shelf token against config.
/// Returns `Ok(())` when allowed, or an OPDS authentication-required feed
/// (200 XML, per the OPDS catalog conventions — not a JSON 404) when the
/// token is missing/invalid and a token is configured.
fn verify_token(
    state: &Arc<AppState>,
    token: Option<&str>,
) -> Result<(), ([(header::HeaderName, &'static str); 1], String)> {
    let expected = state.config.opds_shelf_token.as_str();
    match (token, expected) {
        (Some(t), expected) if !expected.is_empty() && t == expected => Ok(()),
        (_, expected) if expected.is_empty() => {
            // No token configured — allow access
            Ok(())
        }
        _ => Err(auth_required_feed(state)),
    }
}

/// OPDS authentication-required feed: a 200 XML navigation catalog with
/// rel="self" plus an entry pointing at the login/catalog URL. Clients
/// (OPDS readers) render this as "authentication required" instead of
/// failing on a JSON 404.
fn auth_required_feed(state: &Arc<AppState>) -> ([(header::HeaderName, &'static str); 1], String) {
    let now = iso_now();
    let login = format!("/opds/shelves?token=YOUR_TOKEN");
    let entries = format!(
        r#"  <entry>
    <title>Authentication required</title>
    <id>urn:ficnexus:auth-required</id>
    <updated>{now}</updated>
    <summary>This catalog requires a shelf token. Append ?token=YOUR_TOKEN to the catalog URL to view your shelves. (The admin can set OPDS_SHELF_TOKEN.)</summary>
    <link href="{login}" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
  </entry>
"#,
        now = now,
        login = html_escape(&login),
    );
    let body = build_feed(
        "FicNexus — Shelves (authentication required)",
        "urn:ficnexus:shelves:auth",
        &entries,
        &now,
        Some("/opds/shelves"),
        FeedKind::Navigation,
        None,
        state.config.opds_base_url.as_deref(),
    );
    opds_response(body, FeedKind::Navigation)
}

/// GET /opds/shelves?token=XXX — List of shelves for the authenticated user
pub async fn shelf_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ShelfToken>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(resp) = verify_token(&state, query.token.as_deref()) {
        return Ok(resp);
    }

    #[derive(sqlx::FromRow)]
    struct ShelfRow {
        id: i32,
        name: String,
        description: Option<String>,
        created_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let rows: Vec<ShelfRow> = sqlx::query_as::<_, ShelfRow>(
        "SELECT id, name, description, created_at FROM opds_shelves ORDER BY id",
    )
    .fetch_all(&state.db)
    .await?;

    let now = iso_now();
    let mut entries = String::new();

    for row in &rows {
        let desc = row.description.as_deref().unwrap_or("A reading shelf");
        let created = row
            .created_at
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_else(|| now.clone());

        entries.push_str(&format!(
            r#"  <entry>
    <title>{name}</title>
    <link href="/opds/shelf/{id}?token={token}" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <updated>{created}</updated>
    <id>urn:ficnexus:shelf:{id}</id>
    <summary>{desc}</summary>
  </entry>
"#,
            name = html_escape(&row.name),
            id = row.id,
            token = query.token.as_deref().unwrap_or(""),
            created = created,
            desc = html_escape(desc),
        ));
    }

    let body = build_feed(
        "FicNexus — Shelves",
        "urn:ficnexus:shelves",
        &entries,
        &now,
        Some("/opds/shelves"),
        FeedKind::Navigation,
        None,
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Navigation))
}

/// GET /opds/shelf/{shelf_id}?token=XXX — Acquisition feed for a shelf
pub async fn shelf_contents(
    State(state): State<Arc<AppState>>,
    Path(shelf_id): Path<i32>,
    Query(query): Query<ShelfQuery>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(resp) = verify_token(&state, query.token.as_deref()) {
        return Ok(resp);
    }

    // Verify shelf exists
    let shelf_name: Option<String> =
        sqlx::query_scalar("SELECT name FROM opds_shelves WHERE id = $1")
            .bind(shelf_id)
            .fetch_optional(&state.db)
            .await?;

    let shelf_name = match shelf_name {
        Some(name) => name,
        None => return Err(AppError::NotFound(format!("Shelf {} not found", shelf_id))),
    };

    let offset = query.offset();
    let limit = query.per_page() as i64;

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
           FROM opds_shelf_items si
           JOIN fic_info fi ON fi.id = si.url_id
           WHERE si.shelf_id = $1
           ORDER BY si.added_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(shelf_id)
    .bind(limit)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM opds_shelf_items WHERE shelf_id = $1")
        .bind(shelf_id)
        .fetch_one(&state.db)
        .await?;

    let now = iso_now();
    let base_url = state.config.opds_base_url.as_deref();
    let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    let acq_map = super::acquisition_links_many(&state.db, base_url, &ids).await;
    let mut entries = String::new();

    for row in &rows {
        let updated = row.fic_updated.format("%Y-%m-%dT%H:%M:%SZ").to_string();
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
        base_path: format!("/opds/shelf/{}", shelf_id),
        page: query.page(),
        per_page: query.per_page(),
        total: total.0,
    };

    let self_link = format!(
        "/opds/shelf/{}?page={}&per_page={}",
        shelf_id,
        query.page(),
        query.per_page(),
    );

    let body = build_feed(
        &format!("FicNexus — Shelf: {}", shelf_name),
        &format!("urn:ficnexus:shelf:{}", shelf_id),
        &entries,
        &now,
        Some(&self_link),
        FeedKind::Acquisition,
        Some(&pagi),
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Acquisition))
}
