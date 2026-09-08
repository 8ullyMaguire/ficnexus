<script lang="ts">
  // Lane 5: Group detail — view members, join/leave, manage (owner).
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getGroup, joinGroup, leaveGroup, deleteGroup, type GroupDetail } from '$lib/api/groups';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let groupId = $derived(Number(($page?.params as Record<string, string>)?.groupId ?? 0));
  let group = $state<GroupDetail | null>(null);
  let loading = $state(true);
  let error = $state('');
  let acting = $state(false);
  let deleting = $state(false);

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await getGroup(groupId);
      if (res.err === 0 && res.group) {
        group = res.group;
      } else {
        error = res.msg ?? 'Group not found';
      }
    } catch {
      error = 'Failed to load group';
    } finally {
      loading = false;
    }
  }

  onMount(load);

  async function handleJoinLeave() {
    if (!group || !auth.isLoggedIn) return;
    acting = true;
    try {
      const res = group.caller_is_member
        ? await leaveGroup(group.id)
        : await joinGroup(group.id);
      if (res.err === 0) {
        group.caller_is_member = !group.caller_is_member;
        group.member_count += group.caller_is_member ? 1 : -1;
      }
    } finally {
      acting = false;
    }
  }

  async function handleDelete() {
    if (!group || !confirm('Delete this group? This cannot be undone.')) return;
    deleting = true;
    try {
      const res = await deleteGroup(group.id);
      if (res.err === 0) {
        window.location.href = '/forum/groups';
      }
    } finally {
      deleting = false;
    }
  }

  const isOwner = $derived(group && auth.user?.id === group.owner_id);
</script>

<svelte:head><title>{group?.name ?? 'Group'} — FicNexus</title></svelte:head>

<div class="group-detail">
  <header class="page-head">
    <a class="back" href="/forum/groups">← Groups</a>
    {#if loading}
      <p class="muted">Loading…</p>
    {:else if error}
      <p class="error">{error}</p>
    {:else if group}
      <h1>{group.name}</h1>
      <span class="meta">
        {group.member_count} member{group.member_count === 1 ? '' : 's'}
        {#if group.is_private}· Private{/if}
        {#if group.is_system}· System{/if}
        · by {group.owner_username}
      </span>
      {#if group.description}
        <p class="desc">{group.description}</p>
      {/if}

      <div class="actions">
        {#if auth.isLoggedIn && !group.is_system}
          <button
            class="btn"
            class:btn-primary={!group.caller_is_member}
            class:btn-danger={group.caller_is_member}
            type="button"
            disabled={acting}
            onclick={handleJoinLeave}
          >
            {acting ? '…' : group.caller_is_member ? 'Leave' : 'Join'}
          </button>
        {/if}
        {#if isOwner}
          <button class="btn btn-danger" type="button" disabled={deleting} onclick={handleDelete}>
            {deleting ? 'Deleting…' : 'Delete Group'}
          </button>
        {/if}
      </div>
    {/if}
  </header>

  {#if group && group.members}
    <h2>Members</h2>
    <ul class="member-list">
      {#each group.members as m (m.user_id)}
        <li class="member">
          <span class="member-name">{m.username}</span>
          <span class="member-role">{m.role}</span>
          <span class="member-date">joined {new Date(m.joined_at).toLocaleDateString()}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .group-detail { max-width: 700px; margin: 0 auto; padding: 1.5rem; }
  .page-head { margin-bottom: 1.25rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .meta { font-size: 0.85rem; color: var(--color-text-muted, #888); }
  .desc { color: var(--color-text, #333); }
  .actions { display: flex; gap: 0.5rem; margin-top: 0.5rem; }
  .member-list { list-style: none; padding: 0; margin: 0.5rem 0; display: flex; flex-direction: column; gap: 0.4rem; }
  .member { display: flex; gap: 1rem; align-items: center; padding: 0.4rem 0; border-bottom: 1px solid var(--color-border, #eee); }
  .member-name { font-weight: 600; }
  .member-role { font-size: 0.8rem; color: var(--color-text-muted, #888); text-transform: capitalize; }
  .member-date { font-size: 0.8rem; color: var(--color-text-muted, #888); }
  .error { color: #c62828; }
  .muted { color: var(--color-text-muted, #888); }
  .btn { padding: 0.4rem 0.8rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; background: none; cursor: pointer; font: inherit; }
  .btn-primary { background: var(--color-primary, #2b4bd7); color: white; border-color: var(--color-primary, #2b4bd7); }
  .btn-danger { background: #c62828; color: white; border-color: #c62828; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
