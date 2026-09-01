<script lang="ts">
  import { onMount } from 'svelte';
  import { listNotifications, markNotificationRead, markAllNotificationsRead, getUnreadCount } from '$lib/api/social';
  import type { Notification } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { goto } from '$app/navigation';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let notifs = $state<Notification[]>([]);
  let unreadCount = $state(0);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadNotifs();
  });

  async function loadNotifs() {
    loading = true; error = '';
    try {
      const res = await listNotifications(50, 0);
      if (res.err === 0) {
        notifs = res.notifications;
        unreadCount = res.unread_count;
      } else { error = 'Failed to load notifications.'; }
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  async function handleMarkRead(id: number) {
    await markNotificationRead(id);
    notifs = notifs.map(n => n.id === id ? { ...n, is_read: true } : n);
    unreadCount = Math.max(0, unreadCount - 1);
  }

  async function handleMarkAllRead() {
    await markAllNotificationsRead();
    notifs = notifs.map(n => ({ ...n, is_read: true }));
    unreadCount = 0;
  }

  function timeAgo(dateStr: string): string {
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return t('notif.justNow');
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    if (days < 30) return `${days}d ago`;
    return new Date(dateStr).toLocaleDateString();
  }
</script>

{#if uiMode === 'archive'}
  <div class="archive-notif-page">
    <div class="archive-notif-header">
      <h2>Notifications</h2>
      {#if unreadCount > 0}
        <ArchiveButton onclick={handleMarkAllRead}>
          Mark All Read
        </ArchiveButton>
      {/if}
    </div>

    {#if loading}
      <p class="archive-muted">Loading...</p>
    {:else if error}
      <p class="archive-error">{error}</p>
    {:else if notifs.length === 0}
      <p class="archive-muted">No notifications</p>
    {:else}
      <div class="archive-notif-list">
        {#each notifs as n (n.id)}
          <div class="archive-notif-row" class:unread={!n.is_read}>
            <span class="archive-notif-type">{n.title}</span>
            <span class="archive-notif-time">{timeAgo(n.created_at)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <div class="notif-page">
    <div class="notif-header">
      <h1>{t('notif.title')}</h1>
      {#if unreadCount > 0}
        <button class="btn-sm" onclick={handleMarkAllRead}>
          {t('notif.markAllRead', { count: unreadCount })}
        </button>
      {/if}
    </div>

    {#if loading}
      <div class="card"><p class="muted"><span class="spinner"></span> {t('notif.loading')}</p></div>
    {:else if error}
      <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
    {:else if notifs.length === 0}
      <div class="card empty"><p class="muted">{t('notif.empty')}</p></div>
    {:else}
      <div class="notif-list">
        {#each notifs as n (n.id)}
          <div class="card notif" class:unread={!n.is_read}>
            <div class="notif-main">
              <span class="notif-title">{n.title}</span>
              {#if n.body}<p class="notif-body">{n.body}</p>{/if}
              <span class="notif-time">{timeAgo(n.created_at)}</span>
            </div>
            <div class="notif-actions">
              {#if n.link}
                <a href={n.link} class="btn-xs">{t('notif.view')}</a>
              {/if}
              {#if !n.is_read}
                <button class="btn-xs" onclick={() => handleMarkRead(n.id)}>{t('notif.markRead')}</button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  /* ── Archive mode ──────────────────────────────────────────────────── */
  .archive-notif-page {
    max-width: var(--max-width, 700px);
    margin: 0 auto;
    padding: 1rem;
    font-family: 'Georgia', 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-notif-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 2px solid var(--archive-border, #dddddd);
    padding-bottom: 0.5rem;
    margin-bottom: 0.75rem;
  }
  .archive-notif-header h2 {
    margin: 0;
    font-family: 'Georgia', 'Times New Roman', serif;
    color: var(--archive-heading, #990000);
    font-size: 1.3rem;
  }
  .archive-notif-list {
    display: flex;
    flex-direction: column;
  }
  .archive-notif-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.45rem 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    color: var(--archive-text, #2a2a2a);
    font-size: 0.9rem;
  }
  .archive-notif-row:last-child {
    border-bottom: none;
  }
  .archive-notif-row.unread {
    font-weight: 600;
  }
  .archive-notif-type {
    flex: 1;
  }
  .archive-notif-time {
    font-size: 0.82rem;
    color: var(--archive-muted, #666666);
    margin-left: 1rem;
    white-space: nowrap;
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-error {
    color: var(--archive-link, #990000);
  }

  /* ── Modern mode (existing) ───────────────────────────────────────── */
  .notif-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .notif-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  .notif-header h1 { margin: 0; }
  .notif-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .notif { display: flex; justify-content: space-between; align-items: flex-start; gap: 0.8rem; padding: 0.8rem 1rem; }
  .notif.unread { border-left: 3px solid var(--color-primary); }
  .notif-title { font-weight: 600; }
  .notif-body { margin: 0.3rem 0; font-size: 0.9rem; color: var(--color-muted); }
  .notif-time { font-size: 0.8rem; color: var(--color-muted); }
  .notif-actions { display: flex; gap: 0.3rem; flex-shrink: 0; }
  .empty { text-align: center; padding: 2rem; }
  .error-card { border-color: var(--color-error); }
  .btn-sm { padding: 0.4rem 0.8rem; font-size: 0.85rem; border-radius: var(--radius-sm); background: var(--color-surface-2); border: 1px solid var(--color-border); cursor: pointer; }
  .btn-sm:hover { background: var(--color-surface); }
  .btn-xs { padding: 0.25rem 0.5rem; font-size: 0.8rem; border-radius: var(--radius-sm); background: var(--color-surface-2); border: 1px solid var(--color-border); cursor: pointer; text-decoration: none; color: var(--color-text); }
  .btn-xs:hover { background: var(--color-surface); }
</style>
