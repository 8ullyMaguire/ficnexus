//! Atom feed XML construction + shared query helpers.
//!
//! Kept separate from `handlers` so the XML-escaping / feed-shape helpers
//! are unit-testable without a database.

use axum::http::header;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppResult;
use crate::server::AppState;

use super::{ATOM_CONTENT_TYPE, MAX_ENTRIES};

/// Escape a string for use inside XML text/attribute content.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Current UTC time in Atom's RFC 3339 / ISO 8601 form.
pub fn iso_now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// A fic row as rendered into an Atom entry.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FeedFic {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub fic_updated: chrono::DateTime<chrono::Utc>,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub work_id: Option<i32>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
}

impl FeedFic {
    /// RFC 3339 timestamp for the entry's `<updated>`.
    pub fn updated(&self) -> String {
        self.fic_updated.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }
}

/// Build a single Atom `<entry>` for a fic.
///
/// `entry_id` is the stable URN used as `<id>`; `page_path` is the fic page
/// path (`/fic/<url_id>`); `title` is the entry title (defaults to the fic
/// title); `updated` is the RFC 3339 timestamp.
pub fn entry_xml(
    entry_id: &str,
    page_path: &str,
    title: &str,
    author: &str,
    summary: &str,
    updated: &str,
    words: i64,
    chapters: i32,
    status: &str,
) -> String {
    format!(
        r#"  <entry>
    <title>{title}</title>
    <author><name>{author}</name></author>
    <id>{id}</id>
    <updated>{updated}</updated>
    <summary>{summary}</summary>
    <category term="{status}" label="{status}"/>
    <category term="chapters:{chapters}" label="{chapters} chapters"/>
    <category term="words:{words}" label="{words} words"/>
    <link rel="alternate" href="{page}" type="text/html"/>
    <link rel="self" href="{page}.xml" type="{ct}"/>
    <link rel="enclosure" href="/cache/epub/{url}/{url}.epub" type="application/epub+zip" length="0"/>
    <link rel="enclosure" href="/cache/pdf/{url}/{url}.pdf" type="application/pdf" length="0"/>
    <link rel="enclosure" href="/cache/html/{url}/{url}.html" type="text/html" length="0"/>
  </entry>
"#,
        title = html_escape(title),
        author = html_escape(author),
        id = html_escape(entry_id),
        updated = updated,
        summary = html_escape(summary),
        status = html_escape(status),
        chapters = chapters,
        words = words,
        page = html_escape(page_path),
        ct = ATOM_CONTENT_TYPE,
        url = html_escape(url_of(page_path)),
    )
}

/// Extract the url_id from a page path (`/fic/<url_id>` → `<url_id>`).
fn url_of(page_path: &str) -> &str {
    page_path
        .trim_start_matches('/')
        .split('/')
        .nth(1)
        .unwrap_or(page_path)
}

/// Build a complete Atom feed document.
///
/// `feed_id` is the stable URN, `self_path` the path this feed lives at
/// (used for `<link rel="self">`), `entries` the pre-built entry XML.
pub fn atom_feed(
    feed_id: &str,
    title: &str,
    subtitle: &str,
    self_path: &str,
    updated: &str,
    entries: &str,
    base_url: Option<&str>,
) -> String {
    let self_abs = abs_url(self_path, base_url);
    let mut links = String::new();
    links.push_str(&format!(
        r#"  <link href="{self}" rel="self" type="{ct}"/>
"#,
        self = html_escape(&self_abs),
        ct = ATOM_CONTENT_TYPE,
    ));
    links.push_str(&format!(
        r#"  <link href="{alt}" rel="alternate" type="text/html"/>
"#,
        alt = html_escape(&abs_url("/", base_url)),
    ));

    let base_attr = match base_url {
        Some(b) if !b.is_empty() => format!(" xml:base=\"{}\"", html_escape(b)),
        _ => String::new(),
    };

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"{base}>
  <id>{id}</id>
  <title>{title}</title>
  <subtitle>{subtitle}</subtitle>
  <updated>{updated}</updated>
  <author><name>FicNexus</name></author>
{links}{entries}</feed>
"#,
        base = base_attr,
        id = html_escape(feed_id),
        title = html_escape(title),
        subtitle = html_escape(subtitle),
        updated = updated,
        links = links,
        entries = entries,
    )
}

/// Response tuple `(Content-Type header, body)` used by every feed handler.
pub fn atom_response(body: String) -> ([(header::HeaderName, &'static str); 1], String) {
    ([(header::CONTENT_TYPE, ATOM_CONTENT_TYPE)], body)
}

/// Absolutize a path against the optional base URL (same rule as OPDS).
pub fn abs_url(path: &str, base_url: Option<&str>) -> String {
    match base_url {
        Some(base) if !base.is_empty() => {
            let base = base.trim_end_matches('/');
            let path = path.trim_start_matches('/');
            format!("{}/{}", base, path)
        }
        _ => path.to_string(),
    }
}

/// Render a `FeedFic` row into an Atom entry (fic-page variant).
pub fn entry_for_fic(row: &FeedFic) -> String {
    entry_xml(
        &format!("urn:ficnexus:fic:{}", row.id),
        &format!("/fic/{}", row.id),
        &row.title,
        &row.author,
        &row.description,
        &row.updated(),
        row.words,
        row.chapters,
        &row.status,
    )
}

/// Query params shared by the public feeds (limit/ordering knobs).
#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub limit: Option<i64>,
    /// Optional override for how many entries to return (default 20).
    pub per_page: Option<i64>,
}

impl FeedQuery {
    pub fn effective_limit(&self) -> i64 {
        self.limit
            .or(self.per_page)
            .unwrap_or(MAX_ENTRIES)
            .clamp(1, MAX_ENTRIES)
    }
}

/// Load up to `limit` recently-created fics (`fic_info.created` DESC,
/// fallback `fic_updated` DESC).
pub async fn fetch_new_arrivals(state: &Arc<AppState>, limit: i64) -> AppResult<Vec<FeedFic>> {
    let rows = sqlx::query_as::<_, FeedFic>(
        r#"SELECT id, title, author, description,
                  fic_updated, words, chapters, status, work_id, created
           FROM fic_info
           ORDER BY COALESCE(created, fic_updated) DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// Load the most recently updated of the fics a user follows (by JWT `sub`),
/// ordered `fic_updated` DESC. Followed *authors* (author_name rows) are
/// resolved through their fics; followed users are not part of the feed.
pub async fn fetch_followed_updates(
    state: &Arc<AppState>,
    user_id: i32,
    limit: i64,
) -> AppResult<Vec<FeedFic>> {
    let rows = sqlx::query_as::<_, FeedFic>(
        r#"SELECT fi.id, fi.title, fi.author, fi.description,
                  fi.fic_updated, fi.words, fi.chapters, fi.status,
                  fi.work_id, fi.created
           FROM follows f
           JOIN fic_info fi
             ON (f.work_id IS NOT NULL AND fi.work_id = f.work_id)
             OR (f.work_id IS NULL AND f.author_name IS NOT NULL
                 AND fi.author = f.author_name)
           WHERE f.follower_id = $1
           ORDER BY fi.fic_updated DESC
           LIMIT $2"#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// Load a single fic by url_id for the per-fic feed.
pub async fn fetch_fic(state: &Arc<AppState>, url_id: &str) -> AppResult<Option<FeedFic>> {
    let row = sqlx::query_as::<_, FeedFic>(
        r#"SELECT id, title, author, description,
                  fic_updated, words, chapters, status, work_id, created
           FROM fic_info
           WHERE id = $1"#,
    )
    .bind(url_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_handles_all_specials() {
        let out = html_escape("a & b < c > d \" e ' f");
        assert_eq!(out, "a &amp; b &lt; c &gt; d &quot; e &apos; f");
    }

    #[test]
    fn escape_empty_is_empty() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn atom_feed_has_required_elements() {
        let body = atom_feed(
            "urn:ficnexus:feed:new",
            "FicNexus — New Arrivals",
            "Recently added fanfiction",
            "/feed.xml",
            "2024-01-01T00:00:00Z",
            "<entry/>",
            None,
        );
        assert!(body.contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(body.contains("xmlns=\"http://www.w3.org/2005/Atom\""));
        assert!(body.contains("<id>urn:ficnexus:feed:new</id>"));
        assert!(body.contains("<title>FicNexus — New Arrivals</title>"));
        assert!(body.contains("<updated>2024-01-01T00:00:00Z</updated>"));
        assert!(body.contains("rel=\"self\""));
        assert!(body.contains("href=\"/feed.xml\""));
        assert!(body.contains(ATOM_CONTENT_TYPE));
        assert!(body.contains("<entry/>"));
    }

    #[test]
    fn atom_feed_base_url_makes_self_absolute() {
        let body = atom_feed(
            "urn:x",
            "T",
            "S",
            "/feed.xml",
            "2024-01-01T00:00:00Z",
            "",
            Some("https://fichub.example"),
        );
        assert!(body.contains("xml:base=\"https://fichub.example\""));
        assert!(body.contains("https://fichub.example/feed.xml"));
    }

    #[test]
    fn atom_feed_escapes_title_and_subtitle() {
        let body = atom_feed(
            "urn:x",
            "<Titles> & \"Co\"",
            "sub & <sub>",
            "/feed.xml",
            "2024-01-01T00:00:00Z",
            "",
            None,
        );
        assert!(!body.contains("<title><Titles>"));
        assert!(body.contains("&lt;Titles&gt; &amp; &quot;Co&quot;"));
        assert!(body.contains("&lt;sub&gt;"));
    }

    #[test]
    fn entry_has_id_updated_alternate_and_enclosures() {
        let e = entry_xml(
            "urn:ficnexus:fic:abc123",
            "/fic/abc123",
            "A & B",
            "Author <X>",
            "Summary & more",
            "2024-01-01T00:00:00Z",
            50000,
            10,
            "complete",
        );
        assert!(e.contains("<id>urn:ficnexus:fic:abc123</id>"));
        assert!(e.contains("<updated>2024-01-01T00:00:00Z</updated>"));
        assert!(e.contains("rel=\"alternate\" href=\"/fic/abc123\""));
        assert!(e.contains("rel=\"self\" href=\"/fic/abc123.xml\""));
        assert!(e.contains("/cache/epub/abc123/abc123.epub"));
        assert!(e.contains("/cache/pdf/abc123/abc123.pdf"));
        assert!(e.contains("/cache/html/abc123/abc123.html"));
        // escaped content
        assert!(e.contains("<title>A &amp; B</title>"));
        assert!(e.contains("<author><name>Author &lt;X&gt;</name></author>"));
        assert!(!e.contains("<Author"));
    }

    #[test]
    fn entry_escapes_specials_never_breaks_xml() {
        let e = entry_xml(
            "urn:id",
            "/fic/id",
            "<script>alert('x')</script> & \"",
            "A & B",
            "s <b>html</b> & co",
            "2024-01-01T00:00:00Z",
            1,
            1,
            "ongoing",
        );
        assert!(!e.contains("<script>"));
        assert!(e.contains("&lt;script&gt;"));
        assert!(e.contains("&amp; &quot;"));
    }

    #[test]
    fn feed_query_defaults_and_clamps() {
        let q: FeedQuery = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(q.effective_limit(), MAX_ENTRIES);
        let q: FeedQuery = serde_json::from_value(serde_json::json!({ "limit": 1000 })).unwrap();
        assert_eq!(q.effective_limit(), MAX_ENTRIES);
        let q: FeedQuery = serde_json::from_value(serde_json::json!({ "per_page": 5 })).unwrap();
        assert_eq!(q.effective_limit(), 5);
    }

    #[test]
    fn url_of_extracts_id() {
        assert_eq!(url_of("/fic/abc123"), "abc123");
        assert_eq!(url_of("/fic/abc123.xml"), "abc123.xml");
    }
}
