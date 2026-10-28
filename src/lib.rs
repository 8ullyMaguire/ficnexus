pub mod body_cache;
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod heal;
pub mod limiter;
pub mod routes;
pub mod scrape;
pub mod server;
pub mod services;
pub mod tags;
pub mod user_formats;
pub mod works;

/// Re-export key functions for integration testing.
pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
