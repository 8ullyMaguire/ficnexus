  <script lang="ts">
  // Curator queue for work deletion requests. Lists pending proposals from
  // GET /api/curator/work-deletions (already filtered to status='pending') and
  // lets a curator Approve (permanently deletes the work — confirmed first,
  // cannot be undone) or Reject each one, which resolves the row server-side
  // and logs to the modlog.
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { adminFetch } from '$lib/api/admin';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { relativeTime } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  interface DeletionProposal {
    id: number;
    url_id: string;
    reason: string | null;
    proposed_by: number | null;
    proposed_by_username: string | null;
    status: string;
    created_at: string;
  }

  let proposals = $state<DeletionProposal[]>([]);
  let loading = $state(true);
  let error = $state('');
  let actionMsg = $state('');
  let busyId = $state<number | null>(null);
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
  });

  function displayName(p: DeletionProposal): string {
    return p.proposed_by_username ?? (p.proposed_by != null ? String(p.proposed_by) : '—');
  }

  async function load() {
    loading = true;
    error = '';
    actionMsg = '';
    try {
      const res = await adminFetch('/api/curator/work-deletions');
      if (!res.ok) {
        let msg = `HTTP ${res.status}`;
        try {
          const b = await res.json();
          if (b?.msg) msg = b.msg;
        } catch { /* ignore */ }
        throw new Error(msg);
      }
      const body = await res.json();
      proposals = Array.isArray(body?.proposals) ? body.proposals : [];
    } catch (e) {
      error = `${t('workDeletions.error')} ${e instanceof Error ? e.message : e}`;
      proposals = [];
    } finally {
      loading = false;
    }
  }

  async function resolve(p: DeletionProposal, action: 'approve' | 'reject') {
    if (busyId !== null) return;
    if (action === 'approve' && !confirm(t('workDeletions.confirm'))) return;
    busyId = p.id;
    error = '';
    actionMsg = '';
    try {
      const res = await adminFetch(`/api/curator/work-deletions/${p.id}/resolve`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action }),
      });
      if (!res.ok) {
        let msg = `HTTP ${res.status}`;
        try {
          const b = await res.json();
          if (b?.msg) msg = b.msg;
        } catch { /* ignore */ }
        throw new Error(msg);
      }
      const body = await res.json();
      if (body?.err && body.err !== 0) throw new Error(`err ${body.err}`);
      // Refetch so the resolved proposal disappears from the pending queue.
      await load();
    } catch (e) {
      error = `${t('workDeletions.error')} ${e instanceof Error ? e.message : e}`;
    } finally {
      busyId = null;
    }
  }
</script>

<svelte:head><title>{t('workDeletions.title')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  {#if checking}
    <div class="loading"><p>Checking access...</p></div>
  {:else if isAdmin}
    <main class="archive-main">
      <div class="archive-content">
        <header class="archive-header">
          <h1 class="archive-page-title">{t('workDeletions.title')}</h1>
          <p class="archive-summary">{t('workDeletions.subtitle')}</p>
          <div class="archive-form-actions">
            <button class="archive-btn" type="button" onclick={load}>↻ {t('approvals.refresh')}</button>
          </div>
        </header>
        {#if error}
          <p class="archive-error" role="alert">{error}</p>
        {/if}
        {#if actionMsg}<p class="archive-msg" role="status">{actionMsg}</p>{/if}
        {#if loading}
          <p class="archive-note">{t('approvals.loading')}</p>
        {:else if proposals.length === 0}
          <p class="archive-note">{t('workDeletions.empty')}</p>
        {:else}
          <table class="archive-table">
            <thead><tr><th>Work</th><th>Reason</th><th>Proposed by</th><th>Created</th><th>Action</th></tr></thead>
            <tbody>
              {#each proposals as p (p.id)}
                <tr>
                  <td><a class="archive-link archive-mono" href={`/works/${p.url_id}`}>{p.url_id}</a></td>
                  <td>{p.reason ?? '—'}</td>
                  <td class="archive-mono">{displayName(p)}</td>
                  <td class="archive-mono">{relativeTime(p.created_at)}</td>
                  <td>
                    <button class="archive-btn ok" type="button" disabled={busyId !== null} onclick={() => resolve(p, 'approve')}>{busyId === p.id ? '…' : t('approvals.approve')}</button>
                    <button class="archive-btn bad" type="button" disabled={busyId !== null} onclick={() => resolve(p, 'reject')}>{busyId === p.id ? '…' : t('approvals.reject')}</button>
                  </td>
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
    <div class="del-page">
      <header class="page-head">
        <h1>{t('workDeletions.title')}</h1>
        <p class="subtitle">{t('workDeletions.subtitle')}</p>
        <div class="controls">
          <button class="btn" onclick={load}>↻ {t('approvals.refresh')}</button>
        </div>
      </header>

      {#if error}
        <div class="error-card"><strong>{error}</strong></div>
      {/if}

      {#if loading}
        <p class="empty">{t('approvals.loading')}</p>
      {:else if proposals.length === 0}
        <p class="empty">{t('workDeletions.empty')}</p>
      {:else}
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Work</th>
                <th>Reason</th>
                <th>Proposed by</th>
                <th>Created</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {#each proposals as p (p.id)}
                <tr>
                  <td><a class="mono" href={`/works/${p.url_id}`}>{p.url_id}</a></td>
                  <td>{p.reason ?? '—'}</td>
                  <td class="mono">{displayName(p)}</td>
                  <td class="mono">{relativeTime(p.created_at)}</td>
                  <td>
                    <button class="btn ok" disabled={busyId !== null} onclick={() => resolve(p, 'approve')}>{busyId === p.id ? '…' : t('approvals.approve')}</button>
                    <button class="btn bad" disabled={busyId !== null} onclick={() => resolve(p, 'reject')}>{busyId === p.id ? '…' : t('approvals.reject')}</button>
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
    color: var(--archive-text, #2a2a2a); cursor: pointer; font-family: inherit; margin-right: 0.4em;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-btn.ok { color: #1a7a36; border-color: #1a7a36; }
  .archive-btn.bad { color: #990000; border-color: #990000; }
  .archive-msg { font-size: 0.92em; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.92em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .loading { text-align: center; padding: 3rem; }
  .del-page { max-width: 1100px; margin: 0 auto; padding: 1rem; }
  .page-head h1 { margin-bottom: 0.3rem; }
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 90ch; }
  .controls { margin: 1rem 0; }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { text-align: left; padding: 0.5rem 0.7rem; border-bottom: 1px solid var(--color-border, #eee); }
  th { font-weight: 600; color: var(--color-text-muted, #888); font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.03em; }
  .mono { font-family: monospace; }
  .btn { border: 1px solid var(--color-border, #e5e7eb); background: var(--color-surface-1, #fff); border-radius: var(--radius-sm, 6px); padding: 0.3rem 0.7rem; cursor: pointer; font-size: 0.82rem; margin-right: 0.4rem; }
  .btn:disabled { opacity: 0.6; cursor: wait; }
  .btn.ok { color: #1a7a36; border-color: #1a7a36; }
  .btn.bad { color: #990000; border-color: #990000; }
  .empty { padding: 2rem; text-align: center; color: var(--color-text-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: var(--radius-sm, 6px); }
</style>
