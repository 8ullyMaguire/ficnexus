//! Metadata-pass provenance store.
//!
//! Every scrape/fallback pass writes a versioned JSON file per fic so a
//! future, better scraper can re-merge previously scraped works without
//! re-fetching bodies (e.g. "go backfill tags for everything I imported
//! last month"). This is provenance, NOT queryable state — the DB stays
//! free of it.
//!
//! Layout (two-level sharding on `url_id`, mirrors `body_cache.rs`):
//!   META_DIR/<url_id[0:2]>/<url_id[2:4]>/<url_id>.meta.v{N}.json
//!
//! `META_DIR` defaults to `<body_cache_dir>/../meta` (the attached drive).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::scrape::{ExtractedTag, FicMetadata};

/// Raw FicHub leg (re-exported for serialization). The crate's `FallbackMeta`
/// carries no tags, so tags always ride the Wayback leg.
pub use fanfic_scrapers::fichub_net::FallbackMeta;

/// Provenance for one metadata pass. Append-only: a new pass bumps `v{N}`
/// and never overwrites older versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaBlob {
    pub url_id: String,
    pub source_url: String,
    pub fetched_at_ms: i64,
    /// Native scrape result (None when the native adapter failed).
    pub native: Option<FicMetadata>,
    /// FicHub.net fallback leg (None when not attempted / failed).
    pub fichub: Option<FallbackMeta>,
    /// Wayback Machine fallback leg.
    pub wayback: Option<WaybackLeg>,
    /// The merged metadata the host actually persisted.
    pub merged: Option<FicMetadata>,
    /// Tags after merging (deduped, unioned across legs).
    pub merged_tags: Vec<ExtractedTag>,
    /// True when only one fallback leg succeeded.
    pub partial: bool,
    /// Populated on total failure (both legs failed) — exactly what a
    /// backfill sweep needs to retry.
    pub error: Option<String>,
}

/// The Wayback leg of a metadata pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybackLeg {
    pub meta: Option<FicMetadata>,
    pub tags: Vec<ExtractedTag>,
    pub snapshot_url: Option<String>,
    pub snapshot_at_ms: i64,
}

fn shard_dir(root: &Path, url_id: &str) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    if id.len() >= 4 {
        root.join(&id[..2]).join(&id[2..4])
    } else {
        root.join("_").join("_")
    }
}

fn meta_path(root: &Path, url_id: &str, version: i32) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    shard_dir(root, url_id).join(format!("{id}.meta.v{version}.json"))
}

/// Latest metadata-pass version for a fic (1 if none written yet).
pub fn latest_version(config: &Config, url_id: &str) -> i32 {
    let dir = shard_dir(&config.meta_dir, url_id);
    let id = url_id.to_ascii_lowercase();
    let mut max = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(rest) = name.strip_prefix(&format!("{id}.meta.v")) {
                if let Some(ver) = rest.strip_suffix(".json") {
                    if let Ok(n) = ver.parse::<i32>() {
                        max = max.max(n);
                    }
                }
            }
        }
    }
    max.max(1)
}

/// List all written versions for a fic (ascending). Empty when none.
pub fn list_versions(config: &Config, url_id: &str) -> Vec<i32> {
    let dir = shard_dir(&config.meta_dir, url_id);
    let id = url_id.to_ascii_lowercase();
    let mut vers = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(rest) = name.strip_prefix(&format!("{id}.meta.v")) {
                if let Some(ver) = rest.strip_suffix(".json") {
                    if let Ok(n) = ver.parse::<i32>() {
                        vers.push(n);
                    }
                }
            }
        }
    }
    vers.sort_unstable();
    vers
}

/// Write a metadata pass (atomic tmp+rename), bumping the version.
pub fn write_pass(config: &Config, blob: &MetaBlob) -> std::io::Result<PathBuf> {
    let version = latest_version(config, &blob.url_id);
    let dir = shard_dir(&config.meta_dir, &blob.url_id);
    std::fs::create_dir_all(&dir)?;
    let path = meta_path(&config.meta_dir, &blob.url_id, version);
    let tmp = dir.join(format!(
        "{}.meta.v{}.tmp.{}",
        blob.url_id,
        version,
        std::process::id()
    ));
    let bytes = serde_json::to_vec_pretty(blob).map_err(std::io::Error::other)?;
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}

/// Read the latest metadata pass for a fic. Returns None if not stored.
pub fn latest_meta(config: &Config, url_id: &str) -> Option<MetaBlob> {
    let vers = list_versions(config, url_id);
    for ver in vers.into_iter().rev() {
        if let Some(blob) = read_pass(config, url_id, ver) {
            return Some(blob);
        }
    }
    None
}

/// Read a specific version of a fic's metadata pass.
pub fn read_pass(config: &Config, url_id: &str, version: i32) -> Option<MetaBlob> {
    let path = meta_path(&config.meta_dir, url_id, version);
    if !path.exists() {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use chrono::Utc;

    #[test]
    fn write_then_read_roundtrip() {
        let dir = std::env::temp_dir().join(format!("ficnexus-meta-test-{}", std::process::id()));
        let cfg = Config::for_test(dir.join("meta"));
        let blob = MetaBlob {
            url_id: "3_12345".into(),
            source_url: "https://forum.questionablequesting.com/threads/x.12345/".into(),
            fetched_at_ms: Utc::now().timestamp_millis(),
            native: None,
            fichub: None,
            wayback: None,
            merged: Some(FicMetadata {
                url_id: "3_12345".into(),
                title: "T".into(),
                author: "A".into(),
                chapters: 1,
                words: 100,
                desc: "d".into(),
                published: 0,
                updated: 0,
                status: "ongoing".into(),
                source: "s".into(),
                source_id: 3,
                author_id: 0,
                author_url: String::new(),
                author_local_id: "12345".into(),
                content_hash: None,
                extra_meta: None,
                raw_extended_meta: None,
            }),
            merged_tags: vec![ExtractedTag::freeform("scifi")],
            partial: true,
            error: None,
        };
        write_pass(&cfg, &blob).unwrap();
        let read = latest_meta(&cfg, "3_12345").unwrap();
        assert_eq!(read.url_id, "3_12345");
        assert_eq!(read.merged_tags.len(), 1);
        assert!(read.partial);
        let _ = std::fs::remove_dir_all(&cfg.meta_dir);
    }

    #[test]
    fn latest_version_starts_at_one() {
        let dir = std::env::temp_dir().join(format!("ficnexus-meta-test2-{}", std::process::id()));
        let cfg = Config::for_test(dir.join("meta"));
        assert_eq!(latest_version(&cfg, "3_99999"), 1);
        let _ = std::fs::remove_dir_all(&cfg.meta_dir);
    }

    #[test]
    fn list_versions_empty_when_none_written() {
        let dir = std::env::temp_dir().join(format!("ficnexus-meta-test3-{}", std::process::id()));
        let cfg = Config::for_test(dir.join("meta"));
        assert!(list_versions(&cfg, "3_00000").is_empty());
        let _ = std::fs::remove_dir_all(&cfg.meta_dir);
    }
}
