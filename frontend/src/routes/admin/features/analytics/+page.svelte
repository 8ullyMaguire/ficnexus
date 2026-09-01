<script lang="ts">
  import { onMount } from 'svelte';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface FeatureStat {
    slug: string;
    enabled_count: number;
  }

  let stats = $state<FeatureStat[]>([]);
  let totalUsers = $state(0);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    await loadStats();
  });

  async function loadStats() {
    loading = true;
    error = '';
    try {
      const data = await adminFetch('/api/admin/features/stats');
      stats = await data.json();
      // Also fetch total user count for percentage calculation
      try {
        const usersRes = await adminFetch('/api/admin/users?per_page=1');
        const usersBody = await usersRes.json();
        totalUsers = usersBody.total ?? 0;
      } catch {
        // If total users unavailable, use max enabled_count as proxy
        totalUsers = Math.max(...stats.map((s) => s.enabled_count), 1);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function barWidth(count: number): number {
    if (totalUsers === 0) return 0;
    return Math.round((count / totalUsers) * 100);
  }

  function pct(count: number): string {
    if (totalUsers === 0) return '0%';
    return ((count / totalUsers) * 100).toFixed(1) + '%';
  }
</script>

<svelte:head><title>Feature Adoption Analytics — FicNexus Admin</title></svelte:head>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Feature Adoption Analytics</h1>
      </header>

      <p class="archive-links">
        <a class="archive-link" href="/admin/features">← Back to Feature Flags</a>
        {' · '}
        <button class="archive-btn" type="button" onclick={loadStats} disabled={loading}>🔄 Refresh</button>
      </p>

      {#if error}
        <p class="archive-error" role="alert">{error}</p>
      {/if}

      {#if loading}
        <p class="archive-note">Loading stats…</p>
      {:else}
        <p class="archive-note">Total users: <strong>{totalUsers}</strong></p>

        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Adoption Overview</legend>
          <dl class="archive-dl">
            {#each stats as s (s.slug)}
              <div class="dl-row">
                <dt class="archive-mono" title={s.slug}>{s.slug}</dt>
                <dd>
                  <span class="bar-track"><span class="bar-fill" style="width: {barWidth(s.enabled_count)}%"></span></span>
                  {' '}
                  {s.enabled_count} ({pct(s.enabled_count)})
                </dd>
              </div>
            {/each}
          </dl>
        </fieldset>

        <table class="archive-table">
          <thead>
            <tr>
              <th>Feature Slug</th>
              <th>Enabled Users</th>
              <th>Total Users</th>
              <th>Adoption %</th>
            </tr>
          </thead>
          <tbody>
            {#each stats as s (s.slug)}
              <tr>
                <td class="archive-mono"><code>{s.slug}</code></td>
                <td>{s.enabled_count}</td>
                <td>{totalUsers}</td>
                <td>{pct(s.enabled_count)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </main>
{:else}
<div class="admin-page">
  <div class="page-header">
    <h1>📊 Feature Adoption Analytics</h1>
    <div class="header-actions">
      <a class="btn btn-secondary btn-sm" href="/admin/features">← Back to Feature Flags</a>
      <button class="btn btn-secondary btn-sm" onclick={loadStats} disabled={loading}>
        🔄 Refresh
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-card">⚠️ {error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading stats…</p>
  {:else}
    <p class="muted subtitle">Total users: <strong>{totalUsers}</strong></p>

    <!-- Bar chart -->
    <div class="chart-card">
      <h3>Adoption Overview</h3>
      <div class="bar-chart">
        {#each stats as s (s.slug)}
          <div class="bar-row">
            <span class="bar-label" title={s.slug}>{s.slug}</span>
            <div class="bar-track">
              <div class="bar-fill" style="width: {barWidth(s.enabled_count)}%"></div>
            </div>
            <span class="bar-value">{s.enabled_count} ({pct(s.enabled_count)})</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Detail table -->
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Feature Slug</th>
            <th class="right">Enabled Users</th>
            <th class="right">Total Users</th>
            <th class="right">Adoption %</th>
          </tr>
        </thead>
        <tbody>
          {#each stats as s (s.slug)}
            <tr>
              <td><code>{s.slug}</code></td>
              <td class="right">{s.enabled_count}</td>
              <td class="right">{totalUsers}</td>
              <td class="right">{pct(s.enabled_count)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
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
  .archive-links { font-size: 0.92em; margin: 0.9em 0 0; }
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
  .archive-dl .dl-row { display: flex; align-items: center; }
  .archive-dl .dl-row dt {
    font-family: 'Courier New', Courier, monospace;
    font-weight: 400;
    font-size: 0.88em;
    flex: 0 0 12em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0.4em 0;
  }
  .archive-dl .dl-row dd { margin: 0.4em 0; flex: 1; min-width: 0; }
  .bar-track {
    display: inline-block;
    vertical-align: middle;
    width: 60%;
    height: 0.85em;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .bar-fill {
    display: block;
    height: 100%;
    background: var(--archive-accent, #990000);
    min-width: 2px;
  }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td {
    text-align: left;
    padding: 0.45em 0.6em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-table th {
    font-weight: 700;
    font-size: 0.88em;
    border-bottom: 2px solid var(--archive-border, #dddddd);
  }
  .archive-mono { font-family: 'Courier New', Courier, monospace; white-space: nowrap; }
  .archive-mono code { font-family: inherit; font-size: 0.95em; }
  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.9em;
    font-size: 0.85em;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    cursor: pointer;
    font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.9em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .admin-page { max-width: 900px; margin: 0 auto; padding: 1.5rem; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .page-header h1 { margin: 0; }
  .header-actions { display: flex; gap: 0.5rem; align-items: center; }
  .subtitle { margin-bottom: 1rem; }
  .chart-card {
    background: var(--color-surface, #fff);
    border: 1px solid var(--color-border, #ddd);
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1.5rem;
  }
  .chart-card h3 { margin: 0 0 0.75rem; }
  .bar-chart { display: flex; flex-direction: column; gap: 0.4rem; }
  .bar-row { display: flex; align-items: center; gap: 0.5rem; }
  .bar-label {
    width: 180px; flex-shrink: 0; font-size: 0.82rem;
    font-family: monospace; white-space: nowrap; overflow: hidden;
    text-overflow: ellipsis;
  }
  .bar-track {
    flex: 1; height: 18px; background: var(--color-surface-2, #f3f4f6);
    border-radius: 4px; overflow: hidden;
  }
  .bar-fill {
    height: 100%; background: var(--color-accent, #6366f1);
    border-radius: 4px; min-width: 2px; transition: width 0.3s;
  }
  .bar-value { width: 110px; flex-shrink: 0; font-size: 0.82rem; color: var(--color-text-muted, #888); }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.88rem; }
  th { text-align: left; padding: 0.5rem 0.6rem; border-bottom: 2px solid var(--color-border, #ccc); }
  .right { text-align: right; }
  td { padding: 0.4rem 0.6rem; border-bottom: 1px solid var(--color-border, #eee); }
  td code { font-size: 0.82rem; background: var(--color-surface-2, #f3f4f6); padding: 0.15rem 0.35rem; border-radius: 4px; }
  .muted { color: var(--color-text-muted, #888); }
  .error-card { background: #fee2e2; color: #991b1b; padding: 0.75rem; border-radius: 8px; margin-bottom: 1rem; }
  .btn { padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--color-border, #ccc); cursor: pointer; background: var(--color-surface, #fff); text-decoration: none; display: inline-block; }
  .btn-secondary { background: var(--color-surface-2, #f3f4f6); }
  .btn-sm { font-size: 0.82rem; padding: 0.3rem 0.6rem; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
