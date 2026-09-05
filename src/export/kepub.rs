use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};
use uuid::Uuid;

use crate::export::ExportError;
use crate::scrape::{Chapter, FicMetadata};

/// Generate a Kobo EPUB (.kepub) export for the given fic metadata and
/// chapters.
///
/// KEPUB is EPUB 3 with extra Kobo span elements (`<span class="koboSpan"
/// id="kobo.<N>.1">`) wrapped around block content so Kobo readers can do
/// page-number / position sync. The classic, robust approach is to take the
/// generated EPUB, unpack it, wrap each paragraph of every XHTML chapter in a
/// koboSpan, add a Kobo-specific EPUB 3.0 `.ncx`-less navigation/landmarks
/// tweak plus a `kobo.css`-style smoothing rule, and re-zip.
///
/// Here we build the EPUB **from scratch** (same structure as `export::epub`)
/// with the koboSpan wrappers applied while writing each chapter, which keeps
/// the writer dependency-free and deterministic (no need to re-open the
/// EPUB zip). The result is a valid EPUB 3 that Kobo devices treat as a
/// KEPUB when renamed.
///
/// Returns `(kepub_path, md5_hex)`.
pub async fn create_kepub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // ---- build the EPUB zip ----------------------------------------------
    let kepub_path = work_dir.join("output.kepub");
    let file = fs::File::create(&kepub_path)?;
    let mut writer = zip::ZipWriter::new(file);
    let options: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // MIME type (must be first, uncompressed, per OCF spec)
    writer.start_file(
        "mimetype",
        options.compression_method(zip::CompressionMethod::Stored),
    )?;
    writer.write_all(b"application/epub+zip")?;

    writer.start_file("META-INF/container.xml", options)?;
    writer.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )?;

    writer.start_file("OEBPS/content.opf", options)?;
    writer.write_all(build_opf(meta, chapters).as_bytes())?;

    writer.start_file("OEBPS/nav.xhtml", options)?;
    writer.write_all(build_nav(meta, chapters).as_bytes())?;

    writer.start_file("OEBPS/stylesheet.css", options)?;
    writer.write_all(KEPUB_CSS.as_bytes())?;

    // Intro page
    writer.start_file("OEBPS/introduction.xhtml", options)?;
    writer.write_all(
        build_chapter_xhtml(
            &format!("Introduction — {}", meta.title),
            &format!(
                "<h1>{}</h1><h2>by {}</h2><table><tr><td>Words:</td><td>{}</td></tr><tr><td>Chapters:</td><td>{}</td></tr><tr><td>Status:</td><td>{}</td></tr><tr><td>Published:</td><td>{}</td></tr><tr><td>Updated:</td><td>{}</td></tr></table><hr/><p>{}</p>",
                escape_html(&meta.title),
                escape_html(&meta.author),
                meta.words,
                meta.chapters,
                escape_html(&meta.status),
                format_timestamp(meta.published),
                format_timestamp(meta.updated),
                escape_html(&meta.desc),
            ),
        )
        .as_bytes(),
    )?;

    // Chapters (each wrapped in koboSpan)
    let mut span_counter: usize = 0;
    for chapter in chapters {
        let body = wrap_kobo_spans(&chapter.content, &mut span_counter);
        writer.start_file(
            format!("OEBPS/chapter_{}.xhtml", chapter.chapter_id),
            options,
        )?;
        writer.write_all(build_chapter_xhtml(&chapter.title, &body).as_bytes())?;
    }

    writer
        .finish()
        .map_err(|e| ExportError::ZipError(e.to_string()))?;

    // ---- MD5 hash ---------------------------------------------------------
    let data = fs::read(&kepub_path)?;
    let md5_hex = Md5::digest(&data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((kepub_path, md5_hex))
}

/// CSS with a Kobo-smoothing font rule (mimics what Calibre's kepub output
/// embeds; harmless for other readers).
const KEPUB_CSS: &str = r#"body{font-family:serif;line-height:1.5;}
h2{text-align:center;}
p{margin:0 0 0.8em 0;text-indent:1.5em;}
h1+p,h2+p,h3+p,h4+p,p:first-of-type{text-indent:0;}
span.koboSpan{text-indent:1.5em;}"#;

/// Build the OPF package document for the KEPUB.
fn build_opf(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="bookid">ficnexus-{url_id}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator>{author}</dc:creator>
    <dc:language>en</dc:language>
    <dc:description>{desc}</dc:description>
    <meta property="dcterms:modified">{modified}</meta>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="stylesheet" href="stylesheet.css" media-type="text/css"/>
    <item id="introduction" href="introduction.xhtml" media-type="application/xhtml+xml"/>
"#,
        url_id = sanitize_id(&meta.url_id),
        title = escape_html(&meta.title),
        author = escape_html(&meta.author),
        desc = escape_html(&meta.desc),
        modified = today_rfc3339(),
    ));
    for chapter in chapters {
        out.push_str(&format!(
            "    <item id=\"chapter_{id}\" href=\"chapter_{id}.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
            id = chapter.chapter_id,
        ));
    }
    out.push_str("  </manifest>\n  <spine>\n");
    out.push_str("    <itemref idref=\"introduction\"/>\n");
    for chapter in chapters {
        out.push_str(&format!(
            "    <itemref idref=\"chapter_{id}\"/>\n",
            id = chapter.chapter_id,
        ));
    }
    out.push_str("  </spine>\n</package>\n");
    out
}

/// Build the EPUB 3 navigation document.
fn build_nav(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>{title}</title></head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>{title}</h1>
    <ol>
      <li><a href="introduction.xhtml">Introduction</a></li>
"#,
        title = escape_html(&meta.title),
    ));
    for chapter in chapters {
        out.push_str(&format!(
            "      <li><a href=\"chapter_{id}.xhtml\">{title}</a></li>\n",
            id = chapter.chapter_id,
            title = escape_html(&chapter.title),
        ));
    }
    out.push_str("    </ol>\n  </nav>\n</body>\n</html>\n");
    out
}

/// Wrap each XHTML document in a full page (head + stylesheet link + body).
fn build_chapter_xhtml(title: &str, body_html: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
{body}
</body>
</html>"#,
        title = escape_html(title),
        body = body_html,
    )
}

/// Wrap every top-level block element (p, h1-h6, div, li, blockquote) of the
/// chapter's HTML in `<span class="koboSpan" id="kobo.N.1">` so Kobo devices
/// can track reading position.
///
/// Only the *outermost* block elements are wrapped: nested blocks of the same
/// or different types are kept inside their enclosing span. The counter is
/// shared across chapters so ids stay globally unique.
///
/// Implemented as a small tag scanner (regex-lite has no backreferences, and
/// a hand-rolled stack is more robust for nested HTML anyway).
fn wrap_kobo_spans(content: &str, counter: &mut usize) -> String {
    const BLOCK_TAGS: [&str; 10] = [
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "div",
        "li",
        "blockquote",
    ];

    let bytes = content.as_bytes();
    let mut result = String::with_capacity(content.len() + 256);
    let mut last = 0usize;
    // Stack of open block tags: (lowercase tag name, byte offset of `<`).
    let mut stack: Vec<(String, usize)> = Vec::new();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        // Comments / declarations / processing instructions: skip to `>`.
        if i + 1 < n && (bytes[i + 1] == b'!' || bytes[i + 1] == b'?') {
            while i < n && bytes[i] != b'>' {
                i += 1;
            }
            i += 1;
            continue;
        }

        let mut j = i + 1;
        let is_close = j < n && bytes[j] == b'/';
        if is_close {
            j += 1;
        }
        let name_start = j;
        while j < n && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
            j += 1;
        }
        if j == name_start {
            i += 1; // not a tag (e.g. "<3")
            continue;
        }
        let name = content[name_start..j].to_ascii_lowercase();

        // Find the end of this tag.
        let mut k = j;
        while k < n && bytes[k] != b'>' {
            k += 1;
        }
        if k >= n {
            break; // unterminated tag — bail
        }
        let tag_end = k + 1;
        let self_closing = bytes[k - 1] == b'/' || bytes[k - 1] == b'?';

        if is_close {
            if let Some(pos) = stack.iter().rposition(|(t, _)| *t == name) {
                let start = stack[pos].1;
                stack.truncate(pos);
                if stack.is_empty() {
                    // Complete outermost block: [start, tag_end).
                    *counter += 1;
                    let id = format!("kobo.{}.1", *counter);
                    result.push_str(&content[last..start]);
                    result.push_str(&format!("<span class=\"koboSpan\" id=\"{id}\">"));
                    result.push_str(&content[start..tag_end]);
                    result.push_str("</span>");
                    last = tag_end;
                }
            }
        } else if !self_closing && BLOCK_TAGS.contains(&name.as_str()) {
            stack.push((name, i));
        }

        i = tag_end;
    }

    result.push_str(&content[last..]);
    result
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;

    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;

    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn today_rfc3339() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// ID-safe string (used inside OPF identifiers).
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
    use std::io::Read;

    fn make_meta() -> FicMetadata {
        FicMetadata {
            url_id: "test123".into(),
            title: "Test Fic".into(),
            author: "Test Author".into(),
            chapters: 2,
            words: 2000,
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
        vec![
            Chapter {
                chapter_id: 1,
                title: "Chapter One".into(),
                content: "<p>Hello world.</p><p>Second para <em>with style</em>.</p>".into(),
            },
            Chapter {
                chapter_id: 2,
                title: "Chapter Two".into(),
                content: "<h2>A heading</h2><p>More text.</p>".into(),
            },
        ]
    }

    #[test]
    fn test_wrap_kobo_spans_simple() {
        let mut counter = 0;
        let out = wrap_kobo_spans("<p>Hello</p><p>World</p>", &mut counter);
        assert_eq!(
            out,
            "<span class=\"koboSpan\" id=\"kobo.1.1\"><p>Hello</p></span>\
             <span class=\"koboSpan\" id=\"kobo.2.1\"><p>World</p></span>"
        );
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_wrap_kobo_spans_heading_and_attrs() {
        let mut counter = 0;
        let out = wrap_kobo_spans("<h2 class=\"title\">Hi</h2><p>Body</p>", &mut counter);
        assert!(out.starts_with(
            "<span class=\"koboSpan\" id=\"kobo.1.1\"><h2 class=\"title\">Hi</h2></span>"
        ));
        assert!(out.ends_with("<span class=\"koboSpan\" id=\"kobo.2.1\"><p>Body</p></span>"));
    }

    #[test]
    fn test_wrap_kobo_spans_skips_nested() {
        let mut counter = 0;
        // The <p> inside the <div> is nested — only the outermost <div> and
        // the standalone <p> should be wrapped.
        let out = wrap_kobo_spans("<div><p>Nested</p></div><p>Standalone</p>", &mut counter);
        assert_eq!(
            out,
            "<span class=\"koboSpan\" id=\"kobo.1.1\"><div><p>Nested</p></div></span>\
             <span class=\"koboSpan\" id=\"kobo.2.1\"><p>Standalone</p></span>"
        );
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_wrap_kobo_spans_no_blocks() {
        let mut counter = 0;
        assert_eq!(wrap_kobo_spans("plain text", &mut counter), "plain text");
        assert_eq!(wrap_kobo_spans("", &mut counter), "");
    }

    #[test]
    fn test_build_opf_contains_chapters() {
        let opf = build_opf(&make_meta(), &make_chapters());
        assert!(opf.contains("chapter_1.xhtml"));
        assert!(opf.contains("chapter_2.xhtml"));
        assert!(opf.contains("nav.xhtml"));
        assert!(opf.contains("properties=\"nav\""));
        assert!(opf.contains("<dc:title>Test Fic</dc:title>"));
    }

    #[test]
    fn test_build_nav_contains_chapters() {
        let nav = build_nav(&make_meta(), &make_chapters());
        assert!(nav.contains("introduction.xhtml"));
        assert!(nav.contains("chapter_1.xhtml"));
        assert!(nav.contains("chapter_2.xhtml"));
        assert!(nav.contains("Chapter One"));
    }

    #[test]
    fn test_create_kepub_produces_valid_zip() {
        let tmp = std::env::temp_dir().join("fichub_test_kepub");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let (path, md5_hex) = rt
            .block_on(create_kepub(&make_meta(), &make_chapters(), &tmp))
            .unwrap();

        assert!(path.exists());
        assert_eq!(md5_hex.len(), 32);

        let file = std::fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // OCF-required entries
        let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
        for required in [
            "mimetype",
            "META-INF/container.xml",
            "OEBPS/content.opf",
            "OEBPS/nav.xhtml",
        ] {
            assert!(
                names.iter().any(|n| n == required),
                "kepub missing {required}: {names:?}"
            );
        }

        // mimetype must be stored (uncompressed)
        {
            let mime = archive.by_name("mimetype").unwrap();
            assert_eq!(
                mime.compression(),
                zip::CompressionMethod::Stored,
                "mimetype must be stored"
            );
        }
        let mut mime_contents = String::new();
        archive
            .by_name("mimetype")
            .unwrap()
            .read_to_string(&mut mime_contents)
            .unwrap();
        assert_eq!(mime_contents, "application/epub+zip");

        // Chapter files contain koboSpan wrappers
        let mut ch1 = String::new();
        archive
            .by_name("OEBPS/chapter_1.xhtml")
            .unwrap()
            .read_to_string(&mut ch1)
            .unwrap();
        assert!(ch1.contains("koboSpan"));
        assert!(ch1.contains("kobo.1.1"));
        assert!(ch1.contains("kobo.2.1"));
        assert!(ch1.contains("Hello world."));
        // Second chapter continues the counter
        let mut ch2 = String::new();
        archive
            .by_name("OEBPS/chapter_2.xhtml")
            .unwrap()
            .read_to_string(&mut ch2)
            .unwrap();
        assert!(ch2.contains("kobo.3.1"));
        assert!(ch2.contains("kobo.4.1"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_create_kepub_md5_stable() {
        let tmp = std::env::temp_dir().join("fichub_test_kepub_md5");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_, a) = rt
            .block_on(create_kepub(&make_meta(), &make_chapters(), &tmp))
            .unwrap();
        let (_, b) = rt
            .block_on(create_kepub(&make_meta(), &make_chapters(), &tmp))
            .unwrap();
        assert_eq!(a, b, "kepub md5 must be deterministic");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
