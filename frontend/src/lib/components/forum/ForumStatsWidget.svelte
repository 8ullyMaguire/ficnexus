<script lang="ts">
  import { fetchForumStats, type ForumStats } from '$lib/api/forumWidgets';

  let stats = $state<ForumStats | null>(null);
  let loading = $state(true);

  $effect(() => {
    fetchForumStats()
      .then((data) => (stats = data))
      .catch(() => {})
      .finally(() => (loading = false));
  });

  function formatNumber(n: number): string {
    if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M';
    if (n >= 1000) return (n / 1000).toFixed(1) + 'K';
    return n.toString();
  }
</script>

<div class="widget">
  <h3 class="widget-title">Forum Stats</h3>
  {#if loading}
    <p class="widget-loading">Loading...</p>
  {:else if stats}
    <div class="stats-grid">
      <div class="stat">
        <span class="stat-value">{formatNumber(stats.total_topics)}</span>
        <span class="stat-label">Topics</span>
      </div>
      <div class="stat">
        <span class="stat-value">{formatNumber(stats.total_posts)}</span>
        <span class="stat-label">Posts</span>
      </div>
      <div class="stat">
        <span class="stat-value">{formatNumber(stats.total_users)}</span>
        <span class="stat-label">Users</span>
      </div>
    </div>
    {#if stats.newest_user}
      <p class="newest-user">
        Newest: <a href="/forum/users">{stats.newest_user.username}</a>
      </p>
    {/if}
  {/if}
</div>

<style>
  .widget { padding: 0.75rem; }
  .widget-title { font-size: 0.9rem; font-weight: 600; margin-bottom: 0.5rem; color: #e1e4e8; }
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
    text-align: center;
  }
  .stat { display: flex; flex-direction: column; }
  .stat-value { font-size: 1.1rem; font-weight: 700; color: #58a6ff; }
  .stat-label { font-size: 0.7rem; color: #8b949e; text-transform: uppercase; }
  .newest-user {
    font-size: 0.75rem;
    color: #8b949e;
    margin-top: 0.5rem;
    text-align: center;
  }
  .newest-user a { color: #58a6ff; text-decoration: none; }
  .newest-user a:hover { text-decoration: underline; }
  .widget-loading { color: #8b949e; font-size: 0.85rem; }
</style>
