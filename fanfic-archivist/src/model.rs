//! Typed response models matching the FicHub REST API.
//!
//! Shapes mirror `frontend/src/lib/api/types.ts` (the frontend is the
//! authoritative contract). Fields are `Option` where the API can omit them.

use serde::{Deserialize, Serialize};

/// A single export's download URLs.
/// Keys: epub, html, txt, md, mobi, pdf, azw3, docx, fb2, kepub.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExportUrls {
    #[serde(default)]
    pub epub: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub txt: Option<String>,
    #[serde(default)]
    pub md: Option<String>,
    #[serde(default)]
    pub mobi: Option<String>,
    #[serde(default)]
    pub pdf: Option<String>,
    #[serde(default)]
    pub azw3: Option<String>,
    #[serde(default)]
    pub docx: Option<String>,
    #[serde(default)]
    pub fb2: Option<String>,
    #[serde(default)]
    pub kepub: Option<String>,
}

impl ExportUrls {
    /// The preferred download URL for a human: EPUB, else first available.
    pub fn preferred(&self) -> Option<(String, String)> {
        if let Some(e) = &self.epub {
            return Some(("epub".to_string(), e.clone()));
        }
        self.first()
    }

    /// First available (format, url) pair.
    pub fn first(&self) -> Option<(String, String)> {
        macro_rules! try_fmt {
            ($name:literal, $field:ident) => {
                if let Some(u) = &self.$field {
                    return Some(($name.to_string(), u.clone()));
                }
            };
        }
        try_fmt!("html", html);
        try_fmt!("txt", txt);
        try_fmt!("md", md);
        try_fmt!("mobi", mobi);
        try_fmt!("pdf", pdf);
        try_fmt!("azw3", azw3);
        try_fmt!("docx", docx);
        try_fmt!("fb2", fb2);
        try_fmt!("kepub", kepub);
        None
    }

    /// List of (format, url) pairs currently available.
    pub fn available(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        macro_rules! push {
            ($name:literal, $field:ident) => {
                if let Some(u) = &self.$field {
                    out.push(($name.to_string(), u.clone()));
                }
            };
        }
        push!("epub", epub);
        push!("html", html);
        push!("txt", txt);
        push!("md", md);
        push!("mobi", mobi);
        push!("pdf", pdf);
        push!("azw3", azw3);
        push!("docx", docx);
        push!("fb2", fb2);
        push!("kepub", kepub);
        out
    }
}

/// Fic metadata object returned by `/api/epub` and `/api/meta`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FicMeta {
    pub id: String,
    #[serde(default)]
    pub work_id: Option<i64>,
    pub title: String,
    pub author: String,
    pub chapters: i64,
    pub words: i64,
    pub description: String,
    pub status: String,
    pub source: String,
    pub created: String,
    pub updated: String,
    #[serde(default)]
    pub extra_meta: Option<serde_json::Value>,
    #[serde(default)]
    pub raw_extended_meta: Option<serde_json::Value>,
    pub author_url: String,
    pub author_local_id: String,
    pub source_id: i64,
    pub author_id: i64,
}

/// Response from `GET /api/epub?q=` and `GET /api/meta?q=`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportResponse {
    pub err: i32,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub url_id: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub meta: Option<FicMeta>,
    #[serde(default)]
    pub hashes: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    pub urls: Option<ExportUrls>,
    #[serde(default)]
    pub epub_url: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub txt_url: Option<String>,
    #[serde(default)]
    pub md_url: Option<String>,
    #[serde(default)]
    pub mobi_url: Option<String>,
    #[serde(default)]
    pub pdf_url: Option<String>,
    #[serde(default)]
    pub azw3_url: Option<String>,
    #[serde(default)]
    pub docx_url: Option<String>,
    #[serde(default)]
    pub fb2_url: Option<String>,
    #[serde(default)]
    pub kepub_url: Option<String>,
    #[serde(default)]
    pub notes: Option<Vec<String>>,
}

impl ExportResponse {
    /// Build a flat ExportUrls from the `*_url` fields (older API shape).
    pub fn flat_urls(&self) -> ExportUrls {
        ExportUrls {
            epub: self.epub_url.clone(),
            html: self.html_url.clone(),
            txt: self.txt_url.clone(),
            md: self.md_url.clone(),
            mobi: self.mobi_url.clone(),
            pdf: self.pdf_url.clone(),
            azw3: self.azw3_url.clone(),
            docx: self.docx_url.clone(),
            fb2: self.fb2_url.clone(),
            kepub: self.kepub_url.clone(),
        }
    }
}

/// Response from `GET /api/epub/convert?format=mobi|pdf|azw3`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertResponse {
    pub err: i32,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub url_id: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub cached: Option<bool>,
    #[serde(default)]
    pub elapsed_ms: Option<i64>,
}

/// A single recommendation result.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RecResult {
    pub url_id: String,
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub words: i64,
    #[serde(default)]
    pub chapters: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub site_domain: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub score: f64,
    #[serde(default)]
    pub community_score: f64,
    #[serde(default)]
    pub download_urls: serde_json::Map<String, serde_json::Value>,
}

/// Response from `GET /api/recommendations?q=&n=`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecommendationsResponse {
    pub err: i32,
    #[serde(default)]
    pub url_id: String,
    #[serde(default)]
    pub site_domain: Option<String>,
    #[serde(default)]
    pub recommendations: Vec<RecResult>,
    #[serde(default)]
    pub generated_at: String,
}

/// Personal recommendations response (`GET /api/recommendations/personal`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PersonalRecommendationsResponse {
    pub err: i32,
    #[serde(default)]
    pub enough_data: bool,
    #[serde(default)]
    pub recs: Vec<RecResult>,
    /// `[{ "title": ..., "url_id": ... }, ...]` — what the recs were based on.
    #[serde(default)]
    pub based_on: Vec<serde_json::Value>,
    /// Pluggable-mode diagnostics (`strategies`, `curator_alpha`, ...).
    #[serde(default)]
    pub strategies: Vec<serde_json::Value>,
}

/// A community suggestion for a fic (`GET /api/recommendations/votes`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Suggestion {
    pub id: i64,
    pub suggested_url_id: String,
    #[serde(default)]
    pub comment: Option<String>,
    pub net_votes: i64,
    #[serde(default)]
    pub created: Option<String>,
}

/// Response from `GET /api/recommendations/votes?url_id=`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VotesResponse {
    pub err: i32,
    pub url_id: String,
    #[serde(default)]
    pub suggestions: Vec<Suggestion>,
}

/// A search result row (`GET /api/search`, `POST /api/search/ask`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub source: String,
    pub words: i64,
    pub chapters: i64,
    pub status: String,
    pub description: String,
    #[serde(default)]
    pub updated: Option<String>,
    #[serde(default)]
    pub rank: Option<f64>,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub tags: Vec<serde_json::Value>,
    #[serde(default)]
    pub total_freeform: usize,
    #[serde(default)]
    pub comment_count: i64,
    #[serde(default)]
    pub kudos_count: i64,
}

/// Search facets (`GET /api/search`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SearchFacets {
    #[serde(default)]
    pub fandoms: Vec<serde_json::Value>,
    #[serde(default)]
    pub tags: Vec<serde_json::Value>,
}

/// Response envelope from `GET /api/search`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResponse {
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
    pub results: Vec<SearchResult>,
    #[serde(default)]
    pub facets: SearchFacets,
}

/// Response from `POST /api/search/ask` — search envelope plus ask metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AskResponse {
    #[serde(flatten)]
    pub search: SearchResponse,
    #[serde(default)]
    pub used_ask: bool,
    #[serde(default)]
    pub ask_query: Option<String>,
    #[serde(default)]
    pub translation: Option<serde_json::Value>,
}

/// Response from `GET /api/search/body?q=`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BodySearchResponse {
    pub err: i32,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub page: usize,
    #[serde(default)]
    pub per_page: usize,
    #[serde(default)]
    pub results: Vec<BodySearchHit>,
}

/// A body-search hit with `<mark>`-highlighted snippet.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BodySearchHit {
    pub url_id: String,
    #[serde(default)]
    pub work_id: Option<i64>,
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub words: i64,
    #[serde(default)]
    pub chapters: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub body_snippet: Option<String>,
}

/// A feed item (`GET /api/v1/feed`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeedItem {
    #[serde(default)]
    pub work_id: i64,
    #[serde(default)]
    pub url_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub updated: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub note: Option<String>,
}

/// Response from `GET /api/v1/feed`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeedResponse {
    pub err: i32,
    #[serde(default)]
    pub items: Vec<FeedItem>,
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub per_page: i64,
    #[serde(default)]
    pub total: i64,
}

/// A Fic Request board item (`GET /api/requests`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestItem {
    pub id: i64,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub seed_work_id: Option<i64>,
    pub status: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub answer_count: i64,
    #[serde(default)]
    pub upvotes: i64,
}

/// Response from `GET /api/requests?status=`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestsResponse {
    pub err: i32,
    #[serde(default)]
    pub items: Vec<RequestItem>,
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub status: String,
}

/// A roadmap consensus cluster row.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClusterRow {
    pub id: i64,
    pub text: String,
    #[serde(default)]
    pub elo_rating: f64,
    #[serde(default)]
    pub matches_played: i64,
    #[serde(default)]
    pub times_picked_best: i64,
    #[serde(default)]
    pub times_picked_worst: i64,
    #[serde(default)]
    pub suggestions: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub controversy: i64,
}

/// Response from `GET /api/roadmap/consensus`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsensusResponse {
    pub err: i32,
    #[serde(default)]
    pub leaderboard: Vec<ClusterRow>,
    #[serde(default)]
    pub controversy: Vec<ClusterRow>,
}

/// Auth response from `POST /api/auth/login`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthResponse {
    pub token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub user: AuthUserModel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthUserModel {
    pub id: i64,
    pub username: String,
    pub role: i16,
    pub reputation: i64,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub level: i16,
    #[serde(default)]
    pub exp: i64,
}

/// Me response from `GET /api/auth/me`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeResponse {
    pub err: i32,
    #[serde(default)]
    pub user: Option<AuthUserModel>,
}

/// Auth response from `POST /api/auth/refresh` — includes a refresh token.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthResponseWithRefresh {
    pub token: String,
    pub refresh_token: String,
    pub user: AuthUserModel,
}

// ── Work detail + stats ──────────────────────────────────────────────
// Models for `GET /api/works/{id}` and `GET /api/works/{id}/stats`.
// These power the `/work` command + kudos toggle + CLI `fichub work <id>`.

/// A single source (fic_info row) for a work, per `GET /api/works/{id}`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WorkSource {
    #[serde(default)]
    pub id: String,
    pub source: String,
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub words: i64,
    #[serde(default)]
    pub chapters: i64,
    pub status: String,
    pub url: String,
}

/// A work record from `GET /api/works/{id}`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Work {
    pub id: i64,
    pub canonical_title: String,
    pub canonical_author: String,
    pub description: String,
    #[serde(default)]
    pub default_source_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub sources: Vec<WorkSource>,
    #[serde(default)]
    pub total_bookmarks: i64,
    #[serde(default)]
    pub total_ratings: i64,
    #[serde(default)]
    pub total_comments: i64,
}

/// Response envelope for `GET /api/works/{id}`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkDetailResponse {
    pub err: i32,
    #[serde(default)]
    pub work: Option<Work>,
}

/// Response from `GET /api/works/{id}/stats`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkStatsResponse {
    pub err: i32,
    pub work_id: i64,
    #[serde(default)]
    pub kudos_count: i64,
    #[serde(default)]
    pub guest_count: i64,
    #[serde(default)]
    pub total_bookmarks: i64,
    #[serde(default)]
    pub total_ratings: i64,
    #[serde(default)]
    pub total_comments: i64,
    #[serde(default)]
    pub total_views: Option<i64>,
    #[serde(default)]
    pub avg_rating: Option<f64>,
}

/// Kudos state from `GET /api/kudos/{work_id}` (after give/remove).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KudosResponse {
    pub err: i32,
    pub work_id: i64,
    #[serde(default)]
    pub kudos_count: i64,
    #[serde(default)]
    pub guest_count: i64,
    #[serde(default)]
    pub total_kudos: i64,
    #[serde(default)]
    pub my_kudos: bool,
}
