//! Typed HTTP client for the FicHub REST API.
//!
//! Wraps `reqwest` and every endpoint the bot uses. Never touches the FicHub
//! database directly — this is the shared contract between the bot and the
//! archive (mirrors what the frontend's `api/client.ts` does).
//!
//! Auth: endpoints that need a logged-in user (`/api/recommendations/personal`,
//! `/api/v1/feed`, `/api/bookmarks`, `/api/ratings`, `/api/kudos`, `/api/blocks`,
//! `/api/requests` voting, `/api/roadmap` voting) take an `auth_token` and send
//! it as `Authorization: Bearer <token>`.

use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::de::DeserializeOwned;

use crate::config::BotConfig;
use crate::error::{BotError, Result};
use crate::model::*;

/// A client for the FicHub REST API.
#[derive(Debug, Clone)]
pub struct FichubClient {
    /// Shared reqwest client (cookies + rustls).
    http: reqwest::Client,
    /// Bot config (base URL).
    config: Arc<BotConfig>,
}

impl FichubClient {
    /// Build a new client from a config.
    pub fn new(config: Arc<BotConfig>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent("fanfic-archivist/0.1 (FicHub Discord bot)")
            .build()
            .map_err(BotError::Http)?;
        Ok(Self { http, config })
    }

    /// Build a client with a custom `reqwest::Client` (used in tests with a mock).
    pub fn with_client(config: Arc<BotConfig>, http: reqwest::Client) -> Self {
        Self { http, config }
    }

    /// Low-level GET returning deserialized JSON.
    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        auth_token: Option<&str>,
    ) -> Result<T> {
        let url = self.config.url(path);
        let mut req = self.http.get(&url);
        if let Some(tok) = auth_token {
            req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
        }
        let resp = req.send().await?;
        Self::parse_response(resp).await
    }

    /// Low-level POST with a JSON body returning deserialized JSON.
    async fn post<T: DeserializeOwned, B: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
        auth_token: Option<&str>,
    ) -> Result<T> {
        let url = self.config.url(path);
        let mut req = self.http.post(&url).header(CONTENT_TYPE, "application/json");
        if let Some(tok) = auth_token {
            req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
        }
        let resp = req.json(body).send().await?;
        Self::parse_response(resp).await
    }

    /// Low-level DELETE returning deserialized JSON.
    async fn delete<T: DeserializeOwned>(
        &self,
        path: &str,
        auth_token: Option<&str>,
    ) -> Result<T> {
        let url = self.config.url(path);
        let mut req = self.http.delete(&url);
        if let Some(tok) = auth_token {
            req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
        }
        let resp = req.send().await?;
        Self::parse_response(resp).await
    }

    /// Parse a response into the typed shape, checking status + `err` field.
    async fn parse_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status < 200 || status >= 300 {
            return Err(BotError::Status { status, body });
        }
        // Most FicHub endpoints embed `err` (0 = ok). Deserialize first, then
        // check err field if present.
        let value: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| BotError::Json(e))?;
        if let Some(err) = value.get("err").and_then(|e| e.as_i64()) {
            if err != 0 {
                let msg = value
                    .get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                return Err(BotError::Api {
                    err: err as i32,
                    msg,
                });
            }
        }
        serde_json::from_value(value).map_err(BotError::Json)
    }

    // ── Export / metadata ──────────────────────────────────────────────

    /// `GET /api/epub?q=<url>` — export metadata + download URLs.
    pub async fn fetch_export(&self, url: &str) -> Result<ExportResponse> {
        let q = urlencoding::encode(url);
        self.get(&format!("/api/epub?q={q}"), None).await
    }

    /// `GET /api/epub/convert?q=<url>&format=mobi|pdf|azw3` — lazy conversion.
    pub async fn lazy_convert(&self, url: &str, format: &str) -> Result<ConvertResponse> {
        let q = urlencoding::encode(url);
        self.get(&format!("/api/epub/convert?q={q}&format={format}"), None)
            .await
    }

    /// `GET /api/meta?q=<url>` — metadata only (no download URLs).
    pub async fn fetch_meta(&self, url: &str) -> Result<ExportResponse> {
        let q = urlencoding::encode(url);
        self.get(&format!("/api/meta?q={q}"), None).await
    }

    // ── Recommendations ────────────────────────────────────────────────

    /// `GET /api/recommendations?q=&n=` — similar works to a seed URL.
    pub async fn recommendations(&self, url: &str, n: i32) -> Result<RecommendationsResponse> {
        let q = urlencoding::encode(url);
        self.get(&format!("/api/recommendations?q={q}&n={n}"), None)
            .await
    }

    /// `GET /api/recommendations/personal` — personalized recs (auth).
    pub async fn personal_recommendations(
        &self,
        token: &str,
    ) -> Result<PersonalRecommendationsResponse> {
        self.get("/api/recommendations/personal", Some(token)).await
    }

    /// `GET /api/recommendations/strategies` — list available strategies.
    pub async fn strategies(&self) -> Result<serde_json::Value> {
        self.get("/api/recommendations/strategies", None).await
    }

    // ── Search ─────────────────────────────────────────────────────────

    /// `GET /api/search?q=&page=&per_page=` — advanced search.
    pub async fn search(
        &self,
        params: &SearchParams,
    ) -> Result<SearchResponse> {
        self.get(&params.to_query(), None).await
    }

    /// `POST /api/search/ask` — LLM-assisted natural-language search.
    pub async fn ask(&self, q: &str) -> Result<AskResponse> {
        self.post("/api/search/ask", &serde_json::json!({ "q": q }), None)
            .await
    }

    /// `GET /api/search/body?q=&page=&per_page=` — full-text body search.
    pub async fn body_search(
        &self,
        q: &str,
        page: usize,
        per_page: usize,
    ) -> Result<BodySearchResponse> {
        self.get(&format!("/api/search/body?q={q}&page={page}&per_page={per_page}"), None)
            .await
    }

    // ── Social / library (auth) ────────────────────────────────────────

    /// `GET /api/v1/feed?page=` — new chapters from followed works (auth).
    pub async fn feed(&self, token: &str, page: i64) -> Result<FeedResponse> {
        self.get(&format!("/api/v1/feed?page={page}"), Some(token)).await
    }

    /// `POST /api/bookmarks` — bookmark a fic (auth).
    pub async fn add_bookmark(&self, token: &str, url_id: &str) -> Result<serde_json::Value> {
        self.post(
            "/api/bookmarks",
            &serde_json::json!({ "url_id": url_id }),
            Some(token),
        )
        .await
    }

    /// `DELETE /api/bookmarks/{work_id}` — remove a bookmark (auth).
    pub async fn remove_bookmark(
        &self,
        token: &str,
        work_id: i64,
    ) -> Result<serde_json::Value> {
        self.delete(&format!("/api/bookmarks/{work_id}"), Some(token)).await
    }

    /// `POST /api/ratings` — rate a fic (auth).
    pub async fn rate(&self, token: &str, url_id: &str, stars: i32) -> Result<serde_json::Value> {
        self.post(
            "/api/ratings",
            &serde_json::json!({ "url_id": url_id, "stars": stars }),
            Some(token),
        )
        .await
    }

    /// `POST /api/blocks` — block/hide a fic from recommendations (auth).
    pub async fn add_block(&self, token: &str, url_id: &str) -> Result<serde_json::Value> {
        self.post(
            "/api/blocks",
            &serde_json::json!({ "url_id": url_id }),
            Some(token),
        )
        .await
    }

    /// `DELETE /api/blocks/{url_id}` — unblock a fic (auth).
    pub async fn remove_block(&self, token: &str, url_id: &str) -> Result<serde_json::Value> {
        self.delete(&format!("/api/blocks/{url_id}"), Some(token)).await
    }

    /// `GET /api/kudos/{work_id}` — kudos counts + my_kudos state.
    pub async fn kudos(&self, work_id: i64) -> Result<KudosResponse> {
        self.get(&format!("/api/kudos/{work_id}"), None).await
    }

    /// `POST /api/kudos/{work_id}` — give kudos (auth required).
    pub async fn give_kudos(&self, token: &str, work_id: i64) -> Result<KudosResponse> {
        self.post(&format!("/api/kudos/{work_id}"), &serde_json::json!({}), Some(token)).await
    }

    /// `DELETE /api/kudos/{work_id}` — remove kudos (auth required).
    pub async fn remove_kudos(&self, token: &str, work_id: i64) -> Result<KudosResponse> {
        self.delete(&format!("/api/kudos/{work_id}"), Some(token)).await
    }

    // ── Work detail ─────────────────────────────────────────────────

    /// `GET /api/works/{id}` — work record + sources + aggregate counts.
    pub async fn work_detail(&self, work_id: i64) -> Result<WorkDetailResponse> {
        self.get(&format!("/api/works/{work_id}"), None).await
    }

    /// `GET /api/works/{id}/stats` — per-work engagement stats.
    pub async fn work_stats(&self, work_id: i64) -> Result<WorkStatsResponse> {
        self.get(&format!("/api/works/{work_id}/stats"), None).await
    }

    // ── Community: Fic Requests ────────────────────────────────────────

    /// `GET /api/requests?status=&page=` — list requests.
    pub async fn list_requests(&self, status: &str, page: i64) -> Result<RequestsResponse> {
        self.get(&format!("/api/requests?status={status}&page={page}"), None)
            .await
    }

    /// `POST /api/requests` — create a request (auth).
    pub async fn create_request(
        &self,
        token: &str,
        title: &str,
        body: &str,
    ) -> Result<serde_json::Value> {
        self.post(
            "/api/requests",
            &serde_json::json!({ "title": title, "body": body }),
            Some(token),
        )
        .await
    }

    // ── Community: Roadmap consensus ───────────────────────────────────

    /// `GET /api/roadmap/consensus` — public leaderboard + controversy.
    pub async fn consensus(&self) -> Result<ConsensusResponse> {
        self.get("/api/roadmap/consensus", None).await
    }

    // ── Auth ───────────────────────────────────────────────────────────

    /// `POST /api/auth/login` — exchange username/password for a JWT.
    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse> {
        self.post(
            "/api/auth/login",
            &serde_json::json!({ "username": username, "password": password }),
            None,
        )
        .await
    }

    /// `GET /api/auth/me` — verify a token, get the current user.
    pub async fn me(&self, token: &str) -> Result<MeResponse> {
        self.get("/api/auth/me", Some(token)).await
    }

    /// `POST /api/auth/refresh` — exchange a refresh token for a new JWT + refresh token.
    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthResponseWithRefresh> {
        let url = self.config.url("/api/auth/refresh");
        let resp = self
            .http
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&serde_json::json!({ "refresh_token": refresh_token }))
            .send()
            .await?;
        Self::parse_response(resp).await
    }
}

/// Query parameter builder for advanced search.
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub q: String,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub fandom: Option<String>,
    pub exclude_fandom: Option<String>,
    pub main_char_attr: Option<String>,
    pub min_kudos: Option<i64>,
    pub sort: Option<String>,
    pub include_tags: Option<String>,
    pub exclude_tags: Option<String>,
}

impl SearchParams {
    /// Render as a query string for `GET /api/search`.
    pub fn to_query(&self) -> String {
        let mut qs = Vec::new();
        qs.push(format!("q={}", urlencoding::encode(&self.q)));
        if let Some(p) = self.page {
            qs.push(format!("page={p}"));
        }
        if let Some(p) = self.per_page {
            qs.push(format!("per_page={p}"));
        }
        if let Some(v) = self.min_words {
            qs.push(format!("min_words={v}"));
        }
        if let Some(v) = self.max_words {
            qs.push(format!("max_words={v}"));
        }
        if let Some(v) = self.complete {
            qs.push(format!("complete={v}"));
        }
        if let Some(v) = &self.source {
            qs.push(format!("source={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.fandom {
            qs.push(format!("fandom={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.exclude_fandom {
            qs.push(format!("exclude_fandom={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.main_char_attr {
            qs.push(format!("main_char_attr={}", urlencoding::encode(v)));
        }
        if let Some(v) = self.min_kudos {
            qs.push(format!("min_kudos={v}"));
        }
        if let Some(v) = &self.sort {
            qs.push(format!("sort={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.include_tags {
            qs.push(format!("include_tags={}", urlencoding::encode(v)));
        }
        if let Some(v) = &self.exclude_tags {
            qs.push(format!("exclude_tags={}", urlencoding::encode(v)));
        }
        qs.join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_params_query_builds() {
        let p = SearchParams {
            q: "harry potter".into(),
            min_words: Some(50000),
            complete: Some(true),
            sort: Some("kudos".into()),
            ..Default::default()
        };
        let q = p.to_query();
        assert!(q.contains("q=harry%20potter"));
        assert!(q.contains("min_words=50000"));
        assert!(q.contains("complete=true"));
        assert!(q.contains("sort=kudos"));
    }

    #[test]
    fn search_params_omits_none() {
        let p = SearchParams {
            q: "drarry".into(),
            ..Default::default()
        };
        let q = p.to_query();
        assert_eq!(q, "q=drarry");
        assert!(!q.contains("page="));
        assert!(!q.contains("min_words="));
    }

    #[test]
    fn search_params_urlencodes_values() {
        let p = SearchParams {
            q: "dark harry".into(),
            main_char_attr: Some("Harry Potter|Dark Harry".into()),
            ..Default::default()
        };
        let q = p.to_query();
        assert!(q.contains("main_char_attr=Harry%20Potter%7CDark%20Harry"));
    }
}
