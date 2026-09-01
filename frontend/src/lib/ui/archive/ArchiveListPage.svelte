<script lang="ts">
  import WorkBlurb from './WorkBlurb.svelte';
  import ArchiveEmpty from './ArchiveEmpty.svelte';
  import { t } from '$lib/i18n/index.svelte';

  interface FicItem {
    url_id?: string | null;
    title?: string | null;
    author?: string | null;
    source?: string | null;
    words?: number | null;
    chapters?: number | null;
    status?: string | null;
    description?: string | null;
    updated?: string | null;
    rank?: number | null;
    snippet?: string | null;
    tags?: Array<{ category?: number | string; name: string }>;
    total_freeform?: number | null;
    comment_count?: number | null;
    kudos_count?: number | null;
    rating?: string | null;
  }

  let {
    title = 'Works',
    items = [],
    loading = false,
    error = '',
    subtitle = '',
  }: {
    title?: string;
    items?: FicItem[];
    loading?: boolean;
    error?: string;
    subtitle?: string;
  } = $props();
</script>

<div class="archive-list-page">
  <header class="list-header">
    <h1 class="list-title">{title}</h1>
    {#if subtitle}
      <p class="list-subtitle">{subtitle}</p>
    {/if}
    {#if !loading && !error}
      <p class="list-count">{items.length} {items.length === 1 ? 'work' : 'works'}</p>
    {/if}
  </header>

  {#if loading}
    <ArchiveEmpty {loading} />
  {:else if error}
    <ArchiveEmpty loading={false} empty={error} />
  {:else if items.length === 0}
    <ArchiveEmpty loading={false} empty={t('search.noResults')} />
  {:else}
    <div class="blurb-list">
      {#each items as fic (fic.url_id ?? Math.random())}
        <!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
        <WorkBlurb fic={fic as any} />
    {/each}
    </div>
  {/if}
</div>

<style>
  .archive-list-page {
    display: flex;
    flex-direction: column;
    gap: 1em;
  }

  .list-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
  }

  .list-title {
    font-size: 1.3em;
    font-weight: 700;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }

  .list-subtitle {
    font-size: 0.9em;
    color: var(--archive-muted, #666666);
    margin: 0.2em 0 0;
  }

  .list-count {
    font-size: 0.82em;
    color: var(--archive-muted, #666666);
    margin: 0.3em 0 0;
  }

  .blurb-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  /* Skeleton + empty styles now live in ArchiveEmpty.svelte. */
</style>
