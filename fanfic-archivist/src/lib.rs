//! # fanfic-archivist
//!
//! Discord companion bot for FicHub — a self-hosted fanfiction archive
//! (Rust/Axum backend, 107-site scraper parity, pluggable recommendation
//! engine, full-text body search, LLM "Ask the Archive", Fic Requests board,
//! Roadmap consensus).
//!
//! The bot is a thin client over the FicHub REST API. It never touches the
//! FicHub database directly; it talks HTTP to `/api/*` endpoints exactly like
//! the web frontend does, and shares FicHub's Redis for transient pagination
//! cache so `/next` `/prev` don't hammer the DB.
//!
//! ## Command surface
//!
//! *Socrates parity*: `!recs` `/recs`, `!fresh`, `!gems`, `!roll`, `!page X`,
//! `!next`, `!prev`, `!complete`, `!dead`, `!words <min> <max>`, `!fandom`,
//! `!xfandom`, `!xfic`, `!liked`.
//!
//! *FicHub exclusives*: `/ask`, `/quote`, `/download`, `/bookmark`, `/updates`,
//! `/request`, `/roadmap`, `/curator`, `/link`.
//!
//! *UX*: auto-URL detection (ephemeral metadata embed + bookmark button),
//! context menus, filter-chip select menus, ephemeral privacy for library ops.

pub mod api;
pub mod cache;
pub mod commands;
pub mod config;
pub mod error;
pub mod model;
pub mod presenter;
pub mod store;
pub mod util;

pub use config::BotConfig;
pub use error::{BotError, Result};

/// Version of the bot itself.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
