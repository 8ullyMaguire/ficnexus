<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { adminFetch } from '$lib/api/admin';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface ModlogEntry {
    id: number;
    actor_id: number | null;
    actor_username: string | null;
    action: string;
    target_type: string;
    target_id: string;
    details: Record<string, unknown>;
    created_at: string;
  }

  let entries = $state<ModlogEntry[]>([]);
  let loading = $state(true);
  let error = $state('');
  let actionFilter = $state('');

  const ACTION_LABELS: Record<string, string> = {
    set_user_role: 'Changed user role',
    ban_user: 'Banned user',
    unban_user: 'Unbanned user',
    approve_upload: 'Approved upload',
    reject_upload: 'Rejected upload',
    approve_translation: 'Approved translation',
    reject_translation: 'Rejected translation',
    hide_comment: 'Hidden comment',
    delete_comment: 'Deleted comment',
    blacklist_fic: 'Blacklisted fic',
    blacklist_author: 'Blacklisted author',
    create_alias: 'Created tag alias',
    merge_tags: 'Merged tags',
    delete_tag: 'Deleted tag',
    resolve_flag: 'Resolved tag flag',
    propose_fix: 'Proposed body fix',
    vote_fix: 'Voted on body fix',
    delete_body: 'Deleted cached body',
  };

  function actionLabel(a: string): string {
    return ACTION_LABELS[a] ?? a;
  }

  async function load() {
    loading = true;
    error = '';
    try {
      const qs = actionFilter ? `?action=${encodeURIComponent(actionFilter)}` : '';
      const res = await adminFetch(`/api/modlog${qs}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      entries = body.entries ?? [];
    } catch (e) {
      error = `Failed to load modlog: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      goto('/login');
      return;
    }
    load();
  });

  function fmtTime(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function detailText(d: Record<string, unknown>): string {
    const parts = Object.entries(d ?? {})
      .filter(([, v]) => v !== null && v !== undefined && v !== '')
      .map(([k, v]) => `${k}: ${String(v)}`);
    return parts.length ? parts.join(' · ') : '';
  }
</script>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Moderation Log</h1>
      <p class="archive-summary">
        Complete transparency: every moderator, curator and admin action, visible to any logged-in user.
      </p>
      <div class="arc-modlog-controls">
        <select id="modlog-action" class="arc-select" bind:value={actionFilter} onchange={load} aria-label="Filter by action">
          <option value="">All actions</option>
          {#each Object.keys(ACTION_LABELS) as a}
            <option value={a}>{actionLabel(a)}</option>
          {/each}
        </select>
        <button class="archive-btn" type="button" onclick={load}>&#8635; Refresh</button>
      </div>
    </header>

    {#if loading}
      <blockquote class="archive-empty"><p>Loading&hellip;</p></blockquote>
    {:else if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
    {:else if entries.length === 0}
      <blockquote class="archive-empty"><p>No moderation actions recorded yet.</p></blockquote>
    {:else}
      <div class="table-wrap">
        <table class="archive-table">
          <thead>
            <tr>
              <th>When</th>
              <th>Actor</th>
              <th>Action</th>
              <th>Target</th>
              <th>Details</th>
            </tr>
          </thead>
          <tbody>
            {#each entries as e (e.id)}
              <tr>
                <td class="mono">{fmtTime(e.created_at)}</td>
                <td>{e.actor_username ?? '&mdash;'}</td>
                <td><span class="badge">{actionLabel(e.action)}</span></td>
                <td class="mono">{e.target_type ? `${e.target_type} ${e.target_id}` : '&mdash;'}</td>
                <td class="mono detail">{detailText(e.details)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</main>
{:else}
<div class="modlog-page">
  <header class="page-head">
    <h1>🛡️ Moderation Log</h1>
    <p class="subtitle">
      Complete transparency: every moderator, curator and admin action, visible to any logged-in user.
    </p>
    <div class="controls">
      <select bind:value={actionFilter} onchange={load}>
        <option value="">All actions</option>
        {#each Object.keys(ACTION_LABELS) as a}
          <option value={a}>{actionLabel(a)}</option>
        {/each}
      </select>
      <button class="btn" onclick={load}>↻ Refresh</button>
    </div>
  </header>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if entries.length === 0}
    <p class="empty">No moderation actions recorded yet.</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>When</th>
            <th>Actor</th>
            <th>Action</th>
            <th>Target</th>
            <th>Details</th>
          </tr>
        </thead>
        <tbody>
          {#each entries as e (e.id)}
            <tr>
              <td class="mono">{fmtTime(e.created_at)}</td>
              <td>{e.actor_username ?? '—'}</td>
              <td><span class="badge">{actionLabel(e.action)}</span></td>
              <td class="mono">{e.target_type ? `${e.target_type} ${e.target_id}` : '—'}</td>
              <td class="mono detail">{detailText(e.details)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
{/if}

<style>

  /* ── Archive mode ─────────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: 820px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    margin-bottom: 1.25rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.6em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8em;
    font-weight: 700;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
  }
  .archive-summary {
    color: var(--archive-muted, #666666);
    font-size: 0.95em;
    margin: 0.2em 0 0;
  }
  .archive-error {
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
    padding: 0.8em 1em;
    font-weight: 700;
    margin: 0.5rem 0;
  }
  .archive-empty {
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.6em 1em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
    margin: 1rem 0;
  }
  .archive-empty p { margin: 0.2em 0; }

  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.8em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    text-decoration: none;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .archive-table-wrap { overflow-x: auto; margin: 0 0 1rem; }
  .archive-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.92em;
    border: 1px solid var(--archive-border, #dddddd);
  }
  .archive-table th, .archive-table td {
    text-align: left;
    padding: 0.45em 0.7em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    vertical-align: top;
  }
  .archive-table th {
    color: var(--archive-muted, #666666);
    font-weight: 700;
    font-size: 0.82em;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .archive-table tbody tr:last-child td { border-bottom: none; }
  .arc-modlog-controls { display: flex; gap: 0.5rem; margin-top: 0.75em; align-items: center; flex-wrap: wrap; }
  .arc-select {
    padding: 0.35em 0.5em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.9em;
    max-width: 22rem;
  }
  .arc-mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.88em; }
  .modlog-page { padding: 1rem; max-width: 1100px; margin: 0 auto; }
  .page-head { margin-bottom: 1rem; }
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .controls { margin-top: 0.5rem; display: flex; gap: 0.5rem; align-items: center; }
  .btn {
    background: var(--color-primary, #4a7);
    color: #fff; border: 0; border-radius: 6px; padding: 0.45rem 0.9rem; cursor: pointer;
  }
  select {
    padding: 0.4rem; border-radius: 6px; border: 1px solid var(--color-border, #ddd);
    background: var(--color-surface, #fff);
  }
  .empty { color: var(--color-text-muted, #888); padding: 1rem 0; }
  .error-card { background: #fdd; border: 1px solid #f99; padding: 0.75rem; border-radius: 6px; }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { padding: 0.5rem 0.7rem; text-align: left; border-bottom: 1px solid var(--color-border, #eee); }
  th { color: var(--color-text-muted, #888); font-weight: 600; }
  .mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.82rem; }
  .detail { color: var(--color-text-muted, #666); max-width: 320px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .badge {
    background: #e8f0fe; color: #1a56db; padding: 2px 8px; border-radius: 10px; font-size: 0.78rem;
  }
</style>
