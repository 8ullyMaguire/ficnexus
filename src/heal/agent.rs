//! Diagnose-only agent client (OpenAI-compatible chat/completions).
//!
//! Milestone 1: a SINGLE diagnose call asking the agent what changed at a
//! site given a failure summary. The reply is stored verbatim in
//! `agent_runs.diff_summary` (status 'diagnosed'); any transport error
//! records status 'failed' with `diff_summary = "agent unreachable"`.
//!
//! Remote endpoint defaults to CommandCode (see docs/AGENTS.md); the local
//! Ollama fallback is used when the remote is not configured.

use serde::{Deserialize, Serialize};

use crate::config::Config;

const DEFAULT_MODEL: &str = "deepseek/deepseek-v4-flash";

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

/// Whether a remote diagnose call is possible: `AGENT_ENABLED=true` AND a
/// remote base URL AND an API key.
pub fn remote_configured(cfg: &Config) -> bool {
    cfg.agent_enabled
        && !cfg.agent_base_url.is_empty()
        && cfg.agent_api_key.is_some()
}

/// Run a single diagnose call against the remote OpenAI-compatible endpoint.
/// Returns the agent's reply text.
pub async fn diagnose_remote(
    http: &reqwest::Client,
    cfg: &Config,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    let base = cfg.agent_base_url.trim_end_matches('/').to_string();
    let url = format!("{base}/chat/completions");
    let key = cfg
        .agent_api_key
        .as_deref()
        .ok_or_else(|| "no agent API key configured".to_string())?;
    let model = if cfg.agent_model.is_empty() {
        DEFAULT_MODEL.to_string()
    } else {
        cfg.agent_model.clone()
    };

    let body = ChatRequest {
        model,
        messages: vec![
            ChatMessage { role: "system", content: system_prompt.to_string() },
            ChatMessage { role: "user", content: user_prompt.to_string() },
        ],
        max_tokens: 600,
    };

    let resp = http
        .post(&url)
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("agent request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("agent endpoint returned HTTP {status}: {text}"));
    }

    let parsed: ChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("agent response unparseable: {e}"))?;
    parsed
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "agent returned an empty response".to_string())
}

/// Local fallback diagnose call (Ollama `/api/chat`).
pub async fn diagnose_local(
    http: &reqwest::Client,
    cfg: &Config,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    #[derive(Serialize, Deserialize)]
    struct LocalMessage {
        role: String,
        content: String,
    }
    #[derive(Serialize, Deserialize)]
    struct LocalRequest {
        model: String,
        messages: Vec<LocalMessage>,
        stream: bool,
    }
    #[derive(Deserialize)]
    struct LocalResponse {
        message: LocalMessage,
    }

    let model = if cfg.agent_model.is_empty() {
        DEFAULT_MODEL.to_string()
    } else {
        cfg.agent_model.clone()
    };
    let url = format!("{}/api/chat", cfg.agent_ollama_url.trim_end_matches('/'));

    let body = LocalRequest {
        model,
        messages: vec![
            LocalMessage { role: "system".into(), content: system_prompt.to_string() },
            LocalMessage { role: "user".into(), content: user_prompt.to_string() },
        ],
        stream: false,
    };

    let resp = http
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("ollama request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("ollama returned HTTP {}", resp.status()));
    }
    let parsed: LocalResponse = resp
        .json()
        .await
        .map_err(|e| format!("ollama response unparseable: {e}"))?;
    if parsed.message.content.trim().is_empty() {
        return Err("ollama returned an empty response".to_string());
    }
    Ok(parsed.message.content)
}

/// System prompt for the metadata-extraction call: STRICT JSON only.
const EXTRACT_SYSTEM_PROMPT: &str = "\
You extract fanfiction metadata from an HTML snapshot of a story page. \
The site's regular scraper failed to parse this page, so you are the \
on-the-fly scraper. Rules:\n\
1. Output ONLY one JSON object, nothing else — no markdown, no prose, no \
explanations. A fenced ```json block is acceptable.\n\
2. The JSON object must have exactly these keys:\n\
   {\"title\": string, \"author\": string, \"chapters\": integer, \
\"words\": integer, \"desc\": string, \"status\": string, \
\"source\": string, \"url_id\": string}\n\
3. status must be one of: ongoing, complete, hiatus, cancelled.\n\
4. source must be the story URL you were given.\n\
5. url_id: a short stable site-local identifier (e.g. ao3_21845264); if \
the page has none, use the story id found in the URL.\n\
6. desc may be empty if the page has no summary. words may be 0 if \
unknown.\n";

/// Extract structured metadata from an HTML snapshot via the remote
/// OpenAI-compatible endpoint. Returns the raw agent reply (STRICT JSON).
pub async fn extract_metadata_remote(
    http: &reqwest::Client,
    cfg: &Config,
    snapshot_html: &str,
    source_url: &str,
) -> Result<String, String> {
    let truncated: String = snapshot_html
        .chars()
        .take(cfg.agent_extract_max_snapshot_chars)
        .collect();
    let user_prompt = format!(
        "Source URL: {source_url}\n\nHTML snapshot (truncated):\n{truncated}"
    );
    diagnose_remote(http, cfg, EXTRACT_SYSTEM_PROMPT, &user_prompt)
        .await
}

/// Extract structured metadata from an HTML snapshot via local Ollama.
/// Returns the raw agent reply (STRICT JSON).
pub async fn extract_metadata_local(
    http: &reqwest::Client,
    cfg: &Config,
    snapshot_html: &str,
    source_url: &str,
) -> Result<String, String> {
    let truncated: String = snapshot_html
        .chars()
        .take(cfg.agent_extract_max_snapshot_chars)
        .collect();
    let user_prompt = format!(
        "Source URL: {source_url}\n\nHTML snapshot (truncated):\n{truncated}"
    );
    diagnose_local(http, cfg, EXTRACT_SYSTEM_PROMPT, &user_prompt)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_constant_matches_plan() {
        assert_eq!(DEFAULT_MODEL, "deepseek/deepseek-v4-flash");
    }

    #[test]
    fn remote_configured_requires_all_parts() {
        // Build a Config directly (no env) so the test is hermetic.
        let mut cfg = crate::config::Config {
            database_url: "postgres://x".into(),
            redis_url: "redis://x".into(),
            cache_dir: std::path::PathBuf::from("."),
            secondary_cache_dir: None,
            export_version: 1,
            dynamic_rate_limit: false,
            node_name: "test".into(),
            calibre_container: String::new(),
            tmp_dir: std::path::PathBuf::from("."),
            body_cache_dir: std::path::PathBuf::from("/tmp/fichub-body-cache-test"),
            app_port: 3000,
            frontend_dir: std::path::PathBuf::from("."),
            trusted_proxies: Vec::new(),
            ip_tag_sources: Vec::new(),
            maxmind_db: None,
            rec_default_delay_secs: 5,
            rec_site_rate_limits: std::collections::HashMap::new(),
            rec_max_favourite_pages: 3,
            rec_max_user_favourite_pages: 3,
            rec_max_recommendations: 20,
            rec_min_favouriters_for_collab: 5,
            rec_voting_boost_gamma: 0.2,
            rec_cache_ttl_hours: 12,
            rec_suggest_limit_per_hour: 5,
            rec_vote_limit_per_hour: 10,
            rec_precompute_enabled: true,
            rec_precompute_interval_hours: 6,
            rec_enable_cross_site: true,
            rec_engine_mode: crate::config::RecEngineMode::Legacy,
            rec_strategies: "cooccur".into(),
            rec_decay_halflife_days: 30.0,
            rec_embed_model: "nomic-embed-text".into(),
            rec_embed_dim: 384,
            rec_mf_factors: 16,
            rec_mf_iters: 10,
            rec_mf_train_min_signals: 500,
            rec_bandit_slots: 1,
            rec_train_every_h: 6,
            rec_shadow_mode: false,
            rec_curator_prior: None,
            rec_prior_floor: 0.2,
            rec_curator_tau: 25.0,
            rec_strategy_timeout_secs: 10,
            rec_external_url: None,
            curator_token: None,
            tag_hidden_threshold: -3,
            tag_auto_delete_threshold: None,
            tag_submit_limit_per_hour: 10,
            tag_vote_limit_per_hour: 20,
            search_max_per_page: 50,
            max_upload_bytes: 20 * 1024 * 1024,
            wayback_fallback_enabled: false,
            wayback_max_snapshot_age_days: 365,
            wayback_cdx_rate_limit_per_sec: 1,
            opds_shelf_token: "fichub".into(),
            opds_base_url: None,
            public_origin: "http://localhost:8000".into(),
            rl_download_capacity: 10.0,
            rl_download_flow: 60.0 / 3600.0,
            rl_auth_capacity: 10.0,
            rl_auth_flow: 10.0 / 60.0,
            rl_search_capacity: 1000.0,
            rl_search_flow: 1000.0 / 60.0,
            rl_client_bonus_capacity: 5.0,
            rl_client_bonus_flow: 30.0 / 3600.0,
            rl_nat_multiplier: 4.0,
            rl_shadowban_capacity: 5.0,
            rl_shadowban_flow: 5.0 / 3600.0,
            rl_shadowban_ttl: 86400,
            rl_tiered_enabled: true,
            pow_difficulty: 16,
            pow_ttl_secs: 600,
            ollama_url: "http://127.0.0.1:11434".into(),
            ollama_embed_model: "nomic-embed-text".into(),
            ollama_chat_model: "lfm2.5:8b".into(),
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_user: String::new(),
            smtp_pass: String::new(),
            smtp_from: String::new(),
            agent_enabled: false,
            agent_model: "deepseek/deepseek-v4-flash".into(),
            agent_api_key: None,
            agent_base_url: "https://api.commandcode.ai/provider/v1".into(),
            agent_ollama_url: "http://localhost:11434".into(),
            agent_max_runs_per_day: 6,
            agent_cooldown_domain_secs: 3600,
            agent_use_on_fly: false,
            agent_extract_max_snapshot_chars: 120000,
            translation_enabled: false,
            translate_llm_enabled: false,
            translate_model: "lfm2.5:8b".into(),
            translate_chunk_chars: 4000,
            translate_global_budget_per_hour: 1000,
            translate_user_budget_per_hour: 50,
            translate_badge_machine: false,
            translate_auto_approve_machine: false,
            translate_locales: vec!["de".into(), "es".into(), "fr".into(), "pt-BR".into(), "zh".into()],
            translate_reading_locale_ask: true,
            translate_ui_strings: true,
            translate_keep_machine_on_dismiss: false,
            translate_points_approved: 15,
            translate_points_improved: 25,
            translate_points_reviewed: 2,
            translate_points_flag: 1,
            translate_points_daily_cap: 100,
            proposal_quorum_translate: 2,
            proposal_quorum_content_fix: 2,
            proposal_quorum_post_edit: 1,
            proposal_quorum_doc_edit: 1,
            proposal_quorum_work_deletion: 2,
            proposals_per_user_per_day: 10,
            forum_points_per_window: 5,
            forum_window_hours: 72,
            forum_mod_min_level: 5,
            forum_mod_min_exp: 20,
            forum_mod_min_age_days: 14,
            forum_mod_pool_min: 8,
            forum_meta_min_level: 1,
            forum_meta_min_exp: 50,
            forum_meta_min_age_days: 90,
            forum_meta_min_posts: 10,
            forum_meta_ratings: 3,
            forum_meta_pool_min: 8,
            forum_meta_audit_window: 30,
            forum_meta_min_rated: 10,
            forum_meta_unfair_rate: 0.30,
            forum_meta_cooldown_days: 30,
            forum_curator_level: 50,
            forum_admin_level: 100,
            forum_exp_per_level: 100,
            forum_exp_topic_create: 2,
            forum_exp_post_create: 2,
            forum_exp_mod_received: 1,
            forum_exp_mod_daily_cap: 3,
            forum_public_read: true,
            activitypub_enabled: false,
            activitypub_domain: None,
            activitypub_allow_loopback: false,
            trust_preference_similarity_enabled: false,
            trust_similarity_boost: 0,
            trust_recovery_decay: 0.0,
        };
        cfg.agent_enabled = false;
        cfg.agent_base_url = "https://api.commandcode.ai/provider/v1".into();
        cfg.agent_api_key = Some("k".into());
        assert!(!remote_configured(&cfg), "disabled → false");

        cfg.agent_enabled = true;
        cfg.agent_api_key = None;
        assert!(!remote_configured(&cfg), "no key → false");

        cfg.agent_api_key = Some("k".into());
        assert!(remote_configured(&cfg), "all present → true");
    }
}
