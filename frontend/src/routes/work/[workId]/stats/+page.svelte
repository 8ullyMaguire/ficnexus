<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { getPref } from '$lib/prefs';
  import { getWorkStats, getWork } from '$lib/api/social';
  import type { WorkStats, Work } from '$lib/api/social-types';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props<{ data?: { workId: number } }>();
  const workId = $derived(data?.workId ?? 0);

  let stats = $state<WorkStats | null>(null);
  let work = $state<Work | null>(null);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    await auth.init();
    loading = true;
    try {
      const [sRes, wRes] = await Promise.allSettled([
        getWorkStats(workId),
        getWork(workId),
      ]);
      if (sRes.status === 'fulfilled' && sRes.value.err === 0) stats = sRes.value;
      if (wRes.status === 'fulfilled' && wRes.value.err === 0) work = wRes.value.work;
    } catch {
      error = 'Network error loading stats.';
    } finally {
      loading = false;
    }
  });
</script>

{#if uiMode === 'archive'}
<header class="archive-header">
  <h1 class="archive-title">{work ? work.canonical_title : 'Work Statistics'}</h1>
  {#if work}
    <p class="muted">by {work.canonical_author}</p>
  {/if}
</header>

{#if loading}
  <div class="archive-skeleton-list">
    <div class="skeleton-blurb" aria-hidden="true"></div>
  </div>
{:else if error}
  <div class="archive-error"><p>{error}</p></div>
{:else if stats}
  <dl class="work stats-block">
    <dt>Bookmarks</dt><dd>{stats.total_bookmarks}</dd>
    <dt>Ratings</dt><dd>{stats.total_ratings}</dd>
    <dt>Average Rating</dt><dd>{stats.avg_rating && stats.avg_rating > 0 ? `${stats.avg_rating.toFixed(1)}★` : '—'}</dd>
    <dt>Comments</dt><dd>{stats.total_comments}</dd>
    <dt>Kudos</dt><dd>{stats.kudos_count + stats.guest_count}</dd>
    <dt>Guest Kudos</dt><dd>{stats.guest_count}</dd>
  </dl>
  <p class="meta">
    <a href={`/work/${workId}`} class="muted">← Back to work page</a>
  </p>
{:else}
  <p class="muted">No stats available yet.</p>
{/if}

{:else}
<!-- Modern mode -->
<div class="page">
  <h1>Work Statistics</h1>
  {#if stats}
    <table class="stats-table">
      <tbody>
        <tr><th>Bookmarks</th><td>{stats.total_bookmarks}</td></tr>
        <tr><th>Ratings</th><td>{stats.total_ratings}</td></tr>
        <tr><th>Average</th><td>{stats.avg_rating ? `${stats.avg_rating.toFixed(1)} ★` : '—'}</td></tr>
        <tr><th>Comments</th><td>{stats.total_comments}</td></tr>
        <tr><th>Kudos</th><td>{stats.kudos_count + stats.guest_count}</td></tr>
        <tr><th>Guest Kudos</th><td>{stats.guest_count}</td></tr>
      </tbody>
    </table>
  {:else}
    <p class="muted">Loading...</p>
  {/if}
</div>
{/if}

<style>
  dl.work.stats-block {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.3rem 2rem;
    margin: 1rem 0;
  }
  dl.work.stats-block dt { color: var(--color-muted); font-weight: 600; }
  dl.work.stats-block dd { margin: 0; }
  .stats-table { width: 100%; border-collapse: collapse; }
  .stats-table th, .stats-table td { text-align: left; padding: 0.5rem; border-bottom: 1px solid var(--color-border); }
  .stats-table th { color: var(--color-muted); font-weight: 600; }
</style>
