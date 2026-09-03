<script lang="ts">
  import { groupTags, orderedCategories } from './rating.js';
  import type { ExportResponse, FicMeta } from '$lib/api/types';

  interface Crumb {
    label: string;
    href: string;
  }

  let { fic }: { fic: ExportResponse } = $props();

  // AO3 tag-link convention: /search?include_tags=<typeId>:<encoded-name>
  function tagLink(typeId: number, name: string): string {
    return `/search?include_tags=${typeId}:${encodeURIComponent(name)}`;
  }

  // Build breadcrumb crumbs: Fandoms > Tag > Work
  function buildCrumbs(f: ExportResponse): Crumb[] {
    const out: Crumb[] = [];
    const m: FicMeta | undefined = f?.meta;
    if (!m) return out;

    const rawTags = (f as unknown as Record<string, unknown>).tags as
      | Array<{ category?: number | string; name: string }>
      | undefined;

    if (rawTags && rawTags.length > 0) {
      const grouped = groupTags(rawTags);
      const cats = orderedCategories(grouped);
      let addedFandom = false;
      let addedTag = false;
      for (const cat of cats) {
        const items = grouped[cat] ?? [];
        const first = items[0];
        if (!first) continue;
        if (!addedFandom) {
          out.push({ label: first.name, href: tagLink(1, first.name) });
          addedFandom = true;
        } else if (!addedTag) {
          out.push({ label: first.name, href: tagLink(Number(cat), first.name) });
          addedTag = true;
        }
        if (addedFandom && addedTag) break;
      }
    }

    // Fallback: `fandom` field from meta if tags are absent
    if (out.length === 0) {
      const fandom = (f as unknown as Record<string, unknown>).fandom as string | undefined;
      if (fandom) out.push({ label: fandom, href: tagLink(1, fandom) });
    }

    // Final crumb: Work title (current page, plain text)
    out.push({ label: m.title ?? 'Work', href: '' });
    return out;
  }

  const crumbs: Crumb[] = $derived(buildCrumbs(fic));
</script>

{#if crumbs.length > 0}
  <nav class="archive-breadcrumb" aria-label="Breadcrumb">
    <ol class="archive-breadcrumb-trail">
      {#each crumbs as crumb, i}
        <li class="archive-breadcrumb-item">
          {#if i === crumbs.length - 1}
            <span class="archive-breadcrumb-current" aria-current="page">{crumb.label}</span>
          {:else}
            <a class="archive-breadcrumb-link" href={crumb.href}>{crumb.label}</a>
          {/if}
        </li>
      {/each}
    </ol>
  </nav>
{/if}

<style>
  .archive-breadcrumb {
    margin-bottom: 1em;
    font-size: 0.82em;
  }

  .archive-breadcrumb-trail {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.25em 0.5em;
  }

  .archive-breadcrumb-item {
    display: inline-flex;
    align-items: center;
  }

  .archive-breadcrumb-item:not(:last-child)::after {
    content: "▸";
    margin-left: 0.5em;
    color: var(--archive-muted, #666666);
    font-size: 0.75em;
  }

  .archive-breadcrumb-link,
  .archive-breadcrumb-current {
    color: var(--archive-link, #990000);
    text-decoration: none;
    border-bottom: none;
  }

  .archive-breadcrumb-link:hover {
    color: #999;
    text-decoration: none;
  }

  .archive-breadcrumb-current {
    color: var(--archive-text, #2a2a2a);
    font-weight: normal;
    cursor: default;
  }
</style>
