<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let users = $state<any[]>([]);
  let query = $state('');
  let loading = $state(true);

  async function loadUsers(q?: string) {
    loading = true;
    try {
      const url = q ? `/api/admin/users?q=${encodeURIComponent(q)}` : '/api/admin/users';
      const res = await adminFetch(url, { credentials: 'include' });
      if (res.ok) {
        const data = await res.json();
        users = Array.isArray(data.users) ? data.users : [];
      }
    } catch { /* */ }
    finally { loading = false; }
  }

  async function changeRole(userId: number, role: number) {
    await adminFetch(`/api/admin/users/${userId}/role`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ role }), credentials: 'include' });
    await loadUsers(query);
  }

  async function toggleBan(userId: number, isBanned: boolean) {
    await adminFetch(`/api/admin/users/${userId}/ban`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ is_banned: !isBanned }), credentials: 'include' });
    await loadUsers(query);
  }

  function handleSearch(e: Event) {
    e.preventDefault();
    loadUsers(query);
  }

  onMount(() => loadUsers());
</script>

<svelte:head><title>Users | FicNexus Admin</title></svelte:head>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">User Management</h1>
      </header>

      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Search users</legend>
        <form onsubmit={handleSearch}>
          <dl class="archive-dl">
            <dt><label for="au-search">Username</label></dt>
            <dd>
              <input id="au-search" class="archive-input" type="text" bind:value={query} placeholder="Search by username..." />
            </dd>
          </dl>
          <div class="archive-form-actions">
            <button class="archive-btn archive-btn-primary" type="submit">Search</button>
          </div>
        </form>
      </fieldset>

      {#if loading}
        <p class="archive-note">Loading...</p>
      {:else if users.length === 0}
        <p class="archive-note">No users found.</p>
      {:else}
        <table class="archive-table">
          <thead>
            <tr>
              <th>ID</th><th>Username</th><th>Role</th><th>Rep</th><th>Words</th><th>Locale</th><th>Status</th><th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each users as user (user.id)}
              <tr>
                <td>{user.id}</td>
                <td><strong>{user.username}</strong></td>
                <td>
                  <select class="archive-select" value={user.role} onchange={(e) => changeRole(user.id, parseInt((e.target as HTMLSelectElement).value))}>
                    <option value={0}>Regular</option>
                    <option value={1}>Trusted</option>
                    <option value={5}>Curator</option>
                    <option value={10}>Admin</option>
                  </select>
                </td>
                <td>{user.reputation}</td>
                <td>{(user.total_words_read || 0).toLocaleString()}</td>
                <td>{user.locale || 'en'}</td>
                <td>{user.is_banned ? '🚫 Banned' : '✅ Active'}</td>
                <td>
                  <button
                    class="archive-btn"
                    type="button"
                    onclick={() => toggleBan(user.id, user.is_banned)}
                  >{user.is_banned ? 'Unban' : 'Ban'}</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </main>
{:else}
<h1>User Management</h1>

<form onsubmit={handleSearch} class="search-bar">
  <input type="text" bind:value={query} placeholder="Search by username..." />
  <button type="submit">Search</button>
</form>

{#if loading}
  <p>Loading...</p>
{:else}
  <table class="admin-table">
    <thead><tr><th>ID</th><th>Username</th><th>Role</th><th>Rep</th><th>Words</th><th>Locale</th><th>Status</th><th>Actions</th></tr></thead>
    <tbody>
      {#each users as user}
        <tr>
          <td>{user.id}</td>
          <td><strong>{user.username}</strong></td>
          <td>
            <select value={user.role} onchange={(e) => changeRole(user.id, parseInt((e.target as HTMLSelectElement).value))}>
              <option value={0}>Regular</option>
              <option value={1}>Trusted</option>
              <option value={5}>Curator</option>
              <option value={10}>Admin</option>
            </select>
          </td>
          <td>{user.reputation}</td>
          <td>{(user.total_words_read || 0).toLocaleString()}</td>
          <td>{user.locale || 'en'}</td>
          <td>{user.is_banned ? '🚫 Banned' : '✅ Active'}</td>
          <td><button class={user.is_banned ? 'unban' : 'ban'} onclick={() => toggleBan(user.id, user.is_banned)}>{user.is_banned ? 'Unban' : 'Ban'}</button></td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content { max-width: 1040px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 1em 1.25em;
    margin: 1.2rem 0;
  }
  .archive-legend {
    font-weight: 700;
    font-size: 1.05em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.4em;
  }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt { font-weight: 700; font-size: 0.9em; margin: 0.85em 0 0.2em; }
  .archive-dl dt:first-child { margin-top: 0; }
  .archive-dl dd { margin: 0; }
  .archive-input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
  }
  .archive-select {
    padding: 0.25em 0.4em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9em;
    background: var(--archive-bg, #ffffff);
  }
  .archive-form-actions { margin-top: 0.75em; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td {
    text-align: left;
    padding: 0.45em 0.5em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-table th {
    font-weight: 700;
    font-size: 0.88em;
    border-bottom: 2px solid var(--archive-border, #dddddd);
  }
  .archive-btn {
    display: inline-block;
    padding: 0.25em 1em;
    font-size: 0.82em;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    cursor: pointer;
    font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn-primary {
    background: var(--archive-link, #990000);
    color: #ffffff;
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover:not(:disabled) { background: #7a0000; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.92em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .search-bar { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
  .search-bar input { flex: 1; padding: 0.4rem 0.8rem; border-radius: var(--radius-sm); border: 1px solid var(--color-border); }
  .search-bar button { padding: 0.4rem 1rem; border-radius: var(--radius-sm); border: 1px solid var(--color-border); background: var(--color-primary); color: white; cursor: pointer; }
  .admin-table { width: 100%; border-collapse: collapse; }
  .admin-table th, .admin-table td { padding: 0.5rem; text-align: left; border-bottom: 1px solid var(--color-border); font-size: 0.88rem; }
  .admin-table th { font-weight: 600; background: var(--color-surface-2); }
  select { padding: 0.2rem; border-radius: var(--radius-sm); border: 1px solid var(--color-border); font-size: 0.85rem; }
  .ban { background: #ef4444; color: white; border: none; padding: 0.3rem 0.6rem; border-radius: var(--radius-sm); cursor: pointer; }
  .unban { background: #22c55e; color: white; border: none; padding: 0.3rem 0.6rem; border-radius: var(--radius-sm); cursor: pointer; }
</style>
