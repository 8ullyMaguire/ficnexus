/// Tagging system v3 — user-submitted tags, community voting, auto-moderation.
///
/// Modules:
/// - `resolve`: tag string resolution (canonical lookup, alias fallback, creation)
/// - `voting`: vote recording, score management, auto-moderation
/// - `routes`: API handlers for all tag endpoints
/// - `curator`: curator-only endpoints (alias management, flag resolution)

pub mod backfill;
pub mod resolve;
pub mod voting;
pub mod routes;
pub mod curator;

pub use resolve::TagResolution;
pub use voting::{VoteResult, VoteCounts};
