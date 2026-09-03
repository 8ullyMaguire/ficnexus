<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import { getTrending, getTrendingTags } from '$lib/api/social';
  import type { TrendingItem, TrendingTag } from '$lib/api/social-types';
  import { t } from '$lib/i18n/index.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let items = $state<TrendingItem[]>([]);
  let tags = $state<TrendingTag[]>([]);
  let loading = $state(true);
  let error = $state('');
  let activeTab = $state<'fics' | 'tags'>('fics');
  let days = $state(7);

  onMount(async () => {
    loading = true;
    try {
      const [ficsRes, tagsRes] = await Promise.all([
        getTrending(days, 20),
        getTrendingTags(days, 20),
      ]);
      if (ficsRes.err === 0) items = ficsRes.trending;
      if (tagsRes.err === 0) tags = tagsRes.trending_tags;
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  });

  async function switchDays(n: number) {
    days = n; loading = true;
    try {
      const [ficsRes, tagsRes] = await Promise.all([
        getTrending(days, 20),
        getTrendingTags(days, 20),
      ]);
      if (ficsRes.err === 0) items = ficsRes.trending;
      if (tagsRes.err === 0) tags = tagsRes.trending_tags;
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  function typeName(typeId: number): string {
    const names: Record<number, string> = {
      1: t('trending.fandom'), 2: t('trending.character'),
      3: t('trending.relationship'), 4: t('trending.freeform'),
      5: t('trending.warning'), 6: t('trending.category'),
    };
    return names[typeId] || t('trending.other');
  }
</script>
<svelte:head>
  <title>Trending — FicHub</title>
  <meta name="description" content="What the FicHub community is reading and downloading right now." />
</svelte:head>


{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <header class="archive-header">
    <h1 class="archive-page-title">Trending</h1>
  </header>

  <div class="archive-content">
    <!-- Period fieldset — AO3 archive style -->
    <fieldset class="archive-fieldset trending-period">
      <legend>Period</legend>
      <div class="archive-period-options">
        {#each [{ n: 1, label: '1 day' }, { n: 7, label: '7 days' }, { n: 30, label: '30 days' }] as d}
          <button
            class="archive-filter-btn"
            class:current={days === d.n}
            type="button"
            onclick={() => switchDays(d.n)}
            aria-pressed={days === d.n}
          >{d.label}</button>
        {/each}
      </div>
    </fieldset>

    <!-- Archive tabs: Popular Fics / Trending Tags -->
    <ul class="archive-tabs" role="tablist">
      <li class:current={activeTab === 'fics'} role="presentation">
        <button role="tab" aria-selected={activeTab === 'fics'} type="button" onclick={() => (activeTab = 'fics')}>
          {t('trending.popularFics')}
        </button>
      </li>
      <li class:current={activeTab === 'tags'} role="presentation">
        <button role="tab" aria-selected={activeTab === 'tags'} type="button" onclick={() => (activeTab = 'tags')}>
          {t('trending.trendingTags')}
        </button>
      </li>
    </ul>

    {#if loading}
      <div class="archive-skeleton-list">
        {#each Array(5) as _, i}
          <article class="skeleton-blurb" aria-hidden="true">
            <div class="skel-header"><span class="skel-badge"></span><span class="skel-title"></span></div>
            <span class="skel-byline"></span>
            <div class="skel-stats"></div>
          </article>
        {/each}
      </div>
    {:else if error}
      <div class="archive-error">
        <p>{error}</p>
      </div>
    {:else if activeTab === 'fics'}
      {#if items.length === 0}
        <blockquote class="archive-summary">
          <p>No trending works found for this period.</p>
        </blockquote>
      {:else}
        <div class="archive-work-list">
          {#each items as item, i (item.url_id)}
            <WorkBlurb
              fic={{
                url_id: item.url_id,
                title: item.title,
                author: item.author,
                words: item.words,
                chapters: item.chapters,
                source: '',
                status: item.status,
                description: '',
                updated: '',
                tags: [],
                last_visited: null,
                bookmarked: false,
                rec: false,
              }}
            />
          {/each}
        </div>
      {/if}
    {:else}
      {#if tags.length === 0}
        <blockquote class="archive-summary">
          <p>No trending tags found for this period.</p>
        </blockquote>
      {:else}
        <!-- Archive tag list with dl.stats-style meta -->
        <ul class="archive-tag-list">
          {#each tags as tag, i (tag.id)}
            <li>
              <span class="tag-rank">#{i + 1}</span>
              <a class="archive-link tag-name" href={`/search?include_tags=${tag.tag_type_id}:${encodeURIComponent(tag.name)}`}>
                {tag.name}
              </a>
              <span class="archive-muted tag-type">{typeName(tag.tag_type_id)}</span>
              <dl class="stats tag-stats">
                <dt>Works:</dt>
                <dd>{tag.fic_count}</dd>
              </dl>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="trending-page">
  <h1>{t('trending.title')}</h1>
  <div class="trending-controls">
    <div class="tab-bar">
      <button class="tab" class:active={activeTab === 'fics'} onclick={() => activeTab = 'fics'}>{t('trending.popularFics')}</button>
      <button class="tab" class:active={activeTab === 'tags'} onclick={() => activeTab = 'tags'}>{t('trending.trendingTags')}</button>
    </div>
    <div class="day-picker">
      {#each [1, 7, 30] as d}
        <button class="day-btn" class:active={days === d} onclick={() => switchDays(d)}>{d}d</button>
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> {t('trending.loading')}</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text"> {error}</strong></div>
  {:else if activeTab === 'fics'}
    {#if items.length === 0}
      <div class="card empty"><p class="muted">{t('trending.noFics')}</p></div>
    {:else}
      <div class="fic-list">
        {#each items as item, i (item.url_id)}
          <div class="card fic-item">
            <span class="rank-badge">#{i + 1}</span>
            <div class="fic-info">
              <a href={`/works/${item.url_id}`} class="fic-title">{item.title}</a>
              <span class="muted">by {item.author}</span>
              <div class="fic-meta">
                <span>{item.words.toLocaleString()} {t('trending.words')}</span>
                <span>{item.chapters} ch</span>
                <span>{item.status}</span>
              </div>
            </div>
            <div class="fic-stats">
              <span class="stat" title={t('trending.requests')}>{item.requests}</span>
              <span class="stat" title={t('trending.downloads')}>{item.downloads}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    {#if tags.length === 0}
      <div class="card empty"><p class="muted">{t('trending.noTags')}</p></div>
    {:else}
      <div class="tag-list">
        {#each tags as tag (tag.id)}
          <div class="card tag-item">
            <span class="tag-rank">#{tag.fic_count}</span>
            <div class="tag-info">
              <strong>{tag.name}</strong>
              <span class="muted">{typeName(tag.tag_type_id)}</span>
            </div>
            <span class="tag-count">{t('trending.fics', { count: tag.fic_count })}</span>
            <a href={`/search?include_tags=${tag.tag_type_id}:${encodeURIComponent(tag.name)}`} class="btn-xs">{t('trending.browse')}</a>
          </div>
        {/each}
      </div>
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
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 0;
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }

  /* Period fieldset */
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
  .archive-period-options {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
  }
  .archive-filter-btn {
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
  .archive-filter-btn:hover {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-filter-btn.current {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }

  /* Archive tabs */
  .archive-tabs {
    display: flex;
    list-style: none;
    margin: 0 0 1.2rem;
    padding: 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    gap: 0;
  }
  .archive-tabs li {
    margin: 0;
    margin-bottom: -1px;
  }
  .archive-tabs li button {
    display: block;
    padding: 0.45rem 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.9em;
    font-weight: 600;
    color: var(--archive-muted, #666666);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-bottom: 1px solid var(--archive-border, #dddddd);
    cursor: pointer;
    line-height: 1.4;
  }
  .archive-tabs li button:hover {
    color: var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-tabs li.current button {
    color: var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
    border-color: var(--archive-border, #dddddd);
    border-bottom-color: var(--archive-bg, #ffffff);
  }

  /* Work list */
  .archive-work-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  /* Tag list — archive link styling + dl.stats */
  .archive-tag-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-tag-list li {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    padding: 0.55rem 0.6rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    font-size: 0.9em;
    flex-wrap: wrap;
  }
  .archive-tag-list li:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .tag-rank {
    min-width: 2rem;
    text-align: center;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    font-size: 0.85em;
  }
  .tag-name.archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .tag-name.archive-link:hover {
    text-decoration: underline;
  }
  .tag-type.archive-muted {
    color: var(--archive-muted, #666666);
    font-size: 0.85em;
  }
  .tag-stats {
    display: flex;
    gap: 0.3rem;
    margin: 0 0 0 auto;
    font-size: 0.85em;
  }
  .tag-stats dt {
    font-weight: 600;
    color: var(--archive-muted, #666666);
  }
  .tag-stats dd {
    margin: 0;
    color: var(--archive-text, #2a2a2a);
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

  /* Skeleton */
  .archive-skeleton-list article.skeleton-blurb {
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding: 0.8rem 0;
  }
  .skel-badge, .skel-title, .skel-byline, .skel-stats {
    display: block;
    background: var(--archive-border, #dddddd);
    border-radius: 2px;
    margin-bottom: 0.3rem;
  }
  .skel-badge { width: 2rem; height: 0.9rem; }
  .skel-title { width: 40%; height: 1.2rem; }
  .skel-byline { width: 20%; height: 0.8rem; }
  .skel-stats { width: 15%; height: 0.8rem; }

  .archive-error {
    padding: 2rem;
    text-align: center;
    color: var(--archive-muted, #666666);
  }

  /* ── Modern mode (scoped to .trending-page — no bleed) ── */
  .trending-page { max-width: var(--max-width, 900px); margin: 0 auto; padding: 1rem; }
  .trending-page .trending-controls { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem; }
  .trending-page .tab-bar { display: flex; gap: 0.3rem; }
  .trending-page .tab { padding: 0.4rem 0.8rem; font-size: 0.85rem; border-radius: var(--radius-sm, 6px); background: var(--color-surface-2, #f0f0f0); border: 1px solid var(--color-border, #ddd); cursor: pointer; color: var(--color-muted, #666); }
  .trending-page .tab.active { background: var(--color-primary, #990000); color: white; }
  .trending-page .day-picker { display: flex; gap: 0.3rem; }
  .trending-page .day-btn { padding: 0.3rem 0.6rem; font-size: 0.82rem; border-radius: var(--radius-sm, 6px); background: var(--color-surface-2, #f0f0f0); border: 1px solid var(--color-border, #ddd); cursor: pointer; color: var(--color-muted, #666); }
  .trending-page .day-btn.active { background: var(--color-primary, #990000); color: white; }
  .trending-page .fic-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .trending-page .fic-item { display: flex; align-items: center; gap: 1rem; padding: 0.8rem 1rem; }
  .trending-page .rank-badge { font-weight: 700; font-size: 1.1rem; color: var(--color-primary, #990000); width: 2.5rem; text-align: center; }
  .trending-page .fic-info { flex: 1; min-width: 0; }
  .trending-page .fic-title { font-weight: 600; text-decoration: none; color: var(--color-text, #111); }
  .trending-page .fic-title:hover { color: var(--color-primary, #990000); }
  .trending-page .fic-meta { display: flex; gap: 0.8rem; font-size: 0.82rem; color: var(--color-muted, #666); margin-top: 0.2rem; }
  .trending-page .fic-stats { display: flex; gap: 0.5rem; flex-shrink: 0; }
  .trending-page .stat { font-size: 0.85rem; color: var(--color-muted, #666); }
  .trending-page .tag-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .trending-page .tag-item { display: flex; align-items: center; gap: 1rem; padding: 0.7rem 1rem; }
  .trending-page .tag-rank { font-size: 0.9rem; color: var(--color-muted, #666); width: 2.5rem; }
  .trending-page .tag-info { flex: 1; }
  .trending-page .tag-count { font-size: 0.85rem; color: var(--color-muted, #666); }
  .trending-page .btn-xs { padding: 0.25rem 0.5rem; font-size: 0.8rem; border-radius: var(--radius-sm, 6px); background: var(--color-surface-2, #f0f0f0); border: 1px solid var(--color-border, #ddd); cursor: pointer; text-decoration: none; color: var(--color-text, #111); }
  .trending-page .btn-xs:hover { background: var(--color-surface, #fff); }
  .trending-page .empty { text-align: center; padding: 2rem; }
  .trending-page .error-card { border-color: var(--color-error, #c00); }
</style>
