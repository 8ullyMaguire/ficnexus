<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { askArchive, describeAppliedParams, type AskResponse } from '$lib/api/ask';
  import { createRequest, getRequest, type RequestAnswer, type RequestDetail } from '$lib/api/requests';
  import { formatWords, detectSite, stripHtml } from '$lib/util';
  import { TAG_TYPES, type SearchResult } from '$lib/api/search';
  import { getPref } from '$lib/prefs';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import ArchivistCall from '$lib/ui/requests/ArchivistCall.svelte';
  import AnswerRow from '$lib/ui/requests/AnswerRow.svelte';

  const uiMode = $derived(getPref('uiMode'));

  // URL: /ask?q=request:{id}  — open a request thread focused.
  const q = $derived($page.url.searchParams.get('q') ?? '');
  const isRequestRef = $derived(q.startsWith('request:'));
  const requestId = $derived(isRequestRef ? Number(q.slice('request:'.length)) : null);

  // Ask bar state
  let query = $state('');
  let loading = $state(false);
  let error = $state('');
  let searched = $state(false);
  let result: AskResponse | null = $state(null);

  // Auto-persisted request state (human-answers section)
  let persistedRequestId: number | null = $state(null);
  let persistedDetail: RequestDetail | null = $state(null);
  let requestAnswers: RequestAnswer[] = $state([]);
  let requestLoading = $state(false);
  let requestError = $state('');
  let persisted = $state(false);

  let unsubscribe: (() => void) | null = null;

  onMount(() => {
    // If we arrived via ?q=request:{id}, load that request's answers instead of
    // showing the ask bar.
    if (requestId) {
      loadRequest(requestId);
    }
    // Seed the ask box from ?q=<text> if present and not a request ref.
    if (!isRequestRef && q) {
      query = q;
    }
    // React to URL changes (back/forward) for the request:{id} shim.
    unsubscribe = page.subscribe((p) => {
      const newQ = p.url.searchParams.get('q') ?? '';
      const newIsReq = newQ.startsWith('request:');
      const newId = newIsReq ? Number(newQ.slice('request:'.length)) : null;
      if (newId && newId !== requestId && newId !== persistedRequestId) {
        loadRequest(newId);
      }
    });
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });

  async function loadRequest(id: number) {
    requestLoading = true;
    requestError = '';
    requestAnswers = [];
    try {
      const res = await getRequest(id);
      if (res.err === 0) {
        persistedDetail = res.request;
        requestAnswers = res.answers;
        persistedRequestId = id;
      } else {
        requestError = (res as { msg?: string }).msg ?? 'Request not found';
      }
    } catch {
      requestError = 'Failed to load request';
    } finally {
      requestLoading = false;
    }
  }

  /** Submit the ask, then auto-persist the question as a request so the
   *  human-answers section appears below. */
  async function submit() {
    const nl = query.trim();
    if (!nl || loading) return;
    loading = true;
    error = '';
    searched = false;
    result = null;
    persisted = false;
    try {
      result = await askArchive(nl);
      searched = true;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ask failed';
    } finally {
      loading = false;
    }
  }

  /** After a successful ask, turn the NL query into a fic request and attach
   *  the LLM answer (search_url) as an `llm`-kind answer. */
  async function persistAsRequest() {
    if (!result || persistedRequestId) return;
    const nl = result.nl_query || query.trim();
    try {
      const res = await createRequest({ title: nl, body: '' });
      if (res.err === 0 && res.id) {
        persistedRequestId = res.id;
        // Build the llm answer body from the applied search link.
        const searchUrl = buildSearchUrlFromParams(result.applied_params);
        requestAnswers = [
          {
            id: 0,
            user_id: 0,
            username: 'FicNexus Archivist',
            work_id: null,
            fic_title: '',
            fic_author: '',
            pitch: '',
            source: 'ask',
            created_at: new Date().toISOString(),
            score: 0,
            my_vote: null,
            accepted: false,
            answer_kind: 'llm',
            search_query: searchUrl,
            payload: null,
          },
        ];
        persisted = true;
        // Reflect the new request id in the URL so the bookmark is shareable.
        goto(`?q=request:${res.id}`, { replaceState: true });
      }
    } catch {
      // Non-fatal — the ask results are still visible.
    }
  }

  /** Build a /search?… URL from the v2 query string the archivist ran. */
  function buildSearchUrlFromParams(v2Query: string): string {
    return `/search?q=${encodeURIComponent(v2Query)}`;
  }

  /** Applied-params description for the "Interpreted as" chip. */
  function chipText(): string {
    if (!result) return '';
    return describeAppliedParams(result.applied_params);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  // Auto-persist after a successful ask (LLM auto-answer).
  $effect(() => {
    if (searched && result && !persisted) {
      void persistAsRequest();
    }
  });
</script>

<svelte:head>
  <title>Ask the Archive — FicNexus</title>
</svelte:head>

{#if uiMode === 'archive'}
<div class="ask-page archive-mode">
  <h2 class="archive-heading">Ask the Archive</h2>
  <p class="archive-subtitle">
    Describe what you are looking for in plain English and the archive
    will try to find it.
  </p>

  <div class="archive-ask-bar">
    <textarea
      rows="2"
      placeholder="e.g. dark harry potter completed over 50k"
      bind:value={query}
      onkeydown={onKeydown}
      aria-label="Natural language search query"
    ></textarea>
    <ArchiveButton onclick={submit} disabled={loading || !query.trim()}>
      {loading ? 'Asking…' : 'Ask'}
    </ArchiveButton>
  </div>

  {#if error}
    <div class="archive-note archive-error">{error}</div>
  {/if}

  {#if loading}
    <div class="archive-loading">Asking the archive…</div>
  {/if}

  {#if searched && result}
    {#if result.total === 0}
      <div class="archive-note archive-request">
        <p>
          No results found. Turn this ask into a fic request —
          the community can suggest works for it.
        </p>
        <ArchiveButton href={`/requests/new?q=${encodeURIComponent(query.trim())}`}>
          Turn this into a request
        </ArchiveButton>
      </div>
    {/if}

    {#if chipText()}
      <div class="archive-interpreted">
        <span class="archive-chip">
          Interpreted as: {chipText()}
        </span>
        {#if !result.translated}
          <span class="archive-chip archive-chip--plain">
            plain search (no translation)
          </span>
        {/if}
      </div>
    {/if}

    <div class="archive-results-count">
      {result.total.toLocaleString()} result{result.total !== 1 ? 's' : ''}
    </div>

    {#if result.results.length > 0}
      <div class="archive-results">
        {#each result.results as r}
          <WorkBlurb fic={r as any} />
        {/each}
      </div>
    {:else}
      <p class="archive-note">No results found. Try a different ask.</p>
    {/if}
  {/if}

  <!-- LLM auto-answer (archivist) call -->
  {#if result && persisted}
    <ArchivistCall body={buildSearchUrlFromParams(result.applied_params)} />
  {/if}

  <!-- Human-answers section -->
  {#if persistedRequestId}
    <section class="archive-section archive-human-answers">
      <h2>{requestAnswers.length} community answer{requestAnswers.length !== 1 ? 's' : ''}</h2>
      {#if requestLoading}
        <p class="archive-muted">Loading community answers…</p>
      {:else if requestError}
        <p class="archive-note archive-error">{requestError}</p>
      {:else if requestAnswers.length === 0}
        <p class="archive-muted">Be the first to answer this request.</p>
      {:else}
        <ul class="archive-answer-list">
          {#each requestAnswers as a (a.id)}
            <AnswerRow a={a} myId={null} requestOwnerId={0} requestStatus="open" onVote={() => {}} onAccept={() => {}} onRemove={() => {}} uiMode="archive" />
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>
{:else}
<div class="ask-page">
  <h1>Ask the Archive</h1>
  <p class="subtitle">
    Describe what you're looking for in plain English — "dark harry potter
    completed over 50k" — and we'll turn it into a real search.
  </p>

  <div class="ask-bar">
    <textarea
      rows="2"
      placeholder="e.g. dark harry potter completed over 50k"
      bind:value={query}
      onkeydown={onKeydown}
      aria-label="Natural language search query"
    ></textarea>
    <button class="btn" onclick={submit} disabled={loading || !query.trim()}>
      {loading ? 'Asking…' : 'Ask'}
    </button>
  </div>

  {#if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {/if}

  {#if loading}
    <div class="loading"><span class="spinner"></span> Asking the archive…</div>
  {/if}

  {#if searched && result}
  <!-- "Turn into a request": when the ask came up empty (or wasn't
       satisfying), offer to post it as a fic request so the community
       can answer it. -->
  {#if result.total === 0}
    <div class="card request-card">
      <p class="request-card-text">
        <strong>No results found.</strong> Turn this ask into a fic request —
        the community can suggest works for it.
      </p>
      <a class="btn" href={`/requests/new?q=${encodeURIComponent(query.trim())}`}>
        ➕ Turn this into a request
      </a>
    </div>
  {/if}
  <!-- "Interpreted as" chip: shown whenever a translation (or a plain
       fallback) produced applied params. -->
    {#if chipText()}
      <div class="interpreted-row">
        <span class="chip interpreted-chip" title="Filters applied to this search">
          <span class="chip-label">Interpreted as:</span> {chipText()}
        </span>
        {#if !result.translated}
          <span class="chip plain-chip" title="Ollama translation unavailable — searched the raw text">
            plain search (no translation)
          </span>
        {/if}
      </div>
    {/if}

    <div class="results-header">
      <span>{result.total.toLocaleString()} result{result.total !== 1 ? 's' : ''}</span>
    </div>

    {#if result.results.length > 0}
      <div class="results">
        {#each result.results as r}
          <div class="card result-card">
            <h3><a href="/works/{r.url_id}">{r.title}</a></h3>
            <p class="muted">by {r.author} · <span class="tag">{detectSite(r.source)}</span></p>
            <p class="meta">
              {formatWords(r.words)} words · {r.chapters} ch · {r.status}
              {#if r.comment_count > 0} · 💬 {r.comment_count}{/if}
              {#if r.kudos_count > 0} · 👍 {r.kudos_count}{/if}
            </p>
            {#if r.snippet}
              <p class="desc snippet">{@html r.snippet}</p>
            {:else if r.description}
              <p class="desc">{stripHtml(r.description).slice(0, 200)}{r.description.length > 200 ? '…' : ''}</p>
            {/if}
            {#if r.tags.length > 0}
              <div class="tags">
                {#each r.tags as t}
                  <span class="tag-chip" title="{TAG_TYPES[t.type_id as keyof typeof TAG_TYPES] || t.type}: {t.name}">
                    {t.name}
                  </span>
                {/each}
                {#if r.total_freeform > 5}
                  <span class="tag-more">+{r.total_freeform - 5} more</span>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <p class="muted no-results">No results found. Try a different ask.</p>
    {/if}
  {/if}

  <!-- LLM auto-answer (archivist) call -->
  {#if result && persisted}
    <ArchivistCall body={buildSearchUrlFromParams(result.applied_params)} />
  {/if}

  <!-- Human-answers section -->
  {#if persistedRequestId}
    <section class="answers-section">
      <h2>{requestAnswers.length} community answer{requestAnswers.length !== 1 ? 's' : ''}</h2>
      {#if requestLoading}
        <p class="muted">Loading community answers…</p>
      {:else if requestError}
        <div class="card error-card"><strong class="error-text">⚠️ {requestError}</strong></div>
      {:else if requestAnswers.length === 0}
        <p class="muted">Be the first to answer this request.</p>
      {:else}
        <ul class="answer-list">
          {#each requestAnswers as a (a.id)}
            <AnswerRow a={a} myId={null} requestOwnerId={0} requestStatus="open" onVote={() => {}} onAccept={() => {}} onRemove={() => {}} uiMode={uiMode} />
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>
{/if}

<style>
  .ask-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
  }
  .subtitle {
    margin-top: -0.5rem;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }
  .ask-bar {
    display: flex;
    gap: 0.6rem;
    margin-bottom: 0.8rem;
    align-items: flex-start;
  }
  .ask-bar textarea {
    flex: 1;
    resize: vertical;
    min-height: 3.4rem;
  }
  .interpreted-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.8rem;
    align-items: center;
  }
  .chip {
    border-radius: 999px;
    padding: 0.25rem 0.7rem;
    font-size: 0.85rem;
  }
  .interpreted-chip {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }
  .chip-label {
    font-weight: 700;
  }
  .plain-chip {
    background: transparent;
    border: 1px dashed var(--color-border);
    color: var(--color-muted);
  }
  .results-header {
    margin-bottom: 0.6rem;
    color: var(--color-muted);
    font-size: 0.9rem;
  }
  .request-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 1.25rem;
    margin-bottom: 0.8rem;
    border-left: 3px solid var(--color-link, #2b4bd7);
  }
  .request-card-text {
    margin: 0;
    font-size: 0.95rem;
  }
  @media (max-width: 560px) {
    .request-card { flex-direction: column; align-items: flex-start; }
  }

  /* ---- Archive mode ---- */
  .archive-mode {
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-heading {
    font-size: 1.4em;
    font-weight: normal;
    margin: 0 0 0.25em;
    color: var(--archive-heading, #333333);
  }
  .archive-subtitle {
    font-size: 0.92em;
    color: var(--archive-muted, #666666);
    margin: 0 0 1em;
  }
  .archive-ask-bar {
    display: flex;
    gap: 0.6em;
    align-items: flex-start;
    margin-bottom: 1em;
  }
  .archive-ask-bar textarea {
    flex: 1;
    resize: vertical;
    min-height: 3.4em;
    font-family: inherit;
    font-size: 0.92em;
    padding: 0.4em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-ask-bar textarea:focus {
    outline: 1px solid var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-note {
    font-size: 0.92em;
    line-height: 1.5;
    margin-bottom: 0.8em;
    padding: 0.6em 0.8em;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-note p {
    margin: 0 0 0.5em;
  }
  .archive-error {
    border-left: 3px solid var(--archive-link, #990000);
    color: var(--archive-link, #990000);
  }
  .archive-request {
    border-left: 3px solid var(--archive-link, #990000);
  }
  .archive-loading {
    font-size: 0.92em;
    color: var(--archive-muted, #666666);
    margin-bottom: 0.8em;
    font-style: italic;
  }
  .archive-interpreted {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4em;
    margin-bottom: 0.8em;
    align-items: center;
  }
  .archive-chip {
    font-size: 0.85em;
    padding: 0.2em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-muted, #666666);
  }
  .archive-chip--plain {
    background: transparent;
    border-style: dashed;
  }
  .archive-results-count {
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    margin-bottom: 0.6em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.3em;
  }
  .archive-results {
    margin-top: 0.4em;
  }
  .archive-section {
    border-top: 1px solid var(--archive-border, #dddddd);
    padding-top: 1em;
    margin-top: 1em;
  }
  .archive-section h2 {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.15em;
    font-weight: normal;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.75em;
  }
  .archive-human-answers {
    margin-top: 2rem;
  }
</style>
