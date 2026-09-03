<script lang="ts">
  import { onMount } from 'svelte';
  import { getUpdates, markAllUpdatesSeen } from '$lib/api/social';
  import type { FollowedWorkUpdate } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { formatWords, relativeTime } from '$lib/util';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let items = $state<FollowedWorkUpdate[]>([]);
  let loading = $state(true);
  let error = $state('');
  let seenMarked = $state(false);

  onMount(async () => {
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadUpdates();
  });

  async function loadUpdates() {
    loading = true; error = '';
    try {
      const res = await getUpdates();
      if (res.err === 0) {
        items = res.items;
        // Mark everything as seen once so the NEW badges clear (and the nav
        // count drops). Fire-and-forget: the feed itself is the "read" action.
        if (!seenMarked && items.length > 0) {
          seenMarked = true;
          markAllUpdatesSeen(items).catch(() => {});
        }
      } else {
        error = 'Failed to load updates.';
      }
    } catch {
      error = 'Network error loading updates.';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Updates | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Updates</h1>
      <p class="archive-muted archive-sub">Works you follow, newest updates first. NEW marks fics updated since your last visit.</p>
    </header>

    {#if loading}
      <p class="archive-loading">Loading updates…</p>
    {:else if error}
      <p class="archive-error-text">{error}</p>
    {:else if items.length === 0}
      <blockquote class="archive-summary">
        <p>Nothing here yet.</p>
        <p>Follow a fic from its page and updates will show up here.</p>
      </blockquote>
    {:else}
      <dl class="archive-dl updates-dl">
        {#each items as item (item.follow_id)}
          <div class="dl-row">
            <dt>
              {#if item.is_new}
                <span class="new-badge">NEW</span>
              {/if}
              <a class="archive-link update-title" href={`/works/${encodeURIComponent(item.url_id)}`}>{item.title}</a>
            </dt>
            <dd>
              <span class="archive-muted">by {item.author}</span>
              <span class="update-meta">
                {formatWords(item.words)} words · {item.chapters} chapters · {item.status} ·
                updated {relativeTime(item.fic_updated)}
              </span>
              <a class="archive-btn read-link" href={`/read/${encodeURIComponent(item.url_id)}`}>Read</a>
            </dd>
          </div>
        {/each}
      </dl>
    {/if}
  </div>
</main>
{:else}
<div class="updates-page">
  <h1>📬 Updates</h1>
  <p class="muted">Works you follow, newest updates first. NEW marks fics updated since your last visit.</p>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading updates…</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {:else if items.length === 0}
    <div class="card empty">
      <p class="muted">Nothing here yet.</p>
      <p class="muted">Follow a fic from its page and updates will show up here.</p>
    </div>
  {:else}
    <div class="update-list">
      {#each items as item (item.follow_id)}
        <div class="card update-item">
          {#if item.is_new}
            <span class="new-badge">NEW</span>
          {/if}
          <a class="update-title" href={`/works/${encodeURIComponent(item.url_id)}`}>
            {item.title}
          </a>
          <span class="update-author muted">by {item.author}</span>
          <span class="update-meta muted">
            {formatWords(item.words)} words · {item.chapters} chapters · {item.status} ·
            updated {relativeTime(item.fic_updated)}
          </span>
          <a class="read-link" href={`/read/${encodeURIComponent(item.url_id)}`}>Read →</a>
        </div>
      {/each}
    </div>
  {/if}
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
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
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
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-sub {
    margin: 0.4em 0 0;
    font-style: italic;
  }
  .archive-loading,
  .archive-muted {
    color: var(--archive-muted, #666666);
  }
  .archive-error-text {
    color: var(--color-error, #cc0000);
    font-weight: 600;
  }
  .archive-summary {
    border-left: 3px solid var(--archive-border, #dddddd);
    margin: 1rem 0;
    padding: 0.6rem 1rem;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-summary p { margin: 0 0 0.4em; }
  .archive-summary p:last-child { margin-bottom: 0; }
  .updates-dl { margin: 0; }
  .updates-dl .dl-row {
    padding: 0.55em 0.25em;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .updates-dl .dl-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .updates-dl .dl-row:last-child { border-bottom: none; }
  .updates-dl dt {
    display: flex;
    align-items: center;
    gap: 0.6em;
    flex-wrap: wrap;
    font-weight: 700;
    font-size: 0.98em;
  }
  .new-badge {
    background: var(--archive-link, #990000);
    color: #ffffff;
    font-size: 0.68em;
    font-weight: 800;
    letter-spacing: 0.05em;
    padding: 0.15em 0.45em;
  }
  .update-title.archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 700;
  }
  .update-title.archive-link:hover { text-decoration: underline; }
  .updates-dl dd {
    margin: 0.15em 0 0;
    display: flex;
    align-items: center;
    gap: 0.8em;
    flex-wrap: wrap;
    font-size: 0.88em;
  }
  .update-meta {
    color: var(--archive-muted, #666666);
  }
  a.read-link.archive-btn,
  .read-link {
    margin-left: auto;
    padding: 0.22rem 0.65rem;
    font-size: 0.85em;
    font-family: inherit;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a) !important;
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
    text-decoration: none;
  }
  .read-link:hover {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
    text-decoration: none !important;
  }

  /* ── Modern mode ── */
  .updates-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .updates-page .empty { text-align: center; padding: 2rem; }
  .updates-page .error-card { border-color: var(--color-error); }
  .updates-page .update-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .updates-page .update-item { display: flex; flex-wrap: wrap; align-items: center; gap: 0.6rem; padding: 0.8rem 1rem; }
  .updates-page .new-badge {
    background: var(--color-primary);
    color: white;
    font-size: 0.7rem;
    font-weight: 800;
    letter-spacing: 0.05em;
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
  }
  .updates-page .update-title { font-weight: 600; text-decoration: none; color: var(--color-text); }
  .updates-page .update-title:hover { color: var(--color-primary); }
  .updates-page .update-author { font-size: 0.88rem; }
  .updates-page .update-meta { font-size: 0.82rem; width: 100%; }
  .updates-page .read-link { margin-left: auto; font-size: 0.85rem; }
</style>
