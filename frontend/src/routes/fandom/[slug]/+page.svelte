<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import { getFandom } from '$lib/api/fandom';
  import type { FandomDetail, FandomTopFic, FandomRecentFic } from '$lib/api/fandom';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  const slug = $derived(data.slug);

  let fandom = $state<FandomDetail | null>(null);
  let topFics = $state<FandomTopFic[]>([]);
  let recentlyUpdated = $state<FandomRecentFic[]>([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true;
    try {
      const res = await getFandom(slug);
      if (res.err === 0) {
        fandom = res.fandom;
        topFics = res.top_fics;
        recentlyUpdated = res.recently_updated;
      } else {
        error = 'Fandom not found.';
      }
    } catch {
      error = 'Network error loading fandom.';
    } finally {
      loading = false;
    }
  });

  function formatWords(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return String(n);
  }

  function timeAgo(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / 86400000);
    if (diffDays === 0) return 'today';
    if (diffDays === 1) return 'yesterday';
    if (diffDays < 30) return `${diffDays}d ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
    return `${Math.floor(diffDays / 365)}y ago`;
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      {#if loading}
        <p class="archive-muted"><span class="spinner"></span> Loading fandom…</p>
      {:else if error}
        <h1 class="archive-page-title">Fandom</h1>
        <p class="archive-error">{error}</p>
      {:else if fandom}
        <h1 class="archive-page-title">{fandom.name}</h1>
        <dl class="archive-dl archive-stats-dl">
          <dt>Fics</dt>
          <dd>{fandom.fic_count.toLocaleString()}</dd>
          <dt>Total words</dt>
          <dd>{formatWords(fandom.total_words)}</dd>
          <dt>Authors</dt>
          <dd>{fandom.active_authors.toLocaleString()}</dd>
        </dl>
      {/if}
    </header>

    {#if !loading && !error && fandom}
      <!-- Top Fics -->
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Top Fics</legend>
        {#if topFics.length === 0}
          <p class="archive-muted">No fics tagged with this fandom yet. Tag some fics to populate this page!</p>
        {:else}
          <div class="archive-work-list">
            {#each topFics as fic (fic.url_id)}
              <WorkBlurb
                fic={{
                  url_id: fic.url_id,
                  title: fic.title,
                  author: fic.author,
                  words: fic.words,
                  chapters: fic.chapters,
                  status: '',
                  description: '',
                  updated: '',
                  tags: [],
                  last_visited: null,
                  bookmarked: false,
                  rec: false,
                  snippet: `Score: ${fic.score}`,
                }}
              />
            {/each}
          </div>
        {/if}
      </fieldset>

      <!-- Recently Updated -->
      {#if recentlyUpdated.length > 0}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Recently Updated</legend>
          <div class="archive-work-list">
            {#each recentlyUpdated as fic (fic.url_id)}
              <WorkBlurb
                fic={{
                  url_id: fic.url_id,
                  title: fic.title,
                  author: fic.author,
                  words: 0,
                  chapters: 0,
                  status: '',
                  description: '',
                  updated: fic.updated_at,
                  tags: [],
                  last_visited: null,
                  bookmarked: false,
                  rec: false,
                  snippet: `Updated ${timeAgo(fic.updated_at)}`,
                }}
              />
            {/each}
          </div>
        </fieldset>
      {/if}
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="fandom-page">
  {#if loading}
    <div class="card">
      <p class="muted"><span class="spinner"></span> Loading fandom…</p>
    </div>
  {:else if error}
    <div class="card error-card">
      <strong class="error-text">⚠️ {error}</strong>
    </div>
  {:else if fandom}
    <!-- Hero section -->
    <div class="hero card">
      <h1>📚 {fandom.name}</h1>
      <div class="hero-stats">
        <div class="hero-stat">
          <span class="hero-stat-value">{fandom.fic_count.toLocaleString()}</span>
          <span class="hero-stat-label">fics</span>
        </div>
        <div class="hero-stat">
          <span class="hero-stat-value">{formatWords(fandom.total_words)}</span>
          <span class="hero-stat-label">total words</span>
        </div>
        <div class="hero-stat">
          <span class="hero-stat-value">{fandom.active_authors.toLocaleString()}</span>
          <span class="hero-stat-label">authors</span>
        </div>
      </div>
    </div>

    <!-- Top Fics -->
    <section class="section">
      <h2>🏆 Top Fics</h2>
      {#if topFics.length === 0}
        <div class="card empty">
          <p class="muted">No fics tagged with this fandom yet. Tag some fics to populate this page!</p>
        </div>
      {:else}
        <div class="fic-grid">
          {#each topFics as fic (fic.url_id)}
            <a class="fic-card card" href="/works/{encodeURIComponent(fic.url_id)}">
              <h3 class="fic-title">{fic.title}</h3>
              <span class="fic-author">by {fic.author}</span>
              <div class="fic-meta">
                <span>{formatWords(fic.words)} words</span>
                <span>{fic.chapters} ch</span>
                <span class="fic-score" title="Tag score">★ {fic.score}</span>
              </div>
            </a>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Recently Updated -->
    {#if recentlyUpdated.length > 0}
      <section class="section">
        <h2>🕐 Recently Updated</h2>
        <div class="fic-grid">
          {#each recentlyUpdated as fic (fic.url_id)}
            <a class="fic-card card" href="/works/{encodeURIComponent(fic.url_id)}">
              <h3 class="fic-title">{fic.title}</h3>
              <span class="fic-author">by {fic.author}</span>
              <div class="fic-meta">
                <span class="fic-updated">Updated {timeAgo(fic.updated_at)}</span>
              </div>
            </a>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-header {
    padding: 0 0 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1rem;
  }
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    font-size: 0.9rem;
  }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.3rem 1rem;
    margin: 0.5rem 0 0;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-stats-dl {
    grid-template-columns: auto 1fr auto 1fr auto 1fr;
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend,
  .archive-legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .archive-work-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  /* ── Modern mode (scoped — no bleed) ── */
  .fandom-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .fandom-page .hero { padding: 1.5rem; text-align: center; margin-bottom: 1.5rem; }
  .fandom-page .hero h1 { margin: 0 0 1rem; font-size: 1.6rem; }
  .fandom-page .hero-stats { display: flex; justify-content: center; gap: 2rem; flex-wrap: wrap; }
  .fandom-page .hero-stat { display: flex; flex-direction: column; align-items: center; }
  .fandom-page .hero-stat-value { font-size: 1.4rem; font-weight: 700; color: var(--color-primary); }
  .fandom-page .hero-stat-label { font-size: 0.82rem; color: var(--color-muted); }
  .fandom-page .section { margin-bottom: 1.5rem; }
  .fandom-page .section h2 { font-size: 1.15rem; margin: 0 0 0.8rem; }
  .fandom-page .fic-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.7rem;
  }
  .fandom-page .fic-card {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding: 0.8rem 1rem;
    text-decoration: none;
    color: inherit;
    transition: border-color 0.15s;
  }
  .fandom-page .fic-card:hover { border-color: var(--color-primary); text-decoration: none; }
  .fandom-page .fic-title { font-size: 0.95rem; font-weight: 600; margin: 0; }
  .fandom-page .fic-author { font-size: 0.82rem; color: var(--color-muted); }
  .fandom-page .fic-meta { display: flex; gap: 0.7rem; font-size: 0.78rem; color: var(--color-muted); margin-top: 0.2rem; }
  .fandom-page .fic-score { color: var(--color-primary); font-weight: 600; }
  .fandom-page .fic-updated { font-style: italic; }
  .fandom-page .empty { text-align: center; padding: 2rem; }
  .fandom-page .error-card { border-color: var(--color-error); }
</style>
