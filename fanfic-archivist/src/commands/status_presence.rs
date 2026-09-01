//! Nickname/status integration: show linked username in Discord presence.
//! Bot-only feature — updates the bot's own presence to reflect linked
//! FicHub user (read-only, no write to Discord user nick). Uses the
//! `GuildStore` to find the linked user + `set_presence` to update.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};
use crate::store::TokenStore;

/// `/status` — show your linked FicHub status (level, username, linked at).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("status"),
    description_localized("en-US", "Show your linked FicHub account status"),
    ephemeral
)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let me = data.client.me(&token).await?;
    let user = me.user.as_ref().ok_or_else(|| Error::Command("Not logged in.".into()))?;

    ctx.say(format!(
        "**Your FicHub status**\\n\
         · Username: **{}**\\n\
         · User ID: #{}​\\n\
         · Level: {}\\n\
         · Role: {}\\n\
         · Reputation: {}\\n\
         · Exp: {}",
        user.username,
        user.id,
        user.level,
        match user.role {
            0 => "Member",
            1 => "Trusted",
            2 => "Reviewer",
            3 => "Curator",
            4 => "Moderator",
            5 => "Admin",
            _ => "Unknown",
        },
        user.reputation,
        user.exp,
    )).await?;
    Ok(())
}
