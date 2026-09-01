<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import {
    getRequest, addAnswer, voteAnswer, acceptAnswer, deleteAnswer, deleteRequest, upvoteRequest,
    type RequestDetail, type RequestAnswer,
  } from '$lib/api/requests';
  import { t } from '$lib/i18n/index.svelte';
  import DocLink from '$lib/components/DocLink.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import { askArchive, type AskResponse } from '$lib/api/ask';
  import type { SearchResult, SearchFilters } from '$lib/api/search';
  import { buildSearchQuery } from '$lib/api/search';
  import AnswerRow from '$lib/ui/requests/AnswerRow.svelte';
  import ArchivistCall from '$lib/ui/requests/ArchivistCall.svelte';

  const uiMode = $derived(getPref('uiMode'));

  const MAX_ANSWERS = 3; // per user per request (user-approved cap)

  let detail = $state<RequestDetail | null>(null);
  let answers = $state<RequestAnswer[]>([]);
  let loading = $state(true);
  let error = $state('');
  let myAnswerCount = $state(0);

  let answerWorkId = $state<number | null>(null);
  let answerUrl = $state('');
  let answerPitch = $state('');
  let answerSearchQuery = $state(''); // new: search-kind answer
  let submitting = $state(false);
  let formMsg = $state('');

  // Ask-the-Archive box state
  let askOpen = $state(false);
  let askLoading = $state(false);
  let askError = $state('');
  let askResult: AskResponse | null = $state(null);
  let askingUrlId = $state<string | null>(null);
  let askMsg = $state('');

  // Honeypot
  let formOpenedAt = $state(0);
  let website = $state('');

  let myId = $state<number | null>(null);

  onMount(async () => {
    formOpenedAt = Date.now() - 5000;
    const id = Number($page.params.id);
    await load(id);
  });

  async function load(id: number) {
    loading = true;
    error = '';
    try {
      const res = await getRequest(id);
      if (res.err === 0) {
        detail = res.request;
        answers = res.answers;
        // The detail response includes answers with user_id — derive my count
        myAnswerCount = answers.filter((a) => a.user_id === myId).length;
      } else {
        error = (res as { msg?: string }).msg ?? 'Request not found';
      }
    } catch {
      error = 'Failed to load request';
    } finally {
      loading = false;
    }
  }

  async function submitAnswer() {
    if (!detail) return;
    if (website) return; // honeypot
    const workIdNum = answerWorkId ? Number(answerWorkId) : null;
    if (!workIdNum && !answerUrl.trim() && !answerSearchQuery.trim()) {
      formMsg = 'Enter a work id, a fic URL, or a search query.';
      return;
    }
    submitting = true;
    formMsg = '';
    try {
      const res = await addAnswer(detail.id, workIdNum, answerUrl.trim(), answerPitch.trim(), answerSearchQuery.trim() || null, null);
      if (res.err === 0) {
        answerWorkId = null;
        answerUrl = '';
        answerPitch = '';
        answerSearchQuery = '';
        await load(detail.id);
      } else {
        formMsg = res.msg ?? 'Failed to add answer';
      }
    } catch {
      formMsg = 'Failed to add answer';
    } finally {
      submitting = false;
    }
  }

  async function vote(a: RequestAnswer, v: 1 | -1) {
    if (!detail) return;
    const next = a.my_vote === v ? 0 : v;
    const res = await voteAnswer(detail.id, a.id, next);
    if (res.err === 0) {
      await load(detail.id);
    }
  }

  async function accept(a: RequestAnswer) {
    if (!detail) return;
    const res = await acceptAnswer(detail.id, a.id);
    if (res.err === 0) await load(detail.id);
  }

  async function removeAnswer(a: RequestAnswer) {
    if (!detail) return;
    const res = await deleteAnswer(detail.id, a.id);
    if (res.err === 0) await load(detail.id);
  }

  async function removeRequest() {
    if (!detail) return;
    const res = await deleteRequest(detail.id);
    if (res.err === 0) {
      window.location.href = '/requests';
    }
  }

  async function upvote() {
    if (!detail) return;
    const res = await upvoteRequest(detail.id, !detail.my_upvote);
    if (res.err === 0) await load(detail.id);
  }

  /** Ask the Archive: find works already in the library that fit this request. */
  async function askTheArchive() {
    if (!detail || askLoading) return;
    askOpen = true;
    askLoading = true;
    askError = '';
    askResult = null;
    askMsg = '';
    const nl = `${detail.title}${detail.body ? ` ${detail.body}` : ''}`.trim();
    try {
      askResult = await askArchive(nl);
    } catch (e) {
      askError = e instanceof Error ? e.message : 'Ask failed';
    } finally {
      askLoading = false;
    }
  }

  /** Attach an ask result as an answer via its url_id (fic_info id). */
  async function addAskResult(r: SearchResult) {
    if (!detail || askingUrlId) return;
    askingUrlId = r.url_id;
    askMsg = '';
    try {
      const res = await addAnswer(detail.id, null, '', answerPitch.trim() || '', r.url_id, null);
      if (res.err === 0) {
        askMsg = `Added “${r.title}” as an answer.`;
        await load(detail.id);
      } else {
        askMsg = res.msg ?? 'Failed to add answer';
      }
    } catch {
      askMsg = 'Failed to add answer';
    } finally {
      askingUrlId = null;
    }
  }

  /** Build a /search deep link from a search_query answer. The stored value
   * may already be a full /search?... URL (archivist/llm path) or a raw query
   * string (user search answers). */
  function searchAnswerHref(a: RequestAnswer): string {
    const raw = a.search_query ?? '';
    if (raw.startsWith('/search')) return raw;
    return `/search?${buildSearchQuery(searchQueryToFilters(raw))}`;
  }

  /** Parse a search_query string ("q terms +min_words:50000 complete:true")
   * into SearchFilters so we can (re)build a canonical /search URL. Tolerates
   * plain text (treated as the `q` free-text) and AO3-style `field:value`
   * tokens — mirrors the boolean parser the backend uses. */
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
</script>

<svelte:head><title>{detail?.title ?? 'Request'} — FicNexus</title></svelte:head>

<div class="detail">
  {#if loading}
    <p class="muted"><span class="spinner"></span> Loading…</p>
  {:else if error || !detail}
    <div class="error-card"><strong>⚠️ {error || 'Not found'}</strong></div>
  {:else if uiMode === 'archive'}
    <div class="archive-detail">
      <h1 class="archive-title">{detail.title}</h1>
      <div class="archive-meta-row">
        <span class="archive-status">{detail.status}</span>
        <span class="archive-byline">by {detail.username}</span>
        {#if detail.seed_fic}<span class="archive-seed">seeded from "{detail.seed_fic.title}"</span>{/if}
      </div>
      {#if detail.body}<p class="archive-body">{detail.body}</p>{/if}
      {#if detail.status === 'answered'}
        <p class="archive-ok">{t('requests.markedAnswered')}</p>
      {/if}
      <div class="archive-actions">
        <ArchiveButton onclick={upvote}>
          {detail.my_upvote ? 'Remove Upvote' : 'Upvote'} ({detail.upvotes ?? 0})
        </ArchiveButton>
        <ArchiveButton href="#answer-form">Reply</ArchiveButton>
      </div>

      <section class="archive-section">
        <h2>{t('requests.answersTitle', { count: answers.length })}</h2>
        {#if answers.length === 0}
          <p class="archive-muted">{t('requests.noAnswers')}</p>
        {:else}
          <ul class="archive-answer-list">
            {#each answers as a (a.id)}
              <AnswerRow a={a} myId={myId} requestOwnerId={detail.user_id} requestStatus={detail.status} onVote={vote} onAccept={accept} onRemove={removeAnswer} uiMode={uiMode} />
            {/each}
          </ul>
        {/if}
      </section>

      {#if detail.status === 'open'}
        <section class="archive-section archive-answer-form" id="answer-form">
          <h2>{t('requests.addAnswer')}{myAnswerCount >= MAX_ANSWERS ? ` (limit: ${MAX_ANSWERS})` : ''}</h2>
          {#if myAnswerCount >= MAX_ANSWERS}
            <p class="archive-muted">You've added the maximum {MAX_ANSWERS} answers to this request.</p>
          {:else}
            <form class="archive-form" onsubmit={(e) => { e.preventDefault(); submitAnswer(); }}>
              <dl class="archive-dl">
                <dt>{t('requests.workId')}</dt>
                <dd><input type="number" bind:value={answerWorkId} placeholder="e.g. 42" min="1" /></dd>
                <dt>{t('requests.ficUrl')}</dt>
                <dd><input type="url" bind:value={answerUrl} placeholder="https://archiveofourown.org/works/…" /></dd>
                <dt>{t('requests.searchAnswer')}</dt>
                <dd><input type="text" bind:value={answerSearchQuery} maxlength="300" placeholder={t('requests.searchAnswerPlaceholder')} /></dd>
                <dt>{t('requests.whyFits')}</dt>
                <dd><input type="text" bind:value={answerPitch} maxlength="500" placeholder={t('requests.whyFitsPlaceholder')} /></dd>
              </dl>
              <input type="text" name="website" bind:value={website} style="display:none" tabindex="-1" autocomplete="off" />
              {#if formMsg}<p class="archive-error">{formMsg}</p>{/if}
              <ArchiveButton type="submit" disabled={submitting || (!answerWorkId && !answerUrl.trim() && !answerSearchQuery.trim())}>
                {submitting ? t('requests.adding') : t('requests.addAnswer')}
              </ArchiveButton>
            </form>
          {/if}
        </section>
      {/if}

      {#if detail.user_id === myId}
        <ArchiveButton onclick={removeRequest}>{t('requests.deleteRequest')}</ArchiveButton>
      {/if}
    </div>
  {:else}
    <header class="req-head">
      <div class="req-title-row">
        <h1>{detail.title}</h1>
        <button class="upvote-btn" class:active={detail.my_upvote} onclick={upvote} title="Upvote this request" aria-label="Upvote">
          ▲ {detail.upvotes ?? 0}
        </button>
      </div>
      <p class="subtitle">by {detail.username} · {detail.status}{detail.seed_fic ? ` · seeded from "${detail.seed_fic.title}"` : ''}</p>
      {#if detail.body}<p class="body-text">{detail.body}</p>{/if}
      {#if detail.status === 'answered'}
        <p class="ok">{t('requests.markedAnswered')}</p>
      {/if}
    </header>

    <section class="answers">
      <h2>{t('requests.answersTitle', { count: answers.length })}</h2>
      {#if answers.length === 0}
        <p class="empty">{t('requests.noAnswers')}</p>
      {:else}
        <ul class="answer-list">
          {#each answers as a (a.id)}
            <AnswerRow a={a} myId={myId} requestOwnerId={detail.user_id} requestStatus={detail.status} onVote={vote} onAccept={accept} onRemove={removeAnswer} uiMode={uiMode} />
          {/each}
        </ul>
      {/if}
    </section>

    {#if detail.status === 'open'}
      <section class="ask-box">
        <div class="ask-box-head">
          <h3>Ask the Archive for this request</h3>
          <button class="btn btn-small" onclick={askTheArchive} disabled={askLoading}>
            {askLoading ? 'Asking…' : (askOpen ? '↻ Ask again' : 'Find matching fics')}
          </button>
        </div>
        {#if askOpen}
          {#if askLoading}
            <p class="muted"><span class="spinner"></span> Asking the archive…</p>
          {:else if askError}
            <p class="bad">{askError}</p>
          {:else if askResult}
            {#if askResult.results.length === 0}
              <p class="muted">No existing fics matched. Suggest one manually below, or post a new request.</p>
            {:else}
              <ul class="ask-results">
                {#each askResult.results.slice(0, 6) as r (r.url_id)}
                  <li class="ask-result">
                    <div class="ask-result-main">
                      <a href="/works/{r.url_id}" class="fic-title">{r.title}</a>
                      <span class="author">by {r.author}</span>
                      <p class="meta">{r.words.toLocaleString()} words · {r.chapters} ch{r.comment_count > 0 ? ` · 💬 ${r.comment_count}` : ''}</p>
                    </div>
                    <button
                      class="btn btn-small"
                      onclick={() => addAskResult(r)}
                      disabled={askingUrlId !== null}
                    >
                      {askingUrlId === r.url_id ? 'Adding…' : '＋ Add as answer'}
                    </button>
                  </li>
                {/each}
              </ul>
              {#if askMsg}<p class="ok small">{askMsg}</p>{/if}
            {/if}
          {/if}
        {/if}
      </section>
    {/if}

    {#if detail.status === 'open'}
      <section class="answer-form">
        <h3>{t('requests.addAnswer')}{myAnswerCount >= MAX_ANSWERS ? ` (limit: ${MAX_ANSWERS})` : ''} <DocLink slug="features#fic-requests" label="?" title="How Fic Requests answers work" /></h3>
        {#if myAnswerCount >= MAX_ANSWERS}
          <p class="muted">You've added the maximum {MAX_ANSWERS} answers to this request.</p>
        {:else}
          <form onsubmit={(e) => { e.preventDefault(); submitAnswer(); }}>
            <label>
              {t('requests.workId')}
              <input type="number" bind:value={answerWorkId} placeholder="e.g. 42" min="1" />
            </label>
            <p class="muted small">— or paste a fic URL (AO3, FFN, RoyalRoad, forums) —</p>
            <label>
              {t('requests.ficUrl')}
              <input type="url" bind:value={answerUrl} placeholder="https://archiveofourown.org/works/…" />
            </label>
            <p class="muted small">— or share a search link (parsed query or /search URL) —</p>
            <label>
              {t('requests.searchAnswer')}
              <input type="text" bind:value={answerSearchQuery} maxlength="300" placeholder={t('requests.searchAnswerPlaceholder')} />
            </label>
            <label>
              {t('requests.whyFits')}
              <input type="text" bind:value={answerPitch} maxlength="500" placeholder={t('requests.whyFitsPlaceholder')} />
            </label>
            <input type="text" name="website" bind:value={website} style="display:none" tabindex="-1" autocomplete="off" />
            {#if formMsg}<p class="bad">{formMsg}</p>{/if}
            <button class="btn btn-primary" type="submit" disabled={submitting || (!answerWorkId && !answerUrl.trim() && !answerSearchQuery.trim())}>
              {submitting ? t('requests.adding') : t('requests.addAnswer')}
            </button>
          </form>
        {/if}
      </section>
    {/if}

    {#if detail.user_id === myId}
      <button class="btn btn-small btn-danger" onclick={removeRequest}>{t('requests.deleteRequest')}</button>
    {/if}
  {/if}
</div>

{#if uiMode === 'archive'}
<style>
  .archive-detail {
    max-width: var(--archive-max-width, 760px);
    margin: 0 auto;
    padding: 1.5rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.7em;
    font-weight: normal;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.3em;
  }
  .archive-meta-row {
    display: flex;
    gap: 0.75em;
    font-size: 0.9em;
    color: var(--archive-muted, #666666);
    margin-bottom: 0.75em;
  }
  .archive-status { font-variant: small-caps; font-size: 0.9em; }
  .archive-byline { color: var(--archive-muted, #666666); }
  .archive-seed { font-style: italic; }
  .archive-body { white-space: pre-wrap; margin: 0 0 1em; }
  .archive-ok { color: var(--archive-ok, #2e9e5b); margin: 0 0 1em; }
  .archive-actions { display: flex; gap: 0.5em; margin-bottom: 1.5em; }
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
  .archive-muted { color: var(--archive-muted, #666666); font-style: italic; font-size: 0.9em; }
  .archive-answer-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75em; }
  .archive-answer-row {
    display: flex;
    gap: 1em;
    padding: 0.75em;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-answer-votes { display: flex; flex-direction: column; align-items: center; gap: 0.2em; }
  .archive-vote-btn {
    background: none; border: 1px solid var(--archive-border, #dddddd); cursor: pointer;
    padding: 0.15em 0.4em; font-size: 0.8em; color: var(--archive-muted, #666666);
  }
  .archive-vote-btn.active { color: var(--archive-link, #990000); border-color: var(--archive-link, #990000); }
  .archive-answer-body { flex: 1; min-width: 0; }
  .archive-fic-link { color: var(--archive-link, #990000); text-decoration: none; font-weight: 600; }
  .archive-fic-link:hover { color: #999; }
  .archive-accepted-tag {
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-ok, #2e9e5b);
    padding: 0.1em 0.5em;
    font-size: 0.8em;
    font-variant: small-caps;
  }
  .archive-pitch { margin: 0.4em 0; font-style: italic; }
  .archive-answer-actions { display: flex; gap: 0.5em; margin-top: 0.5em; }
  .archive-form { display: flex; flex-direction: column; gap: 1em; max-width: 500px; }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt {
    font-weight: 600;
    font-size: 0.9em;
    margin-bottom: 0.2em;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd {
    margin: 0 0 0.75em 0;
  }
  .archive-dl input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.4em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
  }
  .archive-error { color: var(--archive-heading, #990000); font-size: 0.9em; }
  .archivist-call {
    border: 1px solid var(--archive-link, #990000);
    background: var(--archive-bg-raised, #f5f5f5);
    padding: 0.75em 1em;
    border-radius: 0;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    line-height: 1.5;
  }
  .archivist-attribution {
    font-weight: 700;
    color: var(--archive-link, #990000);
    font-size: 0.85em;
    font-variant: small-caps;
  }
  .archivist-lipsum {
    margin-top: 0.3em;
    white-space: pre-wrap;
    color: var(--archive-text, #2a2a2a);
  }
  .search-answer-card {
    display: inline-block;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    padding: 0.5em 0.85em;
    border-radius: 0;
    margin: 0.3em 0;
    font-size: 0.9em;
  }
  .search-answer-card:hover {
    background: var(--archive-border, #dddddd);
  }
  .search-answer-card a {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .search-answer-card a:hover { color: #999; }
</style>
{/if}

<style>
  .detail { max-width: 760px; margin: 0 auto; padding: 1.5rem; }
  .req-head { margin-bottom: 1.5rem; }
  .req-title-row { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
  .req-title-row h1 { margin: 0; }
  .upvote-btn {
    display: inline-flex; align-items: center; gap: 0.3rem;
    padding: 0.3rem 0.7rem; border-radius: 999px;
    border: 1px solid var(--color-border, #ccc); background: var(--color-bg, #fff);
    cursor: pointer; font-weight: 700; color: inherit;
  }
  .upvote-btn.active { background: var(--color-ok-bg, #e8f7ee); border-color: var(--color-ok, #2e9e5b); color: var(--color-ok, #2e9e5b); }

  .subtitle { color: var(--color-text-muted, #888); }
  .body-text { margin-top: 0.75rem; white-space: pre-wrap; }
  .answer-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .answer-card { display: flex; gap: 1rem; padding: 1rem; }
  .answer-card.accepted { border: 2px solid var(--color-ok, #2e9e5b); }
  .vote-col { display: flex; flex-direction: column; align-items: center; gap: 0.2rem; }
  .vote-btn { background: none; border: 1px solid var(--color-border, #ddd); border-radius: 6px; cursor: pointer;
    padding: 0.2rem 0.5rem; font-size: 0.85rem; color: var(--color-text-muted, #888);
  }
  .vote-btn.active { color: var(--color-link, #2b4bd7); border-color: var(--color-link, #2b4bd7); }
  .score { font-weight: 700; font-size: 1rem; }
  .answer-body { flex: 1; min-width: 0; }
  .fic-line { display: flex; align-items: baseline; gap: 0.5rem; flex-wrap: wrap; }
  .fic-title { font-weight: 700; color: var(--color-link, #2b4bd7); text-decoration: none; }
  .fic-title:hover { color: #999; }
  .author { color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .accepted-tag {
    background: var(--color-ok-bg, #e8f7ee);
    color: var(--color-ok, #2e9e5b);
    border-radius: 999px;
    padding: 0.1rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 700;
  }
  .pitch { margin: 0.4rem 0; font-style: italic; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; margin-top: 0.3rem; }
  .btn { cursor: pointer; }
  .ask-box { margin-top: 2rem; border-top: 1px solid var(--color-border, #ddd); padding-top: 1.5rem; }
  .ask-box-head { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1rem; }
  .ask-results { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem; }
  .ask-result { display: flex; justify-content: space-between; align-items: center; gap: 1rem; padding: 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; }
  .ask-result-main { flex: 1; min-width: 0; }
  .answer-form { margin-top: 2rem; border-top: 1px solid var(--color-border, #ddd); padding-top: 1.5rem; }
  .error-card { color: var(--color-error, #a34); }
  .muted { color: var(--color-text-muted, #888); }
  .ok { color: var(--color-ok, #2e9e5b); }
  .bad { color: var(--color-error, #a34); }
  .empty { color: var(--color-text-muted, #888); }
  .spinner { display: inline-block; width: 1em; height: 1em; border: 2px solid var(--color-border, #ddd); border-top-color: var(--color-link, #2b4bd7); border-radius: 50%; animation: spin 0.7s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .search-answer-link {
    font-weight: 600;
    color: var(--color-link, #2b4bd7);
  }
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
  /* Archive mode LLM / search answers */
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
