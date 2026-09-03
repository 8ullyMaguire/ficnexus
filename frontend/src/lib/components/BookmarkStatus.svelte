<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { listBookmarks, addBookmark, removeBookmark } from '$lib/api/social';

  let { workId, initialBookmarked = false }: { workId: number; initialBookmarked?: boolean } = $props();

  let isBookmarked = $state(initialBookmarked);
  let bookmarkNotes = $state('');
  let bookmarkPrivate = $state(false);
  let saving = $state(false);

  onMount(async () => {
    await auth.init();
    if (auth.isLoggedIn) {
      const res = await listBookmarks();
      if (res.err === 0) {
        isBookmarked = res.bookmarks.some((b) => b.work_id === workId);
      }
    }
  });

  async function toggleBookmark() {
    if (!auth.isLoggedIn || saving) return;
    saving = true;
    try {
      if (isBookmarked) {
        await removeBookmark(workId);
        isBookmarked = false;
      } else {
        await addBookmark(workId, bookmarkNotes || undefined, bookmarkPrivate);
        isBookmarked = true;
        bookmarkNotes = '';
      }
    } catch { /* ignore */ }
    saving = false;
  }
</script>

<div class="bookmark-status">
  {#if auth.isLoggedIn}
    <button
      class="btn bookmark-btn"
      class:bookmarked={isBookmarked}
      onclick={toggleBookmark}
      disabled={saving}
      aria-label={isBookmarked ? 'Cancel Bookmark' : 'Bookmark'}
    >
      {#if saving}
        <span class="spinner"></span>
      {:else if isBookmarked}
        Bookmarked — Cancel Bookmark
      {:else}
        Bookmark
      {/if}
    </button>
    {#if !isBookmarked}
      <input
        type="text"
        class="bookmark-notes"
        placeholder="Add a note…"
        bind:value={bookmarkNotes}
      />
      <label class="private-label">
        <input type="checkbox" bind:checked={bookmarkPrivate} />
        Private
      </label>
    {/if}
  {:else}
    <a class="btn btn-secondary" href="/login">Bookmark</a>
  {/if}
</div>

<style>
  .bookmark-status { display: flex; align-items: center; gap: 0.5rem; }
  .bookmark-notes { width: 120px; padding: 0.3rem 0.5rem; font-size: 0.85rem; }
  .private-label { display: flex; align-items: center; gap: 0.25rem; font-size: 0.82rem; color: var(--color-muted); }
  .private-label input { margin: 0; }
</style>
