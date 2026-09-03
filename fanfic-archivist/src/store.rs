//! Redis-backed auth token store.
//!
//! Maps Discord user ids → FicHub JWT tokens. Tokens live in the same Redis
//! instance FicHub uses (`archivist:token:<discord_id>`), so the bot can be
//! restarted without losing linked accounts. TTL is 30 days by default.

use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};

use crate::config::BotConfig;
use crate::error::Result;

/// Default TTL for stored tokens (365 days — refresh token keeps us alive).
pub const TOKEN_TTL_SECS: u64 = 60 * 60 * 24 * 365;

/// A stored token + the user info it maps to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub token: String,
    pub refresh_token: Option<String>,
    pub fichub_user_id: i64,
    pub username: String,
    pub linked_at: String,
}

/// Redis-backed token store.
#[derive(Clone)]
pub struct TokenStore {
    conn: ConnectionManager,
}

impl TokenStore {
    /// Connect to Redis.
    pub async fn connect(config: &BotConfig) -> Result<Self> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    fn key(discord_id: &str) -> String {
        format!("archivist:token:{discord_id}")
    }

    /// Store a token for a Discord user.
    pub async fn set(&self, discord_id: u64, stored: &StoredToken) -> Result<()> {
        let key = Self::key(&discord_id.to_string());
        let data = serde_json::to_vec(stored)?;
        redis::cmd("SETEX")
            .arg(&key)
            .arg(TOKEN_TTL_SECS)
            .arg(data)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(())
    }

    /// Fetch the stored token for a Discord user.
    pub async fn get(&self, discord_id: &str) -> Result<Option<StoredToken>> {
        let key = Self::key(discord_id);
        let raw: Option<Vec<u8>> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn.clone())
            .await?;
        match raw {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Delete a stored token (unlink).
    pub async fn delete(&self, discord_id: u64) -> Result<()> {
        let key = Self::key(&discord_id.to_string());
        redis::cmd("DEL")
            .arg(&key)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(())
    }

    /// Refresh a token's TTL (called on each successful authed call).
    pub async fn touch(&self, discord_id: &str) -> Result<()> {
        let key = Self::key(discord_id);
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(TOKEN_TTL_SECS)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(())
    }

    /// Refresh an expired JWT using the stored refresh token.
    pub async fn refresh(&self, discord_id: &str, client: &crate::api::FichubClient) -> Result<Option<StoredToken>> {
        let key = Self::key(discord_id);
        let raw: Option<Vec<u8>> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn.clone())
            .await?;

        match raw {
            Some(bytes) => {
                let stored: StoredToken = serde_json::from_slice(&bytes)?;
                if let Some(refresh) = &stored.refresh_token {
                    match client.refresh(refresh).await {
                        Ok(auth) => {
                            let new_stored = StoredToken {
                                token: auth.token,
                                refresh_token: Some(auth.refresh_token),
                                fichub_user_id: stored.fichub_user_id,
                                username: stored.username,
                                linked_at: chrono::Utc::now().to_rfc3339(),
                            };
                            // Store new tokens
                            redis::cmd("SETEX")
                                .arg(&key)
                                .arg(TOKEN_TTL_SECS)
                                .arg(serde_json::to_vec(&new_stored)?)
                                .exec_async(&mut self.conn.clone())
                                .await?;
                            Ok(Some(new_stored))
                        }
                        Err(_) => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Prefix for filter-chip preference keys (`archivist:filters:{discord_id}`).
    const PFX_FILTERS: &'static str = "archivist:filters:";

    /// TTL for filter-chip preferences (30 days).
    const PREF_TTL: u64 = 60 * 60 * 24 * 30;

    /// Persist the user's selected filter chips to Redis.
    pub async fn set_filter_chips(
        &self,
        discord_id: &str,
        chips: &[String],
    ) -> Result<()> {
        let data = serde_json::to_vec(chips)?;
        redis::cmd("SETEX")
            .arg(format!("{}{}", Self::PFX_FILTERS, discord_id))
            .arg(Self::PREF_TTL)
            .arg(data)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(())
    }

    /// Retrieve the user's stored filter chips (empty vec if none).
    pub async fn get_filter_chips(&self, discord_id: &str) -> Result<Vec<String>> {
        let raw: Option<Vec<u8>> = redis::cmd("GET")
            .arg(format!("{}{}", Self::PFX_FILTERS, discord_id))
            .query_async(&mut self.conn.clone())
            .await?;
        match raw {
            Some(b) => Ok(serde_json::from_slice(&b)?),
            None => Ok(vec![]),
        }
    }
}

/// Pending `/link` code: Discord id + creation time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingLink {
    pub discord_id: u64,
    pub created_at: String,
}

/// Store for pending one-time link codes (web-side confirmation flow).
#[derive(Clone)]
pub struct PendingLinkStore {
    conn: ConnectionManager,
    ttl_secs: u64,
}

impl PendingLinkStore {
    pub async fn connect(config: &BotConfig) -> Result<Self> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self {
            conn,
            ttl_secs: config.link_code_ttl_secs,
        })
    }

    fn key(code: &str) -> String {
        format!("archivist:link:{code}")
    }

    /// Generate + store a pending link code.
    pub async fn create(&self, discord_id: u64) -> Result<String> {
        let code = Self::gen_code();
        let pending = PendingLink {
            discord_id,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let data = serde_json::to_vec(&pending)?;
        redis::cmd("SETEX")
            .arg(Self::key(&code))
            .arg(self.ttl_secs)
            .arg(data)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(code)
    }

    /// Consume a pending link code (returns the Discord id it belongs to).
    pub async fn consume(&self, code: &str) -> Result<Option<PendingLink>> {
        let key = Self::key(code);
        let raw: Option<Vec<u8>> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn.clone())
            .await?;
        let pending: Option<PendingLink> = match raw {
            Some(bytes) => Some(serde_json::from_slice(&bytes)?),
            None => None,
        };
        // One-time use: delete regardless of whether it existed.
        redis::cmd("DEL")
            .arg(&key)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(pending)
    }

    /// Generate a short alphanumeric code (8 chars, unambiguous alphabet).
    fn gen_code() -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // no I/O/0/1
        let mut out = String::with_capacity(8);
        for _ in 0..8 {
            let idx = (uuid::Uuid::new_v4().as_u128() % ALPHABET.len() as u128) as usize;
            out.push(ALPHABET[idx] as char);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_code_format() {
        let code = PendingLinkStore::gen_code();
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
        // Alphabet excludes ambiguous chars.
        assert!(!code.contains('I'));
        assert!(!code.contains('O'));
        assert!(!code.contains('0'));
        assert!(!code.contains('1'));
    }

    #[test]
    fn stored_token_roundtrip() {
        let st = StoredToken {
            token: "abc".into(),
            refresh_token: Some("refresh_abc".into()),
            fichub_user_id: 42,
            username: "tester".into(),
            linked_at: "now".into(),
        };
        let bytes = serde_json::to_vec(&st).unwrap();
        let back: StoredToken = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back.token, "abc");
        assert_eq!(back.refresh_token, Some("refresh_abc".into()));
        assert_eq!(back.fichub_user_id, 42);
        assert_eq!(back.username, "tester");
    }
}
