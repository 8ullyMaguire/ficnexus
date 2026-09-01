//! Admin commands: status + health.

use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbed;

use crate::commands::{Context, Error};

/// `/status` — bot + FicHub health.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("status"),
    description_localized("en-US", "Bot + FicHub health status")
)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let mut e = CreateEmbed::new()
        .title("fanfic-archivist")
        .color(0x2ecc71)
        .description(format!("Bot version **{}**", crate::VERSION))
        .field("FicHub API", data.config.base_url.as_str(), true)
        .field("Redis", data.config.redis_url.as_str(), true);
    // Try a health probe.
    match data.client.fetch_export("https://archiveofourown.org/works/1").await {
        Ok(_) => {
            e = e.field("API reachable", "✅", true);
        }
        Err(err) => {
            e = e.field("API reachable", format!("⚠️ {err}"), true);
        }
    }
    ctx.send(poise::CreateReply::default().embed(e)).await?;
    Ok(())
}
