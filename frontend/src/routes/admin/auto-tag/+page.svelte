<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface QueueRow {
    url_id: string;
    title: string;
    tag_id: number;
    tag_name: string;
    score: number;
    tag_type_id: number;
    created: string | null;
  }

  let items = $state<QueueRow[]>([]);
  let total = $state(0);
  let loading = $state(true);
  let error = $state('');
  let urlId = $state('');
  let running = $state(false);
  let runMsg = $state('');
  let backfilling = $state(false);
  let backfillMsg = $state('');
  // Per-row review state: inline feedback instead of alert().
  let actingKey = $state('');
  let actionMsg = $state('');
  let actionErr = $state('');

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch('/api/admin/auto-tag/queue');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      items = body.items ?? [];
      total = body.total ?? 0;
    } catch (e) {
      error = `Failed to load queue: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  onMount(load);

  async function runAutoTag() {
    const id = urlId.trim();
    if (!id) return;
    running = true;
    runMsg = '';
    try {
      const res = await adminFetch('/api/admin/auto-tag', {
        method: 'POST',
        credentials: 'include',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ url_id: id }),
      });
      const body = await res.json();
      if (!res.ok || body.err !== 0) {
        runMsg = `⚠️ ${body.msg ?? `HTTP ${res.status}`}`;
      } else {
        const n = (body.suggested ?? []).length;
        runMsg = `✅ Suggested ${n} tag(s) for ${id}`;
        urlId = '';
        await load();
      }
    } catch (e) {
      runMsg = `⚠️ ${e instanceof Error ? e.message : e}`;
    } finally {
      running = false;
    }
  }

  async function backfill() {
    backfilling = true;
    backfillMsg = '';
    try {
      const res = await adminFetch('/api/admin/auto-tag/backfill', {
        method: 'POST',
        credentials: 'include',
      });
      const body = await res.json();
      if (!res.ok || body.err !== 0) {
        backfillMsg = `⚠️ ${body.msg ?? `HTTP ${res.status}`}`;
      } else {
        backfillMsg = `✅ ${body.msg}`;
      }
    } catch (e) {
      backfillMsg = `⚠️ ${e instanceof Error ? e.message : e}`;
    } finally {
      backfilling = false;
    }
  }

  async function review(url_id: string, tag_id: number, action: 'approve' | 'dismiss') {
    const key = `${url_id}:${tag_id}`;
    actingKey = key;
    actionMsg = '';
    actionErr = '';
    try {
      const res = await adminFetch(`/api/admin/auto-tag/${action}/${url_id}/${tag_id}`, {
        method: 'POST',
        credentials: 'include',
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok || body.err !== 0) {
        actionErr = `${action} failed: ${body.msg ?? `HTTP ${res.status}`}`;
        return;
      }
      // Both approve + dismiss clear the suggestion from the pending queue
      // server-side, so remove the row locally instead of refetching.
      items = items.filter((it) => !(it.url_id === url_id && it.tag_id === tag_id));
      total = Math.max(0, total - 1);
      actionMsg = `${action === 'approve' ? '✓ Approved' : '✗ Dismissed'} tag #${tag_id} on ${url_id}`;
    } catch (e) {
      actionErr = `${action} failed: ${e instanceof Error ? e.message : e}`;
    } finally {
      actingKey = '';
    }
  }

  function simPct(score: number): string {
    return (score / 100).toFixed(2);
  }
</script>

<svelte:head><title>Auto-Tag Queue | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Auto-Tag</h1>
        <p class="archive-summary">Zero-shot tag classification: embed a fic's description + title (nomic-embed-text, 768-d), compare against embedded canonical freeform tags, and queue suggestions above the similarity threshold for review.</p>
      </header>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Run on a fic</legend>
        <form onsubmit={(e) => { e.preventDefault(); runAutoTag(); }}>
          <dl class="archive-dl">
            <dt><label for="at-url">url_id</label></dt>
            <dd><input id="at-url" class="archive-input" type="text" placeholder="url_id (e.g. https://archiveofourown.org/works/1234567)" bind:value={urlId} /></dd>
          </dl>
          <div class="archive-form-actions">
            <button class="archive-btn archive-btn-primary" type="submit" disabled={running || !urlId.trim()}>{running ? 'Running…' : 'Auto-tag'}</button>
          </div>
        </form>
        {#if runMsg}<p class="archive-msg">{runMsg}</p>{/if}
      </fieldset>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Tag embedding cache</legend>
        <div class="archive-form-actions">
          <button class="archive-btn" type="button" disabled={backfilling} onclick={backfill}>{backfilling ? 'Backfilling…' : 'Backfill tag embeddings'}</button>
        </div>
        <p class="archive-note">Embeds canonical freeform tags without an embedding yet (idempotent).</p>
        {#if backfillMsg}<p class="archive-msg">{backfillMsg}</p>{/if}
      </fieldset>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Review queue <span class="archive-note">({total} pending)</span></legend>
        {#if actionMsg}<p class="archive-msg archive-ok">{actionMsg}</p>{/if}
        {#if actionErr}<p class="archive-msg archive-err">{actionErr}</p>{/if}
        {#if loading}
          <p class="archive-note">Loading…</p>
        {:else if error}
          <p class="archive-error" role="alert">{error}</p>
        {:else if items.length === 0}
          <p class="archive-note">No machine-suggested tags pending review.</p>
        {:else}
          <table class="archive-table">
            <thead><tr><th>Fic</th><th>Suggested tag</th><th>Sim</th><th>Suggested</th><th></th></tr></thead>
            <tbody>
              {#each items as it (it.url_id + it.tag_id)}
                {@const key = `${it.url_id}:${it.tag_id}`}
                {@const acting = actingKey === key}
                <tr class:dimmed={acting}>
                  <td><span class="title">{it.title || it.url_id}</span><span class="archive-mono url">{it.url_id}</span></td>
                  <td class="archive-mono">{it.tag_name}</td>
                  <td>{simPct(it.score)}</td>
                  <td class="archive-note">{it.created ? new Date(it.created).toLocaleString() : '—'}</td>
                  <td>
                    <button class="archive-btn" type="button" disabled={actingKey !== '' && !acting} onclick={() => review(it.url_id, it.tag_id, 'approve')}>{acting ? '…' : '✓ Approve'}</button>
                    {' '}
                    <button class="archive-btn" type="button" disabled={actingKey !== '' && !acting} onclick={() => review(it.url_id, it.tag_id, 'dismiss')}>{acting ? '…' : '✗ Dismiss'}</button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </fieldset>
    </div>
  </main>
{:else}
<h1>🤖 Auto-Tag</h1>
<p class="subtitle">
  Zero-shot tag classification: embed a fic's description + title (nomic-embed-text, 768-d),
  compare against embedded canonical freeform tags, and queue suggestions above the
  similarity threshold for review.
</p>

<section class="panel run">
  <h2>Run on a fic</h2>
  <div class="row">
    <input
      type="text"
      placeholder="url_id (e.g. https://archiveofourown.org/works/1234567)"
      bind:value={urlId}
      onkeydown={(e) => { if (e.key === 'Enter') runAutoTag(); }}
    />
    <button disabled={running || !urlId.trim()} onclick={runAutoTag}>
      {running ? 'Running…' : 'Auto-tag'}
    </button>
  </div>
  {#if runMsg}<p class="msg">{runMsg}</p>{/if}
</section>

<section class="panel">
  <h2>Tag embedding cache</h2>
  <div class="row">
    <button disabled={backfilling} onclick={backfill}>
      {backfilling ? 'Backfilling…' : 'Backfill tag embeddings'}
    </button>
    <span class="hint">Embeds canonical freeform tags without an embedding yet (idempotent).</span>
  </div>
  {#if backfillMsg}<p class="msg">{backfillMsg}</p>{/if}
</section>

<section class="panel">
  <h2>Review queue <span class="count">{total} pending</span></h2>
  {#if actionMsg}<p class="msg ok">{actionMsg}</p>{/if}
  {#if actionErr}<p class="msg err">{actionErr}</p>{/if}
  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if items.length === 0}
    <p class="empty">No machine-suggested tags pending review.</p>
  {:else}
    <table>
      <thead>
        <tr><th>Fic</th><th>Suggested tag</th><th>Sim</th><th>Suggested</th><th></th></tr>
      </thead>
      <tbody>
        {#each items as it (it.url_id + it.tag_id)}
          {@const key = `${it.url_id}:${it.tag_id}`}
          {@const acting = actingKey === key}
          <tr class:dimmed={acting}>
            <td>
              <span class="title">{it.title || it.url_id}</span>
              <span class="mono url">{it.url_id}</span>
            </td>
            <td class="mono">{it.tag_name}</td>
            <td>{simPct(it.score)}</td>
            <td class="tiny">{it.created ? new Date(it.created).toLocaleString() : '—'}</td>
            <td class="actions">
              <button class="approve" disabled={actingKey !== '' && !acting} onclick={() => review(it.url_id, it.tag_id, 'approve')}>
                {acting ? '…' : '✓ Approve'}
              </button>
              <button class="dismiss" disabled={actingKey !== '' && !acting} onclick={() => review(it.url_id, it.tag_id, 'dismiss')}>
                {acting ? '…' : '✗ Dismiss'}
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>
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
  .archive-fieldset { border: 1px solid var(--archive-border, #dddddd); padding: 1em 1.25em; margin: 1.2rem 0; }
  .archive-legend { font-weight: 700; font-size: 1.05em; padding: 0 0.4em; }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt { font-weight: 700; font-size: 0.9em; margin: 0.85em 0 0.2em; }
  .archive-dl dt:first-child { margin-top: 0; }
  .archive-dl dd { margin: 0; }
  .archive-input {
    width: 100%; box-sizing: border-box; padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd); font-family: inherit;
    font-size: 0.95em; background: var(--archive-bg, #ffffff);
  }
  .archive-form-actions { margin-top: 0.75em; }
  .archive-btn {
    display: inline-block; padding: 0.3em 1em; font-size: 0.88em; font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd); background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a); cursor: pointer; font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-btn-primary { background: var(--archive-link, #990000); color: #ffffff; border-color: var(--archive-link, #990000); }
  .archive-btn-primary:hover:not(:disabled) { background: #7a0000; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td { text-align: left; padding: 0.45em 0.6em; border-bottom: 1px solid var(--archive-border, #dddddd); vertical-align: top; }
  .archive-table th { font-weight: 700; font-size: 0.85em; border-bottom: 2px solid var(--archive-border, #dddddd); }
  tr.dimmed { opacity: 0.55; }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; }
  .archive-table .title { font-weight: 700; display: block; }
  .archive-table .url { font-size: 0.78em; }
  .archive-msg { font-size: 0.92em; }
  .archive-ok { color: #2e7d32; }
  .archive-err { color: #990000; font-weight: 700; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.85em; font-weight: 400; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 70ch; }
  .panel { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 1rem; margin: 1rem 0; }
  .panel h2 { font-size: 1rem; margin-bottom: 0.6rem; }
  .row { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
  .row input { flex: 1; min-width: 280px; padding: 0.45rem 0.6rem; border-radius: 8px; border: 1px solid var(--color-border, #ccc); }
  .hint { color: var(--color-text-muted, #888); font-size: 0.8rem; }
  .msg { margin-top: 0.5rem; font-size: 0.9rem; }
  .msg.ok { color: var(--color-success, #2a6); }
  .msg.err { color: var(--color-danger, #d33); }
  .count { font-size: 0.75rem; color: var(--color-text-muted, #888); font-weight: normal; }
  table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
  th, td { text-align: left; padding: 0.45rem 0.5rem; border-bottom: 1px solid var(--color-border, #eee); }
  tr.dimmed { opacity: 0.55; }
  .mono { font-family: monospace; }
  .title { font-weight: 600; display: block; }
  .url { font-size: 0.72rem; color: var(--color-text-muted, #888); }
  .tiny { font-size: 0.75rem; color: var(--color-text-muted, #888); }
  .actions { white-space: nowrap; text-align: right; }
  button { padding: 0.3rem 0.7rem; border-radius: 8px; border: 1px solid var(--color-border, #ccc); cursor: pointer; }
  button:disabled { opacity: 0.6; cursor: default; }
  .approve { background: color-mix(in srgb, var(--color-primary, #38c) 15%, transparent); color: var(--color-primary, #38c); }
  .dismiss { background: color-mix(in srgb, var(--color-danger, #d33) 12%, transparent); color: var(--color-danger, #d33); }
  .empty { padding: 1.2rem; text-align: center; color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: 8px; }
</style>
