//! External sidecar strategy — HTTP adapter for optional Python sidecars.
//!
//! When `REC_EXTERNAL_URL` is set (e.g. `http://127.0.0.1:8300`), this
//! strategy proxies scoring to a sidecar that speaks a tiny JSON protocol:
//!
//!   POST {base}/score
//!   {"seed": "<url_id or null>", "user_id": <int or null>, "n": 20}
//!   → 200 {"recs": [{"work_id": "...", "score": 0.5, "reason": "..."}]}
//!
//! The sidecar is OPTIONAL and NOT part of the default runtime; recipes live
//! in `rec-engines/` (LightFM, implicit, VW-bandit, RecBole). A failing or
//! slow sidecar degrades to "not enough data" so the ranker's fallback chain
//! continues with the other strategies.

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Request payload sent to the sidecar.
#[derive(Debug, serde::Serialize)]
pub struct ExternalRequest {
    pub seed: Option<String>,
    pub user_id: Option<i32>,
    pub n: usize,
}

/// Response contract the sidecar must honor.
#[derive(Debug, serde::Deserialize)]
pub struct ExternalResponse {
    pub recs: Vec<ExternalRec>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ExternalRec {
    pub work_id: String,
    pub score: f64,
    #[serde(default)]
    pub reason: Option<String>,
}

pub struct ExternalStrategy {
    /// Base URL, e.g. "http://127.0.0.1:8300". Stored so `score` can use it
    /// without re-reading env (config is the source of truth at build time).
    base_url: Option<String>,
}

impl ExternalStrategy {
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("REC_EXTERNAL_URL").ok().filter(|s| !s.is_empty()),
        }
    }
}

impl Default for ExternalStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RecStrategy for ExternalStrategy {
    fn name(&self) -> &str {
        "external"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        let Some(base) = &self.base_url else {
            return Err(RecError::NotEnoughData(
                "external sidecar not configured (REC_EXTERNAL_URL)".into(),
            ));
        };
        let payload = ExternalRequest {
            seed: seed.map(|s| s.to_string()),
            user_id,
            n: ctx.config.rec_max_recommendations,
        };
        let call = ctx.http_client.post(format!("{}/score", base.trim_end_matches('/'))).json(&payload).send();
        let resp = tokio::time::timeout(ctx.call_timeout(), call)
            .await
            .map_err(|_| RecError::External("sidecar timed out".into()))?
            .map_err(|e| RecError::External(format!("sidecar request failed: {e}")))?;
        if !resp.status().is_success() {
            return Err(RecError::External(format!("sidecar HTTP {}", resp.status())));
        }
        let body: ExternalResponse = resp
            .json()
            .await
            .map_err(|e| RecError::External(format!("sidecar bad body: {e}")))?;
        if body.recs.is_empty() {
            return Err(RecError::NotEnoughData("sidecar returned no recs".into()));
        }
        Ok(body
            .recs
            .into_iter()
            .map(|r| ScoredRec {
                work_id: r.work_id,
                score: r.score,
                strategy: self.name().into(),
                reason: r.reason.unwrap_or_else(|| "external model".into()),
            })
            .collect())
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        let Some(base) = &self.base_url else {
            return Err(RecError::NotEnoughData("external sidecar not configured".into()));
        };
        let call = ctx
            .http_client
            .post(format!("{}/train", base.trim_end_matches('/')))
            .json(&json!({}))
            .send();
        let resp = tokio::time::timeout(ctx.call_timeout(), call)
            .await
            .map_err(|_| RecError::External("sidecar train timed out".into()))?
            .map_err(|e| RecError::External(format!("sidecar train failed: {e}")))?;
        if !resp.status().is_success() {
            return Err(RecError::External(format!("sidecar train HTTP {}", resp.status())));
        }
        Ok(json!({ "sidecar": "trained", "base": base }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconfigured_sidecar_reports_not_configured() {
        // No REC_EXTERNAL_URL in the test env → strategy is inert.
        unsafe { std::env::remove_var("REC_EXTERNAL_URL") };
        let s = ExternalStrategy::new();
        assert!(s.base_url.is_none());
        assert_eq!(s.name(), "external");
    }
}
