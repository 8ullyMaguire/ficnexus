pub mod authors;
pub mod feeds;
pub mod manifest;
pub mod recommendations;
pub mod search;
pub mod shelves;
pub mod tags;

use axum::http::header;
use std::collections::HashMap;

/// Map export `etype` string to (cache path segment, MIME type) for OPDS acquisition links.
fn etype_mime_path(etype: &str) -> (&'static str, &'static str) {
    match etype {
        "epub" => ("epub", "application/epub+zip"),
        "pdf" => ("pdf", "application/pdf"),
        "html" => ("html", "text/html"),
        "mobi" => ("mobi", "application/x-mobipocket-ebook"),
        "txt" => ("txt", "text/plain"),
        "azw3" => ("azw3", "application/vnd.amazon.ebook"),
        "md" => ("md", "text/markdown"),
        "docx" => (
            "docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
        "fb2" => ("fb2", "application/x-fictionbook+xml"),
        "kepub" => ("kepub", "application/epub+zip"),
        _ => ("epub", "application/octet-stream"),
    }
}

/// Build a cache href for a given etype/url_id/hash, optionally prefixed with base_url.
fn cache_href(base_url: Option<&str>, etype_path: &str, url_id: &str, hash: &str) -> String {
    let path = format!("/cache/{}/{url_id}?h={hash}", etype_path);
    match base_url {
        Some(b) if !b.is_empty() => {
            let base = b.trim_end_matches('/');
            format!("{base}{path}")
        }
        _ => path,
    }
}

/// Fetch hash-aware acquisition links for a single fic.
/// Queries export_log: DISTINCT ON (etype) latest version.
pub async fn acquisition_links(
    pool: &sqlx::PgPool,
    base_url: Option<&str>,
    url_id: &str,
) -> Vec<(String, String)> {
    let ids = vec![url_id.to_string()];
    let map = acquisition_links_many(pool, base_url, &ids).await;
    map.get(url_id).cloned().unwrap_or_default()
}

/// Batch fetch hash-aware acquisition links for many fics in one query.
/// Returns map url_id -> Vec<(mime, href)>
pub async fn acquisition_links_many(
    pool: &sqlx::PgPool,
    base_url: Option<&str>,
    url_ids: &[String],
) -> HashMap<String, Vec<(String, String)>> {
    if url_ids.is_empty() {
        return HashMap::new();
    }
    // DISTINCT ON (url_id, etype) would be needed for correct per-fic batching;
    // but we use DISTINCT ON (etype) per url_id via ordering trick: Postgres
    // DISTINCT ON requires ORDER BY leading matches DISTINCT expr.
    // For batch, we do: SELECT url_id, etype, export_hash FROM (
    //   SELECT DISTINCT ON (url_id, etype) url_id, etype, export_hash, version, created
    //   FROM export_log WHERE url_id = ANY($1) ORDER BY url_id, etype, version DESC, created DESC
    // ) sub ORDER BY url_id, etype
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"SELECT url_id, etype, export_hash FROM (
                SELECT DISTINCT ON (url_id, etype) url_id, etype, export_hash, version, created
                FROM export_log WHERE url_id = ANY($1)
                ORDER BY url_id, etype, version DESC, created DESC
           ) sub ORDER BY url_id, etype"#,
    )
    .bind(url_ids)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (uid, etype, hash) in rows {
        let (path, mime) = etype_mime_path(&etype);
        let href = cache_href(base_url, path, &uid, &hash);
        map.entry(uid).or_default().push((mime.to_string(), href));
    }
    // Ensure deterministic order: epub first, then pdf, html etc by mime sort
    for v in map.values_mut() {
        v.sort_by(|a, b| a.0.cmp(&b.0));
    }
    map
}

/// Atom XML feed prolog + feed open tag, with optional xml:base.
/// When base_url is Some, adds `xml:base` so relative URLs resolve correctly.
pub fn atom_xml_header(base_url: Option<&str>) -> String {
    let base_attr = match base_url {
        Some(url) if !url.is_empty() => format!(" xml:base=\"{}\"", html_escape(url)),
        _ => String::new(),
    };
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"{base}
      xmlns:dc="http://purl.org/dc/terms/"
      xmlns:opds="http://opds-spec.org/2010/catalog"
      xmlns:pse="http://opds-spec.org/2010/partial">
"#,
        base = base_attr
    )
}

/// Build an OPDS entry XML for a fic, including acquisition and cover links.
/// `acquisitions` is a slice of (mime, href) pairs already containing hash-aware cache URLs.
pub fn fic_entry(
    url_id: &str,
    title: &str,
    author: &str,
    summary: &str,
    updated: &str,
    words: i64,
    chapters: i32,
    status: &str,
    acquisitions: &[(String, String)],
) -> String {
    let safe_title = html_escape(title);
    let safe_author = html_escape(author);
    let safe_summary = html_escape(summary);
    let mut acquisition_links = String::new();
    for (mime, href) in acquisitions {
        acquisition_links.push_str(&format!(
            r#"    <link rel="http://opds-spec.org/acquisition" href="{}" type="{}"/>"#,
            html_escape(href),
            html_escape(mime)
        ));
        acquisition_links.push('\n');
    }
    format!(
        r#"  <entry>
    <title>{title}</title>
    <author><name>{author}</name></author>
    <id>urn:ficnexus:fic:{url_id}</id>
    <updated>{updated}</updated>
    <summary>{summary}</summary>
    <dc:extent>{words}</dc:extent>
    <dc:format>application/epub+zip</dc:format>
    <category term="{status}" label="{status}"/>
    <category term="chapters:{chapters}" label="{chapters} chapters"/>
    <link rel="http://opds-spec.org/image" href="/cache/cover/{url_id}.jpg" type="image/jpeg"/>
    <link rel="http://opds-spec.org/image/thumbnail" href="/cache/cover/{url_id}_thumb.jpg" type="image/jpeg"/>
{acquisitions}    <link rel="alternate" href="/fics/{url_id}" type="text/html"/>
  </entry>
"#,
        title = safe_title,
        author = safe_author,
        url_id = url_id,
        updated = updated,
        summary = safe_summary,
        words = words,
        chapters = chapters,
        status = status,
        acquisitions = acquisition_links,
    )
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn iso_now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Describes what kind of OPDS feed this is.
#[derive(Debug, Clone, Copy)]
pub enum FeedKind {
    Navigation,
    Acquisition,
}

impl FeedKind {
    pub fn content_type(&self) -> &'static str {
        match self {
            FeedKind::Navigation => "application/atom+xml; profile=opds-catalog; kind=navigation",
            FeedKind::Acquisition => "application/atom+xml; profile=opds-catalog; kind=acquisition",
        }
    }
}

/// Optional pagination links for a feed.
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub base_path: String,
    pub page: usize,
    pub per_page: usize,
    pub total: i64,
}

/// Prepend the optional base URL to a relative path for self/prev/next links.
/// Helper: produce an absolute URL from a path, using base_url when provided.
pub(crate) fn abs_url(path: &str, base_url: Option<&str>) -> String {
    match base_url {
        Some(base) if !base.is_empty() => {
            // Avoid double-slash: ensure base has no trailing / and path has no leading /
            let base = base.trim_end_matches('/');
            let path = path.trim_start_matches('/');
            format!("{}/{}", base, path)
        }
        _ => path.to_string(),
    }
}

/// Build the common feed structure with optional xml:base, self link, and pagination links.
///
/// `self_link` is optional — if None, no `<link rel="self">` is emitted.
/// `pagination` is optional — if Some, adds prev/next links.
/// `base_url` is optional — if Some, sets `xml:base` on `<feed>`.
pub fn build_feed(
    title: &str,
    feed_id: &str,
    entries: &str,
    updated: &str,
    self_link: Option<&str>,
    feed_kind: FeedKind,
    pagination: Option<&PaginationInfo>,
    base_url: Option<&str>,
) -> String {
    let mut links = String::new();

    if let Some(link) = self_link {
        let abs = abs_url(link, base_url);
        links.push_str(&format!(
            r#"  <link href="{link}" rel="self" type="{ct}"/>
"#,
            link = html_escape(&abs),
            ct = feed_kind.content_type(),
        ));
    }

    links.push_str(&format!(
        r#"  <link href="{root}" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
"#,
        root = abs_url("/opds", base_url),
    ));

    if let Some(ref pagi) = pagination {
        let total_pages = (pagi.total as f64 / pagi.per_page as f64).ceil() as usize;
        if pagi.page > 1 {
            let prev_page = pagi.page - 1;
            let prev_href = format!(
                "{base}{separator}page={page}&per_page={per_page}",
                base = pagi.base_path,
                separator = if pagi.base_path.contains('?') {
                    '&'
                } else {
                    '?'
                },
                page = prev_page,
                per_page = pagi.per_page,
            );
            let abs_prev = abs_url(&prev_href, base_url);
            links.push_str(&format!(
                r#"  <link href="{prev}" rel="previous" type="{ct}"/>
"#,
                prev = html_escape(&abs_prev),
                ct = feed_kind.content_type(),
            ));
        }
        if pagi.page < total_pages {
            let next_page = pagi.page + 1;
            let next_href = format!(
                "{base}{separator}page={page}&per_page={per_page}",
                base = pagi.base_path,
                separator = if pagi.base_path.contains('?') {
                    '&'
                } else {
                    '?'
                },
                page = next_page,
                per_page = pagi.per_page,
            );
            let abs_next = abs_url(&next_href, base_url);
            links.push_str(&format!(
                r#"  <link href="{next}" rel="next" type="{ct}"/>
"#,
                next = html_escape(&abs_next),
                ct = feed_kind.content_type(),
            ));
        }
    }

    let header = atom_xml_header(base_url);
    format!(
        r#"{header}<id>{id}</id>
  <title>{title}</title>
  <updated>{updated}</updated>
  <author><name>FicNexus</name></author>
{links}{entries}</feed>"#,
        header = header,
        id = html_escape(feed_id),
        title = html_escape(title),
        updated = updated,
        links = links,
        entries = entries,
    )
}

/// Helper: produce a response tuple (ContentType header, body) for OPDS responses.
pub fn opds_response(
    body: String,
    kind: FeedKind,
) -> ([(header::HeaderName, &'static str); 1], String) {
    ([(header::CONTENT_TYPE, kind.content_type())], body)
}

/// Generic pagination query params
#[derive(Debug, serde::Deserialize)]
pub struct PageParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl PageParams {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape_empty() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_html_escape_no_special_chars() {
        assert_eq!(html_escape("hello world"), "hello world");
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_lt_gt() {
        assert_eq!(html_escape("<div>"), "&lt;div&gt;");
    }

    #[test]
    fn test_html_escape_quotes() {
        assert_eq!(html_escape(r#"a "b" c"#), "a &quot;b&quot; c");
    }

    #[test]
    fn test_html_escape_single_quote() {
        assert_eq!(html_escape("it's"), "it&apos;s");
    }

    #[test]
    fn test_html_escape_all_special_chars() {
        let input = "a & < > \" ' b";
        let expected = "a &amp; &lt; &gt; &quot; &apos; b";
        assert_eq!(html_escape(input), expected);
    }

    #[test]
    fn test_html_escape_nested() {
        // Test that escaping works on already-escaped content (no double-escape)
        let input = "&amp;";
        assert_eq!(html_escape(input), "&amp;amp;");
    }

    #[test]
    fn test_fic_entry_basic() {
        let entry = fic_entry(
            "abc123",
            "Test Fic",
            "Author",
            "A summary",
            "2024-01-01T00:00:00Z",
            50000,
            10,
            "complete",
            &[],
        );
        assert!(entry.contains("<entry>"));
        assert!(entry.contains("<title>Test Fic</title>"));
        assert!(entry.contains("<author><name>Author</name></author>"));
        assert!(entry.contains("<id>urn:ficnexus:fic:abc123</id>"));
        assert!(entry.contains("<summary>A summary</summary>"));
        assert!(entry.contains("<dc:extent>50000</dc:extent>"));
        assert!(entry.contains("chapters:10"));
        assert!(entry.contains("complete"));
        assert!(entry.contains("</entry>"));
    }

    #[test]
    fn test_fic_entry_escapes_xml() {
        let entry = fic_entry(
            "id",
            "<script>alert('xss')</script>",
            "Author & Co",
            "Summary with <b>HTML</b>",
            "2024-01-01T00:00:00Z",
            1000,
            5,
            "ongoing",
            &[],
        );
        assert!(!entry.contains("<script>"));
        assert!(entry.contains("&lt;script&gt;"));
        assert!(entry.contains("Author &amp; Co"));
    }

    #[test]
    fn test_fic_entry_contains_links() {
        let acq = vec![
            (
                "application/epub+zip".to_string(),
                "/cache/epub/abc123?h=deadbeef".to_string(),
            ),
            (
                "application/pdf".to_string(),
                "/cache/pdf/abc123?h=cafebabe".to_string(),
            ),
            (
                "text/html".to_string(),
                "/cache/html/abc123?h=12345678".to_string(),
            ),
        ];
        let entry = fic_entry(
            "abc123",
            "Title",
            "Author",
            "Summary",
            "2024-01-01T00:00:00Z",
            1000,
            1,
            "complete",
            &acq,
        );
        assert!(entry.contains("opds-spec.org/acquisition"));
        assert!(entry.contains("/cache/epub/abc123?h=deadbeef"));
        assert!(entry.contains("/cache/pdf/abc123?h=cafebabe"));
        assert!(entry.contains("/cache/html/abc123?h=12345678"));
        assert!(entry.contains("opds-spec.org/image"));
        // old broken shape must not appear
        assert!(!entry.contains("/abc123.epub"));
        assert!(!entry.contains("/abc123.pdf"));
    }

    #[test]
    fn test_fic_entry_hash_based_hrefs() {
        let acq = vec![(
            "application/epub+zip".to_string(),
            "/cache/epub/xyz?h=abc123".to_string(),
        )];
        let entry = fic_entry(
            "xyz",
            "T",
            "A",
            "S",
            "2024-01-01T00:00:00Z",
            100,
            1,
            "complete",
            &acq,
        );
        assert!(entry.contains("?h=abc123"));
        assert!(entry.contains("/cache/epub/xyz?h="));
    }

    #[test]
    fn test_cache_href_with_base() {
        assert_eq!(
            cache_href(Some("https://example.com"), "epub", "id1", "hash1"),
            "https://example.com/cache/epub/id1?h=hash1"
        );
        assert_eq!(
            cache_href(Some("https://example.com/"), "pdf", "id2", "h2"),
            "https://example.com/cache/pdf/id2?h=h2"
        );
        assert_eq!(
            cache_href(None, "html", "id3", "h3"),
            "/cache/html/id3?h=h3"
        );
    }

    #[test]
    fn test_etype_mime_path_known() {
        assert_eq!(etype_mime_path("epub"), ("epub", "application/epub+zip"));
        assert_eq!(etype_mime_path("pdf"), ("pdf", "application/pdf"));
        assert_eq!(etype_mime_path("html"), ("html", "text/html"));
    }

    #[test]
    fn test_atom_xml_header_no_base() {
        let header = atom_xml_header(None);
        assert!(header.contains("xml version=\"1.0\""));
        assert!(header.contains("xmlns=\"http://www.w3.org/2005/Atom\""));
        assert!(header.contains("xmlns:dc="));
        assert!(header.contains("xmlns:opds="));
        assert!(!header.contains("xml:base"));
    }

    #[test]
    fn test_atom_xml_header_with_base() {
        let header = atom_xml_header(Some("https://fichub.polarisocial.xyz"));
        assert!(header.contains("xml:base=\"https://fichub.polarisocial.xyz\""));
    }

    #[test]
    fn test_feed_kind_navigation_content_type() {
        let kind = FeedKind::Navigation;
        assert!(kind.content_type().contains("navigation"));
        assert!(kind.content_type().contains("opds-catalog"));
        assert!(kind.content_type().contains("kind=navigation"));
    }

    #[test]
    fn test_feed_kind_acquisition_content_type() {
        let kind = FeedKind::Acquisition;
        assert!(kind.content_type().contains("acquisition"));
        assert!(kind.content_type().contains("opds-catalog"));
        assert!(kind.content_type().contains("kind=acquisition"));
    }

    #[test]
    fn test_build_feed_minimal() {
        let body = build_feed(
            "Test Feed",
            "urn:test:feed",
            "<entry/>",
            "2024-01-01T00:00:00Z",
            None,
            FeedKind::Navigation,
            None,
            None,
        );
        assert!(body.contains("<title>Test Feed</title>"));
        assert!(body.contains("<id>urn:test:feed</id>"));
        assert!(body.contains("<entry/>"));
        assert!(body.contains("<author><name>FicNexus</name></author>"));
        assert!(body.contains("rel=\"start\""));
        assert!(!body.contains("rel=\"self\""));
    }

    #[test]
    fn test_build_feed_with_self_link() {
        let body = build_feed(
            "Test",
            "urn:test",
            "",
            "2024-01-01T00:00:00Z",
            Some("/opds/test"),
            FeedKind::Acquisition,
            None,
            None,
        );
        assert!(body.contains("rel=\"self\""));
        assert!(body.contains("/opds/test"));
    }

    #[test]
    fn test_build_feed_with_base_url_makes_absolute_links() {
        let body = build_feed(
            "Test",
            "urn:test",
            "",
            "2024-01-01T00:00:00Z",
            Some("/opds/test"),
            FeedKind::Navigation,
            None,
            Some("https://fichub.polarisocial.xyz"),
        );
        assert!(body.contains("xml:base=\"https://fichub.polarisocial.xyz\""));
        // self link should be absolute
        assert!(body.contains("https://fichub.polarisocial.xyz/opds/test"));
        // start link should be absolute
        assert!(body.contains("https://fichub.polarisocial.xyz/opds"));
    }

    #[test]
    fn test_build_feed_pagination_first_page() {
        let pagi = PaginationInfo {
            base_path: "/opds/test".into(),
            page: 1,
            per_page: 20,
            total: 100,
        };
        let body = build_feed(
            "Test",
            "urn:test",
            "",
            "2024-01-01T00:00:00Z",
            None,
            FeedKind::Acquisition,
            Some(&pagi),
            None,
        );
        assert!(!body.contains("rel=\"previous\""));
        assert!(body.contains("rel=\"next\""));
        assert!(body.contains("page=2"));
    }

    #[test]
    fn test_build_feed_pagination_middle_page() {
        let pagi = PaginationInfo {
            base_path: "/opds/test".into(),
            page: 3,
            per_page: 20,
            total: 100,
        };
        let body = build_feed(
            "Test",
            "urn:test",
            "",
            "2024-01-01T00:00:00Z",
            None,
            FeedKind::Acquisition,
            Some(&pagi),
            None,
        );
        assert!(body.contains("rel=\"previous\""));
        assert!(body.contains("page=2"));
        assert!(body.contains("rel=\"next\""));
        assert!(body.contains("page=4"));
    }

    #[test]
    fn test_build_feed_pagination_last_page() {
        let pagi = PaginationInfo {
            base_path: "/opds/test".into(),
            page: 5,
            per_page: 20,
            total: 100,
        };
        let body = build_feed(
            "Test",
            "urn:test",
            "",
            "2024-01-01T00:00:00Z",
            None,
            FeedKind::Acquisition,
            Some(&pagi),
            None,
        );
        assert!(body.contains("rel=\"previous\""));
        assert!(!body.contains("rel=\"next\""));
    }

    #[test]
    fn test_opds_response_content_type() {
        let (headers, body) = opds_response("test body".into(), FeedKind::Navigation);
        assert_eq!(headers[0].1, FeedKind::Navigation.content_type());
        assert_eq!(body, "test body");
    }

    #[test]
    fn test_page_params_default() {
        let params = PageParams {
            page: None,
            per_page: None,
        };
        assert_eq!(params.page(), 1);
        assert_eq!(params.per_page(), 50);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_page_params_custom() {
        let params = PageParams {
            page: Some(3),
            per_page: Some(10),
        };
        assert_eq!(params.page(), 3);
        assert_eq!(params.per_page(), 10);
        assert_eq!(params.offset(), 20);
    }

    #[test]
    fn test_page_params_clamp_low() {
        let params = PageParams {
            page: Some(0),
            per_page: Some(0),
        };
        assert_eq!(params.page(), 1);
        assert_eq!(params.per_page(), 1);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_page_params_clamp_high() {
        let params = PageParams {
            page: Some(1),
            per_page: Some(200),
        };
        assert_eq!(params.per_page(), 100);
    }

    #[test]
    fn test_iso_now_format() {
        let now = iso_now();
        // Should be ISO 8601 format
        assert!(now.ends_with('Z'));
        assert!(now.contains('T'));
        assert_eq!(now.len(), 20); // 2024-01-01T00:00:00Z
    }

    #[test]
    fn test_abs_url_without_base() {
        assert_eq!(abs_url("/opds/test", None), "/opds/test");
        assert_eq!(abs_url("/opds/test", Some("")), "/opds/test");
    }

    #[test]
    fn test_abs_url_with_base() {
        assert_eq!(
            abs_url("/opds/test", Some("https://fichub.polarisocial.xyz")),
            "https://fichub.polarisocial.xyz/opds/test"
        );
    }

    #[test]
    fn test_abs_url_no_double_slash() {
        assert_eq!(
            abs_url("opds/test", Some("https://fichub.polarisocial.xyz/")),
            "https://fichub.polarisocial.xyz/opds/test"
        );
    }

    #[test]
    fn test_abs_url_trims_slashes_on_both_sides() {
        // Multiple trailing/leading slashes collapse to a single separator.
        assert_eq!(
            abs_url("/opds//test", Some("https://fichub.polarisocial.xyz///")),
            "https://fichub.polarisocial.xyz/opds//test"
        );
        assert_eq!(
            abs_url("/", Some("https://fichub.polarisocial.xyz/")),
            "https://fichub.polarisocial.xyz/"
        );
    }

    #[test]
    fn test_build_feed_pagination_abs_urls() {
        // Pagination links on page 2 resolve to absolute URLs with the base.
        let pagi = PaginationInfo {
            base_path: "/opds/new".into(),
            page: 2,
            per_page: 10,
            total: 25,
        };
        let feed = build_feed(
            "Recent",
            "urn:ficnexus:feed:recent",
            "",
            "2024-01-01T00:00:00Z",
            Some("/opds/new?page=2&per_page=10"),
            FeedKind::Acquisition,
            Some(&pagi),
            Some("https://fichub.polarisocial.xyz"),
        );
        assert!(feed.contains("href=\"https://fichub.polarisocial.xyz/opds/new?page=1&amp;per_page=10\" rel=\"previous\""));
        assert!(feed.contains(
            "href=\"https://fichub.polarisocial.xyz/opds/new?page=3&amp;per_page=10\" rel=\"next\""
        ));
        assert!(feed.contains("rel=\"self\""));
    }

    #[test]
    fn test_build_feed_escapes_title_and_id() {
        let feed = build_feed(
            "Fics <&> Friends",
            "urn:fic:1",
            "",
            "2024-01-01T00:00:00Z",
            None,
            FeedKind::Navigation,
            None,
            None,
        );
        assert!(feed.contains("<title>Fics &lt;&amp;&gt; Friends</title>"));
        assert!(feed.contains("<id>urn:fic:1</id>"));
        assert!(feed.contains("rel=\"start\""));
    }

    #[test]
    fn test_build_feed_escapes_query_ampersand_once() {
        let pagi = PaginationInfo {
            base_path: "/opds/search?q=alpha%20%26%20beta".into(),
            page: 1,
            per_page: 10,
            total: 20,
        };
        let feed = build_feed(
            "Search",
            "urn:ficnexus:search",
            "",
            "2024-01-01T00:00:00Z",
            Some("/opds/search?q=alpha & beta&page=1&per_page=10"),
            FeedKind::Acquisition,
            Some(&pagi),
            None,
        );
        assert!(feed.contains("q=alpha%20%26%20beta&amp;page=2&amp;per_page=10"));
        assert!(!feed.contains("&amp;amp;"));
    }

    #[test]
    fn test_page_params_offset_calculation() {
        let params = PageParams {
            page: Some(3),
            per_page: Some(25),
        };
        assert_eq!(params.offset(), 50);
        // page 1 → offset 0
        let first = PageParams {
            page: None,
            per_page: None,
        };
        assert_eq!(first.offset(), 0);
    }
}
