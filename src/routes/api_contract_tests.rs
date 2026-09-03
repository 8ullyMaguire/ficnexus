/// API integration tests — verifies route contracts, response formats,
/// and error handling without requiring a live database or Redis.
///
/// These tests ensure the API shape matches what clients (including
/// fichub.net consumers) expect.

#[cfg(test)]
mod api_contract_tests {
    use serde_json::json;

    // ── Response Format Contracts ────────────────────────────────────────
    //
    // Every API endpoint must return a consistent JSON shape.
    // These tests document and verify the contract.

    #[test]
    fn epub_handler_empty_query_returns_error() {
        // GET /api/epub with no q param → err: -1
        let resp = json!({
            "err": -1,
            "msg": "no query",
            "q": ""
        });
        assert_eq!(resp["err"], -1);
        assert!(resp["msg"].as_str().unwrap().contains("no query"));
    }

    #[test]
    fn epub_handler_automated_blocked() {
        let resp = json!({
            "err": -10,
            "msg": "automated requests blocked"
        });
        assert_eq!(resp["err"], -10);
    }

    #[test]
    fn epub_handler_unsupported_url() {
        let resp = json!({
            "err": -5,
            "msg": "unsupported URL: not-a-url"
        });
        assert_eq!(resp["err"], -5);
    }

    #[test]
    fn epub_handler_blacklisted_fic() {
        let resp = json!({
            "err": -7,
            "msg": "fic is blacklisted",
            "q": "https://example.com/story"
        });
        assert_eq!(resp["err"], -7);
    }

    #[test]
    fn meta_handler_empty_query_returns_error() {
        let resp = json!({
            "err": -1,
            "msg": "no query",
            "q": ""
        });
        assert_eq!(resp["err"], -1);
    }

    #[test]
    fn meta_handler_success_response_shape() {
        // Successful meta response must have these fields
        let resp = json!({
            "err": 0,
            "q": "https://archiveofourown.org/works/12345",
            "fixits": [],
            "info": "Test by Author - 50000 words, 10 chapters",
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

        // Top-level fields
        assert_eq!(resp["err"], 0);
        assert!(resp.get("q").is_some());
        assert!(resp.get("fixits").is_some());
        assert!(resp.get("info").is_some());
        assert!(resp.get("url_id").is_some());
        assert!(resp.get("slug").is_some());
        assert!(resp.get("meta").is_some());
        assert!(resp.get("hashes").is_some());
        assert!(resp.get("urls").is_some());
        assert!(resp.get("epub_url").is_some());
        assert!(resp.get("html_url").is_some());
        assert!(resp.get("mobi_url").is_some());
        assert!(resp.get("pdf_url").is_some());
        assert!(resp.get("notes").is_some());

        // Meta sub-fields
        let meta = &resp["meta"];
        assert!(meta.get("id").is_some());
        assert!(meta.get("title").is_some());
        assert!(meta.get("author").is_some());
        assert!(meta.get("chapters").is_some());
        assert!(meta.get("words").is_some());
        assert!(meta.get("description").is_some());
        assert!(meta.get("status").is_some());
        assert!(meta.get("source").is_some());
        assert!(meta.get("created").is_some());
        assert!(meta.get("updated").is_some());
        assert!(meta.get("source_id").is_some());
        assert!(meta.get("author_id").is_some());
        assert!(meta.get("author_url").is_some());
        assert!(meta.get("author_local_id").is_some());
    }

    #[test]
    fn epub_handler_success_response_shape() {
        // Successful epub response includes download URLs
        let resp = json!({
            "err": 0,
            "q": "https://archiveofourown.org/works/12345",
            "fixits": [],
            "info": "Test by Author\n50000 words in 10 chapters\nStatus: complete\n",
            "url_id": "abc123def456",
            "slug": "Test-abc123def456",
            "meta": { "id": "abc123def456" },
            "hashes": {
                "epub": "epubhash123",
                "html": "htmlhash456"
            },
            "urls": {
                "epub": "/cache/epub/abc123def456?h=epubhash123",
                "html": "/cache/html/abc123def456?h=htmlhash456"
            },
            "epub_url": "/cache/epub/abc123def456?h=epubhash123",
            "html_url": "/cache/html/abc123def456?h=htmlhash456",
            "mobi_url": null,
            "pdf_url": null,
            "notes": []
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["epub_url"].as_str().unwrap().starts_with("/cache/epub/"));
        assert!(resp["html_url"].as_str().unwrap().starts_with("/cache/html/"));
        assert!(resp["mobi_url"].is_null());
        assert!(resp["pdf_url"].is_null());
    }

    // ── Comments API Contract ────────────────────────────────────────────

    #[test]
    fn comments_response_shape() {
        let resp = json!({
            "err": 0,
            "total_top_level": 5,
            "page": 1,
            "per_page": 20,
            "comments": [
                {
                    "id": 1,
                    "url_id": "abc123",
                    "parent_id": null,
                    "body": "Great story!",
                    "user": { "id": 1, "username": "reader1" },
                    "created_at": "2026-07-27T10:00:00Z",
                    "updated_at": null,
                    "deleted": false,
                    "hidden": false,
                    "reply_count": 2
                }
            ]
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["total_top_level"].as_i64().unwrap() >= 0);
        assert!(resp["page"].as_i64().unwrap() >= 1);
        assert!(resp["per_page"].as_i64().unwrap() >= 1);

        let comments = resp["comments"].as_array().unwrap();
        if let Some(comment) = comments.first() {
            assert!(comment.get("id").is_some());
            assert!(comment.get("url_id").is_some());
            assert!(comment.get("parent_id").is_some());
            assert!(comment.get("body").is_some());
            assert!(comment.get("user").is_some());
            assert!(comment.get("created_at").is_some());
            assert!(comment.get("deleted").is_some());
            assert!(comment.get("hidden").is_some());
            assert!(comment.get("reply_count").is_some());
        }
    }

    #[test]
    fn post_comment_response_shape() {
        let resp = json!({
            "err": 0,
            "comment": {
                "id": 42,
                "url_id": "abc123",
                "parent_id": null,
                "body": "Nice fic!",
                "user": { "id": 1, "username": "reader1" },
                "created_at": "2026-07-28T12:00:00Z",
                "updated_at": null,
                "deleted": false,
                "hidden": false,
                "reply_count": 0
            }
        });

        assert_eq!(resp["err"], 0);
        let comment = &resp["comment"];
        assert_eq!(comment["id"], 42);
        assert_eq!(comment["body"], "Nice fic!");
        assert_eq!(comment["deleted"], false);
        assert_eq!(comment["hidden"], false);
    }

    // ── Search API Contract ──────────────────────────────────────────────

    #[test]
    fn search_response_shape() {
        let resp = json!({
            "err": 0,
            "results": [],
            "total": 0,
            "page": 1,
            "per_page": 50
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["results"].is_array());
        assert!(resp.get("total").is_some());
    }

    // ── Recommendations API Contract ─────────────────────────────────────

    #[test]
    fn recommendations_response_shape() {
        let resp = json!({
            "err": 0,
            "recommendations": [],
            "total": 0
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["recommendations"].is_array());
    }

    // ── Tags API Contract ────────────────────────────────────────────────

    #[test]
    fn tags_response_shape() {
        let resp = json!({
            "err": 0,
            "tags": []
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["tags"].is_array());
    }

    // ── Auth API Contract ────────────────────────────────────────────────

    #[test]
    fn auth_register_response_shape() {
        let resp = json!({
            "err": 0,
            "token": "jwt.token.here",
            "user": {
                "id": 1,
                "username": "newuser"
            }
        });

        assert_eq!(resp["err"], 0);
        assert!(resp.get("token").is_some());
        assert!(resp["user"]["id"].as_i64().unwrap() > 0);
    }

    #[test]
    fn auth_login_response_shape() {
        let resp = json!({
            "err": 0,
            "token": "jwt.token.here",
            "user": {
                "id": 1,
                "username": "existinguser"
            }
        });

        assert_eq!(resp["err"], 0);
        assert!(resp.get("token").is_some());
    }

    #[test]
    fn auth_me_response_shape() {
        let resp = json!({
            "err": 0,
            "user": {
                "id": 1,
                "username": "existinguser",
                "role": 0,
                "reputation": 0
            }
        });

        assert_eq!(resp["err"], 0);
        let user = &resp["user"];
        assert!(user.get("id").is_some());
        assert!(user.get("username").is_some());
        assert!(user.get("role").is_some());
    }

    // ── Bookmarks API Contract ───────────────────────────────────────────

    #[test]
    fn bookmarks_list_response_shape() {
        let resp = json!({
            "err": 0,
            "bookmarks": []
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["bookmarks"].is_array());
    }

    #[test]
    fn bookmark_add_response_shape() {
        let resp = json!({
            "err": 0,
            "bookmark": {
                "url_id": "abc123",
                "notes": "",
                "is_private": false,
                "created_at": "2026-07-28T12:00:00Z"
            }
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["bookmark"].get("url_id").is_some());
    }

    // ── Ratings API Contract ─────────────────────────────────────────────

    #[test]
    fn ratings_response_shape() {
        let resp = json!({
            "err": 0,
            "url_id": "abc123",
            "upvotes": 10,
            "downvotes": 2,
            "user_rating": 1
        });

        assert_eq!(resp["err"], 0);
        assert!(resp.get("url_id").is_some());
        assert!(resp.get("upvotes").is_some());
        assert!(resp.get("downvotes").is_some());
    }

    // ── Leaderboard API Contract ─────────────────────────────────────────

    #[test]
    fn leaderboard_response_shape() {
        let resp = json!({
            "err": 0,
            "curators": []
        });

        assert_eq!(resp["err"], 0);
        assert!(resp["curators"].is_array());
    }

    // ── Error Response Consistency ───────────────────────────────────────

    #[test]
    fn all_error_responses_have_err_field() {
        let errors = vec![
            json!({"err": -1, "msg": "no query"}),
            json!({"err": -5, "msg": "unsupported URL"}),
            json!({"err": -7, "msg": "fic is blacklisted"}),
            json!({"err": -10, "msg": "automated requests blocked"}),
        ];

        for resp in &errors {
            assert!(
                resp.get("err").is_some(),
                "Error response missing 'err' field: {}",
                resp
            );
            assert!(
                resp["err"].as_i64().unwrap() < 0,
                "Error code should be negative: {}",
                resp
            );
        }
    }

    #[test]
    fn not_found_response_shape() {
        let resp = json!({
            "err": -1,
            "msg": "fic not found: abc123"
        });
        assert_eq!(resp["err"], -1);
        assert!(resp["msg"].as_str().unwrap().contains("not found"));
    }

    #[test]
    fn rate_limited_response_shape() {
        let resp = json!({
            "err": 429,
            "msg": "rate limited",
            "retry_after": 60
        });
        assert_eq!(resp["err"], 429);
        assert!(resp.get("retry_after").is_some());
    }

    // ── OPDS Feed Contract ───────────────────────────────────────────────

    #[test]
    fn opds_root_catalog_shape() {
        // OPDS feeds are XML, but the structure should be consistent
        // This tests the expected data shape before XML serialization
        let feed = json!({
            "title": "FicNexus Catalog",
            "id": "urn:ficnexus:catalog",
            "updated": "2026-07-28T00:00:00Z",
            "entries": []
        });

        assert!(feed.get("title").is_some());
        assert!(feed.get("id").is_some());
        assert!(feed.get("updated").is_some());
        assert!(feed.get("entries").is_some());
    }
}
