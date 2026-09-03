<script lang="ts">
  // DocLink — inline "?" help link into the same-origin docs.
  // Opens the help modal by default (keeps SPA state); middle-click /
  // ctrl+click / the "open full docs" footer go to the real page.
  import { openDoc } from '$lib/stores/doc-help.svelte';

  export let slug: string;       // docs-map.json slug, e.g. 'searching#boolean-query-syntax'
  export let label = '?';
  export let title = 'View docs';
  export let inline = false;     // true → render as a plain ? link (no padding)
</script>

<a
  href={`/docs/${slug.replace('#', '.html#')}`}
  class="doc-link"
  class:inline
  title={title}
  aria-label={title}
  onclick={(e) => {
    // Let ctrl/middle-click / target=_blank behavior win (open real page)
    if (e.ctrlKey || e.metaKey || e.shiftKey || e.button === 1) return;
    e.preventDefault();
    openDoc(slug);
  }}
  >{label}</a
>

<style>
  .doc-link {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.35rem;
    height: 1.35rem;
    border-radius: 999px;
    border: 1px solid var(--color-border, #ccc);
    background: var(--color-bg, #fff);
    color: var(--color-text-muted, #888);
    font-size: 0.75rem;
    font-weight: 700;
    text-decoration: none;
    cursor: help;
    vertical-align: middle;
    flex-shrink: 0;
  }
  .doc-link:hover {
    border-color: var(--color-accent, #4a6cf7);
    color: var(--color-accent, #4a6cf7);
  }
  .doc-link.inline {
    border: none;
    background: transparent;
    width: auto;
    height: auto;
    font-size: 0.85rem;
  }
</style>
