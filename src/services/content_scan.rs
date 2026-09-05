//! Content scan: verify cached fic bodies with a local LLM.
//!
//! Two checks in one pass over BODIES_DIR/*.json:
//!   1. **Sanity** — is this body actually story prose, or forum
//!      chatter / scrape noise? Auto-flag noise so curators can fix the
//!      body before it gets exported.
//!   2. **Warnings** — does the text contain graphic violence, sexual
//!      content, or profanity? Keeps the `no_warnings` search filter
//!      honest (a work with no archive warning tags should not actually
//!      contain explicit content).
//!
//! The model is asked to return a single line:
//!   `classification | warnings | confidence | reason`
//!   classification ∈ {story, noise, unclear}
//!   warnings ∈ comma-separated subset of {violence, sexual, profanity} or
//!             `none`
//!   confidence ∈ 0..1
//!
//! Everything is best-effort: an Ollama failure or unparsable reply falls
//! back to `unclear`/`none` and NEVER breaks the caller.

use crate::body_cache::{self, BodyBlob};
use crate::config::Config;
use crate::services::ollama::OllamaClient;
use serde::{Deserialize, Serialize};

/// Canonical classification values.
pub const CLASS_STORY: &str = "story";
pub const CLASS_NOISE: &str = "noise";
pub const CLASS_UNCLEAR: &str = "unclear";

/// Warning tokens the model may report (stable, lowercase).
pub const WARN_VIOLENCE: &str = "violence";
pub const WARN_SEXUAL: &str = "sexual";
pub const WARN_PROFANITY: &str = "profanity";

/// A single scan result row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScanRow {
    pub url_id: String,
    pub classification: String,
    pub confidence: f32,
    pub detected_warnings: String,
    pub reason: String,
}

/// Parse the model's single-line reply into a [`ContentScanRow`].
/// Never panics; unknown values fall back to unclear/none.
pub fn parse_scan_response(s: &str) -> ContentScanRow {
    let line = s.trim().lines().next().unwrap_or("").trim();
    let parts: Vec<&str> = line.splitn(4, '|').map(|p| p.trim()).collect();
    let classification = match parts.first().copied().unwrap_or("") {
        "story" => CLASS_STORY,
        "noise" => CLASS_NOISE,
        _ => CLASS_UNCLEAR,
    };
    let warnings_raw = parts.get(1).copied().unwrap_or("none").to_lowercase();
    let detected: Vec<&str> = if warnings_raw == "none" || warnings_raw.is_empty() {
        vec![]
    } else {
        warnings_raw
            .split(',')
            .map(|w| w.trim())
            .filter(|w| matches!(*w, "violence" | "sexual" | "profanity"))
            .collect()
    };
    let confidence: f32 = parts
        .get(2)
        .and_then(|c| c.parse::<f32>().ok())
        .filter(|c| c.is_finite() && (0.0..=1.0).contains(c))
        .unwrap_or(0.0);
    let reason = parts.get(3).copied().unwrap_or("").to_string();

    ContentScanRow {
        url_id: String::new(), // filled by caller
        classification: classification.to_string(),
        confidence,
        detected_warnings: detected.join(","),
        reason,
    }
}

/// Build the LLM prompt for a single body.
fn build_prompt(blob: &BodyBlob) -> String {
    // Concatenate the first ~6000 chars of chapter text.
    let mut text = String::new();
    'outer: for ch in &blob.chapters {
        // Strip tags crudely (the model only needs the gist).
        let plain = html_to_text(&ch.content);
        text.push_str(&plain);
        if text.chars().count() >= 6000 {
            break 'outer;
        }
    }
    let text: String = text.chars().take(6000).collect();

    format!(
        r#"You are verifying the content of a cached fanfiction page.
Classify the body text below.

Return ONE line, pipe-separated, exactly this format:
classification | warnings | confidence | reason

- classification: "story" if it is narrative fiction (a story chapter),
  "noise" if it is forum comments, navigation text, an error page, or
  other non-story content, "unclear" if you cannot tell.
- warnings: comma-separated subset of violence,sexual,profanity that
  actually appears in the text; "none" if none.
- confidence: 0.0 to 1.0.
- reason: short phrase.

BODY TEXT:
{text}"#,
        text = text
    )
}

/// Crude HTML→text: strip tags + decode common entities (model-only, not
/// for display).
pub fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            out.push(' ');
        } else if !in_tag {
            out.push(c);
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// Scan a single body blob and return the parsed result.
pub async fn scan_blob(ollama: &OllamaClient, chat_model: &str, blob: &BodyBlob) -> ContentScanRow {
    let mut row = ContentScanRow {
        url_id: blob.url_id.clone(),
        classification: CLASS_UNCLEAR.to_string(),
        confidence: 0.0,
        detected_warnings: String::new(),
        reason: String::new(),
    };
    match ollama.generate(&build_prompt(blob), chat_model).await {
        Ok(reply) => {
            let parsed = parse_scan_response(&reply);
            row.classification = parsed.classification;
            row.confidence = parsed.confidence;
            row.detected_warnings = parsed.detected_warnings;
            row.reason = parsed.reason;
        }
        Err(e) => {
            tracing::warn!("content scan skipped for {} (ollama): {}", blob.url_id, e);
        }
    }
    row
}

/// Store a scan row (upsert). Never fails the caller.
pub async fn upsert_scan(db: &sqlx::PgPool, row: &ContentScanRow) {
    let res = sqlx::query(
        r#"INSERT INTO content_scan (url_id, classification, confidence, detected_warnings, reason)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (url_id) DO UPDATE SET
             classification = EXCLUDED.classification,
             confidence = EXCLUDED.confidence,
             detected_warnings = EXCLUDED.detected_warnings,
             reason = EXCLUDED.reason,
             scanned_at = NOW()
           "#,
    )
    .bind(&row.url_id)
    .bind(&row.classification)
    .bind(row.confidence)
    .bind(&row.detected_warnings)
    .bind(&row.reason)
    .execute(db)
    .await;
    if let Err(e) = res {
        tracing::warn!("content scan upsert failed for {}: {}", row.url_id, e);
    }
}

/// Noise confidence threshold — fics below this are flagged for deletion.
const NOISE_CONFIDENCE_THRESHOLD: f32 = 0.8;

/// Grace period before auto-deletion (72 hours).
const DELETION_GRACE_PERIOD_HOURS: i64 = 72;

/// Store a scan row and schedule deletion if it's high-confidence noise.
/// Called after a single fic is scraped and its body is cached.
pub async fn upsert_scan_with_deletion(
    db: &sqlx::PgPool,
    row: &ContentScanRow,
    modlog_actor: Option<i32>,
    modlog_actor_name: Option<&str>,
) {
    let deletion_scheduled =
        if row.classification == CLASS_NOISE && row.confidence >= NOISE_CONFIDENCE_THRESHOLD {
            Some(chrono::Utc::now() + chrono::Duration::hours(DELETION_GRACE_PERIOD_HOURS))
        } else {
            None
        };

    let res = sqlx::query(
        r#"INSERT INTO content_scan (url_id, classification, confidence, detected_warnings, reason, deletion_scheduled_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (url_id) DO UPDATE SET
             classification = EXCLUDED.classification,
             confidence = EXCLUDED.confidence,
             detected_warnings = EXCLUDED.detected_warnings,
             reason = EXCLUDED.reason,
             scanned_at = NOW(),
             deletion_scheduled_at = EXCLUDED.deletion_scheduled_at
           "#,
    )
    .bind(&row.url_id)
    .bind(&row.classification)
    .bind(row.confidence)
    .bind(&row.detected_warnings)
    .bind(&row.reason)
    .bind(deletion_scheduled)
    .execute(db)
    .await;

    if let Err(e) = res {
        tracing::warn!("content scan upsert failed for {}: {}", row.url_id, e);
        return;
    }

    // Log to modlog if flagged for deletion
    if deletion_scheduled.is_some() {
        let _ = crate::modlog::record(
            db,
            modlog_actor,
            Some(modlog_actor_name.unwrap_or("auto-moderator").to_string()),
            "content_scan_flag",
            "work",
            &row.url_id,
            serde_json::json!({
                "classification": row.classification,
                "confidence": row.confidence,
                "reason": &row.reason,
                "deletion_scheduled": deletion_scheduled.unwrap().to_rfc3339(),
                "grace_hours": DELETION_GRACE_PERIOD_HOURS,
            }),
        )
        .await;
        tracing::info!(
            "Auto-moderator flagged {} as noise (conf={:.2}) — deletion scheduled in {}h",
            row.url_id,
            row.confidence,
            DELETION_GRACE_PERIOD_HOURS
        );
    }
}

/// Classify a single fic from its cached body. Best-effort — never fails the caller.
/// Called from the export route after body is saved.
pub async fn scan_single_fic(
    db: &sqlx::PgPool,
    ollama: &OllamaClient,
    config: &Config,
    url_id: &str,
) {
    let Some(chapters) = crate::body_cache::load_body(config, url_id) else {
        tracing::debug!("content scan skipped for {} — no cached body", url_id);
        return;
    };

    let blob = crate::body_cache::BodyBlob {
        url_id: url_id.to_string(),
        chapters,
        saved_at_ms: 0,
        source: None,
    };

    let row = scan_blob(ollama, &config.ollama_chat_model, &blob).await;
    upsert_scan_with_deletion(db, &row, None, None).await;

    tracing::info!(
        "content scan (auto) {}: {:?} conf={:.2}",
        url_id,
        row.classification,
        row.confidence
    );
}

/// Pending noise entries whose grace period has expired → auto-delete.
/// Returns the number of fics deleted.
pub async fn process_expired_deletions(db: &sqlx::PgPool, config: &Config) -> usize {
    let expired: Vec<(String,)> = match sqlx::query_as(
        r#"SELECT url_id FROM content_scan
           WHERE review_status = 'pending'
             AND classification = 'noise'
             AND deletion_scheduled_at IS NOT NULL
             AND deletion_scheduled_at <= NOW()
           ORDER BY deletion_scheduled_at
           LIMIT 50"#,
    )
    .fetch_all(db)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("auto-delete query failed: {e}");
            return 0;
        }
    };

    let mut deleted = 0;
    for (url_id,) in expired {
        // Delete cached body
        let _ = crate::body_cache::delete_body(config, &url_id);

        // Mark as deleted in content_scan
        let _ = sqlx::query(
            "UPDATE content_scan SET review_status = 'confirmed', reviewed_at = NOW() WHERE url_id = $1",
        )
        .bind(&url_id)
        .execute(db)
        .await;

        // Log to modlog
        let _ = crate::modlog::record(
            db,
            None,
            Some("auto-moderator".to_string()),
            "auto_delete_noise",
            "work",
            &url_id,
            serde_json::json!({
                "reason": "LLM classified as noise with high confidence, grace period expired",
            }),
        )
        .await;

        tracing::info!("Auto-deleted noise fic: {}", url_id);
        deleted += 1;
    }

    if deleted > 0 {
        tracing::info!("Auto-delete cron: removed {} expired noise fics", deleted);
    }
    deleted
}

/// Walk the body cache directory and scan every blob found.
///
/// Returns (scanned, failed) counts. Best-effort: a bad file or an Ollama
/// error is logged and skipped, never fatal.
pub async fn scan_cache(
    db: &sqlx::PgPool,
    ollama: &OllamaClient,
    config: &Config,
    limit: Option<usize>,
) -> (usize, usize) {
    let mut scanned = 0usize;
    let mut failed = 0usize;
    for path in walk_json_files(&config.body_cache_dir) {
        if let Some(l) = limit {
            if scanned >= l {
                break;
            }
        }
        // url_id from the .v{N}.json filename
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let Some((url_id, _ver)) = parse_blob_filename(&file_name) else {
            continue;
        };
        match body_cache::load_body(config, &url_id) {
            Some(chapters) => {
                let blob = BodyBlob {
                    url_id,
                    chapters,
                    saved_at_ms: 0,
                    source: None,
                };
                let row = scan_blob(ollama, &config.ollama_chat_model, &blob).await;
                upsert_scan(db, &row).await;
                scanned += 1;
                tracing::info!(
                    "content scan {}: {:?} warnings={:?} conf={:.2}",
                    blob.url_id,
                    row.classification,
                    row.detected_warnings,
                    row.confidence
                );
            }
            None => {
                tracing::warn!("content scan failed to load {url_id}");
                failed += 1;
            }
        }
    }
    (scanned, failed)
}

/// Recursively list `.v{N}.json` body files under a directory.
fn walk_json_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().map(|e| e == "json").unwrap_or(false) {
                out.push(p);
            }
        }
    }
    out
}

/// Parse a blob filename like `abc123.v1.json` → ("abc123", 1).
fn parse_blob_filename(name: &str) -> Option<(String, i32)> {
    let stem = name.strip_suffix(".json")?;
    let (id, ver) = stem.rsplit_once(".v")?;
    let ver: i32 = ver.parse().ok()?;
    Some((id.to_string(), ver))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_story_no_warnings() {
        let r = parse_scan_response("story | none | 0.95 | standard chapter");
        assert_eq!(r.classification, CLASS_STORY);
        assert_eq!(r.detected_warnings, "");
        assert!((r.confidence - 0.95).abs() < 1e-6);
    }

    #[test]
    fn parses_noise_with_warnings() {
        let r = parse_scan_response("noise | violence,profanity | 0.8 | forum chatter");
        assert_eq!(r.classification, CLASS_NOISE);
        assert_eq!(r.detected_warnings, "violence,profanity");
    }

    #[test]
    fn parses_unknown_falls_back() {
        let r = parse_scan_response("banana | weird | 9.9 | ");
        assert_eq!(r.classification, CLASS_UNCLEAR);
        assert_eq!(r.detected_warnings, "");
        assert_eq!(r.confidence, 0.0);
    }

    #[test]
    fn parses_trailing_prose_after_first_line() {
        let r =
            parse_scan_response("story | none | 0.9 | fine\nhere is more text the model appended");
        assert_eq!(r.classification, CLASS_STORY);
        assert_eq!(r.reason, "fine");
    }

    #[test]
    fn html_to_text_strips_tags() {
        assert_eq!(
            html_to_text("<p>Hello &amp; goodbye</p>"),
            " Hello & goodbye "
        );
    }

    #[test]
    fn prompt_contains_text() {
        let blob = BodyBlob {
            url_id: "ao3_1".into(),
            chapters: vec![crate::scrape::Chapter {
                chapter_id: 1,
                title: "C1".into(),
                content: "<p>It was a dark night.</p>".into(),
            }],
            saved_at_ms: 0,
            source: None,
        };
        let p = build_prompt(&blob);
        assert!(p.contains("dark night"));
    }
}
