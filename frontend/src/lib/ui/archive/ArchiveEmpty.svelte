<script lang="ts">
  // Shared empty + loading skeleton component for archive-mode pages.
  // Usage:
  //   <ArchiveEmpty loading={loading} empty={msg} />
  //   - loading=true  → skeleton rows
  //   - loading=false → empty message (empty prop)
  //   - pass empty="" and loading=false to render nothing
  let {
    loading = false,
    empty = 'Nothing here yet.',
    count = 5,
  }: {
    loading?: boolean;
    empty?: string;
    count?: number;
  } = $props();
</script>

{#if loading}
  <div class="archive-skeleton-list">
    {#each Array(count) as _, i}
      <article class="skeleton-blurb" aria-hidden="true">
        <div class="skel-header"><span class="skel-badge"></span><span class="skel-title"></span></div>
        <span class="skel-byline"></span>
        <div class="skel-tags">
          <span class="skel-tag"></span>
          <span class="skel-tag short"></span>
        </div>
        <span class="skel-snippet"></span>
        <div class="skel-stats"></div>
      </article>
    {/each}
  </div>
{:else if empty}
  <div class="archive-empty">
    <p class="archive-empty-msg">{empty}</p>
  </div>
{/if}

<style>
  /* ── Skeleton (loading) ── */
  .archive-skeleton-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  article.skeleton-blurb {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 1em;
    margin-bottom: 0.75em;
    background: var(--archive-bg, #ffffff);
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .skel-header {
    display: flex;
    align-items: center;
    gap: 0.5em;
    margin-bottom: 0.4em;
  }

  .skel-badge,
  .skel-title,
  .skel-byline,
  .skel-tags,
  .skel-snippet,
  .skel-stats {
    display: block;
    background: var(--archive-border, #dddddd);
    border-radius: 2px;
  }

  .skel-badge { width: 2em; height: 1.2em; }
  .skel-title  { width: 40%; height: 1.2em; margin: 0; }
  .skel-byline { width: 30%; height: 0.9em; margin: 0.3em 0; }
  .skel-tags   { display: flex; gap: 0.3em; margin: 0.3em 0; }
  .skel-tag    { width: 3.5em; height: 1em; }
  .skel-tag.short { width: 2.2em; }
  .skel-snippet { width: 80%; height: 0.85em; margin: 0.3em 0; }
  .skel-stats   { width: 40%; height: 0.85em; margin-top: 0.3em; }

  /* ── Empty state ── */
  .archive-empty {
    padding: 2em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .archive-empty-msg {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    margin: 0;
  }
</style>
