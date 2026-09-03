<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface FlagRow {
    id: number;
    url_id: string;
    tag_id: number;
    flagged_by_ip: string;
    reason: string | null;
    resolved: boolean;
    created_at: string;
  }

  let flags = $state<FlagRow[]>([]);
  let loading = $state(true);
  let error = $state('');
  let actionMsg = $state('');
  let resolving = $state<number | null>(null);
  let checking = $state(true);

  const isAdmin = $derived(auth.level >= 100);

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn || auth.level < 100) {
      goto('/');
      return;
    }
    checking = false;
    await load();
    await loadAliases();
  });

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch('/api/curator/flags');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      flags = body.flags ?? [];
    } catch (e) {
      error = `Failed to load tag flags: ${e instanceof Error ? e.message : e}`;
      flags = [];
    } finally {
      loading = false;
    }
  }

  async function resolve(f: FlagRow) {
    resolving = f.id;
    actionMsg = '';
    try {
      const res = await adminFetch(`/api/curator/flags/${f.id}/resolve`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ resolved: true }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      if (body.err !== 0) throw new Error(`err ${body.err}`);
      // Drop the resolved flag from the open queue.
      flags = flags.filter((x) => x.id !== f.id);
      actionMsg = `✅ Flag #${f.id} resolved`;
    } catch (e) {
      actionMsg = `⚠️ Failed to resolve flag #${f.id}: ${e instanceof Error ? e.message : e}`;
    } finally {
      resolving = null;
    }
  }

  function fmtDate(iso: string): string {
    if (!iso) return '—';
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return d.toLocaleString();
  }

  // ── Tag aliases (list / create / delete) ───────────────────────────
  interface AliasRow {
    alias_name: string;
    canonical_tag_id: number;
    canonical_name: string | null;
    tag_type_id: number | null;
    created_at: string;
  }

  let aliases = $state<AliasRow[]>([]);
  let aliasMsg = $state('');
  let aliasError = $state('');
  let newAliasName = $state('');
  let newAliasTagId = $state('');
  let aliasBusy = $state('');

  async function loadAliases() {
    aliasError = '';
    try {
      const res = await adminFetch('/api/curator/aliases');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      aliases = body.aliases ?? [];
    } catch (e) {
      aliasError = `Failed to load aliases: ${e instanceof Error ? e.message : e}`;
    }
  }

  async function createAlias() {
    const name = newAliasName.trim();
    const tagId = Number(newAliasTagId);
    if (!name || !Number.isFinite(tagId) || tagId <= 0) {
      aliasError = 'Enter an alias name and a numeric canonical tag id.';
      return;
    }
    aliasBusy = 'create';
    aliasError = '';
    aliasMsg = '';
    try {
      const res = await adminFetch('/api/curator/alias', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ alias_name: name, canonical_tag_id: tagId }),
      });
      const body = await res.json().catch(() => ({ err: -1 }));
      if (!res.ok || body.err !== 0) throw new Error(body.msg ?? `HTTP ${res.status}`);
      aliasMsg = `Alias "${name}" created.`;
      newAliasName = '';
      newAliasTagId = '';
      await loadAliases();
    } catch (e) {
      aliasError = `Create failed: ${e instanceof Error ? e.message : e}`;
    } finally {
      aliasBusy = '';
    }
  }

  async function deleteAlias(a: AliasRow) {
    aliasBusy = a.alias_name;
    aliasError = '';
    aliasMsg = '';
    try {
      const res = await adminFetch(`/api/curator/aliases/${encodeURIComponent(a.alias_name)}`, {
        method: 'DELETE',
      });
      const body = await res.json().catch(() => ({ err: -1 }));
      if (!res.ok || body.err !== 0) throw new Error(body.msg ?? `HTTP ${res.status}`);
      aliasMsg = `Alias "${a.alias_name}" deleted.`;
      await loadAliases();
    } catch (e) {
      aliasError = `Delete failed: ${e instanceof Error ? e.message : e}`;
    } finally {
      aliasBusy = '';
    }
  }
</script>

{#if uiMode === 'archive'}
  {#if checking}
    <div class="loading"><p>Checking access...</p></div>
  {:else if isAdmin}
    <main class="archive-main">
      <div class="archive-content">
        <header class="archive-header">
          <h1 class="archive-page-title">Curator Flag Queue</h1>
          <p class="archive-summary">Tag flags reported by readers/curators. Resolving a flag removes it from this open queue. Note: the flag endpoints authenticate with the curator token, so this page only works when the token is configured on the server.</p>
        </header>
        <div class="archive-form-actions">
          <button class="archive-btn" type="button" onclick={load}>↻ Refresh</button>
        </div>
        {#if actionMsg}<p class="archive-msg" role="status">{actionMsg}</p>{/if}
        {#if loading}
          <p class="archive-note">Loading…</p>
        {:else if error}
          <p class="archive-error" role="alert">{error}</p>
        {:else if flags.length === 0}
          <p class="archive-note">No open tag flags.</p>
        {:else}
          <table class="archive-table">
            <thead><tr><th>ID</th><th>Work</th><th>Tag</th><th>Reason</th><th>Flagged by</th><th>Created</th><th>Action</th></tr></thead>
            <tbody>
              {#each flags as f (f.id)}
                <tr>
                  <td class="archive-mono">#{f.id}</td>
                  <td><a class="archive-link archive-mono" href={`/works/${f.url_id}`}>{f.url_id}</a></td>
                  <td>#{f.tag_id}</td>
                  <td>{f.reason ?? '—'}</td>
                  <td class="archive-mono">{f.flagged_by_ip}</td>
                  <td class="archive-mono">{fmtDate(f.created_at)}</td>
                  <td><button class="archive-btn" type="button" disabled={resolving === f.id} onclick={() => resolve(f)}>{resolving === f.id ? '…' : '✓ Resolve'}</button></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}

        <!-- ── Tag aliases ─────────────────────────────────────────── -->
        <header class="archive-header" style="margin-top: 2rem;">
          <h2 class="archive-page-title" style="font-size: 1.3em;">Tag Aliases</h2>
          <p class="archive-summary">Map alternative tag spellings to a canonical tag. Readers searching an alias see the canonical tag's works.</p>
        </header>

        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Create alias</legend>
          <form onsubmit={(e) => { e.preventDefault(); createAlias(); }}>
            <dl class="archive-dl">
              <div class="dl-row"><dt><label for="alias-name">Alias name</label></dt><dd><input id="alias-name" class="archive-input" type="text" placeholder="e.g. hp" bind:value={newAliasName} /></dd></div>
              <div class="dl-row"><dt><label for="alias-tag">Canonical tag ID</label></dt><dd><input id="alias-tag" class="archive-input" type="number" min="1" placeholder="e.g. 42" bind:value={newAliasTagId} /></dd></div>
            </dl>
            <div class="archive-form-actions">
              <button class="archive-btn archive-btn-primary" type="submit" disabled={aliasBusy === 'create'}>{aliasBusy === 'create' ? 'Creating…' : 'Create Alias'}</button>
            </div>
          </form>
        </fieldset>

        {#if aliasError}<p class="archive-error" role="alert">{aliasError}</p>{/if}
        {#if aliasMsg}<p class="archive-msg" role="status">{aliasMsg}</p>{/if}

        {#if aliases.length === 0}
          <p class="archive-note">No aliases defined yet.</p>
        {:else}
          <table class="archive-table">
            <thead><tr><th>Alias</th><th>Canonical tag</th><th>Type</th><th>Created</th><th>Action</th></tr></thead>
            <tbody>
              {#each aliases as a (a.alias_name)}
                <tr>
                  <td>{a.alias_name}</td>
                  <td>#{a.canonical_tag_id}{a.canonical_name ? ` — ${a.canonical_name}` : ''}</td>
                  <td>{a.tag_type_id ?? '—'}</td>
                  <td class="archive-mono">{fmtDate(a.created_at)}</td>
                  <td><button class="archive-btn" type="button" disabled={aliasBusy === a.alias_name} onclick={() => deleteAlias(a)}>{aliasBusy === a.alias_name ? '…' : 'Delete'}</button></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    </main>
  {/if}
{:else}
{#if checking}
  <div class="loading"><p>Checking access...</p></div>
{:else if isAdmin}
  <div class="flags-page">
    <header class="page-head">
      <h1>🚩 Curator Flag Queue</h1>
      <p class="subtitle">
        Tag flags reported by readers/curators. Resolving a flag removes it from this open queue.
        Note: the flag endpoints authenticate with the curator token (Authorization: Bearer),
        so this page only works when the token is configured on the server.
      </p>
      <div class="controls">
        <button class="btn" onclick={load}>↻ Refresh</button>
      </div>
    </header>

    {#if actionMsg}<p class="action-msg" role="status">{actionMsg}</p>{/if}

    {#if loading}
      <p class="empty">Loading…</p>
    {:else if error}
      <div class="error-card"><strong>⚠️ {error}</strong></div>
    {:else if flags.length === 0}
      <p class="empty">No open tag flags. 🎉</p>
    {:else}
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Work</th>
              <th>Tag</th>
              <th>Reason</th>
              <th>Flagged by</th>
              <th>Created</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {#each flags as f (f.id)}
              <tr>
                <td class="mono">#{f.id}</td>
                <td><a href={`/works/${f.url_id}`} class="mono">{f.url_id}</a></td>
                <td>#{f.tag_id}</td>
                <td>{f.reason ?? '—'}</td>
                <td class="mono">{f.flagged_by_ip}</td>
                <td class="mono">{fmtDate(f.created_at)}</td>
                <td>
                  <button
                    class="btn btn-resolve"
                    disabled={resolving === f.id}
                    onclick={() => resolve(f)}
                  >
                    {resolving === f.id ? '…' : '✓ Resolve'}
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
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
  .archive-content { max-width: 960px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
  .archive-form-actions { margin: 1rem 0; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td { text-align: left; padding: 0.45em 0.6em; border-bottom: 1px solid var(--archive-border, #dddddd); }
  .archive-table th { font-weight: 700; font-size: 0.85em; border-bottom: 2px solid var(--archive-border, #dddddd); }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-btn {
    display: inline-block; padding: 0.3em 1em; font-size: 0.85em; font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd); background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a); cursor: pointer; font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-msg { font-size: 0.92em; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.92em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .loading { text-align: center; padding: 3rem; }
  .flags-page { max-width: 1100px; margin: 0 auto; padding: 1rem; }
  .page-head h1 { margin-bottom: 0.3rem; }
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 90ch; }
  .controls { margin: 1rem 0; }
  .action-msg { font-size: 0.9rem; margin: 0.5rem 0; }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { text-align: left; padding: 0.5rem 0.7rem; border-bottom: 1px solid var(--color-border, #eee); }
  th { font-weight: 600; color: var(--color-text-muted, #888); font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.03em; }
  .mono { font-family: monospace; }
  .btn-resolve { background: #22c55e; color: #fff; border: none; padding: 0.25rem 0.6rem; border-radius: var(--radius-sm, 6px); cursor: pointer; font-size: 0.8rem; }
  .btn-resolve:disabled { opacity: 0.6; cursor: wait; }
  .empty { padding: 2rem; text-align: center; color: var(--color-text-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: var(--radius-sm, 6px); }
</style>
