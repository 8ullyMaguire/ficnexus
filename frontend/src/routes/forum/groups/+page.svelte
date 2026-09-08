<script lang="ts">
  // Lane 5: Forum groups — list, create, join/leave.
  import { onMount } from 'svelte';
  import { listGroups, createGroup, joinGroup, leaveGroup, type ForumGroup } from '$lib/api/groups';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let loading = $state(true);
  let error = $state('');
  let groups = $state<ForumGroup[]>([]);
  let filter = $state<'all' | 'public' | 'private'>('all');
  let search = $state('');
  let searchInput = $state('');

  // Create group form
  let showCreate = $state(false);
  let newName = $state('');
  let newDesc = $state('');
  let newPrivate = $state(false);
  let creating = $state(false);
  let createError = $state('');

  let actingId = $state<number | null>(null);

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await listGroups({
        type: filter === 'all' ? undefined : filter,
        search: search || undefined,
      });
      if (res.err === 0) {
        groups = res.items ?? [];
      } else {
        error = res.msg ?? 'Failed to load groups';
      }
    } catch {
      error = 'Failed to load groups';
    } finally {
      loading = false;
    }
  }

  onMount(load);

  function doSearch() {
    search = searchInput;
    load();
  }

  async function handleCreate() {
    if (!newName.trim()) return;
    creating = true;
    createError = '';
    try {
      const res = await createGroup({
        name: newName.trim(),
        description: newDesc.trim() || undefined,
        is_private: newPrivate,
      });
      if (res.err === 0) {
        showCreate = false;
        newName = '';
        newDesc = '';
        newPrivate = false;
        await load();
      } else {
        createError = res.msg ?? 'Failed to create group';
      }
    } catch {
      createError = 'Failed to create group';
    } finally {
      creating = false;
    }
  }

  async function handleJoinLeave(g: ForumGroup) {
    if (!auth.isLoggedIn) return;
    actingId = g.id;
    try {
      const res = g.caller_is_member
        ? await leaveGroup(g.id)
        : await joinGroup(g.id);
      if (res.err === 0) {
        g.caller_is_member = !g.caller_is_member;
        g.member_count += g.caller_is_member ? 1 : -1;
      }
    } finally {
      actingId = null;
    }
  }
</script>

<svelte:head><title>Groups — FicNexus</title></svelte:head>

<div class="groups-page">
  <header class="page-head">
    <a class="back" href="/forum">← Forum</a>
    <h1>Groups</h1>
    {#if auth.isLoggedIn}
      <button class="btn btn-primary" type="button" onclick={() => (showCreate = !showCreate)}>
        {showCreate ? 'Cancel' : 'Create Group'}
      </button>
    {/if}
  </header>

  {#if showCreate}
    <form class="create-form" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
      <label>
        Name
        <input type="text" bind:value={newName} maxlength="60" required placeholder="Group name" />
      </label>
      <label>
        Description
        <textarea bind:value={newDesc} rows="3" maxlength="500" placeholder="What is this group about?"></textarea>
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={newPrivate} />
        Private (invite-only)
      </label>
      {#if createError}<p class="error">{createError}</p>{/if}
      <button class="btn btn-primary" type="submit" disabled={creating || !newName.trim()}>
        {creating ? 'Creating…' : 'Create'}
      </button>
    </form>
  {/if}

  <div class="filters">
    <div class="filter-tabs">
      {#each ['all', 'public', 'private'] as f}
        <button
          class="tab"
          class:active={filter === f}
          type="button"
          onclick={() => { filter = f as typeof filter; load(); }}
        >{f}</button>
      {/each}
    </div>
    <form class="search-form" onsubmit={(e) => { e.preventDefault(); doSearch(); }}>
      <input type="text" bind:value={searchInput} placeholder="Search groups…" />
      <button class="btn" type="submit">Search</button>
    </form>
  </div>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if groups.length === 0}
    <p class="muted">No groups found.</p>
  {:else}
    <ul class="group-list">
      {#each groups as g (g.id)}
        <li class="group-card">
          <div class="group-info">
            <a href="/forum/groups/{g.id}" class="group-name">{g.name}</a>
            <span class="group-meta">
              {g.member_count} member{g.member_count === 1 ? '' : 's'}
              {#if g.is_private}· Private{/if}
              {#if g.is_system}· System{/if}
              · by {g.owner_username}
            </span>
            {#if g.description}
              <p class="group-desc">{g.description}</p>
            {/if}
          </div>
          {#if auth.isLoggedIn && !g.is_system}
            <button
              class="btn"
              class:btn-primary={!g.caller_is_member}
              class:btn-danger={g.caller_is_member}
              type="button"
              disabled={actingId === g.id}
              onclick={() => handleJoinLeave(g)}
            >
              {actingId === g.id ? '…' : g.caller_is_member ? 'Leave' : 'Join'}
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .groups-page { max-width: 700px; margin: 0 auto; padding: 1.5rem; }
  .page-head { margin-bottom: 1.25rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .create-form { display: flex; flex-direction: column; gap: 0.75rem; margin-bottom: 1.5rem; padding: 1rem; border: 1px solid var(--color-border, #ddd); border-radius: 8px; }
  .create-form label { display: flex; flex-direction: column; gap: 0.3rem; font-weight: 600; }
  .create-form input, .create-form textarea { padding: 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; font: inherit; }
  .checkbox-label { flex-direction: row !important; align-items: center; gap: 0.5rem; }
  .filters { display: flex; gap: 1rem; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; }
  .filter-tabs { display: flex; gap: 0.25rem; }
  .tab { padding: 0.4rem 0.8rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; background: none; cursor: pointer; font: inherit; }
  .tab.active { background: var(--color-primary, #2b4bd7); color: white; border-color: var(--color-primary, #2b4bd7); }
  .search-form { display: flex; gap: 0.5rem; }
  .search-form input { padding: 0.4rem 0.6rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; font: inherit; }
  .group-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75rem; }
  .group-card { display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1rem; border: 1px solid var(--color-border, #ddd); border-radius: 8px; gap: 1rem; }
  .group-info { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .group-name { font-weight: 600; color: var(--color-link, #2b4bd7); text-decoration: none; }
  .group-name:hover { text-decoration: underline; }
  .group-meta { font-size: 0.8rem; color: var(--color-text-muted, #888); }
  .group-desc { font-size: 0.9rem; color: var(--color-text, #333); margin: 0.2rem 0 0; }
  .error { color: #c62828; }
  .muted { color: var(--color-text-muted, #888); }
  .btn { padding: 0.4rem 0.8rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; background: none; cursor: pointer; font: inherit; }
  .btn-primary { background: var(--color-primary, #2b4bd7); color: white; border-color: var(--color-primary, #2b4bd7); }
  .btn-danger { background: #c62828; color: white; border-color: #c62828; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
