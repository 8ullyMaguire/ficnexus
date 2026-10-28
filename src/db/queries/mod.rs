//! Database queries for the export path.
//!
//! Everything here exists because `/api/epub`, `/api/epub/convert`, `/api/meta`
//! or the cache download routes need it:
//!
//! * `core`  — the fic/export ledger (`fic_info`, `export_log`), blacklists,
//!             and the request log that attributes a download to a source;
//! * `tags`  — canonical tag lookup so a scraped work's tags get recorded;
//! * `works` — the works table the scraped sources hang off.
//!
//! The lookup tables those queries read (bookmarks, comments, ratings, forum,
//! xp, rec_*) are still in the schema — nothing was dropped — but no code
//! here reads or writes them. See `docs/EXPORTER-SCOPE.md`.

// ── Submodules ───────────────────────────────────────────────────────────────

pub mod core;
pub mod tags;
pub mod works;

// ── Re-exports (call sites use `crate::db::queries::fn`) ─────────────────────

pub use core::{
    check_author_blacklist, check_fic_blacklist, find_export_log, get_fic_info,
    get_fic_version_bump, insert_export_log, insert_request_log, insert_request_source,
    upsert_fic_info,
};
pub use tags::{create_tag, get_fic_tags, lookup_alias, upsert_fic_tag};
pub use works::{
    create_work, find_work_by_title_author, get_work_by_source, get_work_sources,
    link_source_to_work, log_auto_merge,
};
