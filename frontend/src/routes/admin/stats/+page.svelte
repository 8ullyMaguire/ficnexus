<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface DailyRow {
    date: string;
    total_users: number;
    new_users: number;
    total_works: number;
    new_works: number;
    manual_uploads: number;
    epubs_downloaded: number;
    words_read: number;
  }

  let daily = $state<DailyRow[]>([]);
  let totals = $state<{ users: number; works: number; bookmarks: number; requests: number } | null>(null);
  let loading = $state(true);
  let error = $state('');

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch('/api/admin/stats');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      daily = body.daily ?? [];
      totals = body.totals ?? null;
    } catch (e) {
      error = `Failed to load stats: ${e instanceof Error ? e.message : e}`;
      daily = [];
      totals = null;
    } finally {
      loading = false;
    }
  }

  onMount(load);

  function fmtWords(n: number): string {
    if (n == null || Number.isNaN(n)) return '—';
    if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return String(n);
  }
</script>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Platform Stats</h1>
        <p class="archive-summary">Live totals plus the last 30 days of the admin_daily_stats rollup.</p>
      </header>

      <div class="archive-form-actions">
        <button class="archive-btn" type="button" onclick={load}>↻ Refresh</button>
      </div>

      {#if loading}
        <p class="archive-note">Loading…</p>
      {:else if error}
        <p class="archive-error" role="alert">{error}</p>
      {:else}
        {#if totals}
          <fieldset class="archive-fieldset">
            <legend class="archive-legend">Live totals</legend>
            <dl class="archive-dl">
              <div class="dl-row"><dt>Users</dt><dd>{totals.users}</dd></div>
              <div class="dl-row"><dt>Works</dt><dd>{totals.works}</dd></div>
              <div class="dl-row"><dt>Bookmarks</dt><dd>{totals.bookmarks}</dd></div>
              <div class="dl-row"><dt>Requests</dt><dd>{totals.requests}</dd></div>
            </dl>
          </fieldset>
        {/if}

        {#if daily.length === 0}
          <p class="archive-note">No daily stats recorded yet.</p>
        {:else}
          <fieldset class="archive-fieldset">
            <legend class="archive-legend">Daily rollup</legend>
            <table class="archive-table">
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Total users</th>
                  <th>New users</th>
                  <th>Total works</th>
                  <th>New works</th>
                  <th>Manual uploads</th>
                  <th>EPUBs downloaded</th>
                  <th>Words read</th>
                </tr>
              </thead>
              <tbody>
                {#each daily as d (d.date)}
                  <tr>
                    <td class="archive-mono">{d.date}</td>
                    <td>{d.total_users}</td>
                    <td>{d.new_users}</td>
                    <td>{d.total_works}</td>
                    <td>{d.new_works}</td>
                    <td>{d.manual_uploads}</td>
                    <td>{d.epubs_downloaded}</td>
                    <td>{fmtWords(d.words_read)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </fieldset>
        {/if}
      {/if}
    </div>
  </main>
{:else}
  <div class="stats-page">
  <header class="page-head">
    <h1>📈 Platform Stats</h1>
    <p class="subtitle">Live totals plus the last 30 days of the admin_daily_stats rollup.</p>
    <div class="controls">
      <button class="btn" onclick={load}>↻ Refresh</button>
    </div>
  </header>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else}
    {#if totals}
      <div class="cards">
        <div class="card"><h3>Users</h3><p class="big">{totals.users}</p></div>
        <div class="card"><h3>Works</h3><p class="big">{totals.works}</p></div>
        <div class="card"><h3>Bookmarks</h3><p class="big">{totals.bookmarks}</p></div>
        <div class="card"><h3>Requests</h3><p class="big">{totals.requests}</p></div>
      </div>
    {/if}

    {#if daily.length === 0}
      <p class="empty">No daily stats recorded yet.</p>
    {:else}
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Date</th>
              <th>Total users</th>
              <th>New users</th>
              <th>Total works</th>
              <th>New works</th>
              <th>Manual uploads</th>
              <th>EPUBs downloaded</th>
              <th>Words read</th>
            </tr>
          </thead>
          <tbody>
            {#each daily as d (d.date)}
              <tr>
                <td class="mono">{d.date}</td>
                <td>{d.total_users}</td>
                <td>{d.new_users}</td>
                <td>{d.total_works}</td>
                <td>{d.new_works}</td>
                <td>{d.manual_uploads}</td>
                <td>{d.epubs_downloaded}</td>
                <td>{fmtWords(d.words_read)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
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
  .archive-form-actions { margin: 1rem 0; }
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
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.92em; flex: 0 0 9em; margin: 0.45em 0; }
  .archive-dl .dl-row dd { margin: 0.45em 0; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.88em; }
  .archive-table th,
  .archive-table td {
    text-align: left;
    padding: 0.45em 0.5em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-table th {
    font-weight: 700;
    font-size: 0.92em;
    border-bottom: 2px solid var(--archive-border, #dddddd);
  }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.94em; }
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
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.92em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .stats-page { max-width: 1100px; }
  .page-head h1 { margin-bottom: 0.3rem; }
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 80ch; }
  .controls { margin: 1rem 0; }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 0.9rem; margin: 1.2rem 0; }
  .card { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 0.9rem 1rem; }
  .card h3 { font-size: 0.78rem; text-transform: uppercase; color: var(--color-text-muted, #888); margin-bottom: 0.3rem; }
  .big { font-size: 1.6rem; font-weight: 700; }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { text-align: left; padding: 0.5rem 0.7rem; border-bottom: 1px solid var(--color-border, #eee); }
  th { font-weight: 600; color: var(--color-text-muted, #888); font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.03em; }
  .mono { font-family: monospace; }
  .empty { padding: 2rem; text-align: center; color: var(--color-text-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: var(--radius-sm, 6px); }
</style>
