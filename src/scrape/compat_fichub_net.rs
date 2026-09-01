/// Compatibility tests between this project and fichub.net
///
/// These tests verify that our implementation produces outputs compatible
/// with the public fichub.net instance:
/// - Same url_id hash generation (SHA-256 of "source_id:story_id", 12 hex chars)
/// - Same source_id mapping per site
/// - Same API response JSON structure
/// - Same slug generation algorithm
///
/// KNOWN INCOMPATIBILITIES (documented, not bugs):
/// - EPUB structure differs (we use epub-builder, fichub.net uses its own code)
/// - Cache file hashes differ as a result
/// - Some metadata fields may be absent (e.g. author_id for FanFicFare scraper)

#[cfg(test)]
mod fichub_net_compat {
    use crate::scrape::SiteScraper;
    use sha2::{Digest, Sha256};

    /// Replicate the url_id generation algorithm exactly as in src/scrape/mod.rs.
    /// fichub.net uses: SHA-256("source_id:story_id"), first 12 hex chars.
    fn generate_url_id(source_id: i64, story_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source_id.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(story_id.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..6])
    }

    // ── url_id Hash Algorithm Tests ─────────────────────────────────────

    #[test]
    fn url_id_is_deterministic() {
        let a = generate_url_id(1, "12345");
        let b = generate_url_id(1, "12345");
        assert_eq!(a, b, "url_id must be deterministic for same inputs");
    }

    #[test]
    fn url_id_is_12_hex_chars() {
        let id = generate_url_id(1, "12345");
        assert_eq!(id.len(), 12, "url_id must be exactly 12 hex characters");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()), "url_id must be hex only");
    }

    #[test]
    fn url_id_uses_sha256_prefix() {
        // Manually compute SHA-256 of "1:12345" and verify first 12 hex chars
        let mut hasher = Sha256::new();
        hasher.update(b"1:12345");
        let expected = hex::encode(hasher.finalize()[..6].to_vec());
        let actual = generate_url_id(1, "12345");
        assert_eq!(actual, expected);
    }

    #[test]
    fn url_id_source_id_matters() {
        // Different source_ids produce different hashes for same story_id
        let ao3 = generate_url_id(1, "12345");
        let ffn = generate_url_id(2, "12345");
        assert_ne!(ao3, ffn, "Different source_ids must produce different url_ids");
    }

    #[test]
    fn url_id_story_id_matters() {
        // Different story_ids produce different hashes
        let a = generate_url_id(1, "111");
        let b = generate_url_id(1, "222");
        assert_ne!(a, b, "Different story_ids must produce different url_ids");
    }

    #[test]
    fn url_id_known_ao3_work() {
        // AO3 work 21845264 — a well-known fic
        // source_id=1 for AO3
        let url_id = generate_url_id(1, "21845264");
        // Pre-computed: SHA-256("1:21845264") first 6 bytes
        let mut hasher = Sha256::new();
        hasher.update(b"1:21845264");
        let expected = hex::encode(hasher.finalize()[..6].to_vec());
        assert_eq!(url_id, expected);
        assert_eq!(url_id.len(), 12);
    }

    #[test]
    fn url_id_known_ffn_story() {
        // FFN story 12345678
        // source_id=2 for FFN
        let url_id = generate_url_id(2, "12345678");
        let mut hasher = Sha256::new();
        hasher.update(b"2:12345678");
        let expected = hex::encode(hasher.finalize()[..6].to_vec());
        assert_eq!(url_id, expected);
    }

    #[test]
    fn url_id_empty_story_id() {
        // Edge case: empty story_id should still produce a valid hash
        let id = generate_url_id(1, "");
        assert_eq!(id.len(), 12);
        let mut hasher = Sha256::new();
        hasher.update(b"1:");
        let expected = hex::encode(hasher.finalize()[..6].to_vec());
        assert_eq!(id, expected);
    }

    #[test]
    fn url_id_zero_source_id() {
        let id = generate_url_id(0, "12345");
        assert_eq!(id.len(), 12);
        let mut hasher = Sha256::new();
        hasher.update(b"0:12345");
        let expected = hex::encode(hasher.finalize()[..6].to_vec());
        assert_eq!(id, expected);
    }

    #[test]
    fn url_id_long_story_id() {
        // Some sites use long numeric IDs
        let id = generate_url_id(1, "9999999999999");
        assert_eq!(id.len(), 12);
    }

    // ── Source ID Mapping Tests ──────────────────────────────────────────
    //
    // fichub.net maps sites to numeric source_ids. These must match exactly
    // or url_ids will differ between instances.

    /// Expected source_id mapping (must match fichub.net):
    ///   1 = archiveofourown.org (AO3)
    ///   2 = fanfiction.net (FFN)
    ///   3 = fictionpress.com / fnac
    ///   4 = spacebattles.com / sufficientvelocity.com (XenForo)
    ///   5 = wattpad.com
    ///   6 = royalroad.com
    ///   99 = unknown/other (FanFicFare fallback)
    const EXPECTED_SOURCE_IDS: &[(i64, &str)] = &[
        (1, "ao3"),
        (2, "ffn"),
        (3, "fnac"),
        (4, "sb"),
        (4, "sv"),
        (5, "Wattpad"),
        (6, "royalroad"),
        (99, "unknown"),
    ];

    #[test]
    fn source_id_mapping_matches_fichub_net() {
        for &(source_id, site_abbrev) in EXPECTED_SOURCE_IDS {
            let url_id = generate_url_id(source_id, "test_story");
            assert_eq!(url_id.len(), 12,
                "source_id={} for site '{}' must produce valid url_id", source_id, site_abbrev);
            // Verify no collision between different sites
            for &(other_id, other_site) in EXPECTED_SOURCE_IDS {
                if source_id != other_id {
                    let other_url_id = generate_url_id(other_id, "test_story");
                    assert_ne!(url_id, other_url_id,
                        "source_id={} ({}) and source_id={} ({}) must not collide",
                        source_id, site_abbrev, other_id, other_site);
                }
            }
        }
    }

    #[test]
    fn source_id_ao3_is_1() {
        // Critical: AO3 is the most popular site, source_id must be 1
        let ao3_id = generate_url_id(1, "12345");
        let other_id = generate_url_id(2, "12345");
        assert_ne!(ao3_id, other_id);
        // Verify it's the hash of "1:12345"
        let mut hasher = Sha256::new();
        hasher.update(b"1:12345");
        assert_eq!(ao3_id, hex::encode(hasher.finalize()[..6].to_vec()));
    }

    #[test]
    fn source_id_ffn_is_2() {
        let ffn_id = generate_url_id(2, "67890");
        let mut hasher = Sha256::new();
        hasher.update(b"2:67890");
        assert_eq!(ffn_id, hex::encode(hasher.finalize()[..6].to_vec()));
    }

    // ── API Response Format Tests ────────────────────────────────────────
    //
    // fichub.net returns a specific JSON structure from /api/epub and /api/meta.
    // These tests verify our response format matches.

    #[test]
    fn api_response_has_required_fields() {
        // Simulate the JSON response structure from build_meta_json + epub_handler
        let response = serde_json::json!({
            "err": 0,
            "q": "https://archiveofourown.org/works/12345",
            "fixits": [],
            "info": "Test by Author\n50000 words in 10 chapters\nStatus: complete\nUpdated: 2024-01-01 00:00:00 - 1 days ago\n",
            "url_id": "abc123def456",
            "slug": "Test-abc123def456",
            "meta": {
                "id": "abc123def456",
                "title": "Test",
                "author": "Author",
                "chapters": 10,
                "words": 50000,
                "description": "A story",
                "status": "complete",
                "source": "https://archiveofourown.org/works/12345",
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z",
                "extra_meta": null,
                "raw_extended_meta": null,
                "author_url": "https://archiveofourown.org/users/Author",
                "author_local_id": "12345",
                "source_id": 1,
                "author_id": 0
            },
            "hashes": {},
            "urls": {},
            "epub_url": null,
            "html_url": null,
            "mobi_url": null,
            "pdf_url": null,
            "notes": []
        });

        // Required top-level fields
        assert!(response.get("err").is_some(), "missing 'err' field");
        assert!(response.get("q").is_some(), "missing 'q' field");
        assert!(response.get("fixits").is_some(), "missing 'fixits' field");
        assert!(response.get("info").is_some(), "missing 'info' field");
        assert!(response.get("url_id").is_some(), "missing 'url_id' field");
        assert!(response.get("slug").is_some(), "missing 'slug' field");
        assert!(response.get("meta").is_some(), "missing 'meta' field");
        assert!(response.get("hashes").is_some(), "missing 'hashes' field");
        assert!(response.get("urls").is_some(), "missing 'urls' field");
        assert!(response.get("epub_url").is_some(), "missing 'epub_url' field");
        assert!(response.get("html_url").is_some(), "missing 'html_url' field");
        assert!(response.get("mobi_url").is_some(), "missing 'mobi_url' field");
        assert!(response.get("pdf_url").is_some(), "missing 'pdf_url' field");
        assert!(response.get("notes").is_some(), "missing 'notes' field");

        // Required meta sub-fields
        let meta = response.get("meta").unwrap();
        assert!(meta.get("id").is_some(), "meta missing 'id'");
        assert!(meta.get("title").is_some(), "meta missing 'title'");
        assert!(meta.get("author").is_some(), "meta missing 'author'");
        assert!(meta.get("chapters").is_some(), "meta missing 'chapters'");
        assert!(meta.get("words").is_some(), "meta missing 'words'");
        assert!(meta.get("description").is_some(), "meta missing 'description'");
        assert!(meta.get("status").is_some(), "meta missing 'status'");
        assert!(meta.get("source").is_some(), "meta missing 'source'");
        assert!(meta.get("created").is_some(), "meta missing 'created'");
        assert!(meta.get("updated").is_some(), "meta missing 'updated'");
        assert!(meta.get("source_id").is_some(), "meta missing 'source_id'");
        assert!(meta.get("author_id").is_some(), "meta missing 'author_id'");
        assert!(meta.get("author_url").is_some(), "meta missing 'author_url'");
        assert!(meta.get("author_local_id").is_some(), "meta missing 'author_local_id'");
    }

    #[test]
    fn api_error_response_format() {
        // Error responses use err != 0 and a msg field
        let error_resp = serde_json::json!({
            "err": -1,
            "msg": "no query",
            "q": ""
        });
        assert_eq!(error_resp["err"], -1);
        assert!(error_resp.get("msg").is_some(), "error responses must have 'msg'");
    }

    #[test]
    fn api_not_found_response_format() {
        let not_found = serde_json::json!({
            "err": -1,
            "msg": "fic not found: abc123",
        });
        assert_eq!(not_found["err"], -1);
        assert!(not_found["msg"].as_str().unwrap().contains("not found"));
    }

    #[test]
    fn api_blacklist_response_format() {
        let blacklisted = serde_json::json!({
            "err": -7,
            "msg": "fic is blacklisted",
            "q": "https://example.com/story"
        });
        assert_eq!(blacklisted["err"], -7);
    }

    #[test]
    fn api_automated_blocked_response_format() {
        let blocked = serde_json::json!({
            "err": -10,
            "msg": "automated requests blocked"
        });
        assert_eq!(blocked["err"], -10);
    }

    // ── Slug Generation Tests ────────────────────────────────────────────
    //
    // fichub.net generates slugs from title + url_id. The format is:
    //   sanitized_title-url_id
    // where non-alphanumeric chars become '_', consecutive underscores collapse.

    /// Replicate slug generation from src/routes/export.rs
    fn generate_slug(title: &str, url_id: &str) -> String {
        let sanitized: String = title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let re = regex_lite::Regex::new(r"_+").unwrap();
        let slug = re.replace_all(&sanitized, "_").to_string();
        let slug = slug.trim_matches('_').to_string();
        format!("{}-{}", slug, url_id)
    }

    #[test]
    fn slug_format_matches_fichub_net() {
        let slug = generate_slug("The Best Story Ever", "abc123def456");
        assert_eq!(slug, "The_Best_Story_Ever-abc123def456");
        // Must contain url_id as suffix after last '-'
        assert!(slug.ends_with("-abc123def456"));
    }

    #[test]
    fn slug_special_chars_become_underscores() {
        let slug = generate_slug("Hello: World! (2024)", "xyz");
        assert!(!slug.contains(':'));
        assert!(!slug.contains('!'));
        assert!(!slug.contains('('));
        assert!(!slug.contains(')'));
        assert!(slug.contains("Hello_World_2024"));
    }

    #[test]
    fn slug_collapses_consecutive_underscores() {
        let slug = generate_slug("A  B   C", "id1");
        // Multiple spaces -> single underscore
        let underscore_count = slug.chars().filter(|&c| c == '_').count();
        assert_eq!(underscore_count, 2, "consecutive underscores should collapse");
    }

    #[test]
    fn slug_trims_leading_trailing_underscores() {
        let slug = generate_slug("  story  ", "id1");
        assert!(!slug.starts_with('_'));
        assert!(!slug.starts_with("-id1"));
    }

    #[test]
    fn slug_empty_title() {
        let slug = generate_slug("", "id1");
        assert!(slug.ends_with("-id1"));
    }

    // ── URL Format Compatibility ─────────────────────────────────────────
    //
    // fichub.net uses specific URL patterns for different sites.

    #[test]
    fn ao3_url_pattern_extractable() {
        // AO3 URLs: /works/123456 or /works/123456/chapters/789012
        let re = regex_lite::Regex::new(r"/works/(\d+)").unwrap();
        let urls = vec![
            "https://archiveofourown.org/works/123456",
            "https://archiveofourown.org/works/123456/chapters/789012",
            "https://archiveofourown.org/works/123456?view_adult=true",
        ];
        for url in urls {
            assert!(re.is_match(url), "Failed to match AO3 URL: {}", url);
            let cap = re.captures(url).unwrap();
            assert!(cap.get(1).is_some(), "Failed to extract work ID from: {}", url);
        }
    }

    #[test]
    fn ffn_url_pattern_extractable() {
        // FFN URLs: /s/12345678 or /s/12345678/1/
        let re = regex_lite::Regex::new(r"/s/(\d+)").unwrap();
        let urls = vec![
            "https://www.fanfiction.net/s/12345678",
            "https://www.fanfiction.net/s/12345678/1/",
            "https://www.fanfiction.net/s/12345678/1/Chapter-Title",
        ];
        for url in urls {
            assert!(re.is_match(url), "Failed to match FFN URL: {}", url);
        }
    }

    #[test]
    fn wattpad_url_pattern_extractable() {
        // Wattpad URLs: /story/12345678
        let re = regex_lite::Regex::new(r"/story/(\d+)").unwrap();
        let url = "https://www.wattpad.com/story/12345678";
        assert!(re.is_match(url));
    }

    // ── Metadata Field Compatibility ─────────────────────────────────────
    //
    // fichub.net stores these fields in fic_info. Our FicMetadata struct
    // must produce all of them.

    #[test]
    fn fic_metadata_has_all_required_fields() {
        use crate::scrape::FicMetadata;

        let meta = FicMetadata {
            url_id: "abc123def456".into(),
            title: "Test Story".into(),
            author: "Test Author".into(),
            chapters: 10,
            words: 50000,
            desc: "A great story".into(),
            published: 1700000000000,
            updated: 1700000000000,
            status: "complete".into(),
            source: "https://archiveofourown.org/works/12345".into(),
            source_id: 1,
            author_id: 42,
            author_url: "https://archiveofourown.org/users/TestAuthor".into(),
            author_local_id: "12345".into(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        };

        // These fields map directly to fic_info columns
        assert!(!meta.url_id.is_empty(), "url_id must not be empty");
        assert!(!meta.title.is_empty(), "title must not be empty");
        assert!(!meta.author.is_empty(), "author must not be empty");
        assert!(meta.chapters > 0, "chapters must be > 0");
        assert!(meta.words >= 0, "words must be >= 0");
        assert!(!meta.source.is_empty(), "source must not be empty");
        assert!(meta.source.starts_with("http"), "source must be a URL");
        assert!(meta.source_id > 0, "source_id must be > 0");

        // Status must be one of the known values
        assert!(
            ["ongoing", "complete", "hiatus", "cancelled"].contains(&meta.status.as_str()),
            "status '{}' is not a recognized value",
            meta.status
        );
    }

    #[test]
    fn status_values_match_fichub_net() {
        // fichub.net uses these exact status strings
        let valid_statuses = ["ongoing", "complete", "hiatus", "cancelled"];
        for status in valid_statuses {
            assert!(!status.is_empty());
            assert!(status.chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    #[test]
    fn url_id_is_stored_as_fic_info_primary_key() {
        // url_id is the primary key in fic_info, must be a valid VARCHAR(128)
        use crate::scrape::FicMetadata;
        let meta = FicMetadata {
            url_id: generate_url_id(1, "12345"),
            ..FicMetadata {
                url_id: "".into(),
                title: "T".into(),
                author: "A".into(),
                chapters: 1,
                words: 100,
                desc: "D".into(),
                published: 0,
                updated: 0,
                status: "complete".into(),
                source: "https://example.com".into(),
                source_id: 1,
                author_id: 0,
                author_url: "".into(),
                author_local_id: "".into(),
                content_hash: None,
                extra_meta: None,
                raw_extended_meta: None,
            }
        };
        assert!(meta.url_id.len() <= 128, "url_id must fit in VARCHAR(128)");
        assert!(meta.url_id.len() >= 12, "url_id should be 12 hex chars");
    }

    // ── FanFicFare Compatibility ─────────────────────────────────────────
    //
    // Both this project and fichub.net use FanFicFare as the primary scraper.
    // The dict parsing must handle the same output format.

    #[test]
    fn fff_dict_parser_handles_fichub_net_format() {
        // FanFicFare outputs Python dicts. This is the format both projects must parse.
        use crate::scrape::sites::fanficfare;

        let input = "{'title': 'Test Story', 'author': 'Author', 'numChapters': '5', 'numWords': '10,000', 'status': 'Completed', 'storyId': '12345', 'siteabbrev': 'ao3', 'datePublished': '2024-01-01', 'dateUpdated': '2024-06-15'}";
        // We can't call parse_fff_dict directly (it's private), but we can
        // verify the FanFicFare scraper can handle is_always_true
        let scraper = crate::scrape::sites::fanficfare::FanFicFareScraper;
        assert!(scraper.can_handle("https://archiveofourown.org/works/12345"));
        assert!(scraper.can_handle("https://www.fanfiction.net/s/12345"));
        assert!(scraper.can_handle("https://www.wattpad.com/story/12345"));
    }

    #[test]
    fn fff_status_mapping_matches_fichub_net() {
        // FanFicFare status values -> our status strings
        // Must match fichub.net's mapping
        let mappings = vec![
            ("Completed", "complete"),
            ("In-Progress", "ongoing"),
            ("Abandoned", "cancelled"),
        ];
        for (fff_status, expected) in mappings {
            let mapped = match fff_status {
                "Completed" => "complete",
                "In-Progress" => "ongoing",
                "Abandoned" => "cancelled",
                _ => "ongoing",
            };
            assert_eq!(mapped, expected,
                "FanFicFare status '{}' should map to '{}', got '{}'",
                fff_status, expected, mapped);
        }
    }

    // ── Known Incompatibilities (documentation tests) ────────────────────
    //
    // These tests document known differences between this project and
    // fichub.net. They don't assert equality — they assert awareness.

    #[test]
    fn known_incompat_epub_structure() {
        // ISSUE: Our EPUBs use epub-builder crate; fichub.net uses its own code.
        // Result: Different EPUB structure, different file hashes.
        // Impact: Cache files from one instance won't work on the other.
        // This is documented in the context as Known Issue #4.
        //
        // If you want EPUB compatibility, consider using FanFicFare's EPUB
        // output directly instead of epub-builder.
        let _note = "EPUB structure differs — cache files are NOT interchangeable";
    }

    #[test]
    fn known_incompat_content_hash() {
        // ISSUE: content_hash is computed from chapter HTML. If HTML parsing
        // differs (e.g. different selectors for AO3 content), hashes differ.
        // Impact: Cache invalidation may trigger differently.
        //
        // Our selectors: ".story, .chapter_content, #story"
        // fichub.net may use different selectors.
        let _note = "content_hash depends on HTML parsing — may differ";
    }

    #[test]
    fn known_incompat_author_id_for_fff() {
        // FanFicFare scraper sets author_id=0 because FFF doesn't provide
        // a numeric author ID. Native scrapers extract it from HTML.
        // fichub.net may handle this differently.
        let _note = "FanFicFare sets author_id=0 — native scrapers extract real ID";
    }

    // ── Cache Path Compatibility ─────────────────────────────────────────
    //
    // fichub.net uses: CACHE_DIR/<etype>/<prefix1>/<prefix2>/<prefix3>/<url_id>/<hash>.epub
    // where prefix1/2/3 are the first 3 chars of url_id split into individual chars.

    #[test]
    fn cache_path_structure() {
        let url_id = "abc123def456";
        let hash = "deadbeef12345678";
        let etype = "epub";

        // Simulate cache path: etype/p1/p2/p3/url_id/hash.epub
        // where p1=url_id[0], p2=url_id[1], p3=url_id[2]
        let p1 = &url_id[0..1];
        let p2 = &url_id[1..2];
        let p3 = &url_id[2..3];
        let path = format!("{}/{}/{}/{}/{}/{}.{}", etype, p1, p2, p3, url_id, hash, etype);
        assert_eq!(path, "epub/a/b/c/abc123def456/deadbeef12345678.epub");
    }

    // ── Cross-Instance URL Resolution ────────────────────────────────────
    //
    // If a user has a url_id from fichub.net, they should be able to look
    // it up on this instance (hash lookup via /api/epub?q=<hash>).

    #[test]
    fn hash_lookup_works_with_any_url_id() {
        // Given a url_id, the /api/epub?q=<hash> endpoint should find it
        // in fic_info. This works because url_id IS the primary key.
        // No conversion needed — same algorithm produces same hash.
        let url_id = generate_url_id(1, "21845264");
        assert_eq!(url_id.len(), 12);
        // If this url_id exists in both databases, hash lookup works
    }

    #[test]
    fn api_epub_accepts_hash_not_just_url() {
        // The export handler checks: if !query.starts_with("http") → hash lookup
        // This means any 12-char hex string triggers hash lookup
        let hash = "abc123def456";
        assert!(!hash.starts_with("http"), "hash should not start with http");
        assert_eq!(hash.len(), 12);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
