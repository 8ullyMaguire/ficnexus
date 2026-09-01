//! Moderation: the async [`ModerationHook`] trait (app-side wiring) and the
//! pure status-transition state machine (no DB).
//!
//! The hook is the crate's only callback into the embedding application: it
//! is called *before* a topic/post persists (reject → 4xx, nothing
//! written) and *after* a moderation action (record a modlog entry, send
//! notifications, …).
#[allow(unused_imports)]
use crate::model::{Post, Topic};
#[allow(unused_imports)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A single moderation action, handed to [`ModerationHook::on_mod_action`]
/// so the application can record a modlog entry (or anything else).
///
/// The `details` map carries action-specific context (e.g. `reason`,
/// `expires_at` for bans) as opaque JSON — the crate defines the action,
/// the application interprets the details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModAction<A> {
    /// The moderator who performed the action.
    pub actor: A,
    /// One of the `ModActionKind` variants, serialized to its wire name.
    pub kind: ModActionKind,
    /// Target of the action, when it concerns a single topic, post or user.
    pub target: Option<ModTarget<A>>,
    /// Extra opaque context (reason, ban expiry, …).
    pub details: serde_json::Value,
}

/// Which moderation action was performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModActionKind {
    DeleteTopic,
    DeletePost,
    HideTopic,
    HidePost,
    Lock,
    Pin,
    Archive,
    Ban,
    Unban,
}

impl ModActionKind {
    /// Stable wire name used in modlogs / audit trails.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeleteTopic => "forum_delete_topic",
            Self::DeletePost => "forum_delete_post",
            Self::HideTopic => "forum_hide_topic",
            Self::HidePost => "forum_hide_post",
            Self::Lock => "forum_lock",
            Self::Pin => "forum_pin",
            Self::Archive => "forum_archive",
            Self::Ban => "forum_ban",
            Self::Unban => "forum_unban",
        }
    }
}

/// What a moderation action targeted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModTarget<A> {
    Topic(i64),
    Post(i64),
    /// A user-targeting action (e.g. a ban on actor id `A`).
    User(A),
}

/// Why a hook rejected a topic/post before it could persist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModReject {
    pub reason: String,
}

impl ModReject {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

/// Application-side moderation hook.
///
/// The embedding app implements this trait (e.g. shadowban check,
/// profanity/link-spam heuristics, optional LLM triage) and passes it to
/// the store / service layer. Rejections are surfaced as `4xx` to the
/// client *before* anything is written.
#[async_trait]
pub trait ModerationHook<A>: Send + Sync {
    /// Called before a topic or post persists. Returning `Err` rejects the
    /// write; nothing is persisted.
    async fn check_post(
        &self,
        actor: &A,
        body: &str,
    ) -> Result<(), ModReject>;

    /// Called after a moderation action so the app can record a modlog
    /// entry (or trigger notifications).
    async fn on_mod_action(&self, action: ModAction<A>);
}

/// Pure status-transition rules — no DB, unit-testable.
pub mod state {
    use crate::model::Status;

    /// Attempt to transition a topic's status. Returns the new status on
    /// success, or an explanation on failure.
    ///
    /// Allowed transitions:
    /// - `Open` → `Locked`, `Pinned`, `Archived`
    /// - `Pinned` → `Locked`, `Archived`, `Open` (unpin)
    /// - `Locked` → `Open`, `Pinned`, `Archived`
    /// - `Archived` → `Open` (unarchive)
    ///
    /// No transition is allowed *from* `Archived` except back to `Open`.
    /// `Locked` may be unpinned (→ `Open`) without unlocking the topic —
    /// the caller controls whether that is desirable.
    pub fn transition(current: Status, target: Status) -> Result<Status, String> {
        if current == target {
            return Ok(current);
        }
        match (current, target) {
            (Status::Open, Status::Locked | Status::Pinned | Status::Archived) => Ok(target),
            (Status::Pinned, Status::Open | Status::Locked | Status::Archived) => Ok(target),
            (Status::Locked, Status::Open | Status::Pinned | Status::Archived) => Ok(target),
            (Status::Archived, Status::Open) => Ok(target),
            (Status::Archived, _) => Err(format!(
                "archived topics can only be reopened ({} → {})",
                current.as_db(),
                target.as_db()
            )),
            _ => Err(format!(
                "invalid status transition: {} → {}",
                current.as_db(),
                target.as_db()
            )),
        }
    }

    /// Whether a new post may be added to a topic in the given status.
    /// `Locked` and `Archived` reject posts.
    pub fn can_post(status: Status) -> bool {
        status.is_postable()
    }
}

#[cfg(test)]
mod tests {
    use super::state::*;
    use super::*;
    use crate::model::Status;

    #[test]
    fn open_transitions() {
        for target in [Status::Locked, Status::Pinned, Status::Archived] {
            assert_eq!(transition(Status::Open, target).unwrap(), target);
        }
        assert!(transition(Status::Open, Status::Open).is_ok());
    }

    #[test]
    fn pinned_unpin_and_lock() {
        assert_eq!(transition(Status::Pinned, Status::Open).unwrap(), Status::Open);
        assert_eq!(transition(Status::Pinned, Status::Locked).unwrap(), Status::Locked);
    }

    #[test]
    fn locked_can_be_reopened_pinned_archived() {
        assert_eq!(transition(Status::Locked, Status::Open).unwrap(), Status::Open);
        assert_eq!(transition(Status::Locked, Status::Pinned).unwrap(), Status::Pinned);
        assert_eq!(transition(Status::Locked, Status::Archived).unwrap(), Status::Archived);
    }

    #[test]
    fn archived_only_reopens() {
        assert_eq!(transition(Status::Archived, Status::Open).unwrap(), Status::Open);
        // Same-status is a no-op (allowed); only forward transitions from
        // Archived are rejected.
        assert!(transition(Status::Archived, Status::Archived).is_ok());
        for target in [Status::Locked, Status::Pinned] {
            assert!(transition(Status::Archived, target).is_err());
        }
    }

    #[test]
    fn no_posts_into_locked_or_archived() {
        assert!(can_post(Status::Open));
        assert!(can_post(Status::Pinned));
        assert!(!can_post(Status::Locked));
        assert!(!can_post(Status::Archived));
    }

    #[test]
    fn mod_action_wire_names() {
        assert_eq!(ModActionKind::DeleteTopic.as_str(), "forum_delete_topic");
        assert_eq!(ModActionKind::DeletePost.as_str(), "forum_delete_post");
        assert_eq!(ModActionKind::HideTopic.as_str(), "forum_hide_topic");
        assert_eq!(ModActionKind::HidePost.as_str(), "forum_hide_post");
        assert_eq!(ModActionKind::Lock.as_str(), "forum_lock");
        assert_eq!(ModActionKind::Pin.as_str(), "forum_pin");
        assert_eq!(ModActionKind::Archive.as_str(), "forum_archive");
        assert_eq!(ModActionKind::Ban.as_str(), "forum_ban");
        assert_eq!(ModActionKind::Unban.as_str(), "forum_unban");
    }

    #[test]
    fn mod_action_serde_roundtrip() {
        let action: ModAction<i32> = ModAction {
            actor: 5,
            kind: ModActionKind::Lock,
            target: Some(ModTarget::Topic(10)),
            details: serde_json::json!({"reason": "flame war"}),
        };
        let wire = serde_json::to_string(&action).unwrap();
        assert!(wire.contains("\"lock\""));
        assert!(wire.contains("\"topic\""));
        assert!(wire.contains("10"));
        let back: ModAction<i32> = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.kind, ModActionKind::Lock);
        assert_eq!(back.target, Some(ModTarget::Topic(10)));
        assert_eq!(back.details["reason"], serde_json::json!("flame war"));
    }

    #[test]
    fn mod_reject_reason() {
        let r = ModReject::new("shadowbanned");
        assert_eq!(r.reason, "shadowbanned");
    }
}
