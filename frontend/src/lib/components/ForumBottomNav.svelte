<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { getUnreadCount } from '$lib/api/social';
  import { auth } from '$lib/stores/auth.svelte';

  let unread = $state(0);
  let pathname = $state('');

  $effect(() => {
    pathname = $page.url.pathname;
  });

  const isForum = $derived(pathname.startsWith('/forum'));
  const active = (href: string) =>
    pathname === href || (href !== '/forum' && pathname.startsWith(href)) ? 'page' : undefined;

  onMount(() => {
    let alive = true;
    async function poll() {
      if (!auth.isLoggedIn) return;
      try {
        const r = await getUnreadCount();
        if (alive && (r as any)?.err === 0) unread = (r as any).unread_count ?? 0;
      } catch {}
    }
    auth.init().then(poll);
    const id = setInterval(poll, 60_000);
    return () => { alive = false; clearInterval(id); };
  });
</script>

{#if isForum}
<nav class="forum-bottom-nav" aria-label="Forum navigation">
  <a href="/forum" aria-current={active('/forum')} class="fbn-item">
    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 9L12 3l9 6v11a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"/></svg>
    <span>Home</span>
  </a>
  <a href="/forum" aria-current={pathname === '/forum' ? 'page' : undefined} class="fbn-item">
    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
    <span>Categories</span>
  </a>
  <a href="/forum/search" aria-current={active('/forum/search')} class="fbn-item">
    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/></svg>
    <span>Search</span>
  </a>
  <a href="/notifications" aria-current={active('/notifications')} class="fbn-item fbn-notif">
    <span class="fbn-icon-wrap">
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M6 9a6 6 0 0 1 12 0c0 7-6 5-6 9"/><path d="M9 21a3 3 0 0 0 6 0"/></svg>
      {#if unread > 0}<span class="fbn-badge">{unread > 99 ? '99+' : unread}</span>{/if}
    </span>
    <span>Alerts</span>
  </a>
  <a href="/dashboard" aria-current={active('/dashboard')} class="fbn-item">
    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="8" r="4"/><path d="M5 20a7 7 0 0 1 14 0"/></svg>
    <span>Me</span>
  </a>
</nav>
{/if}

<style>
  .forum-bottom-nav {
    display: none;
  }
  @media (max-width: 768px) {
    .forum-bottom-nav {
      display: grid;
      grid-template-columns: repeat(5, 1fr);
      position: fixed;
      left: 0; right: 0; bottom: 0;
      z-index: 40;
      background: var(--color-surface, #fff);
      border-top: 1px solid var(--color-border, #ddd);
      min-height: 48px;
      padding: 4px 0 calc(4px + env(safe-area-inset-bottom, 0px));
      box-shadow: 0 -1px 6px rgba(0,0,0,0.08);
    }
  }
  .fbn-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    text-decoration: none;
    color: var(--color-text-muted, #666);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.02em;
    padding: 4px 2px;
  }
  .fbn-item[aria-current='page'] {
    color: var(--archive-link, #990000);
  }
  .fbn-icon-wrap { position: relative; display: inline-flex; }
  .fbn-badge {
    position: absolute;
    top: -6px; right: -8px;
    background: #c00;
    color: #fff;
    font-size: 10px;
    font-weight: 700;
    min-width: 16px;
    height: 16px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
    line-height: 1;
  }
</style>
