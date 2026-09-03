<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { getFandoms, type Fandom } from '$lib/api/fandom';

  const uiMode = $derived(getPref('uiMode'));

  let fandoms = $state<Fandom[]>([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true;
    try {
      const res = await getFandoms();
      if (res.err === 0) fandoms = res.fandoms;
      else error = 'Failed to load fandoms.';
    } catch {
      error = 'Network error loading fandoms.';
    } finally {
      loading = false;
    }
  });

  // Group fandoms by first letter
  const fandomsByLetter = $derived.by(() => {
    const groups: Record<string, Fandom[]> = {};
    for (const f of fandoms) {
      const letter = f.name.charAt(0).toUpperCase() || '#';
      if (!groups[letter]) groups[letter] = [];
      groups[letter].push(f);
    }
    // Sort each group alphabetically
    for (const key of Object.keys(groups)) {
      groups[key].sort((a, b) => a.name.localeCompare(b.name));
    }
    return groups;
  });

  const alphabet = $derived(Object.keys(fandomsByLetter).sort((a, b) => {
    if (a === '#') return -1;
    if (b === '#') return 1;
    return a.localeCompare(b);
  }));
</script>
<svelte:head>
  <title>Browse Fandoms — FicHub</title>
  <meta name="description" content="Browse all fandoms on FicHub — anime, books, cartoons, comics, games, movies and TV." />
</svelte:head>


{#if uiMode === 'archive'}
  <!-- AO3-style alphabetical fandom index -->
  <div class="archive-fandoms">
    <h2 class="archive-heading">Fandoms</h2>

    {#if loading}
      <p class="archive-loading">Loading fandoms…</p>
    {:else if error}
      <p class="archive-error">{error}</p>
    {:else if fandoms.length === 0}
      <p class="archive-empty">No fandoms found.</p>
    {:else}
      <!-- Alphabet navigation -->
      <nav class="alphabet-nav" aria-label="Alphabetical navigation">
        <h3 class="landmark heading">Alphabet Navigation</h3>
        <ul class="alphabet-list">
          {#each alphabet as letter}
            <li><a href={"#letter-" + letter}>{letter}</a></li>
        {/each}
        </ul>
      </nav>

      <h3 class="landmark heading">Alphabetised List of Fandoms</h3>
      <ol class="fandom-index">
        {#each alphabet as letter}
          <li id={"letter-" + letter} class="letter-group">
            <h4 class="letter-heading">
              {letter}
              <a class="top-link" href="#alphabet" title="top">↑</a>
            </h4>
            <ul class="fandom-list">
              {#each fandomsByLetter[letter] as fandom (fandom.slug)}
                <li>
                  <a class="fandom-link" href={`/fandom/${fandom.slug}`}>{fandom.name}</a>
                </li>
              {/each}
            </ul>
          </li>
        {/each}
      </ol>
    {/if}
  </div>
{:else}
  <!-- Modern mode: existing card grid -->
  <div class="fandoms-page">
    <h1>📚 Fandoms</h1>

    {#if loading}
      <div class="card">
        <p class="muted"><span class="spinner"></span> Loading fandoms…</p>
      </div>
    {:else if error}
      <div class="card error-card">
        <strong class="error-text">⚠️ {error}</strong>
      </div>
    {:else if fandoms.length === 0}
      <div class="card empty">
        <p class="muted">No fandom data yet — tag some fics!</p>
      </div>
    {:else}
      <div class="controls">
        <span class="fandom-count">{fandoms.length} fandoms</span>
      </div>

      <div class="fandom-grid">
        {#each fandoms as fandom (fandom.slug)}
          <a class="fandom-card card" href={`/fandom/${fandom.slug}`}>
            <h3 class="fandom-name">{fandom.name}</h3>
            <div class="fandom-stats">
              <span class="stat">
                <strong>{fandom.fic_count}</strong> fics
              </span>
            </div>
          </a>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  /* ── Archive mode styles ── */
  .archive-fandoms {
    max-width: var(--archive-max-width, 900px);
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

  .alphabet-nav {
    margin-bottom: 1.5em;
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
    color: #660066;
  }

  .fandom-index {
    list-style: none;
    margin: 0;
    padding: 0;
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

  .top-link {
    margin-left: 0.5em;
    font-size: 0.8em;
    color: var(--archive-muted, #666666);
    text-decoration: none;
  }

  .top-link:hover {
    color: var(--archive-link, #990000);
  }

  .fandom-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.3em;
  }

  .fandom-link {
    display: block;
    padding: 0.25em 0.5em;
    font-size: 0.92em;
    color: var(--archive-link, #990000);
    text-decoration: none;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg, #ffffff);
  }

  .fandom-link:hover {
    background: var(--archive-bg-raised, #f5f5f5);
    text-decoration: underline;
  }

  .landmark.heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin: 0 0 0.5em;
  }

  /* ── Modern mode styles (existing, slimmed) ── */
  .fandoms-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .fandom-count { font-size: 0.9rem; color: var(--color-muted); }
  .fandom-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.8rem;
  }
  .fandom-card {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 1rem;
    text-decoration: none;
    color: inherit;
    transition: border-color 0.15s;
  }
  .fandom-card:hover { border-color: var(--color-primary); text-decoration: none; }
  .fandom-name { font-size: 1rem; font-weight: 600; margin: 0; }
  .fandom-stats { display: flex; gap: 1rem; font-size: 0.82rem; color: var(--color-muted); }
  .stat strong { color: var(--color-text); }
  .empty { text-align: center; padding: 3rem 1rem; }
  .error-card { border-color: var(--color-error); }
</style>
