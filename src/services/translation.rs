//! Translation service — M1 of docs/plans/translation-everything.md
//!
//! 3-tier fallback chain for any user-visible text:
//! approved community translation > cached machine translation > original.
//! Machine output is produced once by the local Ollama model through a Redis
//! queue (`translate_queue`) and cached in `translation_strings` forever
//! (schema: migrations/074_translation.sql).
//!
//! `target_id` is always TEXT per the 074 schema: numeric ids for
//! request/comment/review/forum rows, the url_id string for fic metadata.
#![allow(dead_code)] // public API; some variants wired from routes/worker only

use crate::config::Config;
use crate::error::{AppError, Result};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use tracing::{info, warn};

/// What kind of content is being translated. `key` is how the row is stored
/// in `translation_strings.target_type`; `numeric` says whether `target_id`
/// must parse as an integer (false ⇒ free-form text key like a url_id).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationType {
    WorkMeta,
    Request,
    RequestAnswer,
    ForumTopic,
    ForumPost,
    Comment,
    Review,
}

impl TranslationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WorkMeta => "work_meta",
            Self::Request => "request",
            Self::RequestAnswer => "request_answer",
            Self::ForumTopic => "forum_topic",
            Self::ForumPost => "forum_post",
            Self::Comment => "comment",
            Self::Review => "review",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "work_meta" => Self::WorkMeta,
            "request" => Self::Request,
            "request_answer" => Self::RequestAnswer,
            "forum_topic" => Self::ForumTopic,
            "forum_post" => Self::ForumPost,
            "comment" => Self::Comment,
            "review" => Self::Review,
            _ => return None,
        })
    }

    /// target_id must be an integer row id (false ⇒ free-form text key).
    pub fn numeric_key(self) -> bool {
        !matches!(self, Self::WorkMeta)
    }

    /// Fields this content type exposes for translation.
    pub fn allowed_fields(self) -> &'static [&'static str] {
        match self {
            Self::WorkMeta => &["title", "summary"],
            Self::Request => &["title", "body"],
            Self::RequestAnswer => &["pitch"],
            Self::ForumTopic => &["title", "body"],
            Self::ForumPost => &["body"],
            Self::Comment => &["body"],
            Self::Review => &["title", "body"],
        }
    }

    pub fn allows_field(self, field: &str) -> bool {
        self.allowed_fields().contains(&field)
    }
}

#[derive(Debug, Clone)]
pub struct TranslationSource {
    pub text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedTranslation {
    pub text: String,
    pub status: TranslationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationStatus {
    Approved,
    Machine,
    /// No translation exists — `text` is the original.
    None,
}

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

/// Public registry: given (type, text key, field), fetch the source text.
/// Returns None when the row or field doesn't exist (or, for chapter bodies,
/// which live in the on-disk body cache and are out of M1 scope).
pub async fn fetch_source(
    db: &PgPool,
    typ: TranslationType,
    key: &str,
    field: &str,
) -> Result<Option<TranslationSource>> {
    if !typ.allows_field(field) {
        return Ok(None);
    }
    // Numeric types: reject non-integer keys instead of hitting the DB.
    let num: Option<i64> = if typ.numeric_key() {
        match key.parse::<i64>() {
            Ok(n) => Some(n),
            Err(_) => return Ok(None),
        }
    } else {
        None
    };

    let text: Option<String> = match (typ, field) {
        (TranslationType::WorkMeta, "title") => {
            sqlx::query_scalar("SELECT title FROM fic_info WHERE id = $1")
                .bind(key)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::WorkMeta, "summary") => {
            sqlx::query_scalar("SELECT description FROM fic_info WHERE id = $1")
                .bind(key)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::Request, "title") => {
            sqlx::query_scalar("SELECT title FROM fic_requests WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::Request, "body") => {
            sqlx::query_scalar("SELECT body FROM fic_requests WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::RequestAnswer, "pitch") => {
            sqlx::query_scalar("SELECT pitch FROM fic_request_answers WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::ForumTopic, "title") => {
            sqlx::query_scalar("SELECT title FROM forum_topics WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::ForumTopic, "body") => {
            sqlx::query_scalar("SELECT body FROM forum_topics WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::ForumPost, "body") => {
            sqlx::query_scalar("SELECT body FROM forum_posts WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::Comment, "body") => {
            sqlx::query_scalar("SELECT body FROM comments WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::Review, "title") => {
            sqlx::query_scalar("SELECT COALESCE(title, '') FROM reviews WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        (TranslationType::Review, "body") => {
            sqlx::query_scalar("SELECT body FROM reviews WHERE id = $1")
                .bind(num)
                .fetch_optional(db)
                .await?
        }
        _ => return Ok(None),
    };
    Ok(text.map(|t| TranslationSource { text: t }))
}

/// Resolve chain: approved > machine (if source_hash still matches the
/// current source) > original text with status "none".
pub async fn resolve_chain(
    db: &PgPool,
    typ: TranslationType,
    key: &str,
    field: &str,
    locale: &str,
) -> Result<ResolvedTranslation> {
    let row = sqlx::query(
        r#"SELECT text, source_hash, status FROM translation_strings
           WHERE target_type = $1 AND target_id = $2 AND field = $3 AND locale = $4
           ORDER BY
             CASE status WHEN 'approved' THEN 0 WHEN 'machine' THEN 1 ELSE 2 END,
             updated_at DESC
           LIMIT 1"#,
    )
    .bind(typ.as_str())
    .bind(key)
    .bind(field)
    .bind(locale)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal(format!("translation_strings lookup: {e}")))?;

    if let Some(r) = row {
        let text: String = r.get("text");
        let source_hash: String = r.get("source_hash");
        let status: String = r.get("status");
        if status == "approved" {
            return Ok(ResolvedTranslation {
                text,
                status: TranslationStatus::Approved,
            });
        }
        if status == "machine" {
            // Only serve machine text while the source hasn't changed.
            if let Some(src) = fetch_source(db, typ, key, field).await? {
                if sha256_hex(&src.text) == source_hash {
                    return Ok(ResolvedTranslation {
                        text,
                        status: TranslationStatus::Machine,
                    });
                }
            }
        }
    }

    // Fallback: original text.
    match fetch_source(db, typ, key, field).await? {
        Some(src) => Ok(ResolvedTranslation {
            text: src.text,
            status: TranslationStatus::None,
        }),
        None => Ok(ResolvedTranslation {
            text: String::new(),
            status: TranslationStatus::None,
        }),
    }
}

/// One item in a batch resolve/enqueue request. `id` is the TEXT key
/// (row id as string, or url_id for fic metadata).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TranslateBatchItem {
    #[serde(rename = "type")]
    pub typ: String,
    pub id: String,
    pub field: String,
}

/// Batch resolve for the frontend: max 50 items, skips unknown types/fields.
pub async fn resolve_batch(
    db: &PgPool,
    items: &[TranslateBatchItem],
    locale: &str,
) -> Result<HashMap<String, ResolvedTranslation>> {
    let mut out = HashMap::new();
    for item in items.iter().take(50) {
        let Some(typ) = TranslationType::from_str(&item.typ) else {
            continue;
        };
        if !typ.allows_field(&item.field) {
            continue;
        }
        let key = format!("{}:{}:{}", item.typ, item.id, item.field);
        // Empty text means the row didn't exist — drop from the map.
        let resolved = resolve_chain(db, typ, &item.id, &item.field, locale).await?;
        if !resolved.text.is_empty() {
            out.insert(key, resolved);
        }
    }
    Ok(out)
}

/// Enqueue machine-translation jobs with SETNX dedupe (7d) and hourly
/// budgets (global + per user). Returns the number of NEW jobs queued.
pub async fn enqueue(
    conn: &mut redis::aio::MultiplexedConnection,
    items: &[TranslateBatchItem],
    locale: &str,
    config: &Config,
    requester_id: Option<i32>,
) -> Result<u64> {
    let mut queued = 0u64;

    // Validate everything BEFORE spending budget, so a bad request can't
    // burn the hourly allowance.
    let valid: Vec<&TranslateBatchItem> = items
        .iter()
        .take(50)
        .filter(|i| {
            TranslationType::from_str(&i.typ)
                .filter(|t| t.allows_field(&i.field))
                .filter(|t| !t.numeric_key() || i.id.parse::<i64>().is_ok())
                .is_some()
        })
        .collect();
    if valid.is_empty() {
        return Ok(0);
    }

    // Global hourly budget.
    let hour_key = format!("translate:budget:{}", chrono::Utc::now().format("%Y%m%d%H"));
    let current: u64 = redis::cmd("INCR")
        .arg(&hour_key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if current == 1 {
        let _: i64 = redis::cmd("EXPIRE")
            .arg(&hour_key)
            .arg(3600i64)
            .query_async(&mut *conn)
            .await
            .unwrap_or(0);
    }
    if current > config.translate_global_budget_per_hour as u64 {
        return Err(AppError::RateLimited(3600));
    }

    // Per-user hourly budget (logged-in only).
    if let Some(uid) = requester_id {
        let ukey = format!(
            "translate:budget:u:{}:{}",
            uid,
            chrono::Utc::now().format("%Y%m%d%H")
        );
        let ucurrent: u64 = redis::cmd("INCR")
            .arg(&ukey)
            .query_async(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if ucurrent == 1 {
            let _: i64 = redis::cmd("EXPIRE")
                .arg(&ukey)
                .arg(3600i64)
                .query_async(&mut *conn)
                .await
                .unwrap_or(0);
        }
        if ucurrent > config.translate_user_budget_per_hour as u64 {
            return Err(AppError::RateLimited(3600));
        }
    }

    for item in valid {
        let dedupe_key = format!(
            "translate:job:{}:{}:{}:{}",
            item.typ, item.id, item.field, locale
        );
        let was_new: i64 = redis::cmd("SET")
            .arg(&dedupe_key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(604800i64) // 7 days
            .query_async(&mut *conn)
            .await
            .unwrap_or(0);
        if was_new != 1 {
            continue; // already queued for this locale within 7d
        }
        let job = serde_json::json!({
            "type": item.typ,
            "id": item.id,
            "field": item.field,
            "locale": locale,
            "enqueued_at": chrono::Utc::now().to_rfc3339(),
            "requester_id": requester_id,
        });
        let _: i64 = redis::cmd("RPUSH")
            .arg("translate_queue")
            .arg(job.to_string())
            .query_async(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queued += 1;
    }
    Ok(queued)
}

/// Chunk text on paragraph boundaries, greedily, <= max_chars per chunk
/// (a lone oversized paragraph is split on newlines, then hard-split).
pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(200);
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut cur = String::new();
    for para in text.split("\n\n") {
        let piece = if cur.is_empty() {
            para.to_string()
        } else {
            format!("\n\n{para}")
        };
        if cur.chars().count() + piece.chars().count() <= max_chars {
            cur.push_str(&piece);
        } else {
            if !cur.is_empty() {
                chunks.push(std::mem::take(&mut cur));
            }
            // para itself too long: split on single newlines
            if para.chars().count() > max_chars {
                let mut buf = String::new();
                for line in para.split('\n') {
                    let l = if buf.is_empty() {
                        line.to_string()
                    } else {
                        format!("\n{line}")
                    };
                    if buf.chars().count() + l.chars().count() <= max_chars {
                        buf.push_str(&l);
                    } else {
                        if !buf.is_empty() {
                            chunks.push(std::mem::take(&mut buf));
                        }
                        // hard split as last resort
                        let mut s = line;
                        while s.chars().count() > max_chars {
                            let mut idx = max_chars;
                            while !s.is_char_boundary(idx) {
                                idx -= 1;
                            }
                            chunks.push(s[..idx].to_string());
                            s = &s[idx..];
                        }
                        buf.push_str(s);
                    }
                }
                cur = buf;
            } else {
                cur = piece;
            }
        }
    }
    if !cur.is_empty() {
        chunks.push(cur);
    }
    chunks
}

/// Build the translation prompt (plan §2.2).
pub fn build_prompt(source_text: &str, target_locale: &str) -> String {
    format!(
        "Translate the following text into {target_locale}. Preserve all HTML tags, \
         markdown, @mentions, and URLs exactly as they are. Return ONLY the translated \
         text, with no explanations or preamble.\n\n{source_text}"
    )
}

/// Worker loop: drains Redis `translate_queue` (BRPOP), calls Ollama once
/// per new job, writes `machine` rows. Never exits, never panics.
pub async fn run_worker(
    db: PgPool,
    mut conn: redis::aio::MultiplexedConnection,
    config: Config,
    ollama_url: String,
    ollama_model: String,
) {
    info!("Translation worker started (model: {ollama_model})");
    let client = reqwest::Client::new();

    loop {
        let popped: Option<(String, String)> = redis::cmd("BRPOP")
            .arg("translate_queue")
            .arg(5u64)
            .query_async(&mut conn)
            .await
            .ok()
            .flatten();
        let Some((_list, raw)) = popped else { continue };

        let job: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(j) => j,
            Err(e) => {
                warn!("translation worker: bad job json: {e}");
                continue;
            }
        };
        let typ_str = job["type"].as_str().unwrap_or("");
        let key = job["id"].as_str().unwrap_or("");
        let field = job["field"].as_str().unwrap_or("");
        let locale = job["locale"].as_str().unwrap_or("");

        let Some(typ) = TranslationType::from_str(typ_str) else {
            continue;
        };

        // Source text + freshness check (dedupe by hash).
        let src = match fetch_source(&db, typ, key, field).await {
            Ok(Some(s)) => s,
            Ok(None) => continue,
            Err(e) => {
                warn!("translation worker: fetch_source failed {typ_str}:{key}:{field}: {e}");
                continue;
            }
        };
        let source_hash = sha256_hex(&src.text);
        let fresh: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM translation_strings \
             WHERE target_type=$1 AND target_id=$2 AND field=$3 AND locale=$4 AND source_hash=$5)",
        )
        .bind(typ_str)
        .bind(key)
        .bind(field)
        .bind(locale)
        .bind(&source_hash)
        .fetch_one(&db)
        .await
        .unwrap_or(false);
        if fresh {
            continue; // same source text already translated for this locale
        }

        // Translate chunk by chunk.
        let chunks = chunk_text(&src.text, config.translate_chunk_chars as usize);
        let mut done_chunks: Vec<String> = Vec::new();
        let mut failed = false;
        for chunk in &chunks {
            let req = serde_json::json!({
                "model": ollama_model,
                "prompt": build_prompt(chunk, locale),
                "stream": false,
                "options": { "temperature": 0.3 },
            });
            let translated = match client
                .post(format!("{ollama_url}/api/generate"))
                .json(&req)
                .timeout(std::time::Duration::from_secs(180))
                .send()
                .await
            {
                Ok(r) => match r.json::<serde_json::Value>().await {
                    Ok(v) => v["response"].as_str().unwrap_or("").trim().to_string(),
                    Err(_) => String::new(),
                },
                Err(e) => {
                    warn!("translation worker: ollama call failed: {e}");
                    String::new()
                }
            };
            if translated.is_empty() {
                failed = true;
                break;
            }
            done_chunks.push(translated);
        }
        if failed {
            // Put the job back once with a marker so it retries after the
            // queue drains; the SETNX dedupe expired path also allows a user
            // to re-enqueue manually.
            let retry = serde_json::json!({ "job": job, "attempt": 1 });
            let _: i64 = redis::cmd("RPUSH")
                .arg("translate_retry")
                .arg(retry.to_string())
                .query_async(&mut conn)
                .await
                .unwrap_or(0);
            continue;
        }

        let full = done_chunks.join("\n\n");
        let status = if config.translate_auto_approve_machine {
            "approved"
        } else {
            "machine"
        };
        let res = sqlx::query(
            r#"INSERT INTO translation_strings
                 (target_type, target_id, field, locale, source_hash, text, status, origin, model)
               VALUES ($1,$2,$3,$4,$5,$6,$7,'llm',$8)
               ON CONFLICT (target_type, target_id, field, locale) DO UPDATE SET
                 text = EXCLUDED.text,
                 source_hash = EXCLUDED.source_hash,
                 status = EXCLUDED.status,
                 origin = 'llm',
                 model = EXCLUDED.model,
                 updated_at = NOW()"#,
        )
        .bind(typ_str)
        .bind(key)
        .bind(field)
        .bind(locale)
        .bind(&source_hash)
        .bind(&full)
        .bind(status)
        .bind(&ollama_model)
        .execute(&db)
        .await;
        if let Err(e) = res {
            warn!("translation worker: upsert failed {typ_str}:{key}:{field}:{locale}: {e}");
            continue;
        }

        // Machine drafts go up for curator review as a pending proposal
        // (unless the instance auto-approves them), so the approve path has
        // the text ready when a curator votes.
        if status == "machine" && !config.translate_auto_approve_machine {
            let exists: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(SELECT 1 FROM proposals
                   WHERE kind='translate' AND target_type=$1 AND target_id=$2
                     AND status='pending' AND payload->>'field'=$3 AND payload->>'locale'=$4)"#,
            )
            .bind(typ_str)
            .bind(key)
            .bind(field)
            .bind(locale)
            .fetch_one(&db)
            .await
            .unwrap_or(true);
            if !exists {
                let payload = serde_json::json!({
                    "field": field,
                    "locale": locale,
                    "text": full,
                    "source_hash": source_hash,
                });
                let _ = crate::services::proposals::submit(
                    &db,
                    "translate",
                    typ_str,
                    key,
                    payload,
                    None,
                    "llm",
                )
                .await;
            }
        }

        // Opportunistically feed the retry list back (max 5 per cycle).
        for _ in 0..5 {
            let r: Option<(String, String)> = redis::cmd("RPOPLPUSH")
                .arg("translate_retry")
                .arg("translate_queue")
                .query_async(&mut conn)
                .await
                .ok()
                .flatten();
            if r.is_none() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_roundtrip() {
        for t in [
            "work_meta",
            "request",
            "request_answer",
            "forum_topic",
            "forum_post",
            "comment",
            "review",
        ] {
            assert_eq!(TranslationType::from_str(t).unwrap().as_str(), t);
        }
        assert!(TranslationType::from_str("bogus").is_none());
    }

    #[test]
    fn fields_per_type() {
        assert!(TranslationType::WorkMeta.allows_field("summary"));
        assert!(TranslationType::WorkMeta.allows_field("title"));
        assert!(!TranslationType::WorkMeta.allows_field("body"));
        assert!(TranslationType::Comment.allows_field("body"));
        assert!(!TranslationType::Comment.allows_field("title"));
        assert!(TranslationType::RequestAnswer.allows_field("pitch"));
    }

    #[test]
    fn chunk_splits_on_paragraphs() {
        // Build a long-enough text that paragraphs alone sum > 200 (the func floor).
        let long_para = "A.".repeat(80); // 160 chars
        let short_para = "B.".repeat(70); // 140 chars
        let text = format!("{long_para}\n\n{short_para}"); // 302 chars total
        let chunks = chunk_text(&text, 200);
        assert!(
            chunks.len() >= 2,
            "expected >= 2 chunks got {}",
            chunks.len()
        );
        assert!(chunks.iter().all(|c| c.chars().count() <= 200));
    }

    #[test]
    fn chunk_preserves_short_text() {
        let text = "Short text.";
        let chunks = chunk_text(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn chunk_handles_no_newlines_long_text() {
        // Requested 100 → clamped to 200; verify split works at the real budget.
        let text = "x".repeat(600);
        let chunks = chunk_text(&text, 100);
        assert_eq!(chunks.iter().map(|c| c.chars().count()).sum::<usize>(), 600);
        assert!(chunks.iter().all(|c| c.chars().count() <= 200));
    }

    #[test]
    fn chunk_respects_unicode_boundaries() {
        // 3-byte € chars: 250 of them = 750 bytes. Budget 200 floor.
        let text = "€".repeat(700);
        let chunks = chunk_text(&text, 100);
        assert!(chunks.iter().all(|c| c.chars().count() <= 200));
        assert_eq!(chunks.iter().map(|c| c.chars().count()).sum::<usize>(), 700);
    }

    #[test]
    fn hash_stable() {
        assert_eq!(sha256_hex("hello world"), sha256_hex("hello world"));
        assert_ne!(sha256_hex("hello"), sha256_hex("world"));
    }

    #[test]
    fn prompt_contains_locale_and_text() {
        let p = build_prompt("Hello world", "de");
        assert!(p.contains("de"));
        assert!(p.contains("Hello world"));
        assert!(p.contains("ONLY the translated text"));
    }
}
