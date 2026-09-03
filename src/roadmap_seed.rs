//! Roadmap consensus seeding — the program-scope feature list.
//!
//! `seed_roadmap_features(pool, ollama)` is the shared core used by both the
//! `seed-roadmap` bin (`src/bin/seed_roadmap.rs`) and the DB-gated idempotency
//! test (`tests/consensus_seed_api.rs`). It embeds each feature text via
//! Ollama (nomic-embed-text, 768-d) and upserts clusters keyed by
//! `representative_text`, so re-running is a no-op against the 134 clusters
//! already present from earlier sessions.
//!
//! Status semantics (rendered as badges in `/admin/consensus`):
//!   * `shipped`  — already implemented (branch exists / shipped)
//!   * `open`     — proposed, vote on it in the arena
//!   * `rejected` — cut from the list (reason is part of the text)
//!   * `deferred` — parked, not scheduled

use sqlx::PgPool;

use crate::services::ollama::{OllamaClient, OllamaError};

/// One seeded feature: the text shown in the arena/leaderboard and whether it
/// is already shipped (status 'shipped') or still open for votes.
pub struct SeedFeature {
    pub text: &'static str,
    pub shipped: bool,
}

pub const SHIPPED_FEATURES: &[&str] = &[
    "Send to Kindle: email an EPUB of any fic to your Kindle address",
    "Strict genre filtering: hard-scoped tag/domain exclusions in search",
    "Offline PWA: installable app shell that works without connectivity",
    "Boolean search: AND/OR/NOT operators, quoted phrases and field scopes",
    "Auto-tagger: zero-shot tag classification into fandom/character/ship/freeform",
    "User data export: one-click ZIP of all personal data (bookmarks, ratings, prefs)",
    "Nav E2E tests: Playwright coverage of the top navigation flows",
    "Feedback rework: constructive-only feedback, 5-star ratings and in-depth reviews",
    "Web reader v1: typography prefs, chapter nav, progress sync, keyboard shortcuts",
    "Follow fic/author + update feed: follow button and a digest of updates",
    "Per-fic recommendation anchors: 'readers also bookmarked' co-occurrence recs",
    "Roadmap consensus engine: arena voting with Elo-ranked feature leaderboard",
    "Typo tolerance: fuzzy search fallbacks and suggestion correction",
    "Shadowban UI: admin panel to shadowban/unshadowban bot-like clients",
    "PoW challenge: proof-of-work gate for anonymous download bursts",
    "Command center: admin dashboard with realtime platform pulse",
    "Cache headers: long-lived HTTP caching for static export assets",
    "Bot-scorer timer: periodic job that recomputes per-client bot scores",
    "Failed-auth logging: track credential-stuffing attempts per client",
    "Backfill scores: retroactively compute tag scores for pre-scoring content",
];

pub const PROPOSED_FEATURES: &[&str] = &[
    "Wayback Machine fallback: fetch fics from web.archive.org when the origin is down",
    "Ingestion relay: queue URLs for background collection and metadata resolution",
    "PWA offline: full offline reading of downloaded fics in the app shell",
    "Auto-tagger: zero-shot tag classification into fandom/character/ship/freeform",
    "Download queue: batch multiple fics and download them as one job",
    "Send to Kindle: email an EPUB of any fic to your Kindle address",
    "Strict genre filtering: hard-scoped tag/domain exclusions in search",
    "Blind Date with a Fic: random discovery hiding title/fandom until reveal",
    "Hide read/bookmarked: filter search results to exclude fics you've read or saved",
    "Default prefs: per-user default typography and download settings",
    "Search-my-library: full-text search across your bookmarks and shelves",
    "Chaos tests: fault-injection tests for scraper and worker failure modes",
    "ODT/DOCX export: additional export formats (rejected — EPUB covers offline needs)",
    "Gzipped HTML export: pre-compressed HTML responses (rejected — negligible gain)",
    "NLLB translation: in-app machine translation via NLLB models (rejected — quality/GPU)",
    "Auto content warnings: model-generated trigger warnings (rejected — accuracy risk)",
    "Bulk DB mirroring: replicate the archive DB to off-site storage (rejected — complexity)",
    "Chapter-level embeddings: embed each chapter for semantic search (deferred — infra)",
    "Embedding cross-platform dedup: dedupe fics across AO3/FFN via embeddings (deferred)",
    "Import from other instances: pull fics from other FicNexus installs (deferred)",
    "RRF hybrid search: reciprocal rank fusion of vector + keyword search",
    "Vibe toggle: mood-based discovery mode for recommendations",
    "FFNet chapter-check refresh: detect new chapters on fanfiction.net",
    "ts_headline highlighting: highlighted search snippets in results",
    "Follow fic/author + update feed: follow button and a digest of updates",
    "Web reader v1: typography prefs, chapter nav, progress sync, keyboard shortcuts",
    "End-of-fic Next Up: series-next, then recs, then top community suggestion",
    "Readers also bookmarked: per-fic 'readers also bookmarked' co-occurrence recs",
    "Reading lists/bundles: curated shareable collections of fics",
    "Quotes/highlights: save passages from fics while reading",
    "Shelves/status UI: visible shelves and reading-status management UI",
    "RSS/Atom feeds: subscribe to fic and author update feeds",
    "Series & author pages: dedicated pages for series and author bibliographies",
    "Ingestion relay: queue URLs for background collection and metadata resolution",
    "Auto-tagger + metadata enrichment: propose tags from summary+first chapter into canonical vocabulary, detect series ('Book 2 of'), generate blurbs, backfill score=0 rows (curator queue, confidence-gated)",
    "Ask-the-archive: NL query → existing advanced-search params (tags, main_char_attr, bounds) as JSON + embedding similarity over summaries, fused via RRF; personalized with your bookmark tags",
    "Comment moderation triage: classify comments (rule-breaking/toxic/non-constructive/spam/fine) on post into the admin moderation queue with reason + confidence; curators decide (never auto-hide); constructive comments feed reputation",
    "On-demand translation: language detection; eager metadata translation, lazy chapter translation (queued + cached); ML draft → human post-edit published through the shipped chapter_translations tables",
    "Fic Requests: prompt board for fic requests — works-only answers (direct or via URL ingest), community up/down votes ranking fit, requester accept; auto-seeded candidates via search parser (ask-the-archive NL→filter reuse); accepted answers feed the recommender",
    "Kudos: one-click anonymous appreciation on a fic (no text, one per user; guest counts separate) — the simplest positive engagement signal, distinct from 5-star reviews",
    "Pseuds: multiple writing identities per user (each with its own author page + bookmarks, all under one account) — pairs with the existing 'propose merging a pseudonym' path",
    "Orphaning: user-initiated 'orphan my work/pseud/series' moves content to a site orphan_account and frees the username — nothing deleted, content survives as ghost",
    "Subscriptions: opt-in per-work/per-author/per-series update digests (email or RSS), distinct from the follow feed",
    "Gift exchanges / fests: structured prompt-meme + gift-exchange events (signup with offers/requests, tag-set restrictions, matching, anonymous reveals)",
    "User skins + work skins: custom CSS theming — users theme the whole site, authors theme how their fic renders",
    "User-level mute/block: hide a user's content site-wide (not just in the forum), with mute limiting how much of their content you see",
    "Username/email history (admin): track past usernames + emails per user for identity forensics (ban evasion, sockpuppets)",
    "Reading history: per-user timeline of what/when you read, with aggregate reading stats (extends the reading-stats dashboard)",
    "Abuse report queue: dedicated admin surface for user-submitted abuse reports (distinct from comment/forum moderation)",
    "Moderated works gate: admins can route new works behind a manual approval queue before they go public",
    "Fannish next-of-kin: designate a trusted user who can take over or delete your works if you disappear",
    "Tag sets / tag-set nominations: structured tag collections for community events + 'nominate your tags' drives (pairs with tag-wrangling)",
    "Fic-level moods: classify each work's tone (Neutral/Funny/Shocky/Flirty/Dramatic/Hurty/Bondy) from content signals (genre ratios, description tone, review sentiment, tag mix) — feed mood-similarity recs + a search facet ('rec me something dramatic'). Adapted from Zeks/flipper's mood-adjusted calculator",
    "Author recommendations: author feature vectors (genre mix, mood distribution, fandom spread, popularity band) → 'authors like X' / 'authors who wrote fic you kudos'd'; new 'Authors you might like' panel + author_graph rec strategy",
    "Rarity-tier weighted rec strategy: per-author overlap ratio + matches + sigma, weighted by rarity tier (unique 0.2×matches, rare 0.05×, uncommon 0.005×, common 1) — explainable recs + list-manipulation detection (Zeks/flipper weighted calculator)",
    "Audience-genre inference: compute each fic's genre profile of its kudos/bookmark graph ('funny to humor-lovers'), one SQL, no ML — powers 'recommend me funny fic' + per-fic audience tastes (Zeks/flipper genre_detector.sql)",
    "Explorer / size / popularity bands: first-class search + rec filters by popularity band (barely-known/relatively-unknown/popular) and size class (small ≤20k/medium ≤100k/big ≤400k/huge); 'obscure gems' exploration (Zeks/flipper fav_list_analysis)",
    "Tracked-message bot state machines: Discord bot persists per-message state (recommendation list, roll, review, fic details, similarity list, help page, delete confirmation) so commands mutate the message in place (Zeks/flipper tracked-messages pattern)",
    "Suspicious-author detection (!sus): flag authors whose fav-list looks gamed — anomalous overlap, ratio outliers, suspicious vote patterns; surface on author pages + Discord",
    "Per-author profile stats: wordcount/size distribution, fandom diversity, crossover ratio, mood uniformity, popularity band, 'explorer' tendency — powers author pages + !stats + author-recs signal",
];

/// Build the full seed list: shipped features (status 'shipped') followed by
/// proposed features (status 'open'). Duplicates across the two lists are
/// intentional (e.g. Send-to-Kindle is shipped AND still proposed) — the
/// upsert keyed on representative_text collapses them, keeping the shipped
/// status.
pub fn seed_list() -> Vec<SeedFeature> {
    let mut out: Vec<SeedFeature> = SHIPPED_FEATURES
        .iter()
        .map(|text| SeedFeature { text, shipped: true })
        .collect();
    out.extend(
        PROPOSED_FEATURES
            .iter()
            .map(|text| SeedFeature { text, shipped: false }),
    );
    out
}

/// Format a 768-d vector as the `'[...]'::vector` literal sqlx binds need.
pub fn embedding_literal(emb: &[f32]) -> String {
    format!(
        "[{}]",
        emb.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// The set of cluster ids whose representative_text the seed list mentions —
/// used by the idempotency test to scope cleanup.
pub async fn seeded_cluster_ids(pool: &PgPool) -> Result<Vec<i32>, sqlx::Error> {
    let texts: Vec<String> = seed_list()
        .iter()
        .map(|f| f.text.to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let rows: Vec<(i32,)> =
        sqlx::query_as("SELECT id FROM feature_clusters WHERE representative_text = ANY($1)")
            .bind(&texts)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// Embed and upsert every feature in the seed list.
///
/// Idempotency: matching is by `representative_text` (exact), so the second
/// run does not duplicate the first. When Ollama is down the whole run fails
/// fast (this is a one-shot admin bin — silent partial seeding would be worse
/// than a clear error).
///
/// Returns per-feature outcomes for the bin's report:
///   (text, action, status)
/// where action ∈ {inserted, updated, unchanged}.
pub async fn seed_roadmap_features(
    pool: &PgPool,
    ollama: &OllamaClient,
) -> Result<Vec<(String, String, String)>, OllamaError> {
    let mut report: Vec<(String, String, String)> = Vec::new();

    for feature in seed_list() {
        let embedding = ollama
            .embed(feature.text)
            .await
            .map_err(|e| OllamaError(format!("embedding '{}': {e}", feature.text)))?;
        if embedding.len() != 768 {
            return Err(OllamaError(format!(
                "unexpected embedding dimension {} for '{}' (want 768)",
                embedding.len(),
                feature.text
            )));
        }
        let emb_sql = embedding_literal(&embedding);
        let status = if feature.shipped { "shipped" } else { "open" };

        let row: Option<(i32, String)> = sqlx::query_as(
            "SELECT id, status FROM feature_clusters WHERE representative_text = $1",
        )
        .bind(feature.text)
        .fetch_optional(pool)
        .await
        .map_err(|e| OllamaError(format!("lookup failed: {e}")))?;

        let action = match row {
            // Existing cluster: refresh the embedding + status. A seeded
            // 'shipped' cluster that a user later re-proposes as 'open' stays
            // shipped (the feature is still shipped); an 'open' cluster that
            // the seed marks shipped gets upgraded. Never downgrade a shipped
            // cluster back to open.
            Some((id, current)) if current == status => {
                sqlx::query(
                    "UPDATE feature_clusters SET embedding = $1::vector WHERE id = $2",
                )
                .bind(&emb_sql)
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| OllamaError(format!("update failed: {e}")))?;
                "unchanged".to_string()
            }
            Some((id, _)) => {
                // Never downgrade a cluster the seed marks shipped (the
                // feature really is shipped); always upgrade open → shipped
                // when the seed list marks it shipped. ('shipped' wins over
                // 'open' because 'open' here means "still proposed" — a
                // shipped feature is no longer open for votes.)
                let effective = if feature.shipped { "shipped" } else { "open" };
                sqlx::query(
                    "UPDATE feature_clusters SET embedding = $1::vector, status = $2 WHERE id = $3",
                )
                .bind(&emb_sql)
                .bind(effective)
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| OllamaError(format!("update failed: {e}")))?;
                "updated".to_string()
            }
            None => {
                sqlx::query(
                    "INSERT INTO feature_clusters (representative_text, embedding, status)
                     VALUES ($1, $2::vector, $3)",
                )
                .bind(feature.text)
                .bind(&emb_sql)
                .bind(status)
                .execute(pool)
                .await
                .map_err(|e| OllamaError(format!("insert failed: {e}")))?;
                "inserted".to_string()
            }
        };

        report.push((feature.text.to_string(), action, status.to_string()));
    }

    Ok(report)
}
