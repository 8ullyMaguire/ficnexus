<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { searchAuthors, type AuthorSummary } from '$lib/api/authors';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let query = $state('');
  let results = $state<AuthorSummary[]>([]);
  let loading = $state(false);
  let searched = $state(false);

  // ── Alphabetical browse state ──────────────────────────────────────────
  let allAuthors = $state<AuthorSummary[]>([]);
  let browseLoading = $state(true);
  let browseError = $state('');

  onMount(async () => {
    // Try to read ?q= or ?name= from URL
    const url = new URL(window.location.href);
    const q = url.searchParams.get('q') ?? url.searchParams.get('name');
    if (q) {
      query = q;
      searched = true;
      await doSearch();
    } else {
      // Load all authors for alphabetical browse (first page of popular/known authors)
      await loadAllAuthors();
    }
  });

  async function loadAllAuthors() {
    browseLoading = true;
    browseError = '';
    try {
      // Fetch a broad set of authors for the alphabetical index
      const data = await searchAuthors('');
      allAuthors = data;
    } catch {
      browseError = 'Could not load people directory.';
    } finally {
      browseLoading = false;
    }
  }

  async function doSearch() {
    const q = query.trim();
    if (!q) return;
    loading = true;
    searched = true;
    try {
      results = await searchAuthors(q);
    } catch {
      results = [];
    } finally {
      loading = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') doSearch();
  }

  // Group authors by first letter of name
  const authorsByLetter = $derived.by(() => {
    const groups: Record<string, AuthorSummary[]> = {};
    for (const a of allAuthors) {
      const name = a.canonical_name || '';
      const letter = name.charAt(0).toUpperCase() || '#';
      if (!groups[letter]) groups[letter] = [];
      groups[letter].push(a);
    }
    return groups;
  });

  const alphabet = $derived(
    Object.keys(authorsByLetter).sort((a, b) => {
      if (a === '#') return -1;
      if (b === '#') return 1;
      return a.localeCompare(b);
    })
  );
</script>

{#if uiMode === 'archive'}
  <!-- AO3-style People page (browse + search) -->
  <div class="archive-people">
    <h2 class="archive-heading">People</h2>

    <!-- Search form -->
    <div class="people-search-form">
      <fieldset>
        <legend>Search People</legend>
        <dl>
          <dt>
            <label for="people-query">Search all fields</label>
          </dt>
          <dd>
            <input
              id="people-query"
              type="search"
              placeholder="Name or fandom"
              bind:value={query}
              onkeydown={onKeydown}
              aria-label="Search people"
            />
          </dd>
          <dt>
            <label for="people-name">Name</label>
          </dt>
          <dd>
            <input
              id="people-name"
              type="search"
              placeholder="Author name or pseud"
              bind:value={query}
              onkeydown={onKeydown}
              aria-label="Search by name"
            />
          </dd>
          <dt>
            <label for="people-fandom">Fandom</label>
          </dt>
          <dd>
            <input
              id="people-fandom"
              type="search"
              placeholder="Fandom name"
              bind:value={query}
              onkeydown={onKeydown}
              aria-label="Search by fandom"
            />
          </dd>
        </dl>
        <p class="submit actions">
          <ArchiveButton onclick={doSearch} disabled={loading}>
            {#if loading}Searching…{:else}Search People{/if}
          </ArchiveButton>
        </p>
      </fieldset>
    </div>

    <!-- Search results or alphabetical browse -->
    {#if searched}
      {#if loading}
        <ul class="people-results">
          {#each Array(3) as _}
            <li class="person-skeleton"> </li>
          {/each}
        </ul>
      {:else if results.length === 0}
        <p class="archive-empty">No people found.</p>
      {:else}
        <h3 class="landmark heading">People List</h3>
        <ul class="people-results">
          {#each results as person (person.id)}
            <li>
              <a class="person-link" href={`/people/${person.id}`}>{person.canonical_name}</a>
              {#if person.bio}
                <blockquote class="person-bio">{person.bio}</blockquote>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    {:else}
      <!-- Alphabetical browse -->
      {#if browseLoading}
        <p class="archive-loading">Loading people directory…</p>
      {:else if browseError}
        <p class="archive-error">{browseError}</p>
      {:else if allAuthors.length === 0}
        <p class="archive-empty">No people found.</p>
      {:else}
        <nav class="alphabet-nav" aria-label="Alphabetical navigation">
          <h3 class="landmark heading">Alphabet Navigation</h3>
          <ul class="alphabet-list">
            {#each alphabet as letter}
              <li><a href={"#letter-" + letter}>{letter}</a></li>
            {/each}
          </ul>
        </nav>

        <h3 class="landmark heading">Listing People</h3>
        <ul class="participant index group">
          {#each alphabet as letter}
            {@const group = authorsByLetter[letter] ?? []}
            <li id={"letter-" + letter} class="letter-group">
              <h4 class="letter-heading">
                {letter}
              </h4>
              {#each group as person (person.id)}
                <li class="user pseud picture blurb group">
                  <div class="header module">
                    <h5 class="heading">
                      <a href={`/people/${person.id}`}>{person.canonical_name}</a>
                    </h5>
                    {#if person.linked_authors > 0}
                      <span class="linked-count">{person.linked_authors} linked {person.linked_authors === 1 ? 'account' : 'accounts'}</span>
                    {/if}
                  </div>
                  {#if person.bio}
                    <blockquote class="userstuff">
                      <p>{person.bio}</p>
                    </blockquote>
                  {/if}
                </li>
              {/each}
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>
{:else}
  <!-- Modern mode: keep existing simple view -->
  <div class="people-page">
    <h1>Authors</h1>
    <p class="muted">
      Search curated author profiles. Each profile links together the pseudonyms and
      accounts that belong to the same author.
    </p>
    <div class="search-row">
      <input
        type="search"
        placeholder="Search authors by name or pseud…"
        bind:value={query}
        onkeydown={onKeydown}
        aria-label="Search authors"
      />
      <button class="btn" onclick={doSearch} disabled={loading}>
        {#if loading}<span class="spinner"></span> Searching…{:else}Search{/if}
      </button>
    </div>
    {#if loading}
      <div class="card"><p class="muted"><span class="spinner"></span> Loading authors…</p></div>
    {:else if searched && results.length === 0}
      <div class="card"><p class="muted">No authors found.</p></div>
    {:else if searched && results.length > 0}
      <div class="results">
        {#each results as a (a.id)}
          <a class="card author-card" href={`/authors/${a.id}`}>
            <div class="author-main">
              <strong>{a.canonical_name}</strong>
              {#if a.bio}
                <p class="muted author-bio">{a.bio}</p>
              {/if}
            </div>
            <span class="linked-count">
              {a.linked_authors} linked {a.linked_authors === 1 ? 'account' : 'accounts'}
            </span>
          </a>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  /* ── Archive mode styles ── */
  .archive-people {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }

  .archive-heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.6em;
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
    margin: 0 0 0.5em;
    padding-bottom: 0.3em;
    border-bottom: 2px solid var(--archive-link, #990000);
  }

  .people-search-form {
    margin-bottom: 1.5em;
  }

  .people-search-form fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.8em 1em;
    background: var(--archive-bg-raised, #f9f9f9);
  }

  .people-search-form legend {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.05em;
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.3em;
  }

  .people-search-form dt {
    font-weight: 600;
    color: var(--archive-muted, #666666);
    margin-top: 0.3em;
  }

  .people-search-form dd {
    margin: 0 0 0.3em 0;
  }

  .people-search-form input {
    padding: 0.3em 0.5em;
    font-size: 0.9em;
    font-family: inherit;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    border-radius: 0;
  }

  .people-search-form input:focus {
    outline: 1px solid var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }

  .submit.actions {
    margin-top: 0.5em;
  }

  .people-results {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .people-results li {
    padding: 0.5em 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }

  .people-results li:last-child {
    border-bottom: none;
  }

  .person-link {
    font-size: 1em;
    font-weight: 700;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .person-link:hover {
    text-decoration: underline;
  }

  .person-bio {
    font-size: 0.9em;
    line-height: 1.5;
    margin: 0.3em 0 0;
    color: var(--archive-text, #2a2a2a);
  }

  .person-skeleton {
    height: 1.4em;
    background: var(--archive-border, #dddddd);
    opacity: 0.4;
    margin-bottom: 0.5em;
  }

  /* Alphabetical browse */
  .alphabet-nav {
    margin-bottom: 1em;
  }

  .alphabet-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3em;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .alphabet-list a {
    display: inline-block;
    padding: 0.2em 0.5em;
    font-size: 0.85em;
    font-weight: 600;
    color: var(--archive-link, #990000);
    text-decoration: none;
    border: 1px solid var(--archive-border, #dddddd);
  }

  .alphabet-list a:hover {
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .letter-group {
    margin-bottom: 1.2em;
  }

  .letter-heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.1em;
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
    margin: 0 0 0.4em;
  }

  /* People blurbs */
  .user.pseud.picture.blurb.group {
    list-style: none;
    margin: 0 0 0.5em 0;
    padding: 0.4em 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }

  .user.pseud.picture.blurb .header.module .heading {
    font-size: 1em;
    font-weight: 700;
    margin: 0;
  }

  .linked-count {
    display: block;
    font-size: 0.82em;
    color: var(--archive-muted, #666666);
  }

  .userstuff {
    font-size: 0.9em;
    line-height: 1.5;
    margin: 0.3em 0 0;
    color: var(--archive-text, #2a2a2a);
  }

  .landmark.heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin: 0.5em 0 0.3em;
  }

  .archive-loading,
  .archive-empty {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    padding: 2em 0;
    text-align: center;
  }

  .archive-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    padding: 2em 0;
    text-align: center;
  }

  /* ── Modern mode (existing) ── */
  .people-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  .search-row { display: flex; gap: 0.5rem; margin: 1rem 0; }
  .results { display: flex; flex-direction: column; gap: 0.5rem; }
  .author-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.8rem 1rem;
    text-decoration: none;
    color: var(--color-text);
  }
  .author-card:hover { background: var(--color-surface-2); }
  .author-bio { font-size: 0.85rem; margin: 0.25rem 0 0; max-width: 60ch; }
  @media (max-width: 640px) {
    .archive-people { padding: 0.75rem; }
    .people-search-form dt, .people-search-form dd { width: 100%; }
  }
</style>
