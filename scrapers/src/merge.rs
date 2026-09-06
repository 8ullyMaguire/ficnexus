//! Fallback-leg merging.
//!
//! When a native scrape fails, the host may gather metadata from several
//! fallback legs (Wayback, FicHub). This module merges two legs into one
//! `FicMetadata`: core fields come from the leg with the newer `updated`
//! (freshest wins; ties → the primary/first leg), everything else unions,
//! and tags dedupe by lowercase name. Pure functions — fully unit-testable.

use crate::{ExtractedTag, FicMetadata};

/// Merge two metadata legs. `primary` is preferred on ties (normally the
/// native scrape; if only fallbacks exist, whichever the host trusts more).
/// Returns the merged metadata plus the unioned, deduped tags.
///
/// Core fields (title, author, chapters, words, status, description) come
/// from the leg whose `updated` is newer. Everything else unions, preferring
/// a non-empty value. Tags from both legs are unioned and deduped.
pub fn merge_metadata(
    primary: &FicMetadata,
    primary_tags: &[ExtractedTag],
    secondary: &FicMetadata,
    secondary_tags: &[ExtractedTag],
) -> (FicMetadata, Vec<ExtractedTag>) {
    // Freshest leg wins core fields. Tie → primary.
    let use_secondary = secondary.updated > primary.updated;

    let mut merged = primary.clone();
    if use_secondary {
        merged.title = secondary.title.clone();
        merged.author = secondary.author.clone();
        merged.chapters = secondary.chapters;
        merged.words = secondary.words;
        merged.status = secondary.status.clone();
        merged.desc = secondary.desc.clone();
        merged.updated = secondary.updated;
        merged.published = secondary.published;
    }

    // Union the rest: prefer any non-empty value primary is missing.
    if merged.author_url.is_empty() {
        merged.author_url = secondary.author_url.clone();
    }
    if merged.author_id == 0 {
        merged.author_id = secondary.author_id;
    }
    if merged.author_local_id.is_empty() {
        merged.author_local_id = secondary.author_local_id.clone();
    }
    if merged.source.is_empty() {
        merged.source = secondary.source.clone();
    }
    if merged.content_hash.is_none() {
        merged.content_hash = secondary.content_hash.clone();
    }
    if merged.extra_meta.is_none() {
        merged.extra_meta = secondary.extra_meta.clone();
    }
    if merged.raw_extended_meta.is_none() {
        merged.raw_extended_meta = secondary.raw_extended_meta.clone();
    }

    // Description: take the longer non-empty one (more detail is better).
    if secondary.desc.trim().len() > merged.desc.trim().len() {
        merged.desc = secondary.desc.clone();
    }

    // Union + dedupe tags by lowercase name (keep the highest-score).
    let mut tags: Vec<ExtractedTag> = Vec::new();
    for t in primary_tags.iter().chain(secondary_tags.iter()) {
        let key = t.name.to_ascii_lowercase();
        if let Some(existing) = tags.iter_mut().find(|e| e.name.to_ascii_lowercase() == key) {
            if t.score > existing.score {
                *existing = t.clone();
            }
        } else {
            tags.push(t.clone());
        }
    }

    (merged, tags)
}

/// Pick the single best leg when the host only managed to gather one. Tags
/// ride along unchanged.
pub fn single_leg(meta: &FicMetadata, tags: &[ExtractedTag]) -> (FicMetadata, Vec<ExtractedTag>) {
    (meta.clone(), tags.to_vec())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn meta(title: &str, updated: i64, words: i64, desc: &str) -> FicMetadata {
        FicMetadata {
            url_id: "3_1".into(),
            title: title.into(),
            author: "A".into(),
            chapters: 1,
            words,
            desc: desc.into(),
            published: 1000,
            updated,
            status: "ongoing".into(),
            source: "s".into(),
            source_id: 3,
            author_id: 0,
            author_url: String::new(),
            author_local_id: String::new(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        }
    }

    #[test]
    fn newer_leg_wins_core_fields() {
        let native = meta("Old Title", 1000, 500, "short");
        let wayback = meta("New Title", 2000, 600, "a much longer description here");
        let (merged, _tags) = merge_metadata(&native, &[], &wayback, &[]);
        assert_eq!(merged.title, "New Title");
        assert_eq!(merged.words, 600);
        assert_eq!(merged.updated, 2000);
        assert_eq!(merged.desc, "a much longer description here");
    }

    #[test]
    fn tie_prefers_primary() {
        let native = meta("Native", 1000, 500, "d");
        let wayback = meta("Wayback", 1000, 999, "other");
        let (merged, _tags) = merge_metadata(&native, &[], &wayback, &[]);
        assert_eq!(merged.title, "Native");
        assert_eq!(merged.words, 500);
    }

    #[test]
    fn unions_empty_fields_from_other_leg() {
        let mut native = meta("T", 2000, 100, "d");
        native.author_url = "https://example.com/u/1".into();
        let mut wayback = meta("T", 1000, 100, "d");
        wayback.author_id = 42;
        wayback.author_local_id = "local-7".into();
        let (merged, _tags) = merge_metadata(&native, &[], &wayback, &[]);
        assert_eq!(merged.author_url, "https://example.com/u/1");
        assert_eq!(merged.author_id, 42);
        assert_eq!(merged.author_local_id, "local-7");
    }

    #[test]
    fn tags_union_and_dedupe_by_lowercase_name() {
        let native = meta("T", 1000, 1, "d");
        let wayback = meta("T", 2000, 1, "d");
        let nt = vec![ExtractedTag::freeform("Sci-Fi"), ExtractedTag::character("Alice")];
        let wt = vec![ExtractedTag::freeform("sci-fi"), ExtractedTag::freeform("Adventure")];
        let (merged, tags) = merge_metadata(&native, &nt, &wayback, &wt);
        assert_eq!(merged.title, "T");
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"Sci-Fi".to_string()));
        assert!(names.contains(&"Adventure".to_string()));
        assert!(names.contains(&"Alice".to_string()));
        let sci_count = tags.iter().filter(|t| t.name.to_ascii_lowercase() == "sci-fi").count();
        assert_eq!(sci_count, 1);
    }

    #[test]
    fn single_leg_returns_input_unchanged() {
        let m = meta("Only", 1000, 10, "d");
        let t = vec![ExtractedTag::freeform("Solo")];
        let (merged, tags) = single_leg(&m, &t);
        assert_eq!(merged.title, "Only");
        assert_eq!(tags.len(), 1);
    }
}
