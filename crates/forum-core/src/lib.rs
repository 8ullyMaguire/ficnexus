//! # forum-core
//!
//! A generic, embeddable forum engine core.
//!
//! This crate implements the *domain model and persistence layer* of a
//! forum-shaped community platform (categories → topics → posts, votes,
//! follows, read-state, bans) without depending on any specific
//! application. It is generic over the *actor id* type (`i32` for FicHub,
//! which uses `users.id`) and treats post/topic `payload` values as opaque
//! JSON — the embedding application decides what they mean
//! (e.g. `{"type":"work","id":…}` link cards).
//!
//! ## Modules
//!
//! - [`model`] — the domain types (`Category`, `Topic`, `Post`, `Vote`,
//!   `Follow`, `ReadState`, `Ban`) plus [`model::Status`] state machine.
//! - [`store`] — the async [`store::Store`] trait and, behind the `postgres`
//!   feature, a sqlx/Postgres implementation (`store::pg::PgStore`).
//! - [`moderation`] — the [`moderation::ModerationHook`] trait and pure
//!   status-transition logic.
//! - [`search`] — a small Postgres full-text-search (tsvector) query
//!   builder, pure string construction with no DB access.
//!
//! ## Features
//!
//! - `postgres` (default): enables the sqlx-based `store::pg::PgStore`. The
//!   pure domain compiles without it, so the crate stays usable in
//!   non-Postgres or dependency-light environments.
//!
//! ## Example
//!
//! ```
//! use forum_core::model::{Category, Topic, Post};
//!
//! let category = Category::<i32> {
//!     id: 1,
//!     slug: "general".into(),
//!     title: "General".into(),
//!     description: String::new(),
//!     position: 0,
//!     is_mod_only: false,
//!     created_at: None,
//!     _actor: std::marker::PhantomData,
//! };
//! let topic = Topic::<i32> {
//!     id: 10,
//!     category_id: category.id,
//!     author_id: 42,
//!     title: "Hello".into(),
//!     body: "World".into(),
//!     payload: serde_json::Value::Null, // opaque — never interpreted by the crate
//!     status: forum_core::model::Status::Open,
//!     view_count: 0,
//!     last_post_id: None,
//!     last_activity_at: None,
//!     created_at: None,
//!     updated_at: None,
//!     deleted_at: None,
//!     is_hidden: false,
//! };
//! let _post = Post::<i32> {
//!     id: 1,
//!     topic_id: topic.id,
//!     author_id: 42,
//!     body: "First!".into(),
//!     payload: serde_json::Value::Null,
//!     quote_of: None,
//!     edited_at: None,
//!     deleted_at: None,
//!     is_hidden: false,
//!     created_at: None,
//! };
//! ```

pub mod model;
pub mod moderation;
pub mod search;
pub mod store;

pub use model::{Ban, Category, Follow, Post, ReadState, Status, Topic, Vote};
pub use store::Store;
