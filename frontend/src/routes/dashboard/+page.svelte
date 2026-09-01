<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { getReadingStats, getUnreadCount, getStreak, listBookmarks, listReadingLists, getFeed } from '$lib/api/social';
  import type { Bookmark, ReadingList, FeedItem } from '$lib/api/social-types';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import { formatWords, relativeTime } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  let loading = $state(true);
  let error = $state('');

  let readingStats = $state<{ total_words_read: number; total_works_read: number; login_streak: number } | null>(null);
  let recentBookmarks: Bookmark[] = $state([]);
  let unreadCount = $state(0);
  let streak = $state<{ current_streak: number; longest_streak: number } | null>(null);
  let recentActivity: FeedItem[] = $state([]);
  let myLists: ReadingList[] = $state([]);
  
  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    await loadOverview();
  });

  async function loadOverview() {
    loading = true;
    error = '';
    const uid = auth.user?.id ?? 0;
    if (!uid) { loading = false; return; }
    try {
      await Promise.allSettled([
        (async () => { const r = await getReadingStats(uid); if ((r as any)?.err === 0) readingStats = r as any; else if (r && !('err' in r)) readingStats = r as any; })(),
        (async () => { const r = await getUnreadCount(); if ((r as any)?.err === 0) unreadCount = (r as any).unread_count; })(),
        (async () => { const r = await getStreak(uid); if ((r as any)?.err === 0) streak = r as any; })(),
        (async () => { const r = await listBookmarks(); if ((r as any)?.err === 0) recentBookmarks = (r as any).bookmarks.slice(0, 4); })(),
        (async () => { const r = await listReadingLists(); if ((r as any)?.err === 0) myLists = (r as any).lists; })(),
        (async () => { const r = await getFeed(1, 10); if ((r as any)?.err === 0) recentActivity = (r as any).items; })(),
      ]);
    } catch {
      error = 'Network error loading dashboard.';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Dashboard — FicNexus</title></svelte:head>

{#if !auth.isLoggedIn}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Dashboard</h1>
      </header>
      <div class="archive-empty">
        <p>Log in to see your dashboard.</p>
        <a class="archive-link" href="/login">Log In</a>
      </div>
    </div>
  </main>
{:else if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Dashboard</h1>
      <p class="archive-summary">Your reading stats, bookmarks, collections, and activity — AO3-style overview.</p>
    </header>

    {#if loading}
      <div class="archive-skeleton-list" aria-hidden="true">
        {#each Array(4) as _, i}
          <article class="skeleton-blurb">
            <div class="skel-header"><span class="skel-badge"></span><span class="skel-title"></span></div>
            <span class="skel-byline"></span>
            <div class="skel-stats"></div>
          </article>
        {/each}
      </div>
    {:else if error}
      <div class="archive-error"><p>{error}</p></div>
    {:else}
      <!-- Reading stats (AO3-style dl.stats) -->
      {#if readingStats}
        <blockquote class="archive-summary">
          <dl class="stats">
            <div class="stat"><dt>Works</dt><dd>{readingStats.total_works_read}</dd></div>
            <div class="stat"><dt>Words</dt><dd>{formatWords(readingStats.total_words_read)}</dd></div>
            <div class="stat"><dt>Login streak</dt><dd>{readingStats.login_streak} day{readingStats.login_streak !== 1 ? 's' : ''}</dd></div>
          </dl>
        </blockquote>
      {/if}

      <!-- Streak -->
      {#if streak}
        <section class="archive-section">
          <h2 class="archive-subhead">Reading Streak</h2>
          <p class="archive-meta">
            Current: {streak.current_streak} day{streak.current_streak !== 1 ? 's' : ''}
            {' '}·{' '}
            Longest: {streak.longest_streak} day{streak.longest_streak !== 1 ? 's' : ''}
          </p>
        </section>
      {/if}

      <!-- Recent Bookmarks -->
      <section class="archive-section">
        <h2 class="archive-subhead">
          Recent Bookmarks
          <ArchiveButton href="/bookmarks">All Bookmarks</ArchiveButton>
        </h2>
        {#if recentBookmarks.length === 0}
          <p class="archive-meta">No bookmarks yet.</p>
        {:else}
          <div class="archive-work-list">
            {#each recentBookmarks as b (b.work_id)}
              <WorkBlurb fic={{
                url_id: String(b.work_id),
                title: '',
                author: '',
                words: 0,
                chapters: 0,
                source: 'bookmark',
                status: '',
                description: b.notes || '',
                updated: b.created_at,
                tags: []
              }} />
            {/each}
          </div>
        {/if}
      </section>

      <!-- Quick links -->
      <section class="archive-section">
        <h2 class="archive-subhead">Your Library</h2>
        <ul class="archive-list">
          <li><a class="archive-link" href="/bookmarks">Bookmarks</a></li>
          <li><a class="archive-link" href="/collections">Collections</a></li>
          <li><a class="archive-link" href="/history">Reading History</a></li>
          <li><a class="archive-link" href="/stats">Statistics</a></li>
        </ul>
      </section>

      <!-- Collections preview -->
      <section class="archive-section">
        <h2 class="archive-subhead">
          Your Collections
          <ArchiveButton href="/collections">All Collections</ArchiveButton>
        </h2>
        {#if myLists.length === 0}
          <p class="archive-meta">You haven't created any collections yet.</p>
          <ArchiveButton href="/collections/new">Create one</ArchiveButton>
        {:else}
          <ul class="archive-list">
            {#each myLists as list (list.id)}
              <li>
                <a class="archive-link" href={`/collections/${list.id}`}>{list.title}</a>
                <span class="archive-meta"> — {list.item_count} item{list.item_count === 1 ? '' : 's'}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <!-- Activity feed -->
      <section class="archive-section">
        <h2 class="archive-subhead">Recent Activity</h2>
        {#if recentActivity.length === 0}
          <p class="archive-meta">No recent activity. Follow works and users to see updates here.</p>
        {:else}
          <ul class="archive-list">
            {#each recentActivity as item (item.work_id)}
              <li class="archive-blurb">
                <span class="archive-blurb-title"><a class="archive-link" href={`/works/${item.work_id}`}>{item.title}</a></span>
                <span class="archive-meta"> by {item.author} · {relativeTime(item.updated_at)}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <!-- Quests -->
      <section class="archive-section">
        <h2 class="archive-subhead">Current Quests</h2>
      </section>
    {/if}
  </div>
</main>

{:else}
  <!-- Modern mode -->
  <div class="dashboard-page">
    <h1>Dashboard</h1>
    {#if loading}
      <p class="muted">Loading…</p>
    {:else if error}
      <div class="error-card"><p>{error}</p></div>
    {:else}
      <p class="muted">Switch to archive mode for the full AO3-style dashboard.</p>
      {#if readingStats}
        <div class="stats-grid">
          <div class="stat-card"><span class="stat-value">{readingStats.total_works_read}</span><span class="stat-label">Works</span></div>
          <div class="stat-card"><span class="stat-value">{formatWords(readingStats.total_words_read)}</span><span class="stat-label">Words</span></div>
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-content {
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8em;
    font-weight: normal;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-summary {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 1em;
    margin: 0.5em 0 1em;
    font-style: italic;
    color: var(--archive-muted, #666666);
  }
  .archive-subhead {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.15em;
    font-weight: 700;
    color: var(--archive-link, #990000);
    margin: 1em 0 0.5em;
  }
  .archive-section {
    margin-bottom: 1.5em;
    padding-bottom: 1em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-section:last-child { border-bottom: none; }
  .archive-meta {
    color: var(--archive-muted, #666666);
    font-style: italic;
    font-size: 0.92em;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .archive-link:hover { text-decoration: underline; }
  .archive-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .archive-list li {
    padding: 0.3em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .archive-list li:last-child { border-bottom: none; }
  .archive-blurb {
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    padding: 0.7em 1em;
    margin-bottom: 0.6em;
  }
  .archive-blurb-title {
    font-weight: 700;
    color: var(--archive-link, #990000);
  }
  .archive-blurb-title a { color: inherit; }
  dl.stats {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    column-gap: 1em;
    row-gap: 0.3em;
    line-height: 1.5;
    margin: 0.4em 0 0;
  }
  dl.stats dt {
    display: inline-flex;
    align-items: baseline;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin: 0;
  }
  dl.stats dd {
    display: inline-flex;
    align-items: baseline;
    font-weight: normal;
    color: var(--archive-text, #2a2a2a);
    margin: 0 0 0 0.35em;
  }
  .archive-work-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .archive-error {
    padding: 1.5em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
  }
  .archive-empty {
    padding: 2em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-empty p {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    margin: 0 0 0.5em;
  }
  .archive-quest-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6em;
  }
  /* Skeleton (AO3-style) */
  .archive-skeleton-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .archive-skeleton-list article.skeleton-blurb {
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding: 0.8rem 0;
  }
  .skel-badge, .skel-title, .skel-byline, .skel-stats {
    display: block;
    background: var(--archive-border, #dddddd);
    border-radius: 2px;
    margin-bottom: 0.3rem;
  }
  .skel-header { display: flex; gap: 0.3rem; align-items: center; }
  .skel-badge { width: 2rem; height: 0.9rem; }
  .skel-title { width: 40%; height: 1.2rem; flex: 1; }
  .skel-byline { width: 20%; height: 0.8rem; }
  .skel-stats { width: 15%; height: 0.8rem; }

  /* ── Modern mode ──────────────────────────────────────────────── */
  .dashboard-page {
    max-width: var(--max-width, 1100px);
    margin: 0 auto;
    padding: 1rem;
  }
  .muted { color: var(--color-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid #d33; border-radius: 8px; background: #fee; }
  .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin: 1rem 0; }
  .stat-card { background: #f6f6f6; border: 1px solid #ddd; border-radius: 8px; padding: 1rem; text-align: center; }
  .stat-value { display: block; font-size: 1.4rem; font-weight: 700; color: #990000; }
  .stat-label { font-size: 0.8rem; color: #888; text-transform: uppercase; }
</style>
