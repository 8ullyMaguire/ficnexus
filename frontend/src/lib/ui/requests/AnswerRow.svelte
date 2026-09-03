<script lang="ts">
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import ArchivistCall from './ArchivistCall.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import type { RequestAnswer } from '$lib/api/requests';
  import type { SearchFilters } from '$lib/api/search';
  import { buildSearchQuery } from '$lib/api/search';

  interface Props {
    a: RequestAnswer;
    myId: number | null;
    requestOwnerId: number;
    requestStatus: string;
    /** Callback for upvote/downvote: (a, vote: 1|-1|0) */
    onVote: (a: RequestAnswer, v: 1 | -1) => void;
    /** Callback for accept */
    onAccept: (a: RequestAnswer) => void;
    /** Callback for remove */
    onRemove: (a: RequestAnswer) => void;
    /** 'archive' | 'modern' — controls layout styling */
    uiMode?: string;
  }

  const {
    a,
    myId,
    requestOwnerId,
    requestStatus,
    onVote,
    onAccept,
    onRemove,
    uiMode = 'modern'
  }: Props = $props();

  const isArchive = $derived(uiMode === 'archive');
  const canAccept = $derived(
    requestOwnerId === myId && requestStatus === 'open' && !a.accepted
  );
  const canRemove = $derived(
    a.user_id === myId || myId === null || requestOwnerId === myId
  );

  function isLlmAnswer(a: RequestAnswer): boolean {
    return a.answer_kind === 'llm';
  }

  function isSearchAnswer(a: RequestAnswer): boolean {
    return a.answer_kind === 'search';
  }

  function isWorkAnswer(a: RequestAnswer): boolean {
    return a.answer_kind === 'work';
  }

  /** Build a /search deep link for a search-kind answer. The stored value
   *  may already be a full /search?... URL (archivist/llm path) or a raw query
   *  string (user search answers). */
  function searchHref(a: RequestAnswer): string {
    const raw = a.search_query ?? '';
    if (raw.startsWith('/search')) return raw;
    return `/search?${buildSearchQuery(searchQueryToFilters(raw))}`;
  }

  /** Parse a search_query string ("q terms +min_words:50000 complete:true")
   *  into SearchFilters so we can (re)build a canonical /search URL. Mirrors
   *  the boolean parser the backend uses. */
  function searchQueryToFilters(raw: string): SearchFilters {
    const filters: SearchFilters = {
      q: '',
      include_tags: '',
      exclude_tags: '',
      include_any_tags: '',
      exclude_tag_types: '',
      strict_gen: false,
      min_words: null,
      max_words: null,
      min_chapters: null,
      max_chapters: null,
      complete: null,
      source: '',
      date_from: '',
      date_to: '',
      sort: '',
      page: 1,
      per_page: 20,
      primary_tag: '',
      relationship_characters: '',
      min_comments: null,
      min_kudos: null,
      no_warnings: null,
      hide_read: false,
      hide_bookmarked: false,
      library_only: false,
      tag_ids: '',
    };
    const parts = raw.trim().split(/\s+/);
    const terms: string[] = [];
    for (const p of parts) {
      const m = /^([^:]+):(.+)$/.exec(p);
      if (m) {
        const field = m[1].toLowerCase();
        const val = m[2];
        switch (field) {
          case 'q': terms.push(val); break;
          case 'min_words': filters.min_words = Number(val) || null; break;
          case 'max_words': filters.max_words = Number(val) || null; break;
          case 'min_chapters': filters.min_chapters = Number(val) || null; break;
          case 'max_chapters': filters.max_chapters = Number(val) || null; break;
          case 'complete': filters.complete = val === 'true'; break;
          case 'source': filters.source = val; break;
          case 'sort': filters.sort = val; break;
          case 'include_tags': filters.include_tags = val; break;
          case 'exclude_tags': filters.exclude_tags = val; break;
          default: terms.push(p); break;
        }
      } else {
        terms.push(p);
      }
    }
    filters.q = terms.join(' ');
    return filters;
  }

  /** Build a SearchResult-shaped object from the answer's work_id/fic data so
   *  we can feed it into the reusable WorkBlurb component. */
  function workForResult(a: RequestAnswer): { url_id: string; title: string | null; author: string | null } {
    return {
      url_id: a.work_id ? String(a.work_id) : (a.fic_title || ''),
      title: a.fic_title,
      author: a.fic_author,
    };
  }
</script>

{#if isArchive}
  <li class="archive-answer-row">
    <div class="archive-answer-votes">
      <button
        class="archive-vote-btn"
        class:active={a.my_vote === 1}
        onclick={() => onVote(a, 1)}
        title="Fits the prompt"
        >▲</button
      >
      <span>{a.score}</span>
      <button
        class="archive-vote-btn"
        class:active={a.my_vote === -1}
        onclick={() => onVote(a, -1)}
        title="Doesn't fit"
      >▼</button>
    </div>
    <div class="archive-answer-body">
      {#if isLlmAnswer(a)}
        <span class="archivist-attribution">FicNexus Archivist</span>
        {#if a.search_query}
          <ArchivistCall body={a.search_query} />
        {/if}
        {#if a.pitch}<p class="archive-pitch">{a.pitch}</p>{/if}
      {:else if isSearchAnswer(a)}
        <a
          class="archive-search-chip"
          href={searchHref(a)}
        >
          Try this search: {a.search_query}
        </a>
      {:else}
        <a
          class="archive-fic-link"
          href={`/works/${a.work_id ?? encodeURIComponent(a.fic_title)}`}
        >{a.fic_title}</a>
        <span class="archive-byline">by {a.fic_author}</span>
      {/if}
      {#if a.accepted}
        <span class="archive-accepted-tag">{t('requests.accepted')}</span>
      {/if}
      {#if a.pitch && !isLlmAnswer(a)}
        <p class="archive-pitch">{a.pitch}</p>
      {/if}
      <p class="archive-muted">
        {a.username} &middot; {a.created_at.slice(0, 10)}
      </p>
      <div class="archive-answer-actions">
        {#if canAccept}
          <ArchiveButton onclick={() => onAccept(a)}>{t('requests.accept')}</ArchiveButton>
        {/if}
        {#if canRemove}
          <ArchiveButton onclick={() => onRemove(a)}>{t('requests.remove')}</ArchiveButton>
        {/if}
      </div>
    </div>
  </li>
{:else}
  <li class="card answer-card" class:accepted={a.accepted}>
    <div class="vote-col">
      <button
        class="vote-btn"
        class:active={a.my_vote === 1}
        onclick={() => onVote(a, 1)}
        title="Fits the prompt"
        aria-label="Upvote"
      >▲</button>
      <span class="score">{a.score}</span>
      <button
        class="vote-btn"
        class:active={a.my_vote === -1}
        onclick={() => onVote(a, -1)}
        title="Doesn't fit"
        aria-label="Downvote"
      >▼</button>
    </div>
    <div class="answer-body">
      {#if isLlmAnswer(a)}
        <ArchivistCall body={a.search_query ?? a.pitch} />
      {:else if isSearchAnswer(a)}
        <a href={searchHref(a)} class="search-chip">
          Try this search
        </a>
        <span class="search-query">{a.search_query}</span>
      {:else if isWorkAnswer(a)}
        <div class="fic-line">
          <a href={`/works/${a.work_id ?? encodeURIComponent(a.fic_title)}`} class="fic-title">{a.fic_title}</a>
          <span class="author">by {a.fic_author}</span>
          {#if a.accepted}
            <span class="accepted-tag">{t('requests.accepted')}</span>
          {/if}
        </div>
      {/if}
      {#if a.pitch && !isLlmAnswer(a) && !isSearchAnswer(a)}
        <p class="pitch">"{a.pitch}"</p>
      {/if}
      <p class="meta">
        {a.username} &middot; {a.created_at.slice(0, 10)}
      </p>
      {#if canAccept}
        <button class="btn btn-small" onclick={() => onAccept(a)}>{t('requests.accept')}</button>
      {/if}
      {#if canRemove}
        <button
          class="btn btn-small btn-danger"
          onclick={() => onRemove(a)}
        >{t('requests.remove')}</button>
      {/if}
    </div>
  </li>
{/if}

<style>
  /* ── LLM Archivist answer (modern) ── */
  .llm-answer {
    border: 1px solid var(--color-border, #ddd);
    background: var(--color-surface, #fafafa);
    border-radius: 8px;
    padding: 0.75rem 1rem;
    margin-bottom: 0.5rem;
  }
  .archivist-attribution {
    font-weight: 700;
    color: var(--color-link, #2b4bd7);
    display: block;
    margin-bottom: 0.2rem;
  }
  .archivist-label {
    font-size: 0.75rem;
    color: var(--color-text-muted, #888);
    background: var(--color-bg, #fff);
    border: 1px solid var(--color-border, #ddd);
    border-radius: 999px;
    padding: 0.1rem 0.5rem;
    display: inline-block;
    margin-bottom: 0.4rem;
  }
  .archivist-body {
    color: var(--color-text, #444);
    line-height: 1.5;
    margin: 0;
  }

  /* ── Search answer chip (modern) ── */
  .search-chip {
    display: inline-block;
    border: 1px solid var(--color-border, #ddd);
    background: var(--color-surface, #fafafa);
    border-radius: 999px;
    padding: 0.2rem 0.7rem;
    font-size: 0.85rem;
    color: var(--color-link, #2b4bd7);
    text-decoration: none;
    margin-bottom: 0.3rem;
  }
  .search-query {
    display: block;
    color: var(--color-text-muted, #888);
    font-size: 0.9rem;
    margin-bottom: 0.4rem;
  }

  /* ── Archive mode LLM / search answers ── */
  .archive-llm-answer {
    margin: 0.3em 0 0.6em;
  }
  .archive-llm-attribution {
    font-weight: 700;
    color: var(--archive-link, #990000);
    display: block;
    margin-bottom: 0.2em;
  }
  .archive-llm-label {
    font-size: 0.75em;
    font-style: italic;
    color: var(--archive-muted, #666);
    display: block;
    margin-bottom: 0.3em;
  }
  .archive-search-chip {
    display: inline-block;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
    text-decoration: none;
    border-radius: 999px;
    padding: 0.2em 0.7em;
    font-size: 0.9em;
    margin-bottom: 0.3em;
  }
</style>
