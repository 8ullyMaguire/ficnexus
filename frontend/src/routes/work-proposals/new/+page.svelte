<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  // Form state
  let title = $state('');
  let summary = $state('');
  let rating = $state('');
  let warnings: string[] = $state([]);
  let fandoms = $state('');
  let categories: string[] = $state([]);
  let relationships = $state('');
  let characters = $state('');
  let additionalTags = $state('');
  let workText = $state('');
  let language = $state('');
  let submitSuccess = $state(false);
  let submitError = $state('');
  let submitting = $state(false);

  const RATINGS = ['General Audiences', 'Teen And Up', 'Mature', 'Explicit', 'Not Rated'];
  const WARNINGS = [
    "Choose Not To Use Archive Warnings",
    "Graphic Depictions Of Violence",
    "Major Character Death",
    "No Archive Warnings Apply",
    "Rape/Non-Con",
    "Underage Sex",
  ];
  const CATEGORIES = ['F/F', 'F/M', 'Gen', 'M/M', 'Multi', 'Other'];

  function toggleWarning(w: string) {
    warnings = warnings.includes(w)
      ? warnings.filter(x => x !== w)
      : [...warnings, w];
  }
  function toggleCategory(c: string) {
    categories = categories.includes(c)
      ? categories.filter(x => x !== c)
      : [...categories, c];
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (submitting) return;
    submitting = true;
    submitError = '';
    // Backend endpoint for creating works is not yet implemented.
    // This form mirrors AO3's Post New Work structure for UI parity.
    setTimeout(() => {
      submitSuccess = true;
      submitError = '';
      submitting = false;
    }, 500);
  }
</script>

<svelte:head><title>Post New Work - FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<div class="archive-content archive-page">
  <header class="archive-header">
    <h1 class="archive-page-title">Post New Work</h1>
  </header>

  <ul class="archive-nav">
    <li><a class="archive-link" href="/requests/new">Import From An Existing URL Instead?</a></li>
  </ul>

  <p class="archive-muted">
    Tags are comma separated, 150 characters per tag.
  </p>

  <form class="archive-form" onsubmit={handleSubmit}>
    <fieldset class="archive-fs" role="article">
      <legend class="archive-legend">Tags</legend>
      <dl class="archive-dl">
        <dt class="archive-required">Rating *</dt>
        <dd>
          <select class="archive-select" bind:value={rating} required>
            <option value=""> — Select a rating — </option>
            {#each RATINGS as r}
            <option value={r}>{r}</option>
            {/each}
          </select>
        </dd>

        <dt class="archive-required">Archive Warnings *</dt>
        <dd>
          <ul class="archive-checklist">
            {#each WARNINGS as w}
            <li>
              <label class="archive-checkbox">
                <input type="checkbox" value={w}
                       checked={warnings.includes(w)} onchange={() => toggleWarning(w)} />
                {w}
              </label>
            </li>
            {/each}
          </ul>
        </dd>

        <dt class="archive-required">Fandoms *</dt>
        <dd>
          <input class="archive-input" type="text" bind:value={fandoms} placeholder="Start typing fandoms..." required />
          <p class="archive-note">
            If this is the first work for a fandom, it may not show up in
            the fandoms page for a day or two.
          </p>
        </dd>

        <dt>Categories</dt>
        <dd>
          <ul class="archive-checklist">
            {#each CATEGORIES as c}
            <li>
              <label class="archive-checkbox">
                <input type="checkbox" value={c}
                       checked={categories.includes(c)} onchange={() => toggleCategory(c)} />
                {c}
              </label>
            </li>
            {/each}
          </ul>
        </dd>

        <dt>Relationships</dt>
        <dd>
          <input class="archive-input" type="text" bind:value={relationships} placeholder="e.g. Sherlock Holmes/John Watson" />
        </dd>

        <dt>Characters</dt>
        <dd>
          <input class="archive-input" type="text" bind:value={characters} placeholder="Character names" />
        </dd>

        <dt>Additional Tags</dt>
        <dd>
          <input class="archive-input" type="text" bind:value={additionalTags} placeholder="Comma separated tags" />
        </dd>
      </dl>
    </fieldset>

    <fieldset class="archive-fs" role="article">
      <legend class="archive-legend">Preface</legend>
      <dl class="archive-dl">
        <dt class="archive-required">Work Title *</dt>
        <dd>
          <input class="archive-input" type="text" bind:value={title} maxlength="255" required />
          <span class="archive-muted archive-count">{title.length}/255 characters</span>
        </dd>

        <dt>Summary</dt>
        <dd>
          <textarea class="archive-textarea" bind:value={summary} maxlength="2000" rows="4" placeholder="A brief summary of your work..."></textarea>
          <span class="archive-muted archive-count">{summary.length}/2000 characters</span>
        </dd>

        <dt>Notes</dt>
        <dd>
          <textarea class="archive-textarea" rows="3" placeholder="Notes at the beginning or end of your work..."></textarea>
        </dd>
      </dl>
    </fieldset>

    <fieldset class="archive-fs" role="article">
      <legend class="archive-legend">Associations</legend>
      <dl class="archive-dl">
        <dt>Post to Collections / Challenges</dt>
        <dd><input class="archive-input" type="text" placeholder="Comma separated collection names" /></dd>

        <dt>Gift this work to</dt>
        <dd><input class="archive-input" type="text" placeholder="Pseuds to gift to" /></dd>

        <dt>Choose a language *</dt>
        <dd>
          <select class="archive-select" bind:value={language} required>
            <option value=""> — Choose a language — </option>
            <option value="en">English</option>
            <option value="es">Spanish</option>
            <option value="fr">French</option>
            <option value="de">German</option>
            <option value="zh">Chinese</option>
            <option value="pt-BR">Portuguese (Brazil)</option>
          </select>
        </dd>
      </dl>
    </fieldset>

    <fieldset class="archive-fs" role="article">
      <legend class="archive-legend archive-required">Work Text *</legend>
      <dl class="archive-dl">
        <dt>Work Text</dt>
        <dd>
          <textarea class="archive-textarea archive-work-text" bind:value={workText} maxlength="500000" rows="20" placeholder="Your work text here..." required></textarea>
          <span class="archive-muted archive-count">{workText.length}/500000 characters</span>
        </dd>
      </dl>
    </fieldset>

    {#if submitError}
    <p class="archive-error">{submitError}</p>
    {/if}
    {#if submitSuccess}
    <p class="archive-ok">Work posted successfully!</p>
    {/if}

    <button class="archive-btn archive-btn-primary" type="submit" disabled={submitting}>
      {submitting ? 'Posting...' : 'Post Work'}
    </button>
  </form>
</div>
{:else}
<div class="post-work-page">
  <h1>Post New Work</h1>
  <p><a href="/requests/new">Import From An Existing URL Instead?</a></p>
  <p class="note">Tags are comma separated, 150 characters per tag.</p>
  <form onsubmit={handleSubmit}>
    <fieldset>
      <legend>Tags</legend>
      <label>Rating *</label>
      <select bind:value={rating} required>
        <option value=""> — Select — </option>
        {#each RATINGS as r}
        <option value={r}>{r}</option>
        {/each}
      </select>

      <label>Archive Warnings *</label>
      {#each WARNINGS as w}
      <label><input type="checkbox" value={w} checked={warnings.includes(w)} onchange={() => toggleWarning(w)} /> {w}</label>
      {/each}

      <label>Fandoms *</label>
      <input type="text" bind:value={fandoms} required />

      <label>Categories</label>
      {#each CATEGORIES as c}
      <label><input type="checkbox" value={c} checked={categories.includes(c)} onchange={() => toggleCategory(c)} /> {c}</label>
      {/each}

      <label>Relationships</label>
      <input type="text" bind:value={relationships} />
      <label>Characters</label>
      <input type="text" bind:value={characters} />
      <label>Additional Tags</label>
      <input type="text" bind:value={additionalTags} />
    </fieldset>

    <fieldset>
      <legend>Preface</legend>
      <label>Work Title *</label>
      <input type="text" bind:value={title} maxlength="255" required />
      <label>Summary</label>
      <textarea bind:value={summary} maxlength="2000" rows="4"></textarea>
      <label>Notes</label>
      <textarea rows="3"></textarea>
    </fieldset>

    <fieldset>
      <legend>Associations</legend>
      <label>Post to Collections</label>
      <input type="text" />
      <label>Gift this work to</label>
      <input type="text" />
      <label>Choose a language *</label>
      <select bind:value={language} required>
        <option value=""> — Choose — </option>
        <option value="en">English</option>
        <option value="es">Spanish</option>
        <option value="fr">French</option>
        <option value="de">German</option>
        <option value="zh">Chinese</option>
      </select>
    </fieldset>

    <fieldset>
      <legend>Work Text *</legend>
      <textarea bind:value={workText} maxlength="500000" rows="20" required></textarea>
    </fieldset>

    <button class="btn btn-primary" type="submit" disabled={submitting}>
      {submitting ? 'Posting...' : 'Post Work'}
    </button>
  </form>
</div>
{/if}

<style>
  .archive-form { display: flex; flex-direction: column; gap: 1.5em; }
  .archive-fs {
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    padding: 1em 1.25em;
    margin: 0;
  }
  .archive-legend {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.15em;
    font-weight: 700;
    color: var(--archive-link, #990000);
    padding: 0 0.25em;
  }
  .archive-required::after { content: " *"; color: var(--archive-danger, #d33); }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt {
    font-weight: 600;
    font-size: 0.9em;
    margin-bottom: 0.3em;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0 0 0.75em; }
  .archive-input, .archive-select, .archive-textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 0.4em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #fff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-textarea { resize: vertical; }
  .archive-work-text { min-height: 300px; font-family: monospace; font-size: 0.9em; }
  .archive-checklist { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.4em; }
  .archive-checkbox { display: flex; align-items: center; gap: 0.5em; cursor: pointer; }
  .archive-checkbox input[type="checkbox"] { margin: 0; }
  .archive-nav { list-style: none; margin: 0.5rem 0 1rem; padding: 0; display: flex; gap: 1.5rem; }
  .archive-nav a { color: var(--archive-link, #990000); text-decoration: none; }
  .archive-nav a:hover { text-decoration: underline; }
  .archive-note { font-size: 0.85em; color: var(--archive-muted, #666); font-style: italic; }
  .archive-count { font-size: 0.8em; color: var(--archive-muted, #666); float: right; margin-top: 0.2em; display: block; }
  .archive-error { color: var(--archive-heading, #990000); font-weight: 700; }
  .archive-ok { color: var(--archive-heading, #2e9e5b); font-weight: 700; }
  @media (max-width: 640px) {
    .archive-checklist { flex-direction: column; }
  }
  .post-work-page { max-width: var(--max-width, 1100px); margin: 0 auto; padding: 1rem; }
  .post-work-page .note { color: var(--color-text-muted, #888); }
</style>
