# Implementation Plan: Archive-Only UI, Tag Extraction, Auto-Moderation, Trust System, Quick Download

This document details every feature implemented in the current session (2026-08-30/31). Each section includes architecture, file locations, database changes, API endpoints, environment variables, and edge cases. Written so an LLM or junior dev can re-implement from this spec alone.

---

## Table of Contents

1. [Archive-Only UI Enforcement + Scraper From-HTML Seam](#1-archive-only-ui-enforcement--scraper-from-html-seam)
2. [Tag Extraction + Source Chips + Admin Backfill](#2-tag-extraction--source-chips--admin-backfill)
3. [LLM Auto-Moderation with Grace Period](#3-llm-auto-moderation-with-grace-period)
4. [Configurable Trust System](#4-configurable-trust-system-with-preference-similarity--diminishing-recovery)
5. [Quick Download Button on Search Results](#5-quick-download-button-on-search-results)
6. [Database Migrations](#6-database-migrations)
7. [Environment Variables Reference](#7-environment-variables-reference)

---

## 1. Archive-Only UI Enforcement + Scraper From-HTML Seam

### Goal
Remove the modern UI mode entirely. The archive UI is the only interface. Also expose a `lookup_from_html` trait method so scrapers can parse metadata from pre-fetched HTML (Wayback snapshots, fixtures).

### Files Changed
- `scrapers/src/lib.rs` — trait `SiteScraper` gets `lookup_from_html()` and `fetch_chapters_from_html()` as default methods returning `Ok(vec![])` / `Err(Unsupported)`
- `scrapers/src/sites/ao3.rs` — `lookup()` calls `self.lookup_from_html(html, url)` after fetching
- `scrapers/src/sites/ffnet.rs` — same pattern, plus `fetch_chapters_from_html()` returns `Unsupported`
- `src/routes/export.rs` — Wayback fallback tries `scraper.lookup_from_html(html, query)` on snapshot HTML before falling to M2 agent
- `src/lib/prefs.ts` — `getPref('uiMode')` always returns `'archive'`, `setPref('uiMode')` ignores modern writes
- `src/lib/prefs.ts` — `applyUiParam` ignores `?ui=modern`
- 54 test files — `uiMode` fixtures changed from `'modern'` to `'archive'`
- All 90 Svelte pages — dead `{:else}` modern branches removed (search, works, forum, settings, etc.)
- `src/routes/search/+page.svelte` — archive search form rendered unconditionally
- `src/routes/works/[urlId]/+page.svelte` — archive work view only; restored similar-fics + also-bookmarked sections (were loaded but not rendered in archive mode)
- `src/routes/forum/[categorySlug]/+page.svelte` — added pagination to archive view (was modern-only)
- `src/lib/ui/archive/ArchiveWork.svelte` — renders pre-grouped tags from API response; adds source chips below author

### Architecture: lookup_from_html

```rust
// scrapers/src/lib.rs — default trait methods
#[async_trait]
pub trait SiteScraper {
    async fn lookup(&self, client: &Client, url: &str) -> Result<FicMetadata, ScrapeError>;

    /// Parse metadata from a pre-fetched HTML page. Default returns Unsupported.
    async fn lookup_from_html(&self, _html: &str, _url: &str) -> Result<FicMetadata, ScrapeError> {
        Err(ScrapeError::Unsupported("not implemented".into()))
    }

    async fn fetch_chapters_from_html(&self, _html: &str, _meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        Err(ScrapeError::Unsupported("not implemented".into()))
    }
}
```

AO3 implementation: parses the full work HTML with `scraper::Html` selectors (same selectors as `lookup()` but without fetching).

### Wayback Integration (in export.rs)
```rust
if let Some(html) = &wayback_snapshot {
    if let Ok(m) = scraper.lookup_from_html(html, query).await {
        // Use the wayback-sourced metadata directly
    } else {
        // Parse failed — save for M2 agent
    }
}
```

### Edge Cases
- If the AO3 page is Cloudflare-challenged, the HTML won't have the expected selectors → `lookup_from_html` returns `ScrapeError::ParseError`
- FFN has no full-work view, so `fetch_chapters_from_html` returns `Unsupported`
- The export route tries Wayback first, then falls through to M2 agent only on structural parse failures

---

## 2. Tag Extraction + Source Chips + Admin Backfill

### Goal
Extract fandom/character/relationship/freeform/warning/category tags from AO3 and FFN HTML. Include tags and sources in the API response. Show source chips on work page. Provide admin backfill endpoints for existing works.

### Database Schema
Tags are stored in the existing `tags` table (id, name, tag_type_id) linked via `fic_tags` (url_id, tag_id, score). Tag type IDs:
- 1 = fandom
- 2 = character
- 3 = relationship
- 4 = freeform
- 5 = warning
- 6 = category

Sources come from `fic_info` table linked to works via `work_id`.

### Files Changed
- `scrapers/src/sites/ao3.rs` — new `extract_tags()` implementation
- `scrapers/src/sites/ffnet.rs` — new `extract_tags()` implementation
- `src/routes/export.rs` — adds `format_tags_json()` and `format_sources_json()` helpers; injects `tags` and `sources` into all 3 return paths
- `src/routes/admin.rs` — `admin_backfill_tags()` and `admin_backfill_bodies()` endpoints
- `src/server.rs` — registers `/api/admin/backfill/tags` and `/api/admin/backfill/bodies`
- `frontend/src/lib/api/types.ts` — adds `FicTag`, `FicTags`, `FicSource` interfaces; adds `tags`/`sources` to `ExportResponse`
- `frontend/src/lib/ui/archive/ArchiveWork.svelte` — uses pre-grouped tags from API; adds source chips

### AO3 Tag Extraction

```rust
async fn extract_tags(&self, client: &Client, url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
    let work_id = Self::extract_work_id(url)?;
    let fic_url = format!("{}/works/{work_id}?view_full_work=true", self.base_url());
    let html = client.get(&fic_url).send().await?.text().await?;
    let doc = Html::parse_document(&html);

    let selectors = [
        ("dd.fandom a.tag",           1i32, 5.0, 1.0),  // fandom, first=5
        ("dd.characters a.tag",       2,   10.0, 1.0),   // character, first=10
        ("dd.relationships a.tag",    3,    5.0, 1.0),   // ship, first=5
        ("dd.freeforms a.tag",        4,    0.0, 0.0),   // freeform
        ("dd.warning a.tag",          5,    0.0, 0.0),   // warning
    ];

    let mut tags = Vec::new();
    for (sel_str, type_id, first_score, other_score) in selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            for (i, el) in doc.select(&sel).enumerate() {
                let name = el.text().collect::<String>().trim().to_string();
                if name.is_empty() { continue; }
                let score = if i == 0 { first_score } else { other_score };
                tags.push(ExtractedTag { name, tag_type_id: type_id, score });
            }
        }
    }
    Ok(tags)
}
```

### FFN Tag Extraction
FFN shows fandom in breadcrumb links (`a[href*='/fanfiction/book/']`) and genre in `select#genre option[selected]`. Characters/relationships are NOT available.

### format_tags_json (in export.rs)
Groups fic_tags by type_id into a JSON object:
```json
{
  "fandom": [{"name": "Harry Potter", "category": 1, "score": 5, "id": 42}],
  "character": [...],
  "relationship": [...],
  "freeform": [...],
  "warning": [...],
  "category": [...]
}
```

### format_sources_json (in export.rs)
Extracts site names from source URLs (archiveofourown → "AO3", fanfiction.net → "FFN"):
```json
[
  {"url_id": "ao3_12345", "site": "AO3", "source_url": "https://archiveofourown.org/works/12345", "words": 52341, "chapters": 21}
]
```

### Source Chips (ArchiveWork.svelte)
Rendered below the author byline when `sources.length > 1`. Shows site name as a link to the source URL with word/chapter counts in the title tooltip.

### Admin Backfill Endpoints
- `POST /api/admin/backfill/tags` — finds works with fic_info but no fic_tags, re-scrapes to extract tags (100 per call, rate-limited)
- `POST /api/admin/backfill/bodies` — finds works without cached body content, re-scrapes chapters (100 per call)

Both require `role >= 10` (admin).

### Edge Cases
- AO3 might return an error page if Cloudflare blocks the fetch → returns `ScrapeError::NotFound`
- FFN breadcrumb may not have fandom for some stories → returns empty vec (not error)
- Tags with score 0 are still stored (for search filtering) but not shown in UI
- Source detection is URL-based: `archiveofourown.org` → AO3, `fanfiction.net` → FFN

---

## 3. LLM Auto-Moderation with Grace Period

### Goal
After scraping a fic, classify its body text via LLM. If classified as "noise" (not actual fanfiction) with high confidence, flag it for curator review with a 72-hour grace period before auto-deletion.

### Architecture
```
Scrape → Save body → Spawn LLM scan (non-blocking) → If noise >= 0.8 conf:
  → content_scan entry with deletion_scheduled_at = now + 72h
  → modlog: content_scan_flag
  → Curator reviews (confirm/dismiss) OR auto-delete after 72h
```

### Database Changes (Migration 070)
```sql
ALTER TABLE public.content_scan
  ADD COLUMN IF NOT EXISTS deletion_scheduled_at timestamp with time zone DEFAULT NULL;

CREATE INDEX IF NOT EXISTS idx_content_scan_deletion_pending
  ON public.content_scan (deletion_scheduled_at)
  WHERE review_status = 'pending' AND classification = 'noise'
    AND deletion_scheduled_at IS NOT NULL;
```

### Files Changed
- `migrations/070_auto_moderation.sql` — adds `deletion_scheduled_at` column + index
- `src/services/content_scan.rs` — adds `upsert_scan_with_deletion()`, `scan_single_fic()`, `process_expired_deletions()`
- `src/routes/export.rs` — spawns `scan_single_fic()` as background task after body save
- `src/routes/admin.rs` — updates `review_content_scan()` to handle dismissal (clears deletion_scheduled_at) and curator access (role >= 5)
- `src/server.rs` — adds auto-delete cron (hourly)

### Key Functions

```rust
// src/services/content_scan.rs

const NOISE_CONFIDENCE_THRESHOLD: f32 = 0.8;
const DELETION_GRACE_PERIOD_HOURS: i64 = 72;

/// Called from export route after body is saved (non-blocking via tokio::spawn)
pub async fn scan_single_fic(db, ollama, config, url_id) {
    let chapters = body_cache::load_body(config, url_id)?;
    let blob = BodyBlob { url_id, chapters, ... };
    let row = scan_blob(ollama, &config.ollama_chat_model, &blob).await;
    upsert_scan_with_deletion(db, &row, None, None).await;
}

/// Sets deletion_scheduled_at for high-confidence noise
pub async fn upsert_scan_with_deletion(db, row, modlog_actor, modlog_actor_name) {
    let deletion_scheduled = if row.classification == CLASS_NOISE
        && row.confidence >= NOISE_CONFIDENCE_THRESHOLD
    {
        Some(Utc::now() + Duration::hours(DELETION_GRACE_PERIOD_HOURS))
    } else { None };

    // INSERT/UPDATE with deletion_scheduled_at
    // Log to modlog if flagged
}

/// Hourly cron: delete expired pending noise entries
pub async fn process_expired_deletions(db, config) -> usize {
    // SELECT url_id WHERE review_status='pending' AND classification='noise'
    //   AND deletion_scheduled_at <= NOW() LIMIT 50
    // For each: delete_body(), set review_status='confirmed', log to modlog
}
```

### Export Route Hook (after body save)
```rust
// tokio::spawn — non-blocking, fic available immediately
{
    let db = state.db.clone();
    let ollama = state.ollama.clone();
    let config = state.config.clone();
    let url_id = meta.url_id.clone();
    tokio::spawn(async move {
        scan_single_fic(&db, &ollama, &config, &url_id).await;
    });
}
```

### Curator Review
- `POST /api/content-scan/{url_id}/review` with `{"action": "confirmed"}` or `{"action": "dismissed"}`
- **Confirm**: delete body cache, clear deletion_scheduled_at, log to modlog
- **Dismiss**: clear deletion_scheduled_at only (keep the fic)
- Available to curators (role >= 5) and admins (role >= 10)

### Auto-Delete Cron
Runs every hour via `tokio::spawn` in server.rs:
```rust
loop {
    let deleted = process_expired_deletions(&state.db, &state.config).await;
    if deleted > 0 { tracing::info!("auto-delete cron: {deleted} expired noise fics removed"); }
    tokio::time::sleep(Duration::from_secs(3600)).await;
}
```

### Edge Cases
- LLM failures return `unclear` classification — fic is NOT flagged
- If `conf < 0.8`, fic is stored as noise but NOT flagged for deletion (manual review only)
- If a curator dismisses, the fic is safe even if it was previously flagged
- The cron only deletes body cache, not the fic_info or works records
- `delete_body` removes the JSON blob from disk, not the database entry

---

## 4. Configurable Trust System with Preference Similarity + Diminishing Recovery

### Goal
Add configurable instance settings for:
1. Preference-similarity trust boost (ON by default)
2. Diminishing recovery after each trust loss event

### Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `TRUST_PREFERENCE_SIMILARITY` | `true` | Enable preference-similarity boost |
| `TRUST_SIMILARITY_BOOST` | `10` | Base boost for aligned preferences (thousandths) |
| `TRUST_RECOVERY_DECAY` | `0.7` | Recovery multiplier after each loss (0.0-1.0) |

### Database Changes (Migration 071)
```sql
ALTER TABLE public.users
  ADD COLUMN IF NOT EXISTS trust_loss_count integer DEFAULT 0 NOT NULL;

CREATE INDEX IF NOT EXISTS idx_users_trust_loss
  ON public.users (trust_loss_count) WHERE trust_loss_count > 0;
```

### Files Changed
- `migrations/071_trust_preferences.sql` — adds `trust_loss_count` column
- `src/config.rs` — adds 3 new Config fields + env var parsing
- `src/services/trust.rs` — modifies `set_trust_level()`, `run_trust_promotion()`, adds `preference_similarity_boost()`
- `src/bin/weekly_digest.rs` — passes config to `run_trust_promotion()`
- `src/bin/trust_promote.rs` — passes config to `run_trust_promotion()`
- `src/server.rs` — passes config to `run_trust_promotion()`

### Diminishing Recovery

When trust is demoted, `trust_loss_count` increments:
```rust
// In set_trust_level():
if to_level < from_level {
    sqlx::query("UPDATE users SET trust_level=$1, ..., trust_loss_count = trust_loss_count + 1 WHERE id=$3")
}
```

During promotion, metrics are reduced by `decay^loss_count`:
```rust
// In run_trust_promotion():
let loss_count: i32 = sqlx::query_scalar("SELECT COALESCE(trust_loss_count, 0) FROM users WHERE id=$1")
    .bind(uid).fetch_one(db).await.unwrap_or(Some(0)).unwrap_or(0);

let decay_factor = if loss_count > 0 {
    config.trust_recovery_decay.powi(loss_count)  // 0.7^1=0.7, 0.7^2=0.49, 0.7^3=0.343
} else { 1.0 };

let adjusted_m = TlMetrics {
    works_entered: (m.works_entered as f64 * decay_factor) as i64,
    works_read: (m.works_read as f64 * decay_factor) as i64,
    words_read: (m.words_read as f64 * decay_factor) as i64,
    days_active_30: (m.days_active_30 as f64 * decay_factor) as i64,
    forum_posts: (m.forum_posts as f64 * decay_factor) as i64,
    reports_received_30d: m.reports_received_30d,  // spam penalties NOT decayed
    ..m.clone()
};
```

### Preference Similarity Boost

When a user bookmarks/rates a work, compare their tag preferences with the work's tags using Jaccard similarity:

```rust
pub async fn preference_similarity_boost(db, config, user_id, work_url_id) -> i64 {
    if !config.trust_preference_similarity_enabled { return 0; }

    // User's most-voted tags (from fic_tag_votes, value > 0)
    let user_tags = sqlx::query_as("SELECT t.name FROM fic_tag_votes v JOIN tags t ON t.id = v.tag_id WHERE v.user_id = $1 AND v.value > 0 ORDER BY v.value DESC LIMIT 20")
        .bind(user_id).fetch_all(db).await?;

    // Work's tags (from fic_tags)
    let work_tags = sqlx::query_as("SELECT t.name FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id WHERE ft.url_id = $1 ORDER BY ft.score DESC LIMIT 20")
        .bind(work_url_id).fetch_all(db).await?;

    // Jaccard similarity
    let intersection = user_set.intersection(&work_set).count();
    let union = user_set.union(&work_set).count();
    let similarity = intersection as f64 / union as f64;

    if similarity < 0.2 { return 0; }
    (similarity * config.trust_similarity_boost as f64) as i64
}
```

### Trust Loss Events
The existing `set_trust_level()` now increments `trust_loss_count` on any demotion (to_level < from_level). This includes:
- Automatic spam revocation
- Manual admin demotion
- All other trust-level reductions

### Edge Cases
- `trust_loss_count` only increments on demotion, never on promotion or lateral move
- A user with 0 losses has `decay_factor = 1.0` (no penalty)
- `reports_received_30d` is NOT decayed — spam penalties are absolute
- The preference boost is 0 if either user or work has no tags
- The boost is best-effort — DB errors return 0 silently
- Config defaults: ON, boost=10 (0.01 trust per unit), decay=0.7

---

## 5. Quick Download Button on Search Results

### Goal
Add a download button to each WorkBlurb (search result card) that downloads in the user's preferred format without navigating to the work page.

### Files Changed
- `frontend/src/lib/ui/archive/WorkBlurb.svelte` — adds download button + `handleQuickDownload()` function + spinner CSS

### Implementation

```svelte
<script>
  import { fetchExport } from '$lib/api/client';
  import { getPref } from '$lib/prefs';

  let downloading = $state(false);
  const preferredFormat = $derived(getPref('defaultFormat') || 'epub');

  async function handleQuickDownload() {
    if (!fic.url_id || downloading) return;
    downloading = true;
    try {
      const res = await fetchExport(fic.url_id);
      if (res.err !== 0) { alert(res.msg || 'Download failed.'); return; }
      const formatKey = `${preferredFormat}_url` as keyof typeof res;
      const formatUrl = res[formatKey] as string | null | undefined;
      if (formatUrl) {
        window.open(formatUrl, '_blank');
      } else if (res.epub_url) {
        window.open(res.epub_url, '_blank');  // fallback to EPUB
      } else {
        alert('This format is not available yet.');
      }
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Download failed.');
    } finally {
      downloading = false;
    }
  }
</script>

<!-- Button in stats section -->
<div class="blurb-actions">
  <button class="archive-btn blurb-download-btn" disabled={downloading} onclick={handleQuickDownload}>
    {#if downloading}
      <span class="spinner-inline"></span> Downloading…
    {:else}
      📥 Download {preferredFormat.toUpperCase()}
    {/if}
  </button>
</div>
```

### Edge Cases
- If preferred format isn't cached yet, falls back to EPUB
- If no format available at all, shows alert
- Button shows loading spinner during fetch
- Prevents double-clicks while downloading

---

## 6. Database Migrations

| Migration | Purpose |
|-----------|---------|
| `070_auto_moderation.sql` | Adds `deletion_scheduled_at` to content_scan, partial index for auto-delete cron |
| `071_trust_preferences.sql` | Adds `trust_loss_count` to users, index for trust promotion queries |

Apply with: `cargo sqlx migrate run`

---

## 7. Environment Variables Reference

| Variable | Default | Feature |
|----------|---------|---------|
| `TRUST_PREFERENCE_SIMILARITY` | `true` | Enable preference-similarity trust boost |
| `TRUST_SIMILARITY_BOOST` | `10` | Base boost per Jaccard similarity unit |
| `TRUST_RECOVERY_DECAY` | `0.7` | Trust recovery multiplier per loss event |
| `BODY_CACHE_DIR` | `/public/literature/fichub/bodies` | Where cached fic bodies are stored |
| `OLLAMA_URL` | `http://127.0.0.1:11434` | Ollama endpoint for LLM classification |
| `OLLAMA_CHAT_MODEL` | `lfm2.5:8b` | Model used for content scan |

---

## Summary of Commits

| Commit | Feature | Lines Changed |
|--------|---------|---------------|
| `1a8f7a9` | Archive-only UI + scraper from-HTML seam | 54 files, +518/-2063 |
| `3edf287` | Tag extraction, source chips, admin backfill | 7 files, +457/-1 |
| `31009ef` | LLM auto-moderation with grace period | 5 files, +223/-4 |
| `bd8ed17` | Trust system with preference similarity | 6 files, +187/-16 |
| `bd89880` | Quick download button on WorkBlurb | 1 file, +76 |
