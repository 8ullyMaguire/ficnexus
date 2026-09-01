<script lang="ts">
  import { onMount } from 'svelte';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let searchQuery = $state('');
  let fics = $state<Array<{ url_id: string; title: string; author: string; source: string }>>([]);
  let selected = $state(new Set<string>());
  let loading = $state(false);
  let searching = $state(false);
  let results = $state<Array<{ url_id: string; status: string; message?: string; suggested_count?: number }>>([]);
  let actionInProgress = $state(false);

  let debounceTimer: ReturnType<typeof setTimeout>;

  let allSelected = $derived(fics.length > 0 && fics.every(f => selected.has(f.url_id)));

  $effect(() => {
    const q = searchQuery;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => searchFics(q), 300);
  });

  async function searchFics(q: string) {
    if (q.trim().length < 1) {
      fics = [];
      return;
    }
    searching = true;
    try {
      const params = new URLSearchParams({ q: q.trim(), limit: '50' });
      const res = await adminFetch(`/api/admin/fics/search?${params}`);
      if (res.ok) {
        const data = await res.json();
        fics = data.fics ?? [];
      } else {
        fics = [];
      }
    } catch {
      fics = [];
    } finally {
      searching = false;
    }
  }

  function toggleSelect(urlId: string) {
    const next = new Set(selected);
    if (next.has(urlId)) {
      next.delete(urlId);
    } else {
      next.add(urlId);
    }
    selected = next;
  }

  function toggleAll() {
    if (allSelected) {
      selected = new Set();
    } else {
      selected = new Set(fics.map(f => f.url_id));
    }
  }

  async function runAction(endpoint: string, label: string) {
    if (selected.size === 0 || actionInProgress) return;
    actionInProgress = true;
    results = [];
    try {
      const res = await adminFetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url_ids: Array.from(selected) }),
      });
      if (res.ok) {
        const data = await res.json();
        results = data.results ?? [];
      } else {
        let msg = `HTTP ${res.status}`;
        try {
          const body = await res.json();
          if (body?.msg) msg = body.msg;
        } catch {}
        results = [{ url_id: '', status: 'error', message: msg }];
      }
    } catch (e) {
      results = [{ url_id: '', status: 'error', message: String(e) }];
    } finally {
      actionInProgress = false;
    }
  }

  function doRefresh() {
    runAction('/api/admin/bulk/refresh', 'Re-scrape');
  }

  function doAutoTag() {
    runAction('/api/admin/bulk/auto-tag', 'Auto-tag');
  }
</script>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Bulk Actions</h1>
        <p class="archive-summary">Search fics, select them, then re-scrape or auto-tag the whole selection in one call.</p>
      </header>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Search fics</legend>
        <dl class="archive-dl">
          <dt><label for="ba-search">Title search</label></dt>
          <dd><input id="ba-search" class="archive-input" type="text" placeholder="Search fics by title…" bind:value={searchQuery} /></dd>
        </dl>
        {#if searching}<p class="archive-note">Searching…</p>{/if}
      </fieldset>
      {#if fics.length > 0}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Select fics <span class="archive-note">({selected.size} selected)</span></legend>
          <dl class="archive-dl">
            <div class="dl-row">
              <dt><label><input type="checkbox" checked={allSelected} onchange={toggleAll} /> Select all</label></dt>
            </div>
            {#each fics as fic (fic.url_id)}
              <div class="dl-row">
                <dt>
                  <label>
                    <input type="checkbox" checked={selected.has(fic.url_id)} onchange={() => toggleSelect(fic.url_id)} />
                    {' '}
                    {fic.title}
                  </label>
                </dt>
                <dd class="archive-note">{fic.author} · {fic.source}</dd>
              </div>
            {/each}
          </dl>
          <div class="archive-form-actions">
            <button class="archive-btn archive-btn-primary" type="button" onclick={doRefresh} disabled={selected.size === 0 || actionInProgress}>🔄 Re-scrape Selected</button>
            {' '}
            <button class="archive-btn" type="button" onclick={doAutoTag} disabled={selected.size === 0 || actionInProgress}>🏷️ Auto-tag Selected</button>
          </div>
        </fieldset>
      {:else if searchQuery.trim().length > 0 && !searching}
        <p class="archive-note">No fics found.</p>
      {/if}
      {#if actionInProgress}
        <p class="archive-note">Processing {selected.size} fic(s)…</p>
      {/if}
      {#if results.length > 0}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Results</legend>
          <table class="archive-table">
            <thead><tr><th>URL ID</th><th>Status</th><th>Message</th></tr></thead>
            <tbody>
              {#each results as r}
                <tr>
                  <td class="archive-mono">{r.url_id || '—'}</td>
                  <td>{#if r.status === 'ok'}✅ ok{:else}❌ error{/if}</td>
                  <td>{r.message ?? ''}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </fieldset>
      {/if}
    </div>
  </main>
{:else}
<h1>⚡ Bulk Actions</h1>

<div class="search-section">
  <input
    type="text"
    placeholder="Search fics by title…"
    bind:value={searchQuery}
    class="search-input"
  />
  {#if searching}
    <span class="spinner">Searching…</span>
  {/if}
</div>

{#if fics.length > 0}
  <div class="fics-table">
    <div class="table-header">
      <label class="checkbox-label">
        <input type="checkbox" checked={allSelected} onchange={toggleAll} />
        Select all
      </label>
      <span class="count">{selected.size} selected</span>
    </div>

    <div class="table-body">
      {#each fics as fic (fic.url_id)}
        <div class="fic-row">
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={selected.has(fic.url_id)}
              onchange={() => toggleSelect(fic.url_id)}
            />
          </label>
          <div class="fic-info">
            <div class="fic-title">{fic.title}</div>
            <div class="fic-meta">{fic.author} · {fic.source}</div>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="actions">
    <button
      class="btn btn-primary"
      onclick={doRefresh}
      disabled={selected.size === 0 || actionInProgress}
    >
      🔄 Re-scrape Selected
    </button>
    <button
      class="btn btn-secondary"
      onclick={doAutoTag}
      disabled={selected.size === 0 || actionInProgress}
    >
      🏷️ Auto-tag Selected
    </button>
  </div>

  {#if actionInProgress}
    <p class="loading-text">Processing {selected.size} fic(s)…</p>
  {/if}

  {#if results.length > 0}
    <div class="results-section">
      <h2>Results</h2>
      <table class="results-table">
        <thead>
          <tr>
            <th>URL ID</th>
            <th>Status</th>
            <th>Message</th>
          </tr>
        </thead>
        <tbody>
          {#each results as r}
            <tr>
              <td class="mono">{r.url_id || '—'}</td>
              <td>
                {#if r.status === 'ok'}
                  ✅ ok
                {:else}
                  ❌ error
                {/if}
              </td>
              <td>{r.message ?? ''}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
{:else if searchQuery.trim().length > 0 && !searching}
  <p class="empty">No fics found.</p>
{/if}
{/if}
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
  .archive-content { max-width: 860px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 1em 1.25em;
    margin: 1.2rem 0;
  }
  .archive-legend {
    font-weight: 700;
    font-size: 1.05em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.4em;
  }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt { font-weight: 700; font-size: 0.9em; margin: 0.85em 0 0.2em; }
  .archive-dl dt:first-child { margin-top: 0; }
  .archive-dl dd { margin: 0; }
  .archive-dl .ba-select-all dt { font-weight: 700; }
  .archive-dl label { font-weight: 400; cursor: pointer; }
  .archive-input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
  }
  .archive-form-actions { margin-top: 0.9em; }
  .archive-btn {
    display: inline-block;
    padding: 0.35em 1em;
    font-size: 0.88em;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    cursor: pointer;
    font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-btn-primary {
    background: var(--archive-link, #990000);
    color: #ffffff;
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover:not(:disabled) { background: #7a0000; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td {
    text-align: left;
    padding: 0.45em 0.6em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-table th {
    font-weight: 700;
    font-size: 0.85em;
    border-bottom: 2px solid var(--archive-border, #dddddd);
  }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; word-break: break-all; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.88em; }
  p.archive-note { margin: 0.5em 0; }

  /* ── Modern mode ──────────────────────────────────────────── */
  h1 { margin-bottom: 1rem; }
  .search-section { display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem; }
  .search-input { flex: 1; padding: 0.6rem 1rem; border: 1px solid var(--color-border, #ccc); border-radius: var(--radius-sm, 4px); font-size: 1rem; }
  .spinner { color: var(--color-text-secondary, #888); font-size: 0.9rem; }
  .fics-table { border: 1px solid var(--color-border, #ccc); border-radius: var(--radius-sm, 4px); overflow: hidden; margin-bottom: 1rem; }
  .table-header { display: flex; align-items: center; gap: 1rem; padding: 0.5rem 1rem; background: var(--color-surface, #f5f5f5); border-bottom: 1px solid var(--color-border, #ccc); }
  .count { font-size: 0.9rem; color: var(--color-text-secondary, #888); }
  .table-body { max-height: 400px; overflow-y: auto; }
  .fic-row { display: flex; align-items: center; gap: 0.75rem; padding: 0.4rem 1rem; border-bottom: 1px solid var(--color-border, #eee); }
  .fic-row:hover { background: var(--color-surface-2, #f9f9f9); }
  .fic-info { flex: 1; min-width: 0; }
  .fic-title { font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .fic-meta { font-size: 0.85rem; color: var(--color-text-secondary, #888); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .checkbox-label { display: flex; align-items: center; gap: 0.4rem; cursor: pointer; }
  .actions { display: flex; gap: 0.75rem; margin-bottom: 1rem; }
  .btn { padding: 0.5rem 1.2rem; border: none; border-radius: var(--radius-sm, 4px); cursor: pointer; font-size: 0.95rem; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-primary { background: var(--color-primary, #2563eb); color: white; }
  .btn-secondary { background: var(--color-surface-2, #e5e7eb); color: var(--color-text, #111); }
  .loading-text { color: var(--color-text-secondary, #888); }
  .empty { color: var(--color-text-secondary, #888); }
  .results-section { margin-top: 1rem; }
  .results-section h2 { font-size: 1.1rem; margin-bottom: 0.5rem; }
  .results-table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  .results-table th, .results-table td { text-align: left; padding: 0.4rem 0.8rem; border-bottom: 1px solid var(--color-border, #eee); }
  .results-table th { background: var(--color-surface, #f5f5f5); }
  .mono { font-family: monospace; }
</style>
