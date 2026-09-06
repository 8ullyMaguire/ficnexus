//! Domain model — generic over the actor id type `A`.
//!
//! No application-specific types live here. `A` is the actor (user) id type
//! — `i32` in FicHub (`users.id`), anything serializable otherwise. Topic
//! and post ids are `i64` (BIGSERIAL in the Postgres schema); payloads are
//! opaque [`serde_json::Value`] stored as JSONB.

use serde::{Deserialize, Serialize};

/// Topic / post lifecycle status.
///
/// # Invariants
///
/// - `Open` → `Locked`, `Pinned`, `Archived` are legal transitions.
/// - `Locked` / `Archived` topics accept no new posts.
/// - `Pinned` is a display preference; pinned topics sort first in a
///   category listing. A topic can be pinned *and* locked at the same time
///   (the flags are orthogonal), which is why the status is not a plain
///   enum — see [`Status::is_postable`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Locked,
    Pinned,
    Archived,
}

impl Status {
    /// Parse a status from its wire/database representation
    /// (`"open"`, `"locked"`, `"pinned"`, `"archived"`). Unknown values
    /// map to `None` rather than erroring, so a forward-incompatible
    /// database value degrades gracefully.
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "locked" => Some(Self::Locked),
            "pinned" => Some(Self::Pinned),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    /// The wire representation stored in the `status` column.
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Locked => "locked",
            Self::Pinned => "pinned",
            Self::Archived => "archived",
        }
    }

    /// Whether new posts may be added to a topic in this status.
    ///
    /// `Locked` and `Archived` reject posts; `Open` and `Pinned` accept
    /// them.
    pub fn is_postable(self) -> bool {
        matches!(self, Self::Open | Self::Pinned)
    }
}

/// A curated forum category (admin-created).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Category<A> {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub position: i32,
    /// Announcement-style categories visible to readers but only writable
    /// by moderators.
    pub is_mod_only: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Type parameter is currently unused by categories but kept for a
    /// uniform generic API (categories will carry actor-level metadata in
    /// future versions).
    #[serde(skip)]
    pub _actor: std::marker::PhantomData<A>,
}

/// A forum topic. The first post of a topic is the OP; its `body` is
/// denormalized here for list/search rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Topic<A> {
    pub id: i64,
    pub category_id: i64,
    pub author_id: A,
    pub title: String,
    /// Markdown body of the opening post (stored raw, rendered client-side).
    pub body: String,
    /// Opaque link-card payload (`{"type":"work"|"list"|"request","id":…}`
    /// in FicHub). The crate never interprets it.
    pub payload: serde_json::Value,
    pub status: Status,
    pub view_count: i64,
    /// Id of the latest post in the topic — avoids a join when computing
    /// per-user unread state.
    pub last_post_id: Option<i64>,
    pub last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Soft delete — content (and its replies) stays intact.
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Hidden content is invisible to everyone but moderators.
    pub is_hidden: bool,
}

/// A flat reply within a topic (no nesting; quoting via [`Self::quote_of`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Post<A> {
    pub id: i64,
    pub topic_id: i64,
    pub author_id: A,
    /// Markdown body, stored raw.
    pub body: String,
    /// Opaque link-card payload, stored as JSONB.
    pub payload: serde_json::Value,
    /// Optional quoted post (soft link — `ON DELETE SET NULL`).
    pub quote_of: Option<i64>,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_hidden: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A ±1 vote on a post. Exactly one per (post, user) pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote<A> {
    pub post_id: i64,
    pub user_id: A,
    pub value: i8,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<A> Vote<A> {
    /// Valid vote values are exactly `-1` and `1`.
    pub fn is_valid_value(v: i8) -> bool {
        matches!(v, -1 | 1)
    }
}

/// A follow of a topic by a user (followers receive reply/mention
/// notifications).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Follow<A> {
    pub user_id: A,
    pub topic_id: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Per-user per-topic read state; `last_read_post_id` drives the unread
/// dot/badge (unread when `last_read_post_id < topic.last_post_id`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadState<A> {
    pub user_id: A,
    pub topic_id: i64,
    pub last_read_post_id: Option<i64>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<A> ReadState<A> {
    /// Whether this read state is "behind" the topic's latest post and
    /// should surface as unread.
    pub fn is_unread(&self, last_post_id: Option<i64>) -> bool {
        match (self.last_read_post_id, last_post_id) {
            (_, None) => false, // no posts at all → nothing to read
            (Some(read), Some(last)) => read < last,
            (None, Some(_)) => true, // never read → unread
        }
    }
}

/// A forum ban. `category_id == None` is a global ban; `expires_at == None`
/// is permanent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ban<A> {
    pub id: i64,
    pub user_id: A,
    pub category_id: Option<i64>,
    pub reason: String,
    pub banned_by: Option<A>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<A> Ban<A> {
    /// A ban is *active* at `now` if it never expires or expires later.
    pub fn is_active_at(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        self.expires_at.map_or(true, |exp| exp > now)
    }
}

/// Aggregate vote counts for a post: `up - down`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VoteCounts {
    pub up: i64,
    pub down: i64,
}

impl VoteCounts {
    pub fn score(&self) -> i64 {
        self.up - self.down
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn status_parse_roundtrip() {
        for s in ["open", "locked", "pinned", "archived"] {
            let st = Status::from_db(s).unwrap();
            assert_eq!(st.as_db(), s);
        }
        assert_eq!(Status::from_db("bogus"), None);
        assert_eq!(Status::from_db(""), None);
    }

    #[test]
    fn status_postable_rules() {
        assert!(Status::Open.is_postable());
        assert!(Status::Pinned.is_postable());
        assert!(!Status::Locked.is_postable());
        assert!(!Status::Archived.is_postable());
    }

    #[test]
    fn status_serde_is_lowercase() {
        assert_eq!(
            serde_json::to_string(&Status::Locked).unwrap(),
            "\"locked\""
        );
        let back: Status = serde_json::from_str("\"archived\"").unwrap();
        assert_eq!(back, Status::Archived);
    }

    #[test]
    fn vote_values_are_strict() {
        assert!(Vote::<i32>::is_valid_value(-1));
        assert!(Vote::<i32>::is_valid_value(1));
        assert!(!Vote::<i32>::is_valid_value(0));
        assert!(!Vote::<i32>::is_valid_value(2));
        assert!(!Vote::<i32>::is_valid_value(-2));
    }

    #[test]
    fn payload_is_opaque_and_roundtrips() {
        // Arbitrary, application-specific JSON must survive an opaque
        // round-trip untouched — the crate never interprets it.
        let original = json!({
            "type": "work",
            "id": 1234,
            "meta": {"tags": ["hurt/comfort"], "words": 5000},
            "nested": {"a": [1, 2, {"b": null}]},
        });
        let topic: Topic<i32> = Topic {
            id: 1,
            category_id: 1,
            author_id: 7,
            title: "t".into(),
            body: "b".into(),
            payload: original.clone(),
            status: Status::Open,
            view_count: 0,
            last_post_id: None,
            last_activity_at: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
            is_hidden: false,
        };
        let wire = serde_json::to_string(&topic).unwrap();
        let back: Topic<i32> = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.payload, original);
        assert_eq!(back.payload["nested"]["a"][2]["b"], json!(null));
    }

    #[test]
    fn payload_default_is_null_ok_and_opaque() {
        // Null payloads (no link card) are fine; they must also round-trip.
        let post: Post<i32> = Post {
            id: 1,
            topic_id: 1,
            author_id: 1,
            body: "hi".into(),
            payload: serde_json::Value::Null,
            quote_of: None,
            edited_at: None,
            deleted_at: None,
            is_hidden: false,
            created_at: None,
        };
        let back: Post<i32> = serde_json::from_str(&serde_json::to_string(&post).unwrap()).unwrap();
        assert_eq!(back.payload, serde_json::Value::Null);
    }

    #[test]
    fn read_state_unread_logic() {
        let never: ReadState<i32> = ReadState {
            user_id: 1,
            topic_id: 1,
            last_read_post_id: None,
            updated_at: None,
        };
        assert!(never.is_unread(Some(5)));
        assert!(!never.is_unread(None));

        let read_3: ReadState<i32> = ReadState {
            user_id: 1,
            topic_id: 1,
            last_read_post_id: Some(3),
            updated_at: None,
        };
        assert!(read_3.is_unread(Some(5))); // behind
        assert!(!read_3.is_unread(Some(3))); // caught up
        assert!(!read_3.is_unread(None)); // no posts at all
    }

    #[test]
    fn ban_active_logic() {
        let now = chrono::Utc::now();
        let permanent: Ban<i32> = Ban {
            id: 1,
            user_id: 1,
            category_id: None,
            reason: String::new(),
            banned_by: Some(2),
            expires_at: None,
            created_at: None,
        };
        assert!(permanent.is_active_at(now));

        let future: Ban<i32> = Ban {
            expires_at: Some(now + chrono::Duration::hours(1)),
            ..permanent.clone()
        };
        assert!(future.is_active_at(now));

        let expired: Ban<i32> = Ban {
            expires_at: Some(now - chrono::Duration::minutes(1)),
            ..permanent
        };
        assert!(!expired.is_active_at(now));
    }

    #[test]
    fn vote_counts_score() {
        let counts = VoteCounts { up: 5, down: 2 };
        assert_eq!(counts.score(), 3);
        assert_eq!(VoteCounts::default().score(), 0);
    }

    #[test]
    fn category_serializes_without_phantom() {
        let cat: Category<i32> = Category {
            id: 1,
            slug: "general".into(),
            title: "General".into(),
            description: String::new(),
            position: 0,
            is_mod_only: false,
            created_at: None,
            _actor: std::marker::PhantomData,
        };
        let wire = serde_json::to_string(&cat).unwrap();
        assert!(!wire.contains("_actor"));
        let back: Category<i32> = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.slug, "general");
    }
}
