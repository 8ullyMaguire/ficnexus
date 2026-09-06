use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::cache;
use crate::db::queries;
use crate::error::AppError;
use crate::server::AppState;

/// Inject data-passage-hash into block elements (<p> <div> etc).
/// Normalizes text: strips tags, collapses whitespace, SHA-256 hex.
/// Uses regex fallback for <5ms; does NOT mutate cached body.
pub fn inject_passage_hashes(html: &str) -> String {
    // Collapse whitespace helper
    inject_with_regex(html)
}

fn normalize_text(raw: &str) -> String {
    // strip inner tags
    let tag_re = regex_lite::Regex::new(r"<[^>]+>").unwrap();
    let stripped = tag_re.replace_all(raw, " ");
    let ws_re = regex_lite::Regex::new(r"\s+").unwrap();
    ws_re.replace_all(stripped.trim(), " ").to_string()
}

fn hash_text(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

fn inject_with_regex(html: &str) -> String {
    // NOTE: regex-lite has NO backreference support (\1 fails to compile),
    // so we build one pattern per known block tag instead of a single
    // alternation with \1. Each pattern is compiled once (LazyLock).
    use std::sync::LazyLock;
    static TAG_RES: LazyLock<Vec<regex_lite::Regex>> = LazyLock::new(|| {
        ["p", "div", "blockquote", "li", "pre", "section", "article"]
            .iter()
            .filter_map(|t| {
                regex_lite::Regex::new(&format!(r#"(?is)<{0}(\s[^>]*)?>(.*?)</\s*{0}\s*>"#, t)).ok()
            })
            .collect()
    });
    // Heading tags handled separately (h1-h6 share the letter but differ in digit)
    static H_RE: LazyLock<Option<regex_lite::Regex>> = LazyLock::new(|| {
        regex_lite::Regex::new(r#"(?is)<(h[1-6])(\s[^>]*)?>(.*?)</\s*h[1-6]\s*>"#).ok()
    });

    let mut out = html.to_string();
    for re in TAG_RES.iter() {
        out = inject_one_tag(&out, re);
    }
    if let Some(hre) = H_RE.as_ref() {
        out = inject_heading_tags(&out, hre);
    }
    out
}

/// Inject hashes for one specific tag type (no backreference needed — the
/// closing-tag shape is fixed per pattern).
fn inject_one_tag(html: &str, re: &regex_lite::Regex) -> String {
    let mut out = String::with_capacity(html.len() + 512);
    let mut last = 0;
    for caps in re.captures_iter(html) {
        let m = match caps.get(0) {
            Some(m) => m,
            None => continue,
        };
        out.push_str(&html[last..m.start()]);
        // group 1 = attrs, group 2 = inner
        let inner = caps.get(2).map(|c| c.as_str()).unwrap_or("");
        let normalized = normalize_text(inner);
        if normalized.is_empty() {
            out.push_str(m.as_str());
        } else {
            // Reconstruct opening tag: find original open tag text
            let open_len = m.as_str().len() - inner.len() - close_tag_len(m.as_str());
            let open_tag = &m.as_str()[..open_len];
            if open_tag.contains("data-passage-hash") {
                out.push_str(m.as_str());
            } else {
                let h = hash_text(&normalized);
                // insert attribute just before the '>' of the opening tag
                let open_trimmed = &open_tag[..open_tag.len() - 1]; // drop '>'
                out.push_str(open_trimmed);
                out.push_str(&format!(" data-passage-hash=\"{}\">", h));
                out.push_str(inner);
                out.push_str(m.as_str()[open_len..].as_ref());
            }
        }
        last = m.end();
    }
    out.push_str(&html[last..]);
    out
}

/// Length of the trailing `</tag>` portion.
fn close_tag_len(full_match: &str) -> usize {
    match full_match.rfind("</") {
        Some(i) => full_match.len() - i,
        None => 0,
    }
}

/// Headings h1-h6: capture which heading level, verify close matches.
fn inject_heading_tags(html: &str, re: &regex_lite::Regex) -> String {
    let mut out = String::with_capacity(html.len() + 256);
    let mut last = 0;
    for caps in re.captures_iter(html) {
        let m = match caps.get(0) {
            Some(m) => m,
            None => continue,
        };
        out.push_str(&html[last..m.start()]);
        let tag = caps.get(1).map(|c| c.as_str()).unwrap_or("");
        let attrs = caps.get(2).map(|c| c.as_str()).unwrap_or("");
        let inner = caps.get(3).map(|c| c.as_str()).unwrap_or("");
        let normalized = normalize_text(inner);
        if normalized.is_empty() {
            out.push_str(m.as_str());
        } else {
            let h = hash_text(&normalized);
            out.push_str(&format!(
                "<{tag}{attrs} data-passage-hash=\"{h}\">{inner}</{tag}>"
            ));
        }
        last = m.end();
    }
    out.push_str(&html[last..]);
    out
}

/// GET /api/reader/{url_id}/sequel — next-in-series for the reader's
/// "Next Up" panel.
///
/// Strategy: if the fic's work belongs to a series, return the work that
/// immediately follows it (series_works ordered by position). If it's the
/// last in its series (or unseriesed), fall back to a same-author
/// title-numeral heuristic (a title that looks like a sequel — contains a
/// numeral / roman numeral / ordinal the current title lacks).
pub async fn reader_sequel_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // Resolve the current work.
    let fic = queries::get_fic_info(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Fic not found".into()))?;
    let work = queries::get_work_by_source(&state.db, &url_id).await?;

    if let Some(work) = &work {
        // Find the work's position in its series (if any).
        let series_row: Option<(i32, i32, i32)> = sqlx::query_as(
            "SELECT series_id, work_id, position FROM series_works WHERE work_id = $1 LIMIT 1",
        )
        .bind(work.id)
        .fetch_optional(&state.db)
        .await?;

        if let Some((series_id, _wid, position)) = series_row {
            // Next work in the same series.
            let next: Option<(i32, String, String)> = sqlx::query_as(
                r#"SELECT sw.work_id, w.canonical_title, w.canonical_author
                   FROM series_works sw
                   JOIN works w ON w.id = sw.work_id
                   WHERE sw.series_id = $1 AND sw.position > $2
                   ORDER BY sw.position
                   LIMIT 1"#,
            )
            .bind(series_id)
            .bind(position)
            .fetch_optional(&state.db)
            .await?;

            if let Some((wid, title, author)) = next {
                // Resolve a canonical url_id for the next work.
                let url_id_next: Option<String> = sqlx::query_scalar(
                    "SELECT id FROM fic_info WHERE work_id = $1 ORDER BY id LIMIT 1",
                )
                .bind(wid)
                .fetch_optional(&state.db)
                .await?;
                if let Some(url_id_next) = url_id_next {
                    // Also fetch Markov next-up from rec_transitions (sequential strategy).
                    let next_up = fetch_sequential_next_up(&state.db, work.id).await;
                    let resp = json!({
                        "err": 0,
                        "sequel": { "url_id": url_id_next, "title": title, "author": author },
                        "next_up": next_up,
                    });
                    return Ok(Json(resp));
                }
            }
        }
    }

    // Fallback: same-author title-numeral heuristic.
    let rows: Vec<(String, String, String)> = sqlx::query_as::<_, (String, String, String)>(
        r#"SELECT fi.id, fi.title, fi.author
           FROM fic_info fi
           WHERE fi.author = $1 AND fi.id <> $2 AND fi.work_id IS NOT NULL
           ORDER BY fi.updated DESC, fi.id
           LIMIT 200"#,
    )
    .bind(&fic.author)
    .bind(&url_id)
    .fetch_all(&state.db)
    .await?;

    let sequel = rows
        .into_iter()
        .find(|(_, title, _)| looks_like_sequel(&fic.title, title))
        .map(|(url_id, title, author)| (url_id, title, author));

    // Fetch Markov next-up even in the fallback path.
    let next_up = if let Some(work) = &work {
        fetch_sequential_next_up(&state.db, work.id).await
    } else {
        vec![]
    };

    match sequel {
        Some((url_id, title, author)) => Ok(Json(json!({
            "err": 0,
            "sequel": { "url_id": url_id, "title": title, "author": author },
            "next_up": next_up,
        }))),
        None => Ok(Json(json!({
            "err": -1,
            "msg": "no sequel",
            "next_up": next_up,
        }))),
    }
}
/// Fetch top-3 next reads from the Markov transition graph (sequential strategy).
/// Degrades to empty vec on any DB error (never 5xx the reader page).
async fn fetch_sequential_next_up(db: &sqlx::PgPool, work_id: i32) -> Vec<serde_json::Value> {
    let rows: Vec<(String, String, String, f64)> = sqlx::query_as(
        r#"SELECT fi.id, w.canonical_title, w.canonical_author, rt.weight
           FROM rec_transitions rt
           JOIN works w ON w.id = rt.to_work
           JOIN fic_info fi ON fi.work_id = rt.to_work
           WHERE rt.from_work = $1
           ORDER BY rt.weight DESC
           LIMIT 3"#,
    )
    .bind(work_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    rows
        .into_iter()
        .map(|(url_id, title, author, score)| {
            json!({
                "url_id": url_id,
                "title": title,
                "author": author,
                "score": score,
            })
        })
        .collect()
}

/// Heuristic sequel detection on two titles (same author assumed). Mirrors
/// the frontend's `looksLikeSequel` (reader-lib.ts).
fn looks_like_sequel(prev_title: &str, candidate_title: &str) -> bool {
    let t1 = normalize_title(prev_title);
    let t2 = normalize_title(candidate_title);
    if t1 == t2 {
        return false;
    }
    let numeral_re =
        regex_lite::Regex::new(r"(\b(?:2|3|4|5|6|7|8|9|10|11|12)\b|\b(?:II|III|IV|V|VI|VII|VIII|IX|X)\b|(?:2nd|3rd|4th|5th|6th|7th|8th|9th|10th)\b)")
            .expect("static regex");
    numeral_re.is_match(&t2) && !numeral_re.is_match(&t1)
}

fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// GET /api/reader/{url_id}/related — "readers also bookmarked"
/// co-occurrence for the "Next Up" panel.
///
/// Users who bookmarked this fic also bookmarked these fics (excludes
/// itself). Ranked by shared-bookmarker count.
pub async fn reader_related_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let rows: Vec<(String, String, String, i64)> = sqlx::query_as(
        r#"SELECT other.url_id, fi.title, fi.author, COUNT(*)::bigint AS shared
           FROM bookmarks b
           JOIN bookmarks other ON other.user_id = b.user_id AND other.url_id <> b.url_id
           JOIN fic_info fi ON fi.id = other.url_id
           WHERE b.url_id = $1
           GROUP BY other.url_id, fi.title, fi.author
           ORDER BY shared DESC, other.url_id
           LIMIT 10"#,
    )
    .bind(&url_id)
    .fetch_all(&state.db)
    .await?;

    let related: Vec<Value> = rows
        .into_iter()
        .map(|(url_id, title, author, shared)| {
            json!({ "url_id": url_id, "title": title, "author": author, "shared": shared })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "related": related })))
}

/// GET /api/reader/{url_id}/sequential — Markov next-read suggestions.
///
/// Wraps `rec_transitions` (built by the `sequential` strategy's `train`).
/// Gated on logged-in: anonymous callers get an empty list. Returns
/// targets ranked by decayed weight, joined to `fic_info` for titles.
pub async fn reader_sequential_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    auth: crate::routes::auth::AuthUser,
) -> Result<Json<Value>, AppError> {
    // Gate on logged-in — sequential Markov is a history-derived signal.
    if auth.user_id.is_none() {
        return Ok(Json(json!({ "err": 0, "sequential": [] })));
    }
    let rows: Vec<(String, String, String, f64)> = sqlx::query_as(
        r#"SELECT rt.to_work, fi.title, fi.author,
                  rt.weight * exp(-(EXTRACT(EPOCH FROM (NOW() - rt.last_seen)) / 86400.0) / 30.0) AS w
           FROM rec_transitions rt
           JOIN fic_info fi ON fi.id = rt.to_work
           WHERE rt.from_work = $1
           ORDER BY w DESC
           LIMIT 10"#,
    )
    .bind(&url_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let sequential: Vec<Value> = rows
        .into_iter()
        .map(|(to_work, title, author, score)| {
            json!({ "url_id": to_work, "title": title, "author": author, "score": score, "reason": "readers typically continue with this" })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "sequential": sequential })))
}

/// GET /api/reader/{url_id}/next-up — aggregated Next Up panel.
///
/// Combines sequel + related + sequential (Markov) sources, deduped by
/// `url_id` and ranked by source priority: sequel > sequential > related.
/// Sequential is gated on logged-in (anonymous callers get the other two
/// sources only). Preserves existing behaviour when sequential is empty.
pub async fn reader_next_up_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    auth: crate::routes::auth::AuthUser,
) -> Result<Json<Value>, AppError> {
    let fic = queries::get_fic_info(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Fic not found".into()))?;
    let work = queries::get_work_by_source(&state.db, &url_id).await?;

    // --- sequel (next-in-series or title-numeral fallback) ---
    let sequel_val: Option<Value> = {
        let mut found: Option<Value> = None;
        if let Some(w) = &work {
            let series_row: Option<(i32, i32, i32)> = sqlx::query_as(
                "SELECT series_id, work_id, position FROM series_works WHERE work_id = $1 LIMIT 1",
            )
            .bind(w.id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);
            if let Some((series_id, _wid, position)) = series_row {
                let next: Option<(i32, String, String)> = sqlx::query_as(
                    r#"SELECT sw.work_id, w.canonical_title, w.canonical_author
                       FROM series_works sw JOIN works w ON w.id = sw.work_id
                       WHERE sw.series_id = $1 AND sw.position > $2
                       ORDER BY sw.position LIMIT 1"#,
                )
                .bind(series_id)
                .bind(position)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);
                if let Some((wid, title, author)) = next {
                    let url_id_next: Option<String> = sqlx::query_scalar(
                        "SELECT id FROM fic_info WHERE work_id = $1 ORDER BY id LIMIT 1",
                    )
                    .bind(wid)
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None);
                    if let Some(uid) = url_id_next {
                        found = Some(json!({ "url_id": uid, "title": title, "author": author }));
                    }
                }
            }
        }
        if found.is_none() {
            let rows: Vec<(String, String, String)> =
                sqlx::query_as::<_, (String, String, String)>(
                    r#"SELECT fi.id, fi.title, fi.author FROM fic_info fi
                   WHERE fi.author = $1 AND fi.id <> $2 AND fi.work_id IS NOT NULL
                   ORDER BY fi.updated DESC, fi.id LIMIT 200"#,
                )
                .bind(&fic.author)
                .bind(&url_id)
                .fetch_all(&state.db)
                .await
                .unwrap_or_default();
            for (uid, title, author) in rows {
                if looks_like_sequel(&fic.title, &title) {
                    found = Some(json!({ "url_id": uid, "title": title, "author": author }));
                    break;
                }
            }
        }
        found
    };

    // --- related ---
    let related_vals: Vec<Value> = {
        let rows: Vec<(String, String, String, i64)> = sqlx::query_as(
            r#"SELECT other.url_id, fi.title, fi.author, COUNT(*)::bigint AS shared
               FROM bookmarks b JOIN bookmarks other ON other.user_id = b.user_id AND other.url_id <> b.url_id
               JOIN fic_info fi ON fi.id = other.url_id
               WHERE b.url_id = $1 GROUP BY other.url_id, fi.title, fi.author
               ORDER BY shared DESC, other.url_id LIMIT 10"#,
        )
        .bind(&url_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        rows.into_iter().map(|(uid, title, author, shared)| json!({ "url_id": uid, "title": title, "author": author, "shared": shared })).collect()
    };

    // --- sequential (gated) ---
    let sequential_vals: Vec<Value> = if auth.user_id.is_some() {
        let rows: Vec<(String, String, String, f64)> = sqlx::query_as(
            r#"SELECT rt.to_work, fi.title, fi.author,
                      rt.weight * exp(-(EXTRACT(EPOCH FROM (NOW() - rt.last_seen)) / 86400.0) / 30.0) AS w
               FROM rec_transitions rt JOIN fic_info fi ON fi.id = rt.to_work
               WHERE rt.from_work = $1 ORDER BY w DESC LIMIT 10"#,
        )
        .bind(&url_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        rows.into_iter().map(|(uid, title, author, score)| json!({ "url_id": uid, "title": title, "author": author, "score": score, "reason": "readers typically continue with this" })).collect()
    } else {
        vec![]
    };

    // Aggregate deduped: sequel first, then sequential, then related.
    let mut seen = std::collections::HashSet::new();
    let mut items: Vec<Value> = Vec::new();
    if let Some(s) = &sequel_val {
        if seen.insert(s["url_id"].as_str().unwrap_or("").to_string()) {
            items.push(json!({ "url_id": s["url_id"], "title": s["title"], "author": s["author"], "reason": "Next in series", "source": "sequel" }));
        }
    }
    for v in &sequential_vals {
        let uid = v["url_id"].as_str().unwrap_or("");
        if !uid.is_empty() && seen.insert(uid.to_string()) {
            items.push(json!({ "url_id": v["url_id"], "title": v["title"], "author": v["author"], "reason": "What to read next", "source": "sequential", "score": v["score"] }));
        }
    }
    for v in &related_vals {
        let uid = v["url_id"].as_str().unwrap_or("");
        if !uid.is_empty() && seen.insert(uid.to_string()) {
            items.push(json!({ "url_id": v["url_id"], "title": v["title"], "author": v["author"], "reason": "Readers also bookmarked", "source": "related", "shared": v["shared"] }));
        }
    }

    Ok(Json(
        json!({ "err": 0, "sequel": sequel_val, "sequential": sequential_vals, "related": related_vals, "next_up": items }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct MarginaliaParams {
    pub chapter: Option<i32>,
}

/// GET /api/reader/{url_id}/marginalia?chapter={n}
pub async fn reader_marginalia_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    Query(params): Query<MarginaliaParams>,
) -> Result<Json<Value>, AppError> {
    let chapter = params.chapter.unwrap_or(0);
    // Resolve work_id via fic_info or works
    let work = queries::get_work_by_source(&state.db, &url_id).await?;
    let work_id = match work {
        Some(w) => w.id,
        None => return Ok(Json(json!({ "err": 0, "marginalia": [] }))),
    };
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT passage_hash, topic_id FROM marginalia WHERE work_id = $1 AND chapter_index = $2",
    )
    .bind(work_id)
    .bind(chapter)
    .fetch_all(&state.db)
    .await?;
    let mut out = Vec::new();
    for (passage_hash, topic_id) in rows {
        let post_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM forum_posts WHERE topic_id = $1 AND deleted_at IS NULL",
        )
        .bind(topic_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
        // topic_slug for frontend linking
        let topic_slug: Option<String> =
            sqlx::query_scalar("SELECT topic_slug FROM forum_topics WHERE id = $1")
                .bind(topic_id)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);
        out.push(json!({
            "passage_hash": passage_hash,
            "topic_id": topic_id,
            "topic_slug": topic_slug,
            "post_count": post_count
        }));
    }
    Ok(Json(json!({ "err": 0, "marginalia": out })))
}

pub async fn reader_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // Get fic info
    let fic = queries::get_fic_info(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Fic not found".into()))?;

    // Determine version and hash for cache lookup
    let version_bump = queries::get_fic_version_bump(&state.db, &url_id)
        .await?
        .unwrap_or(0);
    let version = state.config.export_version + version_bump;
    let input_hash = fic
        .content_hash
        .clone()
        .unwrap_or_else(|| "upstream".to_string());

    // Find the export log for HTML format. Legacy rows were exported with
    // input_hash = "epub:<epub_hash>" (derived-from-epub), not the raw
    // content hash — try both before giving up.
    let log =
        match queries::find_export_log(&state.db, &url_id, version, "html", &input_hash).await? {
            Some(l) => Some(l),
            None => queries::latest_export_log(&state.db, &url_id, version, "html").await?,
        }
        .ok_or_else(|| AppError::NotFound("No HTML cache. Export the fic first.".into()))?;

    // Build cache path and read the zip
    let cache_path = cache::disk::cache_path(
        &state.config.cache_dir,
        &cache::EType::Html,
        &url_id,
        &log.export_hash,
    );

    if !cache_path.exists() {
        return Err(AppError::NotFound("HTML file not found on disk".into()));
    }

    // Read the zip and extract index.html
    let zip_data = std::fs::read(&cache_path)?;
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::Internal(format!("Failed to read HTML cache zip: {}", e)))?;

    let html_content = archive
        .by_name("index.html")
        .map_err(|_| AppError::NotFound("index.html not found in cache bundle".into()))?;

    // Read the entry content into a string
    use std::io::Read;
    let mut html_string = String::new();
    let mut reader = html_content;
    reader.read_to_string(&mut html_string)?;

    // Inject passage hashes (non-destructive, not cached)
    let html_string = inject_passage_hashes(&html_string);

    // Look up work_id for reading_stats tracking
    let work_id = queries::get_work_by_source(&state.db, &url_id)
        .await?
        .map(|w| w.id)
        .unwrap_or(0);

    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "title": fic.title,
        "author": fic.author,
        "work_id": work_id,
        "html": html_string,
        "words": fic.words,
        "chapters": fic.chapters,
    })))
}

#[cfg(test)]
mod tests {
    use super::inject_passage_hashes;

    #[test]
    fn test_inject_p() {
        let h = inject_passage_hashes("<p>Hello <b>world</b></p>");
        assert!(h.contains("data-passage-hash"), "got: {}", h);
        assert!(h.starts_with("<p data-passage-hash="), "got: {}", h);
    }

    #[test]
    fn test_inject_heading() {
        let h = inject_passage_hashes("<h2>A Title</h2>");
        assert!(h.contains("data-passage-hash"), "got: {}", h);
    }

    #[test]
    fn test_empty_skipped() {
        let h = inject_passage_hashes("<p>   </p><p>Real text</p>");
        // empty paragraph untouched, real one hashed
        assert!(
            !h.split("</p>")
                .next()
                .unwrap()
                .contains("data-passage-hash")
        );
        let rest: Vec<&str> = h.splitn(2, "</p>").collect();
        assert!(
            rest.len() == 2 && rest[1].contains("data-passage-hash"),
            "got: {}",
            h
        );
    }

    #[test]
    fn test_stable_hash() {
        let a = inject_passage_hashes("<p>Same text</p>");
        let b = inject_passage_hashes("<p>Same text</p>");
        let ha = a
            .split("data-passage-hash=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let hb = b
            .split("data-passage-hash=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(ha, hb);
    }
}
