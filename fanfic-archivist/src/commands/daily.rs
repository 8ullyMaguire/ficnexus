//! Saved-search alerts: `/daily [N]` — run my saved searches, report new matches.
//!
//! Wraps the 2026-08-25 batch saved-search endpoints:
//!   GET  /api/search/saved            — list my saved searches
//!   POST /api/search/saved/{id}/run  — re-run a saved query
//!   PUT  /api/search/saved/{id}/alert — toggle daily alert (none|rss)

use poise::serenity_prelude as serenity;
use crate::commands::{require_token, Context, Error};

#[poise::command(slash_command, prefix_command, aliases("daily"),
    description_localized("en-US", "Run my saved searches and report new matches"), ephemeral)]
pub async fn daily(ctx: Context<'_>, #[description = "Limit to newest N"] limit: Option<i64>) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    let searches = data.client.list_saved_searches(&token).await?;
    let searches: Vec<serde_json::Value> = searches.get("saved_searches").and_then(|v| v.as_array()).map_or_else(|| vec![], |a| a.to_vec());
    if searches.is_empty() { ctx.say("No saved searches. Create one on FicHub first.").await?; return Ok(()); }
    let limit = limit.unwrap_or(i64::MAX);
    let searches: Vec<_> = searches.into_iter().take(limit as usize).collect();
    let mut msg = format!("**Saved-search run — {} search(s)**\n", searches.len());
    let mut any = false;
    for s in &searches {
        let id = s.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed");
        match data.client.run_saved_search(&token, id).await {
            Ok(resp) => { let total = resp.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
                if total > 0 { any = true;
                    let results = resp.get("results").and_then(|v| v.as_array()).map_or_else(|| vec![], |a| a.to_vec());
                    let top: Vec<_> = results.clone().into_iter().take(3).collect();
                    msg.push_str(&format!("`#{id}` **{}** — {} match(es)\n", name, total));
                    for r in &top { let url_id = r.get("url_id").and_then(|v| v.as_str()).unwrap_or("?"); let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("?"); msg.push_str(&format!("    - /fic/{url_id} **{}**\n", truncate(title, 60))); }
                    if results.len() > 3 { msg.push_str(&format!("    ... plus {} more\n", results.len() - 3)); }
                }
            }
            Err(e) => { msg.push_str(&format!("`#{id}` **{}** — error: {e}\n", name)); }
        }
    }
    if !any { msg.push_str("\nNo new matches."); }
    else { msg.push_str("\n*Tip: toggle daily alerts per search on FicHub to get notified automatically.*"); }
    ctx.say(msg).await?; Ok(())
}
#[poise::command(slash_command, prefix_command, aliases("daily-off"),
    description_localized("en-US", "Turn off daily alerts for a saved search"), ephemeral)]
pub async fn daily_off(ctx: Context<'_>, #[description = "Saved search id"] search_id: i64) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    match data.client.update_saved_search_alert(&token, search_id, "none").await {
        Ok(_) => { ctx.say(format!("Daily alerts turned off for `#{search_id}`.")).await?; }
        Err(e) => { ctx.say(format!("Could not update: {e}")).await?; }
    }
    Ok(())
}
#[poise::command(slash_command, prefix_command, aliases("daily-on"),
    description_localized("en-US", "Turn on daily RSS alerts for a saved search"), ephemeral)]
pub async fn daily_on(ctx: Context<'_>, #[description = "Saved search id"] search_id: i64) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    match data.client.update_saved_search_alert(&token, search_id, "rss").await {
        Ok(_) => { ctx.say(format!("Daily RSS alerts on for `#{search_id}`.")).await?; }
        Err(e) => { ctx.say(format!("Could not update: {e}")).await?; }
    }
    Ok(())
}
fn truncate(s: &str, max: usize) -> String { if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max]) } }
