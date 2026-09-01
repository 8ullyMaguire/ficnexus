<script lang="ts">
  import BlindDateCard from '$lib/components/BlindDateCard.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Blind Date with a Fic</h1>
      <p class="archive-muted archive-sub">
        The title and fandom of each fic stay hidden until you hit Reveal —
        judge purely by summary, length and tropes. "Another!" draws a new fic and never
        repeats one you've already seen this session.
      </p>
    </header>

    <fieldset class="archive-fieldset">
      <legend>Today's Match</legend>
      <BlindDateCard />
    </fieldset>
  </div>
</main>
{:else}
<div class="blind-date-page">
  <BlindDateCard />
  <p class="muted hint">
    The title and fandom of each fic stay hidden until you hit <strong>Reveal</strong> —
    judge purely by summary, length and tropes. “Another!” draws a new fic and never
    repeats one you've already seen this session.
  </p>
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
  .archive-sub {
    margin: 0.4em 0 0;
    font-style: italic;
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset :global(.blind-date) {
    border: none;
    padding: 0;
  }

  /* ── Modern mode ── */
  .blind-date-page { max-width: 640px; margin: 1.5rem auto; padding: 0 1rem; }
  .hint { font-size: 0.85rem; line-height: 1.5; }
</style>
