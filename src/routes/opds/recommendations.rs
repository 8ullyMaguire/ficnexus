use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

use super::{
    build_feed, fic_entry, html_escape, iso_now, opds_response, FeedKind,
};

/// Query params for recommendation feeds
#[derive(Debug, Deserialize)]
pub struct RecQuery {
    pub url_id: Option<String>,
    pub n: Option<usize>,
}

/// GET /opds/recommendations/popular — Acquisition feed of popular fics
/// (proxy: highest word count)
pub async fn popular_recommendations(
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Response, AppError> {
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
           ORDER BY words DESC
           LIMIT 50"#,
    )
    .fetch_all(&state.db)
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

    let body = build_feed(
        "FicNexus — Popular Recommendations",
        "urn:ficnexus:recommendations:popular",
        &entries,
        &now,
        Some("/opds/recommendations/popular"),
        FeedKind::Acquisition,
        None,
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Acquisition).into_response())
}

/// GET /opds/recommendations?url_id=XXX — Recommendations for a specific fic
pub async fn fic_recommendations(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecQuery>,
) -> Result<axum::response::Response, AppError> {
    let url_id = match params.url_id.as_deref() {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => {
            // No url_id provided, serve the popular recommendations instead
            return popular_recommendations(State(state)).await;
        }
    };

    let limit = params.n.unwrap_or(20).min(50) as i64;

    #[derive(sqlx::FromRow)]
    struct RecRow {
        recommended_url_id: String,
        #[allow(dead_code)]
        score: f32,
        title: String,
        author: String,
        words: i64,
        chapters: i32,
        status: String,
        description: String,
        fic_updated: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<RecRow> = sqlx::query_as::<_, RecRow>(
        r#"SELECT pr.recommended_url_id, pr.score,
                  fi.title, fi.author, fi.words, fi.chapters,
                  fi.status, fi.description, fi.fic_updated
           FROM precomputed_recommendations pr
           JOIN fic_info fi ON fi.id = pr.recommended_url_id
           WHERE pr.url_id = $1
           ORDER BY pr.score DESC
           LIMIT $2"#,
    )
    .bind(&url_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let fic_title: Option<String> = sqlx::query_scalar(
        "SELECT title FROM fic_info WHERE id = $1",
    )
    .bind(&url_id)
    .fetch_optional(&state.db)
    .await?;

    let feed_title = match fic_title {
        Some(ref t) => format!("FicNexus — Recommendations for: {}", t),
        None => "FicNexus — Recommendations".to_string(),
    };

    let now = iso_now();
    let base_url2 = state.config.opds_base_url.as_deref();
    let ids2: Vec<String> = rows.iter().map(|r| r.recommended_url_id.clone()).collect();
    let acq_map2 = super::acquisition_links_many(&state.db, base_url2, &ids2).await;
    let mut entries = String::new();

    for row in &rows {
        let updated = row
            .fic_updated
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let acq = acq_map2.get(&row.recommended_url_id).map(|v| v.as_slice()).unwrap_or(&[]);
        entries.push_str(&fic_entry(
            &row.recommended_url_id,
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

    let body = build_feed(
        &feed_title,
        &format!("urn:ficnexus:recommendations:{}", html_escape(&url_id)),
        &entries,
        &now,
        Some(&format!("/opds/recommendations?url_id={}", html_escape(&url_id))),
        FeedKind::Acquisition,
        None,
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Acquisition).into_response())
}
