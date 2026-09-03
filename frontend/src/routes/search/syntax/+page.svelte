<script lang="ts">
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  // ── Syntax reference data (mirrors docs/src/searching.md) ──────────────
  const OPERATORS: { op: string; example: string; meaning: string }[] = [
    { op: 'AND', example: 'harry AND slytherin', meaning: 'Both terms must appear' },
    { op: 'OR', example: 'fluff OR humor', meaning: 'At least one term must appear' },
    { op: 'NOT', example: 'angst NOT fluff', meaning: 'First term appears, second must not' },
    { op: '- (minus)', example: 'angst -fluff', meaning: 'Excludes the term after the minus' },
    { op: 'Implicit AND', example: 'harry potter', meaning: 'Same as harry AND potter' },
  ];

  const FIELDS: { field: string; example: string; meaning: string }[] = [
    { field: 'title:', example: 'title:harry', meaning: 'Matches the story title' },
    { field: 'author:', example: 'author:rowling', meaning: 'Matches the author name' },
    { field: 'fandom:', example: 'fandom:harry potter', meaning: 'Matches a fandom tag' },
    { field: 'character:', example: 'character:hermione', meaning: 'Matches a character tag' },
    { field: 'relationship:', example: 'relationship:draco/hermione', meaning: 'Matches a relationship tag' },
  ];

  const EXAMPLES: { desc: string; query: string }[] = [
    { desc: 'Mix operators freely', query: '(fluff OR humor) AND angst -sad' },
    { desc: 'Quoted phrases match as a whole', query: '"enemies to lovers" AND angst' },
    { desc: 'Fielded search alongside operators', query: 'title:harry AND author:rowling' },
    { desc: 'Exclude a character from a fandom', query: 'fandom:"harry potter" -character:draco' },
  ];

  const TAG_TYPES: { id: string; name: string }[] = [
    { id: '1', name: 'Fandom' },
    { id: '2', name: 'Character' },
    { id: '3', name: 'Relationship' },
    { id: '4', name: 'Freeform' },
    { id: '5', name: 'Warning' },
    { id: '6', name: 'Category' },
  ];

  function searchHref(query: string): string {
    return `/search?q=${encodeURIComponent(query)}`;
  }
</script>

<svelte:head><title>Search Syntax — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Search Syntax</h1>
      <blockquote class="archive-summary">
        FicNexus's search supports boolean operators, quoted phrases, field-specific terms,
        and exclusions. Everything is <strong>case-insensitive</strong>, and multiple words
        with no operator are combined with <strong>AND</strong> automatically.
      </blockquote>
    </header>

    <!-- Boolean operators -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Boolean Operators</legend>
      <table class="archive-table">
        <thead>
          <tr>
            <th scope="col">Operator</th>
            <th scope="col">Example</th>
            <th scope="col">What it does</th>
          </tr>
        </thead>
        <tbody>
          {#each OPERATORS as row (row.op)}
            <tr>
              <td><code>{row.op}</code></td>
              <td><a class="archive-link" href={searchHref(row.example)}><code>{row.example}</code></a></td>
              <td>{row.meaning}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="archive-muted">You can mix them freely: <code>(fluff OR humor) AND angst -sad</code></p>
    </fieldset>

    <!-- Quoted phrases -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Quoted Phrases</legend>
      <p>
        Put quotes around a phrase to match it as a whole:
        <a class="archive-link" href={searchHref('"slow burn"')}><code>"slow burn"</code></a>,
        <a class="archive-link" href={searchHref('"enemies to lovers" AND angst')}><code>"enemies to lovers" AND angst</code></a>.
      </p>
    </fieldset>

    <!-- Parentheses -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Parentheses</legend>
      <p>
        Group terms to control precedence, just like math:
        <a class="archive-link" href={searchHref('(fluff OR humor) AND angst')}><code>(fluff OR humor) AND angst</code></a>,
        <a class="archive-link" href={searchHref('"slow burn" OR (hurt AND comfort)')}><code>"slow burn" OR (hurt AND comfort)</code></a>.
      </p>
    </fieldset>

    <!-- Fielded search -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Fielded Search</legend>
      <p class="archive-muted">Narrow a term to a specific field:</p>
      <dl class="archive-dl">
        {#each FIELDS as row (row.field)}
          <div class="dl-row">
            <dt><code>{row.field}</code></dt>
            <dd>
              <a class="archive-link" href={searchHref(row.example)}><code>{row.example}</code></a>
              — {row.meaning}
            </dd>
          </div>
        {/each}
      </dl>
      <p class="archive-muted">Fielded searches work alongside boolean operators:</p>
      <ul class="archive-list">
        <li><a class="archive-link" href={searchHref('title:harry AND author:rowling')}><code>title:harry AND author:rowling</code></a></li>
        <li><a class="archive-link" href={searchHref('fandom:"harry potter" -character:draco')}><code>fandom:"harry potter" -character:draco</code></a></li>
      </ul>
    </fieldset>

    <!-- Worked examples -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Examples</legend>
      <dl class="archive-dl">
        {#each EXAMPLES as row (row.desc)}
          <div class="dl-row">
            <dt>{row.desc}</dt>
            <dd><a class="archive-link" href={searchHref(row.query)}><code>{row.query}</code></a></dd>
          </div>
        {/each}
      </dl>
    </fieldset>

    <!-- Tag type IDs (for include_tags filters) -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Tag Type IDs</legend>
      <p class="archive-muted">
        Used by tag filters (<code>include_tags</code> / <code>exclude_tags</code>), e.g. <code>1:Harry Potter</code>:
      </p>
      <table class="archive-table">
        <thead>
          <tr>
            <th scope="col">ID</th>
            <th scope="col">Tag type</th>
          </tr>
        </thead>
        <tbody>
          {#each TAG_TYPES as row (row.id)}
            <tr>
              <td>{row.id}</td>
              <td>{row.name}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </fieldset>

    <p class="archive-muted back-link">
      <a class="archive-link" href="/search">← Advanced search</a>
      ·
      <a class="archive-link" href="/search/body">Search inside fics</a>
      ·
      <a class="archive-link" href="/search/tags">Search tags</a>
    </p>
  </div>
</main>
{:else}
<!-- Modern mode -->
<div class="syntax-page">
  <h1>Search Syntax</h1>
  <p class="muted subtitle">
    FicNexus's search supports boolean operators, quoted phrases, field-specific terms,
    and exclusions. Everything is <strong>case-insensitive</strong>, and multiple words
    with no operator are combined with <strong>AND</strong> automatically.
  </p>

  <section class="card">
    <h2>Boolean Operators</h2>
    <table class="syntax-table">
      <thead>
        <tr><th>Operator</th><th>Example</th><th>What it does</th></tr>
      </thead>
      <tbody>
        {#each OPERATORS as row (row.op)}
          <tr>
            <td><code>{row.op}</code></td>
            <td><a href={searchHref(row.example)}><code>{row.example}</code></a></td>
            <td>{row.meaning}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="muted">You can mix them freely: <code>(fluff OR humor) AND angst -sad</code></p>
  </section>

  <section class="card">
    <h2>Quoted Phrases</h2>
    <p>
      Put quotes around a phrase to match it as a whole:
      <a href={searchHref('"slow burn"')}><code>"slow burn"</code></a>,
      <a href={searchHref('"enemies to lovers" AND angst')}><code>"enemies to lovers" AND angst</code></a>.
    </p>
  </section>

  <section class="card">
    <h2>Parentheses</h2>
    <p>
      Group terms to control precedence, just like math:
      <a href={searchHref('(fluff OR humor) AND angst')}><code>(fluff OR humor) AND angst</code></a>,
      <a href={searchHref('"slow burn" OR (hurt AND comfort)')}><code>"slow burn" OR (hurt AND comfort)</code></a>.
    </p>
  </section>

  <section class="card">
    <h2>Fielded Search</h2>
    <p class="muted">Narrow a term to a specific field:</p>
    <table class="syntax-table">
      <thead>
        <tr><th>Field</th><th>Example</th><th>What it does</th></tr>
      </thead>
      <tbody>
        {#each FIELDS as row (row.field)}
          <tr>
            <td><code>{row.field}</code></td>
            <td><a href={searchHref(row.example)}><code>{row.example}</code></a></td>
            <td>{row.meaning}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="muted">Fielded searches work alongside boolean operators:</p>
    <ul>
      <li><a href={searchHref('title:harry AND author:rowling')}><code>title:harry AND author:rowling</code></a></li>
      <li><a href={searchHref('fandom:"harry potter" -character:draco')}><code>fandom:"harry potter" -character:draco</code></a></li>
    </ul>
  </section>

  <section class="card">
    <h2>Examples</h2>
    <ul>
      {#each EXAMPLES as row (row.desc)}
        <li>{row.desc}: <a href={searchHref(row.query)}><code>{row.query}</code></a></li>
      {/each}
    </ul>
  </section>

  <section class="card">
    <h2>Tag Type IDs</h2>
    <p class="muted">
      Used by tag filters (<code>include_tags</code> / <code>exclude_tags</code>), e.g. <code>1:Harry Potter</code>:
    </p>
    <table class="syntax-table">
      <thead>
        <tr><th>ID</th><th>Tag type</th></tr>
      </thead>
      <tbody>
        {#each TAG_TYPES as row (row.id)}
          <tr><td>{row.id}</td><td>{row.name}</td></tr>
        {/each}
      </tbody>
    </table>
  </section>

  <p class="muted back-link">
    <a href="/search">← Advanced search</a>
    ·
    <a href="/search/body">Search inside fics</a>
    ·
    <a href="/search/tags">Search tags</a>
  </p>
</div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-header {
    padding: 0 0 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1rem;
  }
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0 0 0.3rem;
  }
  .archive-summary {
    margin: 0.5rem 0 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend,
  .archive-legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .archive-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.92rem;
    margin: 0.3rem 0;
  }
  .archive-table th,
  .archive-table td {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.35rem 0.6rem;
    text-align: left;
    vertical-align: top;
  }
  .archive-table thead th {
    background: var(--archive-bg-raised, #f5f5f5);
    font-weight: 700;
  }
  .archive-table tbody tr:nth-child(odd) {
    background: var(--archive-bg-raised, #fafafa);
  }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.4rem 0;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-list {
    list-style: none;
    margin: 0.2rem 0 0;
    padding: 0;
  }
  .archive-list li {
    padding: 0.25rem 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
    font-size: 0.92rem;
  }
  .archive-list li:last-child { border-bottom: none; }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .archive-link:hover { text-decoration: underline; }
  code {
    font-family: 'Courier New', Courier, monospace;
    font-size: 0.9em;
    background: var(--archive-bg-raised, #f5f5f5);
    padding: 0.05rem 0.25rem;
    border: 1px solid var(--archive-border, #dddddd);
  }
  .archive-link code { background: transparent; border: none; }
  .back-link { margin-top: 1.2rem; }

  /* ── Modern mode ── */
  .syntax-page {
    max-width: 860px;
    margin: 0 auto;
    padding: 1rem;
  }
  .subtitle { margin-top: 0; margin-bottom: 1rem; }
  .syntax-page .card {
    border: 1px solid var(--border, #ddd);
    border-radius: 8px;
    padding: 1rem 1.2rem;
    margin-bottom: 1rem;
    background: var(--bg-card, #fff);
  }
  .syntax-page h2 {
    margin: 0 0 0.6rem;
    font-size: 1.15rem;
  }
  .syntax-page .muted {
    color: var(--color-muted, #666);
  }
  .syntax-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.92rem;
    margin: 0.4rem 0;
  }
  .syntax-table th,
  .syntax-table td {
    border: 1px solid var(--border, #ddd);
    padding: 0.35rem 0.6rem;
    text-align: left;
    vertical-align: top;
  }
  .syntax-table thead th {
    background: var(--color-surface-2, #f5f5f5);
  }
  .syntax-page code {
    background: var(--color-surface-2, #f5f5f5);
    padding: 0.05rem 0.3rem;
    border-radius: 4px;
    font-size: 0.9em;
  }
  .syntax-page a { color: var(--color-primary, #990000); }
  .back-link { margin-top: 1.2rem; }
</style>
