//! FicNexus's scraping layer — thin re-export of the general-purpose
//! `fanfic-scrapers` crate (AGPL-3.0) plus FicNexus-specific tag scoring.
//!
//! The crate owns the site adapters + metadata model; this module keeps the
//! FicNexus-specific `ExtractedTag` (i16 scores, matching `fic_tags.score`
//! smallint) and FicNexus's `url_id` scheme, and converts between them.

pub mod process;
pub mod quality;
pub mod registry;
pub mod wayback;

pub use fanfic_scrapers::author_link;
pub use fanfic_scrapers::sites;

pub use fanfic_scrapers::{Chapter, FicMetadata, HealHook, NoopHealHook, ScrapeError, SiteScraper};

/// Load site credentials from env vars:
/// `FANFICSCRAPER_<DOMAIN_UPPER_SNAKE>_USER` / `_PASS`
/// (e.g. `FANFICSCRAPER_FANFICTIONHUNT_USER`). Returns the configured set;
/// sites without env creds are simply not logged in (scrapes that need
/// auth fail with `AuthRequired`, which is the documented behavior).
pub fn load_credentials() -> Vec<fanfic_scrapers::SiteCredentials> {
    let mut out = Vec::new();
    for (key, val) in std::env::vars() {
        let Some(rest) = key.strip_prefix("FANFICSCRAPER_") else {
            continue;
        };
        let Some(domain) = rest.strip_suffix("_USER") else {
            continue;
        };
        let domain = domain.to_lowercase().replace('_', ".");
        let pass_key = format!(
            "FANFICSCRAPER_{}_PASS",
            domain.replace('.', "_").to_uppercase()
        );
        if let Ok(password) = std::env::var(&pass_key) {
            out.push(fanfic_scrapers::SiteCredentials {
                domain,
                username: val,
                password,
                is_adult: std::env::var("FANFICSCRAPER_IS_ADULT")
                    .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false),
            });
        }
    }
    out
}

use serde::{Deserialize, Serialize};

/// FicNexus's tag model: i16 scores (DB column `fic_tags.score` smallint).
/// The crate uses f64; convert at the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
    /// Relative importance for "main" semantics: the first character in the
    /// metadata is the MAIN character (score 10), secondary characters 1;
    /// the first ship is the primary pairing (5), others 1; freeforms etc. 0.
    pub score: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 1,
            score: 0,
        }
    }
    pub fn character(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 2,
            score: 1,
        }
    }
    pub fn character_main(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 2,
            score: 10,
        }
    }
    pub fn relationship(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 3,
            score: 1,
        }
    }
    pub fn relationship_primary(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 3,
            score: 5,
        }
    }
    pub fn freeform(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 4,
            score: 0,
        }
    }
    pub fn warning(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 5,
            score: 0,
        }
    }
    pub fn category(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 6,
            score: 0,
        }
    }
}

impl From<fanfic_scrapers::ExtractedTag> for ExtractedTag {
    fn from(t: fanfic_scrapers::ExtractedTag) -> Self {
        Self {
            name: t.name,
            tag_type_id: t.tag_type_id as i16,
            // f64 → i16: round (1.0 stays 1, 0.0 stays 0). The crate's
            // "main" semantics use score 1.0 for main/primary, which maps
            // to FicNexus's 10/5 via the adapters' own constructors — when a
            // crate adapter returns f64, keep it proportional.
            score: (t.score * 10.0).round() as i16,
        }
    }
}

/// FicNexus's url_id scheme: sha256(source:story) → 16 hex chars (8 bytes).
/// (The crate's `generate_url_id` uses `{source}_{local}`; FicNexus kept the
/// hash form for compatibility with existing rows, so we shadow it.)
///
/// Length history: v0 used 12 hex chars (6 bytes = 48-bit) — flagged as
/// collision-risk debt at archive scale. Lengthened to 16 hex chars
/// (64-bit) in the fundamentals-debt pass; existing 12-char ids in the DB
/// remain valid (url_id columns are VARCHAR(128) and cache sharding only
/// needs `[0:2]/[2:4]`).
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_url_id_deterministic() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(1, "story_123");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_url_id_different_source_id() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(2, "story_123");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_url_id_length() {
        let id = generate_url_id(42, "abc123");
        assert_eq!(id.len(), 16);
    }

    #[test]
    fn test_scrape_error_display_not_found() {
        assert_eq!(format!("{}", ScrapeError::NotFound), "fic not found");
    }

    #[test]
    fn test_scrape_error_display_blocked() {
        assert_eq!(format!("{}", ScrapeError::Blocked), "blocked by site");
    }

    #[test]
    fn test_scrape_error_display_network() {
        let err = ScrapeError::Network("connection refused".into());
        assert_eq!(format!("{}", err), "network: connection refused");
    }

    #[test]
    fn test_scrape_error_display_parse_error() {
        let err = ScrapeError::ParseError("unexpected token".into());
        assert_eq!(format!("{}", err), "parse error: unexpected token");
    }

    #[test]
    fn crate_tag_converts_to_fichub_i16() {
        let crate_tag = fanfic_scrapers::ExtractedTag::character_main("Harry");
        let f: ExtractedTag = crate_tag.into();
        assert_eq!(f.score, 10);
        assert_eq!(f.tag_type_id, 2);
    }
}
