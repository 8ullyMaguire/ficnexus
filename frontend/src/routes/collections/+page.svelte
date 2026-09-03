<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { listReadingLists, createReadingList, updateReadingList, deleteReadingList } from '$lib/api/social';
  import type { ReadingList } from '$lib/api/social-types';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let lists = $state<ReadingList[]>([]);
  let loading = $state(true);
  let error = $state('');
  let showCreateForm = $state(false);
  let newTitle = $state('');
  let newDescription = $state('');
  let newIsPublic = $state(true);
  let creating = $state(false);
  let editingId = $state<number | null>(null);
  let editTitle = $state('');
  let editDescription = $state('');

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    await loadLists();
  });

  async function loadLists() {
    loading = true;
    error = '';
    try {
      const res = await listReadingLists();
      if (res.err === 0) lists = res.lists;
    } catch {
      error = 'Network error loading collections.';
    } finally {
      loading = false;
    }
  }

  async function handleCreate() {
    if (!newTitle.trim()) return;
    creating = true;
    try {
      const res = await createReadingList(newTitle, newDescription, newIsPublic);
      if (res.err === 0) {
        showCreateForm = false;
        newTitle = '';
        newDescription = '';
        await loadLists();
      }
    } catch {
      error = 'Failed to create collection.';
    } finally {
      creating = false;
    }
  }

  async function startEdit(list: ReadingList) {
    editingId = list.id;
    editTitle = list.title;
    editDescription = list.description;
  }

  async function saveEdit() {
    if (editingId === null) return;
    try {
      const res = await updateReadingList(editingId, { title: editTitle, description: editDescription });
      if (res.err === 0) {
        editingId = null;
        await loadLists();
      }
    } catch {
      error = 'Failed to update collection.';
    }
  }

  async function deleteList(id: number) {
    if (!confirm('Delete this collection?')) return;
    try {
      const res = await deleteReadingList(id);
      if (res.err === 0) await loadLists();
    } catch {
      error = 'Failed to delete collection.';
    }
  }
</script>

{#if uiMode === 'archive'}
<div class="archive-header">
  <h1 class="archive-title">Collections</h1>
  {#if auth.isLoggedIn}
    <p class="archive-subnav">
      <ArchiveButton onclick={() => showCreateForm = !showCreateForm}>
        {showCreateForm ? 'Cancel' : 'New Collection'}
      </ArchiveButton>
    </p>
  {/if}
</div>

{#if !auth.isLoggedIn}
  <div class="archive-empty">
    <p>Log in to create and browse collections.</p>
    <a href="/login">Log In</a>
  </div>
{:else if showCreateForm}
  <div class="card">
    <h3>Create New Collection</h3>
    <dl>
      <div class="dl-row">
        <dt><label for="list-title">Title</label></dt>
        <dd><input id="list-title" type="text" bind:value={newTitle} placeholder="Collection name" /></dd>
      </div>
      <div class="dl-row">
        <dt><label for="list-desc">Description</label></dt>
        <dd><textarea id="list-desc" bind:value={newDescription} rows="3" placeholder="Optional description"></textarea></dd>
      </div>
      <div class="dl-row">
        <dt><label for="list-public">Visibility</label></dt>
        <dd>
          <select id="list-public" bind:value={newIsPublic}>
            <option value={true}>Public</option>
            <option value={false}>Private</option>
          </select>
        </dd>
      </div>
    </dl>
    <p class="submit actions">
      <ArchiveButton onclick={handleCreate} disabled={creating}>
        {creating ? 'Creating...' : 'Create Collection'}
      </ArchiveButton>
      <ArchiveButton onclick={() => showCreateForm = false}>Cancel</ArchiveButton>
    </p>
  </div>

{:else if loading}
  <div class="archive-skeleton-list">
    {#each Array(3) as _, i}
      <div class="skeleton-collection" aria-hidden="true"></div>
    {/each}
  </div>

{:else if error}
  <div class="archive-error"><p>{error}</p></div>

{:else if lists.length === 0}
  <div class="archive-empty">
    <p>No collections yet. Create one to start organizing your reading list.</p>
  </div>

{:else}
  <div class="collection-list">
    {#each lists as list (list.id)}
      <div class="collection-card">
        <div class="collection-header">
          {#if editingId === list.id}
            <input type="text" bind:value={editTitle} class="edit-title" />
            <input type="text" bind:value={editDescription} class="edit-desc" />
            <p class="submit actions">
              <ArchiveButton onclick={saveEdit}>Save</ArchiveButton>
              <ArchiveButton onclick={() => editingId = null}>Cancel</ArchiveButton>
            </p>
          {:else}
            <h3><a href={`/collections/${list.id}`}>{list.title}</a></h3>
            <p class="collection-desc">{list.description}</p>
            <dl class="stats">
              <div class="stat"><dt>Items</dt><dd>{list.item_count}</dd></div>
              <div class="stat"><dt>Visibility</dt><dd>{list.is_public ? 'Public' : 'Private'}</dd></div>
              <div class="stat"><dt>Updated</dt><dd>{list.updated_at}</dd></div>
            </dl>
            <div class="collection-actions">
              <ArchiveButton onclick={() => startEdit(list)}>Edit</ArchiveButton>
              <ArchiveButton onclick={() => deleteList(list.id)}>Delete</ArchiveButton>
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

{:else}
  <!-- Modern mode -->
  <div class="dashboard-page">
    <h1>Collections</h1>
    <p class="muted">Switch to archive mode for the full collections UI.</p>
  </div>
{/if}

<style>
  .collection-list { display: flex; flex-direction: column; gap: 1rem; }
  .collection-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 1rem;
    background: var(--color-surface-1);
  }
  .collection-header h3 { margin: 0 0 0.3rem; }
  .collection-header h3 a { color: var(--color-text); text-decoration: none; }
  .collection-desc { color: var(--color-muted); font-size: 0.9rem; margin: 0.3rem 0; }
  .collection-actions { display: flex; gap: 0.5rem; margin-top: 0.5rem; }
  .edit-title { width: 100%; font-size: 1.2rem; margin-bottom: 0.3rem; }
  .edit-desc { width: 100%; margin-bottom: 0.5rem; }
  dl.stats { display: flex; gap: 1.5rem; margin: 0.5rem 0; }
  .stat { display: flex; flex-direction: column; align-items: center; }
  .stat dt { font-size: 0.75rem; color: var(--color-muted); }
  .stat dd { font-size: 1.1rem; font-weight: 600; margin: 0; }
  .archive-subnav { margin-top: 0.5rem; }
</style>
