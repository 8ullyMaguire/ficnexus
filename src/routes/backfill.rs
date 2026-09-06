//! Admin backfill: recover tags + merged metadata for already-scraped works.
//!
//! Strategy (disk-first, network-last):
//!   1. If a meta.v{N}.json exists and has non-empty merged_tags -> zero-network
//!      path: re-persist those tags and update fic_info if merged.updated is newer.
//!   2. If no blob or empty tags -> network path: Wayback CDX -> snapshot HTML ->
//!      lookup_from_html + extract_tags_from_html -> persist tags + write new blob.
//!
//! Both paths write a versioned meta_store blob so future sweeps can re-merge
//! with no network calls (zero-network path always wins).

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::config::Config;
use crate::db::queries;
use crate::error::AppError;
use crate::meta_store;
use crate::scrape::wayback::wayback_eligible;
use crate::scrape::ExtractedTag;
use crate::server::AppState;

/// Require admin auth (copied from forum.rs).
fn require_admin(auth: &crate::routes::auth::AuthUser) -> Result<(), AppError> {
    if auth.user_id.is_none() {
        return Err(AppError::Unauthorized("login required".into()));
    }
    if auth.level < 10 {
        return Err(AppError::Forbidden("admin required".into()));
    }
    Ok(())
}

// ── Request / response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BackfillQuery {
    pub source: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BackfillBody {
    pub source: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BackfillResult {
    pub url_id: String,
    pub title: String,
    pub outcome: String,       // "tags_added" | "no_data" | "failed"
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BackfillResponse {
    pub err: i32,
    pub scanned: i64,
    pub tags_added: i64,
    pub core_updated: i64,
    pub failed: i64,
    pub results: Vec<BackfillResult>,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Persist extracted tags for a url_id (mirrors export.rs persist_extracted_tags).
async fn persist_tags(db: &PgPool, url_id: &str, tags: &[ExtractedTag]) -> usize {
    let mut persisted = 0usize;
    for tag in tags {
        if let Ok(resolution) =
            crate::tags::resolve::resolve_tag(db, &tag.name, tag.tag_type_id).await
        {
            if queries::upsert_fic_tag(
                db,
                url_id,
                resolution.tag_id,
                &std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                tag.score,
            )
            .await
            .is_ok()
            {
                persisted += 1;
            }
        }
    }
    persisted
}

/// Attempt zero-network path from a saved meta blob.
/// Returns (tags_added, core_updated, outcome_str, detail).
async fn zero_network(
    db: &PgPool,
    config: &Config,
    url_id: &str,
    blob: &meta_store::MetaBlob,
) -> (i64, i64, String, Option<String>) {
    let mut tags_added = 0i64;
    let mut core_updated = 0i64;

    // Persist the saved tags.
    if !blob.merged_tags.is_empty() {
        tags_added = persist_tags(db, url_id, &blob.merged_tags).await as i64;
    }

    // Check if merged metadata is newer than stored fic_info.
    if let Some(merged) = &blob.merged {
        if let Ok(Some(stored)) = queries::get_fic_info(db, url_id).await {
            let stored_updated = stored
                .fic_updated
                .timestamp_millis();
            let merged_updated = merged.updated;
            if merged_updated > stored_updated {
                let fic_row = crate::db::models::FicInfo {
                    id: url_id.to_string(),
                    created: None,
                    updated: None,
                    title: merged.title.clone(),
                    author: merged.author.clone(),
                    author_url: Some(merged.author_url.clone()),
                    author_local_id: Some(merged.author_local_id.clone()),
                    chapters: merged.chapters,
                    words: merged.words,
                    description: merged.desc.clone(),
                    fic_created: chrono::DateTime::from_timestamp_millis(merged.published)
                        .unwrap_or_default(),
                    fic_updated: chrono::DateTime::from_timestamp_millis(merged.updated)
                        .unwrap_or_default(),
                    status: merged.status.clone(),
                    source: merged.source.clone(),
                    extra_meta: merged.extra_meta.clone(),
                    raw_extended_meta: merged.raw_extended_meta.clone(),
                    source_id: Some(merged.source_id),
                    author_id: Some(merged.author_id),
                    content_hash: merged.content_hash.clone(),
                    work_id: None,
                };
                if queries::upsert_fic_info(db, &fic_row).await.is_ok() {
                    core_updated = 1;
                }
            }
        }
    }

    // Write a new blob version (so the sweep advances the version counter).
    let new_blob = meta_store::MetaBlob {
        url_id: url_id.to_string(),
        source_url: blob.source_url.clone(),
        fetched_at_ms: chrono::Utc::now().timestamp_millis(),
        native: blob.native.clone(),
        fichub: blob.fichub.clone(),
        wayback: blob.wayback.clone(),
        merged: blob.merged.clone(),
        merged_tags: blob.merged_tags.clone(),
        partial: blob.partial,
        error: None,
    };
    let _ = meta_store::write_pass(config, &new_blob);

    if tags_added > 0 {
        (tags_added, core_updated, "tags_added".into(), None)
    } else {
        (0, core_updated, "no_data".into(), Some("blob had no tags to persist".into()))
    }
}

/// Attempt network path: Wayback CDX -> snapshot HTML -> parse + tags.
/// Returns (tags_added, core_updated, outcome_str, detail).
async fn network_path(
    state: &Arc<AppState>,
    url_id: &str,
    source_url: &str,
    scraper: &dyn fanfic_scrapers::SiteScraper,
) -> (i64, i64, String, Option<String>) {
    // Fetch the snapshot HTML.
    let html = match wayback_snap_for(&Arc::clone(&state), &source_url).await {
        Some(html) => html,
        None => {
            return (
                0,
                0,
                "failed".into(),
                Some("wayback unavailable or no snapshot found".into()),
            )
        }
    };

    // Parse metadata and extract tags from the HTML.
    let m = match scraper.lookup_from_html(&html, source_url).await {
        Ok(m) => m,
        Err(e) => {
            return (
                0,
                0,
                "failed".into(),
                Some(format!("wayback parse failed: {}", e)),
            )
        }
    };

    let wayback_tags: Vec<ExtractedTag> = match scraper.extract_tags_from_html(&html).await {
        Ok(t) => t.into_iter()
            .map(|t| crate::scrape::ExtractedTag {
                name: t.name,
                tag_type_id: t.tag_type_id as i16,
                score: t.score as i16,
            })
            .collect(),
        Err(_) => vec![],
    };

    // Persist tags.
    let tags_added = persist_tags(&state.db, url_id, &wayback_tags).await as i64;

    // Upsert fic_info.
    let fic_row = crate::db::models::FicInfo {
        id: url_id.to_string(),
        created: None,
        updated: None,
        title: m.title.clone(),
        author: m.author.clone(),
        author_url: Some(m.author_url.clone()),
        author_local_id: Some(m.author_local_id.clone()),
        chapters: m.chapters,
        words: m.words,
        description: m.desc.clone(),
        fic_created: chrono::DateTime::from_timestamp_millis(m.published).unwrap_or_default(),
        fic_updated: chrono::DateTime::from_timestamp_millis(m.updated).unwrap_or_default(),
        status: m.status.clone(),
        source: m.source.clone(),
        extra_meta: m.extra_meta.clone(),
        raw_extended_meta: m.raw_extended_meta.clone(),
        source_id: Some(m.source_id),
        author_id: Some(m.author_id),
        content_hash: m.content_hash.clone(),
        work_id: None,
    };
    let core_updated = if queries::upsert_fic_info(&state.db, &fic_row).await.is_ok() {
        1
    } else {
        0
    };

    // Write the new blob (wayback leg with no snapshot metadata).
    let blob = meta_store::MetaBlob {
        url_id: url_id.to_string(),
        source_url: source_url.to_string(),
        fetched_at_ms: chrono::Utc::now().timestamp_millis(),
        native: None,
        fichub: None,
        wayback: Some(meta_store::WaybackLeg {
            meta: Some(m),
            tags: wayback_tags,
            snapshot_url: None,
            snapshot_at_ms: 0,
        }),
        merged: None,
        merged_tags: vec![],
        partial: true,
        error: None,
    };
    let _ = meta_store::write_pass(&state.config, &blob);

    (
        tags_added,
        core_updated,
        if tags_added > 0 { "tags_added" } else { "no_data" }.into(),
        None,
    )
}

// Fetch a Wayback Machine snapshot for source_url.
// Uses the configured WaybackService (respects CDX rate limiter).
// Returns the cleaned Wayback HTML, or None.
async fn wayback_snap_for(
    state: &Arc<AppState>,
    source_url: &str,
) -> Option<String> {
    state.wayback.fetch_snapshot(&state.http_client, source_url).await
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/admin/backfill-tags?source=&limit=25&offset=0 — dry run (no writes).
pub async fn backfill_dry_run(
    State(state): State<Arc<AppState>>,
    auth: crate::routes::auth::AuthUser,
    Query(params): Query<BackfillQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    let limit = params.limit.unwrap_or(25).max(1).min(100);
    let offset = params.offset.unwrap_or(0).max(0);

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"SELECT fi.id, fi.source, COALESCE(fi.title, fi.id) AS title
           FROM fic_info fi
           WHERE NOT EXISTS (SELECT 1 FROM fic_tags ft WHERE ft.url_id = fi.id)
             AND ($1::text IS NULL OR fi.source = $1)
           ORDER BY fi.updated DESC NULLS LAST
           LIMIT $2 OFFSET $3"#,
    )
    .bind(&params.source)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let results: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(url_id, source, title)| {
            json!({
                "url_id": url_id,
                "source": source,
                "title": title,
                "outcome": "pending",
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "scanned": results.len() as i64,
        "results": results,
    })))
}

/// POST /api/admin/backfill-tags — real run. Body { source?: String, limit?: i64 }.
pub async fn backfill_run(
    State(state): State<Arc<AppState>>,
    auth: crate::routes::auth::AuthUser,
    Json(body): Json<BackfillBody>,
) -> Result<Json<BackfillResponse>, AppError> {
    require_admin(&auth)?;

    // Kill-switch.
    if !state.config.tag_topup_enabled {
        return Err(AppError::Forbidden(
            "TAG_TOPUP_ENABLED is false".into(),
        ));
    }

    let limit = body.limit.unwrap_or(25).max(1).min(100);

    // Kill-switch: wayback disabled.
    if !state.wayback.config.enabled {
        return Ok(Json(BackfillResponse {
            err: 0,
            scanned: 0,
            tags_added: 0,
            core_updated: 0,
            failed: 0,
            results: vec![],
        }));
    }

    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT fi.id, COALESCE(fi.title, fi.id) AS title
           FROM fic_info fi
           WHERE NOT EXISTS (SELECT 1 FROM fic_tags ft WHERE ft.url_id = fi.id)
             AND ($1::text IS NULL OR fi.source = $1)
           ORDER BY fi.updated DESC NULLS LAST
           LIMIT $2"#,
    )
    .bind(&body.source)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let mut tags_added_total = 0i64;
    let mut core_updated_total = 0i64;
    let mut failed_total = 0i64;
    let mut results = Vec::new();

    for (url_id, title) in rows {
        // Get the source URL from fic_info.
        let source_url: String = {
            let fi = queries::get_fic_info(&state.db, &url_id)
                .await
                .ok()
                .flatten();
            fi.map(|f| f.source).unwrap_or_else(|| url_id.clone())
        };

        // Disk-first: check for existing meta blob.
        if let Some(blob) = meta_store::latest_meta(&state.config, &url_id) {
            if !blob.merged_tags.is_empty() {
                let (ta, cu, outcome, detail) =
                    zero_network(&state.db, &state.config, &url_id, &blob).await;
                tags_added_total += ta;
                core_updated_total += cu;
                results.push(BackfillResult {
                    url_id,
                    title,
                    outcome,
                    detail,
                });
                continue;
            }
        }

        // Network path: find scraper for this source.
        let scraper = state.scraper_registry.find_specific_or_fff(&source_url);
        match scraper {
            Some(s) if source_url.parse::<url::Url>().is_ok_and(|u| wayback_eligible(&u)) => {
                let (ta, cu, outcome, detail) = network_path(
                    &state,
                    &url_id,
                    &source_url,
                    s.as_ref(),
                )
                .await;
                tags_added_total += ta;
                core_updated_total += cu;
                if outcome == "failed" {
                    failed_total += 1;
                }
                results.push(BackfillResult {
                    url_id,
                    title,
                    outcome,
                    detail,
                });
            }
            _ => {
                failed_total += 1;
                results.push(BackfillResult {
                    url_id,
                    title,
                    outcome: "failed".into(),
                    detail: Some("no scraper or wayback not eligible".into()),
                });
            }
        }
    }

    Ok(Json(BackfillResponse {
        err: 0,
        scanned: results.len() as i64,
        tags_added: tags_added_total,
        core_updated: core_updated_total,
        failed: failed_total,
        results,
    }))
}
