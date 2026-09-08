<script lang="ts">
  import { fetchUserProfile, type UserProfile } from '$lib/api/forumUsers';
  import XpProgressBar from './XpProgressBar.svelte';

  let { userId, username }: { userId: number; username: string } = $props();

  let profile = $state<UserProfile | null>(null);
  let loading = $state(false);

  $effect(() => {
    if (userId && !profile && !loading) {
      loading = true;
      fetchUserProfile(userId)
        .then((p) => (profile = p))
        .catch(() => {})
        .finally(() => (loading = false));
    }
  });

  /** Get initials from username. */
  function initials(name: string): string {
    return name.slice(0, 2).toUpperCase();
  }

  /** Rank icon based on level. */
  function rankIcon(level: number): string {
    if (level >= 50) return '👑';
    if (level >= 30) return '⚜️';
    if (level >= 20) return '🛡️';
    if (level >= 10) return '⚔️';
    if (level >= 5) return '🗡️';
    return '🌱';
  }
</script>

<div class="author-card">
  <div class="author-avatar">{initials(username)}</div>
  <div class="author-info">
    <a href="/forum/users/{userId}" class="author-name">{username}</a>
    {#if profile}
      <div class="author-meta">
        <span class="rank-badge" title="Level {profile.level}">
          {rankIcon(profile.level)} Lv.{profile.level}
        </span>
        {#if profile.rank > 1}
          <span class="rank-title">Rank {profile.rank}</span>
        {/if}
      </div>
      <XpProgressBar percent={profile.progress_percent} level={profile.level} />
    {:else if loading}
      <div class="author-loading">Loading...</div>
    {/if}
  </div>
</div>

<style>
  .author-card {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
    padding: 0.5rem;
    border-radius: 6px;
    background: #161b22;
    border: 1px solid #21262d;
  }
  .author-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: linear-gradient(135deg, #1f6feb, #58a6ff);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 700;
    color: white;
    flex-shrink: 0;
  }
  .author-info {
    flex: 1;
    min-width: 0;
  }
  .author-name {
    font-weight: 600;
    color: #e1e4e8;
    text-decoration: none;
    font-size: 0.9rem;
  }
  .author-name:hover {
    color: #58a6ff;
    text-decoration: underline;
  }
  .author-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.15rem;
    font-size: 0.75rem;
  }
  .rank-badge {
    color: #58a6ff;
    font-weight: 500;
  }
  .rank-title {
    color: #8b949e;
  }
  .author-loading {
    color: #8b949e;
    font-size: 0.75rem;
    margin-top: 0.2rem;
  }
</style>
