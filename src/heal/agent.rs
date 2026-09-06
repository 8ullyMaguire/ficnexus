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
    cfg.agent_enabled && !cfg.agent_base_url.is_empty() && cfg.agent_api_key.is_some()
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
            ChatMessage {
                role: "system",
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user",
                content: user_prompt.to_string(),
            },
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
            LocalMessage {
                role: "system".into(),
                content: system_prompt.to_string(),
            },
            LocalMessage {
                role: "user".into(),
                content: user_prompt.to_string(),
            },
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
    let user_prompt =
        format!("Source URL: {source_url}\n\nHTML snapshot (truncated):\n{truncated}");
    diagnose_remote(http, cfg, EXTRACT_SYSTEM_PROMPT, &user_prompt).await
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
    let user_prompt =
        format!("Source URL: {source_url}\n\nHTML snapshot (truncated):\n{truncated}");
    diagnose_local(http, cfg, EXTRACT_SYSTEM_PROMPT, &user_prompt).await
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
        // Use for_test to get all fields with safe defaults.
        let mut cfg = crate::config::Config::for_test(std::path::PathBuf::from("/tmp/fichub-test"));
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
