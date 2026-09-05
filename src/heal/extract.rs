//! On-the-fly metadata extraction service (self-healing milestone 2).
//!
//! When a structural scrape failure happens AND `AGENT_USE_ON_FLY=true`,
//! the export path calls [`run_extraction`]: the agent (remote
//! OpenAI-compatible endpoint, else local Ollama) extracts structured
//! `FicMetadata` from the HTML snapshot. The extraction is validated
//! ([`parse_agent_metadata_json`]), persisted in `heal_extractions`
//! (`validated=true`) with an `agent_runs` ledger row (status `proposed`),
//! and the export request is completed using it. Admin review
//! (`GET/POST /api/admin/heal/extractions`) decides whether an extraction
//! becomes `trusted`, after which [`find_trusted_extraction_for_url`] can
//! replay it.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{FromRow, PgPool};

use crate::config::Config;
use crate::heal::agent;
use crate::heal::classifier;
use crate::heal::store::ScrapeFailureRow;
use crate::scrape::FicMetadata;

/// The service the export path and the admin endpoints use. Constructed on
/// demand from `AppState` pieces (no new AppState field needed).
#[derive(Clone)]
pub struct ExtractService {
    pub db: PgPool,
    pub config: Arc<Config>,
}

/// Validated metadata extracted by the agent from a snapshot.
#[derive(Debug, Clone)]
pub struct AgentMetadata {
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub status: String,
    pub source: String,
    pub url_id: Option<String>,
}

/// A `heal_extractions` row (mirrors migration 034).
#[derive(Debug, Clone, FromRow)]
pub struct ExtractionRow {
    pub id: i64,
    pub failure_id: Option<i64>,
    pub agent_run_id: Option<i64>,
    pub url: String,
    pub url_id: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub chapters: Option<i32>,
    pub words: Option<i64>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub validated: bool,
    pub trusted: bool,
    pub created_at: DateTime<Utc>,
}

/// Raw JSON shape the agent must produce (STrict JSON only).
#[derive(Debug, Deserialize)]
struct AgentJson {
    title: String,
    author: String,
    #[serde(default)]
    chapters: Option<i32>,
    #[serde(default)]
    words: Option<i64>,
    #[serde(default)]
    desc: String,
    status: String,
    source: String,
    #[serde(default)]
    url_id: Option<String>,
}

/// Parse + validate the agent's raw reply into [`AgentMetadata`].
/// Tolerates a fenced ```json ... ``` block around the payload.
pub fn parse_agent_metadata_json(text: &str, source_url: &str) -> Result<AgentMetadata, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("agent returned empty extraction".to_string());
    }
    // Strip a ```json ... ``` fence if present.
    let body = if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        let end = rest
            .rfind("```")
            .ok_or_else(|| "unterminated code fence in agent reply".to_string())?;
        rest[..end].trim()
    } else {
        trimmed
    };
    // If a fence still appears in the middle (e.g. leading prose before the
    // block), take the LAST fenced block: the content between the
    // second-to-last ``` (the opener) and the final ``` (the closer).
    let body = if body.starts_with('{') {
        body
    } else {
        let last = body.rfind("```");
        match last {
            Some(last_idx) => {
                let before = &body[..last_idx];
                match before.rfind("```") {
                    Some(open_idx) => {
                        let after_open = &before[open_idx + 3..];
                        let after_open = after_open.strip_prefix("json").unwrap_or(after_open);
                        after_open.trim()
                    }
                    None => body,
                }
            }
            None => body,
        }
    };
    let body = body
        .trim()
        .trim_start_matches("json")
        .trim()
        .trim_matches('`')
        .trim();

    let parsed: AgentJson =
        serde_json::from_str(body).map_err(|e| format!("agent reply is not valid JSON: {e}"))?;

    // ── Validation ──────────────────────────────────────────────────
    let title = parsed.title.trim().to_string();
    if title.is_empty() {
        return Err("extraction rejected: empty title".to_string());
    }
    if title.chars().count() > 500 {
        return Err("extraction rejected: title exceeds 500 chars".to_string());
    }
    let author = parsed.author.trim().to_string();
    let chapters = parsed.chapters.unwrap_or(0);
    if chapters < 1 {
        return Err(format!(
            "extraction rejected: chapters must be >= 1 (got {chapters})"
        ));
    }
    let words = parsed.words.unwrap_or(0).max(0);
    let status = parsed.status.trim().to_lowercase();
    if !["ongoing", "complete", "hiatus", "cancelled"].contains(&status.as_str()) {
        return Err(format!(
            "extraction rejected: status must be ongoing|complete|hiatus|cancelled (got {status})"
        ));
    }
    let source = parsed.source.trim().to_string();
    let source_domain = classifier::url_domain(&source)
        .ok_or_else(|| "extraction rejected: source is not a valid URL".to_string())?;
    let expected_domain = classifier::url_domain(source_url)
        .ok_or_else(|| "extraction rejected: source_url is not a valid URL".to_string())?;
    if source_domain != expected_domain {
        return Err(format!(
            "extraction rejected: source host {source_domain} does not match {expected_domain}"
        ));
    }

    Ok(AgentMetadata {
        title,
        author,
        chapters,
        words,
        desc: parsed.desc,
        status,
        source,
        url_id: parsed.url_id.filter(|s| !s.trim().is_empty()),
    })
}

/// Run the extraction agent for a scrape failure: record an `agent_runs`
/// row (`heal_extract` / class `structural`), call remote-or-local, and on
/// success persist a validated `heal_extractions` row + mark the run
/// `proposed`; on failure mark the run `failed` and return the reason.
pub async fn run_extraction(
    db: &PgPool,
    cfg: &Config,
    http: &reqwest::Client,
    failure: &ScrapeFailureRow,
    snapshot_html: Option<&str>,
) -> Result<AgentMetadata, String> {
    let snapshot = snapshot_html
        .map(|s| s.to_string())
        .or_else(|| load_snapshot(failure.html_snapshot_path.as_deref()))
        .ok_or_else(|| "no snapshot available for extraction".to_string())?;

    let run = crate::heal::store::record_agent_run(
        db,
        "heal_extract",
        Some(failure.id),
        "structural",
        Some(cfg.agent_model.as_str()),
    )
    .await
    .map_err(|e| format!("failed to record agent run: {e}"))?;

    let reply = if agent::remote_configured(cfg) {
        agent::extract_metadata_remote(http, cfg, &snapshot, &failure.url).await
    } else {
        agent::extract_metadata_local(http, cfg, &snapshot, &failure.url).await
    };

    match reply {
        Ok(text) => {
            let meta = match parse_agent_metadata_json(&text, &failure.url) {
                Ok(m) => m,
                Err(reason) => {
                    let _ = crate::heal::store::update_agent_run(
                        db,
                        run.id,
                        "failed",
                        Some(&format!("extraction failed validation: {reason}")),
                        Some(1),
                        Some(0),
                    )
                    .await;
                    return Err(reason);
                }
            };
            let _ = store_extraction(db, run.id, failure.id, &meta, true).await;
            let _ = crate::heal::store::update_agent_run(
                db,
                run.id,
                "proposed",
                Some(&format!(
                    "extracted {} chapters={} words={} status={}",
                    meta.title, meta.chapters, meta.words, meta.status
                )),
                Some(1),
                Some(0),
            )
            .await;
            Ok(meta)
        }
        Err(reason) => {
            let _ = crate::heal::store::update_agent_run(
                db,
                run.id,
                "failed",
                Some("agent unreachable"),
                Some(1),
                Some(0),
            )
            .await;
            Err(reason)
        }
    }
}

/// Read a snapshot file written by [`crate::heal::snapshot::capture_snapshot`].
fn load_snapshot(path: Option<&str>) -> Option<String> {
    let path = path?;
    std::fs::read_to_string(path).ok()
}

/// INSERT a heal_extractions row; returns the new row id. `agent_run_id`
/// 0 means "no agent run" (tests / direct callers) and is stored as NULL.
pub async fn store_extraction(
    db: &PgPool,
    agent_run_id: i64,
    failure_id: i64,
    meta: &AgentMetadata,
    validated: bool,
) -> Result<i64, sqlx::Error> {
    let agent_run_id: Option<i64> = if agent_run_id > 0 {
        Some(agent_run_id)
    } else {
        None
    };
    sqlx::query_scalar(
        r#"INSERT INTO heal_extractions
           (failure_id, agent_run_id, url, url_id, title, author, chapters,
            words, description, status, validated)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING id"#,
    )
    .bind(failure_id)
    .bind(agent_run_id)
    .bind(&meta.source)
    .bind(&meta.url_id)
    .bind(&meta.title)
    .bind(&meta.author)
    .bind(meta.chapters)
    .bind(meta.words)
    .bind(&meta.desc)
    .bind(&meta.status)
    .bind(validated)
    .fetch_one(db)
    .await
}

/// List heal_extractions rows, optionally trusted-only.
pub async fn list_extractions(
    db: &PgPool,
    trusted_only: bool,
    limit: i64,
) -> Result<Vec<ExtractionRow>, sqlx::Error> {
    sqlx::query_as::<_, ExtractionRow>(
        r#"SELECT id, failure_id, agent_run_id, url, url_id, title, author,
                  chapters, words, description, status, validated, trusted,
                  created_at
           FROM heal_extractions
           WHERE trusted = $1
           ORDER BY created_at DESC
           LIMIT $2"#,
    )
    .bind(trusted_only)
    .bind(limit)
    .fetch_all(db)
    .await
}

/// Mark an extraction trusted. Returns false when the row doesn't exist.
pub async fn mark_trusted(db: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("UPDATE heal_extractions SET trusted = true WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Find the most recent TRUSTED extraction for a URL (replay support).
pub async fn find_trusted_extraction_for_url(
    db: &PgPool,
    url: &str,
) -> Result<Option<ExtractionRow>, sqlx::Error> {
    sqlx::query_as::<_, ExtractionRow>(
        r#"SELECT id, failure_id, agent_run_id, url, url_id, title, author,
                  chapters, words, description, status, validated, trusted,
                  created_at
           FROM heal_extractions
           WHERE trusted = true AND url = $1
           ORDER BY created_at DESC
           LIMIT 1"#,
    )
    .bind(url)
    .fetch_optional(db)
    .await
}

/// Enqueue a user export request into the pending queue after a transient or
/// blocked failure so a later successful heal can auto-complete it.
pub async fn enqueue_pending_export(
    db: &PgPool,
    url: &str,
    format: &str,
    client_ip: Option<String>,
    client_id: Option<&str>,
    error_kind: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"INSERT INTO pending_exports (url, format, client_ip, client_id, error_kind)
           VALUES ($1, $2, $3::inet, $4, $5)
           RETURNING id"#,
    )
    .bind(url)
    .bind(format)
    .bind(client_ip)
    .bind(client_id)
    .bind(error_kind)
    .fetch_one(db)
    .await
}

impl AgentMetadata {
    /// Build a FicMetadata from a validated extraction. url_id derives from
    /// the source URL when the agent didn't provide one; published/updated
    /// are set to now (the snapshot is all we have).
    pub fn to_fic_metadata(&self, source_url: &str) -> FicMetadata {
        let now_millis = chrono::Utc::now().timestamp_millis();
        let url_id = self
            .url_id
            .clone()
            .unwrap_or_else(|| derive_url_id(source_url));
        FicMetadata {
            url_id,
            title: self.title.clone(),
            author: self.author.clone(),
            chapters: self.chapters,
            words: self.words,
            desc: self.desc.clone(),
            published: now_millis,
            updated: now_millis,
            status: self.status.clone(),
            source: self.source.clone(),
            source_id: 0,
            author_id: 0,
            author_url: String::new(),
            author_local_id: String::new(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        }
    }
}

/// Derive a stable url_id from the last non-empty path segment of a URL
/// (e.g. `https://site.example/works/21845264` → `21845264`).
fn derive_url_id(url: &str) -> String {
    let path = url.split("://").nth(1).unwrap_or(url);
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let seg = path
        .split('/')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("")
        .to_string();
    if seg.is_empty() {
        classifier::fingerprint(url, &classifier::ErrorKind::Unknown, "derive_url_id")
    } else {
        seg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_plain_json() {
        let meta = parse_agent_metadata_json(
            r#"{"title":"A Test","author":"Bob","chapters":4,"words":12345,
                "desc":"A story","status":"ongoing","source":"https://healtest.example.com/s/1001",
                "url_id":"ht_1001"}"#,
            "https://healtest.example.com/s/1001",
        )
        .expect("valid extraction");
        assert_eq!(meta.title, "A Test");
        assert_eq!(meta.chapters, 4);
        assert_eq!(meta.words, 12345);
        assert_eq!(meta.status, "ongoing");
        assert_eq!(meta.url_id.as_deref(), Some("ht_1001"));
    }

    #[test]
    fn parse_tolerates_fenced_json() {
        let meta = parse_agent_metadata_json(
            "Here is the result:\n```json\n{\"title\":\"T\",\"author\":\"A\",\
             \"chapters\":1,\"words\":0,\"desc\":\"\",\"status\":\"complete\",\
             \"source\":\"https://healtest.example.com/s/1002\",\"url_id\":\"ht_1002\"}\n```",
            "https://healtest.example.com/s/1002",
        )
        .expect("fenced extraction");
        assert_eq!(meta.status, "complete");
    }

    #[test]
    fn parse_rejects_garbage() {
        let err =
            parse_agent_metadata_json("not json at all", "https://x.example/s/1").unwrap_err();
        assert!(err.contains("not valid JSON"), "err: {err}");
    }

    #[test]
    fn parse_rejects_bad_status() {
        let err = parse_agent_metadata_json(
            r#"{"title":"T","author":"A","chapters":1,"words":0,"desc":"",
                "status":"on-hiatus","source":"https://healtest.example.com/s/1"}"#,
            "https://healtest.example.com/s/1",
        )
        .unwrap_err();
        assert!(err.contains("status"), "err: {err}");
    }

    #[test]
    fn parse_rejects_zero_chapters() {
        let err = parse_agent_metadata_json(
            r#"{"title":"T","author":"A","chapters":0,"words":0,"desc":"",
                "status":"ongoing","source":"https://healtest.example.com/s/1"}"#,
            "https://healtest.example.com/s/1",
        )
        .unwrap_err();
        assert!(err.contains("chapters"), "err: {err}");
    }

    #[test]
    fn parse_rejects_cross_host_source() {
        let err = parse_agent_metadata_json(
            r#"{"title":"T","author":"A","chapters":1,"words":0,"desc":"",
                "status":"ongoing","source":"https://evil.example/s/1"}"#,
            "https://healtest.example.com/s/1",
        )
        .unwrap_err();
        assert!(err.contains("host"), "err: {err}");
    }

    #[test]
    fn url_id_derives_from_url_when_missing() {
        let meta = AgentMetadata {
            title: "T".into(),
            author: "A".into(),
            chapters: 1,
            words: 0,
            desc: String::new(),
            status: "ongoing".into(),
            source: "https://healtest.example.com/s/1001".into(),
            url_id: None,
        };
        let fic = meta.to_fic_metadata("https://healtest.example.com/s/1001");
        assert_eq!(fic.url_id, "1001");
        assert_eq!(fic.chapters, 1);
        assert_eq!(fic.status, "ongoing");
        assert_eq!(fic.source, "https://healtest.example.com/s/1001");
        assert!(fic.published > 0);
    }
}
