<script lang="ts">
  import { onMount } from 'svelte';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let workId = $state('');
  // Svelte 5 runes mode ($derived above forces it): page state MUST use
  // $state — plain `let` fields are NOT reactive, so fetched data never
  // re-rendered the edit form out of the search UI (same bug the
  // translations page shipped with).
  let current = $state<{ id: number; title: string; author: string; status: string; description: string } | null>(null);
  let title = $state('');
  let author = $state('');
  let status = $state('');
  let description = $state('');
  let searching = $state(false);
  let loading = $state(false);
  let error = $state('');
  let msg = $state('');

  async function searchWork() {
    if (!workId.trim()) return;
    searching = true;
    error = '';
    msg = '';
    try {
      // Fetch the work via the public works endpoint (admin sees all)
      const res = await fetch(`/api/works/${encodeURIComponent(workId.trim())}`, { credentials: 'include' });
      const body = await res.json();
      if (body.err === 0 && body.work) {
        current = {
          id: body.work.id,
          title: body.work.title ?? '',
          author: body.work.author ?? '',
          status: body.work.status ?? '',
          description: body.work.description ?? '',
        };
        title = current.title;
        author = current.author;
        status = current.status;
        description = current.description;
      } else {
        // fall back to search
        const sres = await fetch(`/api/search?q=${encodeURIComponent(workId.trim())}`, { credentials: 'include' });
        const sbody = await sres.json();
        const first = sbody.items?.[0];
        if (first) {
          current = { id: first.id ?? first.work_id, title: first.title, author: first.author, status: '', description: '' };
          title = current.title;
          author = current.author;
          description = current.description;
          msg = 'Loaded from search (first result).';
        } else {
          error = 'Work not found by that id or query.';
          current = null;
        }
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      searching = false;
    }
  }

  async function save() {
    if (!current) return;
    loading = true;
    error = '';
    msg = '';
    const payload: Record<string, string> = {};
    if (title.trim() !== current.title) payload.title = title.trim();
    if (author.trim() !== current.author) payload.author = author.trim();
    if (status.trim() !== current.status) payload.status = status.trim();
    if (description.trim() !== current.description) payload.description = description.trim();
    if (Object.keys(payload).length === 0) {
      msg = 'No changes to save.';
      loading = false;
      return;
    }
    try {
      const res = await adminFetch(`/api/admin/works/${current.id}/metadata`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const body = await res.json();
      if (body.err === 0) { msg = '✓ Metadata corrected (canonical + default source synced).'; await searchWork(); }
      else { error = body.msg ?? 'Save failed'; }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => { /* no auto-load */ });
</script>

<svelte:head><title>Metadata Correction — FicNexus Admin</title></svelte:head>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Metadata Correction</h1>
        <p class="archive-summary">
          Correct the canonical metadata of a work. Changes sync to the default source so
          public pages reflect the fix immediately. This is logged to the modlog.
        </p>
      </header>

      {#if error}<p class="archive-error" role="alert">{error}</p>{/if}
      {#if msg && !error}<p class="archive-msg">{msg}</p>{/if}

      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Find a work</legend>
        <form onsubmit={(e) => { e.preventDefault(); searchWork(); }}>
          <dl class="archive-dl">
            <dt><label for="md-search">Work id or search query</label></dt>
            <dd>
              <input id="md-search" class="archive-input" type="text" placeholder="work id or search query" bind:value={workId} />
            </dd>
          </dl>
          <div class="archive-form-actions">
            <button class="archive-btn archive-btn-primary" type="submit" disabled={searching}>{searching ? 'Searching…' : '🔍 Load work'}</button>
          </div>
        </form>
      </fieldset>

      {#if current}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Edit metadata <span class="archive-note">(Work #{current.id})</span></legend>
          <form onsubmit={(e) => { e.preventDefault(); save(); }}>
            <dl class="archive-dl">
              <dt><label for="amd-title">Title</label></dt>
              <dd><input id="amd-title" class="archive-input" type="text" bind:value={title} /></dd>
              <dt><label for="amd-author">Author</label></dt>
              <dd><input id="amd-author" class="archive-input" type="text" bind:value={author} /></dd>
              <dt><label for="amd-status">Status</label></dt>
              <dd><input id="amd-status" class="archive-input" type="text" bind:value={status} placeholder="ongoing / completed / abandoned" /></dd>
              <dt><label for="amd-desc">Description</label></dt>
              <dd><textarea id="amd-desc" class="archive-textarea" bind:value={description} rows="5"></textarea></dd>
            </dl>
            <div class="archive-form-actions">
              <button class="archive-btn archive-btn-primary" type="submit" disabled={loading}>{loading ? 'Saving…' : '💾 Save correction'}</button>
            </div>
          </form>
        </fieldset>
      {:else if !searching}
        <p class="archive-note">Enter a work id (e.g. 42) or a search query to load a work's current metadata.</p>
      {/if}
    </div>
  </main>
{:else}
<div class="admin-page">
  <h1>✏️ Metadata Correction</h1>
  <p class="muted">Correct the canonical metadata of a work. Changes sync to the default source so public pages reflect the fix immediately. This is logged to the modlog.</p>

  {#if error}<div class="error-card"><strong>⚠️ {error}</strong></div>{/if}
  {#if msg && !error}<p class="ok">{msg}</p>{/if}

  <div class="search-row">
    <input type="text" placeholder="work id or search query" bind:value={workId} onkeydown={(e) => { if (e.key === 'Enter') searchWork(); }} />
    <button class="btn" onclick={searchWork} disabled={searching}>{searching ? 'Searching…' : '🔍 Load work'}</button>
  </div>

  {#if current}
    <div class="card form">
      <p class="work-id">Work #{current.id}</p>
      <label for="md-title">Title</label>
      <input id="md-title" type="text" bind:value={title} />
      <label for="md-author">Author</label>
      <input id="md-author" type="text" bind:value={author} />
      <label for="md-status">Status</label>
      <input id="md-status" type="text" bind:value={status} placeholder="ongoing / completed / abandoned" />
      <label for="md-desc">Description</label>
      <textarea id="md-desc" bind:value={description} rows="5"></textarea>
      <div class="actions">
        <button class="btn btn-primary" onclick={save} disabled={loading}>{loading ? 'Saving…' : '💾 Save correction'}</button>
      </div>
    </div>
  {:else if !searching}
    <p class="muted">Enter a work id (e.g. 42) or a search query to load a work's current metadata.</p>
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
  .archive-content { max-width: 760px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
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
  .archive-textarea { resize: vertical; min-height: 5em; }
  .archive-form-actions { margin-top: 0.75em; }
  .archive-btn {
    display: inline-block;
    padding: 0.35em 1.1em;
    font-size: 0.88em;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    cursor: pointer;
    font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-btn-primary {
    background: var(--archive-link, #990000);
    color: #ffffff;
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover:not(:disabled) { background: #7a0000; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-msg { color: #2e7d32; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.85em; font-weight: 400; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .admin-page { max-width: 720px; margin: 0 auto; padding: 1.5rem; }
  .search-row { display: flex; gap: 0.5rem; margin: 1rem 0; }
  .search-row input { flex: 1; padding: 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; }
  .form { display: flex; flex-direction: column; gap: 0.4rem; padding: 1.25rem; }
  .form label { font-weight: 700; margin-top: 0.5rem; font-size: 0.85rem; }
  .form input, .form textarea { padding: 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; width: 100%; box-sizing: border-box; }
  .work-id { color: var(--color-text-muted, #888); font-size: 0.85rem; margin: 0; }
  .actions { margin-top: 0.75rem; display: flex; gap: 0.5rem; }
  .muted { color: var(--color-text-muted, #888); }
  .error-card { background: #fee2e2; color: #991b1b; padding: 0.75rem; border-radius: 8px; margin: 1rem 0; }
  .ok { color: var(--color-ok, #2e9e5b); }
</style>
