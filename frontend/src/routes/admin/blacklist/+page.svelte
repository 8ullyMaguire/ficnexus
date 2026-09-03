<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface FicEntry {
    url_id: string;
    reason: number;
    created: string;
  }

  interface AuthorEntry {
    source_id: number;
    author_id: number;
    reason: number;
    created: string;
  }

  // Reason codes enforced by the backend (see src/routes/export.rs):
  // 5/7/8 = hard block (metadata may render, download is refused),
  // 6 = greylist (metadata renders, download suppressed).
  const REASON_LABELS: Record<number, string> = {
    5: '5 · hard block',
    6: '6 · greylist',
    7: '7 · hard block',
    8: '8 · hard block',
  };

  // Source platform IDs (see src/scrape/compat_fichub_net.rs).
  const SOURCE_LABELS: Record<number, string> = {
    1: 'ao3',
    2: 'ffn',
    3: 'fnac',
    4: 'sb/sv',
    5: 'wattpad',
    6: 'royalroad',
    99: 'unknown',
  };

  let fics = $state<FicEntry[]>([]);
  let authors = $state<AuthorEntry[]>([]);
  let loading = $state(true);
  let error = $state('');

  // Add-fic form
  let ficUrlId = $state('');
  let ficReason = $state(5);
  let addingFic = $state(false);
  let ficMsg = $state('');

  // Add-author form
  let authorSourceId = $state(1);
  let authorId = $state('');
  let authorReason = $state(5);
  let addingAuthor = $state(false);
  let authorMsg = $state('');

  function numReason(v: string | number): number {
    return typeof v === 'number' ? v : Number(v);
  }

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch('/api/admin/blacklist');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      fics = body.fics ?? [];
      authors = body.authors ?? [];
    } catch (e) {
      error = `Failed to load blacklist: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  onMount(load);

  async function addFic() {
    const id = ficUrlId.trim();
    if (!id) return;
    addingFic = true;
    ficMsg = '';
    try {
      const res = await adminFetch('/api/admin/blacklist/fic', {
        method: 'POST',
        credentials: 'include',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ url_id: id, reason: numReason(ficReason) }),
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok || body.err !== 0) {
        ficMsg = `⚠️ ${body.msg ?? `HTTP ${res.status}`}`;
        return;
      }
      ficUrlId = '';
      ficMsg = `✅ Blacklisted ${id} (reason ${ficReason})`;
      await load();
    } catch (e) {
      ficMsg = `⚠️ ${e instanceof Error ? e.message : e}`;
    } finally {
      addingFic = false;
    }
  }

  async function addAuthor() {
    const aid = Number(authorId.trim());
    if (!authorId.trim() || !Number.isFinite(aid) || aid <= 0) {
      authorMsg = '⚠️ Author ID must be a positive integer.';
      return;
    }
    addingAuthor = true;
    authorMsg = '';
    try {
      const res = await adminFetch('/api/admin/blacklist/author', {
        method: 'POST',
        credentials: 'include',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ source_id: numReason(authorSourceId), author_id: aid, reason: numReason(authorReason) }),
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok || body.err !== 0) {
        authorMsg = `⚠️ ${body.msg ?? `HTTP ${res.status}`}`;
        return;
      }
      authorId = '';
      authorMsg = `✅ Blacklisted author ${SOURCE_LABELS[authorSourceId] ?? authorSourceId}/${aid} (reason ${authorReason})`;
      await load();
    } catch (e) {
      authorMsg = `⚠️ ${e instanceof Error ? e.message : e}`;
    } finally {
      addingAuthor = false;
    }
  }

  function reasonLabel(r: number): string {
    return REASON_LABELS[r] ?? `${r} · custom`;
  }

  function sourceLabel(s: number): string {
    return SOURCE_LABELS[s] ?? String(s);
  }

  function fmtDate(iso: string): string {
    if (!iso) return '—';
    const d = new Date(iso);
    return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
  }
</script>

<svelte:head><title>Blacklist | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Blacklist</h1>
        <p class="archive-summary">Fics and authors refused download/export service. Reason codes: <strong>5/7/8 = hard block</strong> (download refused), <strong>6 = greylist</strong> (metadata renders, download suppressed). Any reason blocks an author. Source IDs: 1 = ao3, 2 = ffn, 3 = fnac, 4 = sb/sv, 5 = wattpad, 6 = royalroad, 99 = unknown.</p>
      </header>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Add a fic</legend>
        <form onsubmit={(e) => { e.preventDefault(); addFic(); }}>
          <dl class="archive-dl">
            <div class="dl-row"><dt><label for="bl-fic-url">url_id</label></dt><dd><input id="bl-fic-url" class="archive-input" type="text" placeholder="url_id (e.g. https://archiveofourown.org/works/1234567)" bind:value={ficUrlId} /></dd></div>
            <div class="dl-row"><dt><label for="bl-fic-reason">Reason</label></dt>
              <dd>
                <select id="bl-fic-reason" class="archive-select" bind:value={ficReason} aria-label="Fic blacklist reason">
                  <option value="5">5 · hard block</option>
                  <option value="6">6 · greylist</option>
                  <option value="7">7 · hard block</option>
                  <option value="8">8 · hard block</option>
                </select>
              </dd>
            </div>
          </dl>
          <div class="archive-form-actions">
            <button class="archive-btn archive-btn-primary" type="submit" disabled={addingFic || !ficUrlId.trim()}>{addingFic ? 'Adding…' : 'Blacklist fic'}</button>
          </div>
        </form>
        {#if ficMsg}<p class="archive-msg">{ficMsg}</p>{/if}
      </fieldset>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Add an author</legend>
        <form onsubmit={(e) => { e.preventDefault(); addAuthor(); }}>
          <dl class="archive-dl">
            <div class="dl-row"><dt><label for="bl-author-source">Source platform</label></dt>
              <dd>
                <select id="bl-author-source" class="archive-select" bind:value={authorSourceId} aria-label="Author source platform">
                  {#each Object.entries(SOURCE_LABELS) as [id, label]}
                    <option value={id}>{id} · {label}</option>
                  {/each}
                </select>
              </dd>
            </div>
            <div class="dl-row"><dt><label for="bl-author-id">author_id</label></dt><dd><input id="bl-author-id" class="archive-input" type="text" inputmode="numeric" placeholder="author_id (numeric ID on that source)" bind:value={authorId} /></dd></div>
            <div class="dl-row"><dt><label for="bl-author-reason">Reason</label></dt>
              <dd>
                <select id="bl-author-reason" class="archive-select" bind:value={authorReason} aria-label="Author blacklist reason">
                  <option value="5">5 · hard block</option>
                  <option value="6">6 · greylist</option>
                  <option value="7">7 · hard block</option>
                  <option value="8">8 · hard block</option>
                </select>
              </dd>
            </div>
          </dl>
          <div class="archive-form-actions">
            <button class="archive-btn archive-btn-primary" type="submit" disabled={addingAuthor || !authorId.trim()}>{addingAuthor ? 'Adding…' : 'Blacklist author'}</button>
          </div>
        </form>
        {#if authorMsg}<p class="archive-msg">{authorMsg}</p>{/if}
      </fieldset>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Fic blacklist <span class="archive-note">({fics.length})</span></legend>
        {#if loading}
          <p class="archive-note">Loading…</p>
        {:else if error}
          <p class="archive-error" role="alert">{error}</p>
        {:else if fics.length === 0}
          <p class="archive-note">No blacklisted fics.</p>
        {:else}
          <table class="archive-table">
            <thead><tr><th>Fic</th><th>Reason</th><th>Added</th></tr></thead>
            <tbody>
              {#each fics as it (it.url_id + it.reason)}
                <tr><td class="archive-mono">{it.url_id}</td><td>{reasonLabel(it.reason)}</td><td class="archive-note">{fmtDate(it.created)}</td></tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </fieldset>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Author blacklist <span class="archive-note">({authors.length})</span></legend>
        {#if loading}
          <p class="archive-note">Loading…</p>
        {:else if error}
          <p class="archive-error" role="alert">{error}</p>
        {:else if authors.length === 0}
          <p class="archive-note">No blacklisted authors.</p>
        {:else}
          <table class="archive-table">
            <thead><tr><th>Author</th><th>Reason</th><th>Added</th></tr></thead>
            <tbody>
              {#each authors as it (`${it.source_id}/${it.author_id}/${it.reason}`)}
                <tr><td class="archive-mono">{sourceLabel(it.source_id)}/{it.author_id}</td><td>{reasonLabel(it.reason)}</td><td class="archive-note">{fmtDate(it.created)}</td></tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </fieldset>
    </div>
  </main>
{:else}
<h1>🚫 Blacklist</h1>
<p class="subtitle">
  Fics and authors refused download/export service. Reason codes: <strong>5/7/8 = hard block</strong>
  (download refused), <strong>6 = greylist</strong> (metadata renders, download suppressed). Any reason
  blocks an author. Source IDs: 1 = ao3, 2 = ffn, 3 = fnac, 4 = sb/sv, 5 = wattpad, 6 = royalroad,
  99 = unknown.
</p>

<section class="panel">
  <h2>Add a fic</h2>
  <div class="row">
    <input
      type="text"
      placeholder="url_id (e.g. https://archiveofourown.org/works/1234567)"
      bind:value={ficUrlId}
      onkeydown={(e) => { if (e.key === 'Enter') addFic(); }}
    />
    <select bind:value={ficReason} aria-label="Fic blacklist reason">
      <option value="5">5 · hard block</option>
      <option value="6">6 · greylist</option>
      <option value="7">7 · hard block</option>
      <option value="8">8 · hard block</option>
    </select>
    <button disabled={addingFic || !ficUrlId.trim()} onclick={addFic}>
      {addingFic ? 'Adding…' : 'Blacklist fic'}
    </button>
  </div>
  {#if ficMsg}<p class="msg">{ficMsg}</p>{/if}
</section>

<section class="panel">
  <h2>Add an author</h2>
  <div class="row">
    <select bind:value={authorSourceId} aria-label="Author source platform">
      {#each Object.entries(SOURCE_LABELS) as [id, label]}
        <option value={id}>{id} · {label}</option>
      {/each}
    </select>
    <input
      type="text"
      inputmode="numeric"
      placeholder="author_id (numeric ID on that source)"
      bind:value={authorId}
      onkeydown={(e) => { if (e.key === 'Enter') addAuthor(); }}
    />
    <select bind:value={authorReason} aria-label="Author blacklist reason">
      <option value="5">5 · hard block</option>
      <option value="6">6 · greylist</option>
      <option value="7">7 · hard block</option>
      <option value="8">8 · hard block</option>
    </select>
    <button disabled={addingAuthor || !authorId.trim()} onclick={addAuthor}>
      {addingAuthor ? 'Adding…' : 'Blacklist author'}
    </button>
  </div>
  {#if authorMsg}<p class="msg">{authorMsg}</p>{/if}
</section>

<section class="panel">
  <h2>Fic blacklist <span class="count">{fics.length}</span></h2>
  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if fics.length === 0}
    <p class="empty">No blacklisted fics.</p>
  {:else}
    <table>
      <thead>
        <tr><th>Fic</th><th>Reason</th><th>Added</th></tr>
      </thead>
      <tbody>
        {#each fics as it (it.url_id + it.reason)}
          <tr>
            <td><span class="mono">{it.url_id}</span></td>
            <td>{reasonLabel(it.reason)}</td>
            <td class="tiny">{fmtDate(it.created)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>
{/if}

<section class="panel">
  <h2>Author blacklist <span class="count">{authors.length}</span></h2>
  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if authors.length === 0}
    <p class="empty">No blacklisted authors.</p>
  {:else}
    <table>
      <thead>
        <tr><th>Author</th><th>Reason</th><th>Added</th></tr>
      </thead>
      <tbody>
        {#each authors as it (`${it.source_id}/${it.author_id}/${it.reason}`)}
          <tr>
            <td><span class="mono">{sourceLabel(it.source_id)}/{it.author_id}</span></td>
            <td>{reasonLabel(it.reason)}</td>
            <td class="tiny">{fmtDate(it.created)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

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
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.9em; margin: 0.85em 0 0.2em; }
  .archive-dl .dl-row:first-child dt { margin-top: 0; }
  .archive-dl .dl-row dd { margin: 0; }
  .archive-input,
  .archive-select {
    width: 100%; box-sizing: border-box; padding: 0.5em 0.6em; border-radius: 0;
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
  .archive-table td { text-align: left; padding: 0.45em 0.6em; border-bottom: 1px solid var(--archive-border, #dddddd); }
  .archive-table th { font-weight: 700; font-size: 0.85em; border-bottom: 2px solid var(--archive-border, #dddddd); }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; word-break: break-all; }
  .archive-msg { font-size: 0.92em; margin: 0.5em 0 0; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.85em; font-weight: 400; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 90ch; }
  .panel { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 1rem; margin: 1rem 0; }
  .panel h2 { font-size: 1rem; margin-bottom: 0.6rem; }
  .row { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
  .row input { flex: 1; min-width: 260px; padding: 0.45rem 0.6rem; border-radius: 8px; border: 1px solid var(--color-border, #ccc); }
  .row select { padding: 0.45rem 0.6rem; border-radius: 8px; border: 1px solid var(--color-border, #ccc); background: var(--color-surface, #fff); }
  .msg { margin-top: 0.5rem; font-size: 0.9rem; }
  .count { font-size: 0.75rem; color: var(--color-text-muted, #888); font-weight: normal; }
  table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
  th, td { text-align: left; padding: 0.45rem 0.5rem; border-bottom: 1px solid var(--color-border, #eee); }
  .mono { font-family: monospace; }
  .tiny { font-size: 0.75rem; color: var(--color-text-muted, #888); }
  button { padding: 0.3rem 0.7rem; border-radius: 8px; border: 1px solid var(--color-border, #ccc); cursor: pointer; }
  button:disabled { opacity: 0.6; cursor: default; }
  .empty { padding: 1.2rem; text-align: center; color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: 8px; }
</style>
