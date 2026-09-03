//! Curator admin list for marginalia (per-passage forum discussions).
//!
//! GET /api/curator/marginalia?page=&limit= — role >= 5 like other curator
//! endpoints. Returns recent marginalia topics joined with work titles,
//! topic slugs and reply counts so curators can review passage discussions
//! without digging through the forum.

use axum::{
    extract::{Query, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(serde::Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// Excerpt for the admin table: first 60 chars of the stored passage text,
/// or the first 12 chars of the passage hash when no text was stored.
fn excerpt(passage_text: Option<&str>, passage_hash: &str) -> String {
    match passage_text.filter(|t| !t.trim().is_empty()) {
        Some(text) => {
            let chars: Vec<char> = text.chars().take(61).collect();
            if chars.len() > 60 {
                chars[..60].iter().collect::<String>() + "…"
            } else {
                chars.iter().collect()
            }
        }
        None => passage_hash.chars().take(12).collect(),
    }
}

/// GET /api/curator/marginalia
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, AppError> {
    if auth.role < 5 {
        return Err(AppError::Forbidden(
            "Level 5 required to view marginalia admin list".to_string(),
        ));
    }

    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let page = params.page.unwrap_or(0).max(0);
    let offset = page * limit;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM marginalia")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let rows: Vec<(
        i64,               // m.id
        String,            // work title
        Option<String>,    // url_id (fic_info id)
        i32,               // chapter_index
        Option<String>,    // passage_text (nullable)
        String,            // passage_hash
        i64,               // topic_id
        Option<String>,    // topic_slug
        i64,               // reply_count
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        r#"SELECT m.id,
                  COALESCE(NULLIF(w.canonical_title, ''), CONCAT('work ', m.work_id)) AS work_title,
                  (SELECT fi.id FROM fic_info fi WHERE fi.work_id = m.work_id LIMIT 1) AS url_id,
                  m.chapter_index,
                  m.passage_text,
                  m.passage_hash,
                  m.topic_id,
                  t.topic_slug,
                  GREATEST(
                      (SELECT COUNT(*) FROM forum_posts p WHERE p.topic_id = m.topic_id AND p.deleted_at IS NULL) - 1,
                      0
                  ) AS reply_count,
                  m.created_at
           FROM marginalia m
           JOIN works w ON w.id = m.work_id
           JOIN forum_topics t ON t.id = m.topic_id
           ORDER BY m.created_at DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let items: Vec<Value> = rows
        .iter()
        .map(
            |(id, title, url_id, chapter_index, passage_text, passage_hash, topic_id, topic_slug, replies, created)| {
                json!({
                    "id": id,
                    "work_title": title,
                    "url_id": url_id,
                    "chapter_index": chapter_index,
                    // Chapter label is 1-based in the UI.
                    "chapter": chapter_index + 1,
                    "excerpt": excerpt(passage_text.as_deref(), passage_hash),
                    "has_passage_text": passage_text.as_deref().map(|t| !t.trim().is_empty()).unwrap_or(false),
                    "topic_id": topic_id,
                    "topic_slug": topic_slug,
                    "reply_count": replies,
                    "created_at": created.to_rfc3339(),
                    "read_url": url_id.as_ref().map(|u| format!(
                        "/read/{}/{}#marginalia", u, chapter_index)),
                    "topic_url": match topic_slug {
                        Some(slug) => format!("/forum/board/{}.{}", slug, topic_id),
                        None => format!("/forum/board/discussion-{}.{}", topic_id, topic_id),
                    },
                })
            },
        )
        .collect();

    Ok(Json(json!({
        "err": 0,
        "items": items,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

#[cfg(test)]
mod tests {
    use super::excerpt;

    #[test]
    fn excerpt_uses_passage_text_when_present() {
        let long = "a".repeat(120);
        let out = excerpt(Some(&long), "hashhashhash");
        assert!(out.starts_with("aaaa"));
        assert!(out.ends_with('…'));
        assert_eq!(out.chars().count(), 61); // 60 + ellipsis
    }

    #[test]
    fn excerpt_falls_back_to_hash_prefix() {
        assert_eq!(excerpt(None, "abcdef1234567890"), "abcdef123456");
        assert_eq!(excerpt(Some("   "), "xyz"), "xyz");
    }
}
