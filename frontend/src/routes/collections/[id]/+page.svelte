<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { getReadingListDetail, addReadingListItem, removeReadingListItem } from '$lib/api/social';
  import type { ReadingListDetail } from '$lib/api/social-types';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props<{ data?: { list_id: number } }>();
  const listId = $derived(data?.list_id ?? 0);

  let listDetail = $state<ReadingListDetail | null>(null);
  let loading = $state(true);
  let error = $state('');
  let addingWorkId = $state<number | null>(null);
  let newWorkId = $state('');

  onMount(async () => {
    await auth.init();
    await loadDetail();
  });

  async function loadDetail() {
    loading = true;
    error = '';
    try {
      const res = await getReadingListDetail(listId);
      if (res.err === 0) listDetail = res;
    } catch {
      error = 'Network error loading collection.';
    } finally {
      loading = false;
    }
  }

  async function addItem() {
    if (!newWorkId.trim()) return;
    addingWorkId = parseInt(newWorkId, 10);
    try {
      const res = await addReadingListItem(listId, addingWorkId);
      if (res.err === 0) {
        newWorkId = '';
        await loadDetail();
      }
    } catch {
      error = 'Failed to add item.';
    } finally {
      addingWorkId = null;
    }
  }

  async function removeItem(work_id: number) {
    try {
      const res = await removeReadingListItem(listId, work_id);
      if (res.err === 0) await loadDetail();
    } catch {
      error = 'Failed to remove item.';
    }
  }
</script>

{#if uiMode === 'archive'}
{#if loading}
  <div class="archive-skeleton-list">
    <div class="skeleton-blurb" aria-hidden="true"></div>
  </div>
{:else if error}
  <div class="archive-error"><p>{error}</p></div>
{:else if listDetail}
  <header class="archive-header">
    <h1 class="archive-title">{listDetail.list.title}</h1>
    <p class="muted">{listDetail.list.description}</p>
    <dl class="stats">
      <div class="stat"><dt>Items</dt><dd>{listDetail.items.length}</dd></div>
      <div class="stat"><dt>Visibility</dt><dd>{listDetail.list.is_public ? 'Public' : 'Private'}</dd></div>
    </dl>
  </header>

  {#if listDetail.is_owner}
    <div class="card">
      <h3>Add Work to Collection</h3>
      <dl>
        <div class="dl-row">
          <dt><label for="work-id">Work ID</label></dt>
          <dd><input id="work-id" type="number" bind:value={newWorkId} placeholder="Enter work ID" /></dd>
        </div>
      </dl>
      <p class="submit actions">
        <ArchiveButton onclick={addItem} disabled={addingWorkId !== null}>
          {addingWorkId !== null ? 'Adding...' : 'Add to Collection'}
        </ArchiveButton>
      </p>
    </div>
  {/if}

  <h3>Works in this collection</h3>
  {#if listDetail.items.length === 0}
    <p class="muted">No works in this collection yet.</p>
  {:else}
    <div class="collection-items">
      {#each listDetail.items as item (item.id)}
        <div class="collection-item">
          <span class="position">#{item.position}</span>
          <a href={`/work/${item.work_id}`}>{item.title}</a>
          <span class="muted">by {item.author}</span>
          {#if listDetail.is_owner}
            <ArchiveButton onclick={() => removeItem(item.work_id)}>Remove</ArchiveButton>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
{/if}
{:else}
  <h1>{listDetail?.list.title || 'Collection'}</h1>
  <p class="muted">{listDetail?.list.description}</p>
  {#if listDetail}
    <div class="collection-items">
      {#each listDetail.items as item (item.id)}
        <div class="collection-item">{item.position}. <a href={`/work/${item.work_id}`}>{item.title}</a></div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .stats { display: flex; gap: 2rem; margin: 0.8rem 0; }
  .stat { text-align: center; }
  .collection-items { display: flex; flex-direction: column; gap: 0.4rem; }
  .collection-item { display: flex; align-items: center; gap: 0.5rem; }
  .position { font-weight: 600; color: var(--color-muted); min-width: 2rem; }
</style>
