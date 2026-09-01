use std::fs;
use std::path::{Path, PathBuf};

use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use md5::{Digest, Md5};
use uuid::Uuid;

use crate::export::ExportError;
use crate::scrape::{Chapter, FicMetadata};

/// Generate an EPUB file for the given fic metadata and chapters.
///
/// Creates a UUID-named subdirectory inside `tmp_dir`, writes the EPUB there,
/// computes its MD5 hash, and returns `(path_to_epub, md5_hex)`.
pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // ---- work directory ---------------------------------------------------
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // ---- builder setup ----------------------------------------------------
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    builder.metadata("title", &meta.title)?;
    builder.metadata("author", &meta.author)?;
    builder.metadata("lang", "en")?;
    builder.metadata("description", &meta.desc)?;

    // ---- inline CSS -------------------------------------------------------
    // Book-style paragraphs: first line indented, no extra margin between
    // paragraphs (issue #21). First paragraph after a heading keeps margin-top.
    let css = concat!(
        "body{font-family:serif;line-height:1.5;}",
        "h2{text-align:center;}",
        "p{margin:0 0 0.8em 0;text-indent:1.5em;}",
        "h1+p, h2+p, h3+p, h4+p, p:first-of-type{text-indent:0;}"
    );
    // epub-builder writes this as "stylesheet.css" automatically
    builder.stylesheet(css.as_bytes())?;

    // ---- introduction page ------------------------------------------------
    let intro_html = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Introduction</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h1>{title}</h1>
    <h2>by {author}</h2>
    <table>
        <tr><td>Words:</td><td>{words}</td></tr>
        <tr><td>Chapters:</td><td>{chapters}</td></tr>
        <tr><td>Status:</td><td>{status}</td></tr>
        <tr><td>Published:</td><td>{published}</td></tr>
        <tr><td>Updated:</td><td>{updated}</td></tr>
    </table>
    <hr/>
    <p>{desc}</p>
</body>
</html>"#,
        title = escape_html(&meta.title),
        author = escape_html(&meta.author),
        words = meta.words,
        chapters = meta.chapters,
        status = escape_html(&meta.status),
        published = format_timestamp(meta.published),
        updated = format_timestamp(meta.updated),
        desc = escape_html(&meta.desc),
    );

    builder.add_content(
        EpubContent::new("introduction.xhtml", intro_html.as_bytes())
            .title("Introduction"),
    )?;

    // ---- chapters ---------------------------------------------------------
    for chapter in chapters {
        let chapter_html = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
            title = escape_html(&chapter.title),
            content = chapter.content,
        );

        let filename = format!("chapter_{}.xhtml", chapter.chapter_id);
        builder.add_content(
            EpubContent::new(filename.as_str(), chapter_html.as_bytes())
                .title(&chapter.title),
        )?;
    }

    // ---- write EPUB file --------------------------------------------------
    let epub_path = work_dir.join("output.epub");
    let file = fs::File::create(&epub_path)?;
    builder.generate(file)?;

    // ---- MD5 hash ---------------------------------------------------------
    let epub_data = fs::read(&epub_path)?;
    let md5_hex = Md5::digest(&epub_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((epub_path, md5_hex))
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
