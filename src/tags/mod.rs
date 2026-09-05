/// Tagging system v3 — user-submitted tags, community voting, auto-moderation.
///
/// Modules:
/// - `resolve`: tag string resolution (canonical lookup, alias fallback, creation)
/// - `voting`: vote recording, score management, auto-moderation
/// - `routes`: API handlers for all tag endpoints
/// - `curator`: curator-only endpoints (alias management, flag resolution)
pub mod backfill;
pub mod curator;
pub mod resolve;
pub mod routes;
pub mod voting;

pub use resolve::TagResolution;
pub use voting::{VoteCounts, VoteResult};
