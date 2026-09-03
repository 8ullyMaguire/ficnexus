<script lang="ts">
  // Per-fic "Similar fic suggestions" panel.
  //
  // Lists community suggestions for the current fic (title/author/comment,
  // vote score + my_vote, owner/admin remove), and lets signed-in users
  // suggest a similar fic either by picking an in-DB fic (autocomplete over
  // /api/search) or pasting a URL (the backend scrapes it).
  import { onMount } from 'svelte';
  import {
    listFicSuggestions,
    createFicSuggestion,
    voteFicSuggestion,
    removeFicSuggestion,
    type FicSuggestion,
  } from '$lib/api/ficSuggestions';
  import { auth } from '$lib/stores/auth.svelte';

  let { urlId } = $props<{ urlId: string }>();

  let suggestions = $state<FicSuggestion[]>([]);
  let loading = $state(true);
  let loadError = $state('');

  // ── Suggest form state ────────────────────────────────────────────────
  let mode = $state<'search' | 'url'>('search');
  let query = $state('');
  let autocomplete = $state<{ url_id: string; title: string; author: string }[]>([]);
  let autocompleteOpen = $state(false);
  let autocompleteLoading = $state(false);
  let selected: { url_id: string; title: string; author: string } | null = $state(null);
  let urlInput = $state('');
  let comment = $state('');
  let submitting = $state(false);
  let formError = $state('');
  let formOk = $state('');

  // Debounce timer for the autocomplete lookup.
  let debounce: ReturnType<typeof setTimeout> | undefined;

  onMount(() => {
    load();
  });

  async function load() {
    loading = true;
    loadError = '';
    try {
      const res = await listFicSuggestions(urlId);
      if (res.err === 0) {
        suggestions = res.suggestions ?? [];
      } else {
        loadError = res.msg || 'Could not load suggestions.';
      }
    } catch {
      loadError = 'Network error loading suggestions.';
    } finally {
      loading = false;
    }
  }

  function onQueryInput() {
    selected = null;
    autocompleteOpen = query.trim().length > 0;
    clearTimeout(debounce);
    debounce = setTimeout(runAutocomplete, 250);
  }

  async function runAutocomplete() {
    const q = query.trim();
    if (!q) {
      autocomplete = [];
      autocompleteLoading = false;
      return;
    }
    autocompleteLoading = true;
    try {
      const res = await fetch(`/api/search?q=${encodeURIComponent(q)}&limit=6`, { credentials: 'include' });
      if (!res.ok) {
        autocomplete = [];
        return;
      }
      const data = await res.json();
      const items = data.results ?? data.fics ?? data.items ?? [];
      autocomplete = items
        .filter((r: { url_id?: string; work_id?: number }) => r.url_id)
        .slice(0, 6)
        .map((r: { url_id: string; title?: string; author?: string; canonical_title?: string; canonical_author?: string }) => ({
          url_id: r.url_id,
          title: r.title ?? r.canonical_title ?? r.url_id,
          author: r.author ?? r.canonical_author ?? '',
        }));
    } catch {
      autocomplete = [];
    } finally {
      autocompleteLoading = false;
    }
  }

  function pick(item: { url_id: string; title: string; author: string }) {
    selected = item;
    query = `${item.title}${item.author ? ` — ${item.author}` : ''}`;
    autocompleteOpen = false;
  }

  function switchMode(m: 'search' | 'url') {
    mode = m;
    formError = '';
    formOk = '';
  }

  async function submit() {
    if (!auth.isLoggedIn) {
      formError = 'Log in to suggest a fic.';
      return;
    }
    let payload: { suggested_url_id?: string; url?: string; comment?: string } = {};
    if (mode === 'search') {
      if (!selected?.url_id) {
        formError = 'Pick a fic from the autocomplete list.';
        return;
      }
      payload.suggested_url_id = selected.url_id;
    } else {
      if (!urlInput.trim()) {
        formError = 'Paste the URL of the fic to suggest.';
        return;
      }
      payload.url = urlInput.trim();
    }
    if (comment.trim()) payload.comment = comment.trim();

    submitting = true;
    formError = '';
    formOk = '';
    try {
      const res = await createFicSuggestion(urlId, payload);
      if (res.err === 0) {
        formOk = 'Suggestion added ✓';
        query = '';
        selected = null;
        urlInput = '';
        comment = '';
        autocomplete = [];
        await load();
      } else {
        formError = res.msg || 'Could not submit suggestion.';
      }
    } catch {
      formError = 'Network error submitting suggestion.';
    } finally {
      submitting = false;
    }
  }

  async function handleVote(s: FicSuggestion, vote: 1 | -1) {
    // Toggle: clicking the active direction retracts; clicking the other
    // direction switches. Then refresh to show the authoritative score.
    const next: 1 | -1 | 0 = s.my_vote === vote ? 0 : vote;
    const prev = s.my_vote;
    s.my_vote = next;
    s.score += next - prev;
    try {
      await voteFicSuggestion(s.id, next);
      await load();
    } catch {
      s.my_vote = prev;
      s.score += prev - next;
    }
  }

  async function handleRemove(s: FicSuggestion) {
    if (!confirm(`Remove this suggestion for "${s.suggested_title || s.suggested_url_id}"?`)) return;
    try {
      const res = await removeFicSuggestion(s.id);
      if (res.err === 0) {
        suggestions = suggestions.filter((x) => x.id !== s.id);
      } else {
        loadError = res.msg || 'Could not remove suggestion.';
      }
    } catch {
      loadError = 'Network error removing suggestion.';
    }
  }

  const isOwnerOrAdmin = $derived((s: FicSuggestion) => {
    return auth.level >= 100 || (s.user_id != null && s.user_id === auth.user?.id);
  });
</script>

<svelte:head><title>Similar fics — FicNexus</title></svelte:head>

<section class="fic-suggestions" aria-label="Similar fic suggestions">
  <div class="fs-head">
    <h3>💡 Similar fic suggestions</h3>
    <span class="muted fs-count">{suggestions.length} suggestion(s)</span>
  </div>

  {#if loading}
    <p class="muted"><span class="spinner"></span> Loading suggestions…</p>
  {:else if loadError}
    <p class="error-text">{loadError}</p>
  {:else if suggestions.length === 0}
    <p class="no-similar">No suggestions yet — be the first to recommend a similar fic.</p>
  {:else}
    <ul class="fs-list">
      {#each suggestions as s (s.id)}
        <li class="fs-item card">
          <div class="fs-main">
            <a class="fs-link" href={`/works/${encodeURIComponent(s.suggested_url_id)}`}>
              {s.suggested_title || s.suggested_url_id}
            </a>
            {#if s.suggested_author}
              <span class="muted fs-author">by {s.suggested_author}</span>
            {/if}
            {#if s.comment}
              <p class="muted fs-comment">{s.comment}</p>
            {/if}
            {#if isOwnerOrAdmin(s)}
              <button class="fs-remove" type="button" onclick={() => handleRemove(s)}>✕ remove</button>
            {/if}
          </div>
          <div class="fs-vote">
            <button
              class="fs-vote-btn up"
              class:active={s.my_vote === 1}
              type="button"
              onclick={() => handleVote(s, 1)}
              aria-label="Upvote suggestion"
            >▲</button>
            <span class="fs-score" class:positive={s.score > 0} class:negative={s.score < 0}>
              {s.score}
            </span>
            <button
              class="fs-vote-btn down"
              class:active={s.my_vote === -1}
              type="button"
              onclick={() => handleVote(s, -1)}
              aria-label="Downvote suggestion"
            >▼</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="fs-form">
    <div class="fs-form-tabs">
      <button type="button" class:active={mode === 'search'} onclick={() => switchMode('search')}>
        Search a fic
      </button>
      <button type="button" class:active={mode === 'url'} onclick={() => switchMode('url')}>
        Paste a link
      </button>
    </div>

    {#if mode === 'search'}
      <div class="fs-autocomplete">
        <input
          type="text"
          placeholder="Search fics by title…"
          bind:value={query}
          oninput={onQueryInput}
          onfocus={() => { if (query.trim()) autocompleteOpen = true; }}
          onblur={() => setTimeout(() => (autocompleteOpen = false), 150)}
          aria-label="Search a fic to suggest"
        />
        {#if autocompleteOpen && (autocomplete.length > 0 || autocompleteLoading)}
          <ul class="fs-ac-list" role="listbox">
            {#if autocompleteLoading}
              <li class="muted fs-ac-empty">Searching…</li>
            {:else}
              {#each autocomplete as item (item.url_id)}
                <li role="option" aria-selected="false">
                  <button type="button" class="fs-ac-item" onclick={() => pick(item)}>
                    <strong>{item.title}</strong>
                    {#if item.author}<span class="muted"> — {item.author}</span>{/if}
                  </button>
                </li>
              {/each}
            {/if}
          </ul>
        {/if}
      </div>
    {:else}
      <input
        type="url"
        placeholder="https://archiveofourown.org/works/…"
        bind:value={urlInput}
        aria-label="Fic URL to suggest"
      />
    {/if}

    <input
      type="text"
      placeholder="Comment (optional)"
      bind:value={comment}
      aria-label="Suggestion comment"
    />

    {#if formError}<p class="error-text">{formError}</p>{/if}
    {#if formOk}<p class="ok-text">{formOk}</p>{/if}

    <button class="btn btn-secondary" type="button" onclick={submit} disabled={submitting}>
      {submitting ? 'Adding…' : 'Suggest similar fic'}
    </button>
  </div>
</section>

<style>
  .fic-suggestions {
    margin-top: 1.5rem;
  }
  .fs-head {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    margin-bottom: 0.6rem;
  }
  .fs-head h3 {
    margin: 0;
    font-size: 1.05rem;
  }
  .fs-count {
    font-size: 0.85rem;
  }
  .fs-list {
    list-style: none;
    margin: 0 0 1rem;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .fs-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.8rem;
    padding: 0.6rem 0.8rem;
  }
  .fs-main {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }
  .fs-link {
    font-weight: 600;
    color: var(--color-primary);
    text-decoration: none;
  }
  .fs-link:hover {
    text-decoration: underline;
  }
  .fs-author {
    font-size: 0.82rem;
  }
  .fs-comment {
    margin: 0.2rem 0 0;
    font-size: 0.88rem;
  }
  .fs-remove {
    align-self: flex-start;
    margin-top: 0.3rem;
    background: none;
    border: none;
    color: var(--color-error, #c0392b);
    font-size: 0.78rem;
    cursor: pointer;
    padding: 0;
  }
  .fs-vote {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.1rem;
    flex-shrink: 0;
  }
  .fs-vote-btn {
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    width: 1.6rem;
    height: 1.4rem;
    cursor: pointer;
    color: var(--color-muted);
    line-height: 1;
  }
  .fs-vote-btn.active {
    border-color: var(--color-primary);
    color: var(--color-primary);
    background: var(--color-surface-2);
  }
  .fs-score {
    font-family: var(--mono);
    font-weight: 600;
    font-size: 0.9rem;
    min-width: 1.4rem;
    text-align: center;
  }
  .fs-score.positive {
    color: var(--color-positive, #1e8449);
  }
  .fs-score.negative {
    color: var(--color-error, #c0392b);
  }
  .fs-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.8rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    max-width: 560px;
  }
  .fs-form-tabs {
    display: flex;
    gap: 0.3rem;
  }
  .fs-form-tabs button {
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.25rem 0.6rem;
    font-size: 0.82rem;
    cursor: pointer;
    color: var(--color-muted);
  }
  .fs-form-tabs button.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }
  .fs-autocomplete {
    position: relative;
  }
  .fs-ac-list {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 20;
    list-style: none;
    margin: 0;
    padding: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    max-height: 220px;
    overflow-y: auto;
    box-shadow: 0 4px 12px rgb(0 0 0 / 0.12);
  }
  .fs-ac-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 0.45rem 0.6rem;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .fs-ac-item:hover {
    background: var(--color-surface-2);
  }
  .fs-ac-empty {
    padding: 0.45rem 0.6rem;
    font-size: 0.82rem;
  }
  .ok-text {
    color: var(--color-positive, #1e8449);
    font-size: 0.88rem;
    margin: 0;
  }
  .no-similar {
    color: var(--color-muted);
    font-size: 0.88rem;
  }
</style>
