use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;
use crate::scrape::FicMetadata;
use crate::db::models::*;
use crate::db::queries;
use crate::cache::{self, EType};
use crate::export;
use crate::services::pow;

/// Query parameters for export requests
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}

/// Main export handler: GET /api/v0/epub?q=<url>
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    // ── Tiered rate limit (download tier) ─────────────────────────────
    // Keyed by real client IP AND (ip, client_id) so shared-NAT readers are
    // not punished and identified bots are throttled per-client. A
    // shadowbanned client_id gets a much stricter bucket (friction, not a
    // hard block). Errors are treated as allow (fail-open) so a Redis hiccup
    // never blocks real downloads.
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let _client_ip: Option<std::net::IpAddr> = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok());
    let real_ip = crate::limiter::client_ip_from_headers(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );

    // ── Proof-of-work gate (shadowbanned clients only) ───────────────
    // A flagged client must solve a hashcash-style challenge before the
    // export is served. The challenge is re-issued (429 + fresh challenge)
    // until the client POSTs a valid solve, which is cached in Redis for
    // POW_TTL_SECS (10 min by default) so the follow-up export requests
    // pass without re-solving. Non-shadowbanned clients and requests
    // without a client_id skip this entirely. Redis errors fail open: a
    // hiccup re-issues the challenge instead of un-blocking a bot.
    if let Some(ref cid) = client_id {
        if state.rate_limiter.is_shadowbanned(cid) {
            let last_challenge = pow::latest_challenge_for_client(&state, cid);
            let solved = pow::solution_solved(
                &mut state.redis.clone(),
                last_challenge.as_deref().unwrap_or(""),
            )
            .await;
            if !solved {
                let challenge = pow::generate_challenge(
                    state.config.pow_difficulty,
                    state.config.pow_ttl_secs,
                );
                pow::set_latest_challenge_for_client(&state, cid, &challenge.challenge);
                return Err(AppError::RateLimitedJson(json!({
                    "err": -429,
                    "msg": "proof of work required",
                    "challenge": challenge.challenge,
                    "difficulty": challenge.difficulty,
                    "expires_at": challenge.expires_at,
                })));
            }
        }
    }

    let tier = state.rate_limiter.tier_for_path("/api/epub");
    match state
        .rate_limiter
        .check(real_ip, client_id.as_deref(), tier)
        .await
    {
        crate::limiter::TieredRateLimitResult::Wait(secs) => {
            return Err(AppError::RateLimited(secs));
        }
        crate::limiter::TieredRateLimitResult::Allowed => {}
    }

    // Validate query parameter
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    // Check for automated flag (block if present)
    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }

    // If q looks like a hash (not a URL), look up from fic_info directly
    if !query.starts_with("http") {
        return handle_hash_lookup(&state, query, start.elapsed().as_millis() as i32).await;
    }

    // Find the appropriate scraper (prefer native, fall back to FanFicFare)
    let scraper = match state.scraper_registry.find_specific_or_fff(query) {
        Some(s) => s,
        None => {
            state
                .heal
                .record_failure(
                    query,
                    None,
                    &crate::heal::classifier::ErrorKind::Unknown,
                    Some("no scraper registered for host"),
                    None,
                )
                .await;
            return Err(AppError::BadRequest(format!("unsupported URL: {}", query)));
        }
    };

    // Lookup metadata
    let meta = match scraper.lookup(&state.http_client, query).await {
        Ok(m) => m,
        Err(e) => {
            // ── Wayback Machine fallback (P8#5) ────────────────────────
            // On host-blocking/transient failures, try the Internet Archive
            // snapshot. The site adapter's `lookup_from_html` seam (lesson 2:
            // parse-from-HTML) lets us parse metadata directly from the
            // snapshot — no M2 agent needed for supported sites (AO3, FFN).
            // If the site doesn't implement from-html parsing, fall through
            // to the snapshot-for-M2 path as before.
            let wayback_snapshot = if state.wayback.config.enabled
                && crate::heal::classifier::classify(
                    &crate::heal::classifier::ErrorKind::from(&e),
                    &crate::heal::classifier::url_domain(query)
                        .unwrap_or_else(|| query.to_string()),
                    &e.to_string(),
                ) != crate::heal::classifier::Class::Structural
            {
                state.wayback.fetch_snapshot(&state.http_client, query).await
            } else {
                None
            };

            if let Some(html) = &wayback_snapshot {
                // Try deterministic parse-from-HTML first (AO3/FFN and any
                // site that implements lookup_from_html).
                if let Ok(m) = scraper.lookup_from_html(html, query).await {
                    // Use the wayback-sourced metadata (flow continues to
                    // FicInfo upsert below, same as a normal lookup).
                    m
                } else {
                    // Parse failed — save for M2/heal.
                    let snapshot_path = crate::heal::snapshot::save_snapshot_html(query, html).await;
                    state
                        .heal
                        .record_failure(
                            query,
                            None,
                            &crate::heal::classifier::ErrorKind::from(&e),
                            Some(&e.to_string()),
                            snapshot_path.as_deref(),
                        )
                        .await;
                    return Err(AppError::ScrapeError(e.to_string()));
                }
            } else {
                state
                    .heal
                    .record_failure(
                        query,
                        None,
                        &crate::heal::classifier::ErrorKind::from(&e),
                        Some(&e.to_string()),
                        None,
                    )
                    .await;
                return Err(AppError::ScrapeError(e.to_string()));
            }
        }
    };

    let info_request_ms = start.elapsed().as_millis() as i32;

    // Upsert fic_info in database
    let fic_info_row = FicInfo {
        id: meta.url_id.clone(),
        created: None,
        updated: None,
        title: meta.title.clone(),
        author: meta.author.clone(),
        author_url: Some(meta.author_url.clone()),
        author_local_id: Some(meta.author_local_id.clone()),
        chapters: meta.chapters,
        words: meta.words,
        description: meta.desc.clone(),
        fic_created: chrono::DateTime::from_timestamp_millis(meta.published)
            .unwrap_or_default(),
        fic_updated: chrono::DateTime::from_timestamp_millis(meta.updated)
            .unwrap_or_default(),
        status: meta.status.clone(),
        source: meta.source.clone(),
        extra_meta: meta.extra_meta.clone(),
        raw_extended_meta: meta.raw_extended_meta.clone(),
        source_id: Some(meta.source_id),
        author_id: Some(meta.author_id),
        content_hash: meta.content_hash.clone(),
        work_id: None,
    };
    queries::upsert_fic_info(&state.db, &fic_info_row).await?;

    // Auto-merge: find or create work for this source
    let auto_merge_result = crate::works::find_or_create_work(&state.db, &meta).await?;
    let work_id = auto_merge_result.work_id;

    // Auto-populate tags from scraper (v3 tagging system)
    if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
        for tag in &extracted_tags {
            // Convert crate tag (f64/i32) → FicNexus tag (i16 score)
            let tag: crate::scrape::ExtractedTag = tag.clone().into();
            if let Ok(resolution) = crate::tags::resolve::resolve_tag(
                &state.db, &tag.name, tag.tag_type_id,
            ).await {
                let _ = queries::upsert_fic_tag(
                    &state.db, &meta.url_id, resolution.tag_id,
                    &std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                    tag.score,
                ).await;
            }
        }
        if !extracted_tags.is_empty() {
            tracing::debug!("Extracted {} tags for {}", extracted_tags.len(), meta.url_id);
        }
    }

    // Check blacklists
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
    if !fic_blacklist.is_empty() {
        // Greylist (reason 6) shows metadata but no download
        if fic_blacklist.iter().any(|b| b.reason == 6) {
            return Ok(build_metadata_response(&meta, &[], &state.config.export_version, None, true));
        }
        // Hard blacklist
        if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
            return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted", "q": query})));
        }
    }

    // Check author blacklist
    let author_blacklist = queries::check_author_blacklist(
        &state.db, meta.source_id, meta.author_id,
    ).await?;
    if !author_blacklist.is_empty() {
        return Ok(Json(json!({"err": -7, "msg": "author is blacklisted", "q": query})));
    }

    // Compute cache version
    let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;

    // Check cache (try EPUB first, then all formats)
    let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;

    let mut urls = std::collections::HashMap::new();
    let mut hashes = std::collections::HashMap::new();

    if let Some(export_log) = cached {
        // Cache hit - build URLs from cached hash
        let epub_hash = &export_log.export_hash;
        hashes.insert("epub".to_string(), epub_hash.clone());
        urls.insert("epub".to_string(), format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));

        // Other formats from cached EPUB
        for etype_str in &["html", "mobi", "pdf", "txt", "md", "azw3", "docx", "fb2", "kepub"] {
            if let Ok(_etype) = etype_str.parse::<EType>() {
                let e_input_hash = format!("epub:{}", epub_hash);
                if let Ok(Some(entry)) = queries::find_export_log(
                    &state.db, &meta.url_id, version, etype_str, &e_input_hash,
                ).await {
                    hashes.insert(etype_str.to_string(), entry.export_hash.clone());
                    urls.insert(etype_str.to_string(), format!("/cache/{}/{}?h={}", etype_str, meta.url_id, entry.export_hash));
                }
            }
        }

        let slug = generate_slug(&meta.title, &meta.url_id);
        let (info_str, notes) = build_info_string(&meta);

        // Fetch tags and sources for the work
        let tags = queries::get_fic_tags(&state.db, &meta.url_id, 0).await.unwrap_or_default();
        let tags_json = format_tags_json(&tags);
        let sources = queries::get_work_sources(&state.db, work_id).await.unwrap_or_default();
        let sources_json = format_sources_json(&sources);

        return Ok(Json(json!({
            "err": 0,
            "q": query,
            "fixits": [],
            "info": info_str,
            "url_id": meta.url_id,
            "work_id": work_id,
            "slug": slug,
            "meta": build_meta_json(&meta, Some(work_id)),
            "tags": tags_json,
            "sources": sources_json,
            "hashes": hashes,
            "urls": urls,
            "epub_url": urls.get("epub"),
            "html_url": urls.get("html"),
            "mobi_url": urls.get("mobi"),
            "pdf_url": urls.get("pdf"),
            "txt_url": urls.get("txt"),
            "md_url": urls.get("md"),
            "azw3_url": urls.get("azw3"),
            "docx_url": urls.get("docx"),
            "fb2_url": urls.get("fb2"),
            "kepub_url": urls.get("kepub"),
            "notes": notes,
        })));
    }

    // Cache miss - generate EPUB
    // Acquire semaphore to prevent duplicate concurrent exports
    let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
    let _permit = sem.acquire().await.map_err(|e| AppError::Internal(e.to_string()))?;

    // Check cache again (double-check pattern)
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
    if let Some(export_log) = cached {
        // Another concurrent request already generated it
        let epub_hash = &export_log.export_hash;
        hashes.insert("epub".to_string(), epub_hash.clone());
        urls.insert("epub".to_string(), format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
        let slug = generate_slug(&meta.title, &meta.url_id);
        let (info_str, notes) = build_info_string(&meta);

        // Fetch tags and sources for the work
        let tags = queries::get_fic_tags(&state.db, &meta.url_id, 0).await.unwrap_or_default();
        let tags_json = format_tags_json(&tags);
        let sources = queries::get_work_sources(&state.db, work_id).await.unwrap_or_default();
        let sources_json = format_sources_json(&sources);

        return Ok(Json(json!({
            "err": 0,
            "q": query,
            "fixits": [],
            "info": info_str,
            "url_id": meta.url_id,
            "work_id": work_id,
            "slug": slug,
            "meta": build_meta_json(&meta, Some(work_id)),
            "tags": tags_json,
            "sources": sources_json,
            "hashes": hashes,
            "urls": urls,
            "epub_url": urls.get("epub"),
            "html_url": urls.get("html"),
            "mobi_url": urls.get("mobi"),
            "pdf_url": urls.get("pdf"),
            "txt_url": urls.get("txt"),
            "md_url": urls.get("md"),
            "azw3_url": urls.get("azw3"),
            "docx_url": urls.get("docx"),
            "fb2_url": urls.get("fb2"),
            "kepub_url": urls.get("kepub"),
            "notes": notes,
        })));
    }

    // Fetch chapters. The body cache (site = cache of gathered fanfiction)
    // is checked first: if we already have the scraped body on disk, reuse
    // it (no re-scrape). A curator-approved fix is stored as a newer
    // version of the same blob.
    let chapters = if let Some(cached) = crate::body_cache::load_body(&state.config, &meta.url_id) {
        cached
    } else {
        let fetched = match scraper.fetch_chapters(&state.http_client, &meta).await {
            Ok(ch) => ch,
            Err(e) => {
                let kind = crate::heal::classifier::ErrorKind::from(&e);
                let domain = crate::heal::classifier::url_domain(&meta.source)
                    .unwrap_or_else(|| meta.source.clone());
                // Parse failures get an HTML snapshot (best-effort): re-fetch the
                // story URL, strip script/style blocks, truncate to 200KB, store
                // under tests/fixtures/scrape/<domain>/<fingerprint>.html. A
                // snapshot fetch failure is not fatal — the failure is recorded
                // with html_snapshot_path = NULL in that case.
                // ── Wayback Machine fallback (P8#5) ─────────────────────
                // For Blocked/NotFound/transient failures (NOT structural
                // parse errors — those go to M2), try the Internet Archive
                // snapshot FIRST; if found, use it as the snapshot so the
                // M2 single-chapter fallback can build an export from real
                // content instead of a dead host.
                let snapshot_path = if kind == crate::heal::classifier::ErrorKind::Parse {
                    crate::heal::snapshot::capture_snapshot(&state.http_client, &meta.source).await
                } else if state.wayback.config.enabled {
                    match state
                        .wayback
                        .fetch_snapshot(&state.http_client, &meta.source)
                        .await
                    {
                        Some(html) => {
                            crate::heal::snapshot::save_snapshot_html(&meta.source, &html).await
                        }
                        None => {
                            crate::heal::snapshot::capture_snapshot(
                                &state.http_client,
                                &meta.source,
                            )
                            .await
                        }
                    }
                } else {
                    crate::heal::snapshot::capture_snapshot(&state.http_client, &meta.source).await
                };
                // ── Self-healing M2: on-the-fly extraction ──────────────
                // Structural failures + agent enabled + use-on-the-fly: ask
                // the agent to extract metadata from the snapshot, validate
                // it, and complete the pending export with it (re-fetch
                // chapters using the extraction as a hint; last resort = a
                // single chapter built from the snapshot HTML). The export
                // path CONTINUES on success — no Err returned.
                let heal_class = crate::heal::classifier::classify(&kind, &domain, &e.to_string());
                let otf_meta = if heal_class == crate::heal::classifier::Class::Structural
                    && state.config.agent_enabled
                    && state.config.agent_use_on_fly
                {
                    attempt_on_the_fly_heal(&state, &meta.source, &snapshot_path, &e).await
                } else {
                    None
                };
                if let Some(healed) = otf_meta {
                    // Complete the export with the healed metadata: use the
                    // healed chapter fetch if it produced real chapters,
                    // else fall back to a single chapter from the snapshot.
                    let ch = state
                        .scraper_registry
                        .fetch_chapters(&state.http_client, &healed)
                        .await;
                    match ch {
                        Ok(ch) if !ch.is_empty() => {
                            tracing::warn!(
                                "heal: on-the-fly extraction for {} used for export",
                                healed.url_id
                            );
                            ch
                        }
                        _ => {
                            tracing::warn!(
                                "heal: on-the-fly extraction for {} — chapter fetch failed, \
                                 building single chapter from snapshot",
                                healed.url_id
                            );
                            let html = load_snapshot_html(&state, &meta.source, &snapshot_path)
                                .await
                                .unwrap_or_else(|| format!("<p>{}</p>", healed.desc));
                            vec![crate::scrape::Chapter {
                                chapter_id: 1,
                                title: healed.title.clone(),
                                content: html,
                            }]
                        }
                    }
                } else {
                    // ── M2 pending queue ────────────────────────────────
                    // Transient + Blocked failures persist the user request
                    // so a later successful heal auto-completes it.
                    if heal_class == crate::heal::classifier::Class::Transient
                        || heal_class == crate::heal::classifier::Class::Blocked
                    {
                        let _ = crate::heal::extract::enqueue_pending_export(
                            &state.db,
                            &meta.source,
                            "epub",
                            None,
                            None,
                            kind.as_str(),
                        )
                        .await;
                    }
                    state.heal.record_failure(
                        &meta.source,
                        Some(&meta.url_id),
                        &kind,
                        Some(&e.to_string()),
                        snapshot_path.as_deref(),
                    ).await;
                    return Err(AppError::ScrapeError(e.to_string()));
                }
            }
        };
        // Persist the scraped body (JSON chapters) + raw HTML so the site
        // caches all gathered fanfiction (blob on the attached drive, not
        // the DB). Saving raw HTML lets a bad extraction be re-done from
        // the saved page without re-scraping.
        let version = crate::body_cache::current_version(&state.config, &meta.url_id);
        if let Err(e) = crate::body_cache::save_body(
            &state.config,
            &meta.url_id,
            &fetched,
            Some(meta.source.clone()),
            version,
        ) {
            tracing::warn!("body_cache save failed for {}: {e}", meta.url_id);
        }
        // Keep the body FTS index fresh: recompute body_text_search from
        // the blob we just wrote. Best-effort (index_body_text swallows
        // errors) — the scrape path must never fail because indexing did.
        crate::body_cache::index_body_text(&state.db, &state.config, &meta.url_id).await;
        if let Err(e) = crate::body_cache::save_html(
            &state.config,
            &meta.url_id,
            &fetched.iter().map(|c| c.content.clone()).collect::<Vec<_>>().join("\n"),
            version,
        ) {
            tracing::warn!("html_cache save failed for {}: {e}", meta.url_id);
        }

        // ── Auto-moderation: classify the fic in the background ──────
        // Non-blocking — the fic is available immediately. If classified as
        // noise with high confidence, it gets flagged for curator review
        // with a 72h grace period before auto-deletion.
        {
            let db = state.db.clone();
            let ollama = state.ollama.clone();
            let config = state.config.clone();
            let url_id = meta.url_id.clone();
            tokio::spawn(async move {
                crate::services::content_scan::scan_single_fic(&db, &ollama, &config, &url_id).await;
            });
        }

        fetched
    };

    // Generate EPUB
    let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    // Move to cache
    let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
    cache::disk::move_to_cache(&epub_path, &cache_dest)?;

    // Record in export_log
    queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;

    // Notify work followers if this is an update (work was previously exported)
    if version_bump > 0 {
        let _ = queries::notify_work_followers(&state.db, work_id, &meta.title).await;
    }

    // Generate HTML bundle
    let (html_path, html_hash) = export::html_bundle::create_html_bundle(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    let html_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash);
    cache::disk::move_to_cache(&html_path, &html_cache_dest)?;
    let html_input_hash = format!("epub:{}", epub_hash);
    queries::insert_export_log(&state.db, &meta.url_id, version, "html", &html_input_hash, &html_hash).await?;

    // Generate TXT
    let (txt_path, txt_hash) = export::txt::create_txt(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    let txt_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Txt, &meta.url_id, &txt_hash);
    cache::disk::move_to_cache(&txt_path, &txt_cache_dest)?;
    let txt_input_hash = format!("epub:{}", epub_hash);
    queries::insert_export_log(&state.db, &meta.url_id, version, "txt", &txt_input_hash, &txt_hash).await?;

    // Generate Markdown
    let (md_path, md_hash) = export::md::create_md(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    let md_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Md, &meta.url_id, &md_hash);
    cache::disk::move_to_cache(&md_path, &md_cache_dest)?;
    let md_input_hash = format!("epub:{}", epub_hash);
    queries::insert_export_log(&state.db, &meta.url_id, version, "md", &md_input_hash, &md_hash).await?;

    // Generate MOBI via Calibre (lazy — only if already cached; on first
    // export these convert on demand via GET /api/epub/convert so the
    // initial response is fast).
    let mobi_hash = if let Some(h) = cached_format_hash(
        &state.db, &meta.url_id, version, &epub_hash, "mobi",
    ).await {
        Some(h)
    } else {
        None
    };

    // Generate PDF via Calibre (lazy)
    let pdf_hash = if let Some(h) = cached_format_hash(
        &state.db, &meta.url_id, version, &epub_hash, "pdf",
    ).await {
        Some(h)
    } else {
        None
    };

    // Generate AZW3 via Calibre (lazy)
    let azw3_hash = if let Some(h) = cached_format_hash(
        &state.db, &meta.url_id, version, &epub_hash, "azw3",
    ).await {
        Some(h)
    } else {
        None
    };

    // Generate DOCX (native)
    let (docx_path, docx_hash) = export::docx::create_docx(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    let docx_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Docx, &meta.url_id, &docx_hash);
    cache::disk::move_to_cache(&docx_path, &docx_cache_dest)?;
    let docx_input_hash = format!("epub:{}", epub_hash);
    queries::insert_export_log(&state.db, &meta.url_id, version, "docx", &docx_input_hash, &docx_hash).await?;

    // Generate FB2 (native)
    let (fb2_path, fb2_hash) = export::fb2::create_fb2(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    let fb2_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Fb2, &meta.url_id, &fb2_hash);
    cache::disk::move_to_cache(&fb2_path, &fb2_cache_dest)?;
    let fb2_input_hash = format!("epub:{}", epub_hash);
    queries::insert_export_log(&state.db, &meta.url_id, version, "fb2", &fb2_input_hash, &fb2_hash).await?;

    // Generate KEPUB (native)
    let (kepub_path, kepub_hash) = export::kepub::create_kepub(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    let kepub_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Kepub, &meta.url_id, &kepub_hash);
    cache::disk::move_to_cache(&kepub_path, &kepub_cache_dest)?;
    let kepub_input_hash = format!("epub:{}", epub_hash);
    queries::insert_export_log(&state.db, &meta.url_id, version, "kepub", &kepub_input_hash, &kepub_hash).await?;

    // Build response
    hashes.insert("epub".to_string(), epub_hash.clone());
    hashes.insert("html".to_string(), html_hash.clone());
    hashes.insert("txt".to_string(), txt_hash.clone());
    hashes.insert("md".to_string(), md_hash.clone());
    hashes.insert("docx".to_string(), docx_hash.clone());
    hashes.insert("fb2".to_string(), fb2_hash.clone());
    hashes.insert("kepub".to_string(), kepub_hash.clone());
    urls.insert("epub".to_string(), format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
    urls.insert("html".to_string(), format!("/cache/html/{}?h={}", meta.url_id, html_hash));
    urls.insert("txt".to_string(), format!("/cache/txt/{}?h={}", meta.url_id, txt_hash));
    urls.insert("md".to_string(), format!("/cache/md/{}?h={}", meta.url_id, md_hash));
    urls.insert("docx".to_string(), format!("/cache/docx/{}?h={}", meta.url_id, docx_hash));
    urls.insert("fb2".to_string(), format!("/cache/fb2/{}?h={}", meta.url_id, fb2_hash));
    urls.insert("kepub".to_string(), format!("/cache/kepub/{}?h={}", meta.url_id, kepub_hash));
    if let Some(ref h) = mobi_hash {
        hashes.insert("mobi".to_string(), h.clone());
        urls.insert("mobi".to_string(), format!("/cache/mobi/{}?h={}", meta.url_id, h));
    }
    if let Some(ref h) = pdf_hash {
        hashes.insert("pdf".to_string(), h.clone());
        urls.insert("pdf".to_string(), format!("/cache/pdf/{}?h={}", meta.url_id, h));
    }
    if let Some(ref h) = azw3_hash {
        hashes.insert("azw3".to_string(), h.clone());
        urls.insert("azw3".to_string(), format!("/cache/azw3/{}?h={}", meta.url_id, h));
    }

    let export_ms = start.elapsed().as_millis() as i32;
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    // Get or create a request source
    let source_id = queries::insert_request_source(
        &state.db, false, "/api/v0/epub", "web request",
    ).await?;

    // Extract client tracking info from headers
    let client_id = headers.get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    // Real client IP from X-Forwarded-For (set by nginx) — first hop is the
    // client; we only trust it because nginx overwrites the header. Zero-PII:
    // the IP is used for abuse aggregation (bot_scores), never exposed to admins.
    let client_ip: Option<std::net::IpAddr> = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok());

    // Log the request
    let fic_json = serde_json::to_string(&meta).ok();
    queries::insert_request_log(
        &state.db, source_id, "epub", query, info_request_ms,
        Some(&meta.url_id), fic_json.as_deref(),
        Some(export_ms), Some(&format!("{}.epub", epub_hash)),
        Some(&epub_hash), Some(query),
        client_id.as_deref(), user_agent.as_deref(), client_ip,
    ).await?;

    // Fetch tags and sources for the work
    let tags = queries::get_fic_tags(&state.db, &meta.url_id, 0).await.unwrap_or_default();
    let tags_json = format_tags_json(&tags);
    let sources = queries::get_work_sources(&state.db, work_id).await.unwrap_or_default();
    let sources_json = format_sources_json(&sources);

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "fixits": [],
        "info": info_str,
        "url_id": meta.url_id,
        "work_id": work_id,
        "slug": slug,
        "meta": build_meta_json(&meta, Some(work_id)),
        "tags": tags_json,
        "sources": sources_json,
        "hashes": hashes,
        "urls": urls,
        "epub_url": urls.get("epub"),
        "html_url": urls.get("html"),
        "mobi_url": urls.get("mobi"),
        "pdf_url": urls.get("pdf"),
        "txt_url": urls.get("txt"),
        "md_url": urls.get("md"),
        "azw3_url": urls.get("azw3"),
        "docx_url": urls.get("docx"),
        "fb2_url": urls.get("fb2"),
        "kepub_url": urls.get("kepub"),
        "notes": notes,
    })))
}

/// Look up a cached format hash for a given EPUB hash, if one exists.
async fn cached_format_hash(
    db: &sqlx::PgPool,
    url_id: &str,
    version: i32,
    epub_hash: &str,
    etype_str: &str,
) -> Option<String> {
    let e_input_hash = format!("epub:{epub_hash}");
    queries::find_export_log(db, url_id, version, etype_str, &e_input_hash)
        .await
        .ok()
        .flatten()
        .map(|e| e.export_hash)
}

/// ── Self-healing M2: on-the-fly extraction (helper for epub_handler) ────
///
/// Runs the extraction agent against the snapshot for a structural scrape
/// failure. On success the extraction is recorded (`heal_extractions` +
/// `agent_runs` status 'proposed') and its metadata is returned so the
/// export path can continue; on failure `None` is returned and the caller
/// falls through to the normal failure recording + Err.
async fn attempt_on_the_fly_heal(
    state: &Arc<AppState>,
    meta_source: &str,
    snapshot_path: &Option<String>,
    e: &crate::scrape::ScrapeError,
) -> Option<crate::scrape::FicMetadata> {
    let snapshot_html = load_snapshot_html(state, meta_source, snapshot_path).await;
    // Record the failure FIRST so run_extraction can link to it; the row is
    // the trigger_ref for the agent run.
    let kind = crate::heal::classifier::ErrorKind::from(e);
    let domain = crate::heal::classifier::url_domain(meta_source).unwrap_or_else(|| meta_source.to_string());
    state
        .heal
        .record_failure(meta_source, None, &kind, Some(&e.to_string()), snapshot_path.as_deref())
        .await;

    // Find the failure row we just recorded (fingerprint-stable) to pass to
    // run_extraction. Best-effort: if the lookup fails, bail.
    let fp = crate::heal::classifier::fingerprint(meta_source, &kind, &e.to_string());
    let failures = crate::heal::store::recent_failures(
        &state.db,
        &domain,
        chrono::Utc::now() - chrono::Duration::minutes(5),
    )
    .await
    .unwrap_or_default();
    let failure = failures.iter().find(|f| f.fingerprint == fp).cloned();
    let Some(failure) = failure else {
        tracing::warn!("heal: on-the-fly extraction skipped — no failure row for {meta_source}");
        return None;
    };

    match crate::heal::extract::run_extraction(
        &state.db,
        &state.config,
        &state.http_client,
        &failure,
        snapshot_html.as_deref(),
    )
    .await
    {
        Ok(meta) => Some(meta.to_fic_metadata(meta_source)),
        Err(reason) => {
            tracing::warn!("heal: on-the-fly extraction failed for {meta_source}: {reason}");
            None
        }
    }
}

/// Load snapshot HTML: prefer the captured snapshot file; fall back to a
/// best-effort re-fetch.
async fn load_snapshot_html(
    state: &Arc<AppState>,
    source_url: &str,
    snapshot_path: &Option<String>,
) -> Option<String> {
    if let Some(path) = snapshot_path {
        if let Ok(html) = std::fs::read_to_string(path) {
            return Some(html);
        }
    }
    crate::heal::snapshot::capture_snapshot(&state.http_client, source_url)
        .await
        .and_then(|p| std::fs::read_to_string(p).ok())
}

/// Query parameters for on-demand format conversion
#[derive(Debug, Deserialize)]
pub struct ConvertQuery {
    pub q: Option<String>,
    pub format: Option<String>,
}

/// Whitelist of Calibre-backed formats convertible on demand.
pub fn supported_convert_format(format: &str) -> bool {
    matches!(format, "mobi" | "pdf" | "azw3" | "docx" | "fb2" | "kepub" | "html" | "txt" | "md")
}

/// On-demand Calibre conversion: GET /api/epub/convert?q=<url>&format=<fmt>
/// Supported: mobi, pdf, azw3, docx, fb2, kepub, html, txt, md, rtf
///
/// The initial export (GET /api/epub) only generates fast native formats and
/// returns null for the Calibre formats (mobi/pdf/azw3). Clicking one of those
/// buttons calls this endpoint, which loads the cached body, (re)uses the
/// cached EPUB, runs ebook-convert once, stores the result, and returns the
/// final download URL. Converting the same format twice returns the cached
/// file (export_log lookup).
pub async fn convert_format_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConvertQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    let query = params.q.as_deref().unwrap_or("");
    let format = params.format.as_deref().unwrap_or("").to_lowercase();
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }
    if !supported_convert_format(&format) {
        return Ok(Json(json!({
            "err": -1,
            "msg": "unsupported format (use mobi, pdf, or azw3)",
            "q": query,
        })));
    }

    // ── Tiered rate limit (download tier) ─────────────────────────────
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let real_ip = crate::limiter::client_ip_from_headers(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    let tier = state.rate_limiter.tier_for_path("/api/epub");
    match state
        .rate_limiter
        .check(real_ip, client_id.as_deref(), tier)
        .await
    {
        crate::limiter::TieredRateLimitResult::Wait(secs) => {
            return Err(AppError::RateLimited(secs));
        }
        crate::limiter::TieredRateLimitResult::Allowed => {}
    }

    let etype = match format.as_str() {
        "mobi" => EType::Mobi,
        "pdf" => EType::Pdf,
        "azw3" => EType::Azw3,
        "docx" => EType::Docx,
        "fb2" => EType::Fb2,
        "kepub" => EType::Kepub,
        "html" => EType::Html,
        "txt" => EType::Txt,
        "md" => EType::Md,
        _ => unreachable!(),
    };

    // Find the fic (by URL or by url_id/hash) — reuse get_fic_info for hash
    // lookups, otherwise scrape metadata like the main handler.
    let meta = if query.starts_with("http") {
        let scraper = match state.scraper_registry.find_specific_or_fff(query) {
            Some(s) => s,
            None => {
                return Err(AppError::BadRequest(format!("unsupported URL: {query}")));
            }
        };
        match scraper.lookup(&state.http_client, query).await {
            Ok(m) => m,
            Err(e) => return Err(AppError::ScrapeError(e.to_string())),
        }
    } else {
        let fic = queries::get_fic_info(&state.db, query).await?
            .ok_or_else(|| AppError::NotFound(format!("fic not found: {query}")))?;
        // Rebuild a FicMetadata from the stored fic_info row (same fields
        // the main export handler writes).
        FicMetadata {
            url_id: fic.id.clone(),
            title: fic.title,
            author: fic.author,
            author_url: fic.author_url.unwrap_or_default(),
            author_local_id: fic.author_local_id.unwrap_or_default(),
            chapters: fic.chapters,
            words: fic.words,
            desc: fic.description,
            status: fic.status,
            published: fic.fic_created.timestamp_millis(),
            updated: fic.fic_updated.timestamp_millis(),
            source: fic.source,
            content_hash: fic.content_hash,
            extra_meta: fic.extra_meta,
            raw_extended_meta: fic.raw_extended_meta,
            source_id: fic.source_id.unwrap_or(0),
            author_id: fic.author_id.unwrap_or(0),
        }
    };

    // Compute cache version
    let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;

    // Locate the EPUB (source for conversion). Reuse the same logic as
    // epub_handler: cache first, else body_cache, else scrape.
    let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;

    let epub_hash = if let Some(export_log) = cached {
        export_log.export_hash
    } else {
        // No cached EPUB: scrape + generate (same as the main handler's
        // cache-miss path, trimmed to just EPUB).
        let chapters = if let Some(cached_ch) = crate::body_cache::load_body(&state.config, &meta.url_id) {
            cached_ch
        } else {
            let fetched = match scraper_fetch(&state, query).await {
                Ok(ch) => ch,
                Err(e) => return Err(e),
            };
            fetched
        };
        // Acquire semaphore to prevent duplicate concurrent exports
        let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
        let _permit = sem.acquire().await.map_err(|e| AppError::Internal(e.to_string()))?;
        // Re-check cache after semaphore (double-check pattern)
        let cached_again = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
        let epub_hash = if let Some(export_log) = cached_again {
            export_log.export_hash
        } else {
            let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir)
                .await
                .map_err(|e| AppError::ExportError(e.to_string()))?;
            let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
            cache::disk::move_to_cache(&epub_path, &cache_dest)?;
            queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
            epub_hash
        };
        epub_hash
    };

    let epub_input_hash = format!("epub:{epub_hash}");

    // Check for existing converted format
    if let Ok(Some(entry)) = queries::find_export_log(
        &state.db, &meta.url_id, version, format.as_str(), &epub_input_hash,
    ).await {
        let url = format!("/cache/{}/{}/{}", format.as_str(), meta.url_id, entry.export_hash);
        return Ok(Json(json!({
            "err": 0,
            "q": query,
            "url_id": meta.url_id,
            "format": format.as_str(),
            "hash": entry.export_hash,
            "url": url,
            "cached": true,
        })));
    }

    // Acquire per-format semaphore to prevent duplicate conversions
    let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &etype).await;
    let _permit = sem.acquire().await.map_err(|e| AppError::Internal(e.to_string()))?;

    // Re-check after semaphore
    if let Ok(Some(entry)) = queries::find_export_log(
        &state.db, &meta.url_id, version, format.as_str(), &epub_input_hash,
    ).await {
        let url = format!("/cache/{}/{}/{}", format.as_str(), meta.url_id, entry.export_hash);
        return Ok(Json(json!({
            "err": 0,
            "q": query,
            "url_id": meta.url_id,
            "format": format.as_str(),
            "hash": entry.export_hash,
            "url": url,
            "cached": true,
        })));
    }

    // Locate EPUB cache file path for conversion
    let epub_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
    if !epub_cache_dest.exists() {
        return Err(AppError::NotFound(format!(
            "cached EPUB missing for {} (hash {epub_hash})",
            meta.url_id
        )));
    }

    // Convert via Calibre
    let calibre_container = &state.config.calibre_container;
    let (out_path, out_md5) = export::convert::convert_epub(
        &epub_cache_dest, format.as_str(), calibre_container, &state.config.tmp_dir,
    ).await.map_err(|e| AppError::ExportError(e.to_string()))?;

    let out_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &etype, &meta.url_id, &out_md5);
    cache::disk::move_to_cache(&out_path, &out_cache_dest)?;
    queries::insert_export_log(&state.db, &meta.url_id, version, format.as_str(), &epub_input_hash, &out_md5).await?;

    let elapsed = start.elapsed().as_millis() as i32;
    tracing::info!(
        "lazy convert {} {} in {}ms -> {}",
        meta.url_id, format.as_str(), elapsed, out_md5
    );

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "url_id": meta.url_id,
        "format": format.as_str(),
        "hash": out_md5,
        "url": format!("/cache/{}/{}/{}", format.as_str(), meta.url_id, out_md5),
        "cached": false,
        "elapsed_ms": elapsed,
    })))
}

/// Fetch chapters for a fic, persisting to body cache. Shared by the lazy
/// convert path when no cached EPUB exists yet.
async fn scraper_fetch(
    state: &Arc<AppState>,
    query: &str,
) -> Result<Vec<crate::scrape::Chapter>, AppError> {
    let scraper = state.scraper_registry.find_specific_or_fff(query)
        .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {query}")))?;
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let fetched = match scraper.fetch_chapters(&state.http_client, &meta).await {
        Ok(ch) => ch,
        Err(e) => {
            // Wayback fallback: on blocked/transient failures, try the
            // Internet Archive snapshot; if we get one, use it as a single
            // chapter so the export completes (a later re-scrape without
            // the fallback can refresh it).
            if state.wayback.config.enabled
                && crate::heal::classifier::classify(
                    &crate::heal::classifier::ErrorKind::from(&e),
                    &crate::heal::classifier::url_domain(&meta.source)
                        .unwrap_or_else(|| meta.source.clone()),
                    &e.to_string(),
                ) != crate::heal::classifier::Class::Structural
            {
                if let Some(html) = state.wayback.fetch_snapshot(&state.http_client, query).await {
                    let snapshot = crate::scrape::Chapter {
                        chapter_id: 1,
                        title: meta.title.clone(),
                        content: html,
                    };
                    tracing::warn!(
                        "wayback: using Internet Archive snapshot for {} ({} bytes)",
                        meta.url_id,
                        snapshot.content.len()
                    );
                    vec![snapshot]
                } else {
                    return Err(AppError::ScrapeError(e.to_string()));
                }
            } else {
                return Err(AppError::ScrapeError(e.to_string()));
            }
        }
    };
    let version = crate::body_cache::current_version(&state.config, &meta.url_id);
    if let Err(e) = crate::body_cache::save_body(
        &state.config, &meta.url_id, &fetched, Some(meta.source.clone()), version,
    ) {
        tracing::warn!("body_cache save failed for {}: {e}", meta.url_id);
    }
    crate::body_cache::index_body_text(&state.db, &state.config, &meta.url_id).await;
    Ok(fetched)
}

/// Handle lookup by hash (when q is not a URL but a fic hash ID).
/// Looks up fic_info from the DB and returns cached export URLs if available.
async fn handle_hash_lookup(
    state: &Arc<AppState>,
    hash: &str,
    _info_request_ms: i32,
) -> Result<Json<Value>, AppError> {
    // Look up fic from database
    let fic = queries::get_fic_info(&state.db, hash).await?
        .ok_or_else(|| AppError::NotFound(format!("fic not found: {}", hash)))?;

    // Compute cache version
    let version_bump = queries::get_fic_version_bump(&state.db, &fic.id).await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;

    // Check for cached exports
    let input_hash = fic.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
    let cached = queries::find_export_log(&state.db, &fic.id, version, "epub", &input_hash).await?;

    let mut urls = std::collections::HashMap::new();
    let mut hashes = std::collections::HashMap::new();

    if let Some(export_log) = cached {
        let epub_hash = &export_log.export_hash;
        hashes.insert("epub".to_string(), epub_hash.clone());
        urls.insert("epub".to_string(), format!("/cache/epub/{}?h={}", fic.id, epub_hash));

        for etype_str in &["html", "mobi", "pdf", "txt", "md", "azw3", "docx", "fb2", "kepub"] {
            if let Ok(_etype) = etype_str.parse::<EType>() {
                let e_input_hash = format!("epub:{}", epub_hash);
                if let Ok(Some(entry)) = queries::find_export_log(
                    &state.db, &fic.id, version, etype_str, &e_input_hash,
                ).await {
                    hashes.insert(etype_str.to_string(), entry.export_hash.clone());
                    urls.insert(etype_str.to_string(), format!("/cache/{}/{}?h={}", etype_str, fic.id, entry.export_hash));
                }
            }
        }
    }

    let slug = generate_slug(&fic.title, &fic.id);
    let (info_str, notes) = build_info_string_from_fic(&fic);

    // Look up work_id for this source
    let work_id = queries::get_work_by_source(&state.db, &fic.id)
        .await?
        .map(|w| w.id)
        .unwrap_or(0);

    // Fetch tags and sources for the work
    let tags = queries::get_fic_tags(&state.db, &fic.id, 0).await.unwrap_or_default();
    let tags_json = format_tags_json(&tags);
    let sources = queries::get_work_sources(&state.db, work_id).await.unwrap_or_default();
    let sources_json = format_sources_json(&sources);

    Ok(Json(json!({
        "err": 0,
        "q": &fic.source,
        "fixits": [],
        "info": info_str,
        "url_id": fic.id,
        "work_id": work_id,
        "slug": slug,
        "meta": build_meta_json_from_fic(&fic, Some(work_id)),
        "tags": tags_json,
        "sources": sources_json,
        "hashes": hashes,
        "urls": urls,
        "epub_url": urls.get("epub"),
        "html_url": urls.get("html"),
        "mobi_url": urls.get("mobi"),
        "pdf_url": urls.get("pdf"),
        "txt_url": urls.get("txt"),
        "md_url": urls.get("md"),
        "azw3_url": urls.get("azw3"),
        "docx_url": urls.get("docx"),
        "fb2_url": urls.get("fb2"),
        "kepub_url": urls.get("kepub"),
        "notes": notes,
    })))
}

/// Build info string from a FicInfo DB row
fn build_info_string_from_fic(fic: &crate::db::models::FicInfo) -> (String, Vec<String>) {
    let relative_time = {
        let now = chrono::Utc::now().timestamp_millis();
        let updated_ms = fic.fic_updated.timestamp_millis();
        let diff_ms = now - updated_ms;
        let diff_secs = diff_ms / 1000;
        if diff_secs < 60 {
            "less than a minute ago".to_string()
        } else if diff_secs < 3600 {
            format!("{} minutes ago", diff_secs / 60)
        } else if diff_secs < 86400 {
            format!("{} hours ago", diff_secs / 3600)
        } else {
            format!("{} days ago", diff_secs / 86400)
        }
    };

    let info = format!(
        "{title} by {author}\n{words} words in {chapters} chapters\nStatus: {status}\nUpdated: {date} - {relative}\n",
        title = fic.title,
        author = fic.author,
        words = fic.words,
        chapters = fic.chapters,
        status = fic.status,
        date = fic.fic_updated.format("%Y-%m-%d %H:%M:%S"),
        relative = relative_time,
    );

    (info, Vec::new())
}

/// Build meta JSON from a FicInfo DB row
fn build_meta_json_from_fic(fic: &crate::db::models::FicInfo, work_id: Option<i32>) -> Value {
    json!({
        "id": fic.id,
        "work_id": work_id,
        "title": fic.title,
        "author": fic.author,
        "chapters": fic.chapters,
        "words": fic.words,
        "description": fic.description,
        "status": fic.status,
        "source": fic.source,
        "created": fic.fic_created.to_rfc3339(),
        "updated": fic.fic_updated.to_rfc3339(),
        "extra_meta": fic.extra_meta,
        "raw_extended_meta": fic.raw_extended_meta,
        "author_url": fic.author_url,
        "author_local_id": fic.author_local_id,
        "source_id": fic.source_id,
        "author_id": fic.author_id,
    })
}

/// Format tags from fic_tags query into the JSON structure expected by the frontend.
/// Tags are grouped by tag_type_id into: 1=fandom, 2=character, 3=relationship, 4=freeform, 5=warning, 6=category.
fn format_tags_json(tags: &[(i32, String, i16, i16, bool)]) -> Value {
    let mut grouped: std::collections::HashMap<i32, Vec<Value>> = std::collections::HashMap::new();
    for (id, name, type_id, score, _hidden) in tags {
        grouped.entry(*type_id as i32).or_default().push(json!({
            "name": name,
            "category": type_id,
            "score": score,
            "id": id,
        }));
    }
    json!({
        "fandom": grouped.get(&1).cloned().unwrap_or_default(),
        "character": grouped.get(&2).cloned().unwrap_or_default(),
        "relationship": grouped.get(&3).cloned().unwrap_or_default(),
        "freeform": grouped.get(&4).cloned().unwrap_or_default(),
        "warning": grouped.get(&5).cloned().unwrap_or_default(),
        "category": grouped.get(&6).cloned().unwrap_or_default(),
    })
}

/// Format fic_info sources into JSON for the frontend source chips.
fn format_sources_json(sources: &[crate::db::models::FicInfo]) -> Value {
    let source_list: Vec<Value> = sources.iter().map(|s| {
        // Extract site name from source URL (e.g., "archiveofourown.org" → "AO3")
        let site = if s.source.contains("archiveofourown") {
            "AO3".to_string()
        } else if s.source.contains("fanfiction.net") || s.source.contains("fictionpress") {
            "FFN".to_string()
        } else if s.source.contains("wattpad") {
            "Wattpad".to_string()
        } else {
            s.source.clone()
        };
        json!({
            "url_id": s.id,
            "site": site,
            "source_url": s.source,
            "words": s.words,
            "chapters": s.chapters,
        })
    }).collect();
    Value::Array(source_list)
}

/// Generate a URL-safe slug from title and url_id
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    // Collapse multiple underscores
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}

/// Build the info string shown to users
pub fn build_info_string(meta: &FicMetadata) -> (String, Vec<String>) {
    let relative_time = {
        let now = chrono::Utc::now().timestamp_millis();
        let diff_ms = now - meta.updated;
        let diff_secs = diff_ms / 1000;
        if diff_secs < 60 {
            "less than a minute ago".to_string()
        } else if diff_secs < 3600 {
            format!("{} minutes ago", diff_secs / 60)
        } else if diff_secs < 86400 {
            format!("{} hours ago", diff_secs / 3600)
        } else {
            format!("{} days ago", diff_secs / 86400)
        }
    };

    let updated_str = chrono::DateTime::from_timestamp_millis(meta.updated)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let info = format!(
        "{title} by {author}\n{words} words in {chapters} chapters\nStatus: {status}\nUpdated: {date} - {relative}\n",
        title = meta.title,
        author = meta.author,
        words = meta.words,
        chapters = meta.chapters,
        status = meta.status,
        date = updated_str,
        relative = relative_time,
    );

    (info, Vec::new())
}

/// Build the meta JSON object for API response
pub fn build_meta_json(meta: &FicMetadata, work_id: Option<i32>) -> Value {
    json!({
        "id": meta.url_id,
        "work_id": work_id,
        "title": meta.title,
        "author": meta.author,
        "chapters": meta.chapters,
        "words": meta.words,
        "description": meta.desc,
        "status": meta.status,
        "source": meta.source,
        "created": chrono::DateTime::from_timestamp_millis(meta.published)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default(),
        "updated": chrono::DateTime::from_timestamp_millis(meta.updated)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default(),
        "extra_meta": meta.extra_meta,
        "raw_extended_meta": meta.raw_extended_meta,
        "author_url": meta.author_url,
        "author_local_id": meta.author_local_id,
        "source_id": meta.source_id,
        "author_id": meta.author_id,
    })
}

/// Build metadata-only response for greylisted fics
fn build_metadata_response(
    meta: &FicMetadata,
    notes: &[String],
    _export_version: &i32,
    _cached: Option<&crate::db::models::ExportLog>,
    is_greylisted: bool,
) -> axum::Json<serde_json::Value> {
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, _) = build_info_string(meta);

    let mut notes_vec: Vec<String> = if is_greylisted {
        vec!["This fic is greylisted - download links are not available.".to_string()]
    } else {
        Vec::new()
    };
    notes_vec.extend_from_slice(notes);

    Json(json!({
        "err": 0,
        "fixits": [],
        "info": info_str,
        "url_id": meta.url_id,
        "slug": slug,
        "meta": build_meta_json(meta, None),
        "hashes": {},
        "urls": {},
        "epub_url": null,
        "html_url": null,
        "mobi_url": null,
        "pdf_url": null,
        "notes": notes_vec,
    }))
}

#[cfg(test)]
mod tests {
    use crate::scrape::FicMetadata;
    use chrono::Utc;

    fn make_test_meta() -> FicMetadata {
        FicMetadata {
            url_id: "test123".into(),
            title: "Test Fic".into(),
            author: "Test Author".into(),
            chapters: 10,
            words: 50000,
            desc: "<p>A great story</p>".into(),
            published: 1700000000000,
            updated: 1700000000000,
            status: "complete".into(),
            source: "https://archiveofourown.org/works/123456".into(),
            source_id: 1,
            author_id: 42,
            author_url: "https://archiveofourown.org/users/TestAuthor".into(),
            author_local_id: "123456".into(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        }
    }

    #[test]
    fn test_generate_slug_basic() {
        let slug = super::generate_slug("The Best Story", "abc123");
        assert!(slug.contains("The_Best_Story"));
        assert!(slug.contains("abc123"));
        assert!(!slug.starts_with('_'));
        assert!(!slug.ends_with('_'));
    }

    #[test]
    fn test_generate_slug_special_chars() {
        let slug = super::generate_slug("Hello: World? (Part 1/2)", "xyz789");
        assert!(!slug.contains(':'));
        assert!(!slug.contains('?'));
        assert!(!slug.contains('('));
        assert!(!slug.contains(')'));
        assert!(!slug.contains('/'));
        assert!(slug.contains("Hello_World_Part_1_2"));
    }

    #[test]
    fn test_generate_slug_collapse_underscores() {
        let slug = super::generate_slug("A___B___C", "id1");
        assert_eq!(slug.chars().filter(|&c| c == '_').count(), 2);
    }

    #[test]
    fn test_generate_slug_empty_title() {
        let slug = super::generate_slug("", "id1");
        assert!(slug.contains("id1"));
        assert!(slug.starts_with("id1") || slug.ends_with("id1"));
    }

    #[test]
    fn test_build_info_string() {
        let meta = make_test_meta();
        let (info, notes) = super::build_info_string(&meta);
        assert!(info.contains("Test Fic"));
        assert!(info.contains("Test Author"));
        assert!(info.contains("50000"));
        assert!(info.contains("10"));
        assert!(info.contains("complete"));
        assert!(notes.is_empty());
    }

    #[test]
    fn test_build_meta_json() {
        let meta = make_test_meta();
        let json = super::build_meta_json(&meta, Some(42));
        assert_eq!(json["id"], "test123");
        assert_eq!(json["work_id"], 42);
        assert_eq!(json["title"], "Test Fic");
        assert_eq!(json["author"], "Test Author");
        assert_eq!(json["chapters"], 10);
        assert_eq!(json["words"], 50000);
        assert_eq!(json["status"], "complete");
        assert_eq!(json["source_id"], 1);
        assert_eq!(json["author_id"], 42);
    }

    #[test]
    fn test_build_metadata_response_greylisted() {
        let meta = make_test_meta();
        let resp = super::build_metadata_response(
            &meta,
            &[],
            &1,
            None,
            true,
        );
        let json = resp.0; // Json wrapper
        assert_eq!(json["err"], 0);
        assert!(json["notes"][0].as_str()
            .unwrap_or("")
            .contains("greylisted"));
        assert!(json["urls"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_info_string_word_count_formatting() {
        let meta = FicMetadata {
            words: 1234567,
            ..make_test_meta()
        };
        let (info, _) = super::build_info_string(&meta);
        assert!(info.contains("1234567"));
    }

    #[test]
    fn test_build_info_string_from_fic() {
        use crate::db::models::FicInfo;
        use chrono::TimeZone;

        let fic = FicInfo {
            id: "abc123".into(),
            created: None,
            updated: None,
            title: "Test Fic".into(),
            author: "Test Author".into(),
            author_url: None,
            author_local_id: None,
            chapters: 5,
            words: 25000,
            description: "A description".into(),
            fic_created: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            fic_updated: Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 0).unwrap(),
            status: "complete".into(),
            source: "https://archiveofourown.org/works/123".into(),
            extra_meta: None,
            raw_extended_meta: None,
            source_id: Some(1),
            author_id: Some(42),
            content_hash: None,
            work_id: None,
        };

        let (info, notes) = super::build_info_string_from_fic(&fic);
        assert!(info.contains("Test Fic"));
        assert!(info.contains("Test Author"));
        assert!(info.contains("25000"));
        assert!(info.contains("5"));
        assert!(info.contains("complete"));
        assert!(info.contains("2024-06-15"));
        assert!(notes.is_empty());
    }

    #[test]
    fn test_build_meta_json_from_fic() {
        use crate::db::models::FicInfo;
        use chrono::TimeZone;

        let fic = FicInfo {
            id: "abc123".into(),
            created: None,
            updated: None,
            title: "Test Fic".into(),
            author: "Test Author".into(),
            author_url: Some("https://example.com/author".into()),
            author_local_id: Some("999".into()),
            chapters: 5,
            words: 25000,
            description: "A description".into(),
            fic_created: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            fic_updated: Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 0).unwrap(),
            status: "complete".into(),
            source: "https://archiveofourown.org/works/123".into(),
            extra_meta: None,
            raw_extended_meta: None,
            source_id: Some(1),
            author_id: Some(42),
            content_hash: None,
            work_id: None,
        };

        let json = super::build_meta_json_from_fic(&fic, None);
        assert_eq!(json["id"], "abc123");
        assert_eq!(json["work_id"], serde_json::Value::Null);
        assert_eq!(json["title"], "Test Fic");
        assert_eq!(json["author"], "Test Author");
        assert_eq!(json["chapters"], 5);
        assert_eq!(json["words"], 25000);
        assert_eq!(json["status"], "complete");
        assert_eq!(json["source"], "https://archiveofourown.org/works/123");
        assert_eq!(json["source_id"], 1);
        assert_eq!(json["author_id"], 42);
        assert_eq!(json["author_url"], "https://example.com/author");
        assert_eq!(json["author_local_id"], "999");
    }

    #[test]
    fn test_supported_convert_format_whitelist() {
        // Calibre-backed convertible formats
        assert!(super::supported_convert_format("mobi"));
        assert!(super::supported_convert_format("pdf"));
        assert!(super::supported_convert_format("azw3"));
        assert!(super::supported_convert_format("docx"));
        assert!(super::supported_convert_format("fb2"));
        assert!(super::supported_convert_format("kepub"));
        assert!(super::supported_convert_format("html"));
        assert!(super::supported_convert_format("txt"));
        assert!(super::supported_convert_format("md"));
        // The handler lowercases before calling; the guard itself is
        // case-sensitive by contract.
        assert!(!super::supported_convert_format("MOBI"));
        assert!(!super::supported_convert_format("Pdf"));
        // Native format not lazily convertible
        assert!(!super::supported_convert_format("epub"));
        assert!(!super::supported_convert_format(""));
        assert!(!super::supported_convert_format("mobi;rm -rf /"));
    }
}
