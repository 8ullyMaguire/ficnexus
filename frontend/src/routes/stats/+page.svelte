<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { getPref } from '$lib/prefs';
  import { t } from '$lib/i18n/index.svelte';
  import { getReadingStats, getStreak } from '$lib/api/social';
  import { relativeTime } from '$lib/util';

  interface RecentItem {
    work_id: number;
    words_read: number;
    read_count: number;
    last_read_at: string;
    title?: string;
  }

  interface SitePeriodStat {
    period: string;
    work_views: number;
    kudos: number;
    bookmarks: number;
    works_read: number;
    reviews: number;
    total_works: number;
    total_users: number;
    // Back-compat aliases if backend ever returns legacy names
    kudos_given?: number;
    bookmarks_made?: number;
    works_started?: number;
    reviews_written?: number;
  }

  interface SiteStatsResponse {
    err: number;
    periods: SitePeriodStat[];
  }

  let stats = $state<{
    total_words_read: number;
    total_works_read: number;
    login_streak: number;
    recent: RecentItem[];
  } | null>(null);
  let streak = $state<{
    current_streak: number;
    longest_streak: number;
    last_login_date: string;
  } | null>(null);
  let readingList = $state<any[]>([]);
  let siteStats = $state<SiteStatsResponse | null>(null);
  let personalAnalytics = $state<any>(null);
  let loading = $state(true);
  let error = $state('');

  const uiMode = $derived(getPref('uiMode'));

  function fmtPeriod(label: string): string {
    const map: Record<string, string> = {
      '1d': 'Past day',
      '7d': 'Past 7 days',
      '14d': 'Past 14 days',
      '30d': 'Past 30 days',
      '90d': 'Past 90 days',
      '365d': 'Past year',
      'all': 'All time',
    };
    if (map[label]) return map[label];
    // Fallback to i18n if key exists, otherwise raw label
    const k = `stats.period${label}`;
    const translated = t(k);
    return translated !== k ? translated : label;
  }

  function periodVal(p: SitePeriodStat, key: 'kudos' | 'bookmarks' | 'reviews'): number {
    if (key === 'kudos') return p.kudos ?? p.kudos_given ?? 0;
    if (key === 'bookmarks') return p.bookmarks ?? p.bookmarks_made ?? 0;
    return p.reviews ?? p.reviews_written ?? 0;
  }

  onMount(async () => {
    await auth.init();
    const promises: Promise<any>[] = [];

    if (auth.isLoggedIn && auth.user) {
      promises.push(
        Promise.all([
          getReadingStats(auth.user.id),
          fetch('/api/reading/analytics', { credentials: 'include' }).then(r => r.ok ? r.json() : null),
          getStreak(auth.user.id),
          fetch('/api/reading/list?status=completed&per_page=10', { credentials: 'include' }).then(r => r.ok ? r.json() : { reading_list: [] }),
        ]).then(([statsRes, analyticsRes, streakRes, readingRes]) => {
          personalAnalytics = analyticsRes;
          if (statsRes && (statsRes as any).err === 0) stats = statsRes as any;
          else if (statsRes && !(statsRes as any).err) stats = statsRes as any;
          if (streakRes && (streakRes as any).err === 0) streak = streakRes as any;
          readingList = (readingRes as any).reading_list || (readingRes as any).items || [];
        }).catch(() => {})
      );
    }

    promises.push(
      fetch('/api/site/stats', { credentials: 'include' })
        .then(r => r.ok ? r.json() : null)
        .then(data => { if (data && data.err === 0 && Array.isArray(data.periods)) siteStats = data as SiteStatsResponse; })
        .catch(() => null)
    );

    try {
      await Promise.all(promises);
    } catch {
      error = 'Failed to load statistics.';
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head><title>Statistics — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Statistics</h1>
      <p class="archive-summary">Site-wide activity and your personal reading totals.</p>
    </header>

    {#if loading}
      <div class="archive-skeleton-list">
        {#each Array(3) as _, i}
          <article class="skeleton-blurb" aria-hidden="true">
            <div class="skel-header"><span class="skel-badge"></span><span class="skel-title"></span></div>
            <span class="skel-byline"></span>
            <div class="skel-stats"></div>
          </article>
        {/each}
      </div>
    {:else if error}
      <div class="archive-error"><p>{error}</p></div>
    {:else}
      {#if siteStats && siteStats.periods.length > 0}
        <section class="archive-section">
          <h2 class="archive-subhead">Site-wide statistics</h2>
          <p class="archive-muted" style="margin:0 0 0.6em;">Aggregated views, kudos, bookmarks, and reviews.</p>
          <div class="archive-table-wrap">
            <table class="archive-table">
              <thead>
                <tr>
                  <th>Period</th>
                  <th>Views</th>
                  <th>Kudos</th>
                  <th>Bookmarks</th>
                  <th>Works read</th>
                  <th>Reviews</th>
                </tr>
              </thead>
              <tbody>
                {#each siteStats.periods as p (p.period)}
                  <tr>
                    <td class="archive-mono">{fmtPeriod(p.period)}</td>
                    <td>{(p.work_views ?? 0).toLocaleString()}</td>
                    <td>{periodVal(p, 'kudos').toLocaleString()}</td>
                    <td>{periodVal(p, 'bookmarks').toLocaleString()}</td>
                    <td>{(p.works_read ?? 0).toLocaleString()}</td>
                    <td>{periodVal(p, 'reviews').toLocaleString()}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
          {#if siteStats.periods[0]}
            {@const all = siteStats.periods.find(p => p.period === 'all') ?? siteStats.periods[siteStats.periods.length - 1]}
            {#if all}
              <blockquote class="archive-summary" style="margin-top:1em;">
                <dl class="stats">
                  <div class="stat"><dt>Total works</dt><dd>{(all.total_works ?? 0).toLocaleString()}</dd></div>
                  <div class="stat"><dt>Active users</dt><dd>{(all.total_users ?? 0).toLocaleString()}</dd></div>
                </dl>
              </blockquote>
            {/if}
          {/if}
        </section>
      {:else}
        <blockquote class="archive-summary"><p>Site statistics are not yet available.</p></blockquote>
      {/if}

      {#if auth.isLoggedIn}
        {#if stats}
          <section class="archive-section">
            <h2 class="archive-subhead">Your reading</h2>
            <blockquote class="archive-summary">
              <dl class="stats">
                <div class="stat"><dt>Words read</dt><dd>{(stats.total_words_read ?? 0).toLocaleString()}</dd></div>
                <div class="stat"><dt>Works read</dt><dd>{stats.total_works_read ?? 0}</dd></div>
                {#if streak}
                  <div class="stat"><dt>Current streak</dt><dd>{streak.current_streak ?? 0} day{(streak.current_streak ?? 0) === 1 ? '' : 's'}</dd></div>
                  <div class="stat"><dt>Longest streak</dt><dd>{streak.longest_streak ?? 0} day{(streak.longest_streak ?? 0) === 1 ? '' : 's'}</dd></div>
                {:else if stats.login_streak !== undefined}
                  <div class="stat"><dt>Login streak</dt><dd>{stats.login_streak ?? 0} day{(stats.login_streak ?? 0) === 1 ? '' : 's'}</dd></div>
                {/if}
              </dl>
            </blockquote>
          </section>
        {:else}
          <section class="archive-section">
            <h2 class="archive-subhead">Your reading</h2>
            <p class="archive-muted">No personal totals yet — start reading and they will appear here.</p>
          </section>
        {/if}

        {#if readingList.length > 0}
          <section class="archive-section">
            <h2 class="archive-subhead">Recently read</h2>
            <table class="archive-table">
              <thead>
                <tr>
                  <th>Work</th>
                  <th>Words</th>
                  <th>Reads</th>
                  <th>Last read</th>
                </tr>
              </thead>
              <tbody>
                {#each readingList as item (item.work_id)}
                  <tr>
                    <td><a class="archive-link" href={`/works/${item.work_id}`}>{item.title || `Work #${item.work_id}`}</a></td>
                    <td>{(item.words_read ?? item.words ?? 0).toLocaleString()}</td>
                    <td>{item.read_count ?? 1}</td>
                    <td class="archive-mono">{relativeTime(item.last_read_at || item.updated_at)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </section>
        {/if}
      {:else}
        <blockquote class="archive-summary">
          <p><a class="archive-link" href="/login">Log in</a> to see your personal reading totals and streak.</p>
        </blockquote>
      {/if}
    {/if}
  </div>
</main>
{:else}
<div class="stats-page">
  <h1>Reading Statistics</h1>

  {#if loading}
    <p class="loading">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>{error}</strong></div>
  {:else}
    {#if siteStats && siteStats.periods.length > 0}
      <section class="stats-card">
        <h2>Site-wide</h2>
        <table class="recent-table">
          <thead>
            <tr><th>Period</th><th>Views</th><th>Kudos</th><th>Bookmarks</th><th>Works read</th><th>Reviews</th></tr>
          </thead>
          <tbody>
            {#each siteStats.periods as p (p.period)}
              <tr>
                <td>{fmtPeriod(p.period)}</td>
                <td>{(p.work_views ?? 0).toLocaleString()}</td>
                <td>{periodVal(p, 'kudos').toLocaleString()}</td>
                <td>{periodVal(p, 'bookmarks').toLocaleString()}</td>
                <td>{(p.works_read ?? 0).toLocaleString()}</td>
                <td>{periodVal(p, 'reviews').toLocaleString()}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </section>
    {/if}

    {#if stats}
      <div class="stats-grid">
        <div class="stat-card">
          <span class="stat-value">{(stats.total_words_read ?? 0).toLocaleString()}</span>
          <span class="stat-label">Words read</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{stats.total_works_read ?? 0}</span>
          <span class="stat-label">Works read</span>
        </div>
        {#if streak}
          <div class="stat-card">
            <span class="stat-value">{streak.current_streak ?? 0}</span>
            <span class="stat-label">Login streak</span>
          </div>
        {/if}
      </div>
    {/if}

    {#if readingList.length > 0}
      <div class="recent-section">
        <h2>Recently read</h2>
        <table class="recent-table">
          <thead>
            <tr><th>Work</th><th>Words</th><th>Reads</th><th>Last read</th></tr>
          </thead>
          <tbody>
            {#each readingList as item (item.work_id)}
              <tr>
                <td><a href={`/works/${item.work_id}`}>{item.title || `Work #${item.work_id}`}</a></td>
                <td>{(item.words_read ?? 0).toLocaleString()}</td>
                <td>{item.read_count}</td>
                <td>{relativeTime(item.last_read_at)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    {#if !auth.isLoggedIn}
      <p class="loading"><a href="/login">Log in</a> to see personal totals.</p>
    {/if}
  {/if}
</div>
{/if}

<style>
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
  .archive-section {
    margin-bottom: 1.5em;
    padding-bottom: 1em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-section:last-child { border-bottom: none; }
  .archive-subhead {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.15em;
    font-weight: 700;
    color: var(--archive-link, #990000);
    margin: 1em 0 0.5em;
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
    font-size: 0.9em;
  }
  .archive-mono {
    font-family: ui-monospace, monospace;
    font-size: 0.88em;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .archive-link:hover { text-decoration: underline; }
  .archive-table-wrap { overflow-x: auto; }
  .archive-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.92em;
  }
  .archive-table th,
  .archive-table td {
    text-align: left;
    padding: 0.4rem 0.6rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-table th {
    color: var(--archive-muted, #666666);
    font-weight: 700;
    font-size: 0.82em;
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }
  dl.stats {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    column-gap: 1em;
    row-gap: 0.4em;
    line-height: 1.5;
    margin: 0;
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
  .archive-error {
    padding: 1.2em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
  }
  /* Skeleton */
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
  .skel-badge { width: 2rem; height: 0.9rem; }
  .skel-title { width: 40%; height: 1.2rem; }
  .skel-byline { width: 20%; height: 0.8rem; }
  .skel-stats { width: 15%; height: 0.8rem; }
  .skel-header { display: flex; gap: 0.3rem; align-items: center; }

  /* Modern mode (isolated, no :global bleed) */
  .stats-page { max-width: 1100px; margin: 0 auto; padding: 1rem; }
  .stats-card { margin: 1.2rem 0; }
  .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin: 1.5rem 0; }
  .stat-card { background: #f6f6f6; border: 1px solid #ddd; border-radius: 8px; padding: 1rem; text-align: center; }
  .stat-value { display: block; font-size: 1.8rem; font-weight: 700; color: #990000; }
  .stat-label { font-size: 0.8rem; color: #888; text-transform: uppercase; }
  .recent-section { margin-top: 2rem; }
  .recent-table { width: 100%; border-collapse: collapse; }
  .recent-table th, .recent-table td { text-align: left; padding: 0.4rem 0.7rem; border-bottom: 1px solid #eee; }
  .recent-table th { font-weight: 600; color: #888; font-size: 0.8rem; text-transform: uppercase; }
  .loading { padding: 3rem; text-align: center; color: #888; }
  .error-card { padding: 1rem; border: 1px solid #d33; border-radius: 8px; background: #fee; }
</style>
