  <script lang="ts">
  // Unified curator Approvals queue: pending items across every curation and
  // moderation type — collection add-requests, content-fix and metadata
  // proposals, comment triage rows, and forum edit proposals — each with its
  // community vote tally and curator actions.
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';
  import { t } from '$lib/i18n/index.svelte';

  const uiMode = $derived(getPref('uiMode'));

  interface ApprovalItem {
    id: number;
    type: string;
    status: string;
    created_at: string;
    proposer?: string | null;
    votes_for?: number;
    votes_against?: number;
    my_vote?: number;
    // collection
    collection_id?: number;
    collection_title?: string;
    work_title?: string;
    work_author?: string;
    blurb?: string;
    // fix / metadata
    url_id?: string;
    reason?: string;
    field?: string;
    old_value?: string;
    new_value?: string;
    // comment triage
    comment_id?: number;
    body?: string;
    category?: string;
    // forum
    target_type?: string;
    target_id?: number;
    snapshot?: Record<string, unknown>;
  }

  const TYPE_ORDER = [
    'collection_item_request',
    'curator_fix_proposal',
    'metadata_proposal',
    'comment_triage',
    'forum_edit_proposal',
  ];

  let items = $state<ApprovalItem[]>([]);
  let counts = $state<Record<string, number>>({});
  let loading = $state(true);
  let error = $state('');
  let statusFilter = $state('pending');
  let typeFilter = $state('all');
  let curatorFilter = $state('');
  let busyId = $state<string>('');

  async function load() {
    loading = true;
    error = '';
    try {
      const params = new URLSearchParams();
      params.set('status', statusFilter);
      if (typeFilter !== 'all') params.set('type', typeFilter);
      if (curatorFilter.trim()) params.set('curator', curatorFilter.trim());
      const res = await adminFetch(`/api/curator/approvals?${params.toString()}`);
      if (!res.ok) {
        let msg = `HTTP ${res.status}`;
        try {
          const b = await res.json();
          if (b?.msg) msg = b.msg;
        } catch { /* ignore */ }
        throw new Error(msg);
      }
      const body = await res.json();
      items = body.items ?? [];
      counts = body.pending_counts ?? {};
    } catch (e) {
      error = `${t('approvals.error')} ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  async function act(url: string, body?: Record<string, unknown>) {
    if (busyId) return;
    busyId = url;
    error = '';
    try {
      const res = await adminFetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: body ? JSON.stringify(body) : undefined,
      });
      if (!res.ok) {
        let msg = `HTTP ${res.status}`;
        try {
          const b = await res.json();
          if (b?.msg) msg = b.msg;
        } catch { /* ignore */ }
        throw new Error(msg);
      }
      await load();
    } catch (e) {
      error = `${t('approvals.error')} ${e instanceof Error ? e.message : e}`;
    } finally {
      busyId = '';
    }
  }

  function buildUrl(path: string, item: ApprovalItem, type: string): string {
    if (type === 'collection_item_request') {
      return `/api/collections/${item.collection_id}/requests/${item.id}/${path}`;
    }
    return path;
  }

  function clearFilters() {
    statusFilter = 'pending';
    typeFilter = 'all';
    curatorFilter = '';
    load();
  }

  function fmtTime(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function myVoteLabel(v: number): string {
    if (v === 1) return t('approvals.agree');
    if (v === -1) return t('approvals.disagree');
    return t('approvals.abstain');
  }

  function groupTitle(type: string): string {
    return t(`approvals.type.${type}`) as string;
  }

  function summary(item: ApprovalItem): string {
    switch (item.type) {
      case 'collection_item_request':
        return `${item.work_title ?? item.id} — ${item.work_author ?? ''}`;
      case 'curator_fix_proposal':
        return `${t('approvals.type.curator_fix_proposal')}: ${item.url_id ?? item.id}`;
      case 'metadata_proposal':
        return `${item.field ?? ''}: ${item.old_value ?? ''} → ${item.new_value ?? ''}`;
      case 'comment_triage':
        return item.body ?? '';
      case 'forum_edit_proposal':
        return `${t('approvals.type.forum_edit_proposal')} #${item.target_id ?? item.id}`;
      default:
        return String(item.id);
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

  // Compose the ordered groups with their live items.
  const groups = $derived(
    TYPE_ORDER.map((type) => ({ type, label: groupTitle(type), items: items.filter((i) => i.type === type) }))
      .filter((g) => g.items.length > 0),
  );

  const typeOptions = TYPE_ORDER;
  const statusOptions = ['pending', 'approved', 'rejected'];
</script>

{#if uiMode === 'archive'}
  <main class="archive-main appr">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">{t('approvals.title')}</h1>
        <p class="archive-summary">{t('approvals.subtitle')}</p>
        <div class="arc-ctl">
          <label>{t('approvals.filterType')}
            <select class="arc-select" bind:value={typeFilter} onchange={load}>
              <option value="all">{t('approvals.allTypes')}</option>
              {#each typeOptions as k}
                <option value={k}>{groupTitle(k)}{counts[k] !== undefined ? ` (${counts[k]})` : ''}</option>
              {/each}
            </select>
          </label>
          <label>{t('approvals.filterStatus')}
            <select class="arc-select" bind:value={statusFilter} onchange={load}>
              {#each statusOptions as s}
                <option value={s}>{t(`approvals.status.${s}`)}</option>
              {/each}
            </select>
          </label>
          <input class="arc-select" type="text" placeholder={t('approvals.filterCurator')} bind:value={curatorFilter}
                 onkeydown={(e) => e.key === 'Enter' && load()} />
          <button class="archive-btn" type="button" onclick={load}>&#8635; {t('approvals.refresh')}</button>
          <button class="archive-btn" type="button" onclick={clearFilters}>{t('approvals.clear')}</button>
        </div>
      </header>

      {#if error}
        <div class="archive-error"><strong>&#9888; {error}</strong></div>
      {/if}
      {#if loading}
        <blockquote class="archive-empty"><p>{t('approvals.loading')}</p></blockquote>
      {:else if groups.length === 0}
        <blockquote class="archive-empty"><p>{t('approvals.empty')}</p></blockquote>
      {:else}
        <div class="appr-groups">
          {#each groups as g (g.type)}
            <section class="appr-group">
              <h2 class="appr-group-title">{g.label}
                {#if counts[g.type] !== undefined}<span class="arc-count">{counts[g.type]}</span>{/if}
              </h2>
              {#each g.items as it (it.id)}
                <article class="appr-card">
                  <div class="appr-summary">{summary(it)}</div>
                  <div class="appr-meta">
                    {#if it.type === 'collection_item_request'}
                      <span class="arc-note">{t('approvals.type.collection_item_request')} {it.collection_title ?? ''}</span>
                      {#if it.blurb}<span class="arc-note">“{it.blurb}”</span>{/if}
                    {:else if it.type === 'comment_triage'}
                      {#if it.work_title}<span class="arc-note">{it.work_title}</span>{/if}
                      <span class="arc-note">{it.category}</span>
                    {:else if it.type === 'forum_edit_proposal'}
                      <span class="arc-note">{it.target_type ?? ''} #{it.target_id ?? it.id}</span>
                    {/if}
                    {#if it.type === 'curator_fix_proposal' && it.reason}
                      <span class="arc-note">{it.reason}</span>
                    {/if}
                  </div>
                  <div class="appr-row">
                    <span class="arc-note">{t('approvals.proposer')} {it.proposer ?? '—'} · {fmtTime(it.created_at)}</span>
                    {#if it.votes_for !== undefined}
                      <span class="arc-votes">
                        <span class="arc-vote for">{t('approvals.agree')} {it.votes_for}</span>
                        <span class="arc-vote against">{t('approvals.disagree')} {it.votes_against}</span>
                        <span class="arc-vote mine">{t('approvals.myVote')}: {myVoteLabel(it.my_vote ?? 0)}</span>
                      </span>
                    {/if}
                  </div>
                  <div class="appr-actions">
                    {#if it.type === 'collection_item_request'}
                      <button class="archive-btn ok" onclick={() => act(buildUrl('approve', it, it.type))} disabled={busyId !== ''}>{t('approvals.approve')}</button>
                      <button class="archive-btn bad" onclick={() => act(buildUrl('reject', it, it.type))} disabled={busyId !== ''}>{t('approvals.reject')}</button>
                      <button class="vote-btn" disabled={busyId !== '' || it.my_vote === 1}
                              onclick={() => act(buildUrl('vote', it, it.type), { vote: 1 })}>{t('approvals.agree')}</button>
                      <button class="vote-btn" disabled={busyId !== ''}
                              onclick={() => act(buildUrl('vote', it, it.type), { vote: 0 })}>{t('approvals.abstain')}</button>
                      <button class="vote-btn" disabled={busyId !== '' || it.my_vote === -1}
                              onclick={() => act(buildUrl('vote', it, it.type), { vote: -1 })}>{t('approvals.disagree')}</button>
                    {:else if it.type === 'curator_fix_proposal'}
                      <button class="vote-btn" disabled={busyId !== '' || it.my_vote === 1}
                              onclick={() => act(`/api/curator/content/proposals/${it.id}/vote`, { vote: 1 })}>{t('approvals.agree')}</button>
                      <button class="vote-btn" disabled={busyId !== '' || it.my_vote === -1}
                              onclick={() => act(`/api/curator/content/proposals/${it.id}/vote`, { vote: -1 })}>{t('approvals.disagree')}</button>
                    {:else if it.type === 'metadata_proposal'}
                      <button class="vote-btn" disabled={busyId !== '' || it.my_vote === 1}
                              onclick={() => act(`/api/curator/metadata/proposals/${it.id}/vote`, { vote: 1 })}>{t('approvals.agree')}</button>
                      <button class="vote-btn" disabled={busyId !== '' || it.my_vote === -1}
                              onclick={() => act(`/api/curator/metadata/proposals/${it.id}/vote`, { vote: -1 })}>{t('approvals.disagree')}</button>
                    {:else if it.type === 'comment_triage'}
                      <button class="archive-btn ok" disabled={busyId !== ''}
                              onclick={() => act(`/api/admin/moderation/comments/${it.comment_id}/hide`)}>{t('approvals.hide')}</button>
                      <button class="archive-btn bad" disabled={busyId !== ''}
                              onclick={() => act(`/api/admin/moderation/comments/${it.comment_id}/delete`)}>{t('approvals.delete')}</button>
                    {:else if it.type === 'forum_edit_proposal'}
                      <button class="archive-btn ok" disabled={busyId !== ''}
                              onclick={() => act(`/api/forum/edits/${it.id}/review`, { decision: 'approve' })}>{t('approvals.approve')}</button>
                      <button class="archive-btn bad" disabled={busyId !== ''}
                              onclick={() => act(`/api/forum/edits/${it.id}/review`, { decision: 'reject' })}>{t('approvals.reject')}</button>
                    {/if}
                  </div>
                </article>
              {/each}
            </section>
          {/each}
        </div>
      {/if}
    </div>
  </main>
{:else}
  <div class="appr-page">
    <header class="page-head">
      <h1>{t('approvals.title')}</h1>
      <p class="subtitle">{t('approvals.subtitle')}</p>
      <div class="controls">
        <label>{t('approvals.filterType')}
          <select bind:value={typeFilter} onchange={load}>
            <option value="all">{t('approvals.allTypes')}</option>
            {#each typeOptions as k}
              <option value={k}>{groupTitle(k)}{counts[k] !== undefined ? ` (${counts[k]})` : ''}</option>
            {/each}
          </select>
        </label>
        <label>{t('approvals.filterStatus')}
          <select bind:value={statusFilter} onchange={load}>
            {#each statusOptions as s}
              <option value={s}>{t(`approvals.status.${s}`)}</option>
            {/each}
          </select>
        </label>
        <input type="text" placeholder={t('approvals.filterCurator')} bind:value={curatorFilter}
               onkeydown={(e) => e.key === 'Enter' && load()} />
        <button class="btn" onclick={load}>{t('approvals.refresh')}</button>
        <button class="btn ghost" onclick={clearFilters}>{t('approvals.clear')}</button>
      </div>
    </header>

    {#if error}
      <div class="error-card"><strong>{error}</strong></div>
    {/if}
    {#if loading}
      <p class="empty">{t('approvals.loading')}</p>
    {:else if groups.length === 0}
      <p class="empty">{t('approvals.empty')}</p>
    {:else}
      <div class="appr-groups">
        {#each groups as g (g.type)}
          <section class="appr-group">
            <h2 class="appr-group-title">{g.label}
              {#if counts[g.type] !== undefined}<span class="badge">{counts[g.type]}</span>{/if}
            </h2>
            {#each g.items as it (it.id)}
              <article class="appr-card">
                <div class="appr-summary">{summary(it)}</div>
                <div class="appr-meta">
                  {#if it.type === 'collection_item_request'}
                    {#if it.collection_title}<span class="note">→ {it.collection_title}</span>{/if}
                    {#if it.blurb}<span class="note">“{it.blurb}”</span>{/if}
                  {:else if it.type === 'comment_triage'}
                    {#if it.work_title}<span class="note">{it.work_title}</span>{/if}
                    <span class="note">{it.category}</span>
                  {:else if it.type === 'forum_edit_proposal'}
                    <span class="note">{it.target_type ?? ''} #{it.target_id ?? it.id}</span>
                  {/if}
                  {#if it.type === 'curator_fix_proposal' && it.reason}
                    <span class="note">{it.reason}</span>
                  {/if}
                </div>
                <div class="appr-row">
                  <span class="note">{t('approvals.proposer')} {it.proposer ?? '—'} · {fmtTime(it.created_at)}</span>
                  {#if it.votes_for !== undefined}
                    <span class="appr-votes">
                      <span class="vote-chip for">{t('approvals.agree')} {it.votes_for}</span>
                      <span class="vote-chip against">{t('approvals.disagree')} {it.votes_against}</span>
                      <span class="vote-chip mine">{t('approvals.myVote')}: {myVoteLabel(it.my_vote ?? 0)}</span>
                    </span>
                  {/if}
                </div>
                <div class="appr-actions">
                  {#if it.type === 'collection_item_request'}
                    <button class="btn ok" onclick={() => act(buildUrl('approve', it, it.type))} disabled={busyId !== ''}>{t('approvals.approve')}</button>
                    <button class="btn bad" onclick={() => act(buildUrl('reject', it, it.type))} disabled={busyId !== ''}>{t('approvals.reject')}</button>
                    <button class="btn vote" disabled={busyId !== '' || it.my_vote === 1}
                            onclick={() => act(buildUrl('vote', it, it.type), { vote: 1 })}>{t('approvals.agree')}</button>
                    <button class="btn vote" disabled={busyId !== ''}
                            onclick={() => act(buildUrl('vote', it, it.type), { vote: 0 })}>{t('approvals.abstain')}</button>
                    <button class="btn vote" disabled={busyId !== '' || it.my_vote === -1}
                            onclick={() => act(buildUrl('vote', it, it.type), { vote: -1 })}>{t('approvals.disagree')}</button>
                  {:else if it.type === 'curator_fix_proposal'}
                    <button class="btn vote" disabled={busyId !== '' || it.my_vote === 1}
                            onclick={() => act(`/api/curator/content/proposals/${it.id}/vote`, { vote: 1 })}>{t('approvals.agree')}</button>
                    <button class="btn vote" disabled={busyId !== '' || it.my_vote === -1}
                            onclick={() => act(`/api/curator/content/proposals/${it.id}/vote`, { vote: -1 })}>{t('approvals.disagree')}</button>
                  {:else if it.type === 'metadata_proposal'}
                    <button class="btn vote" disabled={busyId !== '' || it.my_vote === 1}
                            onclick={() => act(`/api/curator/metadata/proposals/${it.id}/vote`, { vote: 1 })}>{t('approvals.agree')}</button>
                    <button class="btn vote" disabled={busyId !== '' || it.my_vote === -1}
                            onclick={() => act(`/api/curator/metadata/proposals/${it.id}/vote`, { vote: -1 })}>{t('approvals.disagree')}</button>
                  {:else if it.type === 'comment_triage'}
                    <button class="btn ok" disabled={busyId !== ''}
                            onclick={() => act(`/api/admin/moderation/comments/${it.comment_id}/hide`)}>{t('approvals.hide')}</button>
                    <button class="btn bad" disabled={busyId !== ''}
                            onclick={() => act(`/api/admin/moderation/comments/${it.comment_id}/delete`)}>{t('approvals.delete')}</button>
                  {:else if it.type === 'forum_edit_proposal'}
                    <button class="btn ok" disabled={busyId !== ''}
                            onclick={() => act(`/api/forum/edits/${it.id}/review`, { decision: 'approve' })}>{t('approvals.approve')}</button>
                    <button class="btn bad" disabled={busyId !== ''}
                            onclick={() => act(`/api/forum/edits/${it.id}/review`, { decision: 'reject' })}>{t('approvals.reject')}</button>
                  {/if}
                </div>
              </article>
            {/each}
          </section>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .arc-ctl { display: flex; gap: 0.6rem; flex-wrap: wrap; margin-top: 0.75rem; align-items: center; }
  .arc-ctl label { display: flex; align-items: center; gap: 0.35rem; font-size: 0.85em; }
  .arc-count { display: inline-block; min-width: 1.4rem; text-align: center; border: 1px solid var(--archive-border, #dddddd); background: var(--archive-bg-raised, #f5f5f5); padding: 0 0.4em; font-size: 0.85em; }
  .arc-votes { display: inline-flex; gap: 0.5rem; flex-wrap: wrap; }
  .arc-vote { border: 1px solid var(--archive-border, #dddddd); padding: 0 0.4em; font-size: 0.82em; }
  .arc-vote.for, .vote-chip.for { color: #1a7a36; }
  .arc-vote.against, .vote-chip.against { color: #990000; }
  .arc-vote.mine, .vote-chip.mine { color: var(--archive-muted, #666666); }

  .controls { display: flex; gap: 0.6rem; flex-wrap: wrap; margin-top: 0.75rem; align-items: center; }
  .controls label { display: flex; align-items: center; gap: 0.35rem; }
  .appr-groups { margin-top: 1.25rem; display: flex; flex-direction: column; gap: 1.6rem; }
  .appr-group-title { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.6rem; font-size: 1.1rem; }
  .appr-card { border: 1px solid var(--color-border, #e5e7eb); border-radius: var(--radius-sm, 8px); padding: 0.9rem 1rem; margin-bottom: 0.7rem; background: var(--color-surface-1, #ffffff); }
  .appr-summary { font-weight: 600; }
  .appr-meta { display: flex; gap: 0.75rem; flex-wrap: wrap; margin-top: 0.4rem; }
  .note { color: var(--color-muted, #6b7280); font-size: 0.86rem; }
  .appr-row { display: flex; gap: 1rem; justify-content: space-between; flex-wrap: wrap; align-items: center; margin-top: 0.5rem; }
  .appr-votes { display: inline-flex; gap: 0.5rem; flex-wrap: wrap; }
  .vote-chip { border: 1px solid var(--color-border, #e5e7eb); border-radius: 999px; padding: 0.1rem 0.5rem; font-size: 0.78rem; }
  .appr-actions { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-top: 0.6rem; }
  .btn.ok, .archive-btn.ok { color: #1a7a36; border-color: #1a7a36; }
  .btn.bad, .archive-btn.bad { color: #990000; border-color: #990000; }
  .btn.vote, .vote-btn { all: unset; cursor: pointer; border: 1px solid var(--color-border, #e5e7eb); border-radius: 999px; padding: 0.25rem 0.7rem; font-size: 0.8rem; color: var(--color-muted, #6b7280); }
  .btn.vote:hover, .vote-btn:hover { border-color: var(--color-primary, #6366f1); color: var(--color-primary-hover, #818cf8); }
  .vote-btn { all: unset; cursor: pointer; border: 1px solid var(--archive-border, #dddddd); border-radius: 3px; padding: 0.25rem 0.7rem; font-size: 0.8rem; color: var(--archive-link, #990000); }
  .vote-btn:hover { background: var(--archive-bg-raised, #f5f5f5); }
  .btn.vote:disabled, .vote-btn:disabled { opacity: 0.45; cursor: default; }
  .empty { color: var(--color-muted, #6b7280); padding: 1.5rem; }
  .error-card { color: #990000; font-weight: 700; margin-top: 1rem; }
</style>
