//! RSS/Atom update feeds (program item: RSS feeds).
//!
//! Three feeds, all Atom 1.0 (`application/atom+xml; charset=utf-8`):
//!
//! * `GET /feed.xml` — New arrivals: the 20 most recently created works
//!   (ordered by `fic_info.created` DESC, falling back to `fic_updated`
//!   when created is NULL). Public.
//! * `GET /feed/follows.xml?token=<jwt>` — Updates for the works the
//!   authenticated user follows, ordered by `fic_updated` DESC. The token
//!   is a regular FicNexus JWT (the same one the frontend keeps in
//!   `localStorage['fichub_token']` and sends as `Authorization: Bearer`);
//!   it may come from the `token` query param (feed readers can't send
//!   headers) or the Authorization header.
//! * `GET /feed/works/<url_id>.xml` — Update feed for a single fic,
//!   containing just that fic's entry (its `fic_updated` timestamp).
//!
//! Every entry carries an `<updated>` timestamp, an `<id>`, a
//! `<link rel="alternate">` back to the fic page, and per-format download
//! links. All text is XML-escaped.

/// Atom feed content type (RFC 4287).
pub const ATOM_CONTENT_TYPE: &str = "application/atom+xml; charset=utf-8";

/// Maximum entries in a feed.
pub const MAX_ENTRIES: i64 = 20;

/// Shared helpers for building the Atom feed body.
mod build;

/// New-arrivals + follows + per-fic feed handlers.
mod handlers;

pub use build::{FeedQuery, atom_feed, atom_response, entry_for_fic, html_escape, iso_now};
pub use handlers::{follows_feed, new_arrivals_feed, work_feed};
