<script lang="ts">
  import { onMount } from 'svelte';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  type Translation = {
    id: number;
    work_id: number;
    locale_code: string;
    title: string | null;
    summary: string | null;
    translated_by: number | null;
    translated_at: string;
    status: 'draft' | 'approved' | 'rejected';
    reviewed_by: number | null;
    reviewed_at: string | null;
  };

  // Svelte 5 runes mode: page state MUST use $state — plain `let` fields are
  // NOT reactive here, so fetched data never re-renders the page out of
  // "Loading…" (the bug this page shipped with).
  let items = $state<Translation[]>([]);
  let status: 'draft' | 'approved' | 'rejected' = $state('draft');
  let loading = $state(true);
  let error = $state('');
  let msg = $state('');
  let editingId = $state<number | null>(null);
  let editTitle = $state('');
  let editSummary = $state('');

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch(`/api/admin/translations?status=${status}&per_page=50`);
      const body = await res.json();
      if (body.err !== 0) throw new Error(body.msg ?? `HTTP ${res.status}`);
      items = body.items ?? [];
      msg = '';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function approve(t: Translation) {
    const res = await adminFetch(`/api/admin/translations/${t.id}/approve`, { method: 'POST' });
    const body = await res.json().catch(() => ({ err: -1, msg: `HTTP error` }));
    if (body.err === 0) { msg = `✓ Approved translation #${t.id}`; await load(); }
    else { error = body.msg ?? 'Approve failed'; }
  }

  async function reject(t: Translation) {
    const res = await adminFetch(`/api/admin/translations/${t.id}/reject`, { method: 'POST' });
    const body = await res.json().catch(() => ({ err: -1, msg: `HTTP error` }));
    if (body.err === 0) { msg = `✗ Rejected translation #${t.id}`; await load(); }
    else { error = body.msg ?? 'Reject failed'; }
  }

  function startEdit(t: Translation) {
    editingId = t.id;
    editTitle = t.title ?? '';
    editSummary = t.summary ?? '';
  }

  async function saveEdit(t: Translation) {
    const payload: Record<string, string> = {};
    if (editTitle !== (t.title ?? '')) payload.title = editTitle;
    if (editSummary !== (t.summary ?? '')) payload.summary = editSummary;
    if (Object.keys(payload).length === 0) { editingId = null; return; }
    const res = await adminFetch(`/api/admin/translations/${t.id}/edit`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    const body = await res.json().catch(() => ({ err: -1, msg: `HTTP error` }));
    if (body.err === 0) { msg = `✓ Saved translation #${t.id}`; editingId = null; await load(); }
    else { error = body.msg ?? 'Edit failed'; }
  }

  onMount(load);
</script>

<svelte:head><title>Translations — FicNexus Admin</title></svelte:head>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Translation Review</h1>
        <p class="archive-summary">Approve, reject, or edit ML-generated work translations. Approved translations go live immediately.</p>
      </header>

      <ul class="archive-tabs" role="tablist">
        {#each (['draft', 'approved', 'rejected'] as const) as s (s)}
          <li class:current={status === s} role="presentation">
            <button type="button" onclick={() => { status = s; load(); }}>{s}</button>
          </li>
        {/each}
      </ul>

      {#if error}<p class="archive-error" role="alert">{error}</p>{/if}
      {#if msg && !error}<p class="archive-msg">{msg}</p>{/if}

      {#if loading}
        <p class="archive-note">Loading…</p>
      {:else if items.length === 0}
        <p class="archive-note">No translations in this state.</p>
      {:else}
        {#each items as t (t.id)}
          <fieldset class="archive-fieldset trans-item">
            <legend class="archive-legend">
              {t.locale_code}
              · work #{t.work_id}
              · {t.status}
              · by {t.translated_by ?? 'system'} · {t.translated_at.slice(0, 10)}
            </legend>
            {#if editingId === t.id}
              <form onsubmit={(e) => { e.preventDefault(); saveEdit(t); }}>
                <dl class="archive-dl">
                  <dt><label for={`tr-title-${t.id}`}>Title</label></dt>
                  <dd><input id={`tr-title-${t.id}`} class="archive-input" type="text" placeholder="Title" bind:value={editTitle} /></dd>
                  <dt><label for={`tr-summary-${t.id}`}>Summary</label></dt>
                  <dd><textarea id={`tr-summary-${t.id}`} class="archive-textarea" placeholder="Summary" bind:value={editSummary} rows="3"></textarea></dd>
                </dl>
                <div class="archive-form-actions">
                  <button class="archive-btn archive-btn-primary" type="submit">Save</button>
                  {' '}
                  <button class="archive-btn" type="button" onclick={() => (editingId = null)}>Cancel</button>
                </div>
            </form>
            {:else}
              <dl class="archive-dl">
                <div class="dl-row"><dt>Title</dt><dd><strong>{t.title ?? '(no title)'}</strong></dd></div>
                {#if t.summary}
                  <div class="dl-row"><dt>Summary</dt><dd>{t.summary}</dd></div>
                {/if}
              </dl>
              <div class="archive-form-actions">
                <button class="archive-btn" type="button" onclick={() => approve(t)}>Approve</button>
                {' '}
                <button class="archive-btn archive-btn-danger" type="button" onclick={() => reject(t)}>Reject</button>
                {' '}
                <button class="archive-btn" type="button" onclick={() => startEdit(t)}>Edit</button>
              </div>
            {/if}
          </fieldset>
        {/each}
      {/if}
    </div>
  </main>
{:else}
<div class="admin-page">
  <h1>Translation Review</h1>
  <p class="muted">Approve, reject, or edit ML-generated work translations. Approved translations go live immediately.</p>

  <div class="tabs">
    {#each (['draft', 'approved', 'rejected'] as const) as s (s)}
      <button class="tab" class:active={status === s} onclick={() => { status = s; load(); }}>{s}</button>
    {/each}
  </div>

  {#if error}<div class="error-card"><strong>{error}</strong></div>{/if}
  {#if msg && !error}<p class="ok">{msg}</p>{/if}

  {#if loading}
    <p class="muted"><span class="spinner"></span> Loading…</p>
  {:else if items.length === 0}
    <p class="empty">No translations in this state.</p>
  {:else}
    <ul class="trans-list">
      {#each items as t (t.id)}
        <li class="card trans-card">
          <div class="head">
            <span class="badge">{t.locale_code}</span>
            <span class="work-id">work #{t.work_id}</span>
            <span class="status" class:draft={t.status === 'draft'} class:approved={t.status === 'approved'} class:rejected={t.status === 'rejected'}>{t.status}</span>
            <span class="muted small">by {t.translated_by ?? 'system'} · {t.translated_at.slice(0, 10)}</span>
          </div>
          {#if editingId === t.id}
            <div class="edit-form">
              <input type="text" placeholder="Title" bind:value={editTitle} />
              <textarea placeholder="Summary" bind:value={editSummary} rows="3"></textarea>
              <div class="actions">
                <button class="btn btn-small" onclick={() => saveEdit(t)}>Save</button>
                <button class="btn btn-small" onclick={() => (editingId = null)}>Cancel</button>
              </div>
            </div>
          {:else}
            <div class="content">
              <p class="title"><strong>{t.title ?? '(no title)'}</strong></p>
              {#if t.summary}<p class="summary">{t.summary}</p>{/if}
            </div>
            <div class="actions">
              <button class="btn btn-small" onclick={() => approve(t)}>Approve</button>
              <button class="btn btn-small" onclick={() => reject(t)}>Reject</button>
              <button class="btn btn-small" onclick={() => startEdit(t)}>Edit</button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>
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
  .archive-content { max-width: 860px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
  .archive-tabs {
    display: flex;
    list-style: none;
    margin: 1.2rem 0 0;
    padding: 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    gap: 0;
  }
  .archive-tabs li { margin: 0; margin-bottom: -1px; }
  .archive-tabs li button {
    display: block;
    padding: 0.45rem 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.9em;
    font-weight: 600;
    color: var(--archive-muted, #666666);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-bottom: 1px solid var(--archive-border, #dddddd);
    cursor: pointer;
    line-height: 1.4;
  }
  .archive-tabs li button:hover {
    color: var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-tabs li.current button {
    color: var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
    border-color: var(--archive-border, #dddddd);
    border-bottom-color: var(--archive-bg, #ffffff);
  }
  .archive-fieldset.trans-item {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 1em 1.25em;
    margin: 1.2rem 0;
  }
  .archive-legend { font-weight: 700; font-size: 0.98em; color: var(--archive-text, #2a2a2a); padding: 0 0.4em; }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.88em; flex: 0 0 6em; margin: 0.4em 0; }
  .archive-dl .dl-row dd { margin: 0.4em 0; min-width: 0; flex: 1; }
  .archive-dl dt { font-weight: 700; font-size: 0.9em; margin: 0.85em 0 0.2em; }
  .archive-dl dt:first-child { margin-top: 0; }
  .archive-dl dd { margin: 0; }
  .archive-input,
  .archive-textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
  }
  .archive-textarea { resize: vertical; }
  .archive-form-actions { margin-top: 0.75em; }
  .archive-btn {
    display: inline-block;
    padding: 0.3em 1em;
    font-size: 0.85em;
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
  .archive-btn-danger { color: var(--archive-link, #990000); border-color: var(--archive-link, #990000); }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-msg { color: #2e7d32; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.9em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .admin-page { max-width: 860px; margin: 0 auto; padding: 1.5rem; }
  .tabs { display: flex; gap: 0.25rem; margin: 1rem 0; }
  .tab { padding: 0.4rem 0.9rem; border-radius: 999px; border: 1px solid var(--color-border, #ddd); background: transparent; cursor: pointer; }
  .tab.active { background: var(--color-accent, #4a6cf7); color: #fff; border-color: var(--color-accent, #4a6cf7); }
  .trans-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .trans-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .head { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .badge { background: var(--color-accent-bg, #eef1ff); color: var(--color-accent, #4a6cf7); padding: 0.1rem 0.5rem; border-radius: 999px; font-size: 0.8rem; font-weight: 700; }
  .status { padding: 0.1rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .status.draft { background: #fef3c7; color: #92400e; }
  .status.approved { background: #d1fae5; color: #065f46; }
  .status.rejected { background: #fee2e2; color: #991b1b; }
  .small { font-size: 0.8rem; }
  .title { margin: 0; }
  .summary { margin: 0.25rem 0 0; color: var(--color-text-muted, #555); }
  .actions { display: flex; gap: 0.5rem; }
  .edit-form { display: flex; flex-direction: column; gap: 0.5rem; }
  .edit-form input, .edit-form textarea { padding: 0.4rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; }
  .muted { color: var(--color-text-muted, #888); }
  .error-card { background: #fee2e2; color: #991b1b; padding: 0.75rem; border-radius: 8px; margin: 1rem 0; }
  .ok { color: var(--color-ok, #2e9e5b); }
  .empty { color: var(--color-text-muted, #888); }
</style>