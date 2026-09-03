//! Library commands: `/download`, `/download-direct`, `/bookmark`, `/rate`,
//! `/updates`, `/block`.
//!
//! Library mutations run as ephemeral messages (only the invoking user sees
//! them) — Discord privacy for personal actions.

use crate::commands::{get_valid_token, Context, Error};
use crate::util::normalize_url;
use poise::serenity_prelude as serenity;

/// `/download <url>` — get the EPUB (or convert MOBI/PDF/AZW3 on demand) for a fic.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("download"),
    description_localized("en-US", "Get download links for a fic (EPUB/MOBI/PDF/AZW3)")
)]
pub async fn download(
    ctx: Context<'_>,
    #[description = "URL of the fic"] url: String,
    #[description = "Format: epub | mobi | pdf | azw3 (default epub)"] format: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let url = normalize_url(&url);
    let format = format
        .unwrap_or_else(|| "epub".to_string())
        .to_ascii_lowercase();

    // Fast native formats come straight from /api/epub.
    let export = data.client.fetch_export(&url).await?;

    let mut reply = format!("**{}**\nby {} — {} words\n", 
        export.meta.as_ref().map(|m| m.title.as_str()).unwrap_or("Unknown"),
        export.meta.as_ref().map(|m| m.author.as_str()).unwrap_or("Unknown"),
        export.meta.as_ref().map(|m| crate::util::format_words(m.words)).unwrap_or_else(|| "?".into()),
    );

    // Lazy Calibre formats: call /api/epub/convert for mobi/pdf/azw3.
    let lazy_formats = ["mobi", "pdf", "azw3"];
    let direct = export.urls.as_ref().cloned().unwrap_or_else(|| export.flat_urls());
    for (fmt, u) in direct.available() {
        if fmt == format || format == "epub" && fmt == "epub" {
            reply.push_str(&format!("📥 **{fmt}**: {u}\n"));
        }
    }

    if lazy_formats.contains(&format.as_str()) {
        // On-demand conversion.
        match data.client.lazy_convert(&url, &format).await {
            Ok(cv) => {
                if let Some(u) = &cv.url {
                    reply.push_str(&format!("⚡ **{format}**: {u}\n"));
                    if cv.cached == Some(true) {
                        reply.push_str("*(cached — instant)*\n");
                    }
                } else if let Some(msg) = &cv.msg {
                    reply.push_str(&format!("⚠️ Conversion: {msg}\n"));
                }
            }
            Err(e) => {
                reply.push_str(&format!("⚠️ Could not convert to {format}: {e}\n"));
            }
        }
    }

    if !lazy_formats.contains(&format.as_str()) && format != "epub" {
        reply.push_str(&format!("⚠️ Unknown format `{format}`. Try epub | mobi | pdf | azw3.\n"));
    }

    ctx.say(reply).await?;
    Ok(())
}

/// `/download-direct <url>` — upload the cached EPUB as a Discord file.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("dl-direct", "download_direct"),
    description_localized("en-US", "Fetch an EPUB and upload it as a Discord file")
)]
pub async fn download_direct(
    ctx: Context<'_>,
    #[description = "URL of the fic"] url: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let url = normalize_url(&url);

    // Resolve URL → url_id (also warms the cache).
    let export = data.client.fetch_export(&url).await?;
    let url_id = export
        .url_id
        .ok_or_else(|| Error::Command("Could not resolve that URL to a work id.".into()))?;

    // Build the direct cache URL. FicHub serves cached EPUBs at
    // `/cache/epub/<url_id>`, returning the raw binary document.
    let cache_url = data.config.url(&format!("/cache/epub/{url_id}"));

    // Defer so Discord doesn't mark us as failed while we download the file.
    // We keep this public (non-ephemeral) so the attachment is visible.
    ctx.defer().await?;

    // Fetch the EPUB body through the bot's shared reqwest::Client. We don't
    // rely on `Content-Length` alone (some backends omit it); we download the
    // body then enforce the size cap, which also lets us build the in-memory
    // attachment with `CreateAttachment::bytes`.
    let resp = data.http.get(&cache_url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        ctx.say(format!(
            "Could not fetch EPUB for `{}` (HTTP {}: {}).",
            url_id,
            status.as_u16(),
            body.trim().lines().next().unwrap_or("unknown error")
        ))
        .await?;
        return Ok(());
    }

    // Check Content-Length first — if the server advertises a size over the cap
    // we can fail fast without downloading the whole body.
    let max_bytes = data.config.max_upload_bytes;
    if let Some(len) = resp.content_length() {
        if (len as usize) > max_bytes {
            let over_by = (len as usize) - max_bytes;
            ctx.say(format!(
                "📥 EPUB for `{url_id}` is **{:.0} KiB** — over Discord's 25 MiB upload limit by {:.0} KiB.\nUse `/download {url}` for the file link instead.",
                (len as f64) / 1024.0,
                (over_by as f64) / 1024.0
            ))
            .await?;
            return Ok(());
        }
    }

    // Download the bytes and re-check the actual size (Content-Length isn't
    // guaranteed, and we'd rather be safe than send a truncated/massive file).
    let bytes = resp.bytes().await?;
    if bytes.len() > max_bytes {
        ctx.say(format!(
            "📥 EPUB for `{url_id}` is **{} KiB** — over Discord's 25 MiB upload limit.\nUse `/download {url}` for the file link instead.",
            (bytes.len() as f64) / 1024.0
        ))
        .await?;
        return Ok(());
    }

    let title = export
        .meta
        .as_ref()
        .map(|m| m.title.as_str())
        .unwrap_or(&url_id);

    // Upload as a Discord file attachment via poise's CreateReply.
    // `CreateAttachment::bytes` is the non-async constructor — the body is
    // already in memory, so we hand the owned Vec<u8> over directly.
    let attachment = serenity::CreateAttachment::bytes(bytes, format!("{url_id}.epub"));

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "📎 **{}** — downloaded as a file. ({} KiB)",
                title,
                (attachment.data.len() as f64) / 1024.0
            ))
            .attachment(attachment),
    )
    .await?;

    Ok(())
}

/// `/bookmark <url>` — add a fic to your FicHub library (ephemeral).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("bookmark"),
    description_localized("en-US", "Bookmark a fic to your FicHub library"),
    ephemeral
)]
pub async fn bookmark(
    ctx: Context<'_>,
    #[description = "URL of the fic"] url: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };
    let url = normalize_url(&url);

    // Resolve URL → url_id via export (also warms the cache).
    let export = data.client.fetch_export(&url).await?;
    let url_id = export
        .url_id
        .ok_or_else(|| Error::Command("Could not resolve that URL to a work id.".into()))?;

    match data.client.add_bookmark(&token, &url_id).await {
        Ok(v) => {
            let title = export
                .meta
                .as_ref()
                .map(|m| m.title.as_str())
                .unwrap_or(&url_id);
            ctx.say(format!("📌 Bookmarked **{title}** (via FicHub library)."))
                .await?;
            let _ = v;
        }
        Err(e) => {
            ctx.say(format!("Could not bookmark: {e}")).await?;
        }
    }
    Ok(())
}

/// `/rate <url> <stars>` — rate a fic 1-5 stars (feeds the rec engine, ephemeral).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("rate"),
    description_localized("en-US", "Rate a fic 1-5 stars (feeds recommendations)"),
    ephemeral
)]
pub async fn rate(
    ctx: Context<'_>,
    #[description = "URL of the fic"] url: String,
    #[description = "Stars: 1-5"] stars: i32,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };
    let url = normalize_url(&url);
    if !(1..=5).contains(&stars) {
        ctx.say("Stars must be 1-5.").await?;
        return Ok(());
    }
    let export = data.client.fetch_export(&url).await?;
    let url_id = export
        .url_id
        .ok_or_else(|| Error::Command("Could not resolve that URL to a work id.".into()))?;
    match data.client.rate(&token, &url_id, stars).await {
        Ok(_) => {
            ctx.say(format!("⭐ Rated **{}** / 5.", stars)).await?;
        }
        Err(e) => {
            ctx.say(format!("Could not rate: {e}")).await?;
        }
    }
    Ok(())
}

/// `/updates` — new chapters from fics you follow (auth).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("updates"),
    description_localized("en-US", "New chapters from fics you follow"),
    ephemeral
)]
pub async fn updates(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };
    let feed = data.client.feed(&token, 1).await?;
    if feed.err != 0 {
        return Err(Error::Command(format!("Feed error: {}", feed.err)));
    }
    if feed.items.is_empty() {
        ctx.say("No updates yet — follow some fics first.").await?;
        return Ok(());
    }
    let mut lines = String::from("**New chapters:**\n");
    for item in feed.items.iter().take(10) {
        lines.push_str(&format!(
            "- **{}** by {} — {} ({})\n",
            crate::util::truncate(&item.title, 60),
            crate::util::truncate(&item.author, 30),
            item.updated,
            item.format
        ));
    }
    if feed.total > feed.items.len() as i64 {
        lines.push_str(&format!("\n*{} more…*", feed.total - feed.items.len() as i64));
    }
    ctx.say(lines).await?;
    Ok(())
}

/// `/block <url>` — hide a fic from your recommendations (ephemeral).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("block"),
    description_localized("en-US", "Hide a fic from your future recommendations"),
    ephemeral
)]
pub async fn block(
    ctx: Context<'_>,
    #[description = "URL of the fic"] url: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };
    let url = normalize_url(&url);
    let export = data.client.fetch_export(&url).await?;
    let url_id = export
        .url_id
        .ok_or_else(|| Error::Command("Could not resolve that URL to a work id.".into()))?;
    data.client.add_block(&token, &url_id).await?;
    ctx.say(format!("🚫 Hidden `{url_id}` from your recommendations.")).await?;
    Ok(())
}

/// `/unblock <url>` — remove a block (ephemeral).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("unblock"),
    description_localized("en-US", "Unhide a fic from your recommendations"),
    ephemeral
)]
pub async fn unblock(
    ctx: Context<'_>,
    #[description = "URL of the fic"] url: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = match get_valid_token(&ctx).await? {
        Some(t) => t,
        None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
    };
    let url = normalize_url(&url);
    let export = data.client.fetch_export(&url).await?;
    let url_id = export
        .url_id
        .ok_or_else(|| Error::Command("Could not resolve that URL to a work id.".into()))?;
    data.client.remove_block(&token, &url_id).await?;
    ctx.say(format!("🙌 Unhid `{url_id}`.")).await?;
    Ok(())
}

/// `/kudos <id>` — toggle kudos on a fic.
/// `/kudos count <work_id>` — show kudos count (no auth needed).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("kudos"),
    description_localized("en-US", "Toggle or show kudos on a fic")
)]
pub async fn kudos(
    ctx: Context<'_>,
    #[description = "FicHub work ID (numeric)"] work_id: i64,
    #[description = "\"count\" to show kudos without toggling"] action: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();

    // kudos count subcommand: anonymous-safe, no auth required.
    if action.as_deref() == Some("count") {
        let kudos = data.client.kudos(work_id).await?;
        let total = kudos.kudos_count + kudos.guest_count;
        ctx.say(format!(
            "❤️ **#{}** has **{}** kudos ({} guest).",
            work_id, total, kudos.guest_count,
        ))
        .await?;
        return Ok(());
    }

    // Toggle: needs auth.
    let Some(token) = data
        .tokens
        .get(&ctx.author().id.get().to_string())
        .await?
        .map(|s| s.token)
    else {
        ctx.say(crate::presenter::format_not_linked()).await?;
        return Ok(());
    };

    // Fetch work title + current kudos state.
    let work_resp = data.client.work_detail(work_id).await?;
    let Some(work) = work_resp.work else {
        ctx.say(format!("No work found with ID **{work_id}**."))
            .await?;
        return Ok(());
    };

    let current = data.client.kudos(work_id).await?;

    let response = if current.my_kudos {
        data.client.remove_kudos(&token, work_id).await?
    } else {
        data.client.give_kudos(&token, work_id).await?
    };

    let msg = crate::presenter::format_kudos_response(
        &work,
        response.total_kudos,
        !current.my_kudos,
    );
    ctx.say(&msg).await?;
    Ok(())
}
