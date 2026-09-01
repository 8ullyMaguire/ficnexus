<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { listWorksInShelf, removeWorkFromShelf, getWork } from '$lib/api/social';
  import type { WorkShelfEntry, Work } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { formatWords, detectSite, relativeTime } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  const shelfId = $derived(($page?.params?.id as string | undefined) ?? (data as { id?: string } | undefined)?.id ?? '');

  let entries = $state<WorkShelfEntry[]>([]);
  let works = $state<Record<number, Work | null>>({});
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadWorks();
  });

  async function loadWorks() {
    loading = true; error = '';
    try {
      const res = await listWorksInShelf(Number(shelfId));
      if (res.err === 0) {
        entries = res.works;
        // Fetch work metadata
        await Promise.allSettled(
          entries.map(async (e) => {
            try {
              const w = await getWork(e.work_id);
              works[e.work_id] = w.work;
            } catch {
              works[e.work_id] = null;
            }
          })
        );
      } else {
        error = 'Failed to load shelf contents.';
      }
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  async function handleRemove(workId: number) {
    try {
      const res = await removeWorkFromShelf(Number(shelfId), workId);
      if (res.err === 0) {
        entries = entries.filter(e => e.work_id !== workId);
        delete works[workId];
      }
    } catch { error = 'Failed to remove work.'; }
  }

  function workMeta(workId: number): { title: string; author: string; source: string; words: number; chapters: number } | null {
    const w = works[workId];
    if (!w) return null;
    return {
      title: w.canonical_title,
      author: w.canonical_author,
      source: w.sources[0]?.source ?? '',
      words: w.sources[0]?.words ?? 0,
      chapters: w.sources[0]?.chapters ?? 0,
    };
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <a class="archive-link" href="/shelves">← Back to Shelves</a>
      <h1 class="archive-page-title">Shelf #{shelfId}</h1>
    </header>

    {#if loading}
      <p class="archive-muted"><span class="spinner"></span> Loading...</p>
    {:else if error}
      <p class="archive-error">{error}</p>
    {:else if entries.length === 0}
      <blockquote class="archive-summary">
        This shelf is empty. Add works to it from a work's page.
      </blockquote>
    {:else}
      <p class="archive-muted count">{entries.length} work{entries.length !== 1 ? 's' : ''}</p>
      <ul class="archive-list">
        {#each entries as e (e.id)}
          {@const meta = workMeta(e.work_id)}
          <li class="archive-list-row">
            <div class="archive-list-main">
              {#if meta}
                <a class="archive-link archive-list-title" href="/work/{e.work_id}">{meta.title}</a>
                <div class="archive-muted">
                  by {meta.author} · {detectSite(meta.source) || 'Unknown'}
                </div>
                <div class="archive-meta-line">{formatWords(meta.words)} words · {meta.chapters} chapters</div>
              {:else}
                <span class="archive-list-title">Work #{e.work_id}</span>
                <div class="archive-muted">Loading metadata…</div>
              {/if}
              <div class="archive-meta-line">Added {relativeTime(e.added_at)}</div>
            </div>
            <button class="archive-btn" onclick={() => handleRemove(e.work_id)}>Remove</button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="shelf-detail-page">
  <a href="/shelves" class="back-link">← Back to Shelves</a>
  <h1>📖 Shelf #{shelfId}</h1>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading...</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {:else if entries.length === 0}
    <div class="card empty">
      <p class="muted">This shelf is empty. Add works to it from a work's page.</p>
    </div>
  {:else}
    <p class="muted count">{entries.length} work{entries.length !== 1 ? 's' : ''}</p>
    <div class="work-list">
      {#each entries as e (e.id)}
        {@const meta = workMeta(e.work_id)}
        <div class="card work-item">
          <div class="work-info">
            {#if meta}
              <h3><a href="/work/{e.work_id}">{meta.title}</a></h3>
              <p class="muted">by {meta.author} · <span class="tag">{detectSite(meta.source) || 'Unknown'}</span></p>
              <p class="meta-line muted">{formatWords(meta.words)} words · {meta.chapters} chapters</p>
            {:else}
              <h3>Work #{e.work_id}</h3>
              <p class="muted"><span class="tag">Loading metadata…</span></p>
            {/if}
            <p class="added-time muted">Added {relativeTime(e.added_at)}</p>
          </div>
          <button class="btn-remove" onclick={() => handleRemove(e.work_id)}>Remove</button>
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
    margin: 0.2rem 0 0;
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
  .archive-summary {
    margin: 1rem 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .count { font-size: 0.9rem; margin: 0 0 0.6rem; }
  .archive-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-list-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.8rem;
    padding: 0.6rem 0.3rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    font-size: 0.92rem;
  }
  .archive-list-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-list-main { flex: 1; min-width: 0; }
  .archive-list-title {
    font-size: 1em;
    font-weight: 700;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .archive-link:hover {
    text-decoration: underline;
  }
  .archive-meta-line {
    font-size: 0.82rem;
    color: var(--archive-muted, #666666);
    margin-top: 0.15rem;
  }
  .archive-btn {
    padding: 0.28rem 0.7rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.85rem;
    cursor: pointer;
    flex-shrink: 0;
  }
  .archive-btn:hover { background: var(--archive-border, #dddddd); }

  /* ── Modern mode (scoped — no bleed) ── */
  .shelf-detail-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .shelf-detail-page .back-link { display: inline-block; margin-bottom: 0.5rem; color: var(--color-primary); text-decoration: none; font-size: 0.9rem; }
  .shelf-detail-page .back-link:hover { text-decoration: underline; }
  .shelf-detail-page .empty { text-align: center; padding: 2rem; }
  .shelf-detail-page .error-card { border-color: var(--color-error); }
  .shelf-detail-page .count { font-size: 0.9rem; margin-bottom: 0.8rem; }
  .shelf-detail-page .work-list { display: flex; flex-direction: column; gap: 0.6rem; }
  .shelf-detail-page .work-item { display: flex; justify-content: space-between; align-items: flex-start; gap: 1rem; padding: 0.8rem 1rem; }
  .shelf-detail-page .work-info { flex: 1; }
  .shelf-detail-page .work-info h3 { margin: 0 0 0.2rem; font-size: 1.05rem; }
  .shelf-detail-page .work-info h3 a { color: var(--color-text); text-decoration: none; }
  .shelf-detail-page .work-info h3 a:hover { color: var(--color-primary); }
  .shelf-detail-page .meta-line { margin: 0.2rem 0; font-size: 0.85rem; }
  .shelf-detail-page .added-time { font-size: 0.78rem; }
  .shelf-detail-page .btn-remove { padding: 0.3rem 0.6rem; font-size: 0.8rem; border-radius: var(--radius-sm); background: transparent; border: 1px solid var(--color-error); color: var(--color-error); cursor: pointer; flex-shrink: 0; }
  .shelf-detail-page .btn-remove:hover { background: var(--color-error); color: white; }
</style>
