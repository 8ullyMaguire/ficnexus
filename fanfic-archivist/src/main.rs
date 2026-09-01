//! fanfic-archivist — Discord companion bot for FicHub.
//!
//! Run with `cargo run --release` (set `DISCORD_TOKEN` env var).

use poise::serenity_prelude as serenity;

use fanfic_archivist::api::FichubClient;
use fanfic_archivist::cache::PageCache;
use fanfic_archivist::commands::{self, Data};
use fanfic_archivist::config::BotConfig;
use fanfic_archivist::store::{PendingLinkStore, TokenStore};
use fanfic_archivist::{BotError, Result};

/// Register all commands with poise.
fn commands() -> Vec<poise::Command<Data, BotError>> {
    vec![
        commands::recs::recs(),
        commands::recs::fresh(),
        commands::recs::gems(),
        commands::recs::roll(),
        commands::recs::page(),
        commands::recs::next(),
        commands::recs::prev(),
        commands::recs::complete(),
        commands::recs::dead(),
        commands::recs::words(),
        commands::recs::fandom(),
        commands::recs::xfandom(),
        commands::recs::xfic(),
        commands::recs::liked(),
        commands::recs::liked_slash(),
        commands::search::search(),
        commands::search::ask(),
        commands::search::quote(),
        commands::search::body(),
        commands::library::download(),
        commands::library::download_direct(),
        commands::library::bookmark(),
        commands::library::rate(),
        commands::library::updates(),
        commands::library::block(),
        commands::library::unblock(),
        commands::library::kudos(),
        commands::link::link(),
        commands::link::unlink(),
        commands::link::whoami(),
        commands::link::link_code(),
        commands::community::request(),
        commands::community::roadmap(),
        commands::work::work(),
        commands::admin::status(),
        commands::events::context_menu(),
    ]
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,fanfic_archivist=debug".into()),
        )
        .init();

    dotenvy::dotenv().ok();
    let config = std::sync::Arc::new(BotConfig::default());
    if config.discord_token.is_empty() {
        eprintln!("DISCORD_TOKEN is required (set it in the environment or .env)");
        std::process::exit(1);
    }

    // Build shared state.
    let client = FichubClient::new(config.clone())?;
    let cache = PageCache::connect(&config).await?;
    let tokens = TokenStore::connect(&config).await?;
    let pending_links = PendingLinkStore::connect(&config).await?;

    let data = Data {
        client,
        cache,
        tokens,
        pending_links,
        config: config.clone(),
        http: reqwest::Client::new(),
    };

    // Poise framework (implements serenity::Framework).
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            let _ = ctx.say(format!("⚠️ {error}")).await;
                        }
                        _ => {}
                    }
                })
            },
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    match event {
                        serenity::FullEvent::Message { new_message } => {
                            commands::events::message_handler(ctx, new_message, data).await?;
                        }
                        serenity::FullEvent::InteractionCreate { interaction } => {
                            if let serenity::Interaction::Component(component) = interaction {
                                commands::events::component_handler(ctx, component, data).await?;
                            }
                        }
                        _ => {}
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // Register slash commands + context menus.
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    // Serenity client with the poise framework.
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::Client::builder(&config.discord_token, intents)
        .framework(framework)
        .await
        .map_err(|e| BotError::Other(format!("failed to create serenity client: {e}")))?;

    client
        .start()
        .await
        .map_err(|e| BotError::Other(format!("client error: {e}")))?;
    Ok(())
}
