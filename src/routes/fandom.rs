//! Fandom landing pages.
//!
//! - `GET /api/fandoms` — index of all fandoms (tag_type_id=1), sorted by
//!   fic count descending, with total word count per fandom.
//! - `GET /api/fandoms/{slug}` — per-fandom hub: fandom stats, top fics by
//!   tag score, and recently updated fics.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};

use crate::error::AppError;
use crate::server::AppState;

/// Compute a URL-safe slug from a tag name.
///
/// Lowercases, replaces spaces with hyphens, strips non-alphanumeric
/// characters (keeping hyphens), and collapses consecutive hyphens.
pub fn tag_to_slug(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    slug
}

/// GET /api/fandoms — list all fandoms sorted by fic count.
///
/// Fandoms are tags with `tag_type_id = 1`. The query JOINs `tags` with
/// `fic_tags` to count how many fics each fandom is tagged on, and sums
/// word counts from `fic_info`. Fandoms with no tagged fics are excluded
/// (they'd just show zero/zero which isn't useful).
pub async fn list_fandoms(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"SELECT t.name,
                  COUNT(DISTINCT ft.url_id)::bigint AS fic_count,
                  COALESCE(SUM(DISTINCT fi_summary.total_words), 0)::bigint AS total_words
           FROM tags t
           JOIN fic_tags ft ON ft.tag_id = t.id
           LEFT JOIN LATERAL (
               SELECT SUM(fi.words) AS total_words
               FROM fic_info fi
               WHERE fi.id = ft.url_id
           ) fi_summary ON TRUE
           WHERE t.tag_type_id = 1
           GROUP BY t.id, t.name
           ORDER BY fic_count DESC, t.name ASC"#,
    )
    .fetch_all(&state.db)
    .await?;

    let fandoms: Vec<Value> = rows
        .into_iter()
        .map(|(name, fic_count, total_words)| {
            json!({
                "slug": tag_to_slug(&name),
                "name": name,
                "fic_count": fic_count,
                "total_words": total_words,
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "fandoms": fandoms,
    })))
}

/// GET /api/fandoms/{slug} — fandom landing page data.
///
/// Looks up the fandom tag by computing the expected name from the slug
/// (reverse of `tag_to_slug`). If the slug matches no tag, returns 404.
/// Returns fandom stats, top fics by tag score (limit 20), and recently
/// updated fics (limit 10). When no fics are tagged yet, returns empty
/// arrays (not an error).
pub async fn get_fandom(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    // Find the fandom tag. We match by slug: compare computed slug of each
    // fandom tag against the requested slug. This is O(1) because the number
    // of fandom tags is small (hundreds at most).
    let fandoms =
        sqlx::query_as::<_, (i32, String)>("SELECT id, name FROM tags WHERE tag_type_id = 1")
            .fetch_all(&state.db)
            .await?;

    let (tag_id, tag_name) = fandoms
        .into_iter()
        .find(|(_, name)| tag_to_slug(name) == slug)
        .ok_or_else(|| AppError::NotFound(format!("Fandom '{}' not found", slug)))?;

    // Aggregate stats for this fandom: fic count, total words, active authors.
    let stats = sqlx::query_as::<_, (i64, Option<i64>, Option<i64>)>(
        r#"SELECT
             COUNT(DISTINCT ft.url_id)::bigint AS fic_count,
             COALESCE(SUM(fi.words), 0)::bigint AS total_words,
             COUNT(DISTINCT fi.author)::bigint AS active_authors
           FROM fic_tags ft
           LEFT JOIN fic_info fi ON fi.id = ft.url_id
           WHERE ft.tag_id = $1"#,
    )
    .bind(tag_id)
    .fetch_one(&state.db)
    .await?;

    let fic_count = stats.0;
    let total_words = stats.1.unwrap_or(0);
    let active_authors = stats.2.unwrap_or(0);

    // Top fics: ordered by tag score DESC, limit 20.
    let top_rows = sqlx::query_as::<_, (String, String, String, i64, i32, i32)>(
        r#"SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters, ft.score
           FROM fic_tags ft
           JOIN fic_info fi ON fi.id = ft.url_id
           WHERE ft.tag_id = $1
           ORDER BY ft.score DESC, fi.title ASC
           LIMIT 20"#,
    )
    .bind(tag_id)
    .fetch_all(&state.db)
    .await?;

    let top_fics: Vec<Value> = top_rows
        .into_iter()
        .map(|(url_id, title, author, words, chapters, score)| {
            json!({
                "url_id": url_id,
                "title": title,
                "author": author,
                "words": words,
                "chapters": chapters,
                "score": score,
            })
        })
        .collect();

    // Recently updated fics: ordered by fic_info.fic_updated DESC, limit 10.
    let recent_rows = sqlx::query_as::<_, (String, String, String, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT fi.id, fi.title, fi.author, fi.fic_updated
           FROM fic_tags ft
           JOIN fic_info fi ON fi.id = ft.url_id
           WHERE ft.tag_id = $1
           ORDER BY fi.fic_updated DESC, fi.title ASC
           LIMIT 10"#,
    )
    .bind(tag_id)
    .fetch_all(&state.db)
    .await?;

    let recently_updated: Vec<Value> = recent_rows
        .into_iter()
        .map(|(url_id, title, author, updated_at)| {
            json!({
                "url_id": url_id,
                "title": title,
                "author": author,
                "updated_at": updated_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "fandom": {
            "slug": slug,
            "name": tag_name,
            "fic_count": fic_count,
            "total_words": total_words,
            "active_authors": active_authors,
        },
        "top_fics": top_fics,
        "recently_updated": recently_updated,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_simple_name() {
        assert_eq!(tag_to_slug("Harry Potter"), "harry-potter");
    }

    #[test]
    fn slug_with_special_chars() {
        assert_eq!(tag_to_slug("Naruto ( Anime )"), "naruto-anime");
    }

    #[test]
    fn slug_already_lowercased() {
        assert_eq!(tag_to_slug("marvel"), "marvel");
    }

    #[test]
    fn slug_empty_string() {
        assert_eq!(tag_to_slug(""), "");
    }

    #[test]
    fn slug_multiple_hyphens_collapsed() {
        assert_eq!(tag_to_slug("A  --  B"), "a-b");
    }

    #[test]
    fn slug_punctuation_stripped() {
        assert_eq!(
            tag_to_slug("Kuroshitsuji / Black Butler"),
            "kuroshitsuji-black-butler"
        );
    }
}
