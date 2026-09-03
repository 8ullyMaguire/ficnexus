<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';

  let checking = $state(true);

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn || auth.level < 100) {
      goto('/');
    }
    checking = false;
  });

  let { children } = $props();
</script>

{#if checking}
  <div class="loading"><p>Checking access...</p></div>
{:else}
  <div class="admin-layout">
    <aside class="admin-sidebar">
      <h2>Admin</h2>
      <nav>
        <a href="/admin" data-sveltekit-preload>Dashboard</a>
        <a href="/admin/auto-tag" data-sveltekit-preload>Auto-Tag</a>
        <a href="/admin/bulk-actions" data-sveltekit-preload>Bulk Actions</a>
        <a href="/admin/bots" data-sveltekit-preload>Bot Behavior</a>
        <a href="/admin/moderation" data-sveltekit-preload>Moderation</a>
        <a href="/admin/comment-triage" data-sveltekit-preload>Comment Triage</a>
        <a href="/admin/content-scan" data-sveltekit-preload>Content Scan</a>
        <a href="/admin/blacklist" data-sveltekit-preload>Blacklist</a>
        <a href="/admin/features" data-sveltekit-preload>Feature Flags</a>
        <a href="/admin/stats" data-sveltekit-preload>Stats</a>
        <a href="/admin/analytics" data-sveltekit-preload>Usage Analytics</a>
        <a href="/admin/scrapers" data-sveltekit-preload>Scraper Health</a>
        <a href="/admin/users" data-sveltekit-preload>Users</a>
      </nav>
    </aside>
    <main class="admin-content">
      {@render children()}
    </main>
  </div>
{/if}

<style>
  .loading { text-align: center; padding: 3rem; }
  .admin-layout { display: flex; gap: 1rem; min-height: 80vh; }
  .admin-sidebar { width: 220px; flex-shrink: 0; padding: 1rem; border-right: 1px solid var(--color-border); }
  .admin-sidebar h2 { font-size: 1.2rem; margin-bottom: 1rem; }
  .admin-sidebar nav { display: flex; flex-direction: column; gap: 0.5rem; }
  .admin-sidebar a { padding: 0.4rem 0.8rem; border-radius: var(--radius-sm); text-decoration: none; color: var(--color-text); }
  .admin-sidebar a:hover { background: var(--color-surface-2); }
  .admin-content { flex: 1; padding: 1rem; }
</style>
