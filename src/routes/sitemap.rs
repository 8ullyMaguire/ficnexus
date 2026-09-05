//! Sitemap generation (`feat(seo)`: item 2 of docs/plans/2026-09-03-improve-existing-site.md).
//!
//! Serves `/sitemaps/works-{shard}.xml` and `/sitemaps/index.xml`.
//! Works are sharded alphabetically by first character of `url_id` so each
//! file stays well under the 50k-URL / 50MB sitemap limits. All SQL is
//! on-demand (a nightly crawler hits these endpoints; no background job).
//!
//! Endpoint URLs use the `public_origin` config so the sitemap is valid
//! regardless of which host crawls it.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;

use crate::server::AppState;

/// The 36 shards: 0-9 then a-z (case-insensitive on url_id's first char).
const SHARDS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Map a url_id's first character to its shard: 0-9 stay as-is, letters go
/// lowercase, anything unusual lands in the '0' shard.
#[cfg(test)]
fn shard_for_url_id(url_id: &str) -> char {
    let c = url_id.chars().next().unwrap_or('0').to_ascii_lowercase();
    if c.is_ascii_digit() {
        c
    } else if c.is_ascii_lowercase() {
        c
    } else {
        '0' // anything unusual lands in the '0' shard
    }
}

/// `GET /sitemaps/index.xml` — sitemap index listing all shard files.
pub async fn sitemap_index_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let origin = state.config.public_origin.trim_end_matches('/').to_string();
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for shard in SHARDS {
        xml.push_str(&format!(
            "  <sitemap>\n    <loc>{origin}/sitemaps/works-{shard}.xml</loc>\n  </sitemap>\n"
        ));
    }
    xml.push_str("</sitemapindex>\n");
    ([(axum::http::header::CONTENT_TYPE, "application/xml")], xml)
}

/// One row: the url_id plus the last meaningful update timestamp.
type SitemapRow = (String, Option<chrono::DateTime<chrono::Utc>>);

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// `GET /sitemaps/works-{shard}.xml` — all works whose url_id starts with
/// the shard character (case-insensitive). `lastmod` uses `fic_updated`,
/// falling back to `updated`; omitted when neither is set.
pub async fn sitemap_works_handler(
    State(state): State<Arc<AppState>>,
    Path(shard): Path<String>,
) -> impl IntoResponse {
    let shard_char = shard.chars().next().unwrap_or('0').to_ascii_lowercase();
    if !SHARDS.contains(&shard_char) {
        return (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            "unknown shard".to_string(),
        );
    }

    let rows: Vec<SitemapRow> = sqlx::query_as::<_, SitemapRow>(
        r#"SELECT id AS url_id,
                  GREATEST(fic_updated, updated) AS updated
           FROM fic_info
           WHERE LOWER(LEFT(id, 1)) = $1
           ORDER BY id
           LIMIT 50000"#,
    )
    .bind(shard_char.to_string())
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let origin = state.config.public_origin.trim_end_matches('/').to_string();
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for row in rows {
        let (url_id, updated) = row;
        let loc = format!("{origin}/works/{}", xml_escape(&url_id));
        match updated {
            Some(ts) => xml.push_str(&format!(
                "  <url>\n    <loc>{loc}</loc>\n    <lastmod>{}</lastmod>\n  </url>\n",
                ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            )),
            None => xml.push_str(&format!("  <url>\n    <loc>{loc}</loc>\n  </url>\n")),
        }
    }
    xml.push_str("</urlset>\n");
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_for_url_id_partitions() {
        assert_eq!(shard_for_url_id("abc123"), 'a');
        assert_eq!(shard_for_url_id("ABC123"), 'a');
        assert_eq!(shard_for_url_id("9x"), '9');
        // unusual first chars (e.g. urls with punctuation) land in '0'
        assert_eq!(shard_for_url_id("_weird"), '0');
        assert_eq!(shard_for_url_id(""), '0');
    }

    #[test]
    fn xml_escape_handles_specials() {
        assert_eq!(xml_escape("a&b<c>\"d'e"), "a&amp;b&lt;c&gt;&quot;d&apos;e");
    }
}
