//! Curator approvals queue: `/curator` — show pending curation items.
//!
//! Wraps `GET /api/curator/approvals` — the unified curator queue (role >= 5).

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, require_level, require_level_for, Context, Error};

/// `/curator` — pending curation items (content fixes, metadata, collection adds).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("curator"),
    description_localized("en-US", "Show pending curation items (curator access required)"),
    ephemeral
)]
pub async fn curator(
    ctx: Context<'_>,
    #[description = "Filter by status: pending | approved | rejected (default pending)"] status: Option<String>,
    #[description = "Filter by type: collection_item_request | curator_fix_proposal | metadata_proposal | comment_triage | forum_edit_proposal"] item_type: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_level(&data.tokens, ctx.author().id.get(), require_level_for("curator", None)).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let status = status.as_ref().map(|s| s.as_str());
    let item_type = item_type.as_ref().map(|s| s.as_str());
    let resp = data.client.curator_approvals(&token, status, item_type, None).await?;

    let items: Vec<serde_json::Value> = resp
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if items.is_empty() {
        ctx.say("No pending curation items.").await?;
        return Ok(());
    }

    let mut msg = format!("**{} curation item(s)**\n", items.len());
    for item in items.iter().take(10) {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("?");
        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("?");
        let title = item.get("work_title")
            .or_else(|| item.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or(item.get("url_id").and_then(|v| v.as_str()).unwrap_or("?"));
        let proposer = item.get("proposer").and_then(|v| v.as_str()).unwrap_or("?");
        let votes_for = item.get("votes_for").and_then(|v| v.as_i64()).unwrap_or(0);
        let votes_against = item.get("votes_against").and_then(|v| v.as_i64()).unwrap_or(0);
        let collection = item.get("collection_title").and_then(|v| v.as_str());
        let collection_id = item.get("collection_id").and_then(|v| v.as_i64());

        let location = match (collection, collection_id) {
            (Some(c), Some(id)) => format!("collection `#{id}` {c}"),
            (None, Some(id)) => format!("collection `#{id}`"),
            _ => format!("work `{title}`"),
        };

        msg.push_str(&format!(
            "  `{item_type}` {status} · {location}\n"
        ));
        msg.push_str(&format!("    proposed by {proposer} · votes: +{votes_for}/-{votes_against}\n\n"));
    }

    ctx.say(msg).await?;
    Ok(())
}
