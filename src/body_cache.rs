//! Body cache: persists scraped fic HTML + extracted chapters as files on
//! disk so the site acts as a cache of all gathered fanfiction — no
//! re-scrape needed for repeat exports.
//!
//! Two artifacts per fic:
//! - RAW HTML (the source page as fetched) — sharded + versioned so a bad
//!   body extraction can be re-done from the saved HTML without re-scraping.
//! - Extracted chapters (JSON) — what exports actually render.
//!
//! Layout (two-level hex sharding on `url_id`, git-objects style):
//!   BODIES_DIR/<url_id[0:2]>/<url_id[2:4]>/<url_id>.v{version}.html
//!   BODIES_DIR/<url_id[0:2]>/<url_id[2:4]>/<url_id>.v{version}.json
//!
//! BODIES_DIR defaults to /public/literature/fichub/bodies (the
//! ThinkCentre-attached drive, NOT the database).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::scrape::Chapter;

/// A persisted body blob. Chapters are stored as an ordered list so exports
/// can re-render exactly what was scraped (or curated).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyBlob {
    pub url_id: String,
    pub chapters: Vec<Chapter>,
    pub saved_at_ms: i64,
    pub source: Option<String>,
}

fn shard_dir(root: &Path, url_id: &str) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    if id.len() >= 4 {
        root.join(&id[..2]).join(&id[2..4])
    } else {
        root.join("_").join("_")
    }
}

fn html_path(root: &Path, url_id: &str, version: i32) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    shard_dir(root, url_id).join(format!("{id}.v{version}.html"))
}

fn json_path(root: &Path, url_id: &str, version: i32) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    shard_dir(root, url_id).join(format!("{id}.v{version}.json"))
}

/// Current version for a fic — 1 unless a curator fix bumped it.
pub fn current_version(config: &Config, url_id: &str) -> i32 {
    let dir = shard_dir(&config.body_cache_dir, url_id);
    let id = url_id.to_ascii_lowercase();
    let mut max = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(rest) = name.strip_prefix(&format!("{id}.v")) {
                if let Some(ver) = rest.strip_suffix(".html") {
                    if let Ok(n) = ver.parse::<i32>() {
                        max = max.max(n);
                    }
                } else if let Some(ver) = rest.strip_suffix(".json") {
                    if let Ok(n) = ver.parse::<i32>() {
                        max = max.max(n);
                    }
                }
            }
        }
    }
    max.max(1)
}

/// Save raw HTML for a fic at the given version (atomic tmp+rename).
pub fn save_html(
    config: &Config,
    url_id: &str,
    html: &str,
    version: i32,
) -> std::io::Result<PathBuf> {
    let dir = shard_dir(&config.body_cache_dir, url_id);
    std::fs::create_dir_all(&dir)?;
    let path = html_path(&config.body_cache_dir, url_id, version);
    let tmp = dir.join(format!(
        "{url_id}.v{version}.html.tmp.{}",
        std::process::id()
    ));
    std::fs::write(&tmp, html)?;
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}

/// Load raw HTML for a fic (any version, newest wins). Returns None if not
/// cached.
pub fn load_html(config: &Config, url_id: &str) -> Option<String> {
    let v = current_version(config, url_id);
    for ver in (1..=v).rev() {
        let path = html_path(&config.body_cache_dir, url_id, ver);
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                return String::from_utf8(bytes).ok();
            }
        }
    }
    None
}

/// Save (or overwrite) the extracted chapters for a fic at a version.
pub fn save_body(
    config: &Config,
    url_id: &str,
    chapters: &[Chapter],
    source: Option<String>,
    version: i32,
) -> std::io::Result<PathBuf> {
    let dir = shard_dir(&config.body_cache_dir, url_id);
    std::fs::create_dir_all(&dir)?;
    let blob = BodyBlob {
        url_id: url_id.to_string(),
        chapters: chapters.to_vec(),
        saved_at_ms: chrono::Utc::now().timestamp_millis(),
        source,
    };
    let path = json_path(&config.body_cache_dir, url_id, version);
    let tmp = dir.join(format!(
        "{url_id}.v{version}.json.tmp.{}",
        std::process::id()
    ));
    let bytes = serde_json::to_vec_pretty(&blob).map_err(std::io::Error::other)?;
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}

/// Load a fic's chapters from the blob store (newest version). Returns None
/// if not cached.
pub fn load_body(config: &Config, url_id: &str) -> Option<Vec<Chapter>> {
    let v = current_version(config, url_id);
    for ver in (1..=v).rev() {
        let path = json_path(&config.body_cache_dir, url_id, ver);
        if !path.exists() {
            continue;
        }
        let bytes = std::fs::read(&path).ok()?;
        let blob: BodyBlob = serde_json::from_slice(&bytes).ok()?;
        if blob.url_id == url_id {
            return Some(blob.chapters);
        }
    }
    None
}

/// Bump a fic to a new version (curator fix / re-scrape). Writes the next
/// version; best-effort deletes older versions (bounded disk, no GC job).
pub fn bump_version(config: &Config, url_id: &str) -> i32 {
    let v = current_version(config, url_id) + 1;
    let dir = shard_dir(&config.body_cache_dir, url_id);
    let id = url_id.to_ascii_lowercase();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(&id) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    v
}

/// Delete all cached artifacts for a fic (curator correction → re-scrape).
pub fn delete_body(config: &Config, url_id: &str) -> std::io::Result<()> {
    let dir = shard_dir(&config.body_cache_dir, url_id);
    let id = url_id.to_ascii_lowercase();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(&id) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let _ = std::fs::remove_dir(&dir);
    Ok(())
}

/// True when a body blob exists for the fic.
pub fn has_body(config: &Config, url_id: &str) -> bool {
    json_path(
        &config.body_cache_dir,
        url_id,
        current_version(config, url_id),
    )
    .exists()
}

/// The directory containing body blobs.
pub fn body_dir(config: &Config) -> &Path {
    &config.body_cache_dir
}

/// Plain-text version of a fic's cached body: ALL chapters' HTML stripped
/// to text, whitespace collapsed, joined with paragraph breaks. Returns
/// None when no body blob is cached (or it fails to parse).
pub fn body_plain_text(config: &Config, url_id: &str) -> Option<String> {
    let chapters = load_body(config, url_id)?;
    if chapters.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(chapters.len());
    for ch in &chapters {
        let text = strip_html_to_text(&ch.content);
        if !text.is_empty() {
            parts.push(text);
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("\n\n"))
}

/// Maintain the `body_text_search` tsvector column for a fic from its
/// cached body. Best-effort: never fails the caller — a missing/unparseable
/// body or a DB error is logged and swallowed. An empty/None body clears
/// the column (cache deleted → fic stops matching body searches).
pub async fn index_body_text(db: &sqlx::PgPool, config: &Config, url_id: &str) {
    let text = body_plain_text(config, url_id);
    let tsvector = match text {
        Some(ref t) if !t.trim().is_empty() => Some(t.clone()),
        _ => None,
    };
    let res = sqlx::query(
        "UPDATE fic_info SET body_text_search = CASE WHEN $2::text IS NULL THEN NULL ELSE to_tsvector('english', $2) END WHERE id = $1",
    )
    .bind(url_id)
    .bind(tsvector)
    .execute(db)
    .await;
    if let Err(e) = res {
        tracing::warn!(url_id, error = %e, "body_text_search index update failed");
    }
}

/// Strip HTML tags to plain text (block tags → paragraph breaks, entities
/// decoded, whitespace collapsed) — mirrors src/export/txt.rs::strip_html.
fn strip_html_to_text(input: &str) -> String {
    use regex_lite::Regex;

    let block_re = Regex::new(r"(?i)</?(?:p|div|h[1-6])[^>]*>|<br\s*/?>").unwrap();
    let s = block_re.replace_all(input, "\n\n");

    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let s = tag_re.replace_all(&s, "");
    let mut result = s.to_string();

    // Decode the common HTML entities (after tag stripping, so `<3` in
    // text is safe).
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");
    result = result.replace("&apos;", "'");
    result = result.replace("&#x27;", "'");
    result = result.replace("&#x2F;", "/");
    result = result.replace("&nbsp;", " ");
    result = result.replace("&#xA;", "\n");
    result = result.replace("&#10;", "\n");

    // Collapse whitespace per line, then collapse 3+ newlines into 2.
    let lines: Vec<String> = result
        .split('\n')
        .map(|line| line.split_whitespace().collect::<Vec<&str>>().join(" "))
        .collect();
    let mut cleaned = lines.join("\n");
    while cleaned.contains("\n\n\n") {
        cleaned = cleaned.replace("\n\n\n", "\n\n");
    }
    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn tmp_config() -> Config {
        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://localhost/test_db");
            std::env::set_var("REDIS_URL", "redis://localhost/0");
        }
        static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut c = Config::from_env();
        let dir =
            std::env::temp_dir().join(format!("fichub-body-cache-test-{}-{n}", std::process::id()));
        c.body_cache_dir = dir.clone();
        let _ = std::fs::remove_dir_all(&dir);
        c
    }

    #[test]
    fn shard_path_is_two_level() {
        let c = tmp_config();
        let p = html_path(&c.body_cache_dir, "3fa2c891b04e", 1);
        let rel = p
            .strip_prefix(&c.body_cache_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert_eq!(rel, "3f/a2/3fa2c891b04e.v1.html");
    }

    #[test]
    fn short_url_id_falls_back() {
        let c = tmp_config();
        let p = html_path(&c.body_cache_dir, "ab", 1);
        let rel = p
            .strip_prefix(&c.body_cache_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert_eq!(rel, "_/_/ab.v1.html");
    }

    #[test]
    fn save_load_roundtrip() {
        let c = tmp_config();
        let chapters = vec![
            Chapter {
                chapter_id: 1,
                title: "One".into(),
                content: "<p>body</p>".into(),
            },
            Chapter {
                chapter_id: 2,
                title: "Two".into(),
                content: "<p>body2</p>".into(),
            },
        ];
        save_html(&c, "xenforo_50062326", "<html><body>raw</body></html>", 1).unwrap();
        let path = save_body(
            &c,
            "xenforo_50062326",
            &chapters,
            Some("https://example.com".to_string()),
            1,
        )
        .unwrap();
        assert!(path.exists());
        assert_eq!(
            load_html(&c, "xenforo_50062326").as_deref(),
            Some("<html><body>raw</body></html>")
        );
        let loaded = load_body(&c, "xenforo_50062326").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].content, "<p>body</p>");
        assert!(has_body(&c, "xenforo_50062326"));
        delete_body(&c, "xenforo_50062326").unwrap();
        assert!(!has_body(&c, "xenforo_50062326"));
        assert!(load_body(&c, "xenforo_50062326").is_none());
        let _ = INIT.call_once(|| {});
    }

    #[test]
    fn version_bump_gcs_old() {
        let c = tmp_config();
        save_html(&c, "aaaabbbb", "<html>v1</html>", 1).unwrap();
        let v = bump_version(&c, "aaaabbbb");
        assert_eq!(v, 2);
        save_html(&c, "aaaabbbb", "<html>v2</html>", 2).unwrap();
        // old v1 should be gone
        assert!(!html_path(&c.body_cache_dir, "aaaabbbb", 1).exists());
        assert_eq!(
            load_html(&c, "aaaabbbb").as_deref(),
            Some("<html>v2</html>")
        );
    }

    #[test]
    fn missing_returns_none() {
        let c = tmp_config();
        assert!(load_body(&c, "nope").is_none());
        assert!(load_html(&c, "nope").is_none());
    }
}
