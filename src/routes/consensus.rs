//! Unified curator consensus feed — modlog-style single view over every
//! community-vote surface in the archive.
//!
//! GET /api/curator/consensus?type=&status=&q= — logged-in users (>= 1) like modlog;
//! curator-level (>=5) items (flag/alias/merge detail) are always included since all
//! tables are public-read transparency by design.
//!
//! Aggregates, normalized to one item shape:
//!   content      — peer-voted body fixes        (curator_fix_proposals)
//!   metadata     — peer-voted metadata fixes    (curator_metadata_proposals)
//!   flag         — tag flags (open/resolved)    (tag_flags)
//!   alias        — tag aliases                  (tag_aliases)
//!   author_merge — author merge proposals       (author_merge_proposals)
//!   roadmap      — roadmap consensus clusters   (feature_clusters; formerly
//!                                                 admin-dashboard-only)

use axum::{Json, extract::State};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(serde::Deserialize)]
pub struct ConsensusQuery {
    pub kind: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
}

fn want(kind_filter: &Option<String>, kind: &str) -> bool {
    kind_filter.as_deref().unwrap_or("all") == "all" || kind_filter.as_deref() == Some(kind)
}

fn matches_q(q: &Option<String>, text: &str) -> bool {
    match q {
        None => true,
        Some(q) if q.is_empty() => true,
        Some(q) => text.to_lowercase().contains(&q.to_lowercase()),
    }
}

/// GET /api/curator/consensus
pub async fn consensus_feed(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<ConsensusQuery>,
) -> Result<Json<Value>, crate::error::AppError> {
    crate::modlog::require_logged_in(&auth)?;
    let kind = &params.kind;
    let status = params.status.clone().unwrap_or_default();
    let q = &params.q;

    let mut items: Vec<Value> = Vec::new();

    // ── content fixes ──────────────────────────────────────────────────
    if want(kind, "content") {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                i32,
                i32,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"SELECT id, url_id, reason, status, upvotes, downvotes, created_at
               FROM curator_fix_proposals
               WHERE ($1 = '' OR status = $1)
               ORDER BY created_at DESC LIMIT 200"#,
        )
        .bind(&status)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        for (id, url_id, reason, st, up, down, created) in rows {
            let title = format!("Body fix proposal for {}", url_id);
            if !matches_q(q, &format!("{} {}", title, reason)) {
                continue;
            }
            items.push(json!({
                "kind": "content", "id": id, "title": title, "detail": reason,
                "status": st, "score": up - down, "upvotes": up, "downvotes": down,
                "created_at": created.to_rfc3339(), "link": format!("/curator?proposal={}", id),
            }));
        }
    }

    // ── metadata fixes ─────────────────────────────────────────────────
    if want(kind, "metadata") {
        let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, i32, i32, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT p.id, COALESCE(w.canonical_title, w.title, CONCAT('work ', p.work_id)),
                      p.field, p.old_value, p.new_value, p.status, p.upvotes, p.downvotes, p.created_at
               FROM curator_metadata_proposals p LEFT JOIN works w ON w.id = p.work_id
               WHERE ($1 = '' OR p.status = $1)
               ORDER BY p.created_at DESC LIMIT 200"#,
        )
        .bind(&status)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        for (id, wtitle, field, old_v, new_v, st, up, down, created) in rows {
            let detail = format!("{}: {} → {}", field, old_v, new_v);
            if !matches_q(q, &format!("{} {}", wtitle, detail)) {
                continue;
            }
            items.push(json!({
                "kind": "metadata", "id": id,
                "title": format!("Metadata fix: {}", wtitle), "detail": detail,
                "status": st, "score": up - down, "upvotes": up, "downvotes": down,
                "created_at": created.to_rfc3339(), "link": "/curator",
            }));
        }
    }

    // ── tag flags ──────────────────────────────────────────────────────
    if want(kind, "flag") {
        let resolved_only = matches!(status.as_str(), "resolved" | "applied" | "closed");
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                Option<String>,
                bool,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"SELECT id, url_id, COALESCE(reason, ''), resolved, created_at
               FROM tag_flags WHERE resolved = $1
               ORDER BY created_at DESC LIMIT 200"#,
        )
        .bind(resolved_only)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        for (id, url_id, reason, res, created) in rows {
            let title = format!("Tag flag on {}", url_id);
            if !matches_q(
                q,
                &format!("{} {}", title, reason.clone().unwrap_or_default()),
            ) {
                continue;
            }
            items.push(json!({
                "kind": "flag", "id": id, "title": title,
                "detail": reason.unwrap_or_default(),
                "status": if res { "resolved" } else { "open" },
                "score": 0, "created_at": created.to_rfc3339(), "link": "/curator/flags",
            }));
        }
    }

    // ── tag aliases ────────────────────────────────────────────────────
    if want(kind, "alias") {
        let rows = sqlx::query_as::<_, (String, Option<String>, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT a.alias_name, t.name, a.created_at
               FROM tag_aliases a LEFT JOIN tags t ON t.id = a.canonical_tag_id
               ORDER BY a.created_at DESC LIMIT 200"#,
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        for (alias, canonical, created) in rows {
            let detail = format!("→ {}", canonical.clone().unwrap_or_else(|| "?".into()));
            if !matches_q(q, &format!("{} {}", alias, canonical.unwrap_or_default())) {
                continue;
            }
            items.push(json!({
                "kind": "alias", "id": alias.clone(),
                "title": format!("Tag alias: {}", alias), "detail": detail,
                "status": "active", "score": 0,
                "created_at": created.to_rfc3339(), "link": "/tags",
            }));
        }
    }

    // ── author merges ──────────────────────────────────────────────────
    if want(kind, "author_merge") {
        let rows =
            sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
                r#"SELECT amp.id, amp.source_author, ap.canonical_name, amp.status, amp.created_at
               FROM author_merge_proposals amp
               JOIN author_profiles ap ON ap.id = amp.target_profile_id
               WHERE ($1 = '' OR amp.status = $1)
               ORDER BY amp.created_at DESC LIMIT 200"#,
            )
            .bind(&status)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
        for (id, source_author, target, st, created) in rows {
            let detail = format!("{} → {}", source_author, target);
            if !matches_q(q, &detail) {
                continue;
            }
            items.push(json!({
                "kind": "author_merge", "id": id,
                "title": format!("Author merge: {}", source_author), "detail": detail,
                "status": st, "score": 0,
                "created_at": created.to_rfc3339(), "link": "/authors",
            }));
        }
    }

    // ── roadmap consensus clusters ─────────────────────────────────────
    if want(kind, "roadmap") {
        let rows = sqlx::query_as::<_, (i32, String, f64, i32, String, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT id, representative_text, elo_rating::float8, matches_played, status, created_at
               FROM feature_clusters
               WHERE ($1 = '' OR status = $1)
               ORDER BY elo_rating DESC LIMIT 200"#,
        )
        .bind(&status)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        for (id, text, elo, matches_played, st, created) in rows {
            if !matches_q(q, &text) {
                continue;
            }
            items.push(json!({
                "kind": "roadmap", "id": id,
                "title": format!("Roadmap cluster #{}", id),
                "detail": text,
                "status": st, "score": elo as i64,
                "elo": elo, "matches_played": matches_played,
                "created_at": created.to_rfc3339(), "link": "/roadmap",
            }));
        }
    }

    items.sort_by(|a, b| {
        b.get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(a.get("created_at").and_then(|v| v.as_str()).unwrap_or(""))
    });

    // Per-kind counts (post-filter) so the UI can render chip counters.
    let mut counts = std::collections::BTreeMap::new();
    for it in &items {
        *counts
            .entry(
                it.get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string(),
            )
            .or_insert(0i64) += 1;
    }

    Ok(Json(json!({
        "err": 0,
        "items": items,
        "total": items.len(),
        "counts": counts,
    })))
}
