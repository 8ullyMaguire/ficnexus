<script lang="ts">
  import { onMount } from 'svelte';
  import ArchiveButton from './ArchiveButton.svelte';
  import {
    parseRange,
    mergeChipIntoForm,
    defaultFormState,
    type FormState,
    type Chip,
  } from './searchForm';

  let {
    mode = 'page',
    formState,
    onChange,
    onSubmit,
    notes = [],
  }: {
    mode?: 'page' | 'sidebar';
    formState: FormState;
    onChange: (newState: FormState) => void;
    onSubmit: () => void;
    notes?: string[];
  } = $props();

  let chips: Chip[] = $state([]);
  let wordCountInput = $state(formState.min_words || formState.max_words
    ? `${formState.min_words || ''}${formState.min_words && formState.max_words ? '-' : ''}${formState.max_words || ''}`
    : '');
  let kudosInput = $state(formState.min_kudos || formState.max_kudos
    ? `${formState.min_kudos || ''}${formState.min_kudos && formState.max_kudos ? '-' : ''}${formState.max_kudos || ''}`
    : '');
  let commentsInput = $state(formState.min_comments || formState.max_comments
    ? `${formState.min_comments || ''}${formState.min_comments && formState.max_comments ? '-' : ''}${formState.max_comments || ''}`
    : '');
  let bookmarksInput = $state(formState.min_bookmarks || formState.max_bookmarks
    ? `${formState.min_bookmarks || ''}${formState.min_bookmarks && formState.max_bookmarks ? '-' : ''}${formState.max_bookmarks || ''}`
    : '');

  const RATING_OPTIONS = [
    { label: 'Not Rated', value: '' },
    { label: 'General Audiences', value: 'General Audiences' },
    { label: 'Teen And Up Audiences', value: 'Teen And Up Audiences' },
    { label: 'Mature', value: 'Mature' },
    { label: 'Explicit', value: 'Explicit' },
  ];

  const WARNING_OPTIONS = [
    'Choose Not To Use Archive Warnings',
    'Graphic Depictions Of Violence',
    'Major Character Death',
    'Rape/Non-Con',
    'Underage',
    'No Archive Warnings Apply',
  ];

  const CATEGORY_OPTIONS = ['F/F', 'F/M', 'Gen', 'M/M', 'Multi', 'Other'];

  const SORT_OPTIONS = [
    { label: 'Best Match', value: '', supported: true },
    { label: 'Creator', value: 'creators', supported: true },
    { label: 'Title', value: 'title_to_sort_on', supported: true },
    { label: 'Date Posted', value: 'created_at', supported: true },
    { label: 'Date Updated', value: 'updated_at', supported: true },
    { label: 'Word Count', value: 'word_count', supported: true },
    { label: 'Kudos', value: 'kudos_count', supported: true },
    { label: 'Comments', value: 'comments_count', supported: true },
    { label: 'Bookmarks', value: 'bookmarks_count', supported: true },
    { label: 'Hits', value: 'hits', supported: true },
  ];

  onMount(async () => {
    try {
      const res = await fetch('/api/search/suggest');
      if (res.ok) {
        const data: Chip[] = await res.json();
        chips = data.slice(0, 8);
      }
    } catch {
      chips = [];
    }
  });

  function update(field: keyof FormState, value: unknown) {
    onChange({ ...formState, [field]: value } as FormState);
  }

  function handleChipClick(chip: Chip) {
    const newState = mergeChipIntoForm(formState, chip);
    onChange(newState);
    onSubmit();
  }

  function handleWordCountBlur() {
    const range = parseRange(wordCountInput);
    onChange({
      ...formState,
      min_words: range.min != null ? String(range.min) : '',
      max_words: range.max != null ? String(range.max) : '',
    });
  }

  function handleKudosBlur() {
    const range = parseRange(kudosInput);
    onChange({
      ...formState,
      min_kudos: range.min != null ? String(range.min) : '',
      max_kudos: range.max != null ? String(range.max) : '',
    });
  }

  function handleCommentsBlur() {
    const range = parseRange(commentsInput);
    onChange({
      ...formState,
      min_comments: range.min != null ? String(range.min) : '',
      max_comments: range.max != null ? String(range.max) : '',
    });
  }

  function handleBookmarksBlur() {
    const range = parseRange(bookmarksInput);
    onChange({
      ...formState,
      min_bookmarks: range.min != null ? String(range.min) : '',
      max_bookmarks: range.max != null ? String(range.max) : '',
    });
  }

  function handleSingleChapterChange(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    update('single_chapter', checked);
  }

  function handleWarningChange(warning: string, checked: boolean) {
    const current = formState.warnings;
    const next = checked
      ? [...current, warning]
      : current.filter((w: string) => w !== warning);
    update('warnings', next);
  }

  function handleCategoryChange(category: string, checked: boolean) {
    const current = formState.categories;
    const next = checked
      ? [...current, category]
      : current.filter((c: string) => c !== category);
    update('categories', next);
  }

  function clearFilters() {
    onChange(defaultFormState());
  }

  function handleFormSubmit(e: Event) {
    e.preventDefault();
    onSubmit();
  }
</script>

{#if mode === 'page'}
  <div class="search-form">
    <div class="search-header">
      <h2>Work Search</h2>
      <div class="search-buttons">
        <ArchiveButton onclick={onSubmit}>Search</ArchiveButton>
      </div>
    </div>

    {#if chips.length > 0}
      <div class="chips-row">
        {#each chips as chip}
          <button class="chip" type="button" onclick={() => handleChipClick(chip)}>
            {chip.name}
          </button>
        {/each}
      </div>
    {/if}

    <form onsubmit={handleFormSubmit}>
      <fieldset class="search-fieldset">
        <legend>Work Info</legend>
        <h3 class="landmark heading">Work Info</h3>
        <dl>
          <div class="dl-row">
            <dt><label for="any-field">Any Field</label></dt>
            <dd><input id="any-field" type="text" value={formState.q} oninput={(e) => update('q', e.currentTarget.value)} placeholder="Search all fields..." /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="title-field">Title</label></dt>
            <dd><input id="title-field" type="text" value={formState.q} oninput={(e) => update('q', e.currentTarget.value)} placeholder="Work title" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="creator-field">Creator</label></dt>
            <dd><input id="creator-field" type="text" value={formState.q} oninput={(e) => update('q', e.currentTarget.value)} placeholder="Author name" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="date-field">Date</label></dt>
            <dd><input id="date-field" type="text" value={formState.date_from} oninput={(e) => update('date_from', e.currentTarget.value)} placeholder="YYYY-MM-DD" /></dd>
          </div>
          <div class="dl-row">
            <dt>Completion</dt>
            <dd class="radio-group">
              <label><input type="radio" name="completion" value="all" checked={formState.complete === ''} onchange={() => update('complete', '')} /> All</label>
              <label><input type="radio" name="completion" value="true" checked={formState.complete === 'true'} onchange={() => update('complete', 'true')} /> Complete</label>
              <label><input type="radio" name="completion" value="false" checked={formState.complete === 'false'} onchange={() => update('complete', 'false')} /> In Progress</label>
            </dd>
          </div>
          <div class="dl-row">
            <dt>Crossovers</dt>
            <dd class="radio-group">
              <label class="disabled"><input type="radio" name="crossovers" disabled /> All</label>
              <label class="disabled"><input type="radio" name="crossovers" disabled /> Crossovers</label>
              <label class="disabled"><input type="radio" name="crossovers" disabled /> No Crossovers</label>
            </dd>
          </div>
          <div class="dl-row">
            <dt>Single Chapter</dt>
            <dd>
              <input
                type="checkbox"
                checked={formState.single_chapter}
                onchange={handleSingleChapterChange}
              />
            </dd>
          </div>
          <div class="dl-row">
            <dt><label for="word-count-field">Word Count</label></dt>
            <dd>
              <input
                id="word-count-field"
                type="text"
                value={wordCountInput}
                oninput={(e) => { wordCountInput = e.currentTarget.value; }}
                onblur={handleWordCountBlur}
                placeholder="e.g. 1000-5000"
              />
            </dd>
          </div>
          <div class="dl-row">
            <dt>Language</dt>
            <dd>
              <select disabled>
                <option value="">All languages</option>
              </select>
            </dd>
          </div>
        </dl>
      </fieldset>

      <fieldset class="search-fieldset">
        <legend>Work Tags</legend>
        <h3 class="landmark heading">Work Tags</h3>
        <dl>
          <div class="dl-row">
            <dt><label for="fandoms-field">Fandoms</label></dt>
            <dd><input id="fandoms-field" type="text" value={formState.fandoms} oninput={(e) => update('fandoms', e.currentTarget.value)} placeholder="Fandom name(s)" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="rating-select">Rating</label></dt>
            <dd>
              <select
                id="rating-select"
                value={formState.rating}
                onchange={(e) => update('rating', e.currentTarget.value)}
              >
                {#each RATING_OPTIONS as r}
                  <option value={r.value}>{r.label}</option>
                {/each}
              </select>
            </dd>
          </div>
          <div class="dl-row warnings-row">
            <dt>Warnings</dt>
            <dd class="checkbox-group">
              {#each WARNING_OPTIONS as warning}
                <label>
                  <input
                    type="checkbox"
                    checked={formState.warnings.includes(warning)}
                    onchange={(e) => handleWarningChange(warning, e.currentTarget.checked)}
                  />
                  {warning}
                </label>
              {/each}
            </dd>
          </div>
          <div class="dl-row">
            <dt>Categories</dt>
            <dd class="checkbox-group">
              {#each CATEGORY_OPTIONS as category}
                <label>
                  <input
                    type="checkbox"
                    checked={formState.categories.includes(category)}
                    onchange={(e) => handleCategoryChange(category, e.currentTarget.checked)}
                  />
                  {category}
                </label>
              {/each}
            </dd>
          </div>
          <div class="dl-row">
            <dt><label for="characters-field">Characters</label></dt>
            <dd><input id="characters-field" type="text" value={formState.characters} oninput={(e) => update('characters', e.currentTarget.value)} placeholder="Character name(s)" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="relationships-field">Relationships</label></dt>
            <dd><input id="relationships-field" type="text" value={formState.relationships} oninput={(e) => update('relationships', e.currentTarget.value)} placeholder="Relationship(s)" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="additional-tags-field">Additional Tags</label></dt>
            <dd><input id="additional-tags-field" type="text" value={formState.additional_tags} oninput={(e) => update('additional_tags', e.currentTarget.value)} placeholder="Tag(s)" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="other-tags-field">Other tags</label></dt>
            <dd><input id="other-tags-field" type="text" value={formState.tag_ids} oninput={(e) => update('tag_ids', e.currentTarget.value)} placeholder="Tag IDs" /></dd>
          </div>
        </dl>
      </fieldset>

      <fieldset class="search-fieldset">
        <legend>Work Stats</legend>
        <h3 class="landmark heading">Work Stats</h3>
        <dl>
          <div class="dl-row">
            <dt>Hits</dt>
            <dd class="disabled-text">Not supported yet.</dd>
          </div>
          <div class="dl-row">
            <dt><label for="kudos-field">Kudos</label></dt>
            <dd>
              <input
                id="kudos-field"
                type="text"
                value={kudosInput}
                oninput={(e) => { kudosInput = e.currentTarget.value; }}
                onblur={handleKudosBlur}
                placeholder="e.g. 100"
              />
            </dd>
          </div>
          <div class="dl-row">
            <dt><label for="comments-field">Comments</label></dt>
            <dd>
              <input
                id="comments-field"
                type="text"
                value={commentsInput}
                oninput={(e) => { commentsInput = e.currentTarget.value; }}
                onblur={handleCommentsBlur}
                placeholder="e.g. 10"
              />
            </dd>
          </div>
          <div class="dl-row">
            <dt><label for="bookmarks-field">Bookmarks</label></dt>
            <dd>
              <input
                id="bookmarks-field"
                type="text"
                value={bookmarksInput}
                oninput={(e) => { bookmarksInput = e.currentTarget.value; }}
                onblur={handleBookmarksBlur}
                placeholder="e.g. 50"
              />
            </dd>
          </div>
        </dl>
      </fieldset>

      <fieldset class="search-fieldset sort-fieldset">
        <legend>Sort</legend>
        <h3 class="landmark heading">Sort</h3>
        <dl>
          <div class="dl-row">
            <dt><label for="sort-select">Sort by</label></dt>
            <dd>
              <select
                id="sort-select"
                value={formState.sort}
                onchange={(e) => update('sort', e.currentTarget.value)}
              >
                {#each SORT_OPTIONS as opt}
                  <option value={opt.value} disabled={!opt.supported}>{opt.label}</option>
                {/each}
              </select>
            </dd>
          </div>
          <div class="dl-row">
            <dt>Sort direction</dt>
            <dd class="radio-group">
              <label>
                <input
                  type="radio"
                  name="sort-dir"
                  value=""
                  checked={formState.dir === ''}
                  disabled={!SORT_OPTIONS.find(o => o.value === formState.sort)?.supported}
                  onchange={() => update('dir', '')}
                />
                Descending
              </label>
              <label>
                <input
                  type="radio"
                  name="sort-dir"
                  value="asc"
                  checked={formState.dir === 'asc'}
                  disabled={!SORT_OPTIONS.find(o => o.value === formState.sort)?.supported}
                  onchange={() => update('dir', 'asc')}
                />
                Ascending
              </label>
            </dd>
          </div>
        </dl>
      </fieldset>

      <div class="search-buttons-bottom">
        <ArchiveButton onclick={onSubmit}>Search</ArchiveButton>
      </div>

      <div class="footnote">
        <p>Fields not supported yet: Crossovers, Hits, Language.</p>
      </div>
    </form>
  </div>

{:else}
  <!-- Sidebar mode -->
  <div class="sidebar-search">
    <form onsubmit={handleFormSubmit}>
      <details open>
        <summary>Sort</summary>
        <div class="sidebar-section">
          <label for="sidebar-sort">Sort by
            <select
              id="sidebar-sort"
              value={formState.sort}
              onchange={(e) => update('sort', e.currentTarget.value)}
            >
              {#each SORT_OPTIONS as opt}
                <option value={opt.value} disabled={!opt.supported}>{opt.label}</option>
              {/each}
            </select>
          </label>
          <div class="radio-group">
            <label>
              <input
                type="radio"
                name="sidebar-sort-dir"
                value=""
                checked={formState.dir === ''}
                disabled={!SORT_OPTIONS.find(o => o.value === formState.sort)?.supported}
                onchange={() => update('dir', '')}
              />
              Desc
            </label>
            <label>
              <input
                type="radio"
                name="sidebar-sort-dir"
                value="asc"
                checked={formState.dir === 'asc'}
                disabled={!SORT_OPTIONS.find(o => o.value === formState.sort)?.supported}
                onchange={() => update('dir', 'asc')}
              />
              Asc
            </label>
          </div>
        </div>
      </details>

      <details>
        <summary>Work Info</summary>
        <div class="sidebar-section">
          <label>
            Any Field
            <input type="text" value={formState.q} oninput={(e) => update('q', e.currentTarget.value)} placeholder="Search all..." />
          </label>
          <label>
            Title
            <input type="text" value={formState.q} oninput={(e) => update('q', e.currentTarget.value)} placeholder="Work title" />
          </label>
          <label>
            Creator
            <input type="text" value={formState.q} oninput={(e) => update('q', e.currentTarget.value)} placeholder="Author" />
          </label>
          <label>
            Date
            <input type="text" value={formState.date_from} oninput={(e) => update('date_from', e.currentTarget.value)} placeholder="YYYY-MM-DD" />
          </label>
          <label class="radio-label">
            Completion
            <div class="radio-group">
              <label><input type="radio" name="sidebar-completion" value="all" checked={formState.complete === ''} onchange={() => update('complete', '')} /> All</label>
              <label><input type="radio" name="sidebar-completion" value="true" checked={formState.complete === 'true'} onchange={() => update('complete', 'true')} /> Complete</label>
              <label><input type="radio" name="sidebar-completion" value="false" checked={formState.complete === 'false'} onchange={() => update('complete', 'false')} /> In Progress</label>
            </div>
          </label>
          <label>
            Word Count
            <input type="text" value={wordCountInput} oninput={(e) => { wordCountInput = e.currentTarget.value; }} onblur={handleWordCountBlur} placeholder="e.g. 1000-5000" />
          </label>
        </div>
      </details>

      <details>
        <summary>Work Tags</summary>
        <div class="sidebar-section">
          <label>
            Fandoms
            <input type="text" value={formState.fandoms} oninput={(e) => update('fandoms', e.currentTarget.value)} placeholder="Fandom(s)" />
          </label>
          <label>
            Rating
            <select value={formState.rating} onchange={(e) => update('rating', e.currentTarget.value)}>
              {#each RATING_OPTIONS as r}
                <option value={r.value}>{r.label}</option>
              {/each}
            </select>
          </label>
          <div class="sidebar-label">Warnings</div>
          <div class="checkbox-group">
            {#each WARNING_OPTIONS as warning}
              <label>
                <input
                  type="checkbox"
                  checked={formState.warnings.includes(warning)}
                  onchange={(e) => handleWarningChange(warning, e.currentTarget.checked)}
                />
                {warning}
              </label>
            {/each}
          </div>
          <div class="sidebar-label">Categories</div>
          <div class="checkbox-group">
            {#each CATEGORY_OPTIONS as category}
              <label>
                <input
                  type="checkbox"
                  checked={formState.categories.includes(category)}
                  onchange={(e) => handleCategoryChange(category, e.currentTarget.checked)}
                />
                {category}
              </label>
            {/each}
          </div>
          <label>
            Characters
            <input type="text" value={formState.characters} oninput={(e) => update('characters', e.currentTarget.value)} placeholder="Character(s)" />
          </label>
          <label>
            Relationships
            <input type="text" value={formState.relationships} oninput={(e) => update('relationships', e.currentTarget.value)} placeholder="Relationship(s)" />
          </label>
          <label>
            Additional Tags
            <input type="text" value={formState.additional_tags} oninput={(e) => update('additional_tags', e.currentTarget.value)} placeholder="Tag(s)" />
          </label>
          <label>
            Other tags
            <input type="text" value={formState.tag_ids} oninput={(e) => update('tag_ids', e.currentTarget.value)} placeholder="Tag IDs" />
          </label>
        </div>
      </details>

      <details>
        <summary>Work Stats</summary>
        <div class="sidebar-section">
          <div class="sidebar-label">Hits <span class="disabled-text">(not supported)</span></div>
          <label>
            Kudos
            <input type="text" value={kudosInput} oninput={(e) => { kudosInput = e.currentTarget.value; }} onblur={handleKudosBlur} placeholder="e.g. 100" />
          </label>
          <label>
            Comments
            <input type="text" value={commentsInput} oninput={(e) => { commentsInput = e.currentTarget.value; }} onblur={handleCommentsBlur} placeholder="e.g. 10" />
          </label>
          <label>
            Bookmarks
            <input type="text" value={bookmarksInput} oninput={(e) => { bookmarksInput = e.currentTarget.value; }} onblur={handleBookmarksBlur} placeholder="e.g. 50" />
          </label>
        </div>
      </details>

      <div class="sidebar-actions">
        <ArchiveButton type="submit">Sort and Filter</ArchiveButton>
        <button type="button" class="clear-link" onclick={clearFilters}>Clear Filters</button>
      </div>
    </form>
  </div>
{/if}

<style>
  .search-form {
    max-width: 960px;
    margin: 0 auto;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #333);
  }

  .search-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .search-header h2 {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.5rem;
    color: #800020;
    margin: 0;
  }

  .chips-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .chip {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #ddd);
    border-radius: 3px;
    padding: 0.25rem 0.75rem;
    font-size: 0.85rem;
    color: var(--archive-link, #900);
    cursor: pointer;
    font-family: Georgia, 'Times New Roman', serif;
  }

  .chip:hover {
    background: var(--archive-bg, #eee);
  }

  .search-fieldset {
    border: 1px solid var(--archive-border, #ddd);
    background: var(--archive-bg-raised, #f8f8f8);
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
  }

  .search-fieldset legend {
    font-size: 0.95rem;
    font-weight: bold;
    color: #800020;
    background: var(--archive-bg-raised, #f8f8f8);
    padding: 0 0.5rem;
    border: 1px solid var(--archive-border, #ddd);
  }

  .dl-row {
    display: flex;
    align-items: flex-start;
    padding: 0.35rem 0;
    border-bottom: 1px solid var(--archive-border, #eee);
  }

  .dl-row:last-child {
    border-bottom: none;
  }

  .dl-row dt {
    flex: 0 0 25%;
    text-align: right;
    padding-right: 0.75rem;
    font-size: 0.85rem;
    color: var(--archive-muted, #666);
    padding-top: 0.25rem;
  }

  .dl-row dd {
    flex: 0 0 75%;
    margin: 0;
    padding-top: 0.15rem;
  }

  .dl-row dd input[type="text"] {
    width: 100%;
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--archive-border, #ddd);
    border-radius: 3px;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.9rem;
    background: var(--archive-bg, #fff);
    color: var(--archive-text, #333);
    box-sizing: border-box;
  }

  .dl-row dd select {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--archive-border, #ddd);
    border-radius: 3px;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.9rem;
    background: var(--archive-bg, #fff);
    color: var(--archive-text, #333);
  }

  .radio-group {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    padding-top: 0.2rem;
  }

  .radio-group label {
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .checkbox-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    padding-top: 0.2rem;
  }

  .checkbox-group label {
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .disabled,
  .disabled-text {
    color: var(--archive-muted, #999);
    font-style: italic;
    font-size: 0.85rem;
  }

  .warnings-row dd.checkbox-group {
    flex-direction: column;
    gap: 0.4rem;
  }

  .search-buttons,
  .search-buttons-bottom {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .footnote {
    font-size: 0.8rem;
    color: var(--archive-muted, #888);
    font-style: italic;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--archive-border, #eee);
  }

  .footnote p {
    margin: 0.25rem 0;
  }

  /* Sidebar mode styles */
  .sidebar-search {
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #333);
    font-size: 0.9rem;
  }

  .sidebar-search details {
    border: 1px solid var(--archive-border, #ddd);
    margin-bottom: 0.5rem;
    background: var(--archive-bg-raised, #f8f8f8);
  }

  .sidebar-search summary {
    padding: 0.5rem 0.75rem;
    font-weight: bold;
    color: #800020;
    cursor: pointer;
    font-size: 0.9rem;
    background: var(--archive-bg, #f0f0f0);
  }

  .sidebar-section {
    padding: 0.5rem 0.75rem;
  }

  .sidebar-section > label {
    display: block;
    font-size: 0.85rem;
    margin-bottom: 0.5rem;
    color: var(--archive-muted, #666);
  }

  .sidebar-section input[type="text"],
  .sidebar-section select {
    width: 100%;
    padding: 0.3rem 0.5rem;
    margin-top: 0.15rem;
    border: 1px solid var(--archive-border, #ddd);
    border-radius: 3px;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.85rem;
    background: var(--archive-bg, #fff);
    color: var(--archive-text, #333);
    box-sizing: border-box;
  }

  .sidebar-section .radio-group {
    padding: 0.25rem 0;
  }

  .sidebar-section .checkbox-group label {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.8rem;
    margin-bottom: 0.2rem;
  }

  .sidebar-label {
    font-size: 0.85rem;
    font-weight: bold;
    color: var(--archive-muted, #666);
    margin-top: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .sidebar-actions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--archive-border, #ddd);
  }

  .clear-link {
    background: none;
    border: none;
    color: var(--archive-link, #900);
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.85rem;
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
    text-align: center;
  }

  .clear-link:hover {
    color: #999;
  }

  .radio-label {
    font-weight: bold;
  }

  /* AO3 landmark headings for fieldset sections */
  h3.landmark.heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin: -0.4em 0 0.2em;
    padding: 0 0.2em;
  }
</style>
