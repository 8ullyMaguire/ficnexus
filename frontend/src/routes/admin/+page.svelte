<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface BotRow {
    client_id: string;
    bot_score: number;
    export_ratio: number;
    failed_auths: number;
    flags: string[];
    shadowbanned: boolean;
  }

  interface ZeroResult {
    query: string;
    count: number;
  }

  let realtime = $state<any>(null);
  let bots = $state<BotRow[]>([]);
  let zeroResults = $state<ZeroResult[]>([]);
  let searchVolume = $state<number[]>([]);
  let loading = $state(true);
  let error = $state('');

  const SPARK_W = 120, SPARK_H = 36;

  function sparkPoints(values: number[]): string {
    if (values.length === 0) return '';
    const max = Math.max(1, ...values);
    const step = SPARK_W / Math.max(1, values.length - 1);
    return values
      .map((v, i) => `${(i * step).toFixed(1)},${(SPARK_H - 4 - (v / max) * (SPARK_H - 8)).toFixed(1)}`)
      .join(' ');
  }

  async function loadAll() {
    // Per-card fallback: Promise.allSettled — a failing source never blanks the others.
    const [rt, botsRes, saRes] = await Promise.allSettled([
      adminFetch('/api/admin/realtime').then((r) => r.json()),
      adminFetch('/api/admin/bots?limit=5').then((r) => r.json()),
      adminFetch('/api/admin/search-analytics').then((r) => r.json()),
    ]);
    if (rt.status === 'fulfilled' && rt.value?.err === 0) {
      realtime = rt.value;
    }
    if (botsRes.status === 'fulfilled' && botsRes.value?.err === 0) {
      bots = (botsRes.value.clients ?? []).slice(0, 5);
    }
    if (saRes.status === 'fulfilled' && saRes.value?.err === 0) {
      zeroResults = (saRes.value.zero_result_queries ?? []).slice(0, 5);
      searchVolume = (saRes.value.search_volume ?? []).map((v: any) => v.count ?? 0);
    }
    loading = false;
  }

  onMount(() => {
    loadAll();
    const iv = setInterval(loadAll, 30000);
    return () => clearInterval(iv);
  });

  function fmtBytes(b: number): string {
    if (!b) return 'n/a';
    const mb = b / (1024 * 1024);
    return mb > 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb.toFixed(0)} MB`;
  }
</script>

<svelte:head><title>Command Center | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Command Center</h1>
        <p class="archive-summary">Live abuse + traffic pulse. Polls every 30s. Zero-PII: client IDs only, never IPs.</p>
      </header>
      <p class="archive-links">
        <a class="archive-link" href="/admin/comment-triage">Comment Triage</a> ·
        <a class="archive-link" href="/admin/stats">Platform Stats</a> ·
        <a class="archive-link" href="/admin/translations">Translations</a> ·
        <a class="archive-link" href="/admin/metadata">Metadata</a> ·
        <a class="archive-link" href="/curator/flags">Curator Flag Queue</a> ·
        <a class="archive-link" href="/curator/consensus">Consensus Feed</a>
      </p>
      {#if loading}
        <p class="archive-note">Loading…</p>
      {:else if error}
        <p class="archive-error" role="alert">{error}</p>
      {:else}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Live Pulse (5m / 1h)</legend>
          <dl class="archive-dl">
            <div class="dl-row"><dt>Exports (5m)</dt><dd>{realtime?.exports_5m ?? 0}</dd></div>
            <div class="dl-row"><dt>Searches (5m)</dt><dd>{realtime?.searches_5m ?? 0}</dd></div>
            <div class="dl-row"><dt>Flagged clients (1h)</dt><dd>{realtime?.flagged_clients_1h ?? 0} <span class="archive-note">mirror/stuffing/burst</span></dd></div>
            <div class="dl-row"><dt>Active IPs (1h)</dt><dd>{realtime?.active_ips_1h ?? 0} <span class="archive-note">anonymized, aggregate only</span></dd></div>
            <div class="dl-row"><dt>Redis</dt><dd>{realtime?.redis?.ok ? 'up' : 'down'} <span class="archive-note">mem {fmtBytes(realtime?.redis?.used_memory_bytes)}</span></dd></div>
          </dl>
        </fieldset>
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Top Flags</legend>
          {#if bots.length === 0}
            <p class="archive-note">No flagged clients right now.</p>
          {:else}
            <table class="archive-table">
              <thead><tr><th>Client</th><th>Score</th><th>Ratio</th><th>Auth fails</th><th>Flags</th></tr></thead>
              <tbody>
                {#each bots as b (b.client_id)}
                  <tr>
                    <td class="archive-mono">{b.client_id.slice(0, 12)}…</td>
                    <td>{b.bot_score?.toFixed(2)}</td>
                    <td>{b.export_ratio?.toFixed(2)}</td>
                    <td>{b.failed_auths}</td>
                    <td>{#each b.flags ?? [] as f}<span class="archive-flag">{f}</span>{/each}{#if b.shadowbanned}<span class="archive-flag archive-flag-shadow">shadowbanned</span>{/if}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </fieldset>
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Zero-result Queries</legend>
          {#if zeroResults.length === 0}
            <p class="archive-note">No zero-result queries (good sign!).</p>
          {:else}
            <table class="archive-table">
              <thead><tr><th>Query</th><th>Count</th></tr></thead>
              <tbody>
                {#each zeroResults as z (z.query)}
                  <tr><td class="archive-mono">{z.query}</td><td>{z.count}</td></tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </fieldset>
      {/if}
    </div>
  </main>
{:else}
<h1>Command Center</h1>
<p class="subtitle">Live abuse + traffic pulse. Polls every 30s. Zero-PII: client IDs only, never IPs.</p>
<p class="quick-links">
  <a href="/admin/comment-triage">Comment Triage</a> ·
  <a href="/admin/stats">Platform Stats</a> ·
  <a href="/admin/translations">Translations</a> ·
  <a href="/admin/metadata">Metadata</a> ·
  <a href="/curator/flags">Curator Flag Queue</a> ·
  <a href="/curator/consensus">Consensus Feed</a>
</p>

{#if loading}
  <p class="empty">Loading…</p>
{:else if error}
  <div class="error-card"><strong>{error}</strong></div>
{:else}
  <div class="cards">
    <div class="card">
      <h3>Exports (5m)</h3>
      <p class="big">{realtime?.exports_5m ?? 0}</p>
      <svg viewBox="0 0 {SPARK_W} {SPARK_H}" class="spark"><polyline points={sparkPoints(searchVolume)} class="line" fill="none" /></svg>
    </div>
    <div class="card">
      <h3>Searches (5m)</h3>
      <p class="big">{realtime?.searches_5m ?? 0}</p>
      <svg viewBox="0 0 {SPARK_W} {SPARK_H}" class="spark"><polyline points={sparkPoints(searchVolume)} class="line" fill="none" /></svg>
    </div>
    <div class="card">
      <h3>Flagged clients (1h)</h3>
      <p class="big warn">{realtime?.flagged_clients_1h ?? 0}</p>
      <p class="tiny">mirror/stuffing/burst</p>
    </div>
    <div class="card">
      <h3>Active IPs (1h)</h3>
      <p class="big">{realtime?.active_ips_1h ?? 0}</p>
      <p class="tiny">anonymized, aggregate only</p>
    </div>
    <div class="card">
      <h3>Redis</h3>
      <p class="big">{realtime?.redis?.ok ? 'up' : 'down'}</p>
      <p class="tiny">mem {fmtBytes(realtime?.redis?.used_memory_bytes)}</p>
    </div>
  </div>

  <div class="panels">
    <section class="panel">
      <h2>Top flags</h2>
      {#if bots.length === 0}
        <p class="empty">No flagged clients right now.</p>
      {:else}
        <table>
          <thead><tr><th>Client</th><th>Score</th><th>Ratio</th><th>Auth fails</th><th>Flags</th></tr></thead>
          <tbody>
            {#each bots as b (b.client_id)}
              <tr>
                <td class="mono">{b.client_id.slice(0, 12)}…</td>
                <td>{b.bot_score?.toFixed(2)}</td>
                <td>{b.export_ratio?.toFixed(2)}</td>
                <td>{b.failed_auths}</td>
                <td>
                  {#each b.flags ?? [] as f}<span class="flag">{f}</span>{/each}
                  {#if b.shadowbanned}<span class="flag shadow">shadowbanned</span>{/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="panel">
      <h2>Zero-result queries</h2>
      {#if zeroResults.length === 0}
        <p class="empty">No zero-result queries (good sign!).</p>
      {:else}
        <table>
          <thead><tr><th>Query</th><th>Count</th></tr></thead>
          <tbody>
            {#each zeroResults as z (z.query)}
              <tr><td class="mono">{z.query}</td><td>{z.count}</td></tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
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
  .archive-links { font-size: 0.9em; margin: 0.9em 0 0; }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-fieldset { border: 1px solid var(--archive-border, #dddddd); padding: 1em 1.25em; margin: 1.2rem 0; }
  .archive-legend { font-weight: 700; font-size: 1.05em; padding: 0 0.4em; }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.92em; flex: 0 0 13em; margin: 0.45em 0; }
  .archive-dl .dl-row dd { margin: 0.45em 0; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td { text-align: left; padding: 0.45em 0.6em; border-bottom: 1px solid var(--archive-border, #dddddd); }
  .archive-table th { font-weight: 700; font-size: 0.85em; border-bottom: 2px solid var(--archive-border, #dddddd); }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; }
  .archive-flag { display: inline-block; border: 1px solid var(--archive-border, #dddddd); background: var(--archive-bg-raised, #f5f5f5); padding: 0 0.4em; font-size: 0.8em; margin-right: 0.25em; }
  .archive-flag-shadow { border-color: var(--archive-link, #990000); color: var(--archive-link, #990000); }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.88em; }
  p.archive-note { margin: 0.5em 0; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .quick-links { color: var(--color-text-muted, #888); font-size: 0.85rem; margin-top: 0.3rem; }
  .quick-links a { color: var(--color-primary, #38c); text-decoration: none; }
  .quick-links a:hover { text-decoration: underline; }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 0.9rem; margin: 1.2rem 0; }
  .card { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 0.9rem 1rem; }
  .card h3 { font-size: 0.78rem; text-transform: uppercase; color: var(--color-text-muted, #888); margin-bottom: 0.3rem; }
  .big { font-size: 1.6rem; font-weight: 700; }
  .warn { color: var(--color-danger, #d33); }
  .tiny { font-size: 0.72rem; color: var(--color-text-muted, #888); }
  .spark { width: 100%; height: 40px; margin-top: 0.4rem; }
  .line { stroke: var(--color-primary, #38c); stroke-width: 1.5; }
  .panels { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
  @media (max-width: 900px) { .panels { grid-template-columns: 1fr; } }
  .panel { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 1rem; }
  .panel h2 { font-size: 1rem; margin-bottom: 0.6rem; }
  table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
  th, td { text-align: left; padding: 0.4rem 0.5rem; border-bottom: 1px solid var(--color-border, #eee); }
  .mono { font-family: monospace; }
  .flag { display: inline-block; background: color-mix(in srgb, var(--color-warn, #e90) 15%, transparent); color: var(--color-warn, #e90); border-radius: 999px; padding: 0.05rem 0.45rem; font-size: 0.7rem; margin-right: 0.25rem; }
  .flag.shadow { background: color-mix(in srgb, var(--color-danger, #d33) 15%, transparent); color: var(--color-danger, #d33); }
  .empty { padding: 1.2rem; text-align: center; color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: 8px; }
</style>
