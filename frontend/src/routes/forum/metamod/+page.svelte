<script lang="ts">
  // F6 metamoderation (audit layer): veteran members rate a random sample of
  // moderator actions (fair / unfair / unsure). The moderator's identity is
  // anonymized server-side — cards only show the post excerpt, the reason that
  // was applied, and the score delta. Each action is sampled for 3 ratings, so
  // voting removes the card; the backend drops already-voted actions from the
  // queue. pool_too_small means metamoderation is dormant and the public
  // modlog is the audit.
  import { onMount } from 'svelte';
  import {
    getMetamodQueue,
    voteMetamod,
    type ForumMetamodItem,
    type ForumMetamodVerdict,
  } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  const VERDICTS: ForumMetamodVerdict[] = ['fair', 'unfair', 'unsure'];

  let filter: 'unreviewed' | 'reviewed' | 'all' = $state('unreviewed');
  let verdictFilter: string | null = $state(null);
  let loading = $state(true);
  let error = $state('');
  let items = $state<ForumMetamodItem[]>([]);
  let total = $state(0);
  let poolTooSmall = $state(false);
  let nextCursor = $state<number | null>(null);
  let workingActionId = $state<number | null>(null);
  let failureActionId = $state<number | null>(null);

  function auditCountLabel(count: number): string {
    return count === 1 ? t('forum.metamodCountOne', { count }) : t('forum.metamodCount', { count });
  }

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await getMetamodQueue({ filter, verdict: verdictFilter, cursor: null });
      if (res.err === 0) {
        items = res.items ?? [];
        total = res.count ?? items.length;
        poolTooSmall = res.pool_too_small === true;
        nextCursor = res.next_cursor ?? null;
      } else {
        error = res.msg ?? t('forum.metamodError');
      }
    } catch {
      error = t('forum.metamodError');
    } finally {
      loading = false;
    }
  }

  async function loadMore() {
    if (!nextCursor) return;
    try {
      const res = await getMetamodQueue({ filter, verdict: verdictFilter, cursor: nextCursor });
      if (res.err === 0) {
        items = [...items, ...(res.items ?? [])];
        nextCursor = res.next_cursor ?? null;
      }
    } catch {}
  }

  function setFilter(f: typeof filter, v: string | null = null) {
    filter = f;
    verdictFilter = v;
    load();
  }

  onMount(async () => {
    await auth.init();
    await load();
  });

  async function vote(item: ForumMetamodItem, verdict: ForumMetamodVerdict) {
    if (workingActionId !== null) return;
    workingActionId = item.action_id;
    failureActionId = null;
    try {
      const res = await voteMetamod(item.action_id, verdict);
      if (res.err === 0) {
        // Optimistic removal — this action is now fully rated.
        items = items.filter((it) => it.action_id !== item.action_id);
        total = Math.max(0, total - 1);
      } else if (res.err === 409) {
        // Already voted on (e.g. another tab) — treat as resolved, drop it.
        items = items.filter((it) => it.action_id !== item.action_id);
        total = Math.max(0, total - 1);
      } else {
        failureActionId = item.action_id;
      }
    } catch {
      failureActionId = item.action_id;
    } finally {
      workingActionId = null;
    }
  }
</script>

<svelte:head><title>{t('forum.metamod')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('forum.metamod')}</h1>
      <p class="archive-summary">{t('forum.metamodSubtitle')}</p>
      <p class="archive-backlink"><a class="archive-link" href="/forum">&larr; {t('forum.back')}</a></p>
    </header>

    {#if loading}
      <blockquote class="archive-empty"><p>{t('forum.loading')}</p></blockquote>
    {:else if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
    {:else if items.length === 0}
      <blockquote class="archive-empty">
        <p>{t('forum.metamodEmpty')}</p>
        {#if poolTooSmall}
          <p>{t('forum.metamodDormant')}</p>
        {/if}
      </blockquote>
    {:else}
      <div class="filter-bar" role="tablist" aria-label="queue filter">
        <button role="tab" aria-selected={filter==='unreviewed'} class:active={filter==='unreviewed'} onclick={() => setFilter('unreviewed')}>Unreviewed</button>
        <button role="tab" aria-selected={filter==='reviewed' && !verdictFilter} class:active={filter==='reviewed' && !verdictFilter} onclick={() => setFilter('reviewed')}>Reviewed</button>
        <button role="tab" aria-selected={verdictFilter==='fair'} class:active={verdictFilter==='fair'} onclick={() => setFilter('reviewed','fair')}>Fair</button>
        <button role="tab" aria-selected={verdictFilter==='unfair'} class:active={verdictFilter==='unfair'} onclick={() => setFilter('reviewed','unfair')}>Unfair</button>
      </div>
      <p class="archive-count">{auditCountLabel(total)}</p>
      <ul class="archive-audit-list">
        {#each items as item (item.action_id)}
          <li class="archive-audit-item">
            <div class="archive-audit-meta">
              <span class="archive-chip">{t(`forum.reason.${item.reason}`)}</span>
              <span class="archive-chip archive-delta" class:pos={item.delta > 0} class:neg={item.delta < 0}>
                {item.delta > 0 ? '+' : ''}{item.delta}
              </span>
              <span class="archive-muted">{(item.created_at ?? '').slice(0, 10)}</span>
              <a class="archive-link" href={`/forum/${item.topic_id}`} title="view topic">#</a>
            </div>
            <p class="archive-excerpt">{item.excerpt}</p>
            {#if failureActionId === item.action_id}
              <div class="archive-error"><strong>&#9888; {t('forum.metamodVoteFailed')}</strong></div>
            {/if}
            <div class="archive-verdict-row" aria-label="metamod verdicts">
              {#each VERDICTS as verdict (verdict)}
                <button
                  class="archive-btn"
                  type="button"
                  onclick={() => vote(item, verdict)}
                  disabled={workingActionId !== null}
                >
                  {t(`forum.metamod${verdict[0].toUpperCase()}${verdict.slice(1)}`)}
                </button>
              {/each}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</main>
{:else}
<div class="metamod-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <h1>{t('forum.metamod')}</h1>
    <p class="subtitle">{t('forum.metamodSubtitle')}</p>
  </header>

  {#if loading}
    <p class="muted"><span class="spinner"></span> {t('forum.loading')}</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if items.length === 0}
    <p class="empty">{t('forum.metamodEmpty')}</p>
    {#if poolTooSmall}
      <p class="dormant muted">{t('forum.metamodDormant')}</p>
    {/if}
  {:else}
    <p class="count muted">{auditCountLabel(total)}</p>
    <ul class="audit-list">
      {#each items as item (item.action_id)}
        <li class="card audit-item">
          <div class="meta-row">
            <span class="chip reason-chip">{t(`forum.reason.${item.reason}`)}</span>
            <span class="chip delta" class:pos={item.delta > 0} class:neg={item.delta < 0}>
              {item.delta > 0 ? '+' : ''}{item.delta}
            </span>
            <span class="muted date">{(item.created_at ?? '').slice(0, 10)}</span>
            <a class="context muted" href={`/forum/${item.topic_id}`}>#</a>
          </div>
          <p class="excerpt">{item.excerpt}</p>
          {#if failureActionId === item.action_id}
            <p class="error-card"><strong>⚠️ {t('forum.metamodVoteFailed')}</strong></p>
          {/if}
          <div class="verdict-row" aria-label="metamod verdicts">
            {#each VERDICTS as verdict (verdict)}
              <button
                class="verdict"
                class:fair={verdict === 'fair'}
                class:unfair={verdict === 'unfair'}
                class:unsure={verdict === 'unsure'}
                type="button"
                onclick={() => vote(item, verdict)}
                disabled={workingActionId !== null}
              >
                {t(`forum.metamod${verdict[0].toUpperCase()}${verdict.slice(1)}`)}
              </button>
            {/each}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
{/if}

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
    max-width: 820px;
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
  }
  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.8em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    text-decoration: none;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-muted { color: var(--archive-muted, #666666); }
  .archive-error {
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
    padding: 0.8em 1em;
    font-weight: 700;
    margin: 0.5rem 0;
  }
  .archive-empty {
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.6em 1em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
    margin: 1rem 0;
  }
  .archive-empty p { margin: 0.2em 0; }
  .archive-backlink { font-size: 0.9em; margin: 0.4em 0 0; }

  .archive-count { color: var(--archive-muted, #666666); font-size: 0.9em; margin: 0 0 0.75rem; }
  .archive-audit-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.75rem; }
  .archive-audit-item { border: 1px solid var(--archive-border, #dddddd); padding: 0.7em 1em; background: var(--archive-bg, #ffffff); }
  .archive-audit-meta { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; font-size: 0.88em; }
  .archive-chip { font-size: 0.78em; text-transform: uppercase; letter-spacing: 0.04em; border: 1px solid var(--archive-border, #dddddd); padding: 0.05em 0.4em; color: var(--archive-muted, #666666); }
  .archive-delta { font-weight: 700; text-transform: none; letter-spacing: 0; }
  .archive-delta.pos { color: #1e8e3e; border-color: #1e8e3e; }
  .archive-delta.neg { color: #c0392b; border-color: #c0392b; }
  .archive-excerpt { margin: 0.4em 0 0.6em; line-height: 1.55; overflow-wrap: break-word; }
  .archive-verdict-row { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .metamod-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .count { font-size: 0.9rem; margin: 0 0 0.75rem; }
  .audit-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .audit-item { padding: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .meta-row { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .chip { background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .reason-chip { background: #fdecea; color: #c0392b; }
  .delta { font-weight: 700; }
  .delta.pos { background: #e8f5e9; color: #1e8e3e; }
  .delta.neg { background: #fdecea; color: #c0392b; }
  .date { font-size: 0.8rem; }
  .context { font-size: 0.8rem; text-decoration: none; }
  .excerpt { margin: 0; line-height: 1.5; overflow-wrap: break-word; }
  .verdict-row { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .verdict { font-size: 0.85rem; padding: 0.3rem 0.8rem; border: 1px solid var(--color-border, #ddd); border-radius: 999px; background: var(--color-surface, #fff); cursor: pointer; }
  .verdict.fair { color: #1e8e3e; }
  .verdict.unfair { color: #c0392b; }
  .verdict.unsure { color: #8a6d1a; }
  .verdict:disabled { opacity: 0.6; cursor: default; }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0 0.5rem; }
  .dormant { text-align: center; max-width: 34rem; margin: 0.4rem auto 0; font-size: 0.85rem; }
</style>
