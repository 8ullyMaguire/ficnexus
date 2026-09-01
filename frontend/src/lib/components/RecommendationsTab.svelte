<script lang="ts">
  import { fetchRecommendations, fetchEntityRecommendations, ApiError } from '$lib/api/client';
  import type {
    RecommendationsResponse,
    RecResult,
    EntityKind,
    EntityRec,
  } from '$lib/api/types';
  import { formatWords, stripHtml } from '$lib/util';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  type TabKind = 'works' | EntityKind;
  const TABS: { id: TabKind; label: string }[] = [
    { id: 'works', label: 'Works' },
    { id: 'author', label: 'Authors' },
    { id: 'tag', label: 'Tags' },
    { id: 'fandom', label: 'Fandoms' },
    { id: 'collection', label: 'Collections' },
    { id: 'user', label: 'Users' },
  ];

  let activeTab = $state<TabKind>('works');

  let url = $state('');
  let seed = $state('');
  let loading = $state(false);
  let error = $state('');
  let recs = $state<RecResult[]>([]);
  let entityRecs = $state<EntityRec[]>([]);
  let siteDomain = $state('');

  let seedValue = $state('');
  const inputValue = $derived(activeTab === 'works' ? url : seedValue);
  function onInput(e: Event) {
    const v = (e.target as HTMLInputElement).value;
    if (activeTab === 'works') url = v;
    else seed = v;
  }

  async function handleGetRecs() {
    loading = true;
    error = '';
    recs = [];
    entityRecs = [];
    try {
      if (activeTab === 'works') {
        if (!url.trim()) {
          error = 'Paste the URL of a fic you liked to see similar stories.';
          return;
        }
        const res = await fetchRecommendations(url.trim(), undefined, 20);
        if (res.err !== 0) {
          error = 'Could not load recommendations.';
          return;
        }
        recs = res.recommendations;
        siteDomain = res.site_domain ?? '';
      } else {
        const res = await fetchEntityRecommendations(activeTab, seed.trim() || undefined, 20);
        if (res.err !== 0) {
          error = 'Could not load recommendations.';
          return;
        }
        entityRecs = res.items ?? [];
      }
    } catch (e) {
      error = e instanceof ApiError ? 'Backend error loading recommendations.' : 'Network error.';
    } finally {
      loading = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleGetRecs();
  }

  function switchTab(tab: TabKind) {
    activeTab = tab;
    error = '';
    recs = [];
    entityRecs = [];
  }

  // Sort by combined score (algorithmic + community).
  let sorted = $derived.by(() => {
    return [...recs].sort(
      (a, b) => b.score + b.community_score * 0.1 - (a.score + a.community_score * 0.1),
    );
  });

  function entityHref(kind: EntityKind, name: string): string {
    switch (kind) {
      case 'author':
        return `/authors/${encodeURIComponent(name)}`;
      case 'tag':
      case 'fandom':
        return `/search?include_tags=${kind === 'fandom' ? 'Fandom' : 'Freeform'}:${encodeURIComponent(name)}`;
      case 'collection':
        return `/collections/${encodeURIComponent(name)}`;
      case 'user':
        return `/people/${encodeURIComponent(name)}`;
    }
  }

  const placeholder = $derived(
    activeTab === 'works'
      ? 'URL of a fic you liked…'
      : activeTab === 'author'
        ? 'Author name (optional) to find similar authors…'
        : activeTab === 'tag' || activeTab === 'fandom'
          ? 'Tag name (optional) to find related tags…'
          : activeTab === 'collection'
            ? 'Collection name (optional) to find similar lists…'
            : 'Username (optional) to find similar users…',
  );
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<div class="recs-archive">
  <div class="recs-arc-tabs" role="tablist">
    {#each TABS as tab}
      <button
        type="button"
        role="tab"
        class="recs-arc-tab"
        class:active={activeTab === tab.id}
        onclick={() => switchTab(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="recs-arc-search">
    <input
      class="arc-input"
      type={activeTab === 'works' ? 'url' : 'text'}
      placeholder={placeholder}
      value={inputValue}
      oninput={onInput}
      onkeydown={onKeydown}
      disabled={loading}
      aria-label="Recommendation seed"
    />
    <button class="archive-btn" type="button" onclick={handleGetRecs} disabled={loading}>
      {#if loading}Searching…{:else}Recommend{/if}
    </button>
  </div>

  {#if error}
    <p class="recs-arc-error">{error}</p>
  {/if}

  {#if !loading && activeTab === 'works' && recs.length === 0 && !error}
    <blockquote class="recs-arc-empty"><p>No recommendations yet. Try a fic above!</p></blockquote>
  {/if}

  {#if !loading && activeTab !== 'works' && entityRecs.length === 0 && !error}
    <blockquote class="recs-arc-empty">
      <p>No {activeTab} recommendations yet. Try an optional seed above!</p>
    </blockquote>
  {/if}

  {#if sorted.length > 0}
    <p class="recs-arc-count">{sorted.length} recommendations{siteDomain ? ` from ${siteDomain}` : ''}</p>
    <dl class="archive-dl recs-arc-dl">
      {#each sorted as r (r.url_id)}
        <div class="dl-row">
          <dt>
            <a href={r.download_urls?.epub ?? '#'}>{r.title}</a>
          </dt>
          <dd>
            <span class="recs-arc-byline">by {r.author} · {r.site_domain}</span>
            <span class="recs-arc-meta">
              {formatWords(r.words)} words · {r.chapters} chapters · {r.status}
            </span>
            {#if r.summary}
              <span class="recs-arc-desc">{stripHtml(r.summary).slice(0, 200)}</span>
            {/if}
            <span class="recs-arc-footer">
              {#if r.community_score !== 0}
                <span class="recs-arc-score">
                  {r.community_score > 0 ? '▲' : '▼'} {Math.abs(r.community_score)} community
                </span>
              {/if}
              {#if r.download_urls?.epub}
                <a class="archive-btn" href={r.download_urls.epub} download>EPUB</a>
              {/if}
            </span>
          </dd>
        </div>
      {/each}
    </dl>
  {/if}

  {#if entityRecs.length > 0}
    <p class="recs-arc-count">{entityRecs.length} {activeTab} recommendations</p>
    <dl class="archive-dl recs-arc-dl">
      {#each entityRecs as r (r.id + r.name)}
        <div class="dl-row">
          <dt>
            <a href={entityHref(activeTab as EntityKind, r.name)}>{r.name}</a>
          </dt>
          <dd>
            <span class="recs-arc-byline">
              score {r.score.toFixed(3)}{r.reason ? ` · ${r.reason}` : ''}
            </span>
          </dd>
        </div>
      {/each}
    </dl>
  {/if}
</div>
{:else}
<div class="recs-tab">
  <p class="muted intro">
    {activeTab === 'works'
      ? 'Paste a fic you enjoyed, and FicNexus will suggest similar stories based on what other readers bookmarked together.'
      : 'Explore similar authors, tags, fandoms, collections, and users — not just fics.'}
  </p>

  <div class="tab-row">
    {#each TABS as tab}
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === tab.id}
        onclick={() => switchTab(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="search-row">
    <input
      type={activeTab === 'works' ? 'url' : 'text'}
      placeholder={placeholder}
      value={inputValue}
      oninput={onInput}
      onkeydown={onKeydown}
      disabled={loading}
      aria-label="Recommendation seed"
    />
    <button class="btn" onclick={handleGetRecs} disabled={loading}>
      {#if loading}<span class="spinner"></span> Searching…{:else}Recommend{/if}
    </button>
  </div>

  {#if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {/if}

  {#if !loading && activeTab === 'works' && recs.length === 0 && !error}
    <div class="card empty">
      <p class="muted">No recommendations yet. Try a fic above!</p>
    </div>
  {/if}

  {#if !loading && activeTab !== 'works' && entityRecs.length === 0 && !error}
    <div class="card empty">
      <p class="muted">No {activeTab} recommendations yet. Try an optional seed above!</p>
    </div>
  {/if}

  {#if sorted.length > 0}
    <p class="muted count">{sorted.length} recommendations{siteDomain ? ` from ${siteDomain}` : ''}</p>
    <div class="rec-list">
      {#each sorted as r (r.url_id)}
        <div class="card rec-item">
          <h3>
            <a href={r.download_urls?.epub ?? '#'}>{r.title}</a>
          </h3>
          <p class="muted">by {r.author} · <span class="tag">{r.site_domain}</span></p>
          <p class="meta-line">
            {formatWords(r.words)} words · {r.chapters} chapters · {r.status}
          </p>
          {#if r.summary}
            <p class="desc">{stripHtml(r.summary).slice(0, 200)}</p>
          {/if}
          <div class="rec-footer">
            {#if r.community_score !== 0}
              <span class="badge" class:positive={r.community_score > 0}>
                {r.community_score > 0 ? '▲' : '▼'} {Math.abs(r.community_score)} community
              </span>
            {/if}
            {#if r.download_urls?.epub}
              <a class="btn btn-secondary sm" href={r.download_urls.epub} download>EPUB</a>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if entityRecs.length > 0}
    <p class="muted count">{entityRecs.length} {activeTab} recommendations</p>
    <div class="rec-list">
      {#each entityRecs as r (r.id + r.name)}
        <div class="card rec-item">
          <h3>
            <a href={entityHref(activeTab as EntityKind, r.name)}>{r.name}</a>
          </h3>
          <p class="muted">
            score {r.score.toFixed(3)}{r.reason ? ` · ${r.reason}` : ''}
          </p>
        </div>
      {/each}
    </div>
  {/if}
</div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .recs-archive {
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .recs-arc-search {
    display: flex;
    gap: 0.6rem;
    margin-bottom: 1rem;
  }
  .recs-arc-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-bottom: 0.8rem;
  }
  .recs-arc-tab {
    padding: 0.25rem 0.7rem;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.88em;
    color: var(--archive-muted, #666666);
    background: var(--archive-bg, #ffffff);
    border: 1px solid var(--archive-border, #dddddd);
    cursor: pointer;
  }
  .recs-arc-tab.active {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .arc-input {
    flex: 1;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    padding: 0.35rem 0.6rem;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-btn {
    padding: 0.3rem 0.85rem;
    font-size: 0.88em;
    font-family: inherit;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
    text-decoration: none;
    display: inline-block;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
    text-decoration: none;
  }
  .archive-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .recs-arc-error {
    color: var(--color-error, #cc0000);
    font-weight: 600;
    margin-bottom: 0.8rem;
  }
  .recs-arc-empty {
    border-left: 3px solid var(--archive-border, #dddddd);
    margin: 1rem 0;
    padding: 0.6rem 1rem;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .recs-arc-empty p { margin: 0; }
  .recs-arc-count {
    color: var(--archive-muted, #666666);
    font-size: 0.9em;
    margin-bottom: 0.6rem;
  }
  .archive-dl.recs-arc-dl { margin: 0; }
  .recs-arc-dl .dl-row {
    padding: 0.6em 0.25em;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .recs-arc-dl .dl-row:last-child { border-bottom: none; }
  .recs-arc-dl dt a {
    font-weight: 700;
    font-size: 1.02em;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .recs-arc-dl dt a:hover { text-decoration: underline; }
  .recs-arc-dl dd {
    margin: 0.15em 0 0;
    display: flex;
    flex-direction: column;
    gap: 0.15em;
    font-size: 0.9em;
  }
  .recs-arc-byline,
  .recs-arc-meta {
    color: var(--archive-muted, #666666);
  }
  .recs-arc-desc {
    font-style: italic;
    color: var(--archive-text, #2a2a2a);
  }
  .recs-arc-footer {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    flex-wrap: wrap;
    margin-top: 0.25em;
  }
  .recs-arc-score {
    color: var(--archive-muted, #666666);
    font-size: 0.9em;
  }

  @media (max-width: 600px) {
    .recs-arc-search {
      flex-direction: column;
    }
  }

  /* ── Modern mode (unchanged) ── */
  .intro {
    margin-bottom: 1rem;
  }
  .search-row {
    display: flex;
    gap: 0.6rem;
    margin-bottom: 1rem;
  }
  .tab-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-bottom: 0.8rem;
  }
  .tab-btn {
    padding: 0.3rem 0.7rem;
    font-size: 0.85rem;
    color: var(--color-muted);
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    cursor: pointer;
  }
  .tab-btn.active {
    color: #fff;
    background: var(--color-primary);
    border-color: var(--color-primary);
  }
  .search-row input {
    flex: 1;
  }
  .error-card {
    border-color: var(--color-error);
  }
  .count {
    font-size: 0.9rem;
  }
  .rec-item h3 {
    margin: 0 0 0.3rem;
    font-size: 1.1rem;
  }
  .rec-item h3 a {
    color: var(--color-text);
  }
  .meta-line {
    margin: 0.3rem 0;
  }
  .desc {
    color: var(--color-muted);
    font-size: 0.88rem;
  }
  .rec-footer {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    margin-top: 0.8rem;
  }
  .badge {
    font-size: 0.82rem;
    color: var(--color-muted);
  }
  .badge.positive {
    color: var(--color-success);
  }
  .btn.sm {
    padding: 0.35rem 0.7rem;
    font-size: 0.82rem;
  }
  .rec-list {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  @media (max-width: 600px) {
    .search-row {
      flex-direction: column;
    }
  }
</style>
