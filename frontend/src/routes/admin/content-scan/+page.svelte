<script lang="ts">
  import { onMount } from 'svelte';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let items = $state<ScanItem[]>([]);
  let loading = $state(true);
  let error = $state('');
  let running = $state(false);
  let runMsg = $state('');
  let mining = $state<any>(null);
  let miningLoading = $state(false);
  let dedupeMsg = $state('');

  interface ScanItem {
    url_id: string;
    classification: string;
    confidence: number;
    detected_warnings: string;
    reason: string;
    review_status: string;
    scanned_at: string;
  }

  onMount(() => {
    load();
  });

  async function load() {
    loading = true;
    try {
      const res = await adminFetch('/api/admin/content-scan?review_status=pending');
      const data = await res.json() as { items?: ScanItem[] };
      items = data.items ?? [];
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function review(url_id: string, action: 'confirmed' | 'dismissed') {
    try {
      await adminFetch(`/api/admin/content-scan/${url_id}/review`, {
        method: 'POST',
        body: JSON.stringify({ action }),
        headers: { 'Content-Type': 'application/json' },
      });
      items = items.filter((i) => i.url_id !== url_id);
    } catch (e) {
      error = String(e);
    }
  }

  async function runScan() {
    running = true;
    runMsg = '';
    try {
      const res = await adminFetch('/api/admin/content-scan/run', {
        method: 'POST',
        body: JSON.stringify({}),
        headers: { 'Content-Type': 'application/json' },
      });
      const data = await res.json() as { scanned?: number; failed?: number };
      runMsg = `Scanned ${data.scanned ?? 0}, failed ${data.failed ?? 0}`;
      load();
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  async function runMining() {
    miningLoading = true;
    mining = null;
    try {
      const res = await adminFetch('/api/admin/search-mining');
      mining = res;
    } catch (e) {
      error = String(e);
    } finally {
      miningLoading = false;
    }
  }

  async function runDedupe() {
    dedupeMsg = '';
    try {
      const res = await adminFetch('/api/admin/dedupe/run', {
        method: 'POST',
        body: JSON.stringify({}),
        headers: { 'Content-Type': 'application/json' },
      });
      const data = await res.json() as { examined?: number; proposed?: number };
      dedupeMsg = `Examined ${data.examined ?? 0} pairs, proposed ${data.proposed ?? 0} merges`;
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Content Scan</h1>
        <p class="archive-summary">
          LLM verification of cached fic bodies — story-vs-noise sanity + archive warning
          checks. Keep the <code>no_warnings</code> search honest.
        </p>
      </header>

      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Batch actions</legend>
        <div class="archive-form-actions">
          <button class="archive-btn" type="button" onclick={runScan} disabled={running}>{running ? 'Scanning…' : '▶ Run batch scan'}</button>
          {#if runMsg}<span class="archive-ok">{runMsg}</span>{/if}
          {' '}
          <button class="archive-btn" type="button" onclick={runMining} disabled={miningLoading}>{miningLoading ? 'Mining…' : '📈 Search demand mining'}</button>
          {' '}
          <button class="archive-btn" type="button" onclick={runDedupe}>🔗 Crosspost dedupe</button>
          {#if dedupeMsg}<span class="archive-ok">{dedupeMsg}</span>{/if}
        </div>
      </fieldset>

      {#if error}<p class="archive-error" role="alert">{error}</p>{/if}

      {#if mining}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Demand signals (zero-result searches)</legend>
          {#if mining.themes?.length}
            <dl class="archive-dl">
              {#each mining.themes as theme (theme.theme)}
                <div class="dl-row">
                  <dt>{theme.theme}</dt>
                  <dd>
                    {theme.suggestion}
                    <span class="chips">{#each theme.queries as q (q)}<span class="chip">{q}</span>{/each}</span>
                  </dd>
                </div>
              {/each}
            </dl>
          {:else if mining.raw_queries?.length}
            <p class="archive-note">(LLM clustering unavailable — raw zero-hit queries:)</p>
            <p class="chips">
              {#each mining.raw_queries.slice(0, 20) as r (r.query)}
                <span class="chip">{r.query} ×{r.count}</span>
              {/each}
            </p>
          {:else}
            <p class="archive-note">No zero-hit queries in the window.</p>
          {/if}
        </fieldset>
      {/if}

      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Pending scan results</legend>
        {#if loading}
          <p class="archive-note">Loading…</p>
        {:else if items.length === 0}
          <p class="archive-note">No pending flags — bodies look clean.</p>
        {:else}
          {#each items as item (item.url_id)}
            <div class="scan-item">
              <dl class="archive-dl">
                <div class="dl-row">
                  <dt>Work</dt>
                  <dd><code>{item.url_id}</code></dd>
                </div>
                <div class="dl-row">
                  <dt>Classification</dt>
                  <dd><span class="chip">{item.classification}</span>{#if item.detected_warnings} <span class="chip warn">⚠ {item.detected_warnings}</span>{/if} <span class="archive-note">conf {(item.confidence * 100).toFixed(0)}%</span></dd>
                </div>
                <div class="dl-row">
                  <dt>Reason</dt>
                  <dd>{item.reason}</dd>
                </div>
              </dl>
              <div class="archive-form-actions">
                <button class="archive-btn archive-btn-primary" type="button" onclick={() => review(item.url_id, 'confirmed')}>✓ Confirm</button>
                {' '}
                <button class="archive-btn" type="button" onclick={() => review(item.url_id, 'dismissed')}>✕ Dismiss</button>
              </div>
            </div>
          {/each}
        {/if}
      </fieldset>
    </div>
  </main>
{:else}
<h1>🔍 Content Scan</h1>
<p class="muted">LLM verification of cached fic bodies — story-vs-noise sanity + archive warning checks. Keep the <code>no_warnings</code> search honest.</p>

<div class="actions">
  <button class="btn" onclick={runScan} disabled={running}>{running ? 'Scanning…' : '▶ Run batch scan'}</button>
  {#if runMsg}<span class="ok">{runMsg}</span>{/if}
  <button class="btn" onclick={runMining} disabled={miningLoading}>{miningLoading ? 'Mining…' : '📈 Search demand mining'}</button>
  <button class="btn" onclick={runDedupe}>🔗 Crosspost dedupe</button>
  {#if dedupeMsg}<span class="ok">{dedupeMsg}</span>{/if}
</div>

{#if error}<div class="error-card"><strong>⚠️ {error}</strong></div>{/if}

{#if mining}
  <h2>Demand signals (zero-result searches)</h2>
  {#if mining.themes?.length}
    {#each mining.themes as theme (theme.theme)}
      <div class="card">
        <strong>{theme.theme}</strong> — <span class="muted">{theme.suggestion}</span>
        <div class="chips">{#each theme.queries as q (q)}<span class="chip">{q}</span>{/each}</div>
      </div>
    {/each}
  {:else if mining.raw_queries?.length}
    <p class="muted">(LLM clustering unavailable — raw zero-hit queries:)</p>
    {#each mining.raw_queries.slice(0, 20) as r (r.query)}
      <span class="chip">{r.query} ×{r.count}</span>
    {/each}
  {:else}
    <p class="muted">No zero-hit queries in the window.</p>
  {/if}
{/if}

<h2>Pending scan results</h2>
{#if loading}
  <p>Loading…</p>
{:else if items.length === 0}
  <p class="muted">No pending flags — bodies look clean.</p>
{:else}
  <div class="scan-list">
    {#each items as item (item.url_id)}
      <div class="card scan-item">
        <div class="scan-head">
          <code>{item.url_id}</code>
          <span class="badge {item.classification}">{item.classification}</span>
          {#if item.detected_warnings}
            <span class="badge warn">⚠ {item.detected_warnings}</span>
          {/if}
          <span class="muted">conf {(item.confidence * 100).toFixed(0)}%</span>
        </div>
        <p class="reason">{item.reason}</p>
        <div class="actions">
          <button class="btn btn-primary" onclick={() => review(item.url_id, 'confirmed')}>✓ Confirm</button>
          <button class="btn" onclick={() => review(item.url_id, 'dismissed')}>✕ Dismiss</button>
        </div>
      </div>
    {/each}
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
  .archive-content { max-width: 900px; margin: 0 auto; padding: 1rem; }
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
  .archive-form-actions { margin-top: 0.75em; display: flex; gap: 0.5em; align-items: center; flex-wrap: wrap; }
  .archive-btn {
    display: inline-block;
    padding: 0.3em 1em;
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
  .archive-dl { margin: 0; padding: 0; }
  .scan-item { border-bottom: 1px solid var(--archive-border, #dddddd); padding-bottom: 0.75em; margin-bottom: 0.75em; }
  .scan-item:last-child { border-bottom: none; margin-bottom: 0; }
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.88em; flex: 0 0 8em; margin: 0.4em 0; }
  .archive-dl .dl-row dd { margin: 0.4em 0; min-width: 0; flex: 1; overflow-wrap: anywhere; }
  .chips { display: inline-flex; gap: 0.35em; flex-wrap: wrap; }
  .chip {
    display: inline-block;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    padding: 0 0.45em;
    font-size: 0.82em;
  }
  .chip.warn { border-color: var(--archive-link, #990000); color: var(--archive-link, #990000); }
  .archive-ok { color: #2e7d32; font-size: 0.9em; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.88em; }
  p.archive-note { margin: 0.5em 0; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .actions { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin: 12px 0; }
  .ok { color: var(--color-success, #2e7d32); }
  .error-card { background: #fdecea; border: 1px solid #f5c6cb; padding: 10px; border-radius: 8px; margin: 12px 0; }
  .chips { display: flex; gap: 6px; flex-wrap: wrap; margin-top: 6px; }
  .chip { background: var(--color-surface-2, #eee); padding: 2px 8px; border-radius: 12px; font-size: 12px; }
  .scan-list { display: flex; flex-direction: column; gap: 8px; }
  .scan-head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .badge { padding: 2px 8px; border-radius: 10px; font-size: 12px; background: #e3f2fd; }
  .badge.noise { background: #fff3e0; color: #e65100; }
  .badge.warn { background: #fce4ec; color: #c62828; }
  .reason { color: var(--color-text-muted, #666); font-size: 13px; margin: 6px 0 0; }
</style>
