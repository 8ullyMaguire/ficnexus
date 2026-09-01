use std::fs;
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};
use uuid::Uuid;

use crate::export::ExportError;
use crate::scrape::{Chapter, FicMetadata};

/// Generate a FictionBook 2.1 (.fb2) export for the given fic metadata and
/// chapters.
///
/// Produces a well-formed FB2 XML document (UTF-8) with a `<description>`
/// block (title-info + document-info) and a `<body>` of `<section>` elements.
/// All text content is XML-escaped.
///
/// Returns `(fb2_path, md5_hex)`.
pub async fn create_fb2(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    let fb2 = build_fb2_xml(meta, chapters);

    let fb2_path = work_dir.join("output.fb2");
    fs::write(&fb2_path, fb2.as_bytes())?;

    let data = fs::read(&fb2_path)?;
    let md5_hex = Md5::digest(&data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((fb2_path, md5_hex))
}

/// Build the full FB2 XML document as a string.
fn build_fb2_xml(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut out = String::with_capacity(64 * 1024);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<FictionBook xmlns=\"http://www.gribuser.ru/xml/fictionbook/2.0\" xmlns:l=\"http://www.w3.org/1999/xlink\">\n");

    // ---- description ------------------------------------------------------
    out.push_str("  <description>\n");
    out.push_str("    <title-info>\n");
    out.push_str(&format!(
        "      <genre>fanfiction</genre>\n      <author><first-name>{}</first-name></author>\n",
        escape_xml(&meta.author)
    ));
    out.push_str(&format!(
        "      <book-title>{}</book-title>\n",
        escape_xml(&meta.title)
    ));
    if !meta.desc.is_empty() {
        out.push_str(&format!(
            "      <annotation>{}</annotation>\n",
            escape_xml(&strip_html(&meta.desc))
        ));
    }
    out.push_str(&format!(
        "      <lang>en</lang>\n      <src-lang>en</src-lang>\n      <keywords>fanfiction</keywords>\n"
    ));
    out.push_str("    </title-info>\n");

    out.push_str("    <document-info>\n");
    out.push_str("      <author><nickname>fICHub</nickname></author>\n");
    out.push_str("      <program-used>fICHub</program-used>\n");
    out.push_str(&format!(
        "      <date value=\"{}\">{}</date>\n",
        today(),
        today()
    ));
    out.push_str(&format!(
        "      <src-url>{}</src-url>\n",
        escape_xml(&meta.source)
    ));
    out.push_str("      <id>");
    out.push_str(&sanitize_id(&meta.url_id));
    out.push_str("</id>\n");
    out.push_str("      <version>1.0</version>\n");
    out.push_str("    </document-info>\n");
    out.push_str("  </description>\n");

    // ---- body -------------------------------------------------------------
    out.push_str("  <body>\n");
    out.push_str(&format!(
        "    <title><p>{}</p></title>\n",
        escape_xml(&meta.title)
    ));
    for chapter in chapters {
        out.push_str("    <section>\n");
        out.push_str(&format!(
            "      <title><p>{}</p></title>\n",
            escape_xml(&chapter.title)
        ));
        for para in html_to_paragraphs(&chapter.content) {
            out.push_str(&format!("      <p>{}</p>\n", escape_xml(&para)));
        }
        out.push_str("    </section>\n");
    }
    out.push_str("  </body>\n");
    out.push_str("</FictionBook>\n");
    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Escape XML special characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Strip HTML tags from a string, preserving paragraph boundaries as
/// newlines (mirrors the txt writer so FB2 paragraphs line up with them).
fn strip_html(input: &str) -> String {
    use regex_lite::Regex;

    let block_re = Regex::new(r"(?i)</?(?:p|div|h[1-6]|li|tr)[^>]*>|<br\s*/?>").unwrap();
    let s = block_re.replace_all(input, "\n");
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let s = tag_re.replace_all(&s, "");
    let s = decode_entities(&s);
    let lines: Vec<String> = s
        .split('\n')
        .map(|line| {
            line.split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ")
        })
        .filter(|line| !line.is_empty())
        .collect();
    lines.join("\n")
}

/// Split HTML into plain-text paragraphs (one entry per paragraph).
fn html_to_paragraphs(input: &str) -> Vec<String> {
    strip_html(input)
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Decode common HTML entities to their plain-text equivalents.
fn decode_entities(input: &str) -> String {
    let mut s = input.to_string();
    s = s.replace("&amp;", "&");
    s = s.replace("&lt;", "<");
    s = s.replace("&gt;", ">");
    s = s.replace("&quot;", "\"");
    s = s.replace("&#39;", "'");
    s = s.replace("&apos;", "'");
    s = s.replace("&#x27;", "'");
    s = s.replace("&#x2F;", "/");
    s = s.replace("&nbsp;", " ");
    s = s.replace("&#xA;", "\n");
    s = s.replace("&#10;", "\n");
    s
}

fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// FB2 document ids must match `[A-Za-z0-9_\-]`; strip anything else.
fn sanitize_id(url_id: &str) -> String {
    url_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(60)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta() -> FicMetadata {
        FicMetadata {
            url_id: "test123".into(),
            title: "Test Fic <&> \"Quoted\"".into(),
            author: "Test Author".into(),
            chapters: 1,
            words: 1000,
            desc: "<p>A great story</p>".into(),
            published: 1700000000000,
            updated: 1700000000000,
            status: "complete".into(),
            source: "https://example.com/fiction/123".into(),
            source_id: 1,
            author_id: 42,
            author_url: "https://example.com/u/test".into(),
            author_local_id: "test".into(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        }
    }

    fn make_chapters() -> Vec<Chapter> {
        vec![Chapter {
            chapter_id: 1,
            title: "Chapter One".into(),
            content: "<p>Hello <b>world</b> &amp; friends.</p><p>Second para.</p>".into(),
        }]
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a<b>&\"'"), "a&lt;b&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("abc-123_def"), "abc-123_def");
        assert_eq!(sanitize_id("a b/c!!"), "abc");
    }

    #[test]
    fn test_html_to_paragraphs() {
        let input = "<p>One</p><p>Two <b>bold</b></p>";
        assert_eq!(html_to_paragraphs(input), vec!["One", "Two bold"]);
    }

    #[test]
    fn test_build_fb2_xml_structure() {
        let xml = build_fb2_xml(&make_meta(), &make_chapters());
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<FictionBook"));
        assert!(xml.contains("<book-title>"));
        assert!(xml.contains("&lt;&amp;&gt;"));
        assert!(xml.contains("<section>"));
        assert!(xml.contains("Hello world &amp; friends."));
        assert!(xml.contains("</FictionBook>"));
        // No raw angle-bracket HTML leaks into the XML
        assert!(!xml.contains("<b>"));
    }

    #[test]
    fn test_build_fb2_xml_is_well_formed() {
        let xml = build_fb2_xml(&make_meta(), &make_chapters());
        // quick-xml round-trip: parse succeeds ⇒ well-formed XML
        let mut reader = quick_xml::Reader::from_str(&xml);
        let mut depth = 0usize;
        let mut saw_root = false;
        let mut saw_root_close = false;
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Start(_)) => {
                    depth += 1;
                    saw_root = true;
                }
                Ok(quick_xml::events::Event::End(_)) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        saw_root_close = true;
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("FB2 XML parse error: {e}"),
            }
        }
        assert!(saw_root, "must have a root element");
        assert!(saw_root_close, "root element must close");
        assert_eq!(depth, 0, "balanced tags");
    }

    #[test]
    fn test_create_fb2_produces_file() {
        let tmp = std::env::temp_dir().join("fichub_test_fb2");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let (path, md5_hex) = rt
            .block_on(create_fb2(&make_meta(), &make_chapters(), &tmp))
            .unwrap();

        assert!(path.exists());
        assert_eq!(md5_hex.len(), 32);
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("<FictionBook"));
        assert!(contents.contains("Hello world &amp; friends."));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_create_fb2_md5_stable() {
        let tmp = std::env::temp_dir().join("fichub_test_fb2_md5");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_, a) = rt.block_on(create_fb2(&make_meta(), &make_chapters(), &tmp)).unwrap();
        let (_, b) = rt.block_on(create_fb2(&make_meta(), &make_chapters(), &tmp)).unwrap();
        assert_eq!(a, b);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
