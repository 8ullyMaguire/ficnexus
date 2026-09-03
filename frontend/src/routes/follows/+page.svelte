<script lang="ts">
  import { onMount } from 'svelte';
  import { listFollows, unfollow } from '$lib/api/social';
  import type { Follow } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { goto } from '$app/navigation';
  import { getPref } from '$lib/prefs';
  import Exclusions from './Exclusions.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let follows = $state<Follow[]>([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadFollows();
  });

  async function loadFollows() {
    loading = true; error = '';
    try {
      const res = await listFollows();
      if (res.err === 0) follows = res.follows;
      else error = 'Failed to load follows.';
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  async function handleUnfollow(id: number) {
    await unfollow(id);
    follows = follows.filter(f => f.id !== id);
  }

  function followLabel(f: Follow): string {
    if (f.followee_id) return `User #${f.followee_id}`;
    if (f.work_id) return `Work #${f.work_id}`;
    if (f.author_name) return `Author: ${f.author_name}`;
    return t('follows.unknown');
  }

  function followIcon(f: Follow): string {
    if (f.followee_id) return '👤';
    if (f.work_id) return '📖';
    if (f.author_name) return '✍️';
    return '❓';
  }

  function isAuthorFollow(f: Follow): boolean {
    return !!f.author_name;
  }
</script>

<svelte:head><title>{t('follows.title')} | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('follows.title')}</h1>
      <p class="archive-muted archive-sub">{t('follows.subtitle')}</p>
    </header>

    {#if loading}
      <p class="archive-loading">{t('follows.loading')}</p>
    {:else if error}
      <p class="archive-error-text">{error}</p>
    {:else if follows.length === 0}
      <blockquote class="archive-summary"><p>{t('follows.empty')}</p></blockquote>
    {:else}
      <dl class="archive-dl follows-dl">
        {#each follows as f (f.id)}
          <div class="dl-row">
            <dt><span class="archive-follow-label">{followLabel(f)}</span></dt>
            <dd>
              <span class="archive-muted follow-time">{new Date(f.created_at).toLocaleDateString()}</span>
              <button
                class="archive-btn archive-unfollow"
                type="button"
                onclick={() => handleUnfollow(f.id)}
              >{t('follows.unfollow')}</button>
            </dd>
            {#if isAuthorFollow(f)}
              <div class="archive-exclusions"><Exclusions follow={f} /></div>
            {/if}
          </div>
        {/each}
      </dl>
    {/if}
  </div>
</main>
{:else}
<div class="follows-page">
  <h1>{t('follows.title')}</h1>
  <p class="muted">{t('follows.subtitle')}</p>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> {t('follows.loading')}</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {:else if follows.length === 0}
    <div class="card empty"><p class="muted">{t('follows.empty')}</p></div>
  {:else}
    <div class="follow-list">
      {#each follows as f (f.id)}
        <div class="card follow-item">
          <span class="follow-icon">{followIcon(f)}</span>
          <span class="follow-label">{followLabel(f)}</span>
          <span class="follow-time muted">{new Date(f.created_at).toLocaleDateString()}</span>
          <button class="btn-unfollow" onclick={() => handleUnfollow(f.id)}>{t('follows.unfollow')}</button>
          {#if isAuthorFollow(f)}
            <Exclusions follow={f} />
          {/if}
        </div>
      {/each}
    </div>
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
  .follows-dl { margin: 0; }
  .follows-dl .dl-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1em;
    flex-wrap: wrap;
    padding: 0.55em 0.25em;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .follows-dl .dl-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .follows-dl .dl-row:last-child { border-bottom: none; }
  .follows-dl dt { font-weight: 700; font-size: 0.95em; }
  .archive-follow-label { font-weight: 700; }
  .follows-dl dd {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 0.8em;
    font-size: 0.9em;
  }
  .archive-exclusions { flex-basis: 100%; }
  .archive-btn {
    padding: 0.22rem 0.65rem;
    font-size: 0.85em;
    font-family: inherit;
    font-weight: 600;
    color: var(--color-error, #cc0000);
    background: transparent;
    border: 1px solid var(--color-error, #cc0000);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
  }
  .archive-btn:hover {
    background: var(--color-error, #cc0000);
    color: #ffffff;
  }

  /* ── Modern mode ── */
  .follows-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .follows-page .empty { text-align: center; padding: 2rem; }
  .follows-page .error-card { border-color: var(--color-error); }
  .follows-page .follow-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .follows-page .follow-item { display: flex; align-items: center; gap: 0.8rem; padding: 0.7rem 1rem; flex-wrap: wrap; }
  .follows-page .follow-item :global(.follow-exclusions) { flex-basis: 100%; order: 9; }
  .follows-page .follow-icon { font-size: 1.2rem; }
  .follows-page .follow-label { flex: 1; font-weight: 500; }
  .follows-page .follow-time { font-size: 0.82rem; }
  .follows-page .btn-unfollow { padding: 0.3rem 0.6rem; font-size: 0.8rem; border-radius: var(--radius-sm); background: transparent; border: 1px solid var(--color-error); color: var(--color-error); cursor: pointer; }
  .follows-page .btn-unfollow:hover { background: var(--color-error); color: white; }
</style>
