<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface ConsensusItem {
    kind: string;
    id: number | string;
    title: string;
    detail: string;
    status: string;
    score: number;
    created_at: string;
    link: string;
  }

  let items = $state<ConsensusItem[]>([]);
  let counts = $state<Record<string, number>>({});
  let loading = $state(true);
  let error = $state('');
  let kindFilter = $state('all');
  let q = $state('');

  const KIND_LABELS: Record<string, string> = {
    content: 'Body fixes',
    metadata: 'Metadata fixes',
    flag: 'Tag flags',
    alias: 'Tag aliases',
    author_merge: 'Author merges',
    roadmap: 'Roadmap consensus',
  };

  async function load() {
    loading = true;
    error = '';
    try {
      const params = new URLSearchParams();
      if (kindFilter !== 'all') params.set('kind', kindFilter);
      if (q.trim()) params.set('q', q.trim());
      const qs = params.toString() ? `?${params.toString()}` : '';
      const res = await adminFetch(`/api/curator/consensus${qs}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      items = body.items ?? [];
      counts = body.counts ?? {};
    } catch (e) {
      error = `Failed to load consensus feed: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      goto('/login');
      return;
    }
    load();
  });

  function fmtTime(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function kindLabel(k: string): string {
    return KIND_LABELS[k] ?? k;
  }
</script>
{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Curator Consensus</h1>
      <p class="archive-summary">
        Every peer-voted surface in one feed: body fixes, metadata corrections, tag flags and aliases,
        author merges, and roadmap consensus. Use the filters to narrow the view.
      </p>
      <div class="arc-modlog-controls">
        <select class="arc-select" bind:value={kindFilter} onchange={load} aria-label="Filter by type">
          <option value="all">All types</option>
          {#each Object.keys(KIND_LABELS) as k}
            <option value={k}>{kindLabel(k)}{counts[k] !== undefined ? ` (${counts[k]})` : ''}</option>
          {/each}
        </select>
        <input class="arc-select" type="text" placeholder="Search title or detail…" bind:value={q}
               onkeydown={(e) => e.key === 'Enter' && load()} aria-label="Search consensus items" />
        <button class="archive-btn" type="button" onclick={load}>&#8635; Refresh</button>
      </div>
    </header>

    {#if loading}
      <blockquote class="archive-empty"><p>Loading&hellip;</p></blockquote>
    {:else if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
    {:else if items.length === 0}
      <blockquote class="archive-empty"><p>No consensus items match these filters.</p></blockquote>
    {:else}
      <div class="table-wrap">
        <table class="archive-table">
          <thead><tr><th>When</th><th>Type</th><th>Status</th><th>Title</th><th>Detail</th><th>Score</th></tr></thead>
          <tbody>
            {#each items as it}
              <tr>
                <td class="mono">{fmtTime(it.created_at)}</td>
                <td><span class="badge">{kindLabel(it.kind)}</span></td>
                <td>{it.status}</td>
                <td><a href={it.link}>{it.title}</a></td>
                <td class="mono detail">{it.detail}</td>
                <td class="mono">{it.score}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</main>
{:else}
<div class="consensus-page">
  <header class="page-head">
    <h1>Curator Consensus</h1>
    <p class="subtitle">Every peer-voted surface in one filterable feed.</p>
    <div class="controls">
      <select bind:value={kindFilter} onchange={load} aria-label="Filter by type">
        <option value="all">All types</option>
        {#each Object.keys(KIND_LABELS) as k}
          <option value={k}>{kindLabel(k)}{counts[k] !== undefined ? ` (${counts[k]})` : ''}</option>
        {/each}
      </select>
      <input type="text" placeholder="Search…" bind:value={q}
             onkeydown={(e) => e.key === 'Enter' && load()} aria-label="Search consensus items" />
      <button class="btn" onclick={load}>Refresh</button>
    </div>
  </header>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>{error}</strong></div>
  {:else if items.length === 0}
    <p class="empty">No consensus items match these filters.</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead><tr><th>When</th><th>Type</th><th>Status</th><th>Title</th><th>Detail</th><th>Score</th></tr></thead>
        <tbody>
          {#each items as it}
            <tr>
              <td class="mono">{fmtTime(it.created_at)}</td>
              <td><span class="badge">{kindLabel(it.kind)}</span></td>
              <td>{it.status}</td>
              <td><a href={it.link}>{it.title}</a></td>
              <td class="mono detail">{it.detail}</td>
              <td class="mono">{it.score}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
{/if}

<style>
  .controls { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-top: 0.75rem; }
  .detail { max-width: 340px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .mono { font-family: monospace; font-size: 0.85em; }
  .table-wrap { overflow-x: auto; }
</style>

