pub mod activitypub;
pub mod body_cache;
pub mod cache;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod export;
pub mod fic_suggestions;
pub mod frontend;
pub mod heal;
pub mod ingest;
pub mod leaderboard;
pub mod limiter;
pub mod meta_store;
pub mod modlog;
pub mod realtime;
pub mod recommender;
pub mod roadmap_seed;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod services;
pub mod tags;
pub mod trending;
pub mod user_formats;
pub mod visitor;
pub mod works;

/// Re-export key functions for integration testing.
pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;

/// Matches `@username` tokens (FicNexus usernames: 2-32 alphanumeric/underscore)
/// used by forum mention notifications. regex-lite has no look-around, so the
/// token is matched with a word-boundary + capture group; the caller filters
/// out tokens preceded by another word char.
pub static MENTION_RE: std::sync::LazyLock<regex_lite::Regex> = std::sync::LazyLock::new(|| {
    regex_lite::Regex::new(r"@([A-Za-z0-9_]{2,32})\b").expect("valid mention regex")
});
