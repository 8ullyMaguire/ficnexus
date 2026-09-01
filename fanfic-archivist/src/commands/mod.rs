//! Discord command definitions + shared helpers.
//!
//! The bot exposes every command both as a legacy text command (`!recs`, for
//! Socrates parity) and as a slash command (`/recs`), plus context menus and
//! component interactions (pagination buttons, filter select menus, auto-URL
//! ephemeral embeds).

pub mod admin;
pub mod community;
pub mod events;
pub mod library;
pub mod link;
pub mod recs;
pub mod search;
pub mod work;

use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbed;

use crate::api::FichubClient;
use crate::cache::PageCache;
use crate::config::BotConfig;
use crate::error::BotError;
use crate::model::RecResult;
use crate::store::{PendingLinkStore, TokenStore};
use crate::util::{format_words, truncate};

/// Shared state injected into every command.
#[derive(Clone)]
pub struct Data {
    pub client: FichubClient,
    pub cache: PageCache,
    pub tokens: TokenStore,
    pub pending_links: PendingLinkStore,
    pub config: std::sync::Arc<BotConfig>,
    /// Standalone reqwest client for fetching non-JSON payloads (e.g. cached
    /// EPUBs served from `/cache/epub/<url_id>`).
    pub http: reqwest::Client,
}

/// Command error type (converted to a user-facing message by the error handler).
pub type Error = BotError;

/// Poise context alias.
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Build an embed for a single recommendation result.
pub fn rec_embed(r: &RecResult, index: usize, total: usize, strategy: &str) -> CreateEmbed {
    let mut e = CreateEmbed::new()
        .title(format!("{}. {}", index + 1, truncate(&r.title, 200)))
        .url(format!(
            "https://fichub.example.com/fic/{}",
            r.url_id.replace('_', "-")
        ))
        .description(truncate(&r.summary, 500))
        .color(0x9b59b6)
        .field("Author", truncate(&r.author, 80), true)
        .field("Words", format_words(r.words), true)
        .field("Status", &r.status, true);
    if r.chapters > 0 {
        e = e.field("Chapters", r.chapters.to_string(), true);
    }
    if r.community_score > 0.0 {
        e = e.field("Community", format!("{:.2}", r.community_score), true);
    }
    if !r.site_domain.is_empty() {
        e = e.field("Site", truncate(&r.site_domain, 40), true);
    }
    e.footer(serenity::builder::CreateEmbedFooter::new(format!(
        "Recommendation {}/{} · {}",
        index + 1,
        total,
        strategy
    )))
}

/// Build an embed for a single search result.
pub fn search_embed(r: &crate::model::SearchResult, index: usize, total: usize) -> CreateEmbed {
    let mut e = CreateEmbed::new()
        .title(format!("{}. {}", index + 1, truncate(&r.title, 200)))
        .url(format!("https://fichub.example.com/fic/{}", r.url_id))
        .description(truncate(&r.description, 400))
        .color(0x2ecc71)
        .field("Author", truncate(&r.author, 80), true)
        .field("Words", format_words(r.words), true)
        .field("Status", &r.status, true);
    if let Some(snip) = &r.snippet {
        if !snip.is_empty() {
            e = e.field("Snippet", truncate(snip, 200), false);
        }
    }
    if r.kudos_count > 0 {
        e = e.field("Kudos", r.kudos_count.to_string(), true);
    }
    if r.comment_count > 0 {
        e = e.field("Comments", r.comment_count.to_string(), true);
    }
    e.footer(serenity::builder::CreateEmbedFooter::new(format!(
        "Search result {}/{}",
        index + 1,
        total
    )))
}

/// Get the authenticated token for a Discord user id, or `NotLinked` error.
pub async fn require_token(tokens: &TokenStore, discord_id: u64) -> crate::Result<String> {
    tokens
        .get(&discord_id.to_string())
        .await?
        .map(|s| s.token)
        .ok_or(BotError::NotLinked)
}

/// Get a valid token, refreshing if necessary.
///
/// Returns `Some(token)` if the user is linked (and the token is valid or was
/// successfully refreshed), or `None` if the user is not linked.
pub async fn get_valid_token(ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let data = ctx.data();
    let discord_id = ctx.author().id.get().to_string();

    if let Some(stored) = data.tokens.get(&discord_id).await? {
        // Try current token
        if data.client.me(&stored.token).await.is_ok() {
            return Ok(Some(stored.token));
        }

        // Try refresh
        if let Some(refreshed) = data.tokens.refresh(&discord_id, &data.client).await? {
            return Ok(Some(refreshed.token));
        }
    }

    Ok(None)
}
