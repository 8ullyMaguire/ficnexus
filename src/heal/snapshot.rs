//! HTML snapshot capture for parse failures (best-effort).
//!
//! When a scraper reports a parse error, the export path re-fetches the story
//! URL, strips `<script>`/`<style>` blocks, truncates to 200KB, and stores the
//! result under `tests/fixtures/scrape/<domain>/<fingerprint>.html` so a
//! future fixer agent can inspect what the site actually returned. Any
//! failure here is swallowed — the failure row still records with a NULL
//! snapshot path.

use std::path::PathBuf;

use crate::heal::classifier::{self, ErrorKind};

const MAX_SNAPSHOT_BYTES: usize = 200 * 1024;

/// Sanitize a domain into a filesystem-safe directory name.
fn safe_domain(domain: &str) -> String {
    domain
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '_' })
        .collect()
}

/// Strip `<script>` and `<style>` element contents (naive, case-insensitive).
fn strip_script_style(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        let lower = rest.to_lowercase();
        let (open_script, open_style) = (
            lower.find("<script"),
            lower.find("<style"),
        );
        let open = match (open_script, open_style) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let Some(open_idx) = open else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..open_idx]);
        let after_open = &rest[open_idx..];
        // Find the closing tag: `</script>` / `</style>`.
        let tag = if after_open.to_lowercase().starts_with("<script") { "</script>" } else { "</style>" };
        let close = after_open.to_lowercase().find(tag);
        match close {
            Some(close_idx) => {
                // Skip the whole element.
                let skip_len = close_idx + tag.len();
                rest = &after_open[skip_len.min(after_open.len())..];
            }
            None => {
                // Unclosed block — drop the remainder.
                break;
            }
        }
    }
    out
}

/// Best-effort snapshot: fetch `url`, sanitize, truncate, write to
/// `tests/fixtures/scrape/<domain>/<fingerprint>.html`. Returns the relative
/// path (or None on any failure).
pub async fn capture_snapshot(http: &reqwest::Client, url: &str) -> Option<String> {
    let domain = classifier::url_domain(url)?;
    let resp = http.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    if body.is_empty() {
        return None;
    }
    let cleaned = strip_script_style(&body);
    let truncated: String = cleaned.chars().take(MAX_SNAPSHOT_BYTES).collect();

    let fp = classifier::fingerprint(url, &ErrorKind::Parse, "snapshot");
    let dir = PathBuf::from("tests/fixtures/scrape").join(safe_domain(&domain));
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let path = dir.join(format!("{fp}.html"));
    if std::fs::write(&path, truncated).is_err() {
        return None;
    }
    Some(path.to_string_lossy().to_string())
}

/// Save a pre-fetched HTML snapshot (e.g. from the Wayback Machine) for
/// `url` under the standard snapshot location. Returns the relative path
/// or None on any failure (best-effort, same contract as `capture_snapshot`).
pub async fn save_snapshot_html(url: &str, html: &str) -> Option<String> {
    let domain = classifier::url_domain(url)?;
    if html.is_empty() {
        return None;
    }
    let cleaned = strip_script_style(html);
    let truncated: String = cleaned.chars().take(MAX_SNAPSHOT_BYTES).collect();

    let fp = classifier::fingerprint(url, &ErrorKind::Parse, "snapshot");
    let dir = PathBuf::from("tests/fixtures/scrape").join(safe_domain(&domain));
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let path = dir.join(format!("{fp}.html"));
    if std::fs::write(&path, truncated).is_err() {
        return None;
    }
    Some(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_and_style_blocks() {
        let html = "<html><head><style>body{color:red}</style></head>\
                    <body><script>alert(1)</script><p>Hello</p></body></html>";
        let out = strip_script_style(html);
        assert!(!out.contains("<script"));
        assert!(!out.contains("alert"));
        assert!(!out.contains("<style"));
        assert!(!out.contains("color:red"));
        assert!(out.contains("<p>Hello</p>"));
    }

    #[test]
    fn unclosed_script_drops_remainder() {
        let html = "<p>keep</p><script>alert('never closes'";
        let out = strip_script_style(html);
        assert!(out.contains("<p>keep</p>"));
        assert!(!out.contains("never closes"));
    }

    #[test]
    fn safe_domain_sanitizes() {
        assert_eq!(safe_domain("www.fanfiction.net"), "www.fanfiction.net");
        assert_eq!(safe_domain("bad domain!"), "bad_domain_");
    }
}
