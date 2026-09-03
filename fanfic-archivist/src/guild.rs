//! Guild-level access control: allowlist/denylist by server id.
//! Bot-only feature — persists guild config in Redis (`archivist:guild:<guild_id>`).
//! No server endpoint needed.

use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};

use crate::config::BotConfig;
use crate::error::BotError;

/// Guild access mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GuildAccess {
    /// No restrictions — any linked user can use the bot.
    Open,
    /// Only users on the allowlist can use the bot (denylist ignored).
    Allowlist { ids: Vec<u64> },
    /// All users can use the bot except those on the denylist.
    Denylist { ids: Vec<u64> },
}

/// Redis-backed guild config store.
#[derive(Clone)]
pub struct GuildStore {
    conn: ConnectionManager,
}

impl GuildStore {
    pub async fn connect(config: &BotConfig) -> Result<Self, BotError> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    fn key(guild_id: u64) -> String {
        format!("archivist:guild:{guild_id}")
    }

    /// Get the access mode for a guild.
    pub async fn get(&self, guild_id: u64) -> Result<GuildAccess, BotError> {
        let raw: Option<Vec<u8>> = redis::cmd("GET")
            .arg(Self::key(guild_id))
            .query_async(&mut self.conn.clone())
            .await
            .map_err(BotError::Redis)?;
        match raw {
            Some(b) => Ok(serde_json::from_slice(&b).map_err(BotError::Json)?),
            None => Ok(GuildAccess::Open),
        }
    }

    /// Set the access mode for a guild.
    pub async fn set(&self, guild_id: u64, mode: &GuildAccess) -> Result<(), BotError> {
        let data = serde_json::to_vec(mode).map_err(BotError::Json)?;
        redis::cmd("SETEX")
            .arg(Self::key(guild_id))
            .arg(60 * 60 * 24 * 30)
            .arg(data)
            .exec_async(&mut self.conn.clone())
            .await
            .map_err(BotError::Redis)?;
        Ok(())
    }

    /// Check if a user is allowed in a guild.
    pub async fn allowed(&self, guild_id: u64, user_id: u64) -> Result<bool, BotError> {
        let mode = self.get(guild_id).await?;
        match mode {
            GuildAccess::Open => Ok(true),
            GuildAccess::Allowlist { ids } => Ok(ids.contains(&user_id)),
            GuildAccess::Denylist { ids } => Ok(!ids.contains(&user_id)),
        }
    }
}
