<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { t } from '$lib/i18n/index.svelte';
  import {
    searchTagsAdvanced,
    type TagItem,
    type TagSearchAdvancedResponse,
    TAG_TYPE_LABELS,
    TAG_TYPE_ORDER,
  } from '$lib/api/tags';

  const uiMode = $derived(getPref('uiMode'));

  // Form state mirroring AO3's tag search
  let tagName = $state('');
  let tagType = $state<'any' | '1' | '2' | '3' | '4' | '5' | '6'>('any');
  let wrangling = $state<'canonical' | 'non-canonical' | 'synonymous' | 'canonical_or_syn' | 'non_canonical_non_syn' | 'unwrangleable' | 'any'>('any');
  let fandom = $state('');
  let sortBy = $state<'name' | 'usage' | 'created'>('name');
  let sortDir = $state<'desc' | 'asc'>('desc');

  let results = $state<TagItem[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state('');
  let pageNum = $state(1);
  const perPage = 50;

  async function doSearch(reset = true) {
    if (reset) pageNum = 1;
    loading = true;
    error = '';
    try {
      const params: any = {
        page: pageNum,
        limit: perPage,
        sort: sortBy,
      };
      if (tagName.trim()) params.q = tagName.trim();
      if (tagType !== 'any') params.tag_type = parseInt(tagType);
      if (wrangling === 'canonical') params.canonical = true;
      else if (wrangling === 'non-canonical') params.canonical = false;
      if (fandom.trim()) params.fandom = fandom.trim();

      const res: TagSearchAdvancedResponse = await searchTagsAdvanced(params);
      if (res.err === 0) {
        results = res.tags ?? [];
        total = res.total ?? 0;
      } else {
        error = 'Search failed';
        results = [];
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Search failed';
      results = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    // Don't auto-search on mount — AO3 doesn't either
  });

  function tagUrl(tag: TagItem): string {
    const typeName = TAG_TYPE_LABELS[tag.tag_type_id] ?? 'Tag';
    return `/search?include_tags=${encodeURIComponent(typeName.toLowerCase())}:${encodeURIComponent(tag.name)}`;
  }
</script>

<svelte:head><title>Tag Search — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content archive-page tag-search-page">
      <header class="archive-header">
        <h1 class="archive-page-title">Tag Search</h1>
        <blockquote class="archive-summary">
          <p>Find tags by name, type, or wrangling status. Search for canonical tags (the "master" tag that others redirect to) or non-canonical ones. Find tags wrangled to specific canonical fandoms.</p>
        </blockquote>
      </header>

      <!-- Search form -->
      <div class="tag-search-form">
        <dl class="archive-dl">
          <div class="dl-row">
            <dt><label for="tag-name">Tag name</label></dt>
            <dd><input id="tag-name" type="search" bind:value={tagName} placeholder="e.g. Harry Potter" /></dd>
          </div>
          <div class="dl-row">
            <dt>Wrangling status</dt>
            <dd class="radio-group">
              <label><input type="radio" name="wrangling" value="any" bind:group={wrangling} /> Any status</label>
              <label><input type="radio" name="wrangling" value="canonical" bind:group={wrangling} /> Canonical</label>
              <label><input type="radio" name="wrangling" value="non-canonical" bind:group={wrangling} /> Non-canonical</label>
              <label><input type="radio" name="wrangling" value="synonymous" bind:group={wrangling} /> Synonymous</label>
              <label><input type="radio" name="wrangling" value="canonical_or_syn" bind:group={wrangling} /> Canonical or synonymous</label>
              <label><input type="radio" name="wrangling" value="non_canonical_non_syn" bind:group={wrangling} /> Non-canonical, non-synonymous</label>
              <label><input type="radio" name="wrangling" value="unwrangleable" bind:group={wrangling} /> Unwrangleable</label>
            </dd>
          </div>
        </dl>

        <dl class="archive-dl">
          <div class="dl-row">
            <dt>Type</dt>
            <dd class="radio-group">
              <label><input type="radio" name="type" value="any" bind:group={tagType} /> Any type</label>
              <label><input type="radio" name="type" value="1" bind:group={tagType} /> Fandom</label>
              <label><input type="radio" name="type" value="2" bind:group={tagType} /> Character</label>
              <label><input type="radio" name="type" value="3" bind:group={tagType} /> Relationship</label>
              <label><input type="radio" name="type" value="4" bind:group={tagType} /> Freeform</label>
              <label><input type="radio" name="type" value="5" bind:group={tagType} /> Warning</label>
              <label><input type="radio" name="type" value="6" bind:group={tagType} /> Category</label>
            </dd>
          </div>
          <div class="dl-row">
            <dt><label for="fandom-filter">Find tags wrangled to specific canonical fandoms</label></dt>
            <dd><input id="fandom-filter" type="search" bind:value={fandom} placeholder="e.g. Marvel Cinematic Universe" /></dd>
          </div>
        </dl>

        <dl class="archive-dl">
          <div class="dl-row">
            <dt>Sort by</dt>
            <dd class="radio-group">
              <label><input type="radio" name="sort" value="name" bind:group={sortBy} /> Name</label>
              <label><input type="radio" name="sort" value="usage" bind:group={sortBy} /> Usage</label>
              <label><input type="radio" name="sort" value="created" bind:group={sortBy} /> Created</label>
            </dd>
          </div>
          <div class="dl-row">
            <dt>Sort direction</dt>
            <dd class="radio-group">
              <label><input type="radio" name="dir" value="desc" bind:group={sortDir} /> Descending</label>
              <label><input type="radio" name="dir" value="asc" bind:group={sortDir} /> Ascending</label>
            </dd>
          </div>
        </dl>

        <div class="archive-form-actions">
          <button class="archive-btn" onclick={() => doSearch(true)}>Search</button>
        </div>
      </div>

      <!-- Results -->
      {#if loading}
        <div class="archive-loading">Searching tags…</div>
      {:else if error}
        <div class="archive-error"><strong>{error}</strong></div>
      {:else if results.length === 0 && tagName}
        <blockquote class="archive-summary">
          <p>No tags found matching your search.</p>
        </blockquote>
      {:else if results.length > 0}
        <p class="archive-muted count-line">{total} tag{total === 1 ? '' : 's'} found</p>
        <table class="archive-table tag-result-table">
          <thead>
            <tr>
              <th class="col-name">Tag</th>
              <th class="col-type">Type</th>
              <th class="col-stat">Uses</th>
            </tr>
          </thead>
          <tbody>
            {#each results as tag (tag.id)}
              <tr>
                <td class="col-name">
                  <a class="archive-link" href={tagUrl(tag)}>{tag.name}</a>
                  {#if tag.description}<span class="archive-muted">{tag.description}</span>{/if}
                </td>
                <td class="col-type">{TAG_TYPE_LABELS[tag.tag_type_id] ?? 'Tag'}</td>
                <td class="col-stat">{tag.usage_count}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <nav class="archive-pagination" aria-label="Tag search pages">
          {#if pageNum > 1}
            <button class="archive-page-link" onclick={() => { pageNum--; doSearch(false); }}>Previous</button>
          {/if}
          <span class="archive-muted">Page {pageNum}</span>
          {#if results.length === perPage}
            <button class="archive-page-link" onclick={() => { pageNum++; doSearch(false); }}>Next</button>
          {/if}
        </nav>
      {/if}
    </div>
  </main>
{:else}
  <div class="tag-search-page">
    <h1>Tag Search</h1>
    <p class="muted subtitle">
      Find tags by name, type, or wrangling status. Search for canonical tags or tags wrangled to specific fandoms.
    </p>
    <div class="tag-search-form card">
      <div class="form-row">
        <input type="text" bind:value={tagName} placeholder="Tag name" />
      </div>
      <div class="form-row">
        <select bind:value={tagType}>
          <option value="any">Any type</option>
          <option value="1">Fandom</option>
          <option value="2">Character</option>
          <option value="3">Relationship</option>
          <option value="4">Freeform</option>
          <option value="5">Warning</option>
          <option value="6">Category</option>
        </select>
      </div>
      <button class="btn" onclick={() => doSearch(true)}>Search</button>
    </div>
    {#if loading}<p class="muted">Searching…</p>
    {:else if error}<div class="error-card"><strong>{error}</strong></div>
    {:else if results.length > 0}
      <ul class="tag-results">
        {#each results as tag (tag.id)}
          <li><a href={tagUrl(tag)}>{tag.name}</a> — {TAG_TYPE_LABELS[tag.tag_type_id]} ({tag.usage_count} uses)</li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  .tag-search-page { max-width: 900px; margin: 0 auto; padding: 1rem; }
  .archive-page { max-width: 900px; margin: 0 auto; }
  .archive-header { margin-bottom: 1.25rem; }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8rem;
    color: #990000;
    margin: 0.5rem 0;
  }
  .archive-content { padding: 1rem; font-family: Georgia, 'Times New Roman', serif; color: #2a2a2a; }
  .archive-dl { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; margin-bottom: 1.5rem; }
  .dl-row { margin-bottom: 1rem; }
  .dl-row dt { font-weight: 700; color: #666; margin-bottom: 0.3rem; }
  .dl-row dd { margin: 0; }
  .radio-group { display: flex; flex-direction: column; gap: 0.3rem; }
  .radio-group label { display: flex; align-items: center; gap: 0.4rem; font-size: 0.9rem; }
  .archive-form-actions { margin-top: 1rem; }
  .archive-btn {
    display: inline-block;
    padding: 0.4rem 1em;
    font-size: 0.85em;
    font-weight: 700;
    text-decoration: none;
    border: 1px solid #ddd;
    background: #fff;
    color: #990000;
    border-radius: 0;
    cursor: pointer;
  }
  .archive-btn:hover { background: #f5f5f5; }
  .tag-search-form { margin: 1rem 0 2rem; }
  .archive-loading { padding: 2em 0; text-align: center; color: #666; font-style: italic; }
  .archive-error { color: #990000; font-weight: 700; padding: 1.5rem 0; text-align: center; }
  .count-line { margin-bottom: 0.75rem; }
  .tag-result-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .tag-result-table th {
    text-align: left; padding: 0.45em 0.7em; border-bottom: 2px solid #ddd;
    font-size: 0.82em; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: #666;
  }
  .tag-result-table td { padding: 0.5em 0.7em; border-bottom: 1px solid #ddd; }
  .tag-result-table td.col-name { width: 100%; }
  .archive-link { font-weight: 700; color: #990000; text-decoration: none; }
  .archive-link:hover { text-decoration: underline; }
  .archive-muted { color: #666; font-size: 0.85em; }
  .archive-pagination { display: flex; align-items: center; justify-content: center; gap: 1rem; margin-top: 1.25rem; }
  .archive-page-link {
    display: inline-block; padding: 0.3rem 0.8rem; font-size: 0.85em; font-weight: 700;
    text-decoration: none; border: 1px solid #ddd; background: #fff; color: #990000;
  }

  /* Modern mode */
  .tag-search-page h1 { font-family: Georgia, 'Times New Roman', serif; font-size: 1.8rem; }
  .tag-search-page .subtitle { margin-bottom: 1rem; }
  .form-row { margin-bottom: 0.7rem; }
  .form-row input, .form-row select { padding: 0.4rem 0.6rem; border-radius: 6px; border: 1px solid #ddd; font: inherit; }
  .tag-results { list-style: none; padding: 0; }
  .tag-results li { margin: 0.3rem 0; }
  @media (max-width: 640px) {
    .archive-dl { grid-template-columns: 1fr; }
  }
</style>
