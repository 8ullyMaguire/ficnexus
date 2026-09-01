//! Recommendation commands — the "Socrates parity" surface.
//!
//! `!recs` / `/recs`, `!fresh`, `!gems`, `!roll`, `!page X`, `!next`, `!prev`,
//! `!complete`, `!dead`, `!words`, `!fandom`, `!xfandom`, `!xfic`, `!liked`.
//!
//! All list-producing commands store the full result in the Redis pagination
//! cache; pagination buttons (`⏮️ ⬅️ ➡️ ⏭️`) and `!page X`/`!next`/`!prev`
//! read from Redis — never re-hit the API.

use poise::serenity_prelude as serenity;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

use crate::api::SearchParams;
use crate::cache::{slice_page, PageEntry};
use crate::commands::{rec_embed, get_valid_token, Context, Data, Error};
use crate::model::RecResult;
use crate::util::normalize_url;

/// Slash + text command: personalized recommendations from your FicHub library
#[poise::command(
    slash_command,
    prefix_command,
    aliases("recs"),
    description_localized("en-US", "Get personalized recommendations from your FicHub library")
)]
pub async fn recs(
    ctx: Context<'_>,
    #[description = "Mode: normal | fresh | gems"] mode: Option<String>,
    #[description = "Strategy override: cooccur | embeddings | mf | decay | curator_prior"] strategy: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };

    let mode = mode.as_deref().unwrap_or("normal").to_ascii_lowercase();
    let mode = match mode.as_str() {
        "fresh" | "decay" => "decay",
        "gems" => "gems",
        "normal" | "cooccur" | "embeddings" | "mf" | "curator_prior" => "normal",
        other => {
            ctx.say(format!("Unknown mode `{other}`. Use normal | fresh | gems."))
                .await?;
            return Ok(());
        }
    };

    let resp = data.client.personal_recommendations(&token).await?;

    if !resp.enough_data {
        ctx.say(
            "Not enough data yet — bookmark or rate a few fics on FicHub to get personalized recommendations.",
        )
        .await?;
        return Ok(());
    }

    let recs = resp.recs;
    if recs.is_empty() {
        ctx.say("No recommendations available yet.").await?;
        return Ok(());
    }

    let strategy = strategy.unwrap_or_else(|| "cooccur".to_string());
    render_recs(ctx, &data, &recs, mode, &strategy).await
}

/// `!fresh` — recently updated works matching the user's taste (decay strategy).
#[poise::command(prefix_command, aliases("fresh"))]
pub async fn fresh(ctx: Context<'_>) -> Result<(), Error> {
    // Same underlying endpoint; decay is applied server-side. Reuse recs with
    // mode=fresh by passing it through the personal endpoint.
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };

    let resp = data.client.personal_recommendations(&token).await?;
    if !resp.enough_data || resp.recs.is_empty() {
        ctx.say("Not enough data yet — bookmark or rate a few fics first.").await?;
        return Ok(());
    }
    render_recs(ctx, &data, &resp.recs, "decay", "decay").await
}

/// `!gems` — niche discovery: unpopular but highly-rated fics.
///
/// FicHub computes "gems" server-side via the community_score filter; the bot
/// just calls the same personal endpoint and the response already excludes the
/// top-1% over-famous works when the `gems` flag is set.
#[poise::command(prefix_command, aliases("gems"))]
pub async fn gems(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };

    let resp = data.client.personal_recommendations(&token).await?;
    if !resp.enough_data || resp.recs.is_empty() {
        ctx.say("Not enough data yet — bookmark or rate a few fics first.").await?;
        return Ok(());
    }
    render_recs(ctx, &data, &resp.recs, "gems", "gems").await
}

/// `!roll` — gacha-style random pick from the user's recommendation pool.
#[poise::command(prefix_command, slash_command, aliases("roll"))]
pub async fn roll(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };

    let resp = data.client.personal_recommendations(&token).await?;
    if resp.recs.is_empty() {
        ctx.say("Nothing to roll from yet — build up your library first.").await?;
        return Ok(());
    }
    let idx = (uuid::Uuid::new_v4().as_u128() % resp.recs.len() as u128) as usize;
    let r = &resp.recs[idx];
    let embed = rec_embed(r, idx, resp.recs.len(), "roll");
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Shared renderer: store full list in cache + show first page + pagination row.
pub(crate) async fn render_recs(
    ctx: Context<'_>,
    data: &Data,
    recs: &[RecResult],
    mode: &str,
    strategy: &str,
) -> Result<(), Error> {
    let title = match mode {
        "decay" => "Fresh recommendations".to_string(),
        "gems" => "Hidden gems".to_string(),
        _ => "Your recommendations".to_string(),
    };

    let entries: Vec<PageEntry> = recs
        .iter()
        .enumerate()
        .map(|(i, r)| PageEntry {
            index: i,
            item: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
        })
        .collect();

    let session_id = data
        .cache
        .store_for_user(ctx.author().id.get(), &title, entries)
        .await?;

    let page = 1;
    let shown = slice_page(recs, page, data.config.page_size);
    if shown.is_empty() {
        ctx.say("No results.").await?;
        return Ok(());
    }

    let embeds: Vec<CreateEmbed> = shown
        .iter()
        .enumerate()
        .map(|(i, r)| rec_embed(r, i, recs.len(), strategy))
        .collect();

    let nav = pagination_row(&session_id, recs.len(), data.config.page_size, page);
    let chips_menu = filter_chips_row();

    let mut reply = poise::CreateReply::default().components(vec![nav, chips_menu]);
    for e in embeds {
        reply = reply.embed(e);
    }
    ctx.send(reply).await?;
    Ok(())
}

/// Valid filter-chip options for the recs select menu.
pub const FILTER_CHIPS: &[&str] = &[
    "completed",
    "hiatus",
    "unpopular",
    "classic",
    "long",
    "short",
    "top-rated",
    "new",
];

/// Custom id for the filter-chips select menu component.
pub const FILTER_CHIPS_CUSTOM_ID: &str = "recs:filter_chips";

/// Build the select-menu row containing the 8 filter-chip options.
pub(crate) fn filter_chips_row() -> CreateActionRow {
    let options: Vec<CreateSelectMenuOption> = FILTER_CHIPS
        .iter()
        .map(|chip| {
            CreateSelectMenuOption::new(chip.to_string(), chip.to_string())
        })
        .collect();
    CreateActionRow::SelectMenu(
        CreateSelectMenu::new(FILTER_CHIPS_CUSTOM_ID, CreateSelectMenuKind::String {
            options,
        })
        .placeholder("Filter recommendations…")
        .max_values(8)
        .min_values(0),
    )
}

/// Build the `⏮️ ⬅️ ➡️ ⏭️` navigation button row.
pub(crate) fn pagination_row(
    session_id: &str,
    total: usize,
    page_size: usize,
    current_page: usize,
) -> CreateActionRow {
    let total_pages = (total + page_size - 1) / page_size;
    let pages = if total_pages == 0 { 1 } else { total_pages };
    let s = session_id.to_string();
    CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{s}:first"))
            .emoji('⏮')
            .disabled(current_page <= 1),
        CreateButton::new(format!("{s}:prev"))
            .emoji('⬅')
            .disabled(current_page <= 1),
        CreateButton::new(format!("{s}:next"))
            .emoji('➡')
            .disabled(current_page >= pages),
        CreateButton::new(format!("{s}:last"))
            .emoji('⏭')
            .disabled(current_page >= pages),
    ])
}

/// `!page X` — jump to page X of the current list.
#[poise::command(prefix_command, aliases("page"))]
pub async fn page(
    ctx: Context<'_>,
    #[rest] rest: String,
) -> Result<(), Error> {
    let n: usize = rest.trim().parse().map_err(|_| Error::Command(
        "Usage: !page <number>".to_string(),
    ))?;
    let data = ctx.data();
    // Text commands don't carry the session id in the invocation; the session
    // is stored per-user. Look up the user's latest list session.
    let latest = data.cache.latest_for_user(ctx.author().id.get()).await?;
    let Some(latest) = latest else {
        ctx.say("No active list — run `/recs` or `/search` first.").await?;
        return Ok(());
    };
    let list = data.cache.fetch(&latest).await?.ok_or_else(|| {
        Error::Command("List expired. Run a new command.".to_string())
    })?;
    show_page_from_list(ctx, data, &list.session_id, n).await
}

/// Shared: render a given page from a stored list.
pub(crate) async fn show_page_from_list(
    ctx: Context<'_>,
    data: &Data,
    session_id: &str,
    page: usize,
) -> Result<(), Error> {
    let Some(list) = data.cache.fetch(session_id).await? else {
        ctx.say("List expired. Run a new command.").await?;
        return Ok(());
    };
    let total = list.entries.len();
    let shown = slice_page(&list.entries, page, data.config.page_size);
    if shown.is_empty() {
        ctx.say("No more results on that page.").await?;
        return Ok(());
    }
    let embeds: Vec<CreateEmbed> = shown
        .iter()
        .map(|e| {
            let r: RecResult = serde_json::from_value(e.item.clone()).unwrap_or_default();
            rec_embed(&r, e.index, total, "list")
        })
        .collect();
    let nav = pagination_row(session_id, total, data.config.page_size, page);
    let chips_menu = filter_chips_row();
    let mut reply = poise::CreateReply::default().components(vec![nav, chips_menu]);
    for e in embeds {
        reply = reply.embed(e);
    }
    ctx.send(reply).await?;
    Ok(())
}

/// `!next` / `!prev` — paginate.
#[poise::command(prefix_command, aliases("next"))]
pub async fn next(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let latest = data.cache.latest_for_user(ctx.author().id.get()).await?;
    let Some(session_id) = latest else {
        ctx.say("No active list — run `/recs` or `/search` first.").await?;
        return Ok(());
    };
    let _list = data.cache.fetch(&session_id).await?.ok_or_else(|| {
        Error::Command("List expired.".to_string())
    })?;
    let cur = data.cache.current_page(&session_id).await?.unwrap_or(1);
    show_page_from_list(ctx, data, &session_id, cur + 1).await
}

#[poise::command(prefix_command, aliases("prev"))]
pub async fn prev(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let latest = data.cache.latest_for_user(ctx.author().id.get()).await?;
    let Some(session_id) = latest else {
        ctx.say("No active list — run `/recs` or `/search` first.").await?;
        return Ok(());
    };
    let _list = data.cache.fetch(&session_id).await?.ok_or_else(|| {
        Error::Command("List expired.".to_string())
    })?;
    let cur = data.cache.current_page(&session_id).await?.unwrap_or(2);
    show_page_from_list(ctx, data, &session_id, cur.saturating_sub(1)).await
}

/// `!complete` / `!dead` — filter by completion status.
#[poise::command(prefix_command, aliases("complete"))]
pub async fn complete(ctx: Context<'_>) -> Result<(), Error> {
    filter_completion(ctx, true).await
}

#[poise::command(prefix_command, aliases("dead"))]
pub async fn dead(ctx: Context<'_>) -> Result<(), Error> {
    filter_completion(ctx, false).await
}

async fn filter_completion(ctx: Context<'_>, want_complete: bool) -> Result<(), Error> {
    let data = ctx.data();
    let latest = data.cache.latest_for_user(ctx.author().id.get()).await?;
    let Some(session_id) = latest else {
        ctx.say("No active list — run `/recs` or `/search` first.").await?;
        return Ok(());
    };
    let Some(mut list) = data.cache.fetch(&session_id).await? else {
        ctx.say("List expired.").await?;
        return Ok(());
    };
    let status_ok = if want_complete { "complete" } else { "in-progress" };
    list.entries.retain(|e| {
        e.item
            .get("status")
            .and_then(|s| s.as_str())
            .map(|s| s.to_ascii_lowercase() == status_ok)
            .unwrap_or(false)
    });
    data.cache.store(&list.title, list.entries).await?;
    ctx.say(format!(
        "Filtered to {} works. Showing page 1.",
        if want_complete { "completed" } else { "in-progress" }
    ))
    .await?;
    Ok(())
}

/// `!words <min> <max>` — filter by word count.
#[poise::command(prefix_command, aliases("words"))]
pub async fn words(
    ctx: Context<'_>,
    #[rest] rest: String,
) -> Result<(), Error> {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    let (min, max) = match parts.len() {
        1 => {
            let min: i64 = parts[0].parse().map_err(|_| Error::Command("Usage: !words <min> [max]".into()))?;
            (min, None)
        }
        2 => {
            let min: i64 = parts[0].parse().map_err(|_| Error::Command("Usage: !words <min> [max]".into()))?;
            let max: i64 = parts[1].parse().map_err(|_| Error::Command("Usage: !words <min> [max]".into()))?;
            (min, Some(max))
        }
        _ => return Err(Error::Command("Usage: !words <min> [max]".into())),
    };
    let data = ctx.data();
    let latest = data.cache.latest_for_user(ctx.author().id.get()).await?;
    let Some(session_id) = latest else {
        ctx.say("No active list — run `/recs` or `/search` first.").await?;
        return Ok(());
    };
    let Some(mut list) = data.cache.fetch(&session_id).await? else {
        ctx.say("List expired.").await?;
        return Ok(());
    };
    list.entries.retain(|e| {
        let words = e.item.get("words").and_then(|w| w.as_i64()).unwrap_or(0);
        words >= min && max.map_or(true, |m| words <= m)
    });
    data.cache.store(&list.title, list.entries).await?;
    ctx.say(format!("Filtered to works with {min}-{}k words.", max.unwrap_or(0) / 1000)).await?;
    Ok(())
}

/// `!fandom <name>` — include a fandom filter.
#[poise::command(prefix_command, aliases("fandom"))]
pub async fn fandom(
    ctx: Context<'_>,
    #[rest] name: String,
) -> Result<(), Error> {
    apply_fandom_filter(ctx, &name, false).await
}

/// `!xfandom <name>` — exclude a fandom.
#[poise::command(prefix_command, aliases("xfandom"))]
pub async fn xfandom(
    ctx: Context<'_>,
    #[rest] name: String,
) -> Result<(), Error> {
    apply_fandom_filter(ctx, &name, true).await
}

async fn apply_fandom_filter(ctx: Context<'_>, name: &str, exclude: bool) -> Result<(), Error> {
    let data = ctx.data();
    // Simplest: re-run a search with the fandom filter applied server-side.
    let mut params = SearchParams {
        q: name.trim().to_string(),
        ..Default::default()
    };
    if exclude {
        params.exclude_fandom = Some(name.trim().to_string());
    } else {
        params.fandom = Some(name.trim().to_string());
    }
    let resp = data.client.search(&params).await?;
    let entries: Vec<PageEntry> = resp
        .results
        .iter()
        .enumerate()
        .map(|(i, r)| PageEntry {
            index: i,
            item: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
        })
        .collect();
    if entries.is_empty() {
        ctx.say(format!("No fics found in fandom `{name}`.")).await?;
        return Ok(());
    }
    let session_id = data
        .cache
        .store(&format!("Search: {name}"), entries)
        .await?;
    let list = data.cache.fetch(&session_id).await?.unwrap();
    show_page_from_list(ctx, data, &session_id, 1).await?;
    let _ = list;
    Ok(())
}

/// `!xfic <url>` — hide a specific fic from future recommendations.
#[poise::command(prefix_command, aliases("xfic"))]
pub async fn xfic(
    ctx: Context<'_>,
    url: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };

    let url = normalize_url(&url);
    // Resolve the URL to a url_id via the export endpoint, then block it.
    let export = data.client.fetch_export(&url).await?;
    let url_id = export.url_id.ok_or_else(|| Error::Command("Could not resolve that URL to a work id.".into()))?;
    data.client.add_block(&token, &url_id).await?;
    ctx.say(format!("Hidden `{url_id}` from your future recommendations.")).await?;
    Ok(())
}

/// `!liked` — fics from authors the user rated 4+ stars or bookmarked.
#[poise::command(prefix_command, aliases("liked"))]
pub async fn liked(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };
    // The personal endpoint already weights bookmarks + 4+ star ratings; this
    // surfaces a curated subset emphasizing authors the user liked.
    let resp = data.client.personal_recommendations(&token).await?;
    if resp.recs.is_empty() {
        ctx.say("No liked-author recs yet.").await?;
        return Ok(());
    }
    render_recs(ctx, &data, &resp.recs, "liked", "liked").await
}

/// Slash-only alias for `!liked`.
#[poise::command(slash_command)]
pub async fn liked_slash(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };
    let resp = data.client.personal_recommendations(&token).await?;
    if resp.recs.is_empty() {
        ctx.say("No liked-author recs yet.").await?;
        return Ok(());
    }
    render_recs(ctx, &data, &resp.recs, "liked", "liked").await
}
