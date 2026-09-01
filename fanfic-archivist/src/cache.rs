//! Redis-backed pagination cache for list commands.
//!
//! FicHub shares its Redis instance with the bot. When a user runs `/recs`,
//! `/search`, `/ask`, `/quote`, or `/requests`, the full result list is stored
//! under a short-lived session key; `!next`/`!prev`/`!page X` then read from
//! Redis instead of re-calling the API (no DB hammering).
//!
//! Key layout: `archivist:session:<user_id>:<session_id>` → JSON array of
//! `PageEntry` (the whole list, so pagination is trivial).

use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};

use crate::config::BotConfig;
use crate::error::Result;

/// A single paginated-list entry: the raw item plus a stable ordering index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageEntry {
    /// Stable index into the list (0-based).
    pub index: usize,
    /// The raw JSON object of the item (flexible — each command knows its shape).
    pub item: serde_json::Value,
}

/// A paginated list session stored in Redis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageList {
    /// Session id (uuid) for the list.
    pub session_id: String,
    /// Title of the list ("Recommendations", "Search: drarry", ...).
    pub title: String,
    /// All entries (whole list; may be large but Redis handles it fine).
    pub entries: Vec<PageEntry>,
    /// Optional footer/timestamp.
    pub created_at: String,
}

/// Redis-backed cache for paginated lists.
#[derive(Clone)]
pub struct PageCache {
    /// Connection manager (pooled).
    conn: ConnectionManager,
    /// TTL for session keys.
    ttl_secs: u64,
}

impl PageCache {
    /// Connect to Redis (same URL as FicHub).
    pub async fn connect(config: &BotConfig) -> Result<Self> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self {
            conn,
            ttl_secs: config.link_code_ttl_secs.max(300),
        })
    }

    /// Store a new list, returning its session id.
    pub async fn store(&self, title: &str, entries: Vec<PageEntry>) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let list = PageList {
            session_id: session_id.clone(),
            title: title.to_string(),
            entries,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let key = format!("archivist:session:{session_id}");
        let data = serde_json::to_vec(&list)?;
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.ttl_secs)
            .arg(data)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(session_id)
    }

    /// Store a new list for a user, remembering it as their "latest" session
    /// (so `!next`/`!prev`/`!page X` know which list to paginate).
    pub async fn store_for_user(
        &self,
        user_id: u64,
        title: &str,
        entries: Vec<PageEntry>,
    ) -> Result<String> {
        let session_id = self.store(title, entries).await?;
        let key = format!("archivist:user:{user_id}:latest");
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.ttl_secs)
            .arg(&session_id)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(session_id)
    }

    /// The user's most recent list session id, if any.
    pub async fn latest_for_user(&self, user_id: u64) -> Result<Option<String>> {
        let key = format!("archivist:user:{user_id}:latest");
        let val: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn.clone())
            .await?;
        Ok(val)
    }

    /// Current page for a session (defaults to 1 when unset).
    pub async fn current_page(&self, session_id: &str) -> Result<Option<usize>> {
        let key = format!("archivist:page:{session_id}");
        let val: Option<u64> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn.clone())
            .await?;
        Ok(val.map(|v| v as usize))
    }

    /// Record the current page for a session.
    pub async fn set_current_page(&self, session_id: &str, page: usize) -> Result<()> {
        let key = format!("archivist:page:{session_id}");
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.ttl_secs)
            .arg(page as u64)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(())
    }

    /// Fetch a stored list by session id.
    pub async fn fetch(&self, session_id: &str) -> Result<Option<PageList>> {
        let key = format!("archivist:session:{session_id}");
        let raw: Option<Vec<u8>> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.conn.clone())
            .await?;
        match raw {
            Some(bytes) => {
                let list: PageList = serde_json::from_slice(&bytes)?;
                Ok(Some(list))
            }
            None => Ok(None),
        }
    }

    /// Delete a stored list (call after a session ends).
    pub async fn delete(&self, session_id: &str) -> Result<()> {
        let key = format!("archivist:session:{session_id}");
        redis::cmd("DEL")
            .arg(&key)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(())
    }
}

/// Pagination helper: slice a list for a given page.
pub fn slice_page<T>(items: &[T], page: usize, page_size: usize) -> Vec<&T> {
    let page = page.max(1);
    let start = (page - 1) * page_size;
    if start >= items.len() {
        return Vec::new();
    }
    let end = (start + page_size).min(items.len());
    items[start..end].iter().collect()
}

/// Number of pages for a list.
pub fn page_count(len: usize, page_size: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (len + page_size - 1) / page_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_page_first() {
        let items = vec![1, 2, 3, 4, 5, 6, 7];
        let page = slice_page(&items, 1, 3);
        assert_eq!(page, vec![&1, &2, &3]);
    }

    #[test]
    fn slice_page_middle() {
        let items = vec![1, 2, 3, 4, 5, 6, 7];
        let page = slice_page(&items, 2, 3);
        assert_eq!(page, vec![&4, &5, &6]);
    }

    #[test]
    fn slice_page_last_partial() {
        let items = vec![1, 2, 3, 4, 5, 6, 7];
        let page = slice_page(&items, 3, 3);
        assert_eq!(page, vec![&7]);
    }

    #[test]
    fn slice_page_out_of_range() {
        let items = vec![1, 2, 3, 4, 5, 6, 7];
        let page = slice_page(&items, 99, 3);
        assert!(page.is_empty());
    }

    #[test]
    fn page_count_calc() {
        assert_eq!(page_count(0, 3), 0);
        assert_eq!(page_count(7, 3), 3);
        assert_eq!(page_count(9, 3), 3);
        assert_eq!(page_count(10, 3), 4);
        assert_eq!(page_count(5, 5), 1);
    }
}
