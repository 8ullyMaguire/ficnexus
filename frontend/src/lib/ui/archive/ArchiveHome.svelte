<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { search, defaultFilters, type SearchResult } from '$lib/api/search';
  import { getTrending, getReadingHistory } from '$lib/api/social';
  import type { TrendingItem } from '$lib/api/social-types';
  import type { ReadingHistoryEntry } from '$lib/api/social-types';
  import { fetchPersonalRecs } from '$lib/api/recommendations';
  import type { PersonalRecsResponse } from '$lib/api/recommendations';
  import { fetchEmbeddingRecs } from '$lib/api/embedding-recs';
  import type { EmbeddingRecsResponse } from '$lib/api/embedding-recs';
  import type { RecResult } from '$lib/api/types';
  import { auth } from '$lib/stores/auth.svelte';
  import WorkBlurb from './WorkBlurb.svelte';
  import { relativeTime } from './rating.js';
  import { isFicUrl } from '$lib/util';

  // ── Compact download input ─────────────────────────────────────────
  let dlUrl = $state('');
  let dlLoading = $state(false);

  async function handleDownload() {
    if (!dlUrl.trim()) return;
    dlLoading = true;
    try {
      const v = dlUrl.trim();
      if (isFicUrl(v)) {
        // URL mode: existing flow
        sessionStorage.setItem('fichub_dl_url', v);
      } else {
        // Query mode: "title by author" — DownloadTab will hit /api/find-fic
        sessionStorage.setItem('fichub_dl_query', v);
      }
      goto('/download');
    } finally {
      dlLoading = false;
    }
  }

  function onDlKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleDownload();
  }

  // ── Recent Works (from search API, sorted by updated) ──────────────
  let recentItems = $state<SearchResult[]>([]);
  let recentLoading = $state(true);
  let recentError = $state('');

  // ── Trending this week ─────────────────────────────────────────────
  let trendingItems = $state<TrendingItem[]>([]);
  let trendingLoading = $state(true);
  let trendingError = $state('');

  // ── Personal recommendations ───────────────────────────────────────
  let recs = $state<RecResult[]>([]);
  let recsEnough = $state(false);
  let recsLoading = $state(true);
  let recsError = $state('');

  // ── Continue reading (reading history, signed-in only) ─────────────
  let history = $state<ReadingHistoryEntry[]>([]);
  let historyLoading = $state(false);

  // ── Recommended for You (embeddings cosine recs; hidden when the
  //    backend reports not enough embedded data) ───────────────────────
  let embRecs = $state<RecResult[]>([]);
  let embLoading = $state(true);

  onMount(async () => {
    // Pref-aware personal: only fetch when logged-in and not opted-out (default ON, keep shadow-run)
    let doPersonal = true;
    try {
      await auth.init();
      if (!auth.isLoggedIn) doPersonal = false;
      else {
        try {
          const pr = await fetch('/api/me/prefs', { credentials: 'include' });
          if (pr.ok) {
            const prefs = (await pr.json()) as Array<{ key: string; value: string }>;
            const found = prefs.find((p) => p.key === 'recs.personalized');
            if (found && found.value === 'false') doPersonal = false;
          }
        } catch { /* default ON */ }
      }
    } catch { /* ignore */ }
    const recsPromise: Promise<PersonalRecsResponse> = doPersonal
      ? fetchPersonalRecs()
      : Promise.reject(new Error('opted-out'));
    if (!doPersonal) fetchPersonalRecs().catch(() => {}); // shadow-run
    // Fetch all three sections in parallel (recs promise is gated by pref)
    const [recentRes, trendRes, recsRes] = await Promise.allSettled([
      search({ ...defaultFilters(), sort: 'updated', per_page: 10 }),
      getTrending(7, 10),
      recsPromise,
    ]);

    // Recent
    if (recentRes.status === 'fulfilled' && recentRes.value) {
      recentItems = recentRes.value.results ?? [];
    } else {
      recentError = 'Could not load recent works.';
    }
    recentLoading = false;

    // Trending
    if (trendRes.status === 'fulfilled') {
      trendingItems = (trendRes.value as { err: number; trending: TrendingItem[] }).trending ?? [];
    } else {
      trendingError = 'Could not load trending fics.';
    }
    trendingLoading = false;

    // Personal recs
    if (recsRes.status === 'fulfilled') {
      const r = recsRes.value as PersonalRecsResponse;
      recsEnough = r.enough_data && r.recs.length > 0;
      recs = r.recs ?? [];
    } else {
      recsError = 'error';
    }
    recsLoading = false;

    // Continue reading — reuses the auth state already resolved above.
    if (auth.isLoggedIn) {
      historyLoading = true;
      try {
        const h = await getReadingHistory(3, 0);
        history = (h?.history ?? []).slice(0, 3);
      } catch {
        history = [];
      } finally {
        historyLoading = false;
      }
    }

    // Recommended for You — embeddings recs. Degrades silently: when the
    // backend reports not enough embedded works (or the call fails), the
    // section just never renders. Anonymous users get the same empty shape.
    try {
      const e: EmbeddingRecsResponse = await fetchEmbeddingRecs(6);
      embRecs = e.enough_data ? (e.recs ?? []) : [];
    } catch {
      embRecs = [];
    } finally {
      embLoading = false;
    }
  });
</script>

  <div class="archive-home">
  <!-- Prominent paste-URL bar (above the fold) -->
  <div class="dl-bar dl-bar--prominent">
    <form class="dl-form" onsubmit={(e) => { e.preventDefault(); handleDownload(); }}>
      <label class="dl-label" for="archive-dl-url">Read or download a fic</label>
      <div class="dl-row">
        <input
          id="archive-dl-url"
          class="dl-input"
          type="url"
          placeholder="Paste a fanfiction URL, or type Title by Author…"
          bind:value={dlUrl}
          onkeydown={onDlKeydown}
          disabled={dlLoading}
          required
        />
        <button class="dl-btn" type="submit" disabled={dlLoading || !dlUrl.trim()}>
          {#if dlLoading}…{:else}Go{/if}
        </button>
      </div>
    </form>
  </div>

  <div class="archive-columns">
    <!-- ─── Main column ──────────────────────────────────────────── -->
    <div class="archive-main-col">
      <!-- Trending This Week — moved above personalized widgets -->
      <section class="blurb-section trending-main">
        <h2 class="section-heading">
          <span>Trending This Week</span>
          <a class="see-all" href="/trending">See all →</a>
        </h2>
        {#if trendingLoading}
          <div class="skeleton-block">{#each Array(5) as _}<div class="skeleton-row"><span class="skeleton-line w80"></span></div>{/each}</div>
        {:else if trendingError}
          <p class="empty-msg">{trendingError}</p>
        {:else if trendingItems.length === 0}
          <p class="empty-msg">Nothing trending yet.</p>
        {:else}
          <ol class="trending-list">
            {#each trendingItems as item (item.url_id)}
              <li class="trending-item">
                <a class="trending-title" href="/works/{item.url_id}">{item.title}</a>
                <span class="trending-byline">by {item.author}</span>
                <span class="trending-meta">{item.downloads} downloads · {item.words.toLocaleString()} words</span>
              </li>
            {/each}
          </ol>
        {/if}
      </section>

      <!-- Continue reading (hidden when logged out or empty) -->
      {#if auth.isLoggedIn && !historyLoading && history.length > 0}
        <section class="blurb-section continue-reading">
          <h2 class="section-heading">Continue Reading</h2>
          <div class="blurb-list">
            {#each history as h (h.id)}
              <div class="rec-item">
                <a class="rec-title" href={`/read/${h.url_id}`}>{h.title}</a>
                <span class="rec-byline">
                  by {h.author || 'Anonymous'}
                  · chapter {h.chapter_num ?? 1}
                  · {relativeTime(h.visited_at)}
                </span>
                <span class="continue-link"><a href={`/read/${h.url_id}`}>Continue →</a></span>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Personalized "Works for you" -->
      {#if auth.isLoggedIn && recsEnough && recs.length > 0}
        <section class="blurb-section">
          <h2 class="section-heading">Works for You</h2>
          <div class="blurb-list">
            {#each recs.slice(0, 5) as r (r.url_id)}
              <div class="rec-item">
                <a class="rec-title" href={r.download_urls?.epub ?? '#'}>{r.title}</a>
                <span class="rec-byline">by {r.author}</span>
                {#if r.summary}
                  <p class="rec-snippet">{r.summary.slice(0, 200)}</p>
                {/if}
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Recommended for You (embeddings; hidden until backend has data) -->
      {#if !embLoading && embRecs.length > 0}
        <section class="blurb-section" aria-label="Recommended for You">
          <h2 class="section-heading">Recommended for You</h2>
          <div class="blurb-list">
            {#each embRecs as r (r.url_id)}
              <!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
              <WorkBlurb fic={r as any} />
            {/each}
          </div>
        </section>
      {/if}

      <!-- Recent Works -->
      <section class="blurb-section">
        <h2 class="section-heading">Recent Works</h2>
        {#if recentLoading}
          <div class="skeleton-block">
            {#each Array(3) as _}
              <div class="skeleton-row">
                <span class="skeleton-line w60"></span>
                <span class="skeleton-line w40"></span>
              </div>
            {/each}
          </div>
        {:else if recentError}
          <p class="empty-msg">{recentError}</p>
        {:else if recentItems.length === 0}
          <p class="empty-msg">No works found.</p>
        {:else}
          <div class="blurb-list">
            {#each recentItems as fic (fic.url_id)}
              <!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
              <WorkBlurb fic={fic as any} />
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <!-- ─── Sidebar ──────────────────────────────────────────────── -->
    <aside class="archive-sidebar">
      <div class="sidebar-box">
        <h2 class="sidebar-heading">Browse</h2>
        <ul class="browse-links">
          <li><a href="/search">Advanced Search</a></li>
          <li><a href="/fandoms">Fandoms</a></li>
          <li><a href="/recommendations">Recommendations</a></li>
          <li><a href="/bookmarks">Bookmarks</a></li>
          <li><a href="/requests">Requests</a></li>
        </ul>
      </div>
    </aside>
  </div>
</div>

<style>
  .archive-home {
    display: flex;
    flex-direction: column;
    gap: 1.2em;
  }

    /* ── Download bar ──────────────────────────────────────────────── */
  .dl-bar {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.8em 1em;
  }

  /* Prominent homepage variant — wider input, bigger button */
  .dl-bar--prominent .dl-input {
    font-size: 1em;
    padding: 0.6em 0.8em;
    width: 100%;
    max-width: 38rem;
    border: 1px solid var(--archive-border, #ccc);
    border-radius: 0.3em;
  }

  .dl-bar--prominent .dl-btn {
    font-size: 1em;
    font-weight: 700;
    padding: 0.6em 1.2em;
  }

  .dl-form {
    margin: 0;
  }

  .dl-label {
    display: block;
    font-size: 0.85em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin-bottom: 0.4em;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .dl-row {
    display: flex;
    gap: 0.5em;
  }

  .dl-input {
    flex: 1;
    padding: 0.45em 0.6em;
    font-size: 0.95em;
    font-family: inherit;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }

  .dl-btn {
    padding: 0.45em 1em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 700;
    background: var(--archive-link, #990000);
    color: var(--archive-bg, #ffffff);
    border: 1px solid var(--archive-link, #990000);
    cursor: pointer;
    white-space: nowrap;
  }

  /* AO3 hover on a .current-style chip: inset shadow press feel */
  .dl-btn:hover {
    box-shadow: inset 2px 2px 2px rgba(187,187,187,.6);
    color: #900;
  }

  .dl-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* ── Two-column layout ────────────────────────────────────────── */
  .archive-columns {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 1.5em;
    align-items: start;
  }

  @media (max-width: 768px) {
    .archive-columns {
      grid-template-columns: 1fr;
    }
  }

  /* ── Section headings ──────────────────────────────────────────── */
  .section-heading {
    font-size: 1.1em;
    font-weight: 700;
    margin: 0 0 0.6em;
    padding-bottom: 0.3em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    color: var(--archive-text, #2a2a2a);
  }

  .blurb-section {
    margin-bottom: 1.5em;
  }

  .blurb-list {
    display: flex;
    flex-direction: column;
    gap: 0.75em;
  }

  /* ── Personal recs mini-items ──────────────────────────────────── */
  .rec-item {
    padding: 0.6em 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }

  .rec-item:last-child {
    border-bottom: none;
  }

  .rec-title {
    font-size: 1.05em;
    font-weight: 700;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .rec-title:hover {
    color: #999;
  }

  .rec-byline {
    display: block;
    font-size: 0.88em;
    color: var(--archive-muted, #666666);
    margin-top: 0.1em;
  }

  .rec-snippet {
    font-size: 0.88em;
    line-height: 1.45;
    color: var(--archive-muted, #666666);
    margin: 0.3em 0 0;
  }

  /* ── Continue reading ───────────────────────────────────────────── */
  .continue-link {
    display: block;
    margin-top: 0.2em;
    font-size: 0.85em;
  }

  .continue-link a {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 700;
  }

  .continue-link a:hover {
    color: #999;
  }

  /* ── Sidebar ───────────────────────────────────────────────────── */
  .archive-sidebar {
    display: flex;
    flex-direction: column;
    gap: 1.2em;
  }

  .sidebar-box {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.8em 1em;
  }

  .sidebar-heading {
    font-size: 0.95em;
    font-weight: 700;
    margin: 0 0 0.5em;
    padding-bottom: 0.3em;
    border-bottom: 2px solid var(--archive-link, #990000);
    color: var(--archive-text, #2a2a2a);
  }

  .trending-list {
    list-style: none;
    margin: 0;
    padding: 0;
    counter-reset: trending;
  }

  .trending-item {
    counter-increment: trending;
    padding: 0.4em 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    display: flex;
    flex-direction: column;
  }

  .trending-item:last-child {
    border-bottom: none;
  }

  .trending-item::before {
    content: counter(trending) ".";
    font-weight: 700;
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    margin-right: 0.4em;
  }

  .trending-title {
    font-size: 0.92em;
    font-weight: 700;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .trending-title:hover {
    color: #999;
  }

  .trending-byline {
    font-size: 0.82em;
    color: var(--archive-muted, #666666);
  }

  .trending-meta {
    font-size: 0.78em;
    color: var(--archive-muted, #666666);
  }

  .sidebar-more {
    margin-top: 0.6em;
    font-size: 0.85em;
    text-align: right;
  }

  .sidebar-more a {
    color: var(--archive-link, #990000);
  }

  .browse-links {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .browse-links li {
    padding: 0.3em 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }

  .browse-links li:last-child {
    border-bottom: none;
  }

  .browse-links a {
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .browse-links a:hover {
    color: #999;
  }

  /* ── Empty / skeleton ──────────────────────────────────────────── */
  .empty-msg {
    font-size: 0.9em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    padding: 0.8em 0;
  }

  .skeleton-block {
    display: flex;
    flex-direction: column;
    gap: 0.6em;
  }

  .skeleton-row {
    display: flex;
    flex-direction: column;
    gap: 0.3em;
  }

  .skeleton-line {
    display: block;
    height: 1em;
    background: var(--archive-border, #dddddd);
    opacity: 0.4;
    border-radius: 2px;
  }

  .w80 { width: 80%; }
  .w60 { width: 60%; }
  .w40 { width: 40%; }
</style>
