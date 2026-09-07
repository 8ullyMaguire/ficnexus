<script lang="ts">
  // Site-wide DMs (Lane 3): room list + thread + composer.
  // Supports ?to={userId} to find-or-create a DM room on mount and
  // ?room={roomId} to open a room directly.
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import {
    listRooms,
    createDMRoom,
    getRoomMessages,
    sendRoomMessage,
    updateRoomPrefs,
    type DMRoom,
    type DMMessage,
  } from '$lib/api/messages';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let rooms = $state<DMRoom[]>([]);
  let activeRoomId = $state<number | null>(null);
  let messages = $state<DMMessage[]>([]);
  let hasMore = $state(false);
  let loadingRooms = $state(true);
  let loadingMessages = $state(false);
  let roomsError = $state('');
  let threadError = $state('');
  let composer = $state('');
  let sending = $state(false);
  let sendError = $state('');
  let muting = $state(false);
  let muted = $state(false);
  let flash = $state('');

  async function loadRooms(selectId: number | null = null) {
    loadingRooms = true;
    roomsError = '';
    try {
      const res = await listRooms();
      if (res.err === 0) {
        rooms = res.rooms ?? [];
        if (selectId != null) {
          activeRoomId = selectId;
        } else if (activeRoomId == null && rooms.length > 0) {
          activeRoomId = rooms[0].id;
        }
        if (activeRoomId != null) await loadThread(activeRoomId);
      } else {
        roomsError = res.msg ?? t('messages.loadError');
      }
    } catch {
      roomsError = t('messages.loadError');
    } finally {
      loadingRooms = false;
    }
  }

  async function loadThread(roomId: number, cursor?: number) {
    if (cursor == null) {
      loadingMessages = true;
      threadError = '';
    }
    try {
      const res = await getRoomMessages(roomId, cursor != null ? { cursor, limit: 30 } : { limit: 30 });
      if (res.err === 0) {
        const fetched = res.messages ?? [];
        messages = cursor != null ? [...messages, ...fetched] : [...fetched].reverse();
        hasMore = res.has_more ?? false;
        muted = false;
      } else {
        threadError = res.msg ?? t('messages.threadError');
      }
    } catch {
      threadError = t('messages.threadError');
    } finally {
      loadingMessages = false;
    }
  }

  async function selectRoom(id: number) {
    activeRoomId = id;
    messages = [];
    hasMore = false;
    composer = '';
    sendError = '';
    await loadThread(id);
  }

  async function handleSend() {
    if (activeRoomId == null) return;
    const body = composer.trim();
    if (!body || sending) return;
    sending = true;
    sendError = '';
    try {
      const res = await sendRoomMessage(activeRoomId, body);
      if (res.err === 0) {
        composer = '';
        await loadThread(activeRoomId);
        await loadRooms(activeRoomId);
      } else {
        sendError = res.msg ?? t('messages.sendError');
      }
    } catch {
      sendError = t('messages.sendError');
    } finally {
      sending = false;
    }
  }

  async function toggleMute() {
    if (activeRoomId == null || muting) return;
    muting = true;
    try {
      const res = await updateRoomPrefs(activeRoomId, { mute: !muted });
      if (res.err === 0) {
        muted = !muted;
        flash = muted ? t('messages.muted') : t('messages.unmuted');
      }
    } finally {
      muting = false;
    }
  }

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      goto('/');
      return;
    }
    const to = $page.url.searchParams.get('to');
    const room = $page.url.searchParams.get('room');
    if (to != null) {
      const userId = Number(to);
      if (Number.isInteger(userId) && userId > 0) {
        try {
          const res = await createDMRoom(userId);
          if (res.err === 0 && res.room_id != null) {
            await loadRooms(res.room_id);
            return;
          }
          flash = res.msg ?? t('messages.roomError');
        } catch {
          flash = t('messages.roomError');
        }
      }
    }
    if (room != null) {
      const roomId = Number(room);
      if (Number.isInteger(roomId) && roomId > 0) {
        await loadRooms(roomId);
        return;
      }
    }
    await loadRooms();
  });
</script>

<svelte:head><title>{t('messages.title')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content messages-layout">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('messages.title')}</h1>
    </header>

    {#if !auth.isLoggedIn}
      <blockquote class="archive-empty"><p>{t('messages.login')}</p></blockquote>
    {:else if loadingRooms}
      <p>{t('messages.loading')}</p>
    {:else if roomsError}
      <blockquote class="archive-empty"><p>{roomsError}</p></blockquote>
    {:else}
      {#if flash}<p class="archive-notice">{flash}</p>{/if}
      <div class="messages-columns">
        <aside class="messages-rooms">
          <h2>{t('messages.rooms')}</h2>
          {#if rooms.length === 0}
            <p>{t('messages.noRooms')}</p>
          {:else}
            <ul>
              {#each rooms as room (room.id)}
                <li>
                  <button
                    type="button"
                    class:active={room.id === activeRoomId}
                    onclick={() => selectRoom(room.id)}
                  >
                    {room.name ?? `${t('messages.dm')} #${room.id}`}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </aside>

        <section class="messages-thread">
          {#if activeRoomId == null}
            <p>{t('messages.pickRoom')}</p>
          {:else if loadingMessages}
            <p>{t('messages.loading')}</p>
          {:else if threadError}
            <blockquote class="archive-empty"><p>{threadError}</p></blockquote>
          {:else}
            <div class="messages-actions">
              <ArchiveButton onclick={toggleMute} disabled={muting}>
                {muted ? t('messages.unmute') : t('messages.mute')}
              </ArchiveButton>
            </div>
            {#if hasMore}
              <ArchiveButton
                onclick={() => {
                  const oldest = messages[0];
                  if (oldest) loadThread(activeRoomId!, oldest.id);
                }}
              >
                {t('messages.loadOlder')}
              </ArchiveButton>
            {/if}
            <ol class="messages-list">
              {#each messages as m (m.id)}
                <li class:mine={m.sender_id === auth.user?.id}>
                  <strong>{m.sender_username ?? `#${m.sender_id}`}</strong>
                  <span class="messages-time">{new Date(m.created_at).toLocaleString()}</span>
                  <p>{m.body}</p>
                </li>
              {/each}
            </ol>
            {#if sendError}<p class="archive-error">{sendError}</p>{/if}
            <form
              onsubmit={(e) => {
                e.preventDefault();
                handleSend();
              }}
            >
              <textarea
                class="arc-textarea"
                bind:value={composer}
                rows="3"
                placeholder={t('messages.composerPlaceholder')}
                aria-label={t('messages.composerPlaceholder')}
              ></textarea>
              <ArchiveButton onclick={handleSend} disabled={sending || !composer.trim()}>
                {t('messages.send')}
              </ArchiveButton>
            </form>
          {/if}
        </section>
      </div>
    {/if}
  </div>
</main>
{/if}

<style>
  .messages-layout {
    max-width: 1000px;
  }
  .messages-columns {
    display: grid;
    grid-template-columns: 240px 1fr;
    gap: 1.5rem;
  }
  .messages-rooms ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .messages-rooms button.active {
    font-weight: bold;
  }
  .messages-list {
    list-style: none;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .messages-time {
    font-size: 0.8em;
    opacity: 0.7;
    margin-left: 0.5rem;
  }
</style>
