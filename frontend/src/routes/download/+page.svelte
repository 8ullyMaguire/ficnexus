<script lang="ts">
  import DownloadTab from '$lib/components/DownloadTab.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Download</h1>
    </header>

    <fieldset class="archive-fieldset">
      <legend>Download a Story</legend>
      <DownloadTab />
    </fieldset>
  </div>
</main>
{:else}
<DownloadTab />
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
  .archive-content {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0;
    background: var(--archive-bg, #ffffff);
  }

  /* ── Modern mode ── */
</style>
