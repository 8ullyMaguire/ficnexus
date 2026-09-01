<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface BotClient {
    client_id: string;
    requests: number;
    downloads: number;
    export_ratio: number;
    failed_auths: number;
    windows: number;
    bot_score: number;
    flags: string[];
    shadowbanned?: boolean;
  }

  let clients = $state<BotClient[]>([]);
  let loading = $state(true);
  let error = $state('');
  let actionMsg = $state('');
  let actingId = $state('');
  let windowHours = $state(24);
  let minRequests = $state(10);

  const FLAG_LABELS: Record<string, string> = {
    mirror: '🔁 Mirror bot (downloads ~= requests)',
    stuffing: '💥 Credential stuffing (failed logins)',
    burst: '⚡ Request burst',
  };

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch(`/api/admin/bots?window_hours=${windowHours}&min_requests=${minRequests}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      clients = body.clients ?? [];
    } catch (e) {
      error = `Failed to load bot data: ${e instanceof Error ? e.message : e}`;
      clients = [];
    } finally {
      loading = false;
    }
  }

  onMount(load);

  function pct(r: number): string {
    return `${Math.round(r * 100)}%`;
  }

  function scoreColor(s: number): string {
    if (s > 0.8) return 'var(--color-danger, #d33)';
    if (s > 0.5) return 'var(--color-warning, #d80)';
    return 'var(--color-success, #2a6)';
  }

  function shortId(id: string): string {
    return id.length > 12 ? `${id.slice(0, 8)}…` : id;
  }

  async function shadowban(c: BotClient) {
    actingId = c.client_id;
    actionMsg = '';
    try {
      const res = await adminFetch(`/api/admin/bots/${encodeURIComponent(c.client_id)}/shadowban`, {
        method: 'POST',
        credentials: 'include',
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      if (body.err !== 0) throw new Error(`err ${body.err}`);
      c.shadowbanned = true;
      actionMsg = `🚫 Shadowbanned ${shortId(c.client_id)}`;
    } catch (e) {
      actionMsg = `Failed to shadowban ${shortId(c.client_id)}: ${e instanceof Error ? e.message : e}`;
    } finally {
      actingId = '';
    }
  }

  async function unshadowban(c: BotClient) {
    actingId = c.client_id;
    actionMsg = '';
    try {
      const res = await adminFetch(`/api/admin/bots/${encodeURIComponent(c.client_id)}/unshadowban`, {
        method: 'POST',
        credentials: 'include',
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      if (body.err !== 0) throw new Error(`err ${body.err}`);
      c.shadowbanned = false;
      actionMsg = `✓ Cleared shadowban for ${shortId(c.client_id)}`;
    } catch (e) {
      actionMsg = `Failed to clear shadowban for ${shortId(c.client_id)}: ${e instanceof Error ? e.message : e}`;
    } finally {
      actingId = '';
    }
  }
</script>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Bot Behavior</h1>
        <p class="archive-summary">Clients displaying unusual behavior, ranked by composite bot score. <strong>Zero-PII:</strong> only anonymized client IDs + aggregate signals — never IPs or locations. Shadow-flagged clients face friction (stricter rate limits), not blocks.</p>
      </header>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Filters</legend>
        <dl class="archive-dl">
          <div class="dl-row">
            <dt><label for="bots-window">Window</label></dt>
            <dd>
              <select id="bots-window" class="archive-select" bind:value={windowHours} onchange={load}>
                <option value="6">6h</option>
                <option value="24">24h</option>
                <option value="72">3d</option>
                <option value="168">7d</option>
              </select>
            </dd>
          </div>
          <div class="dl-row">
            <dt><label for="bots-minreq">Min requests</label></dt>
            <dd><input id="bots-minreq" class="archive-input" type="number" bind:value={minRequests} min="1" /></dd>
          </div>
        </dl>
        <div class="archive-form-actions">
          <button class="archive-btn" type="button" onclick={load}>↻ Refresh</button>
        </div>
      </fieldset>
      {#if loading}
        <p class="archive-note">Loading…</p>
      {:else if error}
        <p class="archive-error" role="alert">{error}</p>
      {:else if clients.length === 0}
        <p class="archive-note">No flagged clients in this window.</p>
      {:else}
        <table class="archive-table">
          <thead><tr><th>Client ID</th><th>Bot score</th><th>Requests</th><th>Downloads</th><th>Export ratio</th><th>Failed auths</th><th>Flags</th><th>Shadow</th><th>Action</th></tr></thead>
          <tbody>
            {#each clients as c (c.client_id)}
              <tr>
                <td title={c.client_id} class="archive-mono">{shortId(c.client_id)}</td>
                <td>{c.bot_score.toFixed(2)}</td>
                <td>{c.requests}</td>
                <td>{c.downloads}</td>
                <td>{pct(c.export_ratio)}</td>
                <td>{c.failed_auths}</td>
                <td>
                  {#if c.flags.length === 0}<span class="archive-note">—</span>{:else}
                    {#each c.flags as f}<span class="archive-flag" title={FLAG_LABELS[f] ?? f}>{f}</span>{/each}
                  {/if}
                </td>
                <td>{#if c.shadowbanned}<span title="In the shadowban set (stricter download bucket)">🚫</span>{:else}<span class="archive-note">—</span>{/if}</td>
                <td>
                  {#if c.shadowbanned}
                    <button class="archive-btn" type="button" disabled={actingId === c.client_id} onclick={() => unshadowban(c)}>{actingId === c.client_id ? '…' : '✓ Clear'}</button>
                  {:else}
                    <button class="archive-btn" type="button" disabled={actingId === c.client_id} onclick={() => shadowban(c)}>{actingId === c.client_id ? '…' : '🚫 Shadowban'}</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
        {#if actionMsg}<p class="archive-msg" role="status">{actionMsg}</p>{/if}
        <p class="archive-note bots-hint"><strong>How to read this:</strong> export ratio = downloads ÷ requests. A mirror bot downloads nearly everything it requests (ratio → 1.0). Credential stuffing shows as failed auths. Flagged clients are rate-limited harder — a human solving the friction is never blocked outright.</p>
      {/if}
    </div>
  </main>
{:else}
<div class="bots-page">
  <header class="page-head">
    <h1>🤖 Bot Behavior</h1>
    <p class="subtitle">
      Clients displaying unusual behavior, ranked by composite bot score.
      <strong>Zero-PII:</strong> only anonymized client IDs + aggregate signals — never IPs or locations.
      Shadow-flagged clients face friction (stricter rate limits), not blocks.
    </p>
    <div class="controls">
      <label>Window
        <select bind:value={windowHours} onchange={load}>
          <option value="6">6h</option>
          <option value="24">24h</option>
          <option value="72">3d</option>
          <option value="168">7d</option>
        </select>
      </label>
      <label>Min requests
        <input type="number" bind:value={minRequests} min="1" />
      </label>
      <button class="btn" onclick={load}>↻ Refresh</button>
    </div>
  </header>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if clients.length === 0}
    <p class="empty">No flagged clients in this window. 🎉</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Client ID</th>
            <th>Bot score</th>
            <th>Requests</th>
            <th>Downloads</th>
            <th>Export ratio</th>
            <th>Failed auths</th>
            <th>Flags</th>
            <th>Shadow</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {#each clients as c (c.client_id)}
            <tr>
              <td title={c.client_id} class="mono">{shortId(c.client_id)}</td>
              <td>
                <span class="score" style="color: {scoreColor(c.bot_score)}">
                  {c.bot_score.toFixed(2)}
                </span>
              </td>
              <td>{c.requests}</td>
              <td>{c.downloads}</td>
              <td>{pct(c.export_ratio)}</td>
              <td>{c.failed_auths}</td>
              <td>
                {#if c.flags.length === 0}
                  <span class="ok">—</span>
                {:else}
                  {#each c.flags as f}
                    <span class="flag" title={FLAG_LABELS[f] ?? f}>{f}</span>
                  {/each}
                {/if}
              </td>
              <td>
                {#if c.shadowbanned}
                  <span class="shadowed" title="In the shadowban set (stricter download bucket)">🚫</span>
                {:else}
                  <span class="ok">—</span>
                {/if}
              </td>
              <td>
                {#if c.shadowbanned}
                  <button
                    class="btn btn-clear"
                    disabled={actingId === c.client_id}
                    onclick={() => unshadowban(c)}
                  >
                    {actingId === c.client_id ? '…' : '✓ Clear'}
                  </button>
                {:else}
                  <button
                    class="btn btn-ban"
                    disabled={actingId === c.client_id}
                    onclick={() => shadowban(c)}
                  >
                    {actingId === c.client_id ? '…' : '🚫 Shadowban'}
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if actionMsg}
        <p class="action-msg" role="status">{actionMsg}</p>
      {/if}
    </div>
    <p class="hint">
      <strong>How to read this:</strong> export ratio = downloads ÷ requests. A mirror bot
      downloads nearly everything it requests (ratio → 1.0). Credential stuffing shows as
      failed auths. Flagged clients are rate-limited harder — a human solving the friction
      is never blocked outright.
    </p>
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
  .archive-content { max-width: 1040px; margin: 0 auto; padding: 1rem; }
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
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.9em; flex: 0 0 9em; margin: 0.5em 0; }
  .archive-dl .dl-row dd { margin: 0.5em 0; }
  .archive-input,
  .archive-select {
    padding: 0.3em 0.5em; border-radius: 0;
    border: 1px solid var(--archive-border, #dddddd); font-family: inherit;
    font-size: 0.92em; background: var(--archive-bg, #ffffff);
  }
  .archive-form-actions { margin-top: 0.75em; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td { text-align: left; padding: 0.45em 0.6em; border-bottom: 1px solid var(--archive-border, #dddddd); }
  .archive-table th { font-weight: 700; font-size: 0.85em; border-bottom: 2px solid var(--archive-border, #dddddd); }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; }
  .archive-flag { display: inline-block; border: 1px solid var(--archive-border, #dddddd); background: var(--archive-bg-raised, #f5f5f5); padding: 0 0.4em; font-size: 0.8em; margin-right: 0.25em; }
  .archive-btn {
    display: inline-block; padding: 0.3em 1em; font-size: 0.85em; font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd); background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a); cursor: pointer; font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-msg { font-size: 0.9em; margin: 0.75rem 0 0; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.88em; }
  p.archive-note { margin: 0.5em 0; }
  .bots-hint { margin-top: 1rem; max-width: 80ch; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .bots-page { max-width: 1100px; }
  .page-head h1 { margin-bottom: 0.3rem; }
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 80ch; }
  .controls { display: flex; gap: 1rem; align-items: center; margin: 1rem 0; flex-wrap: wrap; }
  .controls label { display: flex; align-items: center; gap: 0.4rem; font-size: 0.9rem; }
  .controls select, .controls input { padding: 0.3rem 0.5rem; border-radius: var(--radius-sm, 6px); border: 1px solid var(--color-border, #ccc); }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { text-align: left; padding: 0.5rem 0.7rem; border-bottom: 1px solid var(--color-border, #eee); }
  th { font-weight: 600; color: var(--color-text-muted, #888); font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.03em; }
  .mono { font-family: monospace; }
  .score { font-weight: 700; }
  .flag { display: inline-block; background: var(--color-surface-2, #f0f0f0); border-radius: 999px; padding: 0.1rem 0.5rem; font-size: 0.75rem; margin-right: 0.3rem; }
  .ok { color: var(--color-text-muted, #888); }
  .shadowed { font-size: 1rem; }
  .btn-ban { background: #ef4444; color: #fff; border: none; padding: 0.25rem 0.6rem; border-radius: var(--radius-sm, 6px); cursor: pointer; font-size: 0.8rem; }
  .btn-clear { background: #22c55e; color: #fff; border: none; padding: 0.25rem 0.6rem; border-radius: var(--radius-sm, 6px); cursor: pointer; font-size: 0.8rem; }
  .btn-ban:disabled, .btn-clear:disabled { opacity: 0.6; cursor: wait; }
  .action-msg { margin-top: 0.75rem; font-size: 0.85rem; color: var(--color-text-muted, #888); }
  .empty { padding: 2rem; text-align: center; color: var(--color-text-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: var(--radius-sm, 6px); }
  .hint { margin-top: 1rem; font-size: 0.85rem; color: var(--color-text-muted, #888); max-width: 80ch; }
</style>
