<script lang="ts">
  import { onMount } from 'svelte';
  import { listBadgeDefinitions } from '$lib/api/social';
  import type { BadgeDefinition } from '$lib/api/social-types';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let badges = $state<BadgeDefinition[]>([]);
  let loading = $state(true);
  let error = $state('');
  let activeCategory = $state('all');

  const categories = ['all', 'curation', 'reading', 'social', 'streak'];

  let filtered = $derived(activeCategory === 'all'
    ? badges
    : badges.filter(b => b.category === activeCategory));

  onMount(async () => {
    loading = true;
    try {
      const res = await listBadgeDefinitions();
      if (res.err === 0) badges = res.badges;
      else error = 'Failed to load badges.';
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  });
</script>

<svelte:head><title>Badges | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Badges</h1>
      <p class="archive-muted archive-sub">Earn badges by contributing to the community, reading, and keeping streaks.</p>
    </header>

    <!-- Category filter fieldset — AO3 archive style -->
    <fieldset class="archive-fieldset">
      <legend>Category</legend>
      <div class="archive-filter-options">
        {#each categories as cat}
          <button
            class="archive-btn"
            class:current={activeCategory === cat}
            type="button"
            onclick={() => activeCategory = cat}
            aria-pressed={activeCategory === cat}
          >{cat === 'all' ? 'All' : cat.charAt(0).toUpperCase() + cat.slice(1)}</button>
        {/each}
      </div>
    </fieldset>

    {#if loading}
      <p class="archive-loading">Loading...</p>
    {:else if error}
      <p class="archive-error-text">{error}</p>
    {:else if filtered.length === 0}
      <blockquote class="archive-summary"><p>No badges in this category.</p></blockquote>
    {:else}
      <dl class="archive-dl badges-dl">
        {#each filtered as b (b.badge_type)}
          <div class="dl-row">
            <dt>
              <span class="badge-name">{b.name}</span>
              <span class="archive-muted badge-cat">{b.category}</span>
            </dt>
            <dd>
              <span class="badge-desc">{b.description}</span>
              <span class="badge-req archive-muted">Requires: {b.threshold} {b.category} actions</span>
            </dd>
          </div>
        {/each}
      </dl>
    {/if}
  </div>
</main>
{:else}
<div class="badges-page">
  <h1>🏅 Badges</h1>
  <p class="muted">Earn badges by contributing to the community, reading, and keeping streaks.</p>

  <div class="category-tabs">
    {#each categories as cat}
      <button class="cat-tab" class:active={activeCategory === cat} onclick={() => activeCategory = cat}>
        {cat === 'all' ? 'All' : cat.charAt(0).toUpperCase() + cat.slice(1)}
      </button>
    {/each}
  </div>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading...</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {:else}
    <div class="badge-grid">
      {#each filtered as b (b.badge_type)}
        <div class="card badge-card">
          <span class="badge-icon">{b.icon}</span>
          <div class="badge-info">
            <strong>{b.name}</strong>
            <p class="muted">{b.description}</p>
            <span class="badge-req">Requires: {b.threshold} {b.category} actions</span>
          </div>
        </div>
      {/each}
    </div>
    {#if filtered.length === 0}
      <div class="card empty"><p class="muted">No badges in this category.</p></div>
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
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-sub {
    margin: 0.4em 0 0;
    font-style: italic;
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .archive-filter-options {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
  }
  .archive-btn {
    padding: 0.28rem 0.75rem;
    font-size: 0.85em;
    font-family: inherit;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
  }
  .archive-btn:hover {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-btn.current {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-loading,
  .archive-muted {
    color: var(--archive-muted, #666666);
  }
  .archive-error-text {
    color: var(--color-error, #cc0000);
    font-weight: 600;
  }
  .archive-summary {
    border-left: 3px solid var(--archive-border, #dddddd);
    margin: 1rem 0;
    padding: 0.6rem 1rem;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-summary p { margin: 0; }
  .badges-dl { margin: 0; }
  .badges-dl .dl-row {
    display: grid;
    grid-template-columns: minmax(140px, 220px) 1fr;
    gap: 0.3em 1em;
    padding: 0.55em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .badges-dl .dl-row:last-child { border-bottom: none; }
  .badges-dl dt { font-weight: 700; font-size: 0.95em; }
  .badge-name { display: block; }
  .badge-cat {
    display: block;
    font-weight: 400;
    font-size: 0.8em;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .badges-dl dd {
    margin: 0;
    font-size: 0.9em;
  }
  .badge-desc { display: block; }
  .badge-req {
    display: block;
    margin-top: 0.15em;
    font-size: 0.92em;
  }

  @media (max-width: 600px) {
    .badges-dl .dl-row { grid-template-columns: 1fr; }
  }

  /* ── Modern mode ── */
  .badges-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .badges-page .category-tabs { display: flex; gap: 0.3rem; margin-bottom: 1rem; flex-wrap: wrap; }
  .badges-page .cat-tab { padding: 0.4rem 0.8rem; font-size: 0.85rem; border-radius: var(--radius-sm); background: var(--color-surface-2); border: 1px solid var(--color-border); cursor: pointer; color: var(--color-muted); }
  .badges-page .cat-tab.active { background: var(--color-primary); color: white; border-color: var(--color-primary); }
  .badges-page .badge-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 0.8rem; }
  .badges-page .badge-card { display: flex; gap: 0.8rem; padding: 1rem; align-items: flex-start; }
  .badges-page .badge-icon { font-size: 2rem; }
  .badges-page .badge-info { flex: 1; }
  .badges-page .badge-info p { margin: 0.3rem 0; font-size: 0.88rem; }
  .badges-page .badge-req { font-size: 0.8rem; color: var(--color-muted); }
  .badges-page .empty { text-align: center; padding: 2rem; }
  .badges-page .error-card { border-color: var(--color-error); }
</style>
