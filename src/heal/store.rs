//! SQLx persistence for scrape-failure telemetry + the agent-run ledger.
//!
//! All timestamps are `TIMESTAMPTZ`; rows decode into `DateTime<Utc>` (the
//! sqlx gotcha from the fichub-development reference — never `NaiveDateTime`).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::FromRow;

/// A `scrape_failures` row (mirrors migration 029).
#[derive(Debug, Clone, FromRow)]
pub struct ScrapeFailureRow {
    pub id: i64,
    pub url: String,
    pub url_id: Option<String>,
    pub domain: String,
    pub error_kind: String,
    pub message: Option<String>,
    pub html_snapshot_path: Option<String>,
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

/// Payload for inserting a scrape failure (what the instrumentation points
/// produce at the choke points).
#[derive(Debug, Clone)]
pub struct NewFailure {
    pub url: String,
    pub url_id: Option<String>,
    pub domain: String,
    pub error_kind: String,
    pub message: Option<String>,
    pub html_snapshot_path: Option<String>,
    pub fingerprint: String,
}

/// INSERT a failure and return the persisted row.
pub async fn record_failure(pool: &PgPool, f: NewFailure) -> Result<ScrapeFailureRow, sqlx::Error> {
    sqlx::query_as::<_, ScrapeFailureRow>(
        r#"INSERT INTO scrape_failures
           (url, url_id, domain, error_kind, message, html_snapshot_path, fingerprint)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, url, url_id, domain, error_kind, message,
                     html_snapshot_path, fingerprint, created_at, resolved_at, resolution"#,
    )
    .bind(&f.url)
    .bind(&f.url_id)
    .bind(&f.domain)
    .bind(&f.error_kind)
    .bind(&f.message)
    .bind(&f.html_snapshot_path)
    .bind(&f.fingerprint)
    .fetch_one(pool)
    .await
}

/// A `pending_exports` row (mirrors migration 033).
#[derive(Debug, Clone, FromRow)]
pub struct PendingExportRow {
    pub id: i64,
    pub url: String,
    pub format: String,
    /// Raw IP string (the INET column decodes via `::text` in the SELECT).
    pub client_ip: Option<String>,
    pub client_id: Option<String>,
    pub status: String,
    pub attempts: i32,
    pub error_kind: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Pending (uncompleted) export requests, oldest first.
pub async fn list_pending_exports(pool: &PgPool, limit: i64) -> Result<Vec<PendingExportRow>, sqlx::Error> {
    sqlx::query_as::<_, PendingExportRow>(
        r#"SELECT id, url, format, client_ip::text, client_id, status, attempts,
                  error_kind, created_at, completed_at
           FROM pending_exports
           WHERE status = 'pending'
           ORDER BY created_at ASC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Failures for a domain recorded since `since` (UTC). Ordered newest-first.
pub async fn recent_failures(
    pool: &PgPool,
    domain: &str,
    since: DateTime<Utc>,
) -> Result<Vec<ScrapeFailureRow>, sqlx::Error> {
    sqlx::query_as::<_, ScrapeFailureRow>(
        r#"SELECT id, url, url_id, domain, error_kind, message,
                  html_snapshot_path, fingerprint, created_at, resolved_at, resolution
           FROM scrape_failures
           WHERE domain = $1 AND created_at >= $2
           ORDER BY created_at DESC"#,
    )
    .bind(domain)
    .bind(since)
    .fetch_all(pool)
    .await
}

/// Mark a failure resolved (used by later milestones; kept for completeness).
pub async fn mark_resolved(
    pool: &PgPool,
    id: i64,
    resolution: &str,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE scrape_failures SET resolved_at = now(), resolution = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(resolution)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// An `agent_runs` row (mirrors migration 030).
#[derive(Debug, Clone, FromRow)]
pub struct AgentRunRow {
    pub id: i64,
    pub trigger_type: String,
    pub trigger_ref: Option<i64>,
    pub class: String,
    pub model: Option<String>,
    pub iterations: i32,
    pub tokens: i32,
    pub status: String,
    pub diff_summary: Option<String>,
    pub tests_passed: Option<bool>,
    pub merged: bool,
    pub deployed: bool,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

/// INSERT a new agent run (status defaults to 'pending').
pub async fn record_agent_run(
    pool: &PgPool,
    trigger_type: &str,
    trigger_ref: Option<i64>,
    class: &str,
    model: Option<&str>,
) -> Result<AgentRunRow, sqlx::Error> {
    sqlx::query_as::<_, AgentRunRow>(
        r#"INSERT INTO agent_runs (trigger_type, trigger_ref, class, model)
           VALUES ($1, $2, $3, $4)
           RETURNING id, trigger_type, trigger_ref, class, model, iterations, tokens,
                     status, diff_summary, tests_passed, merged, deployed,
                     created_at, finished_at"#,
    )
    .bind(trigger_type)
    .bind(trigger_ref)
    .bind(class)
    .bind(model)
    .fetch_one(pool)
    .await
}

/// Update a run's status + diff_summary (and finish timestamp for terminal
/// statuses). Returns true when the row was updated.
pub async fn update_agent_run(
    pool: &PgPool,
    id: i64,
    status: &str,
    diff_summary: Option<&str>,
    iterations: Option<i32>,
    tokens: Option<i32>,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        r#"UPDATE agent_runs
           SET status = $2,
               diff_summary = COALESCE($3, diff_summary),
               iterations = COALESCE($4, iterations),
               tokens = COALESCE($5, tokens),
               finished_at = CASE WHEN $2 IN ('diagnosed','proposed','merged','deployed','failed','paused')
                                  THEN COALESCE(finished_at, now()) ELSE finished_at END
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(status)
    .bind(diff_summary)
    .bind(iterations)
    .bind(tokens)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Recent runs, newest first.
pub async fn list_agent_runs(pool: &PgPool, limit: i64) -> Result<Vec<AgentRunRow>, sqlx::Error> {
    sqlx::query_as::<_, AgentRunRow>(
        r#"SELECT id, trigger_type, trigger_ref, class, model, iterations, tokens,
                  status, diff_summary, tests_passed, merged, deployed,
                  created_at, finished_at
           FROM agent_runs ORDER BY created_at DESC LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// The most recent run for a fingerprint (via trigger_ref = failure id), used
/// to gate one agent diagnose per 24h per fingerprint.
pub async fn latest_agent_run_for_fingerprint(
    pool: &PgPool,
    failure_id: i64,
) -> Result<Option<AgentRunRow>, sqlx::Error> {
    sqlx::query_as::<_, AgentRunRow>(
        r#"SELECT id, trigger_type, trigger_ref, class, model, iterations, tokens,
                  status, diff_summary, tests_passed, merged, deployed,
                  created_at, finished_at
           FROM agent_runs
           WHERE trigger_ref = $1
           ORDER BY created_at DESC
           LIMIT 1"#,
    )
    .bind(failure_id)
    .fetch_optional(pool)
    .await
}

/// Count runs created in the last `since` — the max-runs-per-day gate.
pub async fn count_agent_runs_since(
    pool: &PgPool,
    since: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM agent_runs WHERE created_at >= $1")
        .bind(since)
        .fetch_one(pool)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pure (no-DB) sanity: NewFailure→ScrapeFailureRow shape matches the
    /// migration columns; real round-trips live in tests/heal_api.rs.
    #[test]
    fn new_failure_fields_match_migration() {
        let f = NewFailure {
            url: "https://example.com/s/1".into(),
            url_id: Some("abc".into()),
            domain: "example.com".into(),
            error_kind: "parse".into(),
            message: Some("no h1".into()),
            html_snapshot_path: None,
            fingerprint: "deadbeefcafe".into(),
        };
        assert_eq!(f.error_kind, "parse");
        assert_eq!(f.fingerprint.len(), 12);
    }
}
