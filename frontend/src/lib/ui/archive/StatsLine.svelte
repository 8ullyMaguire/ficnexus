<script lang="ts">
  import { formatWords, chaptersDisplay, formatUpdated } from './rating.js';

  let {
    words = 0,
    chapters = 0,
    status = '',
    kudos = 0,
    updated = '',
  }: {
    words?: number | null;
    chapters?: number | null;
    status?: string | null;
    kudos?: number | null;
    updated?: string | null;
  } = $props();

  const wordsFmt = $derived(formatWords(words));
  const chapFmt = $derived(chaptersDisplay(chapters, status));
  const kudosFmt = $derived(kudos?.toLocaleString('en-US') ?? '0');
  const updatedFmt = $derived(formatUpdated(updated));
</script>

<div class="stats-line">
  <span>Words: {wordsFmt}</span>
  <span class="stats-sep">·</span>
  <span>Chapters: {chapFmt}</span>
  {#if kudos}
    <span class="stats-sep">·</span>
    <span>Kudos: {kudosFmt}</span>
  {/if}
  {#if updatedFmt}
    <span class="stats-sep">·</span>
    <span>Updated: {updatedFmt}</span>
  {/if}
</div>

<style>
  .stats-line {
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    line-height: 1.5;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stats-sep {
    margin: 0 0.4em;
    opacity: 0.5;
  }
</style>
