//! Self-healing loop — milestone 1: scrape-failure telemetry + diagnose-only
//! agent trigger.
//!
//! * [`classifier`] — pure-Rust error classification (ErrorKind → Class),
//!   fingerprinting, and debounce.
//! * [`store`] — sqlx persistence for `scrape_failures` / `agent_runs`.
//! * [`snapshot`] — HTML snapshot capture for parse failures.
//! * [`agent`] — OpenAI-compatible chat/completions client for the diagnose
//!   call (CommandCode default, Ollama local fallback).
//! * [`HealService`] — the AppState field the admin route and the scrape
//!   choke-points use.

pub mod agent;
pub mod classifier;
pub mod extract;
pub mod snapshot;
pub mod store;

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

/// Facade over the failure/agent-run persistence + agent endpoint. One
/// instance per `AppState` (constructed in tests with the same `db`/`config`
/// the rest of the state uses).
#[derive(Clone)]
pub struct HealService {
    pub db: PgPool,
    pub config: Arc<Config>,
}

impl HealService {
    pub fn new(db: PgPool, config: Config) -> Self {
        HealService {
            db,
            config: Arc::new(config),
        }
    }

    /// Record a scrape failure with classification + fingerprint applied.
    /// Best-effort: a DB hiccup must never break the export path, so errors
    /// are logged and swallowed.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_failure(
        &self,
        url: &str,
        url_id: Option<&str>,
        kind: &classifier::ErrorKind,
        message: Option<&str>,
        html_snapshot_path: Option<&str>,
    ) {
        let domain = classifier::url_domain(url).unwrap_or_else(|| url.to_string());
        let fp = classifier::fingerprint(url, kind, message.unwrap_or(""));
        let f = store::NewFailure {
            url: url.to_string(),
            url_id: url_id.map(|s| s.to_string()),
            domain,
            error_kind: kind.as_str().to_string(),
            message: message.map(|s| s.to_string()),
            html_snapshot_path: html_snapshot_path.map(|s| s.to_string()),
            fingerprint: fp,
        };
        if let Err(e) = store::record_failure(&self.db, f).await {
            tracing::warn!("heal: failed to record scrape failure for {}: {}", url, e);
        }
    }
}
