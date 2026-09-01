<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { listShelves, createShelf, deleteShelf, getReadingList, getWork, updateReadingStatus } from '$lib/api/social';
  import type { Shelf, ReadingListItem, ReadingStatus, Work } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';

  const uiMode = $derived(getPref('uiMode'));

  // ── Reading-status shelves (want-to-read / reading / completed / dropped) ──
  const STATUS_SHELVES: { key: ReadingStatus; label: string; icon: string }[] = [
    { key: 'want_to_read', label: 'Want to Read', icon: '📋' },
    { key: 'reading', label: 'Currently Reading', icon: '📖' },
    { key: 'completed', label: 'Finished', icon: '✅' },
    { key: 'dropped', label: 'Dropped', icon: '❌' },
  ];

  let statusItems = $state<Record<ReadingStatus, ReadingListItem[]>>({
    want_to_read: [],
    reading: [],
    completed: [],
    dropped: [],
  });
  let works = $state<Record<number, Work | null>>({});
  let statusLoading = $state(true);

  async function loadStatusShelves() {
    statusLoading = true;
    try {
      const res = await getReadingList();
      if (res.err === 0) {
        const grouped: Record<ReadingStatus, ReadingListItem[]> = {
          want_to_read: [], reading: [], completed: [], dropped: [],
        };
        for (const item of res.reading_list) {
          grouped[item.status] = [...(grouped[item.status] ?? []), item];
        }
        statusItems = grouped;
        // Fetch work metadata for all items.
        await Promise.allSettled(
          res.reading_list.map(async (r) => {
            try {
              const w = await getWork(r.work_id);
              works[r.work_id] = w.work;
            } catch { works[r.work_id] = null; }
          })
        );
      }
    } catch { /* status shelves are best-effort */ }
    finally { statusLoading = false; }
  }

  async function handleStatusChange(workId: number, status: ReadingStatus | 'remove') {
    try {
      if (status === 'remove') {
        // No explicit "clear" endpoint — setting to dropped is not a remove.
        // We keep it simple: remove from the local view by moving to a
        // pseudo state is not possible; the server has no clear endpoint, so
        // we only offer the 4 statuses. Skip 'remove'.
        return;
      }
      await updateReadingStatus(workId, status);
      await loadStatusShelves();
    } catch { /* ignore */ }
  }

  // ── Custom shelves (existing UI) ─────────────────────────────────────
  let shelves = $state<Shelf[]>([]);
  let loading = $state(true);
  let error = $state('');

  let showCreateForm = $state(false);
  let newName = $state('');
  let newDescription = $state('');
  let newIsPublic = $state(false);
  let creating = $state(false);
  let createError = $state('');

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) { goto('/'); return; }
    await Promise.all([loadShelves(), loadStatusShelves()]);
  });

  async function loadShelves() {
    loading = true; error = '';
    try {
      const res = await listShelves();
      if (res.err === 0) shelves = res.shelves;
      else error = 'Failed to load shelves.';
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  async function handleCreate() {
    if (!newName.trim()) { createError = 'Name is required.'; return; }
    creating = true; createError = '';
    try {
      const res = await createShelf(newName.trim(), newDescription.trim(), newIsPublic);
      if (res.err === 0) {
        shelves = [...shelves, res.shelf];
        showCreateForm = false;
        newName = ''; newDescription = ''; newIsPublic = false;
      } else {
        createError = 'Failed to create shelf.';
      }
    } catch { createError = 'Network error.'; }
    finally { creating = false; }
  }

  async function handleDelete(id: number) {
    if (!confirm('Delete this shelf? Works inside will not be affected.')) return;
    try {
      const res = await deleteShelf(id);
      if (res.err === 0) {
        shelves = shelves.filter(s => s.id !== id);
      }
    } catch { error = 'Failed to delete shelf.'; }
  }

  function workMeta(workId: number): { title: string; author: string } | null {
    const w = works[workId];
    if (!w) return null;
    return { title: w.canonical_title, author: w.canonical_author };
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">My Shelves</h1>
      <p class="archive-muted">Track your reading status and organize fics into custom collections.</p>
    </header>

    <!-- ── Reading-status shelves ─────────────────────────────────── -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Reading Status</legend>
      {#if statusLoading}
        <p class="archive-muted"><span class="spinner"></span> Loading...</p>
      {:else}
        {#each STATUS_SHELVES as shelf (shelf.key)}
          <h3 class="archive-subheading">{shelf.label} <span class="archive-muted">({statusItems[shelf.key].length})</span></h3>
          {#if statusItems[shelf.key].length === 0}
            <p class="archive-muted">Nothing here yet.</p>
          {:else}
            <ul class="archive-list">
              {#each statusItems[shelf.key] as item (item.id)}
                {@const meta = workMeta(item.work_id)}
                <li class="archive-list-row">
                  {#if meta}
                    <a class="archive-link" href="/work/{item.work_id}">{meta.title}</a>
                    <span class="archive-muted"> by {meta.author}</span>
                  {:else}
                    <span class="archive-list-title">Work #{item.work_id}</span>
                  {/if}
                  <span class="archive-status-controls">
                    {#each STATUS_SHELVES as s (s.key)}
                      <button
                        class="archive-btn archive-btn-sm"
                        class:current={item.status === s.key}
                        onclick={() => handleStatusChange(item.work_id, s.key)}
                        title="Set status: {s.label}"
                        aria-pressed={item.status === s.key}
                      >{s.icon}</button>
                    {/each}
                  </span>
                </li>
              {/each}
            </ul>
          {/if}
        {/each}
      {/if}
    </fieldset>

    <!-- ── Custom shelves ─────────────────────────────────────────── -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Custom Shelves</legend>
      <div class="archive-actions">
        <button class="archive-btn archive-btn-primary" onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}>
          {showCreateForm ? 'Cancel' : '+ New Shelf'}
        </button>
      </div>

      {#if showCreateForm}
        <form class="archive-form" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
          <dl class="archive-dl">
            <div class="dl-row">
              <dt><label for="shelf-name">Shelf name</label></dt>
              <dd><input id="shelf-name" class="archive-input" type="text" placeholder="Shelf name" bind:value={newName} maxlength="100" /></dd>
            </div>
            <div class="dl-row">
              <dt><label for="shelf-desc">Description</label></dt>
              <dd><textarea id="shelf-desc" class="archive-input" placeholder="Description (optional)" bind:value={newDescription} rows="2"></textarea></dd>
            </div>
            <div class="dl-row">
              <dt></dt>
              <dd>
                <label class="archive-checkbox-label">
                  <input type="checkbox" bind:checked={newIsPublic} />
                  Public shelf
                </label>
              </dd>
            </div>
          </dl>
          {#if createError}
            <p class="archive-error">{createError}</p>
          {/if}
          <button class="archive-btn archive-btn-primary" type="submit" disabled={creating}>
            {creating ? 'Creating...' : 'Create'}
          </button>
        </form>
      {/if}

      {#if loading}
        <p class="archive-muted"><span class="spinner"></span> Loading...</p>
      {:else if error}
        <p class="archive-error">{error}</p>
      {:else if shelves.length === 0}
        <p class="archive-muted">You haven't created any custom shelves yet.</p>
      {:else}
        <ul class="archive-pref-list">
          {#each shelves as s (s.id)}
            <li class="archive-pref-item">
              <span class="archive-pref-label">
                <a class="archive-link" href="/shelves/{s.id}">{s.name}</a>
                {#if s.description}
                  <span class="archive-muted"> — {s.description}</span>
                {/if}
                <span class="archive-muted">
                  {#if s.is_public}Public{:else}Private{/if}
                  · Created {new Date(s.created_at).toLocaleDateString()}
                </span>
              </span>
              <button class="archive-btn" onclick={() => handleDelete(s.id)} title="Delete shelf">Delete</button>
            </li>
          {/each}
        </ul>
      {/if}
    </fieldset>
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="shelves-page">
  <h1>📚 My Shelves</h1>
  <p class="muted">Track your reading status and organize fics into custom collections.</p>

  <!-- ── Reading-status shelves ─────────────────────────────────────── -->
  <h2 class="section-title">Reading Status</h2>
  {#if statusLoading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading...</p></div>
  {:else}
    <div class="status-shelves">
      {#each STATUS_SHELVES as shelf (shelf.key)}
        <div class="card status-shelf">
          <div class="status-head">
            <span class="status-icon">{shelf.icon}</span>
            <span class="status-label">{shelf.label}</span>
            <span class="status-count muted">({statusItems[shelf.key].length})</span>
          </div>
          {#if statusItems[shelf.key].length === 0}
            <p class="muted status-empty">Nothing here yet.</p>
          {:else}
            <ul class="status-works">
              {#each statusItems[shelf.key] as item (item.id)}
                {@const meta = workMeta(item.work_id)}
                <li class="status-work">
                  {#if meta}
                    <a href="/work/{item.work_id}" class="status-title">{meta.title}</a>
                    <span class="muted">by {meta.author}</span>
                  {:else}
                    <span class="status-title">Work #{item.work_id}</span>
                  {/if}
                  <span class="status-controls">
                    {#each STATUS_SHELVES as s (s.key)}
                      <button
                        class="chip"
                        class:chip-active={item.status === s.key}
                        onclick={() => handleStatusChange(item.work_id, s.key)}
                        title="Set status: {s.label}"
                      >{s.icon}</button>
                    {/each}
                  </span>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <!-- ── Custom shelves ─────────────────────────────────────────────── -->
  <h2 class="section-title">Custom Shelves</h2>

  <div class="actions">
    <button class="btn btn-primary" onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}>
      {showCreateForm ? 'Cancel' : '+ New Shelf'}
    </button>
  </div>

  {#if showCreateForm}
    <div class="card create-form">
      <h3>Create Shelf</h3>
      <input
        type="text"
        placeholder="Shelf name"
        bind:value={newName}
        maxlength="100"
        class="input"
      />
      <textarea
        placeholder="Description (optional)"
        bind:value={newDescription}
        class="input"
        rows="2"
      ></textarea>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={newIsPublic} />
        Public shelf
      </label>
      {#if createError}
        <p class="error-text">{createError}</p>
      {/if}
      <button class="btn btn-primary" onclick={handleCreate} disabled={creating}>
        {creating ? 'Creating...' : 'Create'}
      </button>
    </div>
  {/if}

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading...</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {:else if shelves.length === 0}
    <div class="card empty"><p class="muted">You haven't created any custom shelves yet.</p></div>
  {:else}
    <div class="shelf-list">
      {#each shelves as s (s.id)}
        <div class="card shelf-item">
          <a href="/shelves/{s.id}" class="shelf-link">
            <div class="shelf-info">
              <span class="shelf-name">{s.name}</span>
              {#if s.description}
                <span class="shelf-desc muted">{s.description}</span>
              {/if}
              <span class="shelf-meta muted">
                {#if s.is_public}🌍 Public{:else}🔒 Private{/if}
                · Created {new Date(s.created_at).toLocaleDateString()}
              </span>
            </div>
          </a>
          <button class="btn-unfollow" onclick={() => handleDelete(s.id)} title="Delete shelf">Delete</button>
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
    margin: 0;
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
  .archive-subheading {
    font-size: 1em;
    margin: 0.6rem 0 0.2rem;
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
  .archive-list {
    list-style: none;
    margin: 0 0 0.6rem;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-list-row {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
    flex-wrap: wrap;
    padding: 0.4rem 0.3rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    font-size: 0.92rem;
  }
  .archive-list-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-list-title {
    font-weight: 600;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .archive-link:hover {
    text-decoration: underline;
  }
  .archive-status-controls {
    display: inline-flex;
    gap: 0.25rem;
    margin-left: auto;
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
  }
  .archive-btn:hover { background: var(--archive-border, #dddddd); }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-btn-sm { padding: 0.1rem 0.35rem; }
  .archive-btn.current {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .archive-actions { margin: 0.3rem 0 0.6rem; }
  .archive-form { margin: 0.4rem 0 0.8rem; }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.4rem 0 0.6rem;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-input {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9rem;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-input textarea { font-family: inherit; }
  .archive-checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .archive-pref-list {
    list-style: none;
    margin: 0.4rem 0 0;
    padding: 0;
  }
  .archive-pref-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.5rem 0.2rem;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
    font-size: 0.92rem;
  }
  .archive-pref-item:last-child { border-bottom: none; }
  .archive-pref-label {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  /* ── Modern mode (scoped — no bleed) ── */
  .shelves-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .shelves-page .section-title { margin: 1.4rem 0 0.5rem; font-size: 1.05rem; }
  .shelves-page .actions { margin: 0.8rem 0; }
  .shelves-page .empty { text-align: center; padding: 2rem; }
  .shelves-page .error-card { border-color: var(--color-error); }
  .shelves-page .create-form { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; margin-bottom: 1rem; }
  .shelves-page .input { padding: 0.5rem; border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-surface); color: var(--color-text); }
  .shelves-page .checkbox-label { display: flex; align-items: center; gap: 0.4rem; font-size: 0.9rem; cursor: pointer; }
  .shelves-page .status-shelves { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 0.7rem; }
  .shelves-page .status-shelf { padding: 0.8rem 1rem; }
  .shelves-page .status-head { display: flex; align-items: center; gap: 0.4rem; font-weight: 600; margin-bottom: 0.4rem; }
  .shelves-page .status-icon { font-size: 1.1rem; }
  .shelves-page .status-count { font-weight: 400; }
  .shelves-page .status-empty { font-size: 0.85rem; }
  .shelves-page .status-works { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.45rem; max-height: 240px; overflow-y: auto; }
  .shelves-page .status-work { display: flex; flex-direction: column; gap: 0.15rem; font-size: 0.9rem; }
  .shelves-page .status-title { color: var(--color-text); text-decoration: none; font-weight: 500; }
  .shelves-page .status-title:hover { color: var(--color-primary); }
  .shelves-page .status-controls { display: flex; gap: 0.25rem; margin-top: 0.1rem; }
  .shelves-page .chip { padding: 0.1rem 0.35rem; font-size: 0.8rem; border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-surface); cursor: pointer; }
  .shelves-page .chip:hover { background: var(--color-surface-2); }
  .shelves-page .chip-active { background: var(--color-primary); border-color: var(--color-primary); }
  .shelves-page .shelf-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .shelves-page .shelf-item { display: flex; align-items: center; gap: 0.8rem; padding: 0.8rem 1rem; }
  .shelves-page .shelf-link { flex: 1; text-decoration: none; color: inherit; }
  .shelves-page .shelf-info { display: flex; flex-direction: column; gap: 0.2rem; }
  .shelves-page .shelf-name { font-weight: 600; font-size: 1.05rem; }
  .shelves-page .shelf-desc { font-size: 0.85rem; }
  .shelves-page .shelf-meta { font-size: 0.78rem; }
  .shelves-page .btn-unfollow { padding: 0.3rem 0.6rem; font-size: 0.8rem; border-radius: var(--radius-sm); background: transparent; border: 1px solid var(--color-error); color: var(--color-error); cursor: pointer; }
  .shelves-page .btn-unfollow:hover { background: var(--color-error); color: white; }
</style>
