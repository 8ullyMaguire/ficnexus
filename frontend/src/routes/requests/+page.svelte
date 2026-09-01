<script lang="ts">
  import { onMount } from 'svelte';
  import { listRequests, type RequestItem } from '$lib/api/requests';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let status = $state<'open' | 'answered' | 'closed'>('open');
  let sort = $state<'top' | 'new'>('new');
  let items = $state<RequestItem[]>([]);
  let loading = $state(true);
  let error = $state('');

  const tabs = [
    { key: 'open', label: t('requests.open') },
    { key: 'answered', label: t('requests.answered') },
    { key: 'closed', label: t('requests.closed') },
  ] as const;

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await listRequests(status, sort);
      if (res.err === 0) {
        items = res.items ?? [];
      } else {
        error = res.msg ?? 'Failed to load requests';
      }
    } catch {
      error = 'Failed to load requests';
    } finally {
      loading = false;
    }
  }

  onMount(load);
</script>

<svelte:head><title>Fic Requests — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  <div class="archive-requests-page">
    <h1 class="page-title">{t('requests.title')}</h1>
    <p class="page-subtitle">
      {@html t('requests.subtitle')}
    </p>

    <div class="toolbar">
      <div class="tabs">
        {#each tabs as tab (tab.key)}
          <button
            class="tab"
            class:active={status === tab.key}
            onclick={() => { status = tab.key; load(); }}
          >
            {tab.label}
          </button>
        {/each}
      </div>
      <div class="sort">
        <button class="tab" class:active={sort === 'top'} onclick={() => { sort = 'top'; load(); }}>{t('requests.top')}</button>
        <button class="tab" class:active={sort === 'new'} onclick={() => { sort = 'new'; load(); }}>{t('requests.newSort')}</button>
      </div>
    </div>

    {#if loading}
      <p class="loading-text">Loading...</p>
    {:else if error}
      <p class="error-text">{error}</p>
    {:else if items.length === 0}
      <p class="empty-text">
        {t('requests.empty', { status: status })}
        {' '}
        <a href="/requests/new">{t('requests.beFirst')}</a>
      </p>
    {:else}
      <ul class="request-list">
        {#each items as r (r.id)}
          <li class="request-row">
            <a class="request-title" href={`/requests/${r.id}`}>{r.title}</a>
            <span class="request-status">{r.status}</span>
            <span class="request-stats">
              <span class="stat-item">{r.upvotes ?? 0} upvotes</span>
              <span class="stat-item">{r.answer_count} {r.answer_count === 1 ? 'answer' : 'answers'}</span>
            </span>
            <ArchiveButton href={`/requests/${r.id}`}>
              Upvote
            </ArchiveButton>
          </li>
        {/each}
      </ul>
    {/if}

    <ArchiveButton href="/requests/new">
      Post New Request
    </ArchiveButton>
  </div>
{:else}
  <div class="requests-page">
    <header class="page-head">
      <h1>{t('requests.title')}</h1>
      <p class="subtitle">
        {@html t('requests.subtitle')}
      </p>
      <a class="btn btn-primary" href="/requests/new">{t('requests.new')}</a>
    </header>

    <div class="toolbar">
      <div class="tabs">
        {#each tabs as t (t.key)}
          <button
            class="tab"
            class:active={status === t.key}
            onclick={() => { status = t.key; load(); }}
          >
            {t.label}
          </button>
        {/each}
      </div>
      <div class="sort">
        <button class="tab" class:active={sort === 'top'} onclick={() => { sort = 'top'; load(); }}>{t('requests.top')}</button>
        <button class="tab" class:active={sort === 'new'} onclick={() => { sort = 'new'; load(); }}>{t('requests.newSort')}</button>
      </div>
    </div>

    {#if loading}
      <p class="muted"><span class="spinner"></span> {t('requests.loading')}</p>
    {:else if error}
      <div class="error-card"><strong>{error}</strong></div>
    {:else if items.length === 0}
      <p class="empty">{t('requests.empty', { status: status })} <a href="/requests/new">{t('requests.beFirst')}</a></p>
    {:else}
      <ul class="request-list">
        {#each items as r (r.id)}
          <li class="card request-card">
            <a class="title" href={`/requests/${r.id}`}>{r.title}</a>
            {#if r.seed_work_id}<span class="chip">{t('requests.seedFic')}</span>{/if}
            <span class="meta">▲ {r.upvotes ?? 0} · {t(r.answer_count === 1 ? 'requests.answers' : 'requests.answersPlural', { count: r.answer_count })} · {r.status}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  /* ── Archive-mode styles ─────────────────────────────────────────── */
  .archive-requests-page {
    max-width: var(--archive-max-width, 800px);
    margin: 0 auto;
    padding: 1.5rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }

  .page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.7em;
    font-weight: normal;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.3em;
  }

  .page-subtitle {
    color: var(--archive-muted, #666666);
    font-size: 0.92em;
    margin: 0 0 1em;
  }

  .archive-requests-page .toolbar {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  .archive-requests-page .tabs,
  .archive-requests-page .sort {
    display: flex;
    gap: 0.25rem;
  }

  .archive-requests-page .tab {
    padding: 0.4em 0.8em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: transparent;
    cursor: pointer;
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.9em;
  }

  .archive-requests-page .tab.active {
    background: var(--archive-link, #990000);
    color: var(--archive-bg, #ffffff);
    border-color: var(--archive-link, #990000);
  }

  .archive-requests-page .request-list {
    list-style: none;
    padding: 0;
    margin: 0 0 1.25em;
  }

  .request-row {
    display: flex;
    align-items: center;
    gap: 0.75em;
    padding: 0.5em 0.3em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }

  .request-title {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
    flex: 1;
    min-width: 0;
  }

  .request-title:hover {
    color: #999;
  }

  .request-status {
    font-variant: small-caps;
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    white-space: nowrap;
  }

  .request-stats {
    display: flex;
    gap: 0.75em;
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    white-space: nowrap;
  }

  .stat-item {
    white-space: nowrap;
  }

  .loading-text {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }

  .error-text {
    color: var(--archive-heading, #990000);
    font-weight: 600;
  }

  .empty-text {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }

  .empty-text a {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .empty-text a:hover {
    color: #999;
  }

  /* ── Modern-mode styles (unchanged) ─────────────────────────────── */
  .requests-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .btn-primary { align-self: flex-start; }
  .toolbar { display: flex; justify-content: space-between; gap: 1rem; margin-bottom: 1rem; flex-wrap: wrap; }
  .tabs, .sort { display: flex; gap: 0.25rem; }
  .tab {
    padding: 0.4rem 0.9rem; border-radius: 999px; border: 1px solid var(--color-border, #ddd);
    background: transparent; cursor: pointer; color: var(--color-text, #222);
  }
  .tab.active { background: var(--color-accent, #4a6cf7); color: #fff; border-color: var(--color-accent, #4a6cf7); }
  .request-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .request-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .title { font-size: 1.1rem; font-weight: 600; color: var(--color-link, #2b4bd7); text-decoration: none; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .chip { align-self: flex-start; background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0; }
</style>
