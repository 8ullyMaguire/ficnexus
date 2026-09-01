//! Notifications: `/notify` — list recent notifications + mark read.
//!
//! Wraps `GET /api/notifications`, `/api/notifications/read-all`, and
//! `/api/notifications/{id}/read`. The FicHub server has no separate
//! fic-update subscription webhook endpoint (the existing notification
//! system covers comments/kudos/follows/etc.), so `/notify` surfaces the
//! user's existing notification feed and mark-read actions.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `/notify` — list recent notifications + mark-all-read.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("notify", "notifications"),
    description_localized("en-US", "List recent FicHub notifications + mark all read"),
    ephemeral
)]
pub async fn notify(
    ctx: Context<'_>,
    #[description = "Number of notifications to show (default 10)"] limit: Option<i64>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let effective = limit.unwrap_or(10).min(50).max(1);
    let resp = data.client.notifications(&token, Some(effective), None).await?;
    let items: Vec<serde_json::Value> = resp
        .get("notifications")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if items.is_empty() {
        ctx.say("No notifications. You're all caught up!").await?;
        return Ok(());
    }

    let mut msg = format!("**{} notification(s)**\n", items.len());
    for n in items.iter().take(10) {
        let title = n.get("title").and_then(|v| v.as_str()).unwrap_or("?");
        let body = n.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let link = n.get("link").and_then(|v| v.as_str());
        let is_read = n.get("is_read").and_then(|v| v.as_bool()).unwrap_or(true);
        let prefix = if is_read { "  " } else { "🔔 " };
        msg.push_str(&format!("{prefix}{title}\n{body}\n"));
        if let Some(l) = link {
            msg.push_str(&format!("  → {l}\n"));
        }
        msg.push_str("\n");
    }

    // Offer mark-all-read action.
    let mark_read = data.client.mark_all_notifications_read(&token).await;
    if let Ok(_) = mark_read {
        msg.push_str("*Marked all as read.*\n");
    }

    ctx.say(msg).await?;
    Ok(())
}

/// `/notify mark-read` — mark all notifications as read.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("mark-read", "markread"),
    description_localized("en-US", "Mark all FicHub notifications as read"),
    ephemeral
)]
pub async fn notify_mark_read(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    match data.client.mark_all_notifications_read(&token).await {
        Ok(_) => {
            ctx.say("All notifications marked as read.").await?;
        }
        Err(e) => {
            ctx.say(format!("Could not mark notifications read: {e}")).await?;
        }
    }
    Ok(())
}
