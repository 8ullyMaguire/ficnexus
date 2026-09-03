<script lang="ts">
  import { auth } from '$lib/stores/auth.svelte';
  import { mapRating, groupTags, categoryLabel, formatWords, chaptersDisplay, formatUpdated } from './rating.js';
  import ArchiveButton from './ArchiveButton.svelte';
  import { setPref } from '$lib/prefs';
  import type { ExportResponse, FicMeta } from '$lib/api/types';

  interface Tag {
    category?: number | string;
    name: string;
    [key: string]: unknown;
  }

  let {
    fic,
    isBookmarked = false,
    onToggleBookmark,
    savingBookmark = false,
  }: {
    fic: ExportResponse;
    isBookmarked?: boolean;
    onToggleBookmark?: () => void;
    savingBookmark?: boolean;
  } = $props();

  const meta: FicMeta | undefined = $derived(fic.meta);
  const m = $derived(meta!);

  // Extended metadata that may be present on API response but not on base type
  const lang = $derived((fic as unknown as Record<string, unknown>).language as string | undefined);
  const seriesName = $derived((fic as unknown as Record<string, unknown>).series as string | undefined);
  const ratingRaw = $derived((fic.meta as unknown as Record<string, unknown> | undefined)?.rating as string | undefined);

  // Rating may also ride in extended-meta blobs when scrapers provide it.
  const extMetaRating = $derived.by(() => {
    for (const key of ['raw_extended_meta', 'extra_meta'] as const) {
      const blob = fic.meta?.[key];
      if (blob && typeof blob === 'object') {
        const r = (blob as Record<string, unknown>).rating;
        if (typeof r === 'string' && r) return r;
      }
    }
    return undefined;
  });

  // --- Derived metadata ---
  const title = $derived(m.title ?? 'Untitled');
  const author = $derived(m.author ?? 'Anonymous');
  const ratingDisplay = $derived(mapRating(extMetaRating ?? ratingRaw ?? null));
  const createdDate = $derived(m.created ? new Date(m.created).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' }) : '');
  const updatedDate = $derived(m.updated ? new Date(m.updated).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' }) : '');
  const isLoggedIn = $derived(auth.isLoggedIn);

  // --- Sources (merged from multiple sites) ---
  const sources = $derived((fic as unknown as Record<string, unknown>).sources as Array<{
    url_id: string;
    site: string;
    source_url: string;
    words: number;
    chapters: number;
  }> | undefined);

  // --- Tags grouped by category ---
  const tags = $derived.by(() => {
    // New API: tags is already grouped { fandom: [...], character: [...], ... }
    const grouped = fic.tags;
    if (grouped && typeof grouped === 'object' && 'fandom' in grouped) {
      // Convert grouped format to the record format expected by the template
      const result: Record<string, Tag[]> = {};
      for (const [key, value] of Object.entries(grouped)) {
        if (Array.isArray(value) && value.length > 0) {
          result[key] = value;
        }
      }
      return result;
    }
    // Legacy fallback: flat tag array
    const raw = (fic as unknown as Record<string, unknown>).tags as Tag[] | undefined;
    if (!raw) return {} as Record<string, Tag[]>;
    return groupTags(raw);
  });

  // --- Download formats ---
  let downloads = $derived.by(() => {
    const out: { label: string; type: string; href?: string | null; lazy?: boolean }[] = [];
    const add = (label: string, type: string, href?: string | null, lazy = false) => {
      if (href) out.push({ label, type, href });
      else if (lazy) out.push({ label, type, href: null, lazy: true });
    };
    add('EPUB', 'epub', fic.epub_url);
    add('HTML', 'html', fic.html_url);
    add('TXT', 'txt', fic.txt_url);
    add('MD', 'md', fic.md_url);
    add('MOBI', 'mobi', fic.mobi_url, true);
    add('PDF', 'pdf', fic.pdf_url, true);
    add('AZW3', 'azw3', fic.azw3_url, true);
    add('DOCX', 'docx', fic.docx_url);
    add('FB2', 'fb2', fic.fb2_url);
    add('KEPUB', 'kepub', fic.kepub_url);
    return out;
  });

  let showDownloadMenu = $state(false);
  let selectedFormat: string = $state('epub');

  // --- Chapters list ---
  const chaptersCount = $derived(m.chapters ?? 1);
  const statusLower = $derived((m.status ?? '').toLowerCase());
  const isComplete = $derived(statusLower === 'complete' || statusLower === 'completed');

  // --- Tag link helper ---
  function tagLink(tag: Tag): string {
    const typeId = String(tag.category ?? '4');
    return `/search?include_tags=${typeId}:${encodeURIComponent(tag.name)}`;
  }

  /** Split a comma-joined author string into individual author names. */
  function splitAuthors(authorStr: string): string[] {
    if (!authorStr) return ['Anonymous'];
    return authorStr
      .split(/, | and /)
      .map(a => a.trim())
      .filter(a => a.length > 0);
  }

  /** Generate author links for display. */
  function authorLinks(): Array<{ name: string; href: string }> {
    const authors = splitAuthors(author);
    // Use author_id if available and only one author
    if (m.author_id && authors.length === 1) {
      return [{ name: authors[0], href: `/authors/${m.author_id}` }];
    }
    return authors.map(name => ({
      name,
      href: `/authors/${encodeURIComponent(name)}`
    }));
  }

  // --- Meta label helpers ---
  // AO3 work-page row order: Rating, Warnings, Category, Fandoms,
  // Relationships, Characters, Additional Tags — then the Stats row.
  const META_ROW_ORDER = ['5', '6', '1', '3', '2', '4'];

  const ROW_LABELS: Record<string, string> = {
    '5': 'Archive Warnings',
    '6': 'Category',
    '1': 'Fandoms',
    '3': 'Relationships',
    '2': 'Characters',
    '4': 'Additional Tags',
  };

  function categoryCss(cat: string): string {
    const cssMap: Record<string, string> = {
      '1': 'fandoms',
      '2': 'characters',
      '3': 'relationships',
      '4': 'freeforms',
      '5': 'warnings',
      '6': 'categories',
    };
    return cssMap[cat] ?? 'tags';
  }
</script>

<article class="archive-work">
  <!-- AO3 work header: title + byline -->
  <h2 class="work-title">{title}</h2>
  <p class="byline">
    by {#each authorLinks() as link, i}
      {#if i > 0}, {/if}
      <a class="author-link" href={link.href}>{link.name}</a>
    {/each}
  </p>
  <!-- Source chips: show which sites this fic was scraped from -->
  {#if sources && sources.length > 1}
    <div class="source-chips">
      <span class="source-label">Sources:</span>
      {#each sources as source}
        <a class="source-chip" href={source.source_url} title="{source.words.toLocaleString()} words, {source.chapters} chapters">
          {source.site}
        </a>
      {/each}
    </div>
  {/if}
  <p class="work-date">
    {#if createdDate}
      <em>Added:</em> {createdDate}
    {/if}
    {#if updatedDate && updatedDate !== createdDate}
      · <em>Updated:</em> {updatedDate}
    {/if}
  </p>

  <!-- Chapter navigation (AO3 work header navigation) -->
  <h3 class="landmark heading">Actions</h3>
  <ul class="work-navigation actions">
    <li class="chapter entire">
      <a href={`/read/${m.id}?view_full_work=true`}>Entire Work</a>
    </li>
    <li class="chapter bychapter">
      <a href={`/read/${m.id}`}>Chapter by Chapter</a>
    </li>
    {#if isLoggedIn}
      <li class="bookmark">
        <a href="#bookmark-form">
          {#if isBookmarked}Edit Bookmark{:else}Bookmark{/if}
        </a>
      </li>
      <li class="mark">
        <a href="#">Mark for Later</a>
      </li>
    {/if}
    <li class="comments">
      <a href="#comments">Comments</a>
    </li>
  </ul>

  <!-- Work meta definition list (AO3 dl.work.meta fact pattern:
       Rating / Archive Warnings / Category / Fandoms / Relationships /
       Characters / Additional Tags rows, then the Stats row) -->
  <h3 class="landmark heading">Work Header</h3>
  <dl class="work meta group">
    <dt class="rating tags">Rating:</dt>
    <dd class="rating tags">{ratingDisplay}</dd>

    {#each META_ROW_ORDER as cat}
      {@const items = tags[cat] ?? []}
      {#if items.length > 0}
        <dt class="{categoryCss(cat)} tags">
          {ROW_LABELS[cat] ?? categoryLabel(cat)}:
        </dt>
        <dd class="{categoryCss(cat)} tags">
          <ul class="commas">
            {#each items as tag}
              <li>
                <a href={tagLink(tag)}>{tag.name}</a>
              </li>
            {/each}
          </ul>
        </dd>
      {/if}
    {/each}

    {#if lang}
      <dt class="language">Language:</dt>
      <dd class="language">{lang}</dd>
    {/if}

    {#if seriesName}
      <dt class="series">Series:</dt>
      <dd class="series">{seriesName}</dd>
    {/if}

    <dt class="stats">Stats:</dt>
    <dd class="stats">
      <dl class="stats">
        <dt class="words">Words:</dt>
        <dd class="words">{formatWords(m.words)}</dd>
        <dt class="chapters">Chapters:</dt>
        <dd class="chapters">{chaptersDisplay(m.chapters, m.status)}</dd>
        <dt class="status">Status:</dt>
        <dd class="status">{m.status || 'Unknown'}</dd>
        <dt class="updated">Updated:</dt>
        <dd class="updated">{updatedDate || '—'}</dd>

        <!-- Social metrics (AO3 shows these in dl.stats) -->
        {#if fic.notes && fic.notes.length > 0}
          <dt class="collections">Notes:</dt>
          <dd class="collections">{fic.notes.length} internal note(s)</dd>
        {/if}
      </dl>
    </dd>
  </dl>

  <!-- Summary -->
  <div class="work-summary">
    <h3 class="section-heading">Summary</h3>
    {#if m.description?.trim()}
      <div class="summary-text">{@html m.description}</div>
    {:else}
      <p class="summary-empty"><em>No summary provided.</em></p>
    {/if}
  </div>

  <!-- Action Buttons -->
  <div class="work-actions">
    <a class="action-btn action-read" href={`/read/${encodeURIComponent(m.id)}`}>
      Read Online
    </a>

    <!-- Download dropdown -->
    {#if downloads.length > 0}
      <div class="download-wrapper">
        <button
          class="action-btn action-download"
          onclick={() => (showDownloadMenu = !showDownloadMenu)}
        >
          Download ▾
        </button>
        {#if showDownloadMenu}
          <div class="download-menu">
            {#each downloads as dl}
              {#if dl.href}
                <a class="download-item" href={dl.href} download onclick={() => { showDownloadMenu = false; setPref('defaultFormat', dl.type); }}>
                  {dl.label}
                </a>
              {:else}
                <span class="download-item download-unavailable" title="Conversion not yet available">
                  {dl.label} (unavailable)
                </span>
              {/if}
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Bookmark -->
    {#if isLoggedIn && onToggleBookmark}
      <form id="bookmark-form" class="bookmark-form-inline">
        <button
          class="action-btn action-bookmark"
          class:bookmarked={isBookmarked}
          onclick={onToggleBookmark}
          disabled={savingBookmark}
        >
          {#if savingBookmark}
            Saving…
          {:else if isBookmarked}
            Bookmarked
          {:else}
            Bookmark
          {/if}
        </button>
      </form>
    {:else if isLoggedIn}
      <button class="action-btn action-bookmark" disabled>Bookmark</button>
    {:else}
      <a class="action-btn action-bookmark" href="/">Log In to Bookmark</a>
    {/if}
  </div>

  <!-- Chapters -->
  <div class="work-chapters">
    <h3 class="section-heading">
      Chapters ({chaptersCount}{isComplete ? `/${chaptersCount}` : '/?'})
    </h3>
    <ol class="chapter-list">
      {#each Array.from({ length: chaptersCount }, (_, i) => i + 1) as chNum}
        <li class="chapter-item">
          <a class="chapter-link" href={`/read/${encodeURIComponent(m.id)}?chapter=${chNum}`}>
            Chapter {chNum}
          </a>
        </li>
      {/each}
    </ol>
  </div>

  <!-- Notes (if any) -->
  {#if fic.notes && fic.notes.length > 0}
    <div class="work-notes">
      {#each fic.notes as note}
        <p class="note-text">ℹ️ {note}</p>
      {/each}
    </div>
  {/if}
</article>

<style>
  .archive-work {
    max-width: var(--archive-max-width, 800px);
    margin: 0 auto;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
    line-height: 1.6;
  }

  .work-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.7em;
    font-weight: normal;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.3em;
  }

  .byline {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    margin: 0 0 0.1em;
  }

  /* AO3 byline: dark body link, not red — only work titles are #900 */
  .author-link {
    color: #111;
    text-decoration: none;
  }

  .author-link:hover {
    color: #999;
  }

  .work-date {
    font-size: 0.88em;
    color: var(--archive-muted, #666666);
    margin: 0 0 0.8em;
  }

  .work-date em {
    font-style: normal;
    font-weight: 600;
  }

  .landmark.heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin: 1em 0 0.3em;
  }

  /* ── Work navigation ── */
  .work-navigation.actions {
    list-style: none;
    margin: 0.4em 0 0.8em;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.8em 1.2em;
    font-size: 0.88em;
  }

  .work-navigation.actions li a {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .work-navigation.actions li a:hover {
    color: #999;
  }

  /* ── Work meta (dl.work.meta) ── */
  .work.meta.group {
    margin: 0.6em 0 1em;
    font-size: 0.9em;
  }

  .work.meta.group dt {
    font-weight: 600;
    color: var(--archive-muted, #666666);
    margin-top: 0.4em;
  }

  .work.meta.group dd {
    margin: 0 0 0.3em 0;
    color: var(--archive-text, #2a2a2a);
  }

  .work.meta.group dd ul.commas {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0 0.3em;
  }

  .work.meta.group dd ul.commas li {
    display: inline;
  }

  .work.meta.group dd ul.commas a {
    color: #111;
    text-decoration: none;
    border-bottom: 1px dotted;
  }

  /* AO3 meta-list tag links: dotted -> red box (a.tag contract) */
  .work.meta.group dd ul.commas a:hover {
    background: #900;
    color: #fff;
  }

  /* ── Summary ── */
  .work-summary {
    margin-bottom: 1em;
  }

  .section-heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.05em;
    font-weight: normal;
    color: var(--archive-text, #2a2a2a);
    border-bottom: 1px solid var(--archive-border, #dddddd);
    margin: 1.2em 0 0.4em;
    padding-bottom: 0.2em;
  }

  .summary-text {
    font-size: 0.92em;
    line-height: 1.65;
    color: var(--archive-text, #2a2a2a);
  }

  .summary-text :global(p) {
    margin: 0 0 0.6em;
  }

  .summary-text :global(a) {
    color: #111;
    text-decoration: none;
  }

  .summary-text :global(a:hover) {
    color: #999;
  }

  .summary-empty {
    color: var(--archive-muted, #666666);
    font-style: italic;
    font-size: 0.92em;
  }

  /* ── Actions ── */
  .work-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5em;
    margin: 1em 0;
  }

  .action-btn {
    display: inline-block;
    padding: 0.3em 0.9em;
    font-family: inherit;
    font-size: 0.9em;
    font-weight: 600;
    line-height: 1.5;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    text-decoration: none;
    transition: background 0.15s, border-color 0.15s;
  }

  .action-btn:hover {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
    text-decoration: none;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-read {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }

  .action-read:hover {
    background: var(--archive-link-visited, #660066);
    border-color: var(--archive-link-visited, #660066);
  }

  .action-bookmark.bookmarked {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }

  /* ── Download dropdown ── */
  .download-wrapper {
    position: relative;
    display: inline-block;
  }

  .download-menu {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 10;
    min-width: 140px;
    background: var(--archive-bg, #ffffff);
    border: 1px solid var(--archive-border, #dddddd);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
    margin-top: -1px;
  }

  .download-item {
    display: block;
    padding: 0.35em 0.7em;
    font-size: 0.88em;
    color: var(--archive-text, #2a2a2a);
    text-decoration: none;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    cursor: pointer;
  }

  .download-item:last-child {
    border-bottom: none;
  }

  .download-item:hover {
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .download-unavailable {
    color: var(--archive-muted, #666666);
    font-style: italic;
    cursor: default;
  }

  .download-unavailable:hover {
    background: transparent;
  }

  /* ── Chapters ── */
  .work-chapters {
    margin: 1.2em 0;
  }

  .chapter-list {
    margin: 0;
    padding-left: 1.5em;
    font-size: 0.92em;
  }

  .chapter-item {
    margin-bottom: 0.25em;
  }

  .chapter-link {
    color: #111;
    text-decoration: none;
  }

  .chapter-link:hover {
    color: #999;
  }

  /* ── Notes ── */
  .work-notes {
    margin: 1em 0;
    padding: 0.5em 0.8em;
    background: var(--archive-bg-raised, #f5f5f5);
    border-left: 3px solid var(--archive-border, #dddddd);
    font-size: 0.88em;
  }

  .note-text {
    margin: 0.2em 0;
    color: var(--archive-muted, #666666);
  }

  /* ── Bookmark form inline ── */
  .bookmark-form-inline {
    display: contents;
  }

  /* ── Source chips ── */
  .source-chips {
    display: flex;
    align-items: center;
    gap: 0.4em;
    margin: 0.2em 0 0.5em;
    flex-wrap: wrap;
  }
  .source-label {
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    font-weight: 600;
  }
  .source-chip {
    display: inline-block;
    padding: 0.15em 0.5em;
    font-size: 0.8em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 3px;
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-family: 'Lucida Grande', 'Trebuchet MS', Helvetica, Verdana, sans-serif;
  }
  .source-chip:hover {
    background: var(--archive-border, #dddddd);
    color: var(--archive-heading, #990000);
  }
</style>
