<script lang="ts">
  import {
    categoryLabel,
    orderedCategories,
  } from './rating.js';

  interface Tag {
    category?: number | string;
    name: string;
    [key: string]: unknown;
  }

  let {
    tags = {},
    showAll = false,
  }: {
    tags: Record<string, Tag[]>;
    showAll?: boolean;
  } = $props();

  const MAX_FREEFORM = 6;
  const ordered = $derived(orderedCategories(tags));

  function tagUrl(type: string, name: string): string {
    return `/search?include_tags=${type}:${encodeURIComponent(name)}`;
  }
</script>

<div class="tag-soup">
  {#each ordered as cat}
    {@const items = tags[cat] ?? []}
    {#if items.length > 0}
      <div class="tag-row">
        <span class="tag-label">{categoryLabel(cat)}</span>
        <span class="tag-list">
          {#if cat === '4' && !showAll && items.length > MAX_FREEFORM}
            {#each items.slice(0, MAX_FREEFORM) as tag, i}
              {#if i > 0}<span class="tag-sep">,</span>{/if}
              <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
            {/each}
            <details class="tag-more-details">
              <summary class="tag-more-summary">
                +{items.length - MAX_FREEFORM} more
              </summary>
              <span class="tag-list">
                {#each items.slice(MAX_FREEFORM) as tag, i}
                  {#if i > 0}<span class="tag-sep">,</span>{/if}
                  <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
                {/each}
              </span>
            </details>
          {:else}
            {#each items as tag, i}
              {#if i > 0}<span class="tag-sep">,</span>{/if}
              <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
            {/each}
          {/if}
        </span>
      </div>
    {/if}
  {/each}
</div>

<style>
  .tag-soup {
    margin: 0.3em 0 0.5em;
    font-size: 0.9em;
    line-height: 1.7;
  }

  .tag-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25em;
  }

  .tag-label {
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin-right: 0.35em;
    flex-shrink: 0;
  }

  .tag-list {
    display: inline;
  }

  .tag-link {
    color: #111;
    line-height: 1.5;
    text-decoration: none;
    border-bottom: 1px dotted;
  }

  .tag-link:hover {
    background: #900;
    color: #fff;
    border-bottom: 1px dotted;
  }

  .tag-sep {
    color: var(--archive-muted, #666666);
    margin: 0 0.1em;
  }

  .tag-more-details {
    display: inline;
    vertical-align: baseline;
  }

  .tag-more-summary {
    display: inline;
    color: var(--archive-muted, #666666);
    cursor: pointer;
    font-size: 0.92em;
  }

  .tag-more-summary:hover {
    color: var(--archive-link, #990000);
  }
</style>
