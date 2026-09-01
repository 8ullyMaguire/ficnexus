//! Background bookmark CSV import worker.
//!
//! `POST /api/bookmarks/import` only validates the request and enqueues a
//! job (JSON) onto the Redis list `bookmark_import_queue`; the actual CSV
//! parsing, work resolution, de-duplication and insertion happen here, off
//! the request path, in a tokio task spawned at server startup (see
//! `server::run`). When the import finishes the worker creates a
//! `bookmark_import` notification for the user so the frontend bell lights
//! up with the result.
//!
//! This mirrors the project's existing background-queue pattern
//! (`CollectionWorker` in `src/recommender/worker.rs`): Redis list + LPUSH
//! enqueue + BRPOP worker loop.

use std::time::Duration;

use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::db::queries;
use crate::error::AppResult;

/// Redis list key holding pending bookmark imports (JSON `ImportJob`s).
pub const BOOKMARK_IMPORT_QUEUE: &str = "bookmark_import_queue";

/// A queued bookmark import job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub user_id: i32,
    /// Raw CSV bytes (UTF-8 lossy-decoded at enqueue time).
    pub csv: String,
}

/// Result of a single completed import job.
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub imported: usize,
    pub duplicates: usize,
    pub errors: Vec<String>,
}

/// Background worker that drains `bookmark_import_queue`.
pub struct BookmarkImportWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
}

impl BookmarkImportWorker {
    pub fn new(db: PgPool, redis: MultiplexedConnection) -> Self {
        Self {
            db,
            redis: Mutex::new(redis),
        }
    }

    /// Push a job onto the queue (called by the import handler).
    pub async fn enqueue(&self, job: &ImportJob) -> Result<(), redis::RedisError> {
        self.enqueue_to(BOOKMARK_IMPORT_QUEUE, job).await
    }

    /// Push a job onto a specific queue key. Exposed so tests can verify the
    /// enqueue primitive without racing the live production worker on the
    /// shared queue.
    pub async fn enqueue_to(
        &self,
        key: &str,
        job: &ImportJob,
    ) -> Result<(), redis::RedisError> {
        let json = serde_json::to_string(job).expect("ImportJob serialisation should not fail");
        let mut conn = self.redis.lock().await;
        redis::cmd("LPUSH")
            .arg(key)
            .arg(json)
            .query_async(&mut *conn)
            .await
    }

    /// Main worker loop — runs forever, popping jobs off the queue.
    pub async fn run(&self) {
        info!("Bookmark import worker started — polling {BOOKMARK_IMPORT_QUEUE}");
        loop {
            // BRPOP returns [key, value] as a (String, String) tuple.
            let item: Option<(String, String)> = {
                let mut conn = self.redis.lock().await;
                redis::cmd("BRPOP")
                    .arg(BOOKMARK_IMPORT_QUEUE)
                    .arg(0u64) // block indefinitely
                    .query_async(&mut *conn)
                    .await
                    .unwrap_or(None)
            };

            let Some((_key, json_str)) = item else {
                continue;
            };

            match serde_json::from_str::<ImportJob>(&json_str) {
                Ok(job) => {
                    let result = process_import(&self.db, &job).await;
                    match result {
                        Ok(res) => {
                            info!(
                                "Bookmark import for user {} done: {} imported, {} duplicates, {} errors",
                                job.user_id, res.imported, res.duplicates, res.errors.len()
                            );
                            notify_import_done(&self.db, &job, &res).await;
                        }
                        Err(e) => {
                            error!("Bookmark import job failed for user {}: {e}", job.user_id);
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid bookmark import queue item: {e} — payload: {json_str}");
                }
            }
        }
    }
}

/// Parse one CSV line into fields, honouring double-quoted fields with
/// `""` escapes and embedded commas.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !in_quotes => in_quotes = true,
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Split a raw CSV string into logical rows, honouring quoted fields that
/// contain embedded newlines.
fn split_csv_rows(csv: &str) -> Vec<String> {
    let mut rows = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = csv.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '\n' if !in_quotes => {
                rows.push(current.trim_end_matches('\r').to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        rows.push(current.trim().to_string());
    }
    rows
}

// ── CSV parsing helpers (pure, deterministic) ──────────────────────────
// The parse helpers are private `fn`s — tested via the same-file module.

#[cfg(test)]
mod csv_tests {
    use super::{parse_csv_line, split_csv_rows};

    #[test]
    fn parse_csv_line_splits_plain_fields() {
        assert_eq!(parse_csv_line("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(parse_csv_line("a"), vec!["a"]);
        assert_eq!(parse_csv_line(""), vec![""]);
    }

    #[test]
    fn parse_csv_line_handles_quoted_commas_and_escapes() {
        assert_eq!(
            parse_csv_line(r#""Harry, the Boy","Rowling","ao3""#),
            vec!["Harry, the Boy", "Rowling", "ao3"]
        );
        // "" escape inside a quoted field yields a literal quote.
        assert_eq!(
            parse_csv_line(r#""He said ""hi""",author"#),
            vec!["He said \"hi\"", "author"]
        );
    }

    #[test]
    fn parse_csv_line_trims_fields() {
        assert_eq!(parse_csv_line(" a , b , c "), vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_csv_line_treats_mid_field_quote_as_literal() {
        // A quote in the middle of an unquoted field is kept verbatim in the
        // current field (the parser only toggles quoting at field edges).
        assert_eq!(parse_csv_line("ab\"cd,e"), vec!["abcd,e"]);
    }

    #[test]
    fn split_csv_rows_splits_on_unquoted_newlines() {
        let csv = "a,b\nc,d\n";
        assert_eq!(split_csv_rows(csv), vec!["a,b", "c,d"]);
    }

    #[test]
    fn split_csv_rows_keeps_quoted_newlines_inside_fields() {
        let csv = "\"multi\nline\",field\nnext,row";
        let rows = split_csv_rows(csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "\"multi\nline\",field");
        assert_eq!(rows[1], "next,row");
    }

    #[test]
    fn split_csv_rows_strips_carriage_returns() {
        assert_eq!(split_csv_rows("a,b\r\nc,d\r\n"), vec!["a,b", "c,d"]);
    }

    #[test]
    fn split_csv_rows_drops_trailing_blank_row() {
        // A trailing newline does not produce a phantom empty row.
        assert_eq!(split_csv_rows("a,b\n"), vec!["a,b"]);
    }

    #[test]
    fn split_csv_rows_keeps_explicit_blank_row_in_middle() {
        // An empty line BETWEEN rows yields an empty row string (callers
        // skip `line.trim().is_empty()` in process_import).
        assert_eq!(split_csv_rows("a,b\n\nc,d"), vec!["a,b", "", "c,d"]);
    }

    #[test]
    fn split_csv_rows_handles_empty_input() {
        assert_eq!(split_csv_rows(""), Vec::<String>::new());
        assert_eq!(split_csv_rows("   "), Vec::<String>::new());
    }

    #[test]
    fn parse_and_split_compose_for_realistic_exports() {
        let csv = "title,author,source\n\"The, Great\",\"Author, Name\",https://ao3.org/works/1\nPlain Title,Author Two,https://ffn.example/s/2";
        let rows = split_csv_rows(csv);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], "title,author,source");
        let fields = parse_csv_line(&rows[1]);
        assert_eq!(fields[0], "The, Great");
        assert_eq!(fields[1], "Author, Name");
        let fields2 = parse_csv_line(&rows[2]);
        assert_eq!(
            fields2,
            vec!["Plain Title", "Author Two", "https://ffn.example/s/2"]
        );
    }
}

/// Process a queued import job: parse, resolve works, insert bookmarks
/// (deduplicated), and report errors per malformed/unresolved row.
pub async fn process_import(pool: &PgPool, job: &ImportJob) -> AppResult<ImportResult> {
    let mut imported = 0usize;
    let mut duplicates = 0usize;
    let mut errors: Vec<String> = Vec::new();

    let rows = split_csv_rows(&job.csv);
    for (i, line) in rows.iter().enumerate() {
        // Skip header row (first non-empty line starting with "title")
        if i == 0 && line.to_lowercase().starts_with("title") {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }

        let fields = parse_csv_line(line);
        if fields.len() < 3 {
            errors.push(format!(
                "Row {}: Expected at least 3 columns (title, author, source), got {}",
                i + 1,
                fields.len()
            ));
            continue;
        }

        let title = fields[0].trim().to_string();
        let author = fields[1].trim().to_string();
        let source = fields[2].trim().to_string();

        if source.is_empty() && title.is_empty() {
            continue;
        }

        // Resolve the work: by source URL first, then by title+author.
        let work = if source.starts_with("http://") || source.starts_with("https://") {
            queries::find_work_by_source_url(pool, &source).await?
        } else {
            None
        };
        let work = match work {
            Some(w) => Some(w),
            None => {
                if !title.is_empty() && !author.is_empty() {
                    queries::find_work_by_title_author(pool, &title, &author).await?
                } else {
                    None
                }
            }
        };

        let Some(work) = work else {
            errors.push(format!(
                "Row {}: Work not found: \"{}\" by \"{}\" (source: {})",
                i + 1,
                title,
                author,
                source
            ));
            continue;
        };

        // Resolve the url_id (fic_info.id) for this work. Prefer the work's
        // default source; fall back to any fic_info row linked to the work.
        let url_id = resolve_url_id_for_work(pool, work.id).await?;
        let Some(url_id) = url_id else {
            errors.push(format!(
                "Row {}: Work found but has no fic_info source: \"{}\"",
                i + 1,
                title
            ));
            continue;
        };

        let insert = sqlx::query(
            "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
             VALUES ($1, $2, $3, '', FALSE)
             ON CONFLICT (user_id, url_id) DO NOTHING",
        )
        .bind(job.user_id)
        .bind(&url_id)
        .bind(work.id)
        .execute(pool)
        .await?;

        if insert.rows_affected() > 0 {
            imported += 1;
        } else {
            duplicates += 1;
        }
    }

    Ok(ImportResult {
        imported,
        duplicates,
        errors,
    })
}

/// Resolve the `url_id` (fic_info.id) for a work id, preferring the work's
/// default source. Exposed for handlers that insert bookmarks directly
/// (`bookmarks.url_id` is NOT NULL).
pub async fn resolve_url_id_for_work(pool: &PgPool, work_id: i32) -> AppResult<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"SELECT COALESCE(
               (SELECT fi.id FROM fic_info fi WHERE fi.work_id = w.id AND fi.id = w.default_source_id LIMIT 1),
               (SELECT fi.id FROM fic_info fi WHERE fi.work_id = w.id LIMIT 1)
           )
           FROM works w WHERE w.id = $1"#,
    )
    .bind(work_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// Create a `bookmark_import` notification for the user when a job completes.
/// Public so tests (and future callers) can drive the worker's exact
/// post-processing notification path.
pub async fn notify_import_done(pool: &PgPool, job: &ImportJob, res: &ImportResult) {
    let (title, body, ok) = if res.errors.is_empty() {
        (
            format!(
                "Bookmark import complete: {} imported{}",
                res.imported,
                if res.duplicates > 0 {
                    format!(", {} duplicates skipped", res.duplicates)
                } else {
                    String::new()
                }
            ),
            Some("Your CSV bookmark import finished successfully.".to_string()),
            true,
        )
    } else {
        (
            format!(
                "Bookmark import finished with {} issue(s): {} imported, {} duplicates",
                res.errors.len(),
                res.imported,
                res.duplicates
            ),
            Some(format!(
                "{} row(s) could not be imported. See the bookmarks page for details.",
                res.errors.len()
            )),
            false,
        )
    };

    let body = body.as_deref();
    let reference_id = Some(if ok { "ok" } else { "errors" });

    if let Err(e) = queries::create_notification(
        pool,
        job.user_id,
        "bookmark_import",
        &title,
        body,
        Some("/bookmarks"),
        Some("bookmark_import"),
        reference_id,
    )
    .await
    {
        error!("Failed to create bookmark import notification: {e}");
    }
}

/// Convenience for enqueueing from a handler that has `&AppState`
/// (the handler itself constructs a lightweight worker).
pub async fn enqueue_import(
    db: &PgPool,
    redis: &MultiplexedConnection,
    job: &ImportJob,
) -> Result<(), redis::RedisError> {
    let worker = BookmarkImportWorker::new(db.clone(), redis.clone());
    worker.enqueue(job).await
}

/// Small helper for tests: wait for the worker to drain the queue.
pub async fn drain_queue_for(timeout: Duration) {
    tokio::time::sleep(timeout).await;
}
