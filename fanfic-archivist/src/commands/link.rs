//! Account linking: `/link` + `/unlink`.
//!
//! Two flows:
//! 1. **API-token flow** (recommended once FicHub P8 #88 "API tokens" ships):
//!    user pastes a token from their profile → stored in Redis, verified via
//!    `/api/auth/me`.
//! 2. **Password flow** (today): user passes username+password;
//!    the bot exchanges them for a JWT + refresh token via `/api/auth/login`.
//!    JWT is short-lived (30d) for security; refresh token is long-lived (365d).
//!    Bot auto-refreshes when JWT expires — no re-linking needed.
//!
//! Tokens stored keyed by platform user id in Redis (`archivist:token:<id>`),
//! TTL 365 days, refreshed on each authed call.

use crate::commands::{Context, Error};
use crate::store::StoredToken;

/// `/link <username> <password>` — connect your FicHub account to Discord.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("link"),
    description_localized("en-US", "Link your FicHub account to Discord"),
    ephemeral
)]
pub async fn link(
    ctx: Context<'_>,
    #[description = "FicHub username"] username: String,
    #[description = "FicHub password (sent once, never stored)"] password: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Exchange credentials for a JWT via the login endpoint.
    let auth = data.client.login(&username, &password).await?;
    let user = auth.user;

    let stored = StoredToken {
        token: auth.token,
        refresh_token: auth.refresh_token,
        fichub_user_id: user.id,
        username: user.username.clone(),
        linked_at: chrono::Utc::now().to_rfc3339(),
    };
    data.tokens
        .set(ctx.author().id.get(), &stored)
        .await?;

    ctx.say(format!(
        "✅ Linked **{}** (FicHub user #{}) to your Discord account.\n\
         Personalized recommendations and library commands are now available.",
        user.username, user.id
    ))
    .await?;
    Ok(())
}

/// `/unlink` — remove the Discord ↔ FicHub link.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("unlink"),
    description_localized("en-US", "Unlink your FicHub account from Discord"),
    ephemeral
)]
pub async fn unlink(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    data.tokens.delete(ctx.author().id.get()).await?;
    ctx.say("Unlinked. Your FicHub account is no longer connected.").await?;
    Ok(())
}

/// `/whoami` — show the linked FicHub user (if any).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("whoami"),
    description_localized("en-US", "Show your linked FicHub account"),
    ephemeral
)]
pub async fn whoami(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    match data
        .tokens
        .get(&ctx.author().id.get().to_string())
        .await?
    {
        Some(stored) => {
            data.tokens.touch(&ctx.author().id.get().to_string()).await?;
            let valid = data.client.me(&stored.token).await;
            match valid {
                Ok(me) => {
                    let name = me
                        .user
                        .as_ref()
                        .map(|u| u.username.clone())
                        .unwrap_or_else(|| stored.username.clone());
                    ctx.say(format!("Linked as **{name}** (FicHub user #{})", stored.fichub_user_id))
                        .await?;
                }
                Err(_) => {
                    // Token expired — try refresh
                    if let Some(refreshed) = data.tokens.refresh(
                        &ctx.author().id.get().to_string(),
                        &data.client
                    ).await? {
                        let name = refreshed.username.clone();
                        ctx.say(format!(
                            "✅ Token refreshed! Linked as **{name}** (FicHub user #{})",
                            refreshed.fichub_user_id
                        ))
                        .await?;
                    } else {
                        ctx.say(format!(
                            "Linked as **{}** but the token expired — run `/link` again.",
                            stored.username
                        ))
                        .await?;
                    }
                }
            }
        }
        None => {
            ctx.say("Not linked. Run `/link` to connect your FicHub account.").await?;
        }
    }
    Ok(())
}

/// Generate a one-time code for web-side account linking
#[poise::command(
    slash_command,
    prefix_command,
    aliases("linkcode"),
    description_localized("en-US", "Get a one-time code to link on the web"),
    ephemeral
)]
pub async fn link_code(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let code = data
        .pending_links
        .create(ctx.author().id.get())
        .await?;
    ctx.say(format!(
        "🔗 One-time link code: **`{}`**\n\
         Enter it at FicHub → Settings → Discord (or run `/link` with your credentials).",
        code
    ))
    .await?;
    Ok(())
}
