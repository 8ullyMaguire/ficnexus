<script lang="ts">
  import DOMPurify from 'dompurify';
  import { mapRating, formatUpdated, chaptersDisplay, formatWords, groupTags, categoryLabel, orderedCategories, relativeTime } from './rating.js';
  import SymbolSquares from './SymbolSquares.svelte';
  import { fetchExport } from '$lib/api/client';
  import { getPref } from '$lib/prefs';

  interface FicSearchResult {
    author_id?: number | null;
    chapters?: number | null;
    comment_count?: number | null;
    description?: string | null;
    kudos_count?: number | null;
    rank?: number | null;
    rating?: string | null;
    snippet?: string | null;
    source?: string | null;
    status?: string | null;
    categories?: string[] | null;
    warnings?: string[] | null;
    tags?: Array<{ category?: number | string; name: string }>;
    title?: string | null;
    total_freeform?: number | null;
    updated?: string | null;
    url_id?: string | null;
    work_id?: number | null;
    words?: number | null;
    bookmarks_count?: number | null;
    collections_count?: number | null;
    hits?: number | null;
    language?: string | null;
    series?: Array<{ name: string; id?: number; position?: number }> | null;
    /** ISO timestamp of the last visit to this work (from reading_history), when available. */
    last_visited?: string | null;
    [key: string]: unknown;
  }

  let {
    fic,
    showAllTags = false,
  }: {
    fic: FicSearchResult;
    showAllTags?: boolean;
  } = $props();

  const title = $derived(fic.title ?? 'Untitled');
  const author: string = $derived((fic.author as string | null | undefined) ?? 'Anonymous');
  const rating = $derived(mapRating(fic.rating));
  const snippet = $derived(fic.snippet ?? fic.description ?? '');
  const lastVisited = $derived(relativeTime(fic.last_visited));

  let deleted = $state(false);
  let downloading = $state(false);
  const preferredFormat = $derived(getPref('defaultFormat') || 'epub');
  let cardEl = $state<HTMLElement | null>(null);

  // Group tags by category (AO3-style meta rendering)
  const groupedTags = $derived.by(() => {
    if (!fic.tags) return {} as Record<string, Array<{ category: number | string; name: string }>>;
    return groupTags(fic.tags);
  });

  const ordered = $derived(orderedCategories(groupedTags));

  const fandoms = $derived.by(() => {
    const f = groupedTags['1'] ?? [];
    return f;
  });

  function truncate(text: string, max: number): string {
    if (!text) return '';
    if (text.length <= max) return text;
    return text.slice(0, max).trimEnd() + '…';
  }

  /** Sanitize the snippet before injecting as HTML — the backend may return
   *  truncated HTML from AO3/fanfiction.net descriptions that could contain
   *  <script> tags or on* event handlers. DOMPurify with the html profile
   *  strips anything outside a safe allowlist while preserving <p>, <br>,
   *  <a>, <em>, etc. */
  function safeSnippet(text: string): string {
    return DOMPurify.sanitize(truncate(text, 300), { USE_PROFILES: { html: true } });
  }

  // Build comma-joined tag list for AO3-style rendering
  function tagLink(tag: { category?: number | string; name: string; [key: string]: unknown }): string {
    const typeId = String(tag.category ?? '4');
    return `/search?include_tags=${typeId}:${encodeURIComponent(tag.name)}`;
  }

  /** Split a comma-joined author string into individual author names.
   *  Handles "Author1, Author2" and "Author1 and Author2" formats. */
  function splitAuthors(authorStr: string): string[] {
    if (!authorStr) return ['Anonymous'];
    // Split on comma, but not if it's "Last, First" format (no space after comma suggests name format)
    // Actually, AO3 uses ", " between authors and " and " for the last one
    return authorStr
      .split(/, | and /)
      .map(a => a.trim())
      .filter(a => a.length > 0);
  }

  /** Generate author links for display. Returns array of { name, href }. */
  function authorLinks(): Array<{ name: string; href: string }> {
    const authors = splitAuthors(author);
    // If we have a single author_id and only one author, use the ID-based link
    // Otherwise use name-based links for each author
    if (fic.author_id && authors.length === 1) {
      return [{ name: authors[0], href: `/authors/${fic.author_id}` }];
    }
    return authors.map(name => ({
      name,
      href: `/authors/${encodeURIComponent(name)}`
    }));
  }

  /** Direct download in the user's preferred format. */
  async function handleQuickDownload() {
    if (!fic.url_id || downloading) return;
    downloading = true;
    try {
      const res = await fetchExport(fic.url_id);
      if (res.err !== 0) {
        alert(res.msg || 'Download failed.');
        return;
      }
      // Get the URL for the preferred format (e.g. epub_url, mobi_url)
      const formatKey = `${preferredFormat}_url` as keyof typeof res;
      const formatUrl = res[formatKey] as string | null | undefined;
      if (formatUrl) {
        window.open(formatUrl, '_blank');
      } else {
        // Fallback to EPUB
        if (res.epub_url) {
          window.open(res.epub_url, '_blank');
        } else {
          alert('This format is not available yet. The fic may need to be processed first.');
        }
      }
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Download failed.');
    } finally {
      downloading = false;
    }
  }
</script>

<article class="work-blurb" bind:this={cardEl} class:work-blurb--deleted={deleted}>
  <!-- AO3-style header module: title, byline, fandoms, required-tags, date -->
  <div class="blurb-header module">
    <h4 class="blurb-heading">
      <span class="blurb-rating">{rating}</span>
      <a class="blurb-title" href={`/works/${fic.url_id}`}>{title}</a>
      {''}{''}
      by
      {#each authorLinks() as link, i}
        {#if i > 0}, {/if}
        <a class="blurb-author" href={link.href}>{link.name}</a>
      {/each}
    </h4>

    <!-- AO3-style symbol squares: rating / category / warning / completion -->
    <div class="symbol-squares-wrapper">
      <SymbolSquares
        rating={fic.rating}
        categories={fic.categories}
        warnings={fic.warnings}
        status={fic.status}
      />
    </div>

    {#if fandoms.length > 0}
      <h5 class="fandoms heading">
        <span class="landmark">Fandoms:</span>
        {#each fandoms as tag, i}
          {#if i > 0}, {/if}
          <a class="fandom-link" href={tagLink(tag)}>{tag.name}</a>
        {/each}
      </h5>
    {/if}

    <!-- Required tags symbols (rating shown as badge) -->
    {#if fic.warnings && Array.isArray(fic.warnings) && fic.warnings.length > 0}
      <span class="warning-symbols">
        {#each fic.warnings as w}
          <span class="symbol" title={w}>{w}</span>
        {/each}
      </span>
    {/if}

    <p class="blurb-datetime">
      {formatUpdated(fic.updated)}
      {#if lastVisited}
        · <span class="blurb-visited">Visited: {lastVisited}</span>
      {/if}
    </p>
  </div>

  <!-- Tags (comma-joined list, AO3-style) -->
  {#if ordered.length > 0}
    <h6 class="landmark heading">Tags</h6>
    <div class="tag-groups">
      {#each ordered as cat}
        {@const items = groupedTags[cat] ?? []}
        {#if items.length > 0}
          <section class="tag-group" aria-label={categoryLabel(cat)}>
            <h6 class="tag-category">{categoryLabel(cat)}</h6>
            <ul class="tags commas">
              {#each items as tag, i}
                {#if i > 0}, {/if}
                <li><a class="tag-link" href={tagLink(tag)}>{tag.name}</a></li>
              {/each}
            </ul>
          </section>
        {/if}
      {/each}
    </div>
  {/if}

  <!-- Summary -->
  {#if snippet}
    <h6 class="landmark heading">Summary</h6>
    <blockquote class="userstuff summary">
      {@html safeSnippet(snippet)}
    </blockquote>
  {/if}

  <!-- Series -->
  {#if fic.series && Array.isArray(fic.series) && fic.series.length > 0}
    <h6 class="landmark heading">Series</h6>
    <ul class="series">
      {#each fic.series as s}
        <li>
          <a href={`/series/${s.id ?? ''}`}>{s.name}</a>
          {#if s.position}{` (Part ${s.position})`}{/if}
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Stats (AO3-style dl.stats) -->
  <dl class="stats">
    {#if fic.language}
      <dt class="language">Language:</dt>
      <dd class="language">{fic.language}</dd>
    {/if}
    <dt class="words">Words:</dt>
    <dd class="words">{formatWords(fic.words)}</dd>
    <dt class="chapters">Chapters:</dt>
    <dd class="chapters">{chaptersDisplay(fic.chapters, fic.status)}</dd>
    {#if fic.collections_count}
      <dt class="collections">Collections:</dt>
      <dd class="collections">{fic.collections_count.toLocaleString()}</dd>
    {/if}
    {#if fic.comment_count}
      <dt class="comments">Comments:</dt>
      <dd class="comments">{fic.comment_count.toLocaleString()}</dd>
    {/if}
    {#if fic.kudos_count}
      <dt class="kudos">Kudos:</dt>
      <dd class="kudos">{fic.kudos_count.toLocaleString()}</dd>
    {/if}
    {#if fic.bookmarks_count}
      <dt class="bookmarks">Bookmarks:</dt>
      <dd class="bookmarks">{fic.bookmarks_count.toLocaleString()}</dd>
    {/if}
    {#if fic.hits}
      <dt class="hits">Hits:</dt>
      <dd class="hits">{fic.hits.toLocaleString()}</dd>
    {/if}
  </dl>

  <!-- Quick download button: downloads in preferred format directly -->
  {#if fic.url_id}
    <div class="blurb-actions">
      <button
        class="archive-btn blurb-download-btn"
        type="button"
        disabled={downloading}
        onclick={handleQuickDownload}
      >
        {#if downloading}
          <span class="spinner-inline"></span> Downloading…
        {:else}
          📥 Download {preferredFormat.toUpperCase()}
        {/if}
      </button>
    </div>
  {/if}
</article>

<style>
  .work-blurb {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 1em;
    margin-bottom: 0.75em;
    background: var(--archive-bg, #ffffff);
    font-family: Georgia, 'Times New Roman', serif;
  }

  /* ── Header module (title, byline, fandoms) ── */
  .blurb-header.module {
    margin-bottom: 0.4em;
  }

  .blurb-heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.2em;
    font-weight: 700;
    margin: 0;
    line-height: 1.4;
  }

  .blurb-rating {
    font-size: 0.8em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    background: var(--archive-bg-raised, #f5f5f5);
    padding: 0.15em 0.5em;
    border: 1px solid var(--archive-border, #dddddd);
    white-space: nowrap;
    line-height: 1.4;
    vertical-align: baseline;
    margin-right: 0.4em;
  }

  /* AO3: .blurb h4 a is #900 with NO underline; hover fades the text to
     #999 and drops the border (global rule). Byline/fandom author links are
     plain dark body links (#111, solid bottom border). */
  .blurb-title {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 700;
  }

  .blurb-title:hover {
    color: #999;
  }

  .blurb-author {
    color: #111;
    font-weight: normal;
  }

  .fandoms.heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    font-weight: 600;
    margin: 0.2em 0;
  }

  .fandoms.heading .landmark {
    color: var(--archive-muted, #666666);
  }

  /* AO3 renders blurb fandoms as tags: dotted underline -> red box, white
     text on hover. */
  .fandom-link {
    color: #111;
    line-height: 1.5;
    text-decoration: none;
    border-bottom: 1px dotted;
  }

  .fandom-link:hover {
    background: #900;
    color: #fff;
    border-bottom: 1px dotted;
  }

  .warning-symbols {
    display: inline-block;
    margin: 0.2em 0;
  }

  .symbol-squares-wrapper {
    margin: 0.2em 0;
  }

  .blurb-datetime {
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    margin: 0.1em 0 0;
  }
  /* ── Series list ── */
  .tags.commas + .series,
  .userstuff.summary + .series {
    list-style: none;
    margin: 0.2em 0 0.3em;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.3em;
    font-size: 0.92em;
  }

  .series li {
    margin: 0;
  }

  /* ── Tags grouped by AO3 category ── */
  .tag-groups {
    display: grid;
    gap: 0.35rem;
  }

  .tag-group { margin: 0; }

  .tag-category {
    display: inline;
    margin: 0 0.4rem 0 0;
    font-size: 0.9em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
  }

  .tags.commas {
    list-style: none;
    display: inline;
    margin: 0;
    padding: 0;
    font-size: 0.92em;
    line-height: 1.6;
  }

  .tags.commas li { display: inline; }

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

  /* ── Stats (dl.stats) ── */
  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6em 1.2em;
    margin: 0.4em 0;
    font-size: 0.88em;
  }

  .stats dt {
    font-weight: 600;
    color: var(--archive-muted, #666666);
  }

  .stats dd {
    font-weight: normal;
    color: var(--archive-text, #2a2a2a);
    margin: 0;
  }

  .work-blurb--deleted {
    display: none;
  }

  /* ── Quick download button ── */
  .blurb-actions {
    margin-top: 0.5em;
    padding-top: 0.5em;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .blurb-download-btn {
    font-size: 0.85em;
    padding: 0.2em 0.6em;
  }
  .spinner-inline {
    display: inline-block;
    width: 0.9em;
    height: 0.9em;
    border: 2px solid var(--archive-border, #dddddd);
    border-top-color: var(--archive-link, #990000);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    vertical-align: middle;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
