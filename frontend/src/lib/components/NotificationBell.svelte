<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';

  let unreadCount = $state(0);
  let intervalId: ReturnType<typeof setInterval> | null = null;

  async function fetchCount() {
    if (!auth.isLoggedIn) { unreadCount = 0; return; }
    try {
      const res = await fetch('/api/notifications/unread-count', { credentials: 'include' });
      if (res.ok) {
        const data = await res.json();
        unreadCount = data.count || 0;
      }
    } catch { /* silent */ }
  }

  onMount(() => {
    fetchCount();
    intervalId = setInterval(fetchCount, 60000);
    return () => { if (intervalId) clearInterval(intervalId); };
  });
</script>

<a href="/notifications" class="relative inline-flex items-center gap-1.5" aria-label="Notifications">
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" style="width:16px;height:16px;flex-shrink:0">
    <path stroke-linecap="round" stroke-linejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
  </svg>
  <span class="text-sm">Notifications</span>
  {#if unreadCount > 0}
    <span class="absolute -top-1.5 -right-1.5 inline-flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[10px] font-bold leading-none text-white bg-red-500 rounded-full ring-2 ring-white dark:ring-gray-900">
      {unreadCount > 99 ? '99+' : unreadCount}
    </span>
  {/if}
</a>
