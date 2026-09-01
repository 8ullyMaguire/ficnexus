<script lang="ts">
  import { getPref } from '$lib/prefs';
  import ThemeEditor from '$lib/components/ThemeEditor.svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { onMount } from 'svelte';

  const uiMode = $derived(getPref('uiMode'));

  let ready = $state(false);

  onMount(async () => {
    await auth.init();
    ready = true;
  });
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Theme Settings</h1>
      <p class="archive-muted">Customise colours and typography. The editor below works the same in archive mode.</p>
    </header>

    {#if !ready}
      <p class="archive-muted"><span class="spinner"></span> Loading…</p>
    {:else if !auth.isLoggedIn}
      <blockquote class="archive-summary">
        Log in to save your theme across devices.
        Changes are applied locally right away.
      </blockquote>
    {/if}

    <ThemeEditor />
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="settings-page">
  <h1>🎨 Theme Settings</h1>

  {#if !ready}
    <div class="card">
      <p class="muted"><span class="spinner"></span> Loading…</p>
    </div>
  {:else if !auth.isLoggedIn}
    <div class="card empty">
      <p class="muted">Log in to save your theme across devices.</p>
      <p class="muted" style="margin-top:0.4rem;font-size:0.85rem;">Changes are applied locally right away.</p>
    </div>
  {/if}

  <ThemeEditor />
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
  .archive-content {
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-header {
    padding: 0 0 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1rem;
  }
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-summary {
    margin: 0.5rem 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }

  /* ── Modern mode (scoped — no bleed) ── */
  .settings-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  .settings-page h1 {
    margin-top: 0;
    margin-bottom: 1rem;
  }
  .settings-page .empty {
    text-align: center;
    padding: 1.5rem;
  }
  .settings-page .muted {
    color: var(--color-muted);
  }
  .settings-page .spinner,
  .archive-main .spinner {
    display: inline-block;
    width: 0.8rem;
    height: 0.8rem;
    border: 2px solid var(--color-border, #555);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    vertical-align: middle;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
