<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { listMyWorks } from '$lib/api/social';
  import type { MyWork } from '$lib/api/social';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let loading = $state(true);
  let error = $state('');
  let works: MyWork[] = $state([]);

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      return;
    }
    await load();
  });

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await listMyWorks();
      if (res.err === 0) {
        works = res.works ?? [];
      } else {
        error = res.msg || 'Failed to load your works.';
      }
    } catch (e) {
      error = (e instanceof Error) ? e.message : 'Network error loading my works.';
    } finally {
      loading = false;
    }
  }

  // Convert MyWork -> WorkBlurb fic shape
  function ficFor(w: MyWork) {
    return {
      url_id: w.default_source_id ?? `#work-${w.id}`,
      // WorkBlurb links to /works/{url_id} when url_id looks numeric; fall
      // back to the work id so the card still links somewhere useful.
      title: w.canonical_title,
      author: w.canonical_author,
      words: null,
      chapters: null,
      source: null,
      status: w.is_visible === false ? 'pending' : 'visible',
      description: '',
      updated: w.updated_at ?? w.created_at ?? null,
      tags: [],
    };
  }

  function statusBadge(w: MyWork) {
    if (w.is_visible === false) {
      return { label: 'Pending', cls: 'badge-pending' };
    }
    return { label: 'Visible', cls: 'badge-visible' };
  }

  const goto = (href: string) => {
    if (typeof location !== 'undefined') location.href = href;
  };
</script>

<svelte:head><title>My Works — FicNexus</title></svelte:head>

{#if !auth.isLoggedIn}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">My Works</h1>
      </header>
      <div class="archive-empty">
        <p>Log in to see the works you've uploaded.</p>
        <a class="archive-link" href="/login">Log In</a>
      </div>
    </div>
  </main>
{:else if uiMode === 'archive' || !uiMode}
  <!-- Archive mode (AO3-style) -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">My Works</h1>
        <p class="archive-summary">Stories you have uploaded.</p>
      </header>

      {#if loading}
        <div class="archive-skeleton-list" aria-hidden="true">
          {#each Array(5) as _, i}
            <article class="skeleton-blurb">
              <div class="skel-header"><span class="skel-badge"></span><span class="skel-title"></span></div>
              <span class="skel-byline"></span>
              <div class="skel-stats"></div>
            </article>
          {/each}
        </div>
      {:else if error}
        <div class="archive-error"><p>{error}</p></div>
      {:else if works.length === 0}
        <div class="archive-empty">
          <p>You haven't uploaded any works yet.</p>
          <ArchiveButton href="/upload">Upload your first work</ArchiveButton>
        </div>
      {:else}
        <div class="archive-work-list">
          {#each works as w (w.id)}
            {@const s = statusBadge(w)}
            <article class="archive-blurb">
              <span class="status-badge {s.cls}">{s.label}</span>
              <WorkBlurb fic={ficFor(w)} />
            </article>
          {/each}
        </div>
      {/if}
    </div>
  </main>
{:else}
  <!-- Modern mode -->
  <div class="myworks-page">
    <h1>My Works</h1>
    {#if loading}
      <p class="muted">Loading…</p>
    {:else if error}
      <div class="error-card"><p>{error}</p></div>
    {:else if works.length === 0}
      <p class="muted">You haven't uploaded any works yet.</p>
      <ArchiveButton href="/upload">Upload your first work</ArchiveButton>
    {:else}
      <ul class="mw-list">
        {#each works as w (w.id)}
          <li class="mw-item">
            <a class="mw-link" href={`/works/${w.default_source_id ?? w.id}`}>{w.canonical_title}</a>
            <span class="mw-byline"> by {w.canonical_author}</span>
            <span class="mw-status">{w.is_visible === false ? 'pending' : 'visible'}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-content {
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8em;
    font-weight: normal;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-summary {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 1em;
    margin: 0.5em 0 1em;
    font-style: italic;
    color: var(--archive-muted, #666666);
  }
  .archive-skeleton-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .skeleton-blurb {
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding: 0.8rem 0;
  }
  .skel-header { display: flex; gap: 0.3rem; align-items: center; }
  .skel-badge { width: 2rem; height: 0.9rem; background: var(--archive-border, #dddddd); border-radius: 2px; }
  .skel-title { width: 40%; height: 1.2rem; background: var(--archive-border, #dddddd); border-radius: 2px; flex: 1; }
  .skel-byline { width: 20%; height: 0.8rem; background: var(--archive-border, #dddddd); border-radius: 2px; display: block; margin-top: 0.3rem; }
  .skel-stats { width: 15%; height: 0.8rem; background: var(--archive-border, #dddddd); border-radius: 2px; margin-top: 0.3rem; }
  .archive-error {
    padding: 1.5em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
  }
  .archive-empty {
    padding: 2em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-empty p {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    margin: 0 0 0.5em;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .archive-link:hover { text-decoration: underline; }
  .archive-work-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .archive-blurb {
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    padding: 0.7em 1em;
    margin-bottom: 0.6em;
    position: relative;
  }
  .status-badge {
    position: absolute;
    top: 0.5em;
    right: 0.75em;
    font-size: 0.72em;
    font-weight: 700;
    padding: 0.12em 0.45em;
    border-radius: 2px;
    color: #fff;
  }
  .status-badge.badge-pending { background: #924000; }
  .status-badge.badge-visible { background: #0a7b30; }

  /* ── Modern mode ──────────────────────────────────────────────── */
  .myworks-page {
    max-width: var(--max-width, 1100px);
    margin: 0 auto;
    padding: 1rem;
  }
  .muted { color: var(--color-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid #d33; border-radius: 8px; background: #fee; }
  .mw-list { list-style: none; margin: 0; padding: 0; }
  .mw-item { padding: 0.5em 0; border-bottom: 1px solid var(--archive-border, #eeeeee); }
  .mw-link { font-weight: 700; color: var(--archive-link, #990000); text-decoration: none; }
  .mw-link:hover { text-decoration: underline; }
  .mw-byline { color: var(--archive-muted, #666666); }
  .mw-status { float: right; font-size: 0.8em; color: var(--archive-muted, #666666); text-transform: uppercase; }
</style>
