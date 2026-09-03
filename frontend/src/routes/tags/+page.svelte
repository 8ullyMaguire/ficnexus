<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { auth } from '$lib/stores/auth.svelte';
  import {
    fetchTagsByType,
    TAG_TYPE_LABELS,
    TAG_TYPE_NAMES,
    TAG_TYPE_ORDER,
    type TagItem,
  } from '$lib/api/tags';

  const uiMode = $derived(getPref('uiMode'));

  // Tags grouped by type_id
  let tagsByType = $state<Record<number, TagItem[]>>({});
  let loading = $state(true);
  let error = $state('');
  let showMode = $state<'popular' | 'random'>('popular');
  let personalized = $state(false);
  let personalTagNames = $state<Set<string>>(new Set());

  async function isPersonalizationEnabled(): Promise<boolean> {
    if (!auth.isLoggedIn) return false;
    // Respect explicit opt-out via localStorage pref (cast because prefs.ts may not have the key typed)
    try {
      const local = (getPref as unknown as (k: string) => unknown)('personalizedRecs');
      if (local === false) return false;
      if (local === true) return true;
    } catch {
      // ignore
    }
    // Check server-side pref recs.personalized (settings page uses /api/me/prefs)
    try {
      const res = await fetch('/api/me/prefs', { credentials: 'include' });
      if (res.ok) {
        const data = await res.json();
        const arr: unknown = Array.isArray(data) ? data : (data as { preferences?: unknown }).preferences ?? data;
        if (Array.isArray(arr)) {
          const found = (arr as Array<{ key: string; value: string }>).find((p) => p.key === 'recs.personalized');
          if (found) return found.value !== 'false';
        }
      }
    } catch {
      // degrade gracefully
    }
    // Fallback to /api/preferences (used by some settings flows)
    try {
      const res2 = await fetch('/api/preferences', { credentials: 'include' });
      if (res2.ok) {
        const data2 = await res2.json();
        const arr2 = (data2 as { preferences?: Array<{ key: string; value: string }> }).preferences ?? [];
        if (Array.isArray(arr2)) {
          const found2 = arr2.find((p) => p.key === 'recs.personalized');
          if (found2) return found2.value !== 'false';
        }
      }
    } catch {
      // ignore
    }
    return true;
  }

  async function fetchPersonalTagNames(): Promise<Set<string>> {
    const set = new Set<string>();
    // Primary source: user's bookmark tags (bookmark search includes tags per bookmark)
    try {
      const res = await fetch('/api/bookmarks/search?per_page=50', { credentials: 'include' });
      if (res.ok) {
        const data = await res.json();
        const bookmarks: Array<{ tags?: Array<{ name: string; type: string } | string> }> =
          data.bookmarks ?? data.items ?? [];
        for (const b of bookmarks) {
          for (const t of b.tags ?? []) {
            if (typeof t === 'string') set.add(t.toLowerCase());
            else if (t && typeof (t as { name?: string }).name === 'string')
              set.add((t as { name: string }).name.toLowerCase());
          }
        }
      }
    } catch {
      // degrade gracefully
    }
    // Also probe recommendations endpoint for availability (no tag data expected, but confirms personalization backend is reachable)
    try {
      const res2 = await fetch('/api/recommendations/personal', { credentials: 'include' });
      if (res2.ok) {
        await res2.json().catch(() => null);
      }
    } catch {
      // ignore - personalization still degrades to generic cloud
    }
    return set;
  }

  async function fetchAllTags(sort: string) {
    const results = await Promise.allSettled(
      TAG_TYPE_ORDER.map((id) => fetchTagsByType(id, 500, sort))
    );
    const grouped: Record<number, TagItem[]> = {};
    for (let i = 0; i < TAG_TYPE_ORDER.length; i++) {
      const typeId = TAG_TYPE_ORDER[i];
      const result = results[i];
      if (result.status === 'fulfilled' && result.value.err === 0) {
        grouped[typeId] = result.value.tags;
      } else {
        grouped[typeId] = [];
      }
    }
    tagsByType = grouped;
  }

  onMount(async () => {
    loading = true;
    try {
      await auth.init();
    } catch {
      // keep logged-out state
    }
    // Determine personalization before loading tags so caption is ready
    let shouldPersonalize = false;
    try {
      shouldPersonalize = await isPersonalizationEnabled();
    } catch {
      shouldPersonalize = false;
    }
    if (shouldPersonalize) {
      try {
        const names = await fetchPersonalTagNames();
        personalTagNames = names;
        personalized = true;
      } catch {
        personalized = false;
        personalTagNames = new Set();
      }
    } else {
      personalized = false;
      personalTagNames = new Set();
    }
    try {
      const sort = showMode === 'popular' ? 'usage' : 'created';
      await fetchAllTags(sort);
    } catch {
      error = 'Failed to load tags.';
    } finally {
      loading = false;
    }
  });

  function tagUrl(typeId: number, name: string): string {
    const typeName = TAG_TYPE_NAMES[typeId] ?? 'Tag';
    return `/search?include_tags=${typeName}:${encodeURIComponent(name)}`;
  }

  function switchMode(mode: 'popular' | 'random') {
    showMode = mode;
    loading = true;
    error = '';
    const sortParam = mode === 'popular' ? 'usage' : 'created';
    fetchAllTags(sortParam)
      .then(() => {
        loading = false;
      })
      .catch(() => {
        error = 'Failed to load tags.';
        loading = false;
      });
  }

  // For cloud font sizing, we need usage counts across all tags
  const allTags = $derived.by(() => {
    const arr: TagItem[] = [];
    for (const typeId of TAG_TYPE_ORDER) {
      const tags = tagsByType[typeId] ?? [];
      for (const t of tags) arr.push(t);
    }
    return arr;
  });

  const maxUsage = $derived.by(() => {
    const usages = allTags.map((t) => t.usage_count ?? 0);
    return Math.max(...usages, 1);
  });

  // Map a tag's usage count to a cloud1–cloud8 class, with +1 boost for personalized tags (capped at cloud8)
  function cloudClass(usage: number, tagName?: string): string {
    if (maxUsage === 0) return 'cloud1';
    const ratio = usage / maxUsage;
    let level = Math.min(8, Math.max(1, Math.ceil(ratio * 8)));
    if (personalized && tagName && personalTagNames.has(tagName.toLowerCase())) {
      level = Math.min(8, level + 1);
    }
    return `cloud${level}`;
  }

  /** Total tags across all categories. */
  let totalTags = $derived(
    Object.values(tagsByType).reduce((sum, arr) => sum + arr.length, 0)
  );
</script>
<svelte:head>
  <title>Browse Tags — FicHub</title>
  <meta name="description" content="Browse tags across all fandoms on FicHub." />
</svelte:head>


{#if uiMode === 'archive'}
  <!-- AO3-style tag cloud -->
  <div class="tag-cloud-page">
    <h1 class="tag-cloud-heading">Tags</h1>

    <!-- Subnav: Most Popular / Random -->
    <ul class="tag-navigation actions" role="navigation">
      <li>
        <button
          class="tag-nav-link"
          class:active={showMode === 'popular'}
          type="button"
          onclick={() => switchMode('popular')}
        >
          Most Popular
        </button>
      </li>
      <li>
        <button
          class="tag-nav-link"
          class:active={showMode === 'random'}
          type="button"
          onclick={() => switchMode('random')}
        >
          Random
        </button>
      </li>
    </ul>

    {#if loading}
      <p class="tag-cloud-loading">Loading tags…</p>
    {:else if error}
      <p class="tag-cloud-error">{error}</p>
    {:else if totalTags === 0}
      <p class="tag-cloud-empty">No tags found.</p>
    {:else}
      <h2 class="landmark heading">
        {showMode === 'random' ? 'Browse Random Tags' : 'Browse Popular Tags'}
      </h2>
      <p class="tag-cloud-caption">
        {#if personalized}
          Personalized for you — <a href="/settings">disable in Preferences</a>
        {:else}
          Most used tags
        {/if}
      </p>
      {#each TAG_TYPE_ORDER as typeId}
        {@const tags = tagsByType[typeId] ?? []}
        {#if tags.length > 0}
          <section class="tag-type-group">
            <h3 class="tag-type-heading">{TAG_TYPE_LABELS[typeId]}</h3>
            <ul class="tags cloud index group">
              {#each tags as tag (tag.id)}
                <li>
                  <a
                    class="tag-cloud-link {cloudClass(tag.usage_count ?? 0, tag.name)}"
                    href={tagUrl(typeId, tag.name)}
                    title={tag.description ?? tag.name}
                  >
                    {tag.name}
                  </a>
                </li>
              {/each}
            </ul>
          </section>
        {/if}
      {/each}
    {/if}
  </div>
{:else}
  <!-- Modern UI: redirect or show simple list -->
  <div class="tag-index">
    <h1>Tags</h1>
    <p class="tag-index-loading">Switch to <a href="/tags?ui=archive">Archive mode</a> for the full tag browser.</p>
  </div>
{/if}

<style>
  /* ── Archive mode: AO3 tag cloud ── */
  .tag-cloud-page {
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }

  .tag-cloud-heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.6em;
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
    margin: 0 0 0.5em;
    padding-bottom: 0.3em;
    border-bottom: 2px solid var(--archive-link, #990000);
  }

  .tag-navigation.actions {
    display: flex;
    gap: 0.5em;
    list-style: none;
    padding: 0;
    margin: 0 0 1.2em;
  }

  .tag-nav-link {
    display: inline-block;
    padding: 0.3em 0.8em;
    font-size: 0.85em;
    font-weight: 600;
    color: var(--archive-muted, #666666);
    text-decoration: none;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    cursor: pointer;
  }

  .tag-nav-link.active {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }

  .tag-nav-link:hover:not(.active) {
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .tag-cloud-loading,
  .tag-cloud-empty {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    padding: 2em 0;
    text-align: center;
  }

  .tag-cloud-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    padding: 2em 0;
    text-align: center;
  }

  .tag-cloud-caption {
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    margin: 0 0 1em;
  }

  .tag-cloud-caption a {
    color: var(--archive-link, #990000);
    font-style: normal;
    text-decoration: underline;
  }

  .tag-type-group {
    margin: 0 0 1.2em;
  }

  .tag-type-heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
    margin: 0 0 0.4em;
    padding-bottom: 0.2em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }

  /* AO3 cloud layout: flowing list with font-size weighting */
  .tags.cloud.index.group {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.15em 0.4em;
    line-height: 1.6;
  }

  .tags.cloud.index.group li {
    display: inline;
  }

  /* Cloud font-size classes — AO3 standard cloud1–cloud8 */
  .tag-cloud-link {
    display: inline-block;
    margin: 0 0.2em;
    color: var(--archive-link, #990000);
    text-decoration: none;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    padding: 0.15em 0.4em;
    background: var(--archive-bg, #ffffff);
    line-height: 1.4;
  }

  .tag-cloud-link:hover {
    text-decoration: underline;
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .cloud1 { font-size: 0.8em; }
  .cloud2 { font-size: 0.85em; }
  .cloud3 { font-size: 0.9em; }
  .cloud4 { font-size: 0.95em; }
  .cloud5 { font-size: 1.0em; }
  .cloud6 { font-size: 1.1em; }
  .cloud7 { font-size: 1.2em; }
  .cloud8 { font-size: 1.3em; font-weight: 700; }

  .landmark.heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    margin: 0 0 0.5em;
  }

  /* ── Modern mode styles ── */
  .tag-index { max-width: 900px; margin: 0 auto; padding: 1rem; }
  .tag-index-loading { font-size: 0.95em; color: var(--archive-muted, #666666); padding: 2em 0; text-align: center; }

  @media (max-width: 640px) {
    .tag-cloud-page { padding: 0.75rem; }
    .tags.cloud.index.group { line-height: 1.8; }
  }
</style>
