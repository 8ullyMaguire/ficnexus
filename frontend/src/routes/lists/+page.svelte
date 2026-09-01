<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { listReadingLists, createReadingList, deleteReadingList } from '$lib/api/social';
  import type { ReadingList } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';

  const uiMode = $derived(getPref('uiMode'));

  let lists = $state<ReadingList[]>([]);
  let loading = $state(true);
  let error = $state('');

  // Create form
  let showCreateForm = $state(false);
  let newTitle = $state('');
  let newDescription = $state('');
  let newIsPublic = $state(false);
  let creating = $state(false);
  let createError = $state('');

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadLists();
  });

  async function loadLists() {
    loading = true; error = '';
    try {
      const res = await listReadingLists();
      if (res.err === 0) lists = res.lists;
      else error = 'Failed to load lists.';
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  async function handleCreate() {
    if (!newTitle.trim()) { createError = 'Title is required.'; return; }
    creating = true; createError = '';
    try {
      const res = await createReadingList(newTitle.trim(), newDescription.trim(), newIsPublic);
      if (res.err === 0) {
        lists = [res.list, ...lists];
        showCreateForm = false;
        newTitle = ''; newDescription = ''; newIsPublic = false;
      } else {
        createError = 'Failed to create list.';
      }
    } catch { createError = 'Network error.'; }
    finally { creating = false; }
  }

  async function handleDelete(id: number) {
    if (!confirm('Delete this list? The works themselves are not affected.')) return;
    try {
      const res = await deleteReadingList(id);
      if (res.err === 0) {
        lists = lists.filter(l => l.id !== id);
      }
    } catch { error = 'Failed to delete list.'; }
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Reading Lists</h1>
      <p class="archive-muted">Curate ordered bundles of fics — with your own one-line blurbs for each work.</p>
    </header>

    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Your Lists</legend>
      <div class="archive-actions">
        <button class="archive-btn archive-btn-primary" onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}>
          {showCreateForm ? 'Cancel' : '+ New List'}
        </button>
      </div>

      {#if showCreateForm}
        <form class="archive-form" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
          <dl class="archive-dl">
            <div class="dl-row">
              <dt><label for="list-title">List title</label></dt>
              <dd><input id="list-title" class="archive-input" type="text" placeholder="List title" bind:value={newTitle} maxlength="200" /></dd>
            </div>
            <div class="dl-row">
              <dt><label for="list-desc">Description</label></dt>
              <dd><textarea id="list-desc" class="archive-input" placeholder="Description (optional)" bind:value={newDescription} rows="2"></textarea></dd>
            </div>
            <div class="dl-row">
              <dt></dt>
              <dd>
                <label class="archive-checkbox-label">
                  <input type="checkbox" bind:checked={newIsPublic} />
                  Public list (anyone with the link can view)
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
      {:else if lists.length === 0}
        <blockquote class="archive-summary">
          You haven't created any reading lists yet.
          A reading list is an ordered bundle of fics — like a curated rec list you can share.
        </blockquote>
      {:else}
        <ul class="archive-pref-list">
          {#each lists as l (l.id)}
            <li class="archive-pref-item">
              <span class="archive-pref-label">
                <a class="archive-link" href="/lists/{l.id}">{l.title}</a>
                {#if l.description}
                  <span class="archive-muted"> — {l.description}</span>
                {/if}
                <span class="archive-muted">
                  {#if l.is_public}Public{:else}Private{/if}
                  · {l.item_count} work{l.item_count !== 1 ? 's' : ''}
                </span>
              </span>
              <button class="archive-btn" onclick={() => handleDelete(l.id)} title="Delete list">Delete</button>
            </li>
          {/each}
        </ul>
      {/if}
    </fieldset>
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="lists-page">
  <h1>📚 Reading Lists</h1>
  <p class="muted">Curate ordered bundles of fics — with your own one-line blurbs for each work.</p>

  <div class="actions">
    <button class="btn btn-primary" onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}>
      {showCreateForm ? 'Cancel' : '+ New List'}
    </button>
  </div>

  {#if showCreateForm}
    <div class="card create-form">
      <h3>Create Reading List</h3>
      <input
        type="text"
        placeholder="List title"
        bind:value={newTitle}
        maxlength="200"
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
        Public list (anyone with the link can view)
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
  {:else if lists.length === 0}
    <div class="card empty">
      <p class="muted">You haven't created any reading lists yet.</p>
      <p class="muted">A reading list is an ordered bundle of fics — like a curated rec list you can share.</p>
    </div>
  {:else}
    <div class="list-grid">
      {#each lists as l (l.id)}
        <div class="card list-item">
          <a href="/lists/{l.id}" class="list-link">
            <div class="list-info">
              <span class="list-title">{l.title}</span>
              {#if l.description}
                <span class="list-desc muted">{l.description}</span>
              {/if}
              <span class="list-meta muted">
                {#if l.is_public}🌍 Public{:else}🔒 Private{/if}
                · {l.item_count} work{l.item_count !== 1 ? 's' : ''}
              </span>
            </div>
          </a>
          <button class="btn-danger-ghost" onclick={() => handleDelete(l.id)} title="Delete list">Delete</button>
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
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    font-size: 0.9rem;
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
  .archive-summary {
    margin: 1rem 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
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
  .archive-checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .archive-link:hover {
    text-decoration: underline;
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
  .archive-btn-primary {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .archive-pref-list {
    list-style: none;
    margin: 0.4rem 0 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-pref-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.55rem 0.2rem;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
    font-size: 0.92rem;
  }
  .archive-pref-item:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-pref-item:last-child { border-bottom: none; }
  .archive-pref-label {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  /* ── Modern mode (scoped — no bleed) ── */
  .lists-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .lists-page .actions { margin: 1rem 0; }
  .lists-page .empty { text-align: center; padding: 2rem; }
  .lists-page .error-card { border-color: var(--color-error); }
  .lists-page .create-form { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; margin-bottom: 1rem; }
  .lists-page .input { padding: 0.5rem; border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-surface); color: var(--color-text); }
  .lists-page .checkbox-label { display: flex; align-items: center; gap: 0.4rem; font-size: 0.9rem; cursor: pointer; }
  .lists-page .list-grid { display: flex; flex-direction: column; gap: 0.5rem; }
  .lists-page .list-item { display: flex; align-items: center; gap: 0.8rem; padding: 0.8rem 1rem; }
  .lists-page .list-link { flex: 1; text-decoration: none; color: inherit; }
  .lists-page .list-info { display: flex; flex-direction: column; gap: 0.2rem; }
  .lists-page .list-title { font-weight: 600; font-size: 1.05rem; }
  .lists-page .list-desc { font-size: 0.85rem; }
  .lists-page .list-meta { font-size: 0.78rem; }
  .lists-page .btn-danger-ghost { padding: 0.3rem 0.6rem; font-size: 0.8rem; border-radius: var(--radius-sm); background: transparent; border: 1px solid var(--color-error); color: var(--color-error); cursor: pointer; }
  .lists-page .btn-danger-ghost:hover { background: var(--color-error); color: white; }
</style>
