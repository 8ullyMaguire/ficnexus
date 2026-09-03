<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface Feature {
    id: number;
    text: string;
    elo_rating: number;
    matches_played: number;
    times_picked_best: number;
    times_picked_worst: number;
    status: string;
    category: string;
    suggestions: number;
  }

  const STATUSES = [
    { id: 'up_next', label: 'Up Next', color: '#f59e0b' },
    { id: 'in_progress', label: 'In Progress', color: '#f97316' },
    { id: 'finished', label: 'Finished (Not Shipped)', color: '#22c55e' },
    { id: 'shipped', label: 'Shipped', color: '#10b981' },
    { id: 'medium_term', label: 'Medium-term', color: '#ec4899' },
    { id: 'long_term', label: 'Long-term', color: '#7c3aed' },
    { id: 'idea', label: 'Ideas', color: '#3b82f6' },
    { id: 'rejected', label: 'Rejected', color: '#9ca3af' },
  ] as const;

  const CATEGORIES = [
    'all', 'search', 'scraper', 'social', 'reader', 'admin', 'recs', 'general'
  ] as const;

  let features = $state<Feature[]>([]);
  let loading = $state(true);
  let error = $state('');
  let activeCategory = $state('all');
  let searchQuery = $state('');

  async function loadFeatures() {
    loading = true;
    error = '';
    try {
      const params = new URLSearchParams();
      if (activeCategory !== 'all') params.set('category', activeCategory);
      if (searchQuery) params.set('q', searchQuery);
      const res = await fetch(`/api/roadmap/features?${params.toString()}`, { credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      features = body.features ?? [];
    } catch (e) {
      error = `Failed to load features: ${e instanceof Error ? e.message : e}`;
      features = [];
    } finally {
      loading = false;
    }
  }

  function getFeaturesByStatus(status: string): Feature[] {
    return features.filter(f => f.status === status).sort((a, b) => b.elo_rating - a.elo_rating);
  }

  function filterFeaturesForColumn(features: Feature[]): Feature[] {
    return features.filter(f => {
      if (activeCategory !== 'all' && f.category !== activeCategory) return false;
      if (searchQuery && !f.text.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      return true;
    });
  }

  function statusLabel(status: string): string {
    const s = STATUSES.find(x => x.id === status);
    return s ? s.label : status;
  }

  function statusColor(status: string): string {
    const s = STATUSES.find(x => x.id === status);
    return s ? s.color : '#6b7280';
  }

  let filteredFeatures = $derived(features.filter(f => {
    if (activeCategory !== 'all' && f.category !== activeCategory) return false;
    if (searchQuery && !f.text.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  }));

  let columnData = $derived(STATUSES.map(s => ({
    ...s,
    features: filterFeaturesForColumn(getFeaturesByStatus(s.id))
  })));

  async function moveFeature(id: number, newStatus: string) {
    const feature = features.find(f => f.id === id);
    if (!feature) return;
    try {
      const res = await fetch(`/api/roadmap/features/${id}`, {
        method: 'PATCH',
        credentials: 'include',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ status: newStatus, category: feature.category }),
      });
      const body = await res.json();
      if (!res.ok || body.err !== 0) {
        alert(body.msg ?? 'Move failed');
        return;
      }
      // Optimistic update
      features = features.map(f => f.id === id ? { ...f, status: newStatus } : f);
    } catch (e) {
      alert(`Move failed: ${e instanceof Error ? e.message : e}`);
    }
  }

  onMount(async () => {
    await auth.init();
    await loadFeatures();
  });
</script>


<div class="roadmap-board-page" class:archive={uiMode === 'archive'}>
  {#if uiMode === 'archive'}
    <main class="archive-main" role="main">
      <div class="archive-content">
        <header class="archive-header">
          <h1 class="archive-page-title">{t('roadmap.boardTitle')}</h1>
          <p class="archive-summary">{t('roadmap.boardSub')}</p>
        </header>
        {@render board()}
      </div>
    </main>
  {:else}
    <div class="modern-board">
      <header class="board-header">
        <h1>{t('roadmap.boardTitle')}</h1>
        <p class="board-sub">{t('roadmap.boardSub')}</p>
      </header>
      {@render board()}
    </div>
  {/if}
</div>

{#snippet board()}
  {#if loading}
    <div class="board-loading">{t('roadmap.loading')}</div>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else}
    <div class="board-toolbar">
      <div class="toolbar-search">
        <input
          type="search"
          placeholder={t('roadmap.searchPlaceholder')}
          bind:value={searchQuery}
          class="search-input"
        />
      </div>
      <div class="toolbar-categories">
        {#each CATEGORIES as cat}
          <button
            class="cat-chip {cat === activeCategory ? 'active' : ''}"
            onclick={() => activeCategory = cat}
          >
            {cat === 'all' ? t('roadmap.categoryAll') : t('roadmap.category.' + cat)}
          </button>
        {/each}
      </div>
    </div>

    <div class="kanban-board" role="list" aria-label={t('roadmap.boardAria')}>
      {#each columnData as col (col.id)}
        <section class="kanban-column" style="--col-color: {col.color};" role="listitem" aria-label={col.label}>
          <header class="column-header">
            <span class="column-title" style="background: var(--col-color);">{col.label}</span>
            <span class="column-count">{col.features.length}</span>
            {#if col.id === 'idea'}
              <div class="arena-cta" data-testid="arena-cta">
                <a href="/roadmap" class="arena-link">{t('roadmap.arenaCta')}</a>
              </div>
            {/if}
          </header>
          <div class="column-cards">
            {#each col.features as feature (feature.id)}
              <article class="kanban-card" data-testid="kanban-card">
                <div class="card-header">
                  <span class="card-category">{feature.category}</span>
                  <span class="card-elo">{Math.round(feature.elo_rating)}</span>
                </div>
                <p class="card-text">{feature.text}</p>
                <div class="card-meta">
                  <span class="card-votes">{t('roadmap.votes', { count: feature.matches_played })}</span>
                  <span class="card-suggestions">{feature.suggestions} {t('roadmap.suggestions')}</span>
                </div>
                {#if auth.user?.level !== undefined && auth.user.level >= 50}
                  <div class="card-actions">
                    <select
                      bind:value={feature.status}
                      onchange={() => moveFeature(feature.id, feature.status)}
                      class="status-select"
                    >
                      {#each STATUSES as s}
                        <option value={s.id} selected={s.id === feature.status}>{s.label}</option>
                      {/each}
                    </select>
                  </div>
                {/if}
              </article>
            {/each}
            {#if col.features.length === 0}
              <div class="column-empty">{t('roadmap.columnEmpty')}</div>
            {/if}
          </div>
        </section>
      {/each}
    </div>
  {/if}
{/snippet}

<style>
  /* ── Archive mode ─────────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: 960px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    margin-bottom: 1.25rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.6em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8em;
    font-weight: 700;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
  }
  .archive-summary {
    color: var(--archive-muted, #666666);
    font-size: 0.95em;
    margin: 0.2em 0 0;
    max-width: 80ch;
  }

  /* ── Modern mode ──────────────────────────────────────────────── */
  .modern-board {
    max-width: 1400px;
    margin: 0 auto;
    padding: 1.5rem;
  }
  .board-header {
    margin-bottom: 1.5rem;
  }
  .board-header h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0 0 0.25rem;
  }
  .board-sub {
    color: var(--text-muted, #6b7280);
    margin: 0;
  }

  /* ── Toolbar ──────────────────────────────────────────────────── */
  .board-toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    align-items: center;
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    background: var(--bg-raised, #f9fafb);
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 0.5rem;
  }
  .toolbar-search { flex: 1; min-width: 200px; }
  .search-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 0.375rem;
    font-size: 0.9rem;
  }
  .toolbar-categories {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }
  .cat-chip {
    padding: 0.25rem 0.625rem;
    font-size: 0.8rem;
    font-weight: 500;
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 9999px;
    background: var(--bg, #ffffff);
    color: var(--text, #1f2937);
    cursor: pointer;
    transition: all 0.15s;
  }
  .cat-chip:hover { background: var(--bg-raised, #f9fafb); }
  .cat-chip.active {
    background: var(--primary, #2563eb);
    border-color: var(--primary, #2563eb);
    color: white;
  }

  /* ── Kanban board ────────────────────────────────────────────── */
  .kanban-board {
    display: flex;
    gap: 1rem;
    overflow-x: auto;
    padding-bottom: 1rem;
    min-height: 600px;
  }
  .kanban-column {
    flex: 0 0 280px;
    max-width: 280px;
    min-height: 500px;
    background: var(--bg-raised, #f9fafb);
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
  }
  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border, #e5e7eb);
    position: relative;
  }
  .column-title {
    font-weight: 700;
    font-size: 0.85rem;
    padding: 0.25rem 0.625rem;
    border-radius: 0.25rem;
    color: white;
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }
  .column-count {
    font-size: 0.8rem;
    color: var(--text-muted, #6b7280);
    background: var(--bg, #ffffff);
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    border: 1px solid var(--border, #e5e7eb);
  }
  .arena-cta {
    position: absolute;
    bottom: -12px;
    left: 50%;
    transform: translateX(-50%);
    width: calc(100% - 1rem);
    text-align: center;
  }
  .arena-link {
    display: inline-block;
    font-size: 0.7rem;
    color: var(--primary, #2563eb);
    text-decoration: none;
    font-weight: 500;
  }
  .arena-link:hover { text-decoration: underline; }

  .column-cards {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-height: 0;
  }
  .kanban-card {
    background: var(--bg, #ffffff);
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 0.375rem;
    padding: 0.75rem;
    flex-shrink: 0;
  }
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  .card-category {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted, #6b7280);
    background: var(--bg-raised, #f9fafb);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }
  .card-elo {
    font-weight: 700;
    font-size: 0.85rem;
    color: var(--primary, #2563eb);
  }
  .card-text {
    font-size: 0.875rem;
    line-height: 1.5;
    margin: 0 0 0.5rem;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .card-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.7rem;
    color: var(--text-muted, #6b7280);
    padding-top: 0.5rem;
    border-top: 1px solid var(--border, #e5e7eb);
  }
  .card-actions {
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border, #e5e7eb);
  }
  .status-select {
    width: 100%;
    padding: 0.375rem 0.5rem;
    font-size: 0.8rem;
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 0.375rem;
    background: var(--bg, #ffffff);
    color: var(--text, #1f2937);
  }
  .column-empty {
    padding: 1rem;
    text-align: center;
    color: var(--text-muted, #6b7280);
    font-size: 0.8rem;
    font-style: italic;
  }
  .board-loading {
    padding: 2rem;
    text-align: center;
    color: var(--text-muted, #6b7280);
  }
  .error-card {
    padding: 1rem;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.5rem;
    color: #991b1b;
  }
</style>