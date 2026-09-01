//! Bot configuration loaded from environment variables.
//!
//! All values can be overridden with the usual `FANFIC_ARCHIVIST_*` env
//! prefix; the defaults match the FicHub live deployment (thinkcentre).

use std::env;

/// Bot configuration.
///
/// Loaded once at startup from env vars. `base_url` is the FicHub REST API
/// root (e.g. `http://localhost:8000` or `https://fichub.example.com`).
#[derive(Debug, Clone)]
pub struct BotConfig {
    /// FicHub REST API base URL (no trailing slash). Default `http://localhost:8000`.
    pub base_url: String,
    /// FicHub Redis URL for pagination cache. Default `redis://localhost:6379`.
    pub redis_url: String,
    /// Discord bot token (required to run the bot).
    pub discord_token: String,
    /// The `/link` flow token TTL in seconds (how long a pending link code lives).
    pub link_code_ttl_secs: u64,
    /// Max results per page for Discord embeds (avoids exceeding embed field cap).
    pub page_size: usize,
    /// Max results per page from the API (higher than `page_size` so pagination
    /// has buffer when filtering client-side).
    pub api_page_size: usize,
    /// Whether `/download` uploads the EPUB to Discord directly when < 25 MiB.
    pub allow_direct_upload: bool,
    /// Max bytes for direct Discord upload (Discord's limit is 25 MiB).
    pub max_upload_bytes: usize,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            base_url: env::var("FANFIC_ARCHIVIST_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            redis_url: env::var("FANFIC_ARCHIVIST_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            discord_token: env::var("DISCORD_TOKEN").unwrap_or_default(),
            link_code_ttl_secs: env::var("FANFIC_ARCHIVIST_LINK_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            page_size: env::var("FANFIC_ARCHIVIST_PAGE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            api_page_size: env::var("FANFIC_ARCHIVIST_API_PAGE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
            allow_direct_upload: env::var("FANFIC_ARCHIVIST_ALLOW_UPLOAD")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true),
            max_upload_bytes: env::var("FANFIC_ARCHIVIST_MAX_UPLOAD_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(25 * 1024 * 1024),
        }
    }
}

impl BotConfig {
    /// Full URL for an API path (adds the base + leading slash).
    pub fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.base_url.trim_end_matches('/'), path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_joins_base_and_path() {
        let cfg = BotConfig {
            base_url: "http://localhost:8000".into(),
            ..Default::default()
        };
        assert_eq!(cfg.url("/api/epub"), "http://localhost:8000/api/epub");
        assert_eq!(cfg.url("api/meta"), "http://localhost:8000/api/meta");
    }

    #[test]
    fn url_handles_trailing_slash() {
        let cfg = BotConfig {
            base_url: "http://localhost:8000/".into(),
            ..Default::default()
        };
        assert_eq!(cfg.url("api/search"), "http://localhost:8000/api/search");
    }
}
