//! API-token linking: `/link token <token>` — paste a scoped FicHub token
//! from your profile; bot verifies via `/api/auth/me`, stores JWT + level.
//! No password exchange — recommended flow once FicHub P8#88 ships.

use crate::commands::{Context, Error};
use crate::store::StoredToken;

/// `/link token <token>` — link via an API token (no password exchange).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("link"),
    description_localized("en-US", "Link your FicHub account via an API token"),
    ephemeral
)]
pub async fn link_token(
    ctx: Context<'_>,
    #[description = "FicHub API token (from your profile)"] token: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Verify the token is valid and get the user.
    let me = data.client.me(&token).await?;
    let user = me
        .user
        .as_ref()
        .ok_or_else(|| Error::Command("Invalid token — could not resolve user.".into()))?;

    let stored = StoredToken {
        token: token.clone(),
        refresh_token: None, // API tokens don't have refresh flow
        fichub_user_id: user.id,
        username: user.username.clone(),
        level: user.level,
        linked_at: chrono::Utc::now().to_rfc3339(),
    };
    data.tokens
        .set(ctx.author().id.get(), &stored)
        .await?;

    ctx.say(format!(
        "✅ Linked **{}** (FicHub user #{}) to your Discord account via API token.\n\
         Personalized recommendations and library commands are now available.",
        user.username, user.id
    ))
    .await?;
    Ok(())
}
