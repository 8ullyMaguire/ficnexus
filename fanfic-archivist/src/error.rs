//! Error types for the bot.

use thiserror::Error;

/// Bot-wide error type.
#[derive(Debug, Error)]
pub enum BotError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API returned error {err}: {msg}")]
    Api { err: i32, msg: String },
    #[error("API returned non-200 status {status}: {body}")]
    Status { status: u16, body: String },
    #[error("JSON parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Not linked: run /link to connect your FicHub account")]
    NotLinked,
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),
    #[error("Command error: {0}")]
    Command(String),
    #[error("Internal error: {0}")]
    Other(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, BotError>;
