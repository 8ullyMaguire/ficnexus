<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { page } from '$app/stores';
  import { getSeries } from '$lib/api/series';
  import type { SeriesDetail, SeriesWork } from '$lib/api/series';
  import { formatWords, stripHtml } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  const seriesId = $derived(Number(data.id));

  let series = $state<SeriesDetail | null>(null);
  let works = $state<SeriesWork[]>([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    await load();
  });

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await getSeries(seriesId);
      if (res.err !== 0) {
        error = 'Could not load series.';
        return;
      }
      series = res.series;
      works = res.works ?? [];
    } catch {
      error = 'Network error loading series.';
    } finally {
      loading = false;
    }
  }

  function workHref(w: SeriesWork): string {
    return w.url_id ? `/works/${encodeURIComponent(w.url_id)}` : `/work/${w.work_id}`;
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      {#if loading}
        <p class="archive-muted"><span class="spinner"></span> Loading series…</p>
      {:else if error}
        <h1 class="archive-page-title">Series</h1>
        <p class="archive-error">{error}</p>
      {:else if series}
        <h1 class="archive-page-title">{series.name}</h1>
        {#if series.description}
          <blockquote class="archive-summary">{stripHtml(series.description)}</blockquote>
        {/if}
        <p class="archive-muted">{series.work_count} {series.work_count === 1 ? 'work' : 'works'} in series</p>
      {/if}
    </header>

    {#if !loading && !error && series}
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Works</legend>
        {#if works.length === 0}
          <p class="archive-muted">No works in this series yet.</p>
        {:else}
          <ol class="archive-list">
            {#each works as w (w.work_id)}
              <li class="archive-list-row" class:next-row={w.next_in_series !== null}>
                <span class="work-pos">{works.indexOf(w) + 1}.</span>
                <div class="archive-list-main">
                  <a class="archive-link archive-list-title" href={workHref(w)}>
                    {w.canonical_title || w.title}
                  </a>
                  <div class="archive-muted">by {w.canonical_author || w.author}</div>
                  <div class="archive-meta-line">
                    {formatWords(w.words)} words · {w.chapters} ch · {w.status}
                  </div>
                  {#if w.next_in_series}
                    <div class="next-note">▶ Next in series: {w.next_in_series.canonical_title}</div>
                  {/if}
                </div>
              </li>
            {/each}
          </ol>
        {/if}
      </fieldset>

      <p class="archive-muted back-link"><a class="archive-link" href="/">← Back to home</a></p>
    {/if}
  </div>
</main>
{:else if loading}
<!-- Modern mode (unchanged) -->
  <div class="card">
    <p class="muted"><span class="spinner"></span> Loading series…</p>
  </div>
{:else if error}
  <div class="card error-card">
    <strong class="error-text">⚠️ {error}</strong>
  </div>
{:else if series}
  <div class="series-page">
    <div class="series-header card">
      <h1>📚 {series.name}</h1>
      {#if series.description}
        <p class="desc">{stripHtml(series.description)}</p>
      {/if}
      <p class="muted">
        {series.work_count} {series.work_count === 1 ? 'work' : 'works'} in series
      </p>
    </div>

    {#if works.length === 0}
      <div class="card"><p class="muted">No works in this series yet.</p></div>
    {:else}
      <div class="works-list">
        {#each works as w (w.work_id)}
          <a class="card work-card" class:next-card={w.next_in_series !== null} href={workHref(w)}>
            <span class="work-pos">{works.indexOf(w) + 1}</span>
            <div class="work-info">
              <strong class="work-title">{w.canonical_title || w.title}</strong>
              <span class="muted">by {w.canonical_author || w.author}</span>
              <span class="muted work-meta">
                {formatWords(w.words)} words · {w.chapters} ch · {w.status}
              </span>
            </div>
            {#if w.next_in_series}
              <span class="next-badge">▶ Next in series: {w.next_in_series.canonical_title}</span>
            {/if}
          </a>
        {/each}
      </div>
    {/if}

    <p class="muted back-link"><a href="/">← Back to home</a></p>
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
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .archive-link:hover {
    text-decoration: underline;
  }
  .archive-summary {
    margin: 0.5rem 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    white-space: pre-wrap;
    background: var(--archive-bg-raised, #f5f5f5);
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
  .archive-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-list-row {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    padding: 0.55rem 0.3rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    font-size: 0.92rem;
  }
  .archive-list-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-list-row.next-row {
    border-left: 3px solid var(--archive-link, #990000);
  }
  .work-pos {
    min-width: 1.4rem;
    text-align: right;
    font-weight: 700;
    color: var(--archive-muted, #666666);
  }
  .archive-list-main { flex: 1; min-width: 0; }
  .archive-list-title {
    font-weight: 700;
  }
  .archive-meta-line {
    font-size: 0.82rem;
    color: var(--archive-muted, #666666);
    margin-top: 0.15rem;
  }
  .next-note {
    font-size: 0.82rem;
    color: var(--archive-link, #990000);
    font-weight: 600;
    margin-top: 0.2rem;
  }
  .back-link { margin-top: 1.2rem; }

  /* ── Modern mode (scoped — no bleed) ── */
  .series-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  .series-page .series-header h1 {
    margin: 0 0 0.4rem;
    font-size: 1.6rem;
  }
  .series-page .desc {
    white-space: pre-wrap;
    color: var(--color-muted);
    font-size: 0.92rem;
    line-height: 1.6;
  }
  .series-page .works-list {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    margin-top: 1rem;
  }
  .series-page .work-card {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    text-decoration: none;
    color: inherit;
  }
  .series-page .work-card.next-card {
    border-color: var(--color-primary);
    background: var(--color-primary-alpha);
  }
  .series-page .work-pos {
    font-family: var(--mono);
    font-weight: 700;
    color: var(--color-muted);
    min-width: 1.4rem;
    text-align: center;
  }
  .series-page .work-info {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
    flex: 1;
  }
  .series-page .work-title {
    font-size: 1.05rem;
  }
  .series-page .work-meta {
    font-size: 0.82rem;
  }
  .series-page .next-badge {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--color-primary);
    white-space: nowrap;
  }
  .series-page .back-link {
    margin-top: 1.5rem;
  }
</style>
