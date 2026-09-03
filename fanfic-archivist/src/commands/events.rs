//! Event handlers: auto-URL detection, context menus, component interactions.
//!
//! * Any message containing a supported fanfiction URL → ephemeral metadata
//!   embed with a Bookmark button (only visible to the author).
//! * User context menu "Get Metadata" on a message with a URL.
//! * Message component handler for pagination buttons + bookmark/convert buttons.

use poise::serenity_prelude as serenity;
use serenity::builder::{CreateButton, CreateEmbed, CreateMessage, CreateActionRow};
use serenity::model::prelude::*;

use crate::commands::{Context, Data, Error};
use crate::util::{extract_fanfic_url, normalize_url, truncate};

/// Handle a new message: detect supported fanfiction URLs.
pub async fn message_handler(
    ctx: &serenity::Context,
    msg: &Message,
    data: &Data,
) -> Result<(), Error> {
    // Only respond to real users.
    if msg.author.bot {
        return Ok(());
    }
    let Some(url) = extract_fanfic_url(&msg.content) else {
        return Ok(());
    };
    let url = normalize_url(&url);

    // Fetch metadata from FicHub.
    let http = data.client.clone();
    let export = match http.fetch_meta(&url).await {
        Ok(e) => e,
        Err(_) => return Ok(()), // silent fail for non-fic URLs
    };
    let Some(meta) = export.meta else {
        return Ok(());
    };

    let mut embed = CreateEmbed::new()
        .title(truncate(&meta.title, 200))
        .url(&url)
        .description(truncate(&meta.description, 500))
        .color(0x9b59b6)
        .field("Author", truncate(&meta.author, 80), true)
        .field("Words", crate::util::format_words(meta.words), true)
        .field("Status", &meta.status, true);
    if meta.chapters > 0 {
        embed = embed.field("Chapters", meta.chapters.to_string(), true);
    }
    let url_id = export.url_id.unwrap_or_default();
    let row = CreateActionRow::Buttons(vec![
        CreateButton::new(format!("bm:{url_id}"))
            .label("Bookmark")
            .emoji('📌')
            .style(serenity::ButtonStyle::Primary),
        CreateButton::new(format!("dl:{url_id}:epub"))
            .label("Download EPUB")
            .emoji('📥')
            .style(serenity::ButtonStyle::Secondary),
    ]);

    let _ = msg
        .channel_id
        .send_message(ctx, CreateMessage::new().embed(embed).components(vec![row]))
        .await;
    Ok(())
}

/// Message context menu: "Get Fic Metadata" on a message with a URL.
#[poise::command(context_menu_command = "Get Fic Metadata")]
pub async fn context_menu(
    ctx: Context<'_>,
    #[description = "The message to inspect"] msg: Message,
) -> Result<(), Error> {
    let url = extract_fanfic_url(&msg.content);
    let Some(url) = url.map(|u| normalize_url(&u)) else {
        ctx.say("No supported fanfiction URL found in that message.")
            .await?;
        return Ok(());
    };
    let data = ctx.data();
    let export = data.client.fetch_meta(&url).await?;
    let meta = export.meta.ok_or_else(|| Error::Command("No metadata.".into()))?;
    let mut embed = CreateEmbed::new()
        .title(truncate(&meta.title, 200))
        .url(&url)
        .description(truncate(&meta.description, 500))
        .color(0x9b59b6)
        .field("Author", truncate(&meta.author, 80), true)
        .field("Words", crate::util::format_words(meta.words), true)
        .field("Status", &meta.status, true);
    if meta.chapters > 0 {
        embed = embed.field("Chapters", meta.chapters.to_string(), true);
    }
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Component interaction handler: pagination buttons, bookmark, download.
pub async fn component_handler(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Pagination: `<session_id>:first|prev|next|last`
    if let Some((session_id, action)) = custom_id.split_once(':') {
        if matches!(action, "first" | "prev" | "next" | "last") {
            let page = match action {
                "first" => 1,
                "prev" => {
                    let cur = data.cache.current_page(session_id).await?.unwrap_or(1);
                    cur.saturating_sub(1).max(1)
                }
                "next" => {
                    let cur = data.cache.current_page(session_id).await?.unwrap_or(1);
                    cur + 1
                }
                "last" => usize::MAX,
                _ => unreachable!(),
            };
            let Some(list) = data.cache.fetch(session_id).await? else {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("List expired — run a new command."),
                        ),
                    )
                    .await?;
                return Ok(());
            };
            data.cache.set_current_page(session_id, page).await?;
            let total = list.entries.len();
            let shown = crate::cache::slice_page(&list.entries, page, data.config.page_size);
            if shown.is_empty() {
                // last: clamp to final page.
                let final_page = crate::cache::page_count(total, data.config.page_size).max(1);
                data.cache.set_current_page(session_id, final_page).await?;
                let shown = crate::cache::slice_page(&list.entries, final_page, data.config.page_size);
                let embeds: Vec<CreateEmbed> = shown
                    .iter()
                    .map(|e| {
                        let r: crate::model::RecResult =
                            serde_json::from_value(e.item.clone()).unwrap_or_default();
                        crate::commands::rec_embed(&r, e.index, total, "list")
                    })
                    .collect();
                let row = crate::commands::recs::pagination_row(
                    session_id,
                    total,
                    data.config.page_size,
                    final_page,
                );
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .embeds(embeds)
                                .components(vec![row]),
                        ),
                    )
                    .await?;
                return Ok(());
            }
            let embeds: Vec<CreateEmbed> = shown
                .iter()
                .map(|e| {
                    let r: crate::model::RecResult =
                        serde_json::from_value(e.item.clone()).unwrap_or_default();
                    crate::commands::rec_embed(&r, e.index, total, "list")
                })
                .collect();
            let row = crate::commands::recs::pagination_row(
                session_id,
                total,
                data.config.page_size,
                page,
            );
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embeds(embeds)
                            .components(vec![row]),
                    ),
                )
                .await?;
            return Ok(());
        }
    }

    // Filter-chips select menu: `recs:filter_chips`
    if custom_id == crate::commands::recs::FILTER_CHIPS_CUSTOM_ID {
        let selected: Vec<String> = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values.clone(),
            _ => {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("Unexpected interaction type.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                return Ok(());
            }
        };

        let discord_id = interaction.user.id.get();

        // Validate against the canonical chip set.
        let valid_chips = crate::commands::recs::FILTER_CHIPS;
        let applied: Vec<String> = selected
            .iter()
            .filter(|s| valid_chips.iter().any(|v| &s.to_lowercase() == v))
            .cloned()
            .collect();

        data.tokens
            .set_filter_chips(&discord_id.to_string(), &applied)
            .await?;

        let label: String = if applied.is_empty() {
            "Filter chips cleared (default).".to_string()
        } else {
            format!("Filter chips set to: {}", applied.join(", "))
        };

        interaction
            .create_response(
                ctx,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(label)
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Bookmark button: `bm:<url_id>`
    if let Some(url_id) = custom_id.strip_prefix("bm:") {
        let token = match data.tokens.get(&interaction.user.id.get().to_string()).await? {
            Some(t) => t,
            None => {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("You're not linked — run `/link` first.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                return Ok(());
            }
        };
        data.tokens.touch(&interaction.user.id.get().to_string()).await?;
        match data.client.add_bookmark(&token.token, url_id).await {
            Ok(_) => {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(format!("📌 Bookmarked `{url_id}`."))
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
            Err(e) => {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(format!("Could not bookmark: {e}"))
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }
        return Ok(());
    }

    // Download button: `dl:<url_id>:<format>`
    if let Some(rest) = custom_id.strip_prefix("dl:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 2 {
            let (url_id, format) = (parts[0], parts[1]);
            let export = data.client.fetch_export(&format!("https://fichub.example.com/fic/{url_id}")).await;
            // url_id → canonical URL not directly resolvable; use /api/epub?q=<url_id-as-id>.
            // FicHub export endpoint takes a URL; for ids we can try meta by id via the
            // `q=<url_id>` form which the backend accepts for known works.
            match export {
                Ok(ex) => {
                    let urls = ex.urls.unwrap_or_default();
                    let target = if format == "epub" {
                        urls.epub.clone().or_else(|| urls.first().map(|(_, u)| u))
                    } else {
                        urls.available().iter().find(|(f, _)| f == format).map(|(_, u)| u.clone())
                    };
                    match target {
                        Some(u) => {
                            interaction
                                .create_response(
                                    ctx,
                                    serenity::CreateInteractionResponse::Message(
                                        serenity::CreateInteractionResponseMessage::new()
                                            .content(format!("📥 **{format}**: {u}")),
                                    ),
                                )
                                .await?;
                        }
                        None => {
                            // Try lazy convert for non-native formats.
                            if matches!(format, "mobi" | "pdf" | "azw3") {
                                match data.client.lazy_convert(&format!("https://fichub.example.com/fic/{url_id}"), format).await {
                                    Ok(cv) => {
                                        let msg = cv.url.as_ref().map(|u| format!("⚡ **{format}**: {u}"))
                                            .unwrap_or_else(|| format!("No {format} available."));
                                        interaction
                                            .create_response(
                                                ctx,
                                                serenity::CreateInteractionResponse::Message(
                                                    serenity::CreateInteractionResponseMessage::new().content(msg),
                                                ),
                                            )
                                            .await?;
                                    }
                                    Err(e) => {
                                        interaction
                                            .create_response(
                                                ctx,
                                                serenity::CreateInteractionResponse::Message(
                                                    serenity::CreateInteractionResponseMessage::new()
                                                        .content(format!("Conversion failed: {e}")),
                                                ),
                                            )
                                            .await?;
                                    }
                                }
                            } else {
                                interaction
                                    .create_response(
                                        ctx,
                                        serenity::CreateInteractionResponse::Message(
                                            serenity::CreateInteractionResponseMessage::new()
                                                .content(format!("No {format} available yet.")),
                                        ),
                                    )
                                    .await?;
                            }
                        }
                    }
                }
                Err(_) => {
                    interaction
                        .create_response(
                            ctx,
                            serenity::CreateInteractionResponse::Message(
                                serenity::CreateInteractionResponseMessage::new()
                                    .content("Could not fetch that work."),
                            ),
                        )
                        .await?;
                }
            }
            return Ok(());
        }
    }

    Ok(())
}
