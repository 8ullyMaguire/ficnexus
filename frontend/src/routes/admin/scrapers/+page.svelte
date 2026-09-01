<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let data = $state<any>(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      const res = await adminFetch('/api/admin/scraper-health');
      if (res.ok) data = await res.json();
    } catch { /* */ }
    finally { loading = false; }
  });
</script>

<svelte:head><title>Scraper Health | FicNexus Admin</title></svelte:head>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Scraper Health</h1>
      </header>

      {#if loading}
        <p class="archive-note">Loading...</p>
      {:else if data}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Success Rates (7 days)</legend>
          {#if data.sites?.length}
            <table class="archive-table">
              <thead><tr><th>Site</th><th>Total</th><th>Successes</th><th>Rate</th></tr></thead>
              <tbody>
                {#each data.sites as site}
                  <tr>
                    <td>{site.site}</td>
                    <td>{site.total_requests}</td>
                    <td>{site.successes}</td>
                    <td>
                      {site.success_rate}%
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {:else}
            <p class="archive-note">No scrape data in the last 7 days.</p>
          {/if}
        </fieldset>

        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Recent Errors (24h)</legend>
          {#if data.recent_errors?.length}
            <table class="archive-table">
              <thead><tr><th>Time</th><th>Site</th><th>Error</th></tr></thead>
              <tbody>
                {#each data.recent_errors as err}
                  <tr>
                    <td class="archive-mono">{err.time}</td>
                    <td>{err.site}</td>
                    <td>{err.error}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {:else}
            <p class="archive-note">No errors in the last 24 hours.</p>
          {/if}
        </fieldset>
      {:else}
        <p class="archive-error" role="alert">Failed to load scraper health.</p>
      {/if}
    </div>
  </main>
{:else}
<h1>Scraper Health</h1>

{#if loading}
  <p>Loading...</p>
{:else if data}
  <h2>Success Rates (7 days)</h2>
  {#if data.sites?.length}
    <table class="admin-table">
      <thead><tr><th>Site</th><th>Total</th><th>Successes</th><th>Rate</th></tr></thead>
      <tbody>
        {#each data.sites as site}
          <tr>
            <td>{site.site}</td>
            <td>{site.total_requests}</td>
            <td>{site.successes}</td>
            <td>
              <span class:good={site.success_rate >= 90} class:bad={site.success_rate < 90}>
                {site.success_rate}%
              </span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <p>No scrape data in the last 7 days.</p>
  {/if}

  <h2>Recent Errors (24h)</h2>
  {#if data.recent_errors?.length}
    <table class="admin-table">
      <thead><tr><th>Time</th><th>Site</th><th>Error</th></tr></thead>
      <tbody>
        {#each data.recent_errors as err}
          <tr>
            <td>{err.time}</td>
            <td>{err.site}</td>
            <td class="error-text">{err.error}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <p>No errors in the last 24 hours. 🎉</p>
  {/if}
{:else}
  <p>Failed to load scraper health.</p>
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
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.92em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .admin-table { width: 100%; border-collapse: collapse; margin-top: 1rem; }
  .admin-table th, .admin-table td { padding: 0.5rem; text-align: left; border-bottom: 1px solid var(--color-border); font-size: 0.88rem; }
  .admin-table th { font-weight: 600; background: var(--color-surface-2); }
  .good { color: #22c55e; font-weight: bold; }
  .bad { color: #ef4444; font-weight: bold; }
  .error-text { color: var(--color-error); font-family: var(--mono); font-size: 0.82rem; max-width: 400px; overflow: hidden; text-overflow: ellipsis; }
</style>
