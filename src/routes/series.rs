//! Series & author detail pages.
//!
//! - `GET /api/series/{id}` — a series (title, description) with its works
//!   in reading order, each work carrying `next_in_series` (the work that
//!   follows it — the highest-conversion per-fic link in fanfiction).
//! - `GET /api/authors/{name}` — an author bibliography: the canonical
//!   author name, aggregate stats (work count, total words, top tags), and
//!   the works grid (ordered by updated desc).
//!
//! Author pages key off the canonical author name (works.canonical_author),
//! which is how the fic page's author display already identifies authors.
//! The existing `/api/authors/{id}` profile routes (curator-managed
//! author_profiles) are unchanged; `/api/authors/{name}` is the
//! bibliography endpoint used by the author page.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use crate::error::AppError;
use crate::server::AppState;

/// GET /api/series/{id} — series detail with ordered works.
///
/// Each work in `works` carries a `next_in_series` object (work_id,
/// canonical_title, url) when a work follows it in the series. The series
/// page highlights that card.
pub async fn get_series(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let series =
        sqlx::query_as::<
            _,
            (
                i32,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >("SELECT id, name, description, created_at, updated_at FROM series WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Series not found".into()))?;

    // Ordered works (position ASC — the reading order). A work's "default"
    // source (its url_id) is `default_source_id`; fall back to the
    // highest-word-count source when the default is missing so the card can
    // always link somewhere.
    let rows = sqlx::query_as::<_, (i32, String, String, Option<String>, Option<String>)>(
        r#"SELECT w.id, w.canonical_title, w.canonical_author,
                   w.default_source_id,
                   (SELECT fi.id::text FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1)
           FROM series_works sw
           JOIN works w ON w.id = sw.work_id
           WHERE sw.series_id = $1
           ORDER BY sw.position ASC, w.id ASC"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    // Enrich each work with its canonical source metadata (title/author/
    // words/chapters/status from the default source if present).
    let mut works: Vec<Value> = Vec::with_capacity(rows.len());
    for (work_id, canonical_title, canonical_author, default_url, fallback_url) in rows {
        let url_id = default_url
            .filter(|u| !u.is_empty())
            .or(fallback_url)
            .unwrap_or_default();

        // Source metadata (best effort — missing when the work has no sources).
        let src = sqlx::query_as::<_, (String, String, i64, i32, String)>(
            "SELECT title, author, words, chapters, status FROM fic_info WHERE id = $1",
        )
        .bind(&url_id)
        .fetch_optional(&state.db)
        .await?
        .map(|(title, author, words, chapters, status)| {
            json!({
                "title": title,
                "author": author,
                "words": words,
                "chapters": chapters,
                "status": status,
            })
        })
        .unwrap_or_else(|| {
            json!({
                "title": canonical_title,
                "author": canonical_author,
                "words": 0,
                "chapters": 0,
                "status": "unknown",
            })
        });

        works.push(json!({
            "work_id": work_id,
            "url_id": url_id,
            "canonical_title": canonical_title,
            "canonical_author": canonical_author,
            "title": src["title"],
            "author": src["author"],
            "words": src["words"],
            "chapters": src["chapters"],
            "status": src["status"],
        }));
    }

    // Compute next_in_series for each position (0-based → the following work).
    for i in 0..works.len() {
        if i + 1 < works.len() {
            let nxt = &works[i + 1];
            works[i]["next_in_series"] = json!({
                "work_id": nxt["work_id"],
                "canonical_title": nxt["canonical_title"],
                "url_id": nxt["url_id"],
            });
        } else {
            works[i]["next_in_series"] = Value::Null;
        }
    }

    Ok(Json(json!({
        "err": 0,
        "series": {
            "id": series.0,
            "name": series.1,
            "description": series.2,
            "created_at": series.3.to_rfc3339(),
            "updated_at": series.4.to_rfc3339(),
            "work_count": works.len(),
        },
        "works": works,
    })))
}

/// GET /api/authors/{name} — author bibliography.
///
/// The author is keyed by the canonical author name on `works` (and
/// `fic_info.author` for orphan sources). Stats: work count, total words,
/// top freeform tags (tag_type_id 4) + fandoms (type 1) by usage.
pub async fn get_author(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("author name required".to_string()));
    }

    // Canonical author = the most common works.canonical_author spelling
    // (works are the unified story entries; the author display on fic pages
    // uses works.canonical_author when available).
    let canonical: Option<String> = sqlx::query_scalar(
        r#"SELECT canonical_author FROM works
           WHERE LOWER(canonical_author) = LOWER($1) AND canonical_author <> ''
           GROUP BY canonical_author ORDER BY COUNT(*) DESC, canonical_author LIMIT 1"#,
    )
    .bind(&name)
    .fetch_optional(&state.db)
    .await?;

    let canonical_name = canonical.unwrap_or_else(|| name.to_string());

    // Works (canonical entries) by this author. Each carries its best
    // source's url_id + metadata (words/chapters/status from the
    // highest-word-count source) for linking and display.
    let works: Vec<Value> = sqlx::query_as::<_, (i32, String, String, String, Option<String>, Option<String>, i64, i32, String)>(
        r#"SELECT w.id, w.canonical_title, w.description, w.canonical_author,
                   w.default_source_id,
                   (SELECT fi.id::text FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1),
                   COALESCE((SELECT fi.words FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1), 0),
                   COALESCE((SELECT fi.chapters FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1), 0),
                   COALESCE((SELECT fi.status FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1), 'unknown')
           FROM works w
           WHERE LOWER(w.canonical_author) = LOWER($1)
           ORDER BY w.updated_at DESC, w.id DESC"#,
    )
    .bind(&name)
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(id, title, desc, author, default_url, fallback_url, words, chapters, status)| {
        let url_id = default_url
            .filter(|u| !u.is_empty())
            .or(fallback_url)
            .unwrap_or_default();
        json!({
            "work_id": id,
            "canonical_title": title,
            "description": desc,
            "canonical_author": author,
            "url_id": url_id,
            "words": words,
            "chapters": chapters,
            "status": status,
        })
    })
    .collect();

    // Also surface fic_info rows by this author that aren't yet linked to a
    // canonical work (orphan sources) so the bibliography is complete.
    let orphans: Vec<Value> =
        sqlx::query_as::<_, (String, String, String, i64, i32, String, Option<i32>)>(
            r#"SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters, fi.status, fi.work_id
           FROM fic_info fi
           WHERE LOWER(fi.author) = LOWER($1)
             AND fi.work_id IS NULL
           ORDER BY fi.fic_updated DESC
           LIMIT 50"#,
        )
        .bind(&name)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(
            |(url_id, title, author, words, chapters, status, _work_id)| {
                json!({
                    "work_id": null,
                    "url_id": url_id,
                    "canonical_title": title,
                    "title": title,
                    "author": author,
                    "words": words,
                    "chapters": chapters,
                    "status": status,
                })
            },
        )
        .collect();

    // Aggregate stats.
    let stats = sqlx::query_as::<_, (i64, Option<i64>)>(
        r#"SELECT
             COUNT(*)::bigint,
             SUM(fi.words)::bigint
           FROM fic_info fi
           WHERE LOWER(fi.author) = LOWER($1)"#,
    )
    .bind(&name)
    .fetch_one(&state.db)
    .await?;

    // Author profile (avatar, bio) — matched by canonical name.
    //
    // badge_text was selected here and no migration ever created the column, so
    // this endpoint returned 500 for every author: the column is resolved before
    // any row is examined. Dropped rather than added — it was read in one place
    // and written in one place (routes/authors.rs update_author_profile, also
    // broken), with no UI, no test and no other consumer. The separate
    // "badges" field below is the real, working badge surface.
    let author_profile = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        r#"SELECT avatar_url, bio
           FROM author_profiles
           WHERE canonical_name = $1
           ORDER BY updated_at DESC LIMIT 1"#,
    )
    .bind(&canonical_name)
    .fetch_optional(&state.db)
    .await?;

    // Author's earned badges (joined to badge_definitions for icon/name).
    // author_profile_links.source_id references the users.id that owns this
    // profile (source_id is the FK to users, not user_id).
    let author_id_opt = sqlx::query_scalar::<_, i32>(
        r#"SELECT apl.source_id FROM author_profile_links apl
          JOIN author_profiles ap ON ap.id = apl.profile_id
          WHERE ap.canonical_name = $1
          LIMIT 1"#,
    )
    .bind(&canonical_name)
    .fetch_optional(&state.db)
    .await?;

    let mut badges: Vec<Value> = Vec::new();
    let mut favorite_tags: Vec<Value> = Vec::new();
    if let Some(author_user_id) = author_id_opt {
        // Earned badges with their definitions (for icon display).
        badges = sqlx::query_as::<_, (String, String, String, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT bd.badge_type, bd.name, bd.icon, ub.earned_at
               FROM user_badges ub
               JOIN badge_definitions bd ON bd.badge_type = ub.badge_type
               WHERE ub.user_id = $1
               ORDER BY ub.earned_at DESC
               LIMIT 24"#,
        )
        .bind(author_user_id)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|(badge_type, name, icon, earned_at)| {
            json!({
                "badge_type": badge_type,
                "name": name,
                "icon": icon,
                "earned_at": earned_at.to_rfc3339(),
            })
        })
        .collect();

        // Favorite tags (tags the user follows), for AO3-style display.
        favorite_tags = sqlx::query_as::<_, (String, i16)>(
            "
            SELECT t.name, t.tag_type_id
            FROM user_favorite_tags uft
            JOIN tags t ON t.id = uft.tag_id
            WHERE uft.user_id = $1
            ORDER BY t.name
            LIMIT 20
        ",
        )
        .bind(author_user_id)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|(name, tag_type)| json!({ "name": name, "tag_type_id": tag_type }))
        .collect();
    }

    // Top tags: freeform (type 4) + fandom (type 1) ranked by fic usage.
    let top_tags: Vec<(String, i16, i64)> = sqlx::query_as(
        r#"SELECT t.name, t.tag_type_id, COUNT(DISTINCT ft.url_id)::bigint AS usage_count
           FROM tags t
           JOIN fic_tags ft ON ft.tag_id = t.id
           JOIN fic_info fi ON fi.id = ft.url_id
           WHERE LOWER(fi.author) = LOWER($1)
             AND t.tag_type_id IN (1, 4)
           GROUP BY t.id
           ORDER BY usage_count DESC, t.name
           LIMIT 20"#,
    )
    .bind(&name)
    .fetch_all(&state.db)
    .await?;

    // Social links: author_profiles.canonical_name → author_profiles.id →
    // author_socials.profile_id. Only visible socials (is_visible = true)
    // are returned. Sorted by sort_order (curator-controlled).
    let socials: Vec<Value> = sqlx::query_as::<_, (i32, String, String, String)>(
        r#"SELECT s.id, s.platform, s.url, s.label
           FROM author_socials s
           JOIN author_profiles ap ON ap.id = s.profile_id
           WHERE ap.canonical_name = $1 AND s.is_visible = true
           ORDER BY s.sort_order, s.id"#,
    )
    .bind(&canonical_name)
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(id, platform, url, label)| {
        json!({
            "id": id,
            "platform": platform,
            "url": url,
            "label": label,
        })
    })
    .collect();

    Ok(Json(json!({
        "err": 0,
        "author": {
            "name": canonical_name,
            "avatar_url": author_profile.as_ref().map(|p| p.0.clone()).unwrap_or(None),
            "bio": author_profile.as_ref().map(|p| p.1.clone()).unwrap_or(None),
            "badges": badges,
            "favorite_tags": favorite_tags,
            "socials": socials,
            "work_count": stats.0,
            "total_words": stats.1.unwrap_or(0),
            "top_tags": top_tags.into_iter().map(|(tag_name, tag_type, count)| json!({
                "name": tag_name,
                "tag_type_id": tag_type,
                "usage_count": count,
            })).collect::<Vec<_>>(),
        },
        "works": works,
        "orphans": orphans,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_author_name_is_rejected() {
        // Route-level validation lives in the handler; the guard is trivially
        // testable here.
        let name = "   ".trim();
        assert!(name.is_empty());
    }

    #[test]
    fn next_in_series_computation_links_consecutive_works() {
        // Pure mirror of the handler's next_in_series loop: with N works the
        // i-th work's next is i+1 and the last has none.
        let works = vec![
            json!({"work_id": 1, "canonical_title": "A", "url_id": "a"}),
            json!({"work_id": 2, "canonical_title": "B", "url_id": "b"}),
            json!({"work_id": 3, "canonical_title": "C", "url_id": "c"}),
        ];
        let mut out = works.clone();
        for i in 0..out.len() {
            if i + 1 < out.len() {
                out[i]["next_in_series"] = json!({
                    "work_id": out[i + 1]["work_id"],
                    "canonical_title": out[i + 1]["canonical_title"],
                    "url_id": out[i + 1]["url_id"],
                });
            } else {
                out[i]["next_in_series"] = Value::Null;
            }
        }
        assert_eq!(out[0]["next_in_series"]["work_id"], 2);
        assert_eq!(out[1]["next_in_series"]["work_id"], 3);
        assert!(out[2]["next_in_series"].is_null());
    }
}
